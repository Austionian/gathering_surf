use super::{Location, Spot};
use crate::{convert_celsius_to_fahrenheit, convert_kilo_meter_to_mile, utils};

use anyhow::{anyhow, bail};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::US::Central;
use reqwest::Response;
use std::cmp::Ordering;
use tracing::{error, info, warn};

#[derive(serde::Serialize)]
pub struct Forecast {
    pub last_updated: String,
    pub probability_of_precipitation: Vec<u8>,
    pub quality: Option<Vec<String>>,
    pub temperature: Vec<u8>,
    pub wave_height: Vec<f64>,
    pub wave_direction: Vec<f64>,
    pub wave_period: Vec<f64>,
    pub wind_speed: Vec<f64>,
    pub wind_gust: Vec<f64>,
    pub wind_direction: Vec<f64>,
    pub dewpoint: Vec<String>,
    pub cloud_cover: Vec<u8>,
    pub probability_of_thunder: Vec<u8>,
    pub starting_at: String,
    pub waves: Option<Vec<f64>>,
    pub wave_height_labels: Vec<String>,
    pub current_wave_height: f64,
    pub current_wave_period: f64,
    pub current_wave_direction: f64,
}

impl Forecast {
    pub async fn try_get(spot: &Spot, forecast_url: &str) -> anyhow::Result<Self> {
        let data = Self::fetch_data(spot.forecast_path, forecast_url).await?;

        let mut forecast: Self = (data.json::<serde_json::Value>().await?).try_into()?;

        forecast.compute_and_condense(&spot.location);

        Ok(forecast)
    }

    async fn fetch_data(forecast_path: &str, forecast_url: &str) -> anyhow::Result<Response> {
        let client = reqwest::Client::builder()
            .user_agent("GatheringSurf/0.1 (+https://gathering.surf)")
            .build()
            .unwrap();

        const RETRY: u8 = 2;
        for _ in 0..RETRY {
            let response = client
                .get(format!("{}{}", forecast_url, forecast_path))
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await?;
            if response.status().as_u16() == 200 {
                info!("NOAA 200 success.");
                return Ok(response);
            }
            warn!("NOAA non-200, retrying.");
        }

        error!("Non 200 response from NOAA");
        bail!("Non 200 response from NOAA");
    }

    fn compute_and_condense(&mut self, location: &Location) {
        self.condense();
        self.get_wave_data(location);
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

    fn get_wave_data(&mut self, location: &Location) {
        let mut smoothed_data = Vec::with_capacity(self.wave_height.len());
        let mut out = Vec::with_capacity(self.wave_height.len());
        self.wave_height
            .iter()
            .for_each(|data| smoothed_data.push(utils::convert_meter_to_feet(*data)));

        smoothed_data.windows(3).for_each(|window| match window {
            [x, y, z] => out.push((x + y + z) / 3.0),
            [x, y] => out.push((x + y) / 2.0),
            [x] => out.push(*x),
            _ => panic!("what dafuq is this?"),
        });

        for (forecast, computed) in self.wave_height.iter_mut().zip(out.iter()) {
            *forecast = truncate_to_two_decimals(*computed);
        }

        self.compute_quality(location);
    }

    /// Returns the wave height, period and direction from the forecasted
    /// data relative to the time of request.
    // fn get_current_wave_data(&self) -> (String, String, String) {
    //     for (i, wave_height) in self.wave_height.iter().enumerate() {
    //         if DateTime::parse_from_str(&wave_height.valid_time, "%+").unwrap() > Utc::now() {
    //             let height = wave_height.value as u8;
    //             let period = self.wave_period.get(i).unwrap().value.to_string();
    //             let direction = self.wave_direction.get(i).unwrap().value.to_string();
    //
    //             // Try to get range of current surf
    //             if let Some(last_hour) = self.wave_height.get(i - 1) {
    //                 let last_hour_height = last_hour.value as u8;
    //                 return match height.partial_cmp(&last_hour_height) {
    //                     Some(Ordering::Less) => (
    //                         format!("{:.0}-{:.0}", height, last_hour_height),
    //                         period,
    //                         direction,
    //                     ),
    //                     Some(Ordering::Greater) => (
    //                         format!("{:.0}-{:.0}+", last_hour_height, height),
    //                         period,
    //                         direction,
    //                     ),
    //                     Some(Ordering::Equal) => (format!("{:.0}", height), period, direction),
    //                     None => unreachable!("Found no ordering in wave heights."),
    //                 };
    //             }
    //             return (format!("{:.0}", height), period, direction);
    //         }
    //     }
    //
    //     ("0".to_string(), String::default(), String::default())
    // }
    //
    pub fn compute_quality(&mut self, location: &Location) {
        let mut qualities = Vec::with_capacity(self.wind_direction.len());
        for ((wind_direction, wind_speed), wave_height) in self
            .wind_direction
            .iter()
            .zip(self.wind_speed.iter())
            .zip(self.wave_height.iter())
        {
            qualities.push(
                location
                    .get_quality(*wave_height, *wind_speed, *wind_direction)
                    .1
                    .to_string(),
            );
        }

        self.quality = Some(qualities)
    }
}

impl TryFrom<serde_json::Value> for Forecast {
    type Error = anyhow::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let properties = value
            .get("properties")
            .ok_or(anyhow!("no properties found!"))?;

