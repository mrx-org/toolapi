//! This module helps sending data between the client and the server via WebSocket,
//! as well as between the async server and the sync tool via channels.
pub mod message;

use crate::ValueDict;
use axum::extract::ws::{Message, WebSocket};

// TODO: use proper error struct (and maybe thiserror) instead of strings.
// Except for the Result<ValueDict, String> - this is a string the tool returned
// as error. It's okay here so the tool can send any error message.

pub async fn send_result(mut socket: WebSocket, result: Result<ValueDict, String>) -> Result<(), String> {
    let serialized = serde_json::to_string(&result)
        .map_err(|err| format!("Failed to serialize ValueDict: {err}"))?;
    socket
        .send(Message::Text(serialized.into()))
        .await
        .map_err(|err| format!("Failed to send ValueDict: {err}"))?;

    Ok(())
}

pub async fn recv_values(socket: &mut WebSocket) -> Result<ValueDict, String> {
    match socket.recv().await {
        Some(Ok(msg)) => {
            if let axum::extract::ws::Message::Text(msg) = msg {
                match serde_json::from_str(&msg) {
                    Ok(x) => Ok(x),
                    Err(err) => Err(format!("Failed to parse input: {err}")),
                }
            } else {
                Err(format!("Expected a WS Text message, got {msg:?} instead"))
            }
        }
        Some(Err(err)) => Err(format!("Failed to read from WebSocket: {err}")),
        None => Err(format!(
            "tool_handler: WebSocket stream was closed immediately"
        )),
    }
}
