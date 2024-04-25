use crate::{capitalize, AppState, Forecast, Latest, TEMPLATES};
use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;

#[derive(Deserialize, Debug)]
pub struct Spot {
    pub spot: Option<String>,
}

pub fn get_spot(selected_spot: Query<Spot>, state: &AppState) -> String {
    // Get the selected spot, fallback to Atwater
    let mut spot = selected_spot.0.spot.unwrap_or("Atwater".to_string());

    // Make sure the selected spot is valid, fallback to Atwater if not
    if !state
        .breaks
        .iter()
        .any(|b| b.name.to_lowercase() == spot.to_lowercase())
    {
        spot = "Atwater".to_string();
    }

    spot
}

/// Handler to return the website's index
pub async fn root(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<Spot>,
) -> impl IntoResponse {
    // Create a channel to stream content to client as we get it
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);

    tokio::spawn(async move {
        let mut context = tera::Context::new();

        let spot = get_spot(selected_spot, &state);

        context.insert("spot", &capitalize(&spot));
        context.insert("breaks", &state.breaks);

        tx.send(Ok(TEMPLATES.render("index.html", &context).unwrap()))
            .await
            .unwrap();

        match Latest::try_get().await {
            Ok(latest) => {
                context.insert("latest_json", &serde_json::to_string(&latest).unwrap());

                tx.send(Ok(TEMPLATES.render("latest.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
            Err(e) => {
                context.insert("error", &e.to_string());
                context.insert("error_type", &"latest");
                context.insert("container", &"latest-container");
                context.insert("error_container", &"latest-error");
                tx.send(Ok(TEMPLATES.render("error.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
        }

        match Forecast::try_get(&spot).await {
            Ok(mut forecast) => {
                forecast.condense();
                forecast.get_quality();
                context.insert("forecast_json", &forecast.to_json());

                tx.send(Ok(TEMPLATES.render("forecast.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
            Err(e) => {
                context.insert("error", &e.to_string());
                context.insert("error_type", &"forecast");
                context.insert("container", &"forecast-container");
                context.insert("error_container", &"forecast-error");
                tx.send(Ok(TEMPLATES.render("error.html", &context).unwrap()))
                    .await
                    .unwrap();
            }
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=UTF-8")
        .header("X-Content-Type-Options", "nosniff")
        .header("content-encoding", "")
        .body(body)
        .unwrap()
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
