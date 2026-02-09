use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::{AbortReason, ConnectionError, ToolFn};

#[derive(Clone)]
pub struct ToolState {
    pub tool: ToolFn,
    pub index_html: Option<&'static str>,
}

pub async fn index_handler(State(state): State<ToolState>) -> Response {
    match state.index_html {
        Some(html) => Html(html).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn socket_handler(ws: WebSocketUpgrade, State(state): State<ToolState>) -> Response {
    // print errors to stdout (logged by fly.io, might need explicit logging for other platforms)
    ws.max_message_size(256 * 1024 * 1024)
        .max_frame_size(256 * 1024 * 1024)
        .on_upgrade(async move |socket| {
            if let Err(err) = tool_handler(socket, state.tool).await {
                // TODO: we should send the error to the tool as well!
                eprintln!("{err}");
            }
        })
}

async fn tool_handler(socket: WebSocket, tool: ToolFn) -> Result<(), ConnectionError> {
    // TODO: would it help the code to split the socket into read and write?
    // https://docs.rs/axum/latest/axum/extract/ws/index.html#read-and-write-concurrently

    // Wrap the socket in a helper struct
    let mut ws_server = crate::connection::websocket::WsChannelServer::new(socket);
    // First, read the input from the socket
    let input = ws_server
        .read_values()
        .await?
        .ok_or(ConnectionError::ConnectionClosed)?;
    // Channel for sending messages to the client and abort signal back
    let (mut msg_tx, mut msg_rx) = crate::connection::channel::connect();
    // Run the tool, give it the input and the channel to send messages
    let mut send_msg = move |msg| {
        println!(" > {msg}");
        msg_tx.send(msg)
    };
    let result = tokio::task::spawn_blocking(move || tool(input, &mut send_msg));

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
