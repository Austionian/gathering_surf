use crate::utils::{convert_celsius_to_fahrenheit, convert_meter_to_feet, convert_meter_to_mile};
use chrono::{TimeZone, Utc};
use chrono_tz::US::Central;

use super::Spot;

#[derive(serde::Serialize)]
pub struct Latest {
    pub as_of: String,
    pub wind_direction: u32,
    pub wind_speed: String,
    pub gusts: String,
    pub water_temp: String,
    pub air_temp: String,
    pub quality_color: &'static str,
    pub quality_text: &'static str,
    pub wave_height: Option<String>,
    pub wave_period: Option<u8>,
    pub wave_direction: Option<u16>,
}

impl Latest {
    pub async fn try_get(spot: &Spot) -> anyhow::Result<Self> {
        static MID_LAKE_BOUY: &str = "https://www.ndbc.noaa.gov/data/realtime2/45214.txt";

        let data = Self::get_latest_data(spot).await?;

        let latest = data.lines().collect::<Vec<_>>();
        let line = latest.get(2).unwrap();

        let (as_of, measurements) = line.split_at(16);

        let as_of = Self::parse_as_of(as_of)?;
        let mut measurements = measurements.split_whitespace();
        let wind_direction = measurements.next().unwrap().parse().unwrap_or(0);

        let wind_speed = convert_meter_to_mile(measurements.next().unwrap());
        let gusts = convert_meter_to_mile(measurements.next().unwrap());

        let wave_height = Self::parse_wave_height(measurements.next().unwrap_or(""));
        let wave_period = measurements.next().unwrap().parse().ok();

        let _ = measurements.next();

        let wave_direction = measurements
            .next()
            .unwrap()
            .parse::<u16>()
            .ok()
            .map(|v| v + 180);

        let _ = measurements.next();

        let air_temp =
            convert_celsius_to_fahrenheit(measurements.next().unwrap().parse().unwrap_or(0.0));

        let raw_water_temp = measurements.next().unwrap_or("MM");

        let water_temp = if raw_water_temp == "MM" {
            let bouy_data = reqwest::get(MID_LAKE_BOUY).await?.text().await?;
            convert_celsius_to_fahrenheit(
                bouy_data
                    .lines()
                    .nth(2)
                    .unwrap()
                    .split_whitespace()
                    .nth(14)
                    .unwrap()
                    .parse()
                    .unwrap_or(0.0),
            )
        } else {
            convert_celsius_to_fahrenheit(raw_water_temp.parse().unwrap_or(0.0))
        };

        let wave_quality = spot
            .location
            .get_quality(wind_speed.parse().unwrap(), wind_direction as f64);

        Ok(Self {
            air_temp,
            as_of,
            wind_direction,
            wind_speed,
            gusts,
            water_temp,
            quality_text: wave_quality.0,
            quality_color: wave_quality.1,
            wave_height,
            wave_period,
            wave_direction,
        })
    }

    async fn get_latest_data(spot: &Spot) -> Result<String, reqwest::Error> {
        let response = reqwest::get(spot.latest_url).await?;

        if response.status().as_u16() != 200 {
            if let Some(fallback_url) = spot.fallback_latest_url {
                return reqwest::get(fallback_url).await?.text().await;
            }
        }

        response.text().await
    }

    fn parse_wave_height(wave_height: &str) -> Option<String> {
        if let Ok(v) = wave_height.parse::<f64>() {
            Some(format!("{:.2}", convert_meter_to_feet(v)))
        } else {
            None
        }
    }

    fn parse_as_of(as_of: &str) -> anyhow::Result<String> {
        let as_of = as_of.trim().split(' ').collect::<Vec<_>>();
        let as_of = Utc
            .with_ymd_and_hms(
                as_of.first().unwrap().parse::<i32>().unwrap(),
                as_of.get(1).unwrap().parse::<u32>().unwrap(),
                as_of.get(2).unwrap().parse::<u32>().unwrap(),
                as_of.get(3).unwrap().parse::<u32>().unwrap(),
                as_of.get(4).unwrap().parse::<u32>().unwrap(),
                00,
            )
            .unwrap();

        let as_of = as_of.with_timezone(&Central).to_rfc2822();

        Ok(as_of.split(" -").next().unwrap().to_string())
    }
}
