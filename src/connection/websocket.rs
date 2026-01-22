use crate::{ValueDict, connection::message::AbortReason};
use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use tungstenite::{client::IntoClientRequest, connect, stream::MaybeTlsStream};

// TODO: better error handling: use struct instead of string
// But note that the tool can send arbitrary errors as string which we should return from recv_result

// TODO: split into two structs for server and client since one uses axum WebSockets and the other tungstenite
// (even though the former uses the latter, it seems like we cannot easily convert them)

/// Connection from client to server over a WebSocket. All calls are blocking.
pub struct WsClient(tungstenite::WebSocket<MaybeTlsStream<TcpStream>>);

impl WsClient {
    pub fn connect<Req: IntoClientRequest>(request: Req) -> Result<Self, String> {
        // We ignore the response (for now?)
        let (socket, _) = connect(request).map_err(|err| format!("Failed to connect: {err}"))?;

        Ok(Self(socket))
    }

    pub fn send_values(&mut self, values: ValueDict) -> Result<(), String> {
        self.0
            .send(WsMessage::Values(values).to_tungstenite()?)
            .map_err(|err| format!("Failed to send ValueDict: {err}"))
    }

    /// If this returns None, the tool is done
    pub fn recv_message(&mut self) -> Option<String> {
        // TODO: return result to signal unrecoverable errors
        match self.0.read() {
            Ok(msg) => match WsMessage::from_tungstenite(msg) {
                Ok(WsMessage::Message(msg)) => Some(msg),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn recv_result(&mut self) -> Result<ValueDict, String> {
        match self.0.read() {
            Ok(msg) => match WsMessage::from_tungstenite(msg)? {
                WsMessage::Values(values) => Ok(values),
                msg => Err(format!(
                    "Expected WsMessage::Values, got {}",
                    msg.typename()
                )),
            },
            Err(err) => Err(format!("Connection error: {err}")),
        }
    }
}

impl Drop for WsClient {
    fn drop(&mut self) {
        self.0.close(None).unwrap();
    }
}

/// Connection from server to client over a WebSocket
/// # Cancel safety
/// We assume all functions are cancel safe:
/// WebSocket does not document this but the internet says so
pub struct WsServer(pub axum::extract::ws::WebSocket);

impl WsServer {
    pub async fn send_result(&mut self, result: Result<ValueDict, String>) -> Result<(), String> {
        self.0
            .send(WsMessage::Result(result).to_axum()?)
            .await
            .map_err(|err| format!("Failed to send ValueDict: {err}"))
    }

    pub async fn send_message(&mut self, msg: String) -> Result<(), String> {
        self.0
            .send(WsMessage::Message(msg).to_axum()?)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn recv_values(&mut self) -> Result<ValueDict, String> {
        match self.0.recv().await {
            Some(Ok(msg)) => match WsMessage::from_axum(msg)? {
                WsMessage::Values(values) => Ok(values),
                msg => Err(format!(
                    "Expected WsMessage::Values, got {}",
                    msg.typename()
                )),
            },
            Some(Err(err)) => Err(format!("Connection error: {err}")),
            None => Err("Connection closed".to_owned()),
        }
    }

    pub async fn is_aborted(&mut self) -> Option<AbortReason> {
        match self.0.recv().await {
            // Client connection was closed on purpose, send this to tool
            Some(Ok(Message::Close(_))) | None => Some(AbortReason::RequestedByClient),
            // Connection failed, send this to tool
            Some(Err(_)) => Some(AbortReason::WebSocketError),
            // Ignore all other messages sent from the client
            Some(_) => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum WsMessage {
    Values(ValueDict),
    Result(Result<ValueDict, String>),
    Message(String),
}

impl WsMessage {
    fn typename(&self) -> &'static str {
        match self {
            WsMessage::Values(_) => "WsMessage::Values",
            WsMessage::Result(_) => "WsMessage::Result",
            WsMessage::Message(_) => "WsMessage::Message",
        }
    }

    fn to_axum(&self) -> Result<Message, String> {
        let string =
            serde_json::to_string(&self).map_err(|err| format!("Failed to serialize: {err}"))?;
        Ok(Message::Text(string.into()))
    }

    fn from_axum(serialized: Message) -> Result<Self, String> {
        match serialized {
            Message::Text(msg) => Ok(serde_json::from_str(&msg)
                .map_err(|err| format!("Failed to deserialize: {err}"))?),
            _ => Err(format!(
                "Unexpected message type: expected Text, got {serialized:?}"
            )),
        }
    }

    fn to_tungstenite(&self) -> Result<tungstenite::Message, String> {
        let string =
            serde_json::to_string(&self).map_err(|err| format!("Failed to serialize: {err}"))?;
        Ok(tungstenite::Message::Text(string.into()))
    }

    fn from_tungstenite(serialized: tungstenite::Message) -> Result<Self, String> {
        match serialized {
            tungstenite::Message::Text(msg) => Ok(serde_json::from_str(&msg)
                .map_err(|err| format!("Failed to deserialize: {err}"))?),
            _ => Err(format!(
                "Unexpected message type: expected Text, got {serialized:?}"
            )),
        }
    }
}
