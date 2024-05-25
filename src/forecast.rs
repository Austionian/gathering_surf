use super::{Location, Spot};
use crate::{convert_celsius_to_fahrenheit, convert_kilo_meter_to_mile, utils};
use anyhow::{anyhow, bail};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::US::Central;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::cmp::Ordering;

pub struct Forecast {
    pub last_updated: String,
    pub probability_of_precipitation: Vec<ForecastValue>,
    pub quality: Option<Vec<String>>,
    pub temperature: Vec<ForecastValue>,
    pub wave_height: Vec<ForecastValue>,
    pub wave_period: Vec<ForecastValue>,
    pub wave_direction: Vec<ForecastValue>,
    pub wind_speed: Vec<ForecastValue>,
    pub wind_gust: Vec<ForecastValue>,
    pub wind_direction: Vec<ForecastValue>,
    pub dewpoint: Vec<ForecastValue>,
    pub cloud_cover: Vec<ForecastValue>,
    pub probability_of_thunder: Vec<ForecastValue>,
}

impl Forecast {
    pub async fn try_get(spot: &Spot) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("GatheringSurf/0.1 (+https://gathering.surf)")
            .build()
            .unwrap();

        let mut response = client.get(spot.forecast_url).send().await?;

        // Retry once if non 200 from NOAA
        if response.status().as_u16() != 200 {
            response = client.get(spot.forecast_url).send().await?;

            if response.status().as_u16() != 200 {
                bail!("Non 200 response from NOAA");
            }
        }

        let mut forecast: Self = (response.json::<serde_json::Value>().await?).try_into()?;

        forecast.condense();
        forecast.get_quality(&spot.location);

        Ok(forecast)
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

    fn get_labels(&self) -> Vec<String> {
        self.wave_height
            .iter()
            .map(|v| v.display_time.clone().unwrap_or_default())
            .collect()
    }

    fn get_wave_data(&self) -> (Vec<f64>, u8) {
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

        // Make sure the graph max is always an even number. Otherwise the graph
        // won't display the y axis labels correctly
        let graph_max = if let Some(mut v) = smoothed_data.iter().map(|v| *v as u8).max() {
            if (v & 1) == 0 {
                v += 2;
            } else {
                v += 1;
            }
            v
        } else {
            4
        };

        (out.iter().map(|v| Self::truncc(*v)).collect(), graph_max)
    }

    /// Limits the f64 to two decimal points
    fn truncc(v: f64) -> f64 {
        (v * 100.0).trunc() / 100.0
    }

    fn get_wind_data(&self) -> Vec<f64> {
        self.wind_speed
            .iter()
            .map(|v| Self::truncc(convert_kilo_meter_to_mile(v.value)))
            .collect()
    }

    fn get_wind_direction_data(&self) -> Vec<f64> {
        self.wind_direction
            .iter()
            .map(|v| Self::truncc(v.value) + 180.0)
            .collect()
    }

    fn get_wind_gust_data(&self) -> Vec<f64> {
        self.wind_gust
            .iter()
            .map(|v| Self::truncc(convert_kilo_meter_to_mile(v.value)))
            .collect()
    }

    fn get_temperature(&self) -> Vec<u8> {
        self.temperature
            .iter()
            .filter_map(|v| convert_celsius_to_fahrenheit(v.value).parse().ok())
            .collect()
    }

    fn get_probability_of_precipitation(&self) -> Vec<u8> {
        self.probability_of_precipitation
            .iter()
            .map(|v| v.value as u8)
            .collect()
    }

    fn get_wave_period_data(&self) -> Vec<f64> {
        self.wave_period.iter().map(|v| v.value).collect()
    }

    fn get_dewpoint(&self) -> Vec<String> {
        self.dewpoint
            .iter()
            .map(|v| convert_celsius_to_fahrenheit(v.value))
            .collect()
    }

    fn get_cloud_cover(&self) -> Vec<u8> {
        self.cloud_cover.iter().map(|v| v.value as u8).collect()
    }

    fn get_probability_of_thunder(&self) -> Vec<u8> {
        self.probability_of_thunder
            .iter()
            .map(|v| v.value as u8)
            .collect()
    }

    /// Returns the wave height, period and direction from the forecasted
    /// data relative to the time of request.
    fn get_current_wave_data(&self) -> (String, String, String) {
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

    pub fn get_quality(&mut self, location: &Location) {
        let mut qualities = Vec::with_capacity(self.wind_direction.len());
        for (wind_direction, wind_speed) in self.wind_direction.iter().zip(self.wind_speed.iter()) {
            qualities.push(
                location
                    .get_quality(wind_speed.value, wind_direction.value)
                    .1
                    .to_string(),
            );
        }

        self.quality = Some(qualities)
    }
}

impl Serialize for Forecast {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (wave_height_data, graph_max) = self.get_wave_data();
        let (current_wave_height, current_wave_period, current_wave_direction) =
            self.get_current_wave_data();

        let mut state = serializer.serialize_struct("Forecast", 17)?;
        state.serialize_field("forecast_as_of", &self.last_updated)?;
        state.serialize_field("graph_max", &(graph_max))?;
        state.serialize_field("wave_height_data", &wave_height_data)?;
        state.serialize_field("current_wave_height", &current_wave_height)?;
        state.serialize_field("current_wave_direction", &current_wave_direction)?;
        state.serialize_field("current_wave_period", &current_wave_period)?;
        state.serialize_field("wind_speed_data", &self.get_wind_data())?;
        state.serialize_field("wind_direction_data", &self.get_wind_direction_data())?;
        state.serialize_field("wind_gust_data", &self.get_wind_gust_data())?;
        state.serialize_field("wave_period_data", &self.get_wave_period_data())?;
        state.serialize_field("wave_height_labels", &self.get_labels())?;
        state.serialize_field("temperature", &self.get_temperature())?;
        state.serialize_field(
            "probability_of_precipitation",
            &self.get_probability_of_precipitation(),
        )?;
        state.serialize_field("dewpoint", &self.get_dewpoint())?;
        state.serialize_field("cloud_cover", &self.get_cloud_cover())?;
        state.serialize_field("probability_of_thunder", &self.get_probability_of_thunder())?;
        state.serialize_field("qualities", &self.quality.clone().unwrap())?;
        state.end()
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
            .filter_map(|value| Self::try_from(value).ok())
            .flat_map(|w| Self::expand_time_ranges(&w).unwrap())
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
        let temperature = ForecastValue::get(properties, "temperature")?;
        let probability_of_precipitation =
            ForecastValue::get(properties, "probabilityOfPrecipitation")?;
        let dewpoint = ForecastValue::get(properties, "dewpoint")?;
        let cloud_cover = ForecastValue::get(properties, "skyCover")?;
        let probability_of_thunder = ForecastValue::get(properties, "probabilityOfThunder")?;

        Ok(Self {
            last_updated,
            wave_height,
            wave_period,
            wave_direction,
            wind_speed,
            wind_gust,
            wind_direction,
            quality: None,
            temperature,
            probability_of_precipitation,
            dewpoint,
            cloud_cover,
            probability_of_thunder,
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
