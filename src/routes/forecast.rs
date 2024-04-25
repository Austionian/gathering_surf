use crate::{AppState, Forecast};
use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn forecast(
    State(_state): State<Arc<AppState>>,
    // selected_spot: Query<Spot>,
) -> impl IntoResponse {
    // Get the selected spot, fallback to Atwater
    // let mut spot = selected_spot.0.spot.unwrap_or("Atwater".to_string());

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    let mut forecast = Forecast::try_get().await.unwrap();
    forecast.condense();
    forecast.get_quality();

    (headers, forecast.to_json())
}
