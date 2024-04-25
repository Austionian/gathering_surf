use crate::{routes::Spot, AppState, Forecast};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap},
    response::IntoResponse,
};
use std::sync::Arc;

use super::get_spot;

pub async fn forecast(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<Spot>,
) -> impl IntoResponse {
    let spot = get_spot(selected_spot, &state);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    let mut forecast = Forecast::try_get(&spot).await.unwrap();
    forecast.condense();
    forecast.get_quality();

    (headers, forecast.to_json())
}
