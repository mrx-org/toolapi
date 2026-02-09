mod common;
pub use common::WsMessageType;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::WsChannelServer;

#[cfg(all(feature = "client", not(target_arch = "wasm32")))]
mod client_native;
#[cfg(all(feature = "client", not(target_arch = "wasm32")))]
pub use client_native::WsChannelClientNative;

#[cfg(all(feature = "client", target_arch = "wasm32"))]
mod client_wasm;
#[cfg(all(feature = "client", target_arch = "wasm32"))]
pub use client_wasm::WsChannelClientWasm;
