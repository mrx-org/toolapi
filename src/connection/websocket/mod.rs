mod common;
mod sync;
mod r#async;

pub use sync::WsChannelSync;
pub use r#async::WsChannelAsync;

#[derive(Debug)]
pub enum ConnectionError {
    TungsteniteError(tungstenite::Error),
    AxumError(axum::Error),
    ParseError(common::ParseError),
    ConnectionClosed
}
