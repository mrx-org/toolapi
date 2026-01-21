mod channel;
mod value;

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
};

pub use channel::Channel;
use tungstenite::connect;
pub use value::{Value, ValueDict};

type ToolFn = fn(ValueDict, Channel) -> Result<ValueDict, String>;

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
    // First, read the input from the socket
    let input: ValueDict = match socket.recv().await {
        Some(Ok(msg)) => {
            if let axum::extract::ws::Message::Text(msg) = msg {
                match serde_json::from_str(&msg) {
                    Ok(x) => x,
                    Err(err) => {
                        println!("Failed to parse input: {err}");
                        return;
                    }
                }
            } else {
                println!("Expected a WS Text message, got {msg:?} instead");
                return;
            }
        }
        Some(Err(err)) => todo!(),
        None => {
            println!("tool_handler: WebSocket stream was closed immediately");
            return;
        }
    };

    // Channel for sending the MRX input values to the tool/server
    let (input_tx, input_rx) = tokio::sync::oneshot::channel();
    // Channel for sending messages to the client
    let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(1024);
    // Channel for sending an abort message from to the tool/server
    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel();
    // Channel to return the MRX output values to the client
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    // Pass input directly into channel
    input_tx.send(input).unwrap(); // cannot fail; reciever still alive
    // TODO: rename - create message channel
    let channel = Channel { msg_tx, abort_rx };
    // Run the tool, pass all channels in with the struct above
    let result = tokio::task::spawn_blocking(|| tool(input, channel));

    // TODO: the blocking tasks sends messages over a channel, forward over socket
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }

    // No more messages, wait for tool to finish and return its data
    // This panicks if the tool did
    match result.await.unwrap() {
        Ok(output) => todo!("Send output over channel"),
        Err(err) => todo!("Send error over channel"),
    }
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
