use thiserror::Error;

use crate::connection::websocket::WsMessageType;

#[derive(Error, Debug)]
pub enum AbortReason {
    #[error("requested by client")]
    RequestedByClient,
    #[error("connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("connection closed")]
    ConnectionClosed
    // WebSocketError,
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionError {
    pub from: &'static str,
    pub into: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum LookupError {
    KeyError,
    ConversionError(ConversionError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("serialization failed: {0}")]
    SerializationError(serde_json::Error),
    #[error("deserialization failed: {0}")]
    DeserializationError(serde_json::Error),
    #[error("wrong message type (expected {expected:?}, found {found:?})")]
    WrongMessageType {
        expected: WsMessageType,
        found: WsMessageType,
    },
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("WebSocket error (tungstenite): {0}")]
    TungsteniteError(tungstenite::Error),
    #[error("WebSocket error (axum): {0}")]
    AxumError(axum::Error),
    #[error("Channel error (tokio): {0}")]
    TokioError(tokio::sync::mpsc::error::SendError<String>),
    #[error("parsing a WebSocket message failed: {0}")]
    ParseError(ParseError),
    #[error("connection closed")]
    ConnectionClosed,
}
