use super::AppError;
use crate::{AppState, Realtime, SpotQuery};
use axum::{extract::State, response::Json};
use std::sync::Arc;

pub async fn realtime(
    selected_spot: SpotQuery,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Realtime>, AppError> {
    Ok(Json(
        Realtime::try_get(Arc::new(selected_spot.0.into()), state.realtime_url).await?,
    ))
}
