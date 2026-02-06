use axum::{
    Router,
    routing::{any, get},
};

mod connection;
mod error;
mod util;

// =====================================
// Public API of toolapi
// =====================================

pub mod value;

pub use error::*;
pub use value::{Value, ValueDict};

/// Function which prints a message, sends it to the client, and returns weather
/// the client requested to abort the running tool.
///
/// This function is not static but an object passed to the tool as a parameter
/// because it contains the unique data (the connection to the client). Use it
/// as a logging function while propagating errors to abort on request.
///
/// See [`run_server`] for an example on how to use it
pub type MessageFn = dyn FnMut(String) -> Result<(), AbortReason>;

/// Signature of tool functions passed to [`run_server`].
///
/// It recieves the inputs of the caller as argument, as well as a instance of
/// [`MessageFn`] to log messages and abort on request. It returns the computed
/// value (e.g.: a simulation result, a parsed sequence) or an error, which will
/// be communicated to the client appropriately.
///
/// # Examples
/// ```no_run
/// # use toolapi::{ValueDict, MessageFn, ToolError};
///
/// /// Tool which debug prints the input arguents and returns them to sender.
/// fn tool(input: ValueDict, send_msg: &mut MessageFn) -> Result<ValueDict, ToolError> {
///     send_msg(format!("Args: {input:?}"))?;
///     Ok(input)
/// }
/// ```
pub type ToolFn = fn(ValueDict, &mut MessageFn) -> Result<ValueDict, ToolError>;

/// Starts a server, running `tool` in parallel for every requesting client.
///
/// Routes:
/// - `/` (GET): Returns an optional static web page (`index_html`) or 404
/// - `/tool` (WebSocket): Runs the tool, pass this url to [`call`]
///
/// `tool` is a blocking function that implements the actual business logic of
/// this server. It runs on a separate thread and will not block the server from
/// hanlding more requests in parallel. See [`ToolFn`] for more details.
///
/// # Examples
/// ```no_run
/// # use toolapi::{run_server, ValueDict, MessageFn, ToolError};
///
/// fn main() -> Result<(), std::io::Error> {
///     run_server(tool, Some(INDEX_HTML))
/// }
///
/// fn tool(input: ValueDict, send_msg: &mut MessageFn) -> Result<ValueDict, ToolError> {
///     send_msg(format!("Args: {input:?}"))?;
///     Ok(input)
/// }
///
/// const INDEX_HTML: &'static str = "
///     <!doctype html>
///     <html lang='en'>
///       <head>
///         <meta charset='utf-8'>
///         <title>My fancy tool</title>
///       </head>
///       <body>
///         <p>This tool debug prints all passed values and returns them to sender</p>
///       </body>
///     </html>
/// ";
/// ```
pub fn run_server(tool: ToolFn, index_html: Option<&'static str>) -> Result<(), std::io::Error> {
    // Setup routes and state to pass data to handlers
    let state = util::ToolState { tool, index_html };
    let routes = Router::new()
        .route("/", get(util::index_handler))
        .route("/tool", any(util::socket_handler))
        .with_state(state);

    // We can configure the runtime here: single / multithreaded, number of workers...
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Server code that runs continuously until the program dies
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
            axum::serve(listener, routes).await
        })
}

/// Execute a tool hosted at url `addr` with inputs `input`.
/// 
/// This is meant to act as close as possible to a simple local function call.
/// Code should not worry about the fact that this sends the inputs to a server,
/// blocks on waiting for it to finish and returns the computed result. The
/// only hint is the `on_message` callback: A function that will be called on
/// every message sent by the server, which can request it to abort.
/// 
/// - `addr`: WebSocket url of the server, e.g.: `"wss://tool-xxx-flyio.fly.dev/tool"`
/// - `input`: [`ValueDict`] of parameters that are passed to the tool
/// - `on_message`: callback function that receives a message string and returns
///   `true` if the tool should continue running or `false` if it should abort.
/// 
/// `on_message` could be a closure containing a stop time, requesting the tool
/// to abort after a timeout; it could carry a channel to GUI user abort button.
/// 
/// # Example
/// ```no_run
/// fn on_message(msg: String) -> bool {
///     println!("[TOOL] {msg}");
///     true
/// }
/// 
/// let input = todo!();
/// 
/// call("wss://tool-xxx-flyio.fly.dev/tool", input, on_message)
/// ```
pub fn call(
    addr: &str,
    input: ValueDict,
    mut on_message: impl FnMut(String) -> bool,
) -> Result<ValueDict, ToolCallError> {
    // Create a connection between client and server over WebSocket
    let mut ws_client = connection::websocket::WsChannelSync::connect(addr)?;
    // Send the input parameters to the server
    ws_client.send_values(input)?;

    // Loop over messages sent by the server and ask the callback if we should abort
    while let Some(msg) = ws_client.read_message()? {
        if !on_message(msg) {
            // abort was requested by client callback
            ws_client.send_abort()?;
            ws_client.close()?;
            return Err(ToolCallError::OnMessageAbort);
        }
    }

    // Read result, handle shutdown, return result
    let result = ws_client
        .read_result()?
        .ok_or(ToolCallError::ProtocolError)?;
    // TODO: add a variant `ToolCallError::CloseFailed` which contains the already received result
    ws_client.close()?;
    result.map_err(ToolCallError::ToolReturnedError)
}
