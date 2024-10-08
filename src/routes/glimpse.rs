use super::AppError;
use crate::{AppState, Realtime, Spot, SpotParam, TEMPLATES};
use axum::{
    extract::{Query, State},
    response::Html,
};
use std::sync::Arc;

/// Handler to return the website's index without streaming the
/// html to the client. Meant to support Glimpse
/// https://apps.apple.com/us/app/glimpse-2/id1524217845
pub async fn glimpse(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<SpotParam>,
) -> Result<Html<String>, AppError> {
    let mut context = tera::Context::new();

    let spot: Arc<Spot> = Arc::new(selected_spot.0.into());

    context.insert("spot", &*spot);
    context.insert("breaks", &state.breaks);
    #[cfg(debug_assertions)]
    context.insert("live_reload", &true);
    #[cfg(not(debug_assertions))]
    context.insert("live_reload", &false);

    match Realtime::try_get(spot, state.realtime_url).await {
        Ok(latest) => {
            context.insert("as_of", &latest.as_of);
            context.insert("wind_direction", &latest.wind_direction);
            context.insert("wind_speed", &latest.wind_speed);
            context.insert("wind_gust", &latest.gusts);
            context.insert("water_temp", &latest.water_temp);
            context.insert("air_temp", &latest.air_temp);
            context.insert("wave_height", &latest.wave_height);
            context.insert("quality", &latest.quality_text);
            context.insert("quality_color", &latest.quality_color);
            context.insert("wave_period", &latest.wave_period);
            context.insert("wave_direction", &latest.wave_direction);
        }
        Err(e) => {
            context.insert("error", &e.to_string());
            context.insert("error_type", &"latest");
            context.insert("container", &"latest-container");
            context.insert("error_container", &"latest-error");
        }
    }

    match TEMPLATES.render("glimpse.html", &context) {
        Ok(s) => Ok(Html(s)),
        Err(e) => {
            tracing::error!("{:?}", e);
            Ok(Html("<html>error</html>".to_string()))
        }
    }
}
