use super::{GetQuality, Quality};
use crate::{GOOD, OK, POOR, VERY_POOR};
use std::fmt::Display;

#[derive(serde::Deserialize, Debug)]
pub struct SpotParam {
    pub spot: Option<String>,
}

impl SpotParam {
    pub fn get_spot(&mut self) -> String {
        // Get the selected spot, fallback to Atwater
        self.spot.take().unwrap_or("Atwater".to_string())
    }
}

pub struct Spot {
    pub forecast_url: &'static str,
    pub latest_url: &'static str,
    pub bouy_url: Option<&'static str>,
    pub location: Location,
}

impl Display for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location)
    }
}

impl From<SpotParam> for Spot {
    fn from(mut val: SpotParam) -> Self {
        let atwater = Spot {
            forecast_url: ATWATER_URL,
            latest_url: ATWATER_LATEST_URL,
            bouy_url: Some("https://www.ndbc.noaa.gov/data/realtime2/45013.txt"),
            location: Location::Atwater(Atwater),
        };

        match val.get_spot().to_lowercase().as_str() {
            "bradford" => Spot {
                forecast_url: BRADFORD_URL,
                latest_url: BRADFORD_LATEST_URL,
                bouy_url: None,
                location: Location::Bradford(Bradford),
            },
            "atwater" => atwater,
            _ => atwater,
        }
    }
}

static ATWATER_URL: &str = "https://api.weather.gov/gridpoints/MKX/90,67";
static BRADFORD_URL: &str = "https://api.weather.gov/gridpoints/MKX/91,67";

static ATWATER_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/MLWW3.txt";
static BRADFORD_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/MLWW3.txt";

pub struct Atwater;
pub struct Bradford;

impl Display for Atwater {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Atwater")
    }
}

impl Display for Bradford {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bradford")
    }
}

pub enum Location {
    Atwater(Atwater),
    Bradford(Bradford),
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atwater(Atwater) => write!(f, "{}", Atwater),
            Self::Bradford(Bradford) => write!(f, "{}", Bradford),
        }
    }
}

impl Location {
    pub fn get_quality(&self, wind_speed: f64, wind_direction: f64) -> &'static Quality {
        match self {
            Self::Atwater(Atwater) => Atwater::get_quality(wind_speed, wind_direction),
            Self::Bradford(Bradford) => Bradford::get_quality(wind_speed, wind_direction),
        }
    }
}

impl GetQuality for Bradford {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
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
}

impl GetQuality for Atwater {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
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
}