        let last_updated = properties
            .get("updateTime")
            .ok_or(anyhow!("no updateTime found"))?
            .as_str()
            .ok_or(anyhow!("string not found"))?
            .strip_suffix("+00:00")
            .ok_or(anyhow!("Unidentified suffix"))?
            .parse::<NaiveDateTime>()?;

        let last_updated = Central
            .from_utc_datetime(&last_updated)
            .to_rfc2822()
            .strip_suffix(" -0500")
            .ok_or(anyhow!("Unidentified suffix"))?
            .to_string();

        let wave_height = try_from_value(properties, "waveHeight", &no_op)?;
        let wave_period = try_from_value(properties, "wavePeriod", &no_op)?;
        let wave_direction = try_from_value(properties, "waveDirection", &no_op)?;
        let wind_speed = try_from_value(properties, "windSpeed", &convert_wind_speed)?;
        let wind_gust = try_from_value(properties, "windGust", &convert_gust)?;
        let wind_direction = try_from_value(properties, "windDirection", &convert_wind_direction)?;
        let temperature = try_from_value(properties, "temperature", &convert_temperature)?;
        let probability_of_precipitation =
            try_from_value(properties, "probabilityOfPrecipitation", &convert_to_u8)?;
        let dewpoint = try_from_value(properties, "dewpoint", &convert_dewpoint)?;
        let cloud_cover = try_from_value(properties, "skyCover", &convert_to_u8)?;
        let probability_of_thunder =
            try_from_value(properties, "probabilityOfThunder", &convert_to_u8)?;
        let wave_height_labels = try_labels_from_value(properties, "waveHeight")?;

        let starting_at = DateTime::parse_from_str(
            properties
                .get("validTimes")
                .unwrap()
                .to_string()
                .split_once('/')
                .unwrap()
                .0
                .strip_prefix('"')
                .unwrap(),
            "%+",
        )
        .unwrap()
        .to_string();

        let current_time_index = get_current_time_index(properties);

        let current_wave_height = wave_height[current_time_index as usize];
        let current_wave_period = wave_period[current_time_index as usize];
        let current_wave_direction = wave_direction[current_time_index as usize];

        Ok(Self {
            last_updated,
            wave_height,
            wave_direction,
            wave_period,
            wind_speed,
            wind_gust,
            wind_direction,
            quality: None,
            temperature,
            probability_of_precipitation,
            dewpoint,
            cloud_cover,
            probability_of_thunder,
            starting_at,
            waves: None,
            wave_height_labels,
            current_wave_period,
            current_wave_height,
            current_wave_direction,
        })
    }
}

