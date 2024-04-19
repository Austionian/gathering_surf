use crate::{
    quality,
    utils::{convert_celsius_to_fahrenheit, convert_meter_to_mile},
};
use chrono::{TimeZone, Utc};
use chrono_tz::US::Central;

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
}

impl Latest {
    pub async fn try_get() -> anyhow::Result<Self> {
        let data = reqwest::get("https://www.ndbc.noaa.gov/data/realtime2/MLWW3.txt")
            .await?
            .text()
            .await?;

        let bouy_data = reqwest::get("https://www.ndbc.noaa.gov/data/realtime2/45214.txt")
            .await?
            .text()
            .await?;

        let water_temp = convert_celsius_to_fahrenheit(
            bouy_data
                .lines()
                .nth(2)
                .unwrap()
                .split_whitespace()
                .nth(14)
                .unwrap(),
        );

        let latest = data.lines().collect::<Vec<_>>();
        let line = latest.get(2).unwrap();

        let (as_of, measurements) = line.split_at(16);

        let as_of = Self::parse_as_of(as_of)?;
        let mut measurements = measurements.split_whitespace();
        let wind_direction = measurements.next().unwrap().parse().unwrap_or(0);

        let wind_speed = convert_meter_to_mile(measurements.next().unwrap());
        let gusts = convert_meter_to_mile(measurements.next().unwrap());

        let air_temp = convert_celsius_to_fahrenheit(measurements.nth(5).unwrap());

        let wave_quality = quality::get_quality(wind_speed.parse().unwrap(), wind_direction as f64);

        Ok(Self {
            air_temp,
            as_of,
            wind_direction,
            wind_speed,
            gusts,
            water_temp,
            quality_text: wave_quality.0,
            quality_color: wave_quality.1,
        })
    }

    pub fn parse_as_of(as_of: &str) -> anyhow::Result<String> {
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
