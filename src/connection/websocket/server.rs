//! Async implementation of the WebSocket communication.
//! This is used by the server (which hosts the tool).

use crate::{ConnectionError, ToolError, Value};

use super::common::Message;

// NOTE: implementation is analoguous to sync, look there for more comments

pub struct WsChannelServer {
    socket: axum::extract::ws::WebSocket,
    buffer: Option<Message>,
}

impl WsChannelServer {
    pub fn new(socket: axum::extract::ws::WebSocket) -> Self {
        Self {
            socket,
            buffer: None,
        }
    }

    pub async fn send_message(&mut self, msg: String) -> Result<(), ConnectionError> {
        self.socket
            .send(Message::ToolMsg(msg).try_into()?)
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))
    }

    pub async fn send_output(
        &mut self,
        result: Result<Value, ToolError>,
    ) -> Result<(), ConnectionError> {
        self.socket
            .send(Message::Output(result).try_into()?)
            .await
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))
    }

    async fn read(&mut self) -> Result<(), ConnectionError> {
        if self.buffer.is_none() {
            // Difference to tungstenite: there is no can_read() method;
            // instead None is returned from a closed stream.
            if let Some(msg) = self.socket.recv().await {
                let msg = msg.map_err(|err| ConnectionError::WebSocketError(err.to_string()))?;
                self.buffer = Some(msg.try_into()?)
            }
        }

        Ok(())
    }

    pub async fn read_abort(&mut self) -> Result<Option<()>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(Message::Abort) => Ok(Some(())),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }

    pub async fn read_input(&mut self) -> Result<Option<Value>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(Message::Input(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }
}
