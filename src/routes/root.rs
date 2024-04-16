use crate::{quality, AppState, Forecast, Latest, TEMPLATES};
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;

/// Handler to return the website's index
pub async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Create a channel to stream content to client as we get it
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);

    tokio::spawn(async move {
        let mut context = tera::Context::new();

        tx.send(Ok(TEMPLATES.render("base.html", &context).unwrap()))
            .await
            .unwrap();

        match Latest::try_get().await {
            Ok(latest) => {
                context.insert("as_of", &latest.as_of);
                context.insert("wind_direction", &latest.wind_direction);
                context.insert("wind", &latest.get_wind_data());
                context.insert("wind_icon_direction", &(latest.wind_direction + 180));
                context.insert("current_water_temp", &latest.water_temp);
                context.insert("current_air_temp", &latest.air_temp);
                context.insert(
                    "wave_quality_text",
                    quality::get_quality(
                        latest.wind_speed.parse().unwrap(),
                        latest.wind_direction as f64,
                    )
                    .0,
                );
                context.insert(
                    "wave_quality",
                    quality::get_quality(
                        latest.wind_speed.parse().unwrap(),
                        latest.wind_direction as f64,
                    )
                    .1,
                );

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

        match Forecast::try_get().await {
            Ok(mut forecast) => {
                forecast.condense();
                forecast.get_quality();

                let (wave_height_data, graph_max) = forecast.get_wave_data();
                let (current_wave_height, current_wave_period, current_wave_direction) =
                    forecast.get_current_wave_data();

                context.insert("title", &state.title);
                context.insert("wave_height_data", &wave_height_data);
                context.insert("wind_speed_data", &forecast.get_wind_data());
                context.insert("wind_direction_data", &forecast.get_wind_direction_data());
                context.insert("wind_gust_data", &forecast.get_wind_gust_data());
                context.insert("wave_period_data", &forecast.get_wave_period_data());
                context.insert("graph_max", &(graph_max + 2));
                context.insert("wave_height_labels", &forecast.get_labels());
                context.insert("current_wave_height", &current_wave_height);
                context.insert("current_wave_period", &current_wave_period);
                context.insert(
                    "current_wave_direction",
                    &(current_wave_direction.parse::<u32>().unwrap() + 180),
                );
                context.insert("forecast_as_of", &forecast.last_updated);
                context.insert("qualities", &forecast.quality.unwrap());

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
