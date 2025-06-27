use anyhow::anyhow;
use chrono::{DateTime, Timelike};
use chrono_tz::US::Central;
pub mod redis_utils;

pub fn convert_meter_to_feet(value: f64) -> f64 {
    value * 3.281
}

pub fn convert_kilo_meter_to_mile(value: f64) -> f64 {
    value * 0.621
}

pub fn convert_meter_per_second_to_miles_per_hour(value: &str) -> String {
    format!("{:.0}", value.parse().unwrap_or(0.0) * 2.2369)
}

pub fn convert_celsius_to_fahrenheit(value: f64) -> String {
    format!("{:.0}", value * 1.8 + 32.0)
}

pub fn parse_hour(s: &str) -> anyhow::Result<usize> {
    if let Some((_, hour)) = s.split_once('T') {
        let hour = hour.strip_suffix("H").ok_or(anyhow!("no hour found!"))?;
        return Ok(hour.parse()?);
    };

    Err(anyhow!("no hour found!"))
}

fn convert_24_to_12_hour(hour: u32) -> String {
    if hour == 12 {
        return "12 PM".to_string();
    }
    if hour == 0 {
        return "12 AM".to_string();
    }

    if hour < 10 {
        return format!("0{hour} AM");
    }

    if hour < 12 {
        return format!("{hour} AM");
    }

    let hour = hour - 12;
    if hour < 10 {
        return format!("0{hour} PM");
    }

    format!("{hour} PM")
}

/// Given a time string, e.g. "2024-09-06T11:00:00+00:00" and a number
/// of hours to increase to, e.g. 2, returns a display friendly time, e.g.
/// "Fri 09 AM"
pub fn increment_time(t: &str, hours: usize) -> anyhow::Result<String> {
    let time = DateTime::parse_from_str(t, "%+")
        .unwrap()
        .with_timezone(&Central)
        + chrono::Duration::hours(hours as i64);

    let hour = convert_24_to_12_hour(time.hour());

    let time = time.to_rfc2822();

    let (day, _) = time.split_once(',').unwrap();

    Ok(format!("{day} {hour}"))
}

/// Limits f64 to two decimal points
pub fn truncate_to_two_decimals(v: f64) -> f64 {
    (v * 100.0).trunc() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_to_two_decimals_limits_f64_to_two_decimals() {
        assert_eq!(truncate_to_two_decimals(12.121212), 12.12)
    }

    #[test]
    fn increment_time_creates_display_string() {
        assert_eq!(
            increment_time("2024-09-06T11:00:00+00:00", 2).unwrap(),
            "Fri 08 AM"
        )
    }

    #[test]
    fn increment_time_creates_display_string_with_a_pm_time() {
        assert_eq!(
            increment_time("2024-09-06T11:00:00+00:00", 12).unwrap(),
            "Fri 06 PM"
        )
    }

    #[test]
    fn increment_time_creates_handles_noon() {
        assert_eq!(
            increment_time("2024-09-06T11:00:00+00:00", 6).unwrap(),
            "Fri 12 PM"
        )
    }

    #[test]
    fn increment_time_creates_handles_midnight() {
        assert_eq!(
            increment_time("2024-09-06T11:00:00+00:00", 18).unwrap(),
            "Sat 12 AM"
        )
    }

    #[test]
    fn convert_24_to_12_hour_adds_a_leading_zero_to_am() {
        assert_eq!(convert_24_to_12_hour(7), "07 AM")
    }

    #[test]
    fn convert_24_to_12_hour_adds_a_leading_zero_to_pm() {
        assert_eq!(convert_24_to_12_hour(13), "01 PM")
    }

    #[test]
    fn convert_24_to_12_hour_does_not_add_a_leading_zero_to_am() {
        assert_eq!(convert_24_to_12_hour(10), "10 AM")
    }

    #[test]
    fn convert_24_to_12_hour_does_not_add_a_leading_zero_to_pm() {
        assert_eq!(convert_24_to_12_hour(22), "10 PM")
    }

    #[test]
    fn convert_24_to_12_hour_handles_noon() {
        assert_eq!(convert_24_to_12_hour(12), "12 PM")
    }

    #[test]
    fn convert_24_to_12_hour_handles_midnight() {
        assert_eq!(convert_24_to_12_hour(00), "12 AM")
    }

    #[test]
    fn parse_hour_gets_the_number_of_hours_from_a_period_string() {
        assert_eq!(parse_hour("T12H").unwrap(), 12)
    }

    #[test]
    fn convert_meter_per_second_to_miles_per_hour_converts_correctly() {
        assert_eq!(convert_meter_per_second_to_miles_per_hour("6.0"), "13")
    }

    #[test]
    fn convert_meter_to_feet_converts_correctly() {
        assert_eq!(convert_meter_to_feet(1.0), 3.281)
    }

    #[test]
    fn convert_kilo_meter_to_mile_converts_correctly() {
        assert_eq!(convert_kilo_meter_to_mile(11.5), 7.1415)
    }

    #[test]
    fn convert_celsius_to_fahrenheit_converts_correctly() {
        assert_eq!(convert_celsius_to_fahrenheit(66.0), "151")
    }
}
