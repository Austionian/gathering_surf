use crate::AppState;
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn watch(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut rx = state.event_stream.clone().subscribe();
    ws.on_upgrade(move |mut socket| async move {
        if let Ok(_) = rx.recv().await {
            if socket.send("reload".into()).await.is_err() {
                // client disconnected
                return;
            }
        }
    })
}
