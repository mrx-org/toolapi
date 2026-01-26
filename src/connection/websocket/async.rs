//! Async implementation of the WebSocket communication.
//! This is used by the server (which hosts the tool).

use crate::{ValueDict, error::ConnectionError};

// NOTE: implementation is analoguous to sync, look there for more comments

pub struct WsChannelAsync {
    socket: axum::extract::ws::WebSocket,
    buffer: Option<super::common::Message>,
}

impl WsChannelAsync {
    pub fn new(socket: axum::extract::ws::WebSocket) -> Self {
        Self {
            socket,
            buffer: None,
        }
    }

    pub async fn send_message(&mut self, msg: String) -> Result<(), ConnectionError> {
        self.socket
            .send(
                super::common::Message::Message(msg)
                    .try_into()
                    .map_err(ConnectionError::ParseError)?,
            )
            .await
            .map_err(ConnectionError::AxumError)
    }

    pub async fn send_result(&mut self, result: Result<ValueDict, String>) -> Result<(), ConnectionError> {
        self.socket
            .send(
                super::common::Message::Result(result)
                    .try_into()
                    .map_err(ConnectionError::ParseError)?,
            )
            .await
            .map_err(ConnectionError::AxumError)
    }

    async fn read(&mut self) -> Result<(), ConnectionError> {
        if self.buffer.is_none() {
            // Difference to tungstenite: there is no can_read() method;
            // instead None is returned from a closed stream.
            if let Some(msg) = self.socket.recv().await {
                let msg = msg.map_err(ConnectionError::AxumError)?;
                let msg = msg.try_into().map_err(ConnectionError::ParseError)?;
                self.buffer = Some(msg)
            }
        }

        Ok(())
    }

    pub async fn read_abort(&mut self) -> Result<Option<()>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(super::common::Message::Abort) => Ok(Some(())),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }

    pub async fn read_values(&mut self) -> Result<Option<ValueDict>, ConnectionError> {
        self.read().await?;
        match self.buffer.take() {
            Some(super::common::Message::Values(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }
}
