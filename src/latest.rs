use crate::utils::convert_celsius_to_fahrenheit;
use chrono::{TimeZone, Utc};
use chrono_tz::US::Central;

pub struct Latest {
    pub as_of: String,
    pub wind_direction: String,
    pub wind_speed: String,
    pub gusts: String,
    pub water_temp: String,
}

impl Latest {
    pub async fn get() -> anyhow::Result<Self> {
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
            water_temp,
        })
    }

    pub fn parse_as_of(as_of: &str) -> anyhow::Result<String> {
        let as_of = as_of.trim().split(" ").collect::<Vec<_>>();
        let as_of = Utc
            .with_ymd_and_hms(
                as_of.get(0).unwrap().parse::<i32>().unwrap(),
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
