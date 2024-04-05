use anyhow::anyhow;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub fn convert_meter_to_feet(value: f64) -> f64 {
    value * 3.281
}

pub fn convert_meter_to_mile(value: &str) -> String {
    format!("{:.0}", value.parse().unwrap_or(0.0) * 2.2369)
}

pub fn parse_hour(s: &str) -> anyhow::Result<usize> {
    if let Some((_, hour)) = s.split_once("T") {
        let hour = hour.strip_suffix("H\"").ok_or(anyhow!("no hour found!"))?;
        return Ok(hour.parse()?);
    };

    Err(anyhow!("no hour found!"))
}

pub fn increment_time(t: &str, amount: usize) -> anyhow::Result<String> {
    let time = t.strip_prefix("\"").unwrap();
    let time = time.strip_suffix("+00:00").unwrap();
    let time = time.parse::<NaiveDateTime>().unwrap();
    let time: DateTime<Local> = Local.from_local_datetime(&time).unwrap().into();

    Ok((time + std::time::Duration::from_secs(amount as u64 * 3_600)).to_string())
}
