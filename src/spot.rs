use super::{GetQuality, Quality};
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
    pub fallback_latest_url: Option<&'static str>,
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
            fallback_latest_url: Some(BRADFORD_LATEST_URL),
            location: Location::Atwater(Atwater),
        };

        match val.get_spot().to_lowercase().as_str() {
            "bradford" => Spot {
                forecast_url: BRADFORD_URL,
                latest_url: BRADFORD_LATEST_URL,
                fallback_latest_url: None,
                location: Location::Bradford(Bradford),
            },
            "port washington" => Spot {
                forecast_url: PORT_WASHINGTON_URL,
                latest_url: PORT_WASHINGTON_LATEST_URL,
                fallback_latest_url: None,
                location: Location::PortWashington(PortWashington),
            },
            "sheboygan" => Spot {
                forecast_url: SHEBOYGAN_URL,
                latest_url: SHEBOYGAN_LATEST_URL,
                fallback_latest_url: Some(SHEBOYGAN_FALLBACK_LATEST_URL),
                location: Location::Sheboygan(Sheboygan),
            },
            "racine" => Spot {
                forecast_url: RACINE_URL,
                latest_url: RACINE_LATEST_URL,
                fallback_latest_url: Some(RACINE_FALLBACK_LATEST_URL),
                location: Location::Racine(Racine),
            },
            "atwater" => atwater,
            _ => atwater,
        }
    }
}

static ATWATER_URL: &str = "https://api.weather.gov/gridpoints/MKX/90,67";
static BRADFORD_URL: &str = "https://api.weather.gov/gridpoints/MKX/91,67";
static SHEBOYGAN_URL: &str = "https://api.weather.gov/gridpoints/MKX/94,98";
static PORT_WASHINGTON_URL: &str = "https://api.weather.gov/gridpoints/MKX/91,80";
static RACINE_URL: &str = "https://api.weather.gov/gridpoints/MKX/94,52";

static ATWATER_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/45013.txt";
static SHEBOYGAN_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/45218.txt";
static RACINE_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/45199.txt";

// -- These are all land based weather stations --
static BRADFORD_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/MLWW3.txt";
static PORT_WASHINGTON_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/PWAW3.txt";

static SHEBOYGAN_FALLBACK_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/SGNW3.txt";
static RACINE_FALLBACK_LATEST_URL: &str = "https://www.ndbc.noaa.gov/data/realtime2/KNSW3.txt";
// -- --

pub struct Atwater;
pub struct Bradford;
pub struct Sheboygan;
pub struct PortWashington;
pub struct Racine;

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

impl Display for Sheboygan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sheboygan")
    }
}

impl Display for PortWashington {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Port Washington")
    }
}

impl Display for Racine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Racine")
    }
}

pub enum Location {
    Atwater(Atwater),
    Bradford(Bradford),
    Sheboygan(Sheboygan),
    PortWashington(PortWashington),
    Racine(Racine),
}

impl Location {
    pub fn get_all() -> Vec<String> {
        Self::into_iter().map(|v| v.to_string()).collect()
    }

    fn into_iter() -> core::array::IntoIter<Self, 5> {
        [
            Self::Atwater(Atwater),
            Self::Bradford(Bradford),
            Self::Sheboygan(Sheboygan),
            Self::PortWashington(PortWashington),
            Self::Racine(Racine),
        ]
        .into_iter()
    }

    pub fn get_quality(&self, wind_speed: f64, wind_direction: f64) -> &'static Quality {
        match self {
            Self::Atwater(_) => Atwater::get_quality(wind_speed, wind_direction),
            Self::Bradford(_) => Bradford::get_quality(wind_speed, wind_direction),
            Self::Sheboygan(_) => Sheboygan::get_quality(wind_speed, wind_direction),
            Self::PortWashington(_) => PortWashington::get_quality(wind_speed, wind_direction),
            Self::Racine(_) => Racine::get_quality(wind_speed, wind_direction),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atwater(_) => write!(f, "{}", Atwater),
            Self::Bradford(_) => write!(f, "{}", Bradford),
            Self::Sheboygan(_) => write!(f, "{}", Sheboygan),
            Self::PortWashington(_) => write!(f, "{}", PortWashington),
            Self::Racine(_) => write!(f, "{}", Racine),
        }
    }
}

impl GetQuality for Bradford {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
        Quality::south(wind_speed, wind_direction)
    }
}

impl GetQuality for Atwater {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
        Quality::south(wind_speed, wind_direction)
    }
}

impl GetQuality for Sheboygan {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
        Quality::south(wind_speed, wind_direction)
    }
}

impl GetQuality for PortWashington {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
        Quality::north(wind_speed, wind_direction)
    }
}

impl GetQuality for Racine {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality {
        Quality::north(wind_speed, wind_direction)
    }
}
