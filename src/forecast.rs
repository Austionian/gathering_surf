use std::cmp::Ordering;

use super::{Location, Spot};
use crate::utils::*;

use anyhow::{anyhow, bail};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::US::Central;
use reqwest::Response;
use tracing::{error, info, warn};

#[derive(serde::Serialize)]
pub struct Forecast {
    pub as_of: String,
    pub cloud_cover: Vec<u8>,
    pub current_wave_height: String,
    pub current_wave_period: f64,
    pub current_wave_direction: f64,
    pub dewpoint: Vec<String>,
    pub probability_of_precipitation: Vec<u8>,
    pub probability_of_thunder: Vec<u8>,
    pub quality: Option<Vec<String>>,
    pub starting_at: String,
    pub temperature: Vec<u8>,
    #[serde(skip_serializing)]
    pub wave_direction: Vec<f64>,
    pub wave_height: Vec<f64>,
    pub wave_height_labels: Vec<String>,
    pub wave_period: Vec<f64>,
    pub wind_speed: Vec<f64>,
    pub wind_gust: Vec<f64>,
    pub wind_direction: Vec<f64>,
}

impl Forecast {
    pub async fn try_get(spot: &Spot, forecast_url: &str) -> anyhow::Result<Self> {
        let data = Self::fetch_data(spot.forecast_path, forecast_url).await?;

        let mut forecast: Self = (data.json::<serde_json::Value>().await?).try_into()?;

        forecast.condense();
        forecast.compute_quality(&spot.location);

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

    /// Condenses the forecast to equal length vecs.
    ///
    /// Not all data from the api will cover the same length of
    /// time. This ensures no dangling data.
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
        let _ = self.wave_height_labels.split_off(*min);
    }

    /// Takes the relative attributes and computes their quality
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

    /// Smooths the wave data by taking the average of three data points, turns data
    /// from something like [0,0,1,2] into [.33, 1, 1.5, 2] to better show growing wave heights.
    fn smooth_wave_data(wave_height: &[f64]) -> Vec<f64> {
        let mut smoothed_data = Vec::with_capacity(wave_height.len());
        let mut out = Vec::with_capacity(wave_height.len());
        wave_height
            .iter()
            .for_each(|data| smoothed_data.push(convert_meter_to_feet(*data)));

        smoothed_data.windows(3).for_each(|window| match window {
            [x, y, z] => out.push(truncate_to_two_decimals((x + y + z) / 3.0)),
            [x, y] => out.push(truncate_to_two_decimals((x + y) / 2.0)),
            [x] => out.push(truncate_to_two_decimals(*x)),
            _ => panic!("what dafuq is this?"),
        });

        out
    }

    /// Returns the wave height, period and direction from the forecasted
    /// data relative to the time of request.
    fn get_current_wave_data(
        wave_height: &[f64],
        wave_period: &[f64],
        wave_direction: &[f64],
        starting_at: &str,
    ) -> anyhow::Result<(String, f64, f64)> {
        // Required for unit tests to have a consistent as of time
        #[cfg(not(feature = "mock-time"))]
        let current_time_index = Self::get_current_time_index(starting_at)? + 1;
        #[cfg(feature = "mock-time")]
        let current_time_index = 1;

        if current_time_index > wave_height.len() {
            bail!("Invalid accessing index found!");
        }

        let height = *wave_height.get(current_time_index).unwrap() as u8;
        let period = wave_period.get(current_time_index).unwrap();
        let direction = wave_direction.get(current_time_index).unwrap() + 180.0;

        // Try to get range of current surf
        if let Some(last_hour) = wave_height.get(current_time_index - 1) {
            let last_hour = *last_hour as u8;
            return match height.partial_cmp(&last_hour) {
                Some(Ordering::Less) => Ok((
                    format!("{:.0}-{:.0}", height, last_hour),
                    *period,
                    direction,
                )),
                Some(Ordering::Greater) => Ok((
                    format!("{:.0}-{:.0}+", last_hour, height),
                    *period,
                    direction,
                )),
                Some(Ordering::Equal) => Ok((format!("{:.0}", height), *period, direction)),
                None => unreachable!("Found no ordering in wave heights."),
            };
        }

        Ok((format!("{:.0}", height), *period, direction))
    }

    fn get_current_time_index(starting_at: &str) -> anyhow::Result<usize> {
        Ok((Utc::now()
            - DateTime::parse_from_str(starting_at, "%+")
                .unwrap()
                .with_timezone(&Utc))
        .num_hours()
        .try_into()?)
    }

