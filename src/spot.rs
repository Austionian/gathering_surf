use super::Quality;
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
    pub forecast_path: &'static str,
    pub realtime_path: &'static str,
    pub fallback_realtime_path: Option<&'static str>,
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
            forecast_path: ATWATER_PATH,
            realtime_path: ATWATER_REALTIME_PATH,
            fallback_realtime_path: Some(BRADFORD_REALTIME_PATH),
            location: Location::Atwater(Atwater),
        };

        match val.get_spot().to_lowercase().as_str() {
            "bradford" => Spot {
                forecast_path: BRADFORD_PATH,
                realtime_path: BRADFORD_REALTIME_PATH,
                fallback_realtime_path: None,
                location: Location::Bradford(Bradford),
            },
            "port washington" => Spot {
                forecast_path: PORT_WASHINGTON_PATH,
                realtime_path: PORT_WASHINGTON_REALTIME_PATH,
                fallback_realtime_path: None,
                location: Location::PortWashington(PortWashington),
            },
            "sheboygan" => Spot {
                forecast_path: SHEBOYGAN_PATH,
                realtime_path: SHEBOYGAN_REALTIME_PATH,
                fallback_realtime_path: Some(SHEBOYGAN_FALLBACK_REALTIME_PATH),
                location: Location::Sheboygan(Sheboygan),
            },
            "racine" => Spot {
                forecast_path: RACINE_PATH,
                realtime_path: RACINE_REALTIME_PATH,
                fallback_realtime_path: Some(RACINE_FALLBACK_REALTIME_PATH),
                location: Location::Racine(Racine),
            },
            "atwater" => atwater,
            _ => atwater,
        }
    }
}

// -- Forecast PATHs --
pub const ATWATER_PATH: &str = "/gridpoints/MKX/90,67";
const BRADFORD_PATH: &str = "/gridpoints/MKX/91,67";
const SHEBOYGAN_PATH: &str = "/gridpoints/MKX/94,98";
const PORT_WASHINGTON_PATH: &str = "/gridpoints/MKX/91,80";
const RACINE_PATH: &str = "/gridpoints/MKX/94,52";
// -- --

// -- Realtime PATHs --
//
// -- Bouy PATHs --
pub const ATWATER_REALTIME_PATH: &str = "/data/realtime2/45013.txt";
const SHEBOYGAN_REALTIME_PATH: &str = "/data/realtime2/45218.txt";
const RACINE_REALTIME_PATH: &str = "/data/realtime2/45199.txt";
// -- --

// -- Land Based Weather Station PATHs --
const BRADFORD_REALTIME_PATH: &str = "/data/realtime2/MLWW3.txt";
const PORT_WASHINGTON_REALTIME_PATH: &str = "/data/realtime2/PWAW3.txt";

const SHEBOYGAN_FALLBACK_REALTIME_PATH: &str = "/data/realtime2/SGNW3.txt";
const RACINE_FALLBACK_REALTIME_PATH: &str = "/data/realtime2/KNSW3.txt";
// -- --
//
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

    pub fn get_quality(
        &self,
        wave_height: f64,
        wind_speed: f64,
        wind_direction: f64,
    ) -> &'static Quality {
        match self {
            Self::Atwater(_) | Self::Bradford(_) | Self::Sheboygan(_) => {
                Quality::south(wave_height, wind_speed, wind_direction)
            }
            Self::PortWashington(_) | Self::Racine(_) => {
                Quality::north(wave_height, wind_speed, wind_direction)
            }
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
