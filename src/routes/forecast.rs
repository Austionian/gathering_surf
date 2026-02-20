use super::AppError;
use crate::{AppState, Forecast, SpotQuery};
use axum::{Json, extract::State};
use std::sync::Arc;

pub async fn forecast(
    selected_spot: SpotQuery,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Forecast>, AppError> {
    Ok(Json(
        Forecast::try_get(&selected_spot.0.into(), state.forecast_url).await?,
    ))
}
