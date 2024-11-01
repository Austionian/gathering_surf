use crate::{AppState, Forecast, Realtime, Spot, SpotParam, WaterQuality, TEMPLATES};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::{html, Markup, PreEscaped};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;

/// This is ugly. Might be worth reverting to tera to create the
/// inline JS. Allows not having to pass the context around in a
/// mutex to the other threads though.
fn error_markup(error_type: &str, e: anyhow::Error) -> Markup {
    html! {
        script {
            (PreEscaped("document.querySelector('#"))
            (error_type)
            (PreEscaped(r#"-container').classList.add("hidden");
                    document.querySelector('#"#))
            (error_type)
            (PreEscaped(r#"-error').innerHTML = `
                        <div class='p-12 flex flex-col items-center align-middle justify-center text-center'>
                          <h2 class='text-xl'>
                          Error loading "#))
            (error_type)
            (PreEscaped(r#" data - please refresh the page or try again later.
                          </h2>
                          <p>"#))
            span class="font-mono" {
                "Error: " (e)
            }
            (PreEscaped("</p></div>`;"))
        }
    }
}

/// Handler to return the website's index
pub async fn root(
    State(state): State<Arc<AppState>>,
    selected_spot: Query<SpotParam>,
) -> Result<Response, AppError> {
    // Create a channel to stream content to client as we get it
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);

    let mut context = tera::Context::new();

    let spot: Arc<Spot> = Arc::new(selected_spot.0.into());

    // Add the initial context to the page for the loading state
    context.insert("spot", &*spot);
    context.insert("breaks", &state.breaks);
    #[cfg(debug_assertions)]
    context.insert("live_reload", &true);
    #[cfg(not(debug_assertions))]
    context.insert("live_reload", &false);

    tx.send(Ok(TEMPLATES.render("index.html", &context)?))
        .await?;

    let realtime_tx = tx.clone();
    let realtime_spot = spot.clone();
    let realtime_state = state.clone();
    tokio::spawn(async move {
        match Realtime::try_get_string(realtime_spot, realtime_state.realtime_url).await {
            Ok(realtime) => {
                let html = html!(
                    script type="application/json" id="realtime-data" {(
                    PreEscaped(
                            realtime)
                    )}
                )
                .into();

                realtime_tx.send(Ok(html)).await.unwrap();
            }
            Err(e) => {
                realtime_tx
                    .send(Ok(html!((error_markup("latest", e))).into()))
                    .await
                    .unwrap();
            }
        }
    });

    let water_quality_tx = tx.clone();
    let water_quality_spot = spot.clone();
    let water_quality_state = state.clone();
    tokio::spawn(async move {
        match WaterQuality::try_get(water_quality_spot, water_quality_state.quality_url).await {
            Ok(water_quality) => {
                let html = html!(
                    script type="application/json" id="water-quality-data" {(
                    PreEscaped(
                            serde_json::to_string(&water_quality).unwrap())
                    )}
                )
                .into();

                water_quality_tx.send(Ok(html)).await.unwrap();
            }
            Err(e) => {
                water_quality_tx
                    .send(Ok(html!((error_markup("water quality", e))).into()))
                    .await
                    .unwrap();
            }
        }
    });

    tokio::spawn(async move {
        match Forecast::try_get(&spot, state.forecast_url).await {
            Ok(forecast) => {
                let html = html!(
                    script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.3/dist/chart.umd.min.js" {}
                    script type="application/json" id="forecast-data" {(
                    PreEscaped(
                            serde_json::to_string(&forecast).unwrap())
                    )}
                )
                .into();

                tx.send(Ok(html)).await.unwrap();

                Ok(())
            }
            Err(e) => {
                tx.send(Ok(html!((error_markup("forecast", e))).into()))
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
        .header("content-encoding", "none")
        .header("cache-control", "no-transform")
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
