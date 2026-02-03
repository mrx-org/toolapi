//! Common structures shared by client and server / sync and async impls.
//! This is the heart of the communication - both sides have to agree on this!

use crate::{ParseError, ToolError, ValueDict};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Values(ValueDict),
    Result(Result<ValueDict, ToolError>),
    Message(String),
    Abort,
}

type WsMessageAxum = axum::extract::ws::Message;
type WsMessageTung = tungstenite::Message;
/// Used for error messages only on message type mismatch
#[derive(Debug)]
pub enum WsMessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

impl From<WsMessageAxum> for WsMessageType {
    fn from(value: WsMessageAxum) -> Self {
        match value {
            WsMessageAxum::Text(_) => Self::Text,
            WsMessageAxum::Binary(_) => Self::Binary,
            WsMessageAxum::Ping(_) => Self::Ping,
            WsMessageAxum::Pong(_) => Self::Pong,
            WsMessageAxum::Close(_) => Self::Close,
        }
    }
}

impl From<WsMessageTung> for WsMessageType {
    fn from(value: WsMessageTung) -> Self {
        match value {
            WsMessageTung::Text(_) => Self::Text,
            WsMessageTung::Binary(_) => Self::Binary,
            WsMessageTung::Ping(_) => Self::Ping,
            WsMessageTung::Pong(_) => Self::Pong,
            WsMessageTung::Close(_) => Self::Close,
            WsMessageTung::Frame(_) => unreachable!("Raw frame not encountered in normal use"),
        }
    }
}

impl TryFrom<WsMessageAxum> for Message {
    type Error = ParseError;

    fn try_from(value: WsMessageAxum) -> Result<Self, Self::Error> {
        match value {
            WsMessageAxum::Binary(raw) => {
                let decompressed =
                    zstd::decode_all(raw.as_ref()).map_err(ParseError::DecompressionError)?;
                Ok(rmp_serde::from_slice(&decompressed)
                    .map_err(ParseError::DeserializationError)?)
            }
            msg => Err(ParseError::WrongMessageType {
                expected: WsMessageType::Binary,
                found: msg.into(),
            }),
        }
    }
}

impl TryFrom<WsMessageTung> for Message {
    type Error = ParseError;

    fn try_from(value: WsMessageTung) -> Result<Self, Self::Error> {
        match value {
            WsMessageTung::Binary(raw) => {
                let decompressed =
                    zstd::decode_all(raw.as_ref()).map_err(ParseError::DecompressionError)?;
                Ok(rmp_serde::from_slice(&decompressed)
                    .map_err(ParseError::DeserializationError)?)
            }
            msg => Err(ParseError::WrongMessageType {
                expected: WsMessageType::Binary,
                found: msg.into(),
            }),
        }
    }
}

/// Compression level for zstd (0 = default, typically 3)
const ZSTD_COMPRESSION_LEVEL: i32 = 0;

impl TryFrom<Message> for WsMessageAxum {
    type Error = ParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let raw = rmp_serde::to_vec(&value).map_err(ParseError::SerializationError)?;
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_COMPRESSION_LEVEL)
            .map_err(ParseError::CompressionError)?;
        Ok(WsMessageAxum::Binary(compressed.into()))
    }
}

impl TryFrom<Message> for WsMessageTung {
    type Error = ParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let raw = rmp_serde::to_vec(&value).map_err(ParseError::SerializationError)?;
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_COMPRESSION_LEVEL)
            .map_err(ParseError::CompressionError)?;
        Ok(WsMessageTung::Binary(compressed.into()))
    }
}
