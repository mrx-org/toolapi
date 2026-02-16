use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::connection::websocket::WsMessageType;

/// Sent over the server <-> tool channel to communicate an abort
#[derive(Error, Debug)]
pub enum AbortReason {
    #[error("requested by client")]
    RequestedByClient,
    #[cfg(feature = "server")]
    #[error("channel error: {0}")]
    ChannelError(#[from] tokio::sync::mpsc::error::SendError<String>),
    #[error("connection closed")]
    ConnectionClosed,
}

/// Exclusively used by the Values struct when looking up a value
#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("dynamic type contained a `{from}`, tried to extract a `{into}`")]
    TypeMismatch {
        from: &'static str,
        into: &'static str,
    },
    #[error("tried to index further into atomic type")]
    TooMuchNesting,
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("key not found")]
    KeyNotFound,
    #[error("tried to index a Dict with an integer")]
    IndexForDict,
    #[error("tried to index a List with a string")]
    KeyForList,
}

/// Exclusively used by the Values struct when looking up a value
#[derive(Error, Debug)]
pub enum LookupError {
    #[error("key {0} does not exist")]
    KeyError(String),
    #[error("wrong type: {0}")]
    ConversionError(#[from] ExtractionError),
}

/// Created during Message (de)serialization, part of ConnectionError
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("serialization failed: {0}")]
    SerializationError(rmp_serde::encode::Error),
    #[error("deserialization failed: {0}")]
    DeserializationError(rmp_serde::decode::Error),
    #[error("compression failed: {0}")]
    CompressionError(std::io::Error),
    #[error("decompression failed: {0}")]
    DecompressionError(std::io::Error),
    #[error("wrong message type (expected {expected:?}, found {found:?})")]
    WrongMessageType {
        expected: WsMessageType,
        found: WsMessageType,
    },
}

/// Returned by the WebSocket impls when trying to connect, send, recv
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("parsing a WebSocket message failed: {0}")]
    ParseError(#[from] ParseError),
    #[error("connection closed")]
    ConnectionClosed,
    #[cfg(feature = "server")]
    #[error("the tool crashed, err='{0}'")]
    ToolPanic(#[from] tokio::task::JoinError),
}

/// Returned by the call() function running on the client
#[derive(Error, Debug)]
pub enum ToolCallError {
    #[error("connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    /// Either the tool is not sending a result or there is a bug and we were
    /// not reading all messages before the result (we are still receiving messages)
    #[error("tool didn't send a result")]
    ProtocolError,
    #[error("tool returned an error message: {0}")]
    ToolError(String),
    #[error("client requested abort in on_message")]
    OnMessageAbort,
    #[error("tool returned an error: {0}")]
    ToolReturnedError(#[from] ToolError),
}

/// Returned by the tool in the final result() call as reason if no value was computed.
/// It is seriazable since it is the only error that ist actually sent over the WebSocket connection.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ToolError {
    /// This does not contain the abort reason because not all of them are seriazable.
    /// The server logs should have the abort with the reason logged.
    #[error("tool was requested to abort")]
    Abort,
    #[error("custom tool error: {0}")]
    Custom(String),
}