fn get_current_time_index(value: &serde_json::Value) -> u8 {
    let diff = DateTime::parse_from_str(
        value
            .get("validTimes")
            .unwrap()
            .to_string()
            .split_once('/')
            .unwrap()
            .0
            .strip_prefix('"')
            .unwrap(),
        "%+",
    )
    .unwrap()
    .time()
        - Utc::now().time();

    diff.num_hours().try_into().unwrap()
}

fn no_op(v: f64) -> f64 {
    v
}

fn convert_to_u8(v: f64) -> u8 {
    v as u8
}

fn convert_wind_speed(v: f64) -> f64 {
    truncate_to_two_decimals(convert_kilo_meter_to_mile(v))
}

fn convert_wind_direction(v: f64) -> f64 {
    truncate_to_two_decimals(v)
}

fn convert_gust(v: f64) -> f64 {
    truncate_to_two_decimals(convert_kilo_meter_to_mile(v))
}

fn convert_temperature(v: f64) -> u8 {
    convert_celsius_to_fahrenheit(v).parse().unwrap()
}

fn convert_dewpoint(v: f64) -> String {
    convert_celsius_to_fahrenheit(v)
}

fn try_from_value<T>(
    properties: &serde_json::Value,
    key: &str,
    f: &dyn Fn(f64) -> T,
) -> anyhow::Result<Vec<T>> {
    Ok(properties
        .get(key)
        .ok_or(anyhow!("no {key} found!"))?
        .get("values")
        .ok_or(anyhow!("no values found!"))?
        .as_array()
        .ok_or(anyhow!("array not found!"))?
        .clone()
        .into_iter()
        .flat_map(|value| convert(&value, f).unwrap())
        .collect())
}

fn get_value_and_time(v: &serde_json::Value) -> anyhow::Result<(f64, String)> {
    let value = v
        .get("value")
        .ok_or(anyhow!("No value found."))?
        .as_f64()
        .ok_or(anyhow!("Not an f64"))?;
    let valid_time = v
        .get("validTime")
        .ok_or(anyhow!("No validTime found."))?
        .to_string();

    Ok((value, valid_time))
}

fn convert<T>(v: &serde_json::Value, f: &dyn Fn(f64) -> T) -> anyhow::Result<Vec<T>> {
    let (value, valid_time) = get_value_and_time(v)?;

    let (_, period) = valid_time
        .split_once("/P")
        .ok_or(anyhow!("Unknown period found!"))?;

    let mut total = 0;
    if let Some((day, hour)) = period.split_once('D') {
        total += day.parse::<usize>().unwrap() * 24;
        total += utils::parse_hour(hour).unwrap_or(0);
    } else {
        total += utils::parse_hour(period).unwrap_or(0);
    };

    let mut out = Vec::with_capacity(total);

    for _ in 0..total {
        out.push(f(value))
    }

    Ok(out)
}
/// Limits the f64 to two decimal points
fn truncate_to_two_decimals(v: f64) -> f64 {
    (v * 100.0).trunc() / 100.0
}

// Try to write this better !
fn try_labels_from_value(properties: &serde_json::Value, key: &str) -> anyhow::Result<Vec<String>> {
    Ok(properties
        .get(key)
        .ok_or(anyhow!("no {key} found!"))?
        .get("values")
        .ok_or(anyhow!("no values found!"))?
        .as_array()
        .ok_or(anyhow!("array not found!"))?
        .clone()
        .into_iter()
        .flat_map(|value| get_time(&value).unwrap())
        .collect())
}

fn get_time(v: &serde_json::Value) -> anyhow::Result<Vec<String>> {
    let (_, valid_time) = get_value_and_time(v)?;

    let (time, period) = valid_time
        .split_once("/P")
        .ok_or(anyhow!("Unknown period found!"))?;

    let mut total = 0;
    if let Some((day, hour)) = period.split_once('D') {
        total += day.parse::<usize>().unwrap() * 24;
        total += utils::parse_hour(hour).unwrap_or(0);
    } else {
        total += utils::parse_hour(period).unwrap_or(0);
    };

    let mut out = Vec::with_capacity(total);

    for i in 0..total {
        let (_, display_time) = utils::increment_time(time, i)?;
        out.push(display_time.unwrap());
    }

    Ok(out)
}