    fn try_from_value<T: std::clone::Clone>(
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
            .flat_map(|value| Self::expand_and_convert(&value, f).unwrap())
            .collect())
    }

    fn expand_and_convert<T: std::clone::Clone>(
        v: &serde_json::Value,
        f: &dyn Fn(f64) -> T,
    ) -> anyhow::Result<Vec<T>> {
        let (value, valid_time) = Self::get_value_and_time(v)?;

        let (_, period) = valid_time
            .split_once("/P")
            .ok_or(anyhow!("Unknown period found!"))?;

        let period_len = Self::parse_period(period);

        Ok(vec![f(value); period_len])
    }

    /// Extracts the value and it's time attribute from the object,
    /// e.g. {
    ///     "value": 5.3,
    ///     "validTime": "2024-07-18T05:00:00+00:00/P1D5H"
    /// } -> (5.3, "2024-07-18T05:00:00+00:00/P1D5H")
    fn get_value_and_time(v: &serde_json::Value) -> anyhow::Result<(f64, &str)> {
        let value = v
            .get("value")
            .ok_or(anyhow!("No value found."))?
            .as_f64()
            .ok_or(anyhow!("Not an f64"))?;
        let valid_time = v
            .get("validTime")
            .ok_or(anyhow!("No validTime found."))?
            .as_str()
            .unwrap();

        Ok((value, valid_time))
    }

    /// Parses the period into its length in hours,
    /// e.g. 1DT5H -> 29
    fn parse_period(period: &str) -> usize {
        let mut period_len = 0;

        if let Some((day, hour)) = period.split_once('D') {
            period_len += day.parse::<usize>().unwrap() * 24;
            period_len += parse_hour(hour).unwrap_or(0);
        } else {
            period_len += parse_hour(period).unwrap_or(0);
        };

        period_len
    }

    // Try to write this better !
    /// Expands and creates the time labels from the wave_height data
    /// in the JSON. e.g. ["Fri 10 AM", "Fri 11 AM", ...]
    fn try_labels(properties: &serde_json::Value) -> anyhow::Result<Vec<String>> {
        Ok(properties
            .get("waveHeight")
            .ok_or(anyhow!("no waveHeight found!"))?
            .get("values")
            .ok_or(anyhow!("no values found!"))?
            .as_array()
            .ok_or(anyhow!("array not found!"))?
            .clone()
            .into_iter()
            .flat_map(|value| {
                let (_, valid_time) = Self::get_value_and_time(&value).unwrap();

                let (time, period) = valid_time
                    .split_once("/P")
                    .ok_or(anyhow!("Unknown period found!"))
                    .unwrap();

                let period_len = Self::parse_period(period);
                let mut labels = Vec::with_capacity(period_len);

                for i in 0..period_len {
                    labels.push(increment_time(time, i).unwrap());
                }

                labels
            })
            .collect())
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

        let as_of = Central
            .from_utc_datetime(&last_updated)
            .to_rfc2822()
            .strip_suffix(" -0500")
            .ok_or(anyhow!("Unidentified suffix"))?
            .to_string();

        let wave_height =
            Self::smooth_wave_data(&Self::try_from_value(properties, "waveHeight", &|v| v)?);

        let wave_period = Self::try_from_value(properties, "wavePeriod", &|v| v)?;
        let wave_direction = Self::try_from_value(properties, "waveDirection", &|v| v)?;
        let wind_speed = Self::try_from_value(properties, "windSpeed", &|v| {
            truncate_to_two_decimals(convert_kilo_meter_to_mile(v))
        })?;
        let wind_gust = Self::try_from_value(properties, "windGust", &|v| {
            truncate_to_two_decimals(convert_kilo_meter_to_mile(v))
        })?;
        let wind_direction = Self::try_from_value(properties, "windDirection", &|v| {
            truncate_to_two_decimals(v)
        })?;
        let temperature = Self::try_from_value(properties, "temperature", &|v| {
            convert_celsius_to_fahrenheit(v).parse::<u8>().unwrap()
        })?;
        let probability_of_precipitation =
            Self::try_from_value(properties, "probabilityOfPrecipitation", &|v| v as u8)?;
        let dewpoint = Self::try_from_value(properties, "dewpoint", &|v| {
            convert_celsius_to_fahrenheit(v)
        })?;
        let cloud_cover = Self::try_from_value(properties, "skyCover", &|v| v as u8)?;
        let probability_of_thunder =
            Self::try_from_value(properties, "probabilityOfThunder", &|v| v as u8)?;
        let wave_height_labels = Self::try_labels(properties)?;

        let starting_at = properties
            .get("validTimes")
            .unwrap()
            .as_str()
            .unwrap()
            .split_once('/')
            .unwrap()
            .0;

        let (current_wave_height, current_wave_period, current_wave_direction) =
            Self::get_current_wave_data(&wave_height, &wave_period, &wave_direction, starting_at)?;

        Ok(Self {
            as_of,
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
            starting_at: starting_at.to_string(),
            wave_height_labels,
            current_wave_period,
            current_wave_height,
            current_wave_direction,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_period_parses_an_hours_only_string() {
        assert_eq!(Forecast::parse_period("T2H"), 2);
    }
    #[test]
    fn parse_period_parses_an_hours_and_days_string() {
        assert_eq!(Forecast::parse_period("2DT10H"), 58);
    }
    #[test]
    fn parse_period_parses_an_empty_string() {
        assert_eq!(Forecast::parse_period(""), 0);
    }

    #[test]
    fn get_value_and_time_gets_values() {
        assert_eq!(
            Forecast::get_value_and_time(&serde_json::json!({
                "value": 81.15151,
                "validTime": "2024-09-06T11:00:00+00:00/PT1H",
            }))
            .unwrap(),
            (81.15151, "2024-09-06T11:00:00+00:00/PT1H")
        )
    }
}
