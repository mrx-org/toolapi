mod common;
pub use common::WsMessageType;

#[cfg(feature = "server")]
mod r#async;
#[cfg(feature = "server")]
pub use r#async::WsChannelAsync;

#[cfg(all(feature = "client", not(target_arch = "wasm32")))]
mod sync;
#[cfg(all(feature = "client", not(target_arch = "wasm32")))]
pub use sync::WsChannelSync;

#[cfg(all(feature = "client", target_arch = "wasm32"))]
mod sync_wasm;
#[cfg(all(feature = "client", target_arch = "wasm32"))]
pub use sync_wasm::WsChannelSyncWasm;
