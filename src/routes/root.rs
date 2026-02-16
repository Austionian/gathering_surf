use crate::{AppState, Forecast, Realtime, Spot, SpotParam, WaterQuality, TEMPLATES};
use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::{html, Markup, PreEscaped};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc::{self, Sender};
use tracing::error;

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
            (PreEscaped(r#" data</h2>
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
    // Create a channel to stream content to client as we get it.
    // Only allow one message at time so the buffer is cleared out
    // as messages are sent.
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(1);

    let mut context = tera::Context::new();

    let spot: Arc<Spot> = Arc::new(selected_spot.0.into());
    // Wrap the sender in an arc so the channel doesn't close early
    let tx: Arc<Sender<Result<_, _>>> = Arc::new(tx);

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
        match Realtime::try_get_string(realtime_spot, realtime_state).await {
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
                error!("Failed to load realtime data: {e}");
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
        match WaterQuality::try_get_string(water_quality_spot, water_quality_state).await {
            Ok(water_quality) => {
                let html = html!(
                    script type="application/json" id="water-quality-data" {(
                    PreEscaped(
                            water_quality)
                    )}
                )
                .into();

                water_quality_tx.send(Ok(html)).await.unwrap();
            }
            Err(e) => {
                error!("Failed to load water quality: {e}");

                // If there's a water quality error, just hide the container. It's not
                // pivotal to the page.
                water_quality_tx
                    .send(Ok(html!(
                        script {
                            (PreEscaped("document.getElementById(\"water-quality-container\").classList.add(\"hidden\")"))
                        }
                    )
                    .into()))
                    .await
                    .unwrap();
            }
        }
    });

    tokio::spawn(async move {
        match Forecast::try_get_string(&spot, state).await {
            Ok(forecast) => {
                let html = html!
                    (
                        script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.3/dist/chart.umd.min.js" {}
                        script type="application/json" id="forecast-data" {(
                        PreEscaped(
                                forecast)
                        )}
                        // send completion div so JSON parsing of data isn't attempted until it's all
                        // there.
                        div id="forecast-complete" {}
                    )
                .into();

                tx.send(Ok(html)).await.unwrap();
            }
            Err(e) => {
                error!("Failed to load the forecast data: {e}");
                tx.send(Ok(html!((error_markup("forecast", e))).into()))
                    .await
                    .unwrap();
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
pub struct AppError(pub anyhow::Error);

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
