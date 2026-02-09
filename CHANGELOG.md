# Changelog

Newest changes are first, releases to [crates.io](https://crates.io/crates/toolapi) are **bold**.

- **toolapi 0.3.1**
- Replace `zstd` (C dependency) with `ruzstd` (pure Rust) for wasm32 compatibility
- Add wasm32 WebSocket client using `ws_stream_wasm`, selected automatically by target
- **toolapi 0.3.0**
- Add `server` and `client` feature flags to gate code paths and their dependencies
- **toolapi 0.2.2**
- Set license to AGPL-3.0-only in Cargo.toml
- **toolapi 0.2.1**
- Introduce changelog
- Clean up lib.rs, add documentation