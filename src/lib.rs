mod value;
use axum::{
    Router,
    extract::{
        State,
        // WebSocketUpgrade, ws::WebSocket
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
};
pub use value::ValueDict;

#[tokio::main]
pub async fn run(tool: fn(ValueDict) -> ValueDict, index_html: Option<&'static str>) {
    let state = ToolState { tool, index_html };

    let routes = Router::new()
        .route("/", get(index_handler))
        // .route("/tool", any(socket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}

#[derive(Clone)]
struct ToolState {
    tool: fn(ValueDict) -> ValueDict,
    index_html: Option<&'static str>,
}

// async fn tool_handler(mut socket: WebSocket, tool: fn(ValueDict) -> ValueDict) {
//     // do something with state.tool

//     while let Some(msg) = socket.recv().await {
//         let msg = if let Ok(msg) = msg {
//             msg
//         } else {
//             // client disconnected
//             return;
//         };

//         if socket.send(msg).await.is_err() {
//             // client disconnected
//             return;
//         }
//     }
// }

// async fn socket_handler(ws: WebSocketUpgrade, State(state): State<ToolState>) -> Response {
//     ws.on_upgrade(move |socket| tool_handler(socket, state.tool))
// }

async fn index_handler(State(state): State<ToolState>) -> Response {
    match state.index_html {
        Some(html) => Html(html).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
