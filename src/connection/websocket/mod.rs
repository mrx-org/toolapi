#[cfg(feature = "server")]
mod r#async;
mod common;
#[cfg(feature = "client")]
mod sync;

#[cfg(feature = "server")]
pub use r#async::WsChannelAsync;
pub use common::WsMessageType;
#[cfg(feature = "client")]
pub use sync::WsChannelSync;
