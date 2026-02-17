# Changelog

Newest changes are first, releases to [crates.io](https://crates.io/crates/toolapi) are **bold**.

- **toolapi 0.4.2**
- Add optional `pyo3` feature with `FromPyObject` implementations for all Value types
- **toolapi 0.4.1**
- `Int`, `Float`, and other atomic types are no longer newtype-wrapped
- All supported types can now by extracted into concrete Rust types
- Values can now by indexed by a "pointer" (e.g.: `"phantom/tissues/3/density"`)
- Indexing now returns a proper result instead of an Option
- **toolapi 0.4.0**
- Remodel `Value` type hierarchy, add homo- and heterogeneous collections to `Value` directly
- Add `Value::index()` and conversion traits for working with new values
- Rename client/server channel methods to match (`send_values` -> `send_input`, `read_result` -> `read_output`, etc.)
- **toolapi 0.3.2**
- `ruzstd` only implements compression mode `Fastest` - switch to avoid crash
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