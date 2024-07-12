use crate::{AppState, Forecast, Realtime, Spot, SpotParam, TEMPLATES};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;

/// Handler to return the website's index
pub async fn root(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<SpotParam>,
) -> Result<Response, AppError> {
    // Create a channel to stream content to client as we get it
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);

    tokio::spawn(async move {
        let mut context = tera::Context::new();

        let spot: Spot = selected_spot.0.into();

        context.insert("spot", &spot);
        context.insert("breaks", &state.breaks);

        tx.send(Ok(TEMPLATES.render("index.html", &context)?))
            .await?;

        match Realtime::try_get(&spot, state.realtime_url).await {
            Ok(latest) => {
                context.insert("latest_json", &serde_json::to_string(&latest)?);

                tx.send(Ok(TEMPLATES.render("latest.html", &context)?))
                    .await?;
            }
            Err(e) => {
                context.insert("error", &e.to_string());
                context.insert("error_type", &"latest");
                context.insert("container", &"latest-container");
                context.insert("error_container", &"latest-error");
                tx.send(Ok(TEMPLATES.render("error.html", &context)?))
                    .await?;
            }
        }

        match Forecast::try_get(&spot, state.forecast_url).await {
            Ok(forecast) => {
                context.insert("forecast_json", &serde_json::to_string(&forecast)?);

                tx.send(Ok(TEMPLATES.render("forecast.html", &context)?))
                    .await?;

                Ok(())
            }
            Err(e) => {
                context.insert("error", &e.to_string());
                context.insert("error_type", &"forecast");
                context.insert("container", &"forecast-container");
                context.insert("error_container", &"forecast-error");
                tx.send(Ok(TEMPLATES.render("error.html", &context)?))
                    .await?;

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
