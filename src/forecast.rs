use crate::{convert_kilo_meter_to_mile, quality, utils};
use anyhow::{anyhow, bail};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::US::Central;
use std::cmp::Ordering;
use std::fmt::Write;

#[derive(serde::Serialize)]
pub struct Forecast {
    pub last_updated: String,
    // probability_of_precipitation: Vec<ForecastValue>,
    pub quality: Option<Vec<String>>,
    // temperature: Vec<ForecastValue>,
    pub wave_height: Vec<ForecastValue>,
    pub wave_period: Vec<ForecastValue>,
    pub wave_direction: Vec<ForecastValue>,
    pub wind_speed: Vec<ForecastValue>,
    pub wind_gust: Vec<ForecastValue>,
    pub wind_direction: Vec<ForecastValue>,
}

impl Forecast {
    pub async fn try_get() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("GatheringSurf/0.1 (+https://gathering.surf)")
            .build()
            .unwrap();

        // milwauke = "https://api.weather.gov/gridpoints/MKX/91,67"
        let response = client
            .get("https://api.weather.gov/gridpoints/MKX/90,67")
            .send()
            .await?;

        if response.status().as_u16() != 200 {
            bail!("Non 200 response from NOAA");
        }

        Forecast::try_from(response.json::<serde_json::Value>().await?)
    }

    /// Condenses the forecast to even length vecs.
    ///
    /// Not all data from the api will cover the same length of
    /// time. This ensures no dangling data.
    ///
    /// TODO - Might not be necessary. Prone to future bug. No way
    /// to ensure all relevent fields are taken into account.
    pub fn condense(&mut self) {
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

    pub fn get_labels(&mut self) -> String {
        self.wave_height
            .iter_mut()
            .fold(String::new(), |mut acc, data| {
                let _ = write!(acc, "'{}',", data.display_time.take().unwrap());
                acc
            })
    }

    pub fn get_wave_data(&self) -> (String, u8) {
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
            out.iter().fold(String::new(), |mut acc, value| {
                let _ = write!(acc, "{:.3},", value);
                acc
            }),
            smoothed_data.iter().map(|v| *v as u8).max().unwrap(),
        )
    }

    pub fn get_wind_data(&self) -> String {
        self.wind_speed.iter().fold(String::new(), |mut acc, v| {
            let _ = write!(acc, "{:.2},", convert_kilo_meter_to_mile(v.value));
            acc
        })
    }

    pub fn get_wind_direction_data(&self) -> String {
        self.wind_direction
            .iter()
            .fold(String::new(), |mut acc, v| {
                let _ = write!(acc, "{:.2},", v.value + 180.0);
                acc
            })
    }

    pub fn get_wind_gust_data(&self) -> String {
        self.wind_gust.iter().fold(String::new(), |mut acc, v| {
            let _ = write!(acc, "{:.2},", convert_kilo_meter_to_mile(v.value));
            acc
        })
    }

    pub fn get_wave_period_data(&self) -> String {
        self.wave_period.iter().fold(String::new(), |mut acc, v| {
            let _ = write!(acc, "{},", v.value);
            acc
        })
    }

    /// Returns the wave height, period and direction from the forecasted
    /// data relative to the time of request.
    pub fn get_current_wave_data(&self) -> (String, String, String) {
        for (i, wave_height) in self.wave_height.iter().enumerate() {
            if DateTime::parse_from_str(&wave_height.valid_time, "%+").unwrap() > Utc::now() {
                let height = utils::convert_meter_to_feet(wave_height.value);
                let period = self.wave_period.get(i).unwrap().value.to_string();
                let direction = self.wave_direction.get(i).unwrap().value.to_string();

                if height < 1.0 {
                    return ("Flat".to_string(), period, direction);
                }

                // Try to get range of current surf
                if let Some(last_hour) = self.wave_height.get(i - 1) {
                    let last_hour_height = utils::convert_meter_to_feet(last_hour.value);
                    return match height.partial_cmp(&last_hour_height) {
                        Some(Ordering::Less) => (
                            format!("{:.0}-{:.0}", height, last_hour_height),
                            period,
                            direction,
                        ),
                        Some(Ordering::Greater) => (
                            format!("{:.0}-{:.0}+", last_hour_height, height),
                            period,
                            direction,
                        ),
                        Some(Ordering::Equal) => (format!("{:.0}", height), period, direction),
                        None => unreachable!("Found no ordering in wave heights."),
                    };
                }
                return (format!("{:.0}", height), period, direction);
            }
        }

        ("Flat".to_string(), String::default(), String::default())
    }

    pub fn get_quality(&mut self) {
        let mut qualities = Vec::with_capacity(self.wind_direction.len());
        for (wind_direction, wind_speed) in self.wind_direction.iter().zip(self.wind_speed.iter()) {
            qualities.push(format!(
                "'{}'",
                quality::get_quality(wind_speed.value, wind_direction.value).1
            ));
        }

        self.quality = Some(qualities)
    }
}

#[derive(serde::Serialize)]
pub struct ForecastValue {
    value: f64,
    valid_time: String,
    display_time: Option<String>,
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
            .filter_map(|value| ForecastValue::try_from(value).ok())
            .flat_map(|w| ForecastValue::expand_time_ranges(&w).unwrap())
            .collect())
    }

    fn expand_time_ranges(&self) -> anyhow::Result<Vec<Self>> {
        let (time, period) = self
            .valid_time
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
            let (valid_time, display_time) = utils::increment_time(time, i)?;
            out.push(Self {
                value: self.value,
                valid_time,
                display_time,
            })
        }

        Ok(out)
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

        let wave_height = ForecastValue::get(properties, "waveHeight")?;
        let wave_period = ForecastValue::get(properties, "wavePeriod")?;
        let wave_direction = ForecastValue::get(properties, "waveDirection")?;
        let wind_speed = ForecastValue::get(properties, "windSpeed")?;
        let wind_gust = ForecastValue::get(properties, "windGust")?;
        let wind_direction = ForecastValue::get(properties, "windDirection")?;
        // let temperature = ForecastValue::get(properties, "temperature").unwrap();
        // let probability_of_precipitation =
        //     ForecastValue::get(properties, "probabilityOfPrecipitation").unwrap();

        Ok(Self {
            last_updated,
            // probability_of_precipitation,
            // temperature,
            wave_height,
            wave_period,
            wave_direction,
            wind_speed,
            wind_gust,
            wind_direction,
            quality: None,
        })
    }
}

impl TryFrom<serde_json::Value> for ForecastValue {
    type Error = anyhow::Error;
    fn try_from(v: serde_json::Value) -> Result<Self, Self::Error> {
        let value = v
            .get("value")
            .ok_or(anyhow!("No value found."))?
            .as_f64()
            .ok_or(anyhow!("Not an f64"))?;
        let valid_time = v
            .get("validTime")
            .ok_or(anyhow!("No validTime found."))?
            .to_string();

        Ok(Self {
            value,
            valid_time,
            display_time: None,
        })
    }
}
