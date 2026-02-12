//! Async WebSocket client for wasm32 targets using the browser's WebSocket API.
//! This mirrors the interface of `WsChannelSync` but uses async methods since
//! blocking is not possible on wasm32-unknown-unknown.

use crate::{ToolError, Value, error::ConnectionError};
use futures::{SinkExt, StreamExt};
use ws_stream_wasm::{WsMeta, WsStream};

use super::common::Message;

/// Async WebSocket client for wasm targets.
///
/// Uses the browser's `WebSocket` API via [`ws_stream_wasm`]. The API mirrors
/// [`super::sync::WsChannelSync`] but with async methods where the native
/// version would block.
pub struct WsChannelClientWasm {
    ws_meta: WsMeta,
    ws_stream: WsStream,
    /// If we tried to read a message of one type but received another, the message is buffered here.
    buffer: Option<Message>,
}

impl WsChannelClientWasm {
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
        self.ws_stream
            .send(Message::Abort.try_into()?)
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))
    }

    pub async fn send_input(&mut self, input: Value) -> Result<(), ConnectionError> {
        self.ws_stream
            .send(Message::Input(input).try_into()?)
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))
    }

    /// Fill the message buffer by reading the next message from the stream
    async fn read(&mut self) -> Result<(), ConnectionError> {
        if self.buffer.is_none() {
            if let Some(msg) = self.ws_stream.next().await {
                self.buffer = Some(msg.try_into()?);
            }
        }

        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Option<String>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(Message::ToolMsg(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }

    pub async fn read_output(
        &mut self,
    ) -> Result<Option<Result<Value, ToolError>>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(Message::Output(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }
}
