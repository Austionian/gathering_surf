use crate::AppState;
use axum::{
    extract::{State, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn watch(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut rx = state.event_stream.clone().subscribe();
    ws.on_upgrade(move |mut socket| async move {
        tracing::info!("new connection, watching");
        if rx.recv().await.is_ok() {
            let _ = socket.send("reload".into()).await;
        }
    })
}
