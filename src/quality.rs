/// Contains quality text and associated color.
pub struct Quality(pub &'static str, pub &'static str);

static GOOD: Quality = Quality("Good", "#0bd674");
static OK: Quality = Quality("Fair to Good", "#ffcd1e");
static POOR: Quality = Quality("Poor", "#ff9500");
static VERY_POOR: Quality = Quality("Very Poor", "#f4496d");

pub fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
    if wind_speed < 5.0 {
        return &GOOD;
    }

    // Essentially offshore
    if (150.0..210.0).contains(&wind_direction) {
        return &GOOD;
    }

    if (120.0..330.0).contains(&wind_direction) {
        if wind_speed <= 30.0 {
            return &GOOD;
        }
        return &OK;
    }

    if wind_direction >= 330.0 {
        if wind_speed <= 30.0 {
            return &POOR;
        }
        return &VERY_POOR;
    }

    if (80.0..120.0).contains(&wind_direction) {
        if wind_speed <= 30.0 {
            return &OK;
        }
        return &POOR;
    }

    if (0.0..80.0).contains(&wind_direction) {
        if wind_speed <= 30.0 {
            return &POOR;
        }
        return &VERY_POOR;
    }

    &POOR
}