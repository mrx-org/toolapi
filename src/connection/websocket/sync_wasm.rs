//! Async WebSocket client for wasm32 targets using the browser's WebSocket API.
//! This mirrors the interface of `WsChannelSync` but uses async methods since
//! blocking is not possible on wasm32-unknown-unknown.

use crate::{ToolError, ValueDict, error::ConnectionError};
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, CloseEvent, ErrorEvent, Event, MessageEvent, WebSocket};

use super::common::Message;

/// Async WebSocket client for wasm targets.
///
/// Uses the browser's `WebSocket` API via `web-sys`. The API mirrors
/// [`super::sync::WsChannelSync`] but with async methods where the native
/// version would block.
pub struct WsChannelSyncWasm {
    socket: WebSocket,
    rx: futures_channel::mpsc::UnboundedReceiver<Result<Vec<u8>, ConnectionError>>,
    buffer: Option<Message>,
    // Store closures to prevent them from being dropped (which would unregister the callbacks)
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
    _onerror: Closure<dyn FnMut(ErrorEvent)>,
    _onclose: Closure<dyn FnMut(CloseEvent)>,
}

/// Format a JsValue into a human-readable string for error messages
fn js_to_string(val: &JsValue) -> String {
    if let Some(s) = val.as_string() {
        s
    } else {
        format!("{val:?}")
    }
}

impl WsChannelSyncWasm {
    /// Connect to a WebSocket server. Resolves when the connection is open.
    pub async fn connect(addr: &str) -> Result<Self, ConnectionError> {
        let socket = WebSocket::new(addr)
            .map_err(|e| ConnectionError::WebSocketError(js_to_string(&e)))
            .map_err(|err| ConnectionError::WebSocketError(err.to_string()))?;
        socket.set_binary_type(BinaryType::Arraybuffer);

        // Wait for the connection to open (or fail)
        Self::wait_for_open(&socket).await?;

        // Set up a channel that the onmessage callback writes into
        let (tx, rx) = futures_channel::mpsc::unbounded();

        // onmessage: extract binary data and push into channel
        let tx_msg = tx.clone();
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            let data = e.data();
            if let Ok(buf) = data.dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&buf);
                let bytes = array.to_vec();
                let _ = tx_msg.unbounded_send(Ok(bytes));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        // onerror: push an error into the channel
        let tx_err = tx.clone();
        let onerror = Closure::wrap(Box::new(move |_e: ErrorEvent| {
            let _ =
                tx_err.unbounded_send(Err(ConnectionError::WebSocketError("WebSocket error".into())));
        }) as Box<dyn FnMut(ErrorEvent)>);
        socket.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        // onclose: close the channel so reads return None
        let onclose = Closure::wrap(Box::new(move |_e: CloseEvent| {
            tx.close_channel();
        }) as Box<dyn FnMut(CloseEvent)>);
        socket.set_onclose(Some(onclose.as_ref().unchecked_ref()));

        Ok(Self {
            socket,
            rx,
            buffer: None,
            _onmessage: onmessage,
            _onerror: onerror,
            _onclose: onclose,
        })
    }

    /// Wait for the WebSocket `open` event, or fail on `error`/`close`.
    async fn wait_for_open(socket: &WebSocket) -> Result<(), ConnectionError> {
        let (tx, rx) = futures_channel::oneshot::channel::<Result<(), ConnectionError>>();
        let tx = Rc::new(RefCell::new(Some(tx)));

        let tx_open = Rc::clone(&tx);
        let onopen = Closure::once(move |_: Event| {
            if let Some(tx) = tx_open.borrow_mut().take() {
                let _ = tx.send(Ok(()));
            }
        });
        socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));

        let tx_err = Rc::clone(&tx);
        let onerror = Closure::once(move |_: ErrorEvent| {
            if let Some(tx) = tx_err.borrow_mut().take() {
                let _ = tx.send(Err(ConnectionError::WebSocketError(
                    "WebSocket connection failed".into(),
                )));
            }
        });
        socket.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        let tx_close = Rc::clone(&tx);
        let onclose = Closure::once(move |_: CloseEvent| {
            if let Some(tx) = tx_close.borrow_mut().take() {
                let _ = tx.send(Err(ConnectionError::ConnectionClosed));
            }
        });
        socket.set_onclose(Some(onclose.as_ref().unchecked_ref()));

        let result = rx
            .await
            .map_err(|_| ConnectionError::WebSocketError("open channel dropped".into()))?;

        // Clear the temporary open/error/close handlers (they'll be replaced after connect)
        socket.set_onopen(None);
        socket.set_onerror(None);
        socket.set_onclose(None);

        result
    }

    pub fn close(self) -> Result<(), ConnectionError> {
        self.socket
            .close()
            .map_err(|e| ConnectionError::WebSocketError(js_to_string(&e)))
    }

    pub fn send_abort(&self) -> Result<(), ConnectionError> {
        let bytes = Message::Abort
            .to_bytes()
            .map_err(ConnectionError::ParseError)?;
        self.socket
            .send_with_u8_array(&bytes)
            .map_err(|e| ConnectionError::WebSocketError(js_to_string(&e)))
    }

    pub fn send_values(&self, values: ValueDict) -> Result<(), ConnectionError> {
        let bytes = Message::Values(values)
            .to_bytes()
            .map_err(ConnectionError::ParseError)?;
        self.socket
            .send_with_u8_array(&bytes)
            .map_err(|e| ConnectionError::WebSocketError(js_to_string(&e)))
    }

    /// Fill the message buffer by reading the next message from the channel
    async fn read(&mut self) -> Result<(), ConnectionError> {
        if self.buffer.is_none() {
            // Await the next item from the mpsc channel
            let next = std::future::poll_fn(|cx| {
                use futures_core::Stream;
                Pin::new(&mut self.rx).poll_next(cx)
            })
            .await;

            match next {
                Some(Ok(bytes)) => {
                    self.buffer =
                        Some(Message::from_bytes(&bytes).map_err(ConnectionError::ParseError)?);
                }
                Some(Err(e)) => return Err(e),
                None => {} // channel closed (WebSocket closed), buffer stays None
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
