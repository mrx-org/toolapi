//! Async WebSocket client for wasm32 targets using the browser's WebSocket API.
//! This mirrors the interface of `WsChannelSync` but uses async methods since
//! blocking is not possible on wasm32-unknown-unknown.

use crate::{ToolError, ValueDict, error::ConnectionError};
use futures::{SinkExt, StreamExt};
use ws_stream_wasm::{WsMeta, WsMessage, WsStream};

use super::common::Message;

/// Async WebSocket client for wasm targets.
///
/// Uses the browser's `WebSocket` API via [`ws_stream_wasm`]. The API mirrors
/// [`super::sync::WsChannelSync`] but with async methods where the native
/// version would block.
pub struct WsChannelSyncWasm {
    ws_meta: WsMeta,
    ws_stream: WsStream,
    /// If we tried to read a message of one type but received another, the message is buffered here.
    buffer: Option<Message>,
}

impl WsChannelSyncWasm {
    /// Connect to a WebSocket server. Resolves when the connection is open.
    pub async fn connect(addr: &str) -> Result<Self, ConnectionError> {
        let (ws_meta, ws_stream) = WsMeta::connect(addr, None)
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))?;

        Ok(Self {
            ws_meta,
            ws_stream,
            buffer: None,
        })
    }

    pub async fn close(self) -> Result<(), ConnectionError> {
        self.ws_meta
            .close()
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))?;
        Ok(())
    }

    pub async fn send_abort(&mut self) -> Result<(), ConnectionError> {
        let bytes = Message::Abort
            .to_bytes()
            .map_err(ConnectionError::ParseError)?;
        self.ws_stream
            .send(WsMessage::Binary(bytes))
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))
    }

    pub async fn send_values(&mut self, values: ValueDict) -> Result<(), ConnectionError> {
        let bytes = Message::Values(values)
            .to_bytes()
            .map_err(ConnectionError::ParseError)?;
        self.ws_stream
            .send(WsMessage::Binary(bytes))
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))
    }

    /// Fill the message buffer by reading the next message from the stream
    async fn read(&mut self) -> Result<(), ConnectionError> {
        if self.buffer.is_none() {
            match self.ws_stream.next().await {
                Some(WsMessage::Binary(bytes)) => {
                    self.buffer = Some(
                        Message::from_bytes(&bytes).map_err(ConnectionError::ParseError)?,
                    );
                }
                Some(WsMessage::Text(_)) => {
                    // Unexpected text message; skip it
                }
                None => {} // stream closed, buffer stays None
            }
        }

        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Option<String>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(Message::Message(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }

    pub async fn read_result(
        &mut self,
    ) -> Result<Option<Result<ValueDict, ToolError>>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(Message::Result(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }
}
