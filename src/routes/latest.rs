use super::AppError;
use crate::{Latest, SpotParam};
use axum::{extract::Query, response::Json};

pub async fn latest(selected_spot: Query<SpotParam>) -> Result<Json<Latest>, AppError> {
    Ok(Json(Latest::try_get(&selected_spot.0.into()).await?))
}
