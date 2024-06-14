use super::AppError;
use crate::{AppState, Realtime, SpotParam};
use axum::{
    extract::{Query, State},
    response::Json,
};
use std::sync::Arc;

pub async fn realtime(
    selected_spot: Query<SpotParam>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Realtime>, AppError> {
    Ok(Json(
        Realtime::try_get(&selected_spot.0.into(), state.realtime_url).await?,
    ))
}
