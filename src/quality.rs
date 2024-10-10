/// Contains quality text and associated color.
#[derive(Debug)]
pub struct Quality(pub &'static str, pub &'static str);

pub const GOOD: Quality = Quality("Good", "#0bd674");
pub const OK: Quality = Quality("Fair to Good", "#ffcd1e");
pub const POOR: Quality = Quality("Poor", "#ff9500");
pub const VERY_POOR: Quality = Quality("Very Poor", "#f4496d");
pub const FLAT: Quality = Quality("Flat", "#a8a29e");

const HIGH_WIND: f64 = 25.0;

impl Quality {
    fn basic_wave_check(wave_height: f64) -> Option<&'static Self> {
        if wave_height < 0.98 {
            return Some(&FLAT);
        }

        None
    }

    pub fn north(wave_height: f64, wind_speed: f64, wind_direction: f64) -> &'static Self {
        if let Some(quality) = Self::basic_wave_check(wave_height) {
            return quality;
        }

        if wind_speed < 5.0 {
            return &GOOD;
        }

        // Essentially offshore
        if (300.0..340.0).contains(&wind_direction) {
            return &GOOD;
        }

        // Primarily north wind
        if (0.0..70.0).contains(&wind_direction) || (270.0..361.0).contains(&wind_direction) {
            if wind_speed <= HIGH_WIND {
                return &GOOD;
            }
            return &OK;
        }

        if (70.0..120.0).contains(&wind_direction) || (230.0..270.0).contains(&wind_direction) {
            if wind_speed <= HIGH_WIND {
                return &OK;
            }
            return &POOR;
        }

        if (120.0..230.0).contains(&wind_direction) {
            if wind_speed <= HIGH_WIND {
                return &POOR;
            }
            return &VERY_POOR;
        }

        &POOR
    }

    pub fn south(wave_height: f64, wind_speed: f64, wind_direction: f64) -> &'static Self {
        if let Some(quality) = Self::basic_wave_check(wave_height) {
            return quality;
        }

        if wind_speed < 5.0 {
            return &GOOD;
        }

        // Essentially offshore
        if (240.0..310.0).contains(&wind_direction) {
            return &GOOD;
        }

        if (120.0..330.0).contains(&wind_direction) {
            if wind_speed <= HIGH_WIND {
                return &GOOD;
            }
            return &OK;
        }

        if wind_direction >= 330.0 {
            if wind_speed <= HIGH_WIND {
                return &OK;
            }
            return &POOR;
        }

        if (80.0..120.0).contains(&wind_direction) {
            if wind_speed <= HIGH_WIND {
                return &OK;
            }
            return &POOR;
        }

        if (0.0..80.0).contains(&wind_direction) {
            if wind_speed <= HIGH_WIND {
                return &POOR;
            }
            return &VERY_POOR;
        }

        &POOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGH_WIND: f64 = 33.3;
    const LOW_WIND: f64 = 5.5;

    const HIGH_WAVES: f64 = 5.7;
    const SMALL_WAVES: f64 = 0.3;

    const NORTH_WIND: f64 = 180.0;
    const SOUTH_WIND: f64 = 0.0;
    const SOUTH_WEST_WIND: f64 = 275.0;
    const NORTH_WEST_WIND: f64 = 95.0;

    #[test]
    fn basic_wave_check_works_when_its_not_flat() {
        assert!(Quality::basic_wave_check(2.1).is_none());
    }

    #[test]
    fn basic_wave_check_works_when_it_is_flat() {
        assert!(Quality::basic_wave_check(0.1).is_some());
        assert_eq!(Quality::basic_wave_check(0.1).unwrap().0, "Flat");
        assert_eq!(Quality::basic_wave_check(0.1).unwrap().1, "#a8a29e");
    }

    #[test]
    fn a_north_beach_should_be_good_in_some_condition() {
        assert_eq!(Quality::north(HIGH_WAVES, LOW_WIND, SOUTH_WIND).0, "Good");
    }

    #[test]
    fn a_north_beach_shoud_be_bad_in_some_condition() {
        assert_eq!(
            Quality::north(HIGH_WAVES, HIGH_WIND, NORTH_WIND).0,
            "Very Poor"
        );
    }

    #[test]
    fn a_north_beach_shoud_be_ok_in_some_condition() {
        assert_eq!(
            Quality::north(HIGH_WAVES, HIGH_WIND, SOUTH_WEST_WIND).0,
            "Fair to Good"
        );
    }

    #[test]
    fn a_north_beach_shoud_be_poor_in_some_condition() {
        assert_eq!(
            Quality::north(HIGH_WAVES, HIGH_WIND, NORTH_WEST_WIND).0,
            "Poor"
        );
    }

    #[test]
    fn a_north_beach_shoud_be_flat_in_some_condition() {
        assert_eq!(Quality::north(SMALL_WAVES, HIGH_WIND, NORTH_WIND).0, "Flat");
    }
}
