mod connection;
mod value;

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
};

use connection::message;
use tungstenite::connect;
pub use value::{Value, ValueDict};

type ToolFn = fn(ValueDict, message::Sender) -> Result<ValueDict, String>;

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
    let (mut socket, response) = connect(addr).unwrap();

    // TODO: all of the below will change

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    socket
        .send(Message::Text("Hello WebSocket".into()))
        .unwrap();

    let msg = socket.read().expect("Error reading message");
    println!("Received: {msg}");

    socket.close(None).unwrap();
}

#[derive(Clone)]
struct ToolState {
    tool: ToolFn,
    index_html: Option<&'static str>,
}

async fn tool_handler(mut socket: WebSocket, tool: ToolFn) {
    // TODO: better error handling - results are unwrapped!
    // TODO: tool thread should have a timeout!

    // First, read the input from the socket
    let input = connection::recv_values(&mut socket).await.unwrap();
    // Channel for sending messages to the client and abort signal back
    let (msg_tx, mut msg_rx) = message::channel();
    // Run the tool, give it the input and the channel to send messages
    let result = tokio::task::spawn_blocking(move || tool(input, msg_tx));

    // Now the tool is running. Before collecting the result, we run two loops:

    // TODO: the socket is used twice, in the abort loop and in the message sender.
    // Two &mut are not allowed, we need to build it into one loop somehow!

    // In the background, we listen for abort signals:
    let abort_listener = tokio::spawn(async move {
        while let Some(msg) = socket.recv().await {
            match msg {
                Ok(msg) => match msg {
                    axum::extract::ws::Message::Close(_) => {
                        // The client can request an abort by closing the connection
                        msg_rx.abort(message::AbortReason::RequestedByClient);
                    }
                    _ => (), // Ignore other messages
                },
                Err(_) => {
                    // Client disconnected or connection failed - abort tool
                    msg_rx.abort(message::AbortReason::WebSocketError);
                }
            }
        }
    });

    // In parallel, we forward messages from the tool to the client
    while let Some(msg) = msg_rx.recv().await {
        socket.send(axum::extract::ws::Message::Text(msg.into())).await.unwrap();
    }

    // Tool closed connection - we can stop the abort forward loop now
    abort_listener.abort();

    // Wait for tool completion and collect result - panics if tool panicked
    let result = result.await.unwrap();
    // Return the output to the client
    connection::send_result(socket, result).await.unwrap();
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
