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

pub fn convert_celsius_to_fahrenheit(value: f64) -> String {
    format!("{:.0}", value * 1.8 + 32.0)
}

pub fn parse_hour(s: &str) -> anyhow::Result<usize> {
    if let Some((_, hour)) = s.split_once('T') {
        let hour = hour.strip_suffix("H\"").ok_or(anyhow!("no hour found!"))?;
        return Ok(hour.parse()?);
    };

    Err(anyhow!("no hour found!"))
}

pub fn convert_military_to_standard_full(time: &str) -> String {
    let (hour, min) = time.split_once(':').unwrap();

    let hour = hour.parse::<u8>().unwrap();

    if hour == 12 {
        return format!("12:{min} PM");
    }
    if hour == 0 {
        return format!("12:{min} AM");
    }

    if hour < 12 {
        return format!("{time}:{min} AM");
    }

    let hour = hour - 12;
    if hour < 10 {
        return format!("0{hour}:{min} PM");
    }

    format!("{hour}:{min} PM")
}

fn convert_military_to_standard(time: &str) -> String {
    let time = time.split(':').next().unwrap();

    let value = time.parse::<u8>().unwrap();

    if value == 12 {
        return "12 PM".to_string();
    }
    if value == 0 {
        return "12 AM".to_string();
    }

    if value < 12 {
        return format!("{time} AM");
    }

    let value = value - 12;
    if value < 10 {
        return format!("0{} PM", value);
    }

    format!("{} PM", value)
}

pub fn increment_time(t: &str, amount: usize) -> anyhow::Result<(String, Option<String>)> {
    let time = t.strip_prefix('"').unwrap();
    let time = time.strip_suffix("+00:00").unwrap();
    let mut time = time.parse::<NaiveDateTime>().unwrap();
    time += std::time::Duration::from_secs(amount as u64 * 3_600);
    let time: DateTime<_> = Central.from_utc_datetime(&time);

    let valid_time = time.to_rfc3339();
    let time = time.to_rfc2822();

    let (day, rest) = time.split_once(',').unwrap();

    let time = rest.split_whitespace().nth(3).unwrap();

    Ok((
        valid_time,
        Some(format!("{day} {}", convert_military_to_standard(time))),
    ))
}
