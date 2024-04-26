use crate::{Latest, SpotParam};
use axum::{
    extract::Query,
    response::{IntoResponse, Json},
};

pub async fn latest(selected_spot: Query<SpotParam>) -> impl IntoResponse {
    let latest = Latest::try_get(&selected_spot.0.into()).await.unwrap();

    Json(latest)
}
