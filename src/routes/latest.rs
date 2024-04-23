use crate::{AppState, Latest};
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use std::sync::Arc;

pub async fn latest(
    State(_state): State<Arc<AppState>>,
    // selected_spot: Query<Spot>,
) -> impl IntoResponse {
    // Get the selected spot, fallback to Atwater
    // let mut spot = selected_spot.0.spot.unwrap_or("Atwater".to_string());

    let latest = Latest::try_get().await.unwrap();

    Json(latest)
}
