use crate::{AppState, Latest};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};
use std::sync::Arc;

use super::{get_spot, Spot};

pub async fn latest(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<Spot>,
) -> impl IntoResponse {
    let latest = Latest::try_get(&get_spot(selected_spot, &state))
        .await
        .unwrap();

    Json(latest)
}
