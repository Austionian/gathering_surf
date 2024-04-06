/// Contains quality text and associated color.
pub struct Quality(pub &'static str, pub &'static str);

static GOOD: Quality = Quality("Good", "#0bd674");
static OK: Quality = Quality("OK", "#f4496d");
static POOR: Quality = Quality("Poor", "#ff9500");
static VERY_POOR: Quality = Quality("Very Poor", "#f4496d");

pub fn get_quality(wind_speed: &str, wind_direction: u32) -> &'static Quality {
    let wind_direction = wind_direction as f64;
    let wind_speed = wind_speed.parse::<f64>().unwrap();

    if wind_direction <= 310.0 || wind_direction >= 180.0 {
        if wind_speed <= 30.0 {
            return &GOOD;
        }
        return &OK;
    }

    if wind_direction > 310.0 {
        if wind_speed <= 30.0 {
            return &OK;
        }
        return &POOR;
    }

    if wind_direction < 180.0 && wind_direction >= 120.0 {
        if wind_speed <= 30.0 {
            return &OK;
        }
        return &POOR;
    }

    if wind_direction < 120.0 {
        if wind_speed <= 30.0 {
            return &POOR;
        }
        return &VERY_POOR;
    }

    &POOR
}
