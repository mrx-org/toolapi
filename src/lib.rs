mod connection;
mod error;
mod value;

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
};
pub use connection::channel::Sender;
pub use error::*;
pub use value::{Value, ValueDict};

type ToolFn = fn(ValueDict, Sender) -> Result<ValueDict, String>;

#[tokio::main]
pub async fn run_server(
    tool: ToolFn,
    index_html: Option<&'static str>,
) -> Result<(), std::io::Error> {
    let state = ToolState { tool, index_html };

    let routes = Router::new()
        .route("/", get(index_handler))
        .route("/tool", any(socket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, routes).await
}

pub fn call(
    addr: &str,
    input: ValueDict,
    on_message: fn(String) -> bool,
) -> Result<ValueDict, ToolCallError> {
    // Create a connection to the server, send inputs, run callback message loop
    let mut ws_client = connection::websocket::WsChannelSync::connect(addr)?;

    ws_client.send_values(input)?;
    while let Some(msg) = ws_client.read_message()? {
        if !on_message(msg) {
            // abort was requested by client callback
            ws_client.send_abort()?;
            ws_client.close()?;
            return Err(ToolCallError::OnMessageAbort);
        }
    }

    // Read result, handle shutdown, return result
    let result = ws_client
        .read_result()?
        .ok_or(ToolCallError::ProtocolError)?;
    ws_client.close()?; // TODO: we have the result, shouldn't we return it?
    result.map_err(ToolCallError::ToolError)
}

#[derive(Clone)]
struct ToolState {
    tool: ToolFn,
    index_html: Option<&'static str>,
}

async fn tool_handler(socket: WebSocket, tool: ToolFn) -> Result<(), ConnectionError> {
    // Wrap the socket in a helper struct
    let mut ws_server = connection::websocket::WsChannelAsync::new(socket);
    // First, read the input from the socket
    let input = ws_server
        .read_values()
        .await?
        .ok_or(ConnectionError::ConnectionClosed)?;
    // Channel for sending messages to the client and abort signal back
    let (msg_tx, mut msg_rx) = connection::channel::connect();
    // Run the tool, give it the input and the channel to send messages
    let result = tokio::task::spawn_blocking(move || tool(input, msg_tx));

    // Run a loop which forwards tool messages to the client or abort messages to the tool
    loop {
        // WARN: axum does not document this - we assume WebSocket.send() and .recv() is cancel safe
        // TODO: tool thread should have a timeout!
        tokio::select! {
            tool_msg = msg_rx.recv() => {
                match tool_msg {
                    Some(msg) => ws_server.send_message(msg).await?,
                    None => break,  // msg_rx was closed: tool no longer running
                }
            },
            aborted = ws_server.read_abort() => {
                if aborted?.is_some() {
                    msg_rx.abort(AbortReason::RequestedByClient);
                    break;
                }
            }
        }
    }

    // Wait for tool completion and collect result - panics if tool panicked
    let result = result.await?;
    // Return the output to the client
    ws_server.send_result(result).await
}

async fn socket_handler(ws: WebSocketUpgrade, State(state): State<ToolState>) -> Response {
    // print errors to stdout (logged by fly.io, might need explicit logging for other platforms)
    ws.on_upgrade(async move |socket| {
        if let Err(err) = tool_handler(socket, state.tool).await {
            // TODO: we should send the error to the tool as well!
            eprintln!("{err}");
        }
    })
}

async fn index_handler(State(state): State<ToolState>) -> Response {
    match state.index_html {
        Some(html) => Html(html).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
