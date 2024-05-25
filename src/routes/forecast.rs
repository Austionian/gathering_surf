use super::AppError;
use crate::{Forecast, SpotParam};
use axum::{extract::Query, response::Json};

pub async fn forecast(selected_spot: Query<SpotParam>) -> Result<Json<Forecast>, AppError> {
    Ok(Json(Forecast::try_get(&selected_spot.0.into()).await?))
}
