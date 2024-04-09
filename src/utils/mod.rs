use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use chrono_tz::US::Central;

pub fn convert_meter_to_feet(value: f64) -> f64 {
    value * 3.281
}

pub fn convert_kilo_meter_to_mile(value: f64) -> f64 {
    value * 0.621
}

pub fn convert_meter_to_mile(value: &str) -> String {
    format!("{:.0}", value.parse().unwrap_or(0.0) * 2.2369)
}

pub fn convert_celsius_to_fahrenheit(value: &str) -> String {
    format!("{:.0}", value.parse().unwrap_or(0.0) * 1.8 + 32.0)
}

pub fn parse_hour(s: &str) -> anyhow::Result<usize> {
    if let Some((_, hour)) = s.split_once("T") {
        let hour = hour.strip_suffix("H\"").ok_or(anyhow!("no hour found!"))?;
        return Ok(hour.parse()?);
    };

    Err(anyhow!("no hour found!"))
}

pub fn increment_time(t: &str, amount: usize) -> anyhow::Result<(String, Option<String>)> {
    let time = t.strip_prefix("\"").unwrap();
    let time = time.strip_suffix("+00:00").unwrap();
    let mut time = time.parse::<NaiveDateTime>().unwrap();
    time = time + std::time::Duration::from_secs(amount as u64 * 3_600);
    let time: DateTime<_> = Central.from_local_datetime(&time).unwrap().into();

    let valid_time = time.to_rfc3339();
    let time = time.to_rfc2822();

    let (day, rest) = time.split_once(",").unwrap();

    let time = rest.split_whitespace().nth(3).unwrap();

    Ok((valid_time, Some(format!("{day} {time}"))))
}
