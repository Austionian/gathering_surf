use crate::{AppState, TEMPLATES};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Local, TimeZone};
use std::sync::Arc;

struct Latest {
    as_of: String,
    wind_direction: String,
    wind_speed: String,
    gusts: String,
}

impl Latest {
    async fn get() -> anyhow::Result<Self> {
        let data = reqwest::get("https://www.ndbc.noaa.gov/data/realtime2/MLWW3.txt")
            .await?
            .text()
            .await?;

        let latest = data.lines().collect::<Vec<_>>();
        let latest = latest.get(2).unwrap();

        let (as_of, measurements) = latest.split_at(16);

        let as_of = Self::parse_as_of(as_of)?;
        let mut measurements = measurements.trim().split_whitespace();
        let wind_direction = measurements.next().unwrap().to_string();

        let wind_speed = measurements.next().unwrap().to_string();
        let gusts = measurements.next().unwrap().to_string();

        Ok(Self {
            as_of,
            wind_direction,
            wind_speed,
            gusts,
        })
    }

    fn parse_as_of(as_of: &str) -> anyhow::Result<String> {
        let as_of = as_of.trim().split(" ").collect::<Vec<_>>();
        let as_of: DateTime<Local> = chrono::Utc
            .with_ymd_and_hms(
                as_of.get(0).unwrap().parse::<i32>().unwrap(),
                as_of.get(1).unwrap().parse::<u32>().unwrap(),
                as_of.get(2).unwrap().parse::<u32>().unwrap(),
                as_of.get(3).unwrap().parse::<u32>().unwrap(),
                as_of.get(4).unwrap().parse::<u32>().unwrap(),
                00,
            )
            .unwrap()
            .try_into()
            .unwrap();

        let as_of = as_of.to_rfc2822();

        Ok(as_of.split(" -").next().unwrap().to_string())
    }
}

/// Handler to return the website's index
pub async fn root(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let mut context = tera::Context::new();

    let latest = Latest::get().await?;

    let client = reqwest::Client::builder()
        .user_agent("GatheringSurf/0.1 (+https://gathering.surf)")
        .build()
        .unwrap();

    let forecast = client
        .get("https://api.weather.gov/gridpoints/MKX/91,67")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!(
        "{:?}",
        forecast
            .get("properties")
            .unwrap()
            .get("waveHeight")
            .unwrap()
            .get("values")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .get("value")
            .unwrap()
            .as_f64()
            .unwrap()
    );

    context.insert("title", &state.title);
    context.insert("as_of", &latest.as_of);
    context.insert("wind_direction", &latest.wind_direction);
    context.insert("wind_speed", &latest.wind_speed);
    context.insert("gusts", &latest.gusts);

    match TEMPLATES.render("index.html", &context) {
        Ok(s) => Ok(Html(s)),
        Err(_) => Ok(Html("<html><body>Error</body></html>".to_string())),
    }
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
