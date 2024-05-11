/// Contains quality text and associated color.
pub struct Quality(pub &'static str, pub &'static str);

pub const GOOD: Quality = Quality("Good", "#0bd674");
pub const OK: Quality = Quality("Fair to Good", "#ffcd1e");
pub const POOR: Quality = Quality("Poor", "#ff9500");
pub const VERY_POOR: Quality = Quality("Very Poor", "#f4496d");

pub trait GetQuality {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality;
}

impl Quality {
    pub fn north(wind_speed: f64, wind_direction: f64) -> &'static Self {
        if wind_speed < 5.0 {
            return &GOOD;
        }

        // Essentially offshore
        if (300.0..340.0).contains(&wind_direction) {
            return &GOOD;
        }

        // Primarily north wind
        if (0.0..70.0).contains(&wind_direction) || (270.0..361.0).contains(&wind_direction) {
            if wind_speed <= 30.0 {
                return &GOOD;
            }
            return &OK;
        }

        if (70.0..120.0).contains(&wind_direction) || (230.0..270.0).contains(&wind_direction) {
            if wind_speed <= 30.0 {
                return &OK;
            }
            return &POOR;
        }

        if (120.0..230.0).contains(&wind_direction) {
            if wind_speed <= 30.0 {
                return &POOR;
            }
            return &VERY_POOR;
        }

        &POOR
    }

    pub fn south(wind_speed: f64, wind_direction: f64) -> &'static Self {
        if wind_speed < 5.0 {
            return &GOOD;
        }

        // Essentially offshore
        if (240.0..310.0).contains(&wind_direction) {
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
}
