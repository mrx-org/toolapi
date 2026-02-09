//! This module helps sending data between the client and the server via WebSocket,
//! as well as between the async server and the sync tool via channels.
#[cfg(feature = "server")]
pub mod channel;
pub mod websocket;
