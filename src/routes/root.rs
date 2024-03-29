use crate::{AppState, TEMPLATES};
use anyhow::anyhow;
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

#[derive(Debug)]
struct WaveHeightData(Vec<WaveHeightForecast>);

impl WaveHeightData {
    fn get_data(&self) -> String {
        self.0
            .iter()
            .map(|data| format!("{:.3},", data.value * 3.281))
            .collect()
    }

    fn get_labels(&self) -> String {
        self.0
            .iter()
            .map(|data| format!("'{}',", data.valid_time))
            .collect()
    }
}

#[derive(Debug)]
struct WaveHeightForecast {
    value: f64,
    valid_time: String,
}

fn parse_hour(s: &str) -> anyhow::Result<usize> {
    if let Some((_, hour)) = s.split_once("T") {
        let hour = hour.strip_suffix("H\"").ok_or(anyhow!("no hour found!"))?;
        return Ok(hour.parse()?);
    };

    Err(anyhow!("no T found!"))
}

fn increment_time(t: &str, amount: usize) -> anyhow::Result<String> {
    let time = t.strip_prefix("\"").unwrap();
    let time: DateTime<Local> = chrono::DateTime::parse_from_str(time, "%+").unwrap().into();

    Ok((time + std::time::Duration::from_secs(amount as u64 * 3_600)).to_string())
}

impl WaveHeightForecast {
    fn expand_time_ranges(&self) -> anyhow::Result<Vec<Self>> {
        let (time, period) = self
            .valid_time
            .split_once("/P")
            .ok_or(anyhow!("Unknown period found!"))?;

        let mut total = 0;
        if let Some((day, hour)) = period.split_once("D") {
            total += day.parse::<usize>().unwrap() * 24;
            total += parse_hour(hour)?;
        } else {
            total += parse_hour(period)?;
        };

        let mut out = Vec::with_capacity(total);

        for i in 0..total {
            out.push(Self {
                value: self.value.clone(),
                valid_time: increment_time(time, i)?,
            })
        }

        Ok(out)
    }
}

impl From<serde_json::Value> for WaveHeightForecast {
    fn from(v: serde_json::Value) -> Self {
        let value = v.get("value").unwrap().as_f64().unwrap();
        let valid_time = v.get("validTime").unwrap().to_string();

        Self { value, valid_time }
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

    // milwauke = "https://api.weather.gov/gridpoints/MKX/91,67"
    let forecast = client
        .get("https://api.weather.gov/gridpoints/MKX/90,67")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let wave_heights = WaveHeightData(
        forecast
            .get("properties")
            .ok_or(anyhow!("no properties found!"))?
            .get("waveHeight")
            .ok_or(anyhow!("no wave heights found!"))?
            .get("values")
            .ok_or(anyhow!("no values found!"))?
            .as_array()
            .ok_or(anyhow!("array not found!"))?
            .clone()
            .into_iter()
            .map(WaveHeightForecast::from)
            .map(|w| WaveHeightForecast::expand_time_ranges(&w).unwrap())
            .flatten()
            .collect::<Vec<_>>(),
    );

    context.insert("title", &state.title);
    context.insert("as_of", &latest.as_of);
    context.insert("wind_direction", &latest.wind_direction);
    context.insert("wind_speed", &latest.wind_speed);
    context.insert("gusts", &latest.gusts);
    context.insert("wave_height_data", &wave_heights.get_data());
    context.insert("wave_height_labels", &wave_heights.get_labels());

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
