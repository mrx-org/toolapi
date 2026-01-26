use crate::connection::websocket::WsMessageType;

pub enum AbortReason {
    RequestedByClient,
    ConnectionError,
    WebSocketError,
}

impl From<AbortReason> for String {
    fn from(value: AbortReason) -> Self {
        let reason = match value {
            AbortReason::RequestedByClient => "RequestedByClient",
            AbortReason::ConnectionError => "ConnectionError",
            AbortReason::WebSocketError => "WebSocketError",
        };
        format!("Tool was asked to abort: {reason}")
    }
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

#[derive(Debug)]
pub enum ParseError {
    SerializationError(serde_json::Error),
    DeserializationError(serde_json::Error),
    WrongMessageType {
        expected: WsMessageType,
        got: WsMessageType,
    },
}

#[derive(Debug)]
pub enum ConnectionError {
    TungsteniteError(tungstenite::Error),
    AxumError(axum::Error),
    ParseError(ParseError),
    ConnectionClosed,
}
