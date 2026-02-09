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

#[cfg(feature = "server")]
type WsMessageAxum = axum::extract::ws::Message;
#[cfg(feature = "client")]
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

#[cfg(feature = "server")]
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

#[cfg(feature = "client")]
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

fn decompress(raw: &[u8]) -> Result<Vec<u8>, ParseError> {
    use ruzstd::io::Read;
    let mut decoder = ruzstd::decoding::StreamingDecoder::new(raw)
        .map_err(|e| ParseError::DecompressionError(std::io::Error::other(e)))?;
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(ParseError::DecompressionError)?;
    Ok(decompressed)
}

fn compress(raw: &[u8]) -> Result<Vec<u8>, ParseError> {
    Ok(ruzstd::encoding::compress_to_vec(
        raw,
        ruzstd::encoding::CompressionLevel::Default,
    ))
}

#[cfg(feature = "server")]
impl TryFrom<WsMessageAxum> for Message {
    type Error = ParseError;

    fn try_from(value: WsMessageAxum) -> Result<Self, Self::Error> {
        match value {
            WsMessageAxum::Binary(raw) => {
                let decompressed = decompress(raw.as_ref())?;
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

#[cfg(feature = "client")]
impl TryFrom<WsMessageTung> for Message {
    type Error = ParseError;

    fn try_from(value: WsMessageTung) -> Result<Self, Self::Error> {
        match value {
            WsMessageTung::Binary(raw) => {
                let decompressed = decompress(raw.as_ref())?;
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

#[cfg(feature = "server")]
impl TryFrom<Message> for WsMessageAxum {
    type Error = ParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let raw = rmp_serde::to_vec(&value).map_err(ParseError::SerializationError)?;
        let compressed = compress(&raw)?;
        Ok(WsMessageAxum::Binary(compressed.into()))
    }
}

#[cfg(feature = "client")]
impl TryFrom<Message> for WsMessageTung {
    type Error = ParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let raw = rmp_serde::to_vec(&value).map_err(ParseError::SerializationError)?;
        let compressed = compress(&raw)?;
        Ok(WsMessageTung::Binary(compressed.into()))
    }
}
