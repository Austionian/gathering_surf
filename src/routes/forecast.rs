use crate::{Forecast, SpotParam};
use axum::{
    extract::Query,
    http::{header, HeaderMap},
    response::IntoResponse,
};

pub async fn forecast(selected_spot: Query<SpotParam>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    let forecast = Forecast::try_get(&selected_spot.0.into()).await.unwrap();

    (headers, forecast.to_json())
}