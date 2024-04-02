use crate::{AppState, TEMPLATES};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Local, TimeZone, Utc};
use std::{cmp::Ordering, convert::Infallible, sync::Arc};
use tokio::sync::mpsc;

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
        let as_of: DateTime<Local> = Utc
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

#[derive(Debug, Default)]
struct WaveHeightData(Vec<WaveHeightForecast>);

fn convert_meter_to_feet(value: f64) -> f64 {
    value * 3.281
}

impl WaveHeightData {
    fn get_data(&self) -> (String, u8) {
        let mut smoothed_data = Vec::new();
        let mut out = Vec::new();
        self.0
            .iter()
            .for_each(|data| smoothed_data.push(convert_meter_to_feet(data.value)));

        smoothed_data.windows(3).for_each(|window| match window {
            [x, y, z] => out.push((x + y + z) / 3.0),
            [x, y] => out.push((x + y) / 2.0),
            [x] => out.push(*x),
            _ => panic!("what dafuq is this?"),
        });

        (
            out.iter().map(|value| format!("{:.3},", value)).collect(),
            smoothed_data.iter().map(|v| v.clone() as u8).max().unwrap(),
        )
    }

    fn get_labels(&self) -> String {
        self.0
            .iter()
            .map(|data| format!("'{}',", data.valid_time))
            .collect()
    }

    fn get_current_wave_height(&self) -> String {
        for (i, wave_height) in self.0.iter().enumerate() {
            if DateTime::parse_from_str(&wave_height.valid_time, "%+").unwrap() > Local::now() {
                let height = convert_meter_to_feet(wave_height.value);
                if height < 1.5 {
                    return "Flat".to_string();
                }
                // Try to get range of current surf
                if let Some(last_hour) = self.0.get(i - 1) {
                    let last_hour_height = convert_meter_to_feet(last_hour.value);
                    return match height.partial_cmp(&last_hour_height) {
                        Some(Ordering::Less) => format!("{:.0}-{:.0}", height, last_hour_height),
                        Some(Ordering::Greater) => format!("{:.0}-{:.0}", last_hour_height, height),
                        Some(Ordering::Equal) => format!("{:.0}", height),
                        None => unreachable!("Found no ordering in wave heights."),
                    };
                }
                return format!("{:.0}", height);
            }
        }

        "Flat".to_string()
    }
}

#[derive(Debug, Default)]
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
    let time: DateTime<Local> = DateTime::parse_from_str(time, "%+").unwrap().into();

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
pub async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);

    tokio::spawn(async move {
        let mut context = tera::Context::new();
        let mut buffer_beater = Vec::with_capacity(5000);
        buffer_beater.fill(" ");
        tx.send(Ok(buffer_beater
            .iter()
            .map(|v| v.to_string())
            .collect::<String>()))
            .await
            .unwrap();
        tx.send(Ok(TEMPLATES.render("base.html", &context).unwrap()))
            .await
            .unwrap();

        let latest = Latest::get().await.unwrap();

        let client = reqwest::Client::builder()
            .user_agent("GatheringSurf/0.1 (+https://gathering.surf)")
            .build()
            .unwrap();

        // milwauke = "https://api.weather.gov/gridpoints/MKX/91,67"
        let forecast = client
            .get("https://api.weather.gov/gridpoints/MKX/90,67")
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        let wave_heights = WaveHeightData(
            forecast
                .get("properties")
                .ok_or(anyhow!("no properties found!"))
                .unwrap()
                .get("waveHeight")
                .ok_or(anyhow!("no wave heights found!"))
                .unwrap()
                .get("values")
                .ok_or(anyhow!("no values found!"))
                .unwrap()
                .as_array()
                .ok_or(anyhow!("array not found!"))
                .unwrap()
                .clone()
                .into_iter()
                .map(WaveHeightForecast::from)
                .map(|w| WaveHeightForecast::expand_time_ranges(&w).unwrap())
                .flatten()
                .collect::<Vec<_>>(),
        );

        let (wave_height_data, graph_max) = wave_heights.get_data();

        context.insert("title", &state.title);
        context.insert("as_of", &latest.as_of);
        context.insert("wind_direction", &latest.wind_direction);
        context.insert("wind_speed", &latest.wind_speed);
        context.insert("gusts", &latest.gusts);
        context.insert("wave_height_data", &wave_height_data);
        context.insert("graph_max", &(graph_max + 2));
        context.insert("wave_height_labels", &wave_heights.get_labels());
        context.insert(
            "current_wave_height",
            &wave_heights.get_current_wave_height(),
        );
        tx.send(Ok(TEMPLATES.render("index.html", &context).unwrap()))
            .await
            .unwrap();
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=UTF-8")
        .header("X-Content-Type-Options", "nosniff")
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
