use crate::{AppState, Forecast};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap},
    response::IntoResponse,
};
use std::sync::Arc;

use super::{get_spot, Spot};

pub async fn forecast(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<Spot>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    let forecast = Forecast::try_get(&get_spot(selected_spot, &state))
        .await
        .unwrap();

    (headers, forecast.to_json())
}
