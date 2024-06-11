use super::AppError;
use crate::{AppState, Forecast, SpotParam};
use axum::{
    extract::{Query, State},
    response::Json,
};
use std::sync::Arc;

pub async fn forecast(
    selected_spot: Query<SpotParam>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Forecast>, AppError> {
    Ok(Json(
        Forecast::try_get(&selected_spot.0.into(), &state.noaa_api).await?,
    ))
}
