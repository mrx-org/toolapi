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
pub async fn run_server(tool: ToolFn, index_html: Option<&'static str>) {
    let state = ToolState { tool, index_html };

    let routes = Router::new()
        .route("/", get(index_handler))
        .route("/tool", any(socket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}

pub fn call(
    addr: &str,
    input: ValueDict,
    on_message: fn(String) -> bool,
) -> Result<ValueDict, String> {
    // Create a connection to the server
    let mut ws_client = connection::websocket::WsChannelSync::connect(addr).unwrap();
    // Send the input to the server
    ws_client.send_values(input).unwrap();
    // Recieve tool messages and abort on request
    while let Some(msg) = ws_client.read_message().unwrap() {
        if !on_message(msg) {
            ws_client.send_abort().unwrap();
            ws_client.close().unwrap();
            return Err("Client aborted the operation".to_owned());
        }
    }
    // If not aborted, wait for result and return
    let result = ws_client.read_result().unwrap().unwrap();
    ws_client.close().unwrap();
    result
}

#[derive(Clone)]
struct ToolState {
    tool: ToolFn,
    index_html: Option<&'static str>,
}

async fn tool_handler(socket: WebSocket, tool: ToolFn) {
    // TODO: better error handling - results are unwrapped!
    // TODO: tool thread should have a timeout!
    // TODO: We could send input and output over https and use the websocket only for messages and aborts!

    // Wrap the socket in a helper struct
    let mut ws_server = connection::websocket::WsChannelAsync::new(socket);
    // First, read the input from the socket
    let input = ws_server.read_values().await.unwrap().unwrap();
    // Channel for sending messages to the client and abort signal back
    let (msg_tx, mut msg_rx) = connection::channel::connect();
    // Run the tool, give it the input and the channel to send messages
    let result = tokio::task::spawn_blocking(move || tool(input, msg_tx));

    // Run a loop which forwards tool messages to the client or abort messages to the tool
    loop {
        // WARN: axum does not document this - we assume WebSocket.send() and .recv() is cancel safe
        tokio::select! {
            tool_msg = msg_rx.recv() => {
                match tool_msg {
                    // TODO: currently panicks if WebSocket connection failed - instead send abort to tool
                    Some(msg) => ws_server.send_message(msg).await.unwrap(),
                    // msg_rx was closed: tool no longer running (most likely finished)
                    None => break,
                }
            },
            aborted = ws_server.read_abort() => {
                if aborted.unwrap().is_some() {
                    // TODO: handle abort reasons with new web socket impls!
                    msg_rx.abort(AbortReason::RequestedByClient);
                    break;
                }
            }
        }
    }

    // Wait for tool completion and collect result - panics if tool panicked
    let result = result.await.unwrap();
    // Return the output to the client
    ws_server.send_result(result).await.unwrap();
}

async fn socket_handler(ws: WebSocketUpgrade, State(state): State<ToolState>) -> Response {
    ws.on_upgrade(move |socket| tool_handler(socket, state.tool))
}

async fn index_handler(State(state): State<ToolState>) -> Response {
    match state.index_html {
        Some(html) => Html(html).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
