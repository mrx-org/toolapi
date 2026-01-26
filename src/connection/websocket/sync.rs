//! Sync / blocking implementation of the WebSocket communication.
//! This is used by the client (usually some Python script).

use crate::{ValueDict, error::ConnectionError};
use std::net::TcpStream;
use tungstenite::{client::IntoClientRequest, stream::MaybeTlsStream};

pub struct WsChannelSync {
    socket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
    /// If we tried to read a message of one type but received another, the message is buffered here.
    buffer: Option<super::common::Message>,
}

impl WsChannelSync {
    pub fn connect<Req: IntoClientRequest>(request: Req) -> Result<Self, ConnectionError> {
        // TODO: should we look at the (ignored _) response?
        let (socket, _) = tungstenite::connect(request)?;

        Ok(Self {
            socket,
            buffer: None,
        })
    }

    pub fn close(mut self) -> Result<(), ConnectionError> {
        self.socket.close(None)?;
        Ok(())
    }

    pub fn send_abort(&mut self) -> Result<(), ConnectionError> {
        self.socket
            .send(super::common::Message::Abort.try_into()?)?;
        Ok(())
    }

    pub fn send_values(&mut self, values: ValueDict) -> Result<(), ConnectionError> {
        self.socket
            .send(super::common::Message::Values(values).try_into()?)?;
        Ok(())
    }

    /// Fill the message buffer, error on connection failure (but not on closed stream)
    fn read(&mut self) -> Result<(), ConnectionError> {
        // Only try to read if we need to and are able to:
        if self.buffer.is_none() && self.socket.can_read() {
            self.buffer = Some(self.socket.read()?.try_into()?);
        }

        Ok(())
    }

    pub fn read_message(&mut self) -> Result<Option<String>, ConnectionError> {
        self.read()?;
        match self.buffer.take() {
            Some(super::common::Message::Message(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }

    pub fn read_result(&mut self) -> Result<Option<Result<ValueDict, String>>, ConnectionError> {
        self.read()?;
        match self.buffer.take() {
            Some(super::common::Message::Result(x)) => Ok(Some(x)),
            Some(msg) => {
                self.buffer = Some(msg);
                Ok(None)
            }
            None => Err(ConnectionError::ConnectionClosed),
        }
    }
}
