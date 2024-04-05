use crate::{utils, AppState, Latest, TEMPLATES};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Local, TimeZone, Utc};
use scraper::{Html, Selector};
use std::{cmp::Ordering, convert::Infallible, sync::Arc};
use tokio::sync::mpsc;

struct WaterTemp(String);

impl WaterTemp {
    async fn get() -> Self {
        let response = reqwest::get("https://seatemperature.net/current/united-states/milwaukee-wisconsin-united-states-sea-temperature")
            .await
            .unwrap();
        let html = Html::parse_document(&response.text().await.unwrap());
        let selector = Selector::parse("#temp1").unwrap();

        Self(
            html.select(&selector)
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap()
                .parse::<String>()
                .unwrap(),
        )
    }
}

struct Forecast {
    last_updated: String,
    wave_height: Vec<ForecastValue>,
    wave_period: Vec<ForecastValue>,
    wind_speed: Vec<ForecastValue>,
    wind_gust: Vec<ForecastValue>,
    wind_direction: Vec<ForecastValue>,
}

impl Forecast {
    async fn get() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("GatheringSurf/0.1 (+https://gathering.surf)")
            .build()
            .unwrap();

        // milwauke = "https://api.weather.gov/gridpoints/MKX/91,67"
        Ok(Forecast::from(
            client
                .get("https://api.weather.gov/gridpoints/MKX/90,67")
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?,
        ))
    }

    /// Condenses the forecast to even length vecs.
    ///
    /// Not all data from the api will cover the same length of
    /// time. This ensures no dangling data.
    ///
    /// TODO - Might not be necessary. Prone to future bug. No way
    /// to ensure all relevent fields are taken into account.
    fn condense(&mut self) {
        let lengths = [
            self.wave_period.len(),
            self.wave_height.len(),
            self.wind_speed.len(),
            self.wind_gust.len(),
            self.wind_direction.len(),
        ];

        let min = lengths.iter().min().unwrap();

        let _ = self.wave_height.split_off(*min);
        let _ = self.wind_speed.split_off(*min);
        let _ = self.wind_gust.split_off(*min);
        let _ = self.wind_direction.split_off(*min);
        let _ = self.wave_height.split_off(*min);
    }

    fn get_labels(&self) -> String {
        self.wave_height
            .iter()
            .map(|data| format!("'{}',", data.valid_time))
            .collect()
    }

    fn get_wave_data(&self) -> (String, u8) {
        let mut smoothed_data = Vec::new();
        let mut out = Vec::new();
        self.wave_height
            .iter()
            .for_each(|data| smoothed_data.push(utils::convert_meter_to_feet(data.value)));

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

    fn get_current_wave_height_and_period(&self) -> (String, String) {
        for (i, wave_height) in self.wave_height.iter().enumerate() {
            if DateTime::parse_from_str(&wave_height.valid_time, "%+").unwrap() > Local::now() {
                let height = utils::convert_meter_to_feet(wave_height.value);
                let period = self.wave_period.get(i).unwrap().value.to_string();

                if height < 1.5 {
                    return ("Flat".to_string(), period);
                }

                // Try to get range of current surf
                if let Some(last_hour) = self.wave_height.get(i - 1) {
                    let last_hour_height = utils::convert_meter_to_feet(last_hour.value);
                    return match height.partial_cmp(&last_hour_height) {
                        Some(Ordering::Less) => {
                            (format!("{:.0}-{:.0}", height, last_hour_height), period)
                        }
                        Some(Ordering::Greater) => {
                            (format!("{:.0}-{:.0}+", last_hour_height, height), period)
                        }
                        Some(Ordering::Equal) => (format!("{:.0}", height), period),
                        None => unreachable!("Found no ordering in wave heights."),
                    };
                }
                return (format!("{:.0}", height), period);
            }
        }

        ("Flat".to_string(), String::default())
    }
}

struct ForecastValue {
    value: f64,
    valid_time: String,
}

