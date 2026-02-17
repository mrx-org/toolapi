use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Value, connection::websocket::WsMessageType};

/// Sent over the server <-> tool channel to communicate an abort
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AbortReason {
    #[error("requested by client")]
    RequestedByClient,
    #[cfg(feature = "server")]
    #[error("tokio channel error: {0}")]
    ChannelError(String),
    #[error("connection closed")]
    ConnectionClosed,
}

/// Returned when extracting a value fails (wrong type, key not found etc)
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
    #[error("tool finished but didn't shut down properly: {err}")]
    CloseFailed {
        result: Value,
        err: ConnectionError,
    },
    #[error("tool didn't send a result")]
    ProtocolError,
    #[error("client requested abort in on_message")]
    OnMessageAbort,
    #[error("tool returned an error: {0}")]
    ToolReturnedError(#[from] ToolError),
}

/// Returned by the tool in the final result() call as reason if no value was computed.
/// It is seriazable since it is the only error that ist actually sent over the WebSocket connection.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ToolError {
    #[error("tool was requested to abort: {0}")]
    Abort(#[from] AbortReason),
    #[error("custom tool error: {0}")]
    Custom(String),
}
