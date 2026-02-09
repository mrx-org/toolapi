#[cfg(feature = "server")]
mod r#async;
mod common;
#[cfg(feature = "client")]
mod sync;
#[cfg(all(feature = "wasm-client", target_arch = "wasm32"))]
mod sync_wasm;

#[cfg(feature = "server")]
pub use r#async::WsChannelAsync;
pub use common::WsMessageType;
#[cfg(feature = "client")]
pub use sync::WsChannelSync;
#[cfg(all(feature = "wasm-client", target_arch = "wasm32"))]
pub use sync_wasm::WsChannelSyncWasm;
