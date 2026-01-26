use thiserror::Error;

use crate::connection::websocket::WsMessageType;

#[derive(Error, Debug)]
pub enum AbortReason {
    #[error("requested by client")]
    RequestedByClient,
    #[error("channel error: {0}")]
    ChannelError(#[from] tokio::sync::mpsc::error::SendError<String>),
    #[error("connection closed")]
    ConnectionClosed
}

#[derive(Error, Debug)]
#[error("dynamic type contained a `{from}`, tried to extract a `{into}`")]
pub struct TypeExtractionError {
    pub from: &'static str,
    pub into: &'static str,
}

#[derive(Error, Debug)]
pub enum LookupError {
    #[error("key {0} does not exist")]
    KeyError(String),
    #[error("wrong type: {0}")]
    ConversionError(TypeExtractionError),
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
    #[error("parsing a WebSocket message failed: {0}")]
    ParseError(ParseError),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("the tool crashed, err='{0}'")]
    ToolPanic(String)
}