impl ForecastValue {
    fn get(properties: &serde_json::Value, key: &str) -> anyhow::Result<Vec<Self>> {
        Ok(properties
            .get(key)
            .ok_or(anyhow!("no {key} found!"))?
            .get("values")
            .ok_or(anyhow!("no values found!"))?
            .as_array()
            .ok_or(anyhow!("array not found!"))?
            .clone()
            .into_iter()
            .map(ForecastValue::from)
            .map(|w| ForecastValue::expand_time_ranges(&w).unwrap())
            .flatten()
            .collect())
    }

    fn expand_time_ranges(&self) -> anyhow::Result<Vec<Self>> {
        let (time, period) = self
            .valid_time
            .split_once("/P")
            .ok_or(anyhow!("Unknown period found!"))?;

        let mut total = 0;
        if let Some((day, hour)) = period.split_once("D") {
            total += day.parse::<usize>().unwrap() * 24;
            total += utils::parse_hour(hour).unwrap_or(0);
        } else {
            total += utils::parse_hour(period).unwrap_or(0);
        };

        let mut out = Vec::with_capacity(total);

        for i in 0..total {
            out.push(Self {
                value: self.value.clone(),
                valid_time: utils::increment_time(time, i)?,
            })
        }

        Ok(out)
    }
}

impl From<serde_json::Value> for Forecast {
    fn from(value: serde_json::Value) -> Self {
        let properties = value
            .get("properties")
            .ok_or(anyhow!("no properties found!"))
            .unwrap();

        let last_updated: DateTime<Utc> = DateTime::parse_from_str(
            properties.get("updateTime").unwrap().as_str().unwrap(),
            "%+",
        )
        .unwrap()
        .into();

        let last_updated: DateTime<Local> = DateTime::from(last_updated);
        let last_updated = last_updated
            .to_rfc2822()
            .strip_suffix(" -0500")
            .unwrap()
            .to_string();

        let wave_height = ForecastValue::get(properties, "waveHeight").unwrap();
        let wave_period = ForecastValue::get(properties, "wavePeriod").unwrap();
        let wind_speed = ForecastValue::get(properties, "windSpeed").unwrap();
        let wind_gust = ForecastValue::get(properties, "windGust").unwrap();
        let wind_direction = ForecastValue::get(properties, "windDirection").unwrap();

        Self {
            last_updated,
            wave_height,
            wave_period,
            wind_speed,
            wind_gust,
            wind_direction,
        }
    }
}

impl From<serde_json::Value> for ForecastValue {
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

        tx.send(Ok(TEMPLATES.render("base.html", &context).unwrap()))
            .await
            .unwrap();

        let latest = Latest::get().await.unwrap();
        if let Ok(mut forecast) = Forecast::get().await {
            forecast.condense();

            let (wave_height_data, graph_max) = forecast.get_wave_data();
            let (current_wave_height, current_wave_period) =
                forecast.get_current_wave_height_and_period();

            let water_temp = WaterTemp::get().await;

            context.insert("title", &state.title);
            context.insert("as_of", &latest.as_of);
            context.insert("wind_direction", &latest.wind_direction);
            context.insert(
                "wind",
                &format!(
                    "{}-{}",
                    utils::convert_meter_to_mile(&latest.wind_speed),
                    utils::convert_meter_to_mile(&latest.gusts)
                ),
            );
            context.insert("wave_height_data", &wave_height_data);
            context.insert("graph_max", &(graph_max + 2));
            context.insert("wave_height_labels", &forecast.get_labels());
            context.insert("current_wave_height", &current_wave_height);
            context.insert("current_wave_period", &current_wave_period);
            context.insert(
                "wind_icon_direction",
                &(latest.wind_direction.parse::<u32>().unwrap() + 180),
            );
            context.insert("forecast_as_of", &forecast.last_updated);
            context.insert("water_temp", &water_temp.0);

            tx.send(Ok(TEMPLATES.render("index.html", &context).unwrap()))
                .await
                .unwrap();
        } else {
            tx.send(Ok(TEMPLATES.render("error.html", &context).unwrap()))
                .await
                .unwrap();
        }
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
