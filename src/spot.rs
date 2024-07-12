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

#[derive(serde::Serialize)]
pub struct Spot {
    pub forecast_path: &'static str,
    pub realtime_path: &'static str,
    pub fallback_realtime_path: Option<&'static str>,
    pub location: Location,
    pub live_feed_url: Option<&'static str>,
    pub name: &'static str,
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
            location: Location::Atwater,
            live_feed_url: None,
            name: "Atwater",
        };

        match val.get_spot().to_lowercase().as_str() {
            "bradford" => Spot {
                forecast_path: BRADFORD_PATH,
                realtime_path: BRADFORD_REALTIME_PATH,
                fallback_realtime_path: None,
                location: Location::Bradford,
                live_feed_url: None,
                name: "Bradford"
            },
            "port washington" => Spot {
                forecast_path: PORT_WASHINGTON_PATH,
                realtime_path: PORT_WASHINGTON_REALTIME_PATH,
                fallback_realtime_path: None,
                location: Location::PortWashington,
                live_feed_url: None,
                name: "Port Washington"
            },
            "sheboygan - north" => Spot {
                forecast_path: SHEBOYGAN_PATH,
                realtime_path: SHEBOYGAN_REALTIME_PATH,
                fallback_realtime_path: Some(SHEBOYGAN_FALLBACK_REALTIME_PATH),
                location: Location::Sheboygan,
                live_feed_url: Some("https://www.youtube-nocookie.com/embed/p780CkCgNVE?si=qBa_a4twCnOprcG1&amp;controls=0"),
                name: "Sheboygan - North"
            },
            "sheboygan - south" => Spot {
                forecast_path: SHEBOYGAN_SOUTH_PATH,
                realtime_path: SHEBOYGAN_REALTIME_PATH,
                fallback_realtime_path: Some(SHEBOYGAN_FALLBACK_REALTIME_PATH),
                location: Location::SheboyganSouth,
                live_feed_url: Some("https://www.youtube.com/embed/M0Ion4MpsgU?si=yCi2OVy3RIbY_5kC&amp;controls=0"),
                name: "Sheboygan - South"
            },
            "racine" => Spot {
                forecast_path: RACINE_PATH,
                realtime_path: RACINE_REALTIME_PATH,
                fallback_realtime_path: Some(RACINE_FALLBACK_REALTIME_PATH),
                location: Location::Racine,
                live_feed_url: None,
                name: "Racine"
            },
            "atwater" => atwater,
            _ => atwater,
        }
    }
}

// -- Forecast PATHs --
pub const ATWATER_PATH: &str = "/gridpoints/MKX/90,67";
const BRADFORD_PATH: &str = "/gridpoints/MKX/90,66";
const SHEBOYGAN_PATH: &str = "/gridpoints/MKX/94,99";
const SHEBOYGAN_SOUTH_PATH: &str = "/gridpoints/MKX/94,98";
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

#[derive(serde::Serialize)]
pub enum Location {
    Atwater,
    Bradford,
    Sheboygan,
    SheboyganSouth,
    PortWashington,
    Racine,
}

impl Location {
    pub fn get_all() -> Vec<String> {
        Self::into_iter().map(|v| v.to_string()).collect()
    }

    fn into_iter() -> core::array::IntoIter<Self, 6> {
        [
            Self::Atwater,
            Self::Bradford,
            Self::Sheboygan,
            Self::SheboyganSouth,
            Self::PortWashington,
            Self::Racine,
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
            Self::Atwater | Self::Bradford | Self::Sheboygan => {
                Quality::south(wave_height, wind_speed, wind_direction)
            }
            Self::PortWashington | Self::Racine | Self::SheboyganSouth => {
                Quality::north(wave_height, wind_speed, wind_direction)
            }
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atwater => write!(f, "Atwater"),
            Self::Bradford => write!(f, "Bradford"),
            Self::Sheboygan => write!(f, "Sheboygan - North"),
            Self::SheboyganSouth => write!(f, "Sheboygan - South"),
            Self::PortWashington => write!(f, "Port Washington"),
            Self::Racine => write!(f, "Racine"),
        }
    }
}
