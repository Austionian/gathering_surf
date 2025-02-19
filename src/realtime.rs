use super::Spot;
use crate::{
    utils::{
        convert_celsius_to_fahrenheit, convert_meter_per_second_to_miles_per_hour,
        convert_meter_to_feet, redis_utils,
    },
    AppState,
};

use anyhow::bail;
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use chrono_tz::US::Central;
use std::sync::Arc;
use tracing::{error, info, warn};

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
    pub loaded_from_fallback: bool,
}

impl Realtime {
    pub async fn try_get_string(spot: Arc<Spot>, state: Arc<AppState>) -> anyhow::Result<String> {
        if let Some(data) =
            redis_utils::get(&format!("realtime-{}", spot.name), &state.redis_pool).await
        {
            tracing::info!("redis cache hit!");
            return Ok(data);
        }

        let data = Self::try_get(spot.clone(), state.realtime_url).await?;
        let data = serde_json::to_string(&data)?;

        redis_utils::set(&format!("realtime-{}", spot.name), &data, &state.redis_pool).await?;

        Ok(data)
    }

    pub async fn try_get(spot: Arc<Spot>, realtime_url: &'static str) -> anyhow::Result<Self> {
        const FALLBACK_BOUY: &str = "/data/realtime2/45007.txt";

        let mut from_fallback = false;

        // Seems as though bouy data is removed from noaa after it gets stale enough with no new
        // information. Check if there was an error and then try the fallback.
        let data = if let Ok(data) = Self::get_latest_data(&spot, realtime_url).await {
            data
        } else {
            from_fallback = true;
            Self::get_fallback_data(&spot, realtime_url, FALLBACK_BOUY).await?
        };

        let mut loaded_from_fallback = !spot.has_bouy || from_fallback;

        let latest = data.lines().collect::<Vec<_>>();
        let line = latest.get(2).unwrap();

        let (as_of, measurements) = line.split_at(16);

        let as_of = Self::parse_as_of(as_of)?;

        // Check if the bouy data is older than a day, if so fallback to other path. And only if
        // the data isn't already from the fallback.
        if Utc::now() - as_of > TimeDelta::days(1) && !from_fallback {
            let data = Self::get_fallback_data(&spot, realtime_url, FALLBACK_BOUY).await?;
            loaded_from_fallback = true;
            let latest = data.lines().collect::<Vec<_>>();
            let line = latest.get(2).unwrap();

            let (as_of, measurements) = line.split_at(16);
            let as_of = Self::parse_as_of(as_of)?;
            return Self::parse_data(
                measurements,
                &latest,
                &as_of,
                &spot,
                realtime_url,
                loaded_from_fallback,
            )
            .await;
        }

        Self::parse_data(
            measurements,
            &latest,
            &as_of,
            &spot,
            realtime_url,
            loaded_from_fallback,
        )
        .await
    }

    async fn parse_data(
        measurements: &str,
        latest: &[&str],
        as_of: &DateTime<Utc>,
        spot: &Spot,
        realtime_url: &str,
        loaded_from_fallback: bool,
    ) -> anyhow::Result<Self> {
        // MID Lake bouy is in the water yeat round
        const FALLBACK_BOUY: &str = "/data/realtime2/45007.txt";

        let as_of = as_of
            .with_timezone(&Central)
            .to_rfc2822()
            .split(" -")
            .next()
            .unwrap()
            .to_string();

        let mut measurements = measurements.split_whitespace();
        let wind_direction = measurements.next().unwrap().parse().unwrap_or(0);

        let wind_speed = convert_meter_per_second_to_miles_per_hour(measurements.next().unwrap());
        let gusts = convert_meter_per_second_to_miles_per_hour(measurements.next().unwrap());

        let wave_height = Self::parse_wave_height(measurements.next().unwrap_or(""));
        let wave_period = measurements.next().unwrap().parse().ok();

        let _ = measurements.next();

        // Sometimes bouys only update the wave direction every third hour,
        // this attempts to fallback to earlier readings.
        let wave_direction = match measurements
            .next()
            .unwrap()
            .parse::<u16>()
            .ok()
            .map(|v| v + 180)
        {
            Some(v) => Some(v),
            None => match Self::get_wave_direction(latest, 1) {
                Some(v) => Some(v),
                None => Self::get_wave_direction(latest, 2),
            },
        };

        let _ = measurements.next();

        let air_temp =
            convert_celsius_to_fahrenheit(measurements.next().unwrap().parse().unwrap_or(0.0));

        let raw_water_temp = measurements.next().unwrap_or("MM");

        let water_temp = if raw_water_temp == "MM" {
            info!("fetching fallback bouy data for water temp");
            let bouy_data = reqwest::get(format!("{}{}", realtime_url, FALLBACK_BOUY))
                .await?
                .text()
                .await?;

            // Start at row two to get past the table headers
            let mut row = 2;
            // Loop until a valid value is found.
            while bouy_data
                .lines()
                .nth(row)
                .unwrap_or("")
                .split_whitespace()
                .nth(14)
                .unwrap_or("0.0")
                .parse::<f64>()
                .is_err()
            {
                row += 1;
            }

            convert_celsius_to_fahrenheit(
                bouy_data
                    .lines()
                    .nth(row)
                    .unwrap_or("")
                    .split_whitespace()
                    .nth(14)
                    .unwrap_or("0.0")
                    .parse()
                    .unwrap(),
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
            loaded_from_fallback,
        })
    }

    async fn get_data(path: &str, realtime_url: &str) -> Result<String, anyhow::Error> {
        const RETRY: u8 = 2;
        for _ in 0..RETRY {
            let response = reqwest::get(format!("{}{}", realtime_url, path)).await?;
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

    async fn get_latest_data(spot: &Spot, realtime_url: &str) -> Result<String, anyhow::Error> {
        Self::get_data(spot.realtime_path, realtime_url).await
    }

    /// Checks if the bouy has a fallback available, otherwise uses the path provided.
    async fn get_fallback_data(
        spot: &Spot,
        realtime_url: &str,
        fallback_url: &str,
    ) -> Result<String, anyhow::Error> {
        if let Some(path) = spot.fallback_realtime_path {
            Self::get_data(path, realtime_url).await
        } else {
            Self::get_data(fallback_url, realtime_url).await
        }
    }

    fn parse_wave_height(wave_height: &str) -> Option<String> {
        if let Ok(v) = wave_height.parse::<f64>() {
            Some(format!("{:.2}", convert_meter_to_feet(v)))
        } else {
            None
        }
    }

    fn parse_as_of(as_of: &str) -> anyhow::Result<DateTime<Utc>> {
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

        Ok(as_of)
    }

    fn get_wave_direction(latest: &[&str], offset: usize) -> Option<u16> {
        latest
            .get(2 + offset)
            .unwrap()
            .split_whitespace()
            .nth(11)
            .unwrap()
            .parse::<u16>()
            .ok()
            .map(|v| v + 180)
    }
}
