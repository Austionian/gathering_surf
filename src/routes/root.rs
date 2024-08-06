use crate::{templates, AppState, Forecast, Realtime, Spot, SpotParam, WaterQuality};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{convert::Infallible, ops::Deref, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::info;

/// Handler to return the website's index
pub async fn root(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<SpotParam>,
) -> Result<Response, AppError> {
    // Create a channel to stream content to client as we get it
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);

    let tx = Arc::new(tx);
    let context = Arc::new(Mutex::new(tera::Context::new()));

    let spot: Arc<Spot> = Arc::new(selected_spot.0.into());

    context.lock().await.insert("spot", &*spot);
    context.lock().await.insert("breaks", &state.breaks);

    tx.send(Ok(
        templates().render("index.html", context.lock().await.deref())?
    ))
    .await?;

    // Figure out a better way than unwraping in the spawned tasks
    let realtime_tx = tx.clone();
    let realtime_context = context.clone();
    let realtime_spot = spot.clone();
    let realtime_state = state.clone();
    tokio::spawn(async move {
        match Realtime::try_get(realtime_spot, realtime_state.realtime_url).await {
            Ok(realtime) => {
                let mut context = realtime_context.lock().await;
                context.insert("latest_json", &serde_json::to_string(&realtime).unwrap());
                info!("realtime data parsed");

                realtime_tx
                    .send(Ok(templates().render("latest.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
            Err(e) => {
                let mut context = realtime_context.lock().await;
                context.insert("error", &e.to_string());
                context.insert("error_type", &"latest");
                context.insert("container", &"latest-container");
                context.insert("error_container", &"latest-error");
                realtime_tx
                    .send(Ok(templates().render("error.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
        }
    });

    let water_quality_tx = tx.clone();
    let water_quality_context = context.clone();
    let water_quality_spot = spot.clone();
    let water_quality_state = state.clone();
    tokio::spawn(async move {
        match WaterQuality::try_get(water_quality_spot, water_quality_state.quality_url).await {
            Ok(water_quality) => {
                let mut context = water_quality_context.lock().await;
                context.insert(
                    "water_quality_json",
                    &serde_json::to_string(&water_quality).unwrap(),
                );
                info!("water quality data parsed");

                water_quality_tx
                    .send(Ok(templates()
                        .render("water_quality.html", &context)
                        .unwrap()))
                    .await
                    .unwrap();
            }
            Err(e) => {
                let mut context = water_quality_context.lock().await;
                context.insert("error", &e.to_string());
                context.insert("error_type", &"beach status");
                context.insert("container", &"water-quality-container");
                context.insert("error_container", &"water-quality-error");
                water_quality_tx
                    .send(Ok(templates().render("error.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
        }
    });

    tokio::spawn(async move {
        match Forecast::try_get(&spot, state.forecast_url).await {
            Ok(forecast) => {
                let mut context = context.lock().await;
                context.insert("forecast_json", &serde_json::to_string(&forecast).unwrap());
                info!("forecast data parsed");

                tx.send(Ok(templates().render("forecast.html", &context).unwrap()))
                    .await
                    .unwrap();

                Ok(())
            }
            Err(e) => {
                let mut context = context.lock().await;
                context.insert("error", &e.to_string());
                context.insert("error_type", &"forecast");
                context.insert("container", &"forecast-container");
                context.insert("error_container", &"forecast-error");
                tx.send(Ok(templates().render("error.html", &context).unwrap()))
                    .await
                    .unwrap();

                Err(AppError(anyhow!("Failed to load forecast.")))
            }
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=UTF-8")
        .header("X-Content-Type-Options", "nosniff")
        .header("content-encoding", "")
        .body(body)?)
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
