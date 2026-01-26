//! Common structures shared by client and server / sync and async impls.
//! This is the heart of the communication - both sides have to agree on this!

use crate::{ValueDict, error::ParseError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Values(ValueDict),
    Result(Result<ValueDict, String>),
    Message(String),
    Abort,
}

type WsMessageAxum = axum::extract::ws::Message;
type WsMessageTung = tungstenite::Message;
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
            WsMessageAxum::Text(string) => {
                Ok(serde_json::from_str(&string).map_err(ParseError::DeserializationError)?)
            }
            msg => Err(ParseError::WrongMessageType {
                expected: WsMessageType::Text,
                found: msg.into(),
            }),
        }
    }
}

impl TryFrom<WsMessageTung> for Message {
    type Error = ParseError;

    fn try_from(value: WsMessageTung) -> Result<Self, Self::Error> {
        match value {
            WsMessageTung::Text(string) => {
                Ok(serde_json::from_str(&string).map_err(ParseError::DeserializationError)?)
            }
            msg => Err(ParseError::WrongMessageType {
                expected: WsMessageType::Text,
                found: msg.into(),
            }),
        }
    }
}

impl TryFrom<Message> for WsMessageAxum {
    type Error = ParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let string = serde_json::to_string(&value).map_err(ParseError::SerializationError)?;
        Ok(WsMessageAxum::Text(string.into()))
    }
}

impl TryFrom<Message> for WsMessageTung {
    type Error = ParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let string = serde_json::to_string(&value).map_err(ParseError::SerializationError)?;
        Ok(WsMessageTung::Text(string.into()))
    }
}
