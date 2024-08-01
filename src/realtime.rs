use crate::{
    utils::{convert_celsius_to_fahrenheit, convert_meter_to_feet, convert_meter_to_mile},
    QUALITY_PATH,
};
use anyhow::{anyhow, bail};
use chrono::{TimeZone, Utc};
use chrono_tz::US::Central;
use tracing::{error, info, warn};

use super::Spot;

#[derive(serde::Serialize)]
pub struct Realtime {
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
    pub water_quality: String,
    pub water_quality_text: String,
}

impl Realtime {
    pub async fn try_get(
        spot: &Spot,
        realtime_url: &'static str,
        quality_url: &'static str,
    ) -> anyhow::Result<Self> {
        // MID Lake bouy is in the water yeat round
        // const MID_LAKE_BOUY: &str = "https://www.ndbc.noaa.gov/data/realtime2/45214.txt";
        // Fallback to Atwater bouy for now.
        const FALLBACK_BOUY: &str = "/data/realtime2/45013.txt";

        let quality_query = spot.quality_query;
        let status_query = spot.status_query;
        let water_quality_data = tokio::spawn(async move {
            Self::get_quality_data(quality_query, status_query, quality_url)
                .await
                .unwrap()
        });

        let data = Self::get_latest_data(spot, realtime_url).await?;

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
            let bouy_data = reqwest::get(format!("{}{}", realtime_url, FALLBACK_BOUY))
                .await?
                .text()
                .await?;
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

        let wave_quality = spot.location.get_quality(
            wave_height
                .clone()
                .unwrap_or("99.0".to_string())
                .parse()
                .unwrap(),
            wind_speed.parse().unwrap(),
            wind_direction as f64,
        );

        let (water_quality, water_quality_text) = water_quality_data.await?;

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
            water_quality,
            water_quality_text,
        })
    }

    async fn get_quality_data(
        quality_query: &'static str,
        status_query: &'static str,
        quality_url: &'static str,
    ) -> anyhow::Result<(String, String)> {
        let status = reqwest::get(format!("{quality_url}{QUALITY_PATH}{status_query}"))
            .await?
            .json::<serde_json::Value>()
            .await?;

        let response = reqwest::get(format!("{quality_url}{QUALITY_PATH}{quality_query}"))
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok((
            status
                .get("features")
                .ok_or(anyhow!("no features found."))?
                .as_array()
                .ok_or(anyhow!("features is not an array."))?
                .first()
                .ok_or(anyhow!("empty array of features."))?
                .get("attributes")
                .ok_or(anyhow!("no attributes found."))?
                .get("MAP_STATUS")
                .ok_or(anyhow!("no map status found."))?
                .as_str()
                .ok_or(anyhow!("map status not a string."))?
                .to_string(),
            response
                .get("features")
                .ok_or(anyhow!("no features found."))?
                .as_array()
                .ok_or(anyhow!("features is not an array."))?
                .first()
                .ok_or(anyhow!("empty array of features."))?
                .get("attributes")
                .ok_or(anyhow!("no attributes found."))?
                .get("STATUS")
                .ok_or(anyhow!("no status found."))?
                .as_str()
                .ok_or(anyhow!("status not a string."))?
                .to_string(),
        ))
    }

    async fn get_latest_data(spot: &Spot, realtime_url: &str) -> Result<String, anyhow::Error> {
        const RETRY: u8 = 2;
        for _ in 0..RETRY {
            let response = reqwest::get(format!("{}{}", realtime_url, spot.realtime_path)).await?;
            if response.status().as_u16() == 200 {
                info!("NOAA realtime 200 success.");
                return match response.text().await {
                    Ok(r) => Ok(r),
                    Err(e) => Err(anyhow::anyhow!("Error reading realtime message: {}", e)),
                };
            }
            warn!("NOAA realtime non-200, retrying.");
        }

        error!("Non 200 response from NOAA realtime");
        bail!("Non 200 response from NOAA realtime");
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
