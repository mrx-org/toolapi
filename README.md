# ToolAPI

MRX ToolAPI — connect clients and tools running in the cloud.

ToolAPI is a Rust framework for building client-server applications that communicate over WebSocket connections. It enables clients to invoke remote tools via WebSocket, with support for bidirectional message passing, abort signaling, and strongly-typed parameter passing using a dynamic `Value` system.

## Usage

Add `toolapi` to your `Cargo.toml`:

```toml
[dependencies]
toolapi = "0.1"
```

### Defining a Tool (Server)

A tool is a function that receives a `ValueDict` of inputs and a `Sender` for sending progress messages back to the client:

```rust
use toolapi::{ValueDict, Sender, Value, ToolError};

fn my_tool(mut input: ValueDict, mut sender: Sender) -> Result<ValueDict, ToolError> {
    let param: String = input.pop("param")?;

    sender.send("Processing...".to_string())?;

    Ok(ValueDict::from([
        ("output".to_string(), Value::String("done".to_string())),
    ]))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    toolapi::run_server(my_tool, None).await
}
```

The server listens on `0.0.0.0:8080` and accepts WebSocket connections at `/tool`. An optional HTML string can be served at `/`.

### Calling a Tool (Client)

```rust
use toolapi::{call, ValueDict, Value};

let input = ValueDict::from([
    ("param".to_string(), Value::String("hello".to_string())),
]);

let result = call("ws://localhost:8080/tool", input, |msg| {
    println!("{msg}");
    true // return false to abort
});
```

The callback receives progress messages from the tool. Returning `false` sends an abort signal.

## Core Types

| Type | Description |
|---|---|
| `Value` | Dynamic typed enum carrying booleans, integers, floats, strings, and MR-specific data |
| `ValueDict` | Dictionary of named `Value` entries, used for tool input and output |
| `Sender` | Channel for sending progress messages from a tool to the client |
| `ToolError` | Error type returned by tools (abort or custom error) |
| `ToolCallError` | Client-side error from `call()` |

## MR-Specific Types

ToolAPI includes domain-specific types for MRI simulation and analysis:

- **Signal** / **Encoding** — MR signal data and k-space encoding trajectories
- **TissueProperties** — T1, T2, T2', ADC parameters
- **VoxelPhantom** / **VoxelGridPhantom** / **MultiTissuePhantom** — phantom representations
- **EventSeq** / **BlockSeq** — MRI pulse sequence descriptions

## Protocol

Communication uses JSON messages over WebSocket:

- `Values(ValueDict)` — input/output data
- `Message(String)` — progress messages from tool to client
- `Result(Result<ValueDict, ToolError>)` — final result
- `Abort` — client-requested abort

## Disclaimer

This README was generated using [Claude Code](https://claude.com/claude-code). No LLMs were used in the writing of the code itself.

## License

AGPL — see [LICENSE](LICENSE) for details.
