use super::Quality;
use std::sync::OnceLock;

#[derive(serde::Deserialize, Debug)]
pub struct SpotParam {
    pub spot: Option<Location>,
}

impl SpotParam {
    pub fn get_spot(&mut self) -> Location {
        // Get the selected spot, fallback to Atwater
        self.spot.take().unwrap_or(Location::Atwater)
    }
}

#[derive(serde::Serialize, Debug)]
pub struct Spot {
    pub forecast_path: &'static str,
    pub realtime_path: &'static str,
    pub quality_query: &'static str,
    pub status_query: &'static str,
    pub fallback_realtime_path: Option<&'static str>,
    pub location: Location,
    pub live_feed_url: Option<&'static str>,
    pub name: &'static str,
    pub has_bouy: bool,
}

fn get_status_query(id: &str) -> String {
    format!("?f=json&objectIds={id}&outFields=MAP_STATUS")
}

fn get_quality_query(id: &str) -> String {
    format!(
        "?f=json&objectIds={id}&outFields=ECOLIPRONAME%2CECOLIVALUE%2CISSUED%2COGW_BEACH_NAME_TEXT%2CSAMPLEDATE%2CSTATIONNAME%2CSTATUS%2CWATERTEMP%2COBJECTID"
    )
}

macro_rules! quality_once_lock {
    ($id:expr) => {{
        // Identities created in Rust macros are hygienic, attaching invisible syntax context to
        // all indentifiers so repeated static QUALITYs never collide
        static QUALITY: OnceLock<String> = OnceLock::new();

        QUALITY.get_or_init(|| get_quality_query($id))
    }};
}

macro_rules! status_once_lock {
    ($id:expr) => {{
        static STATUS: OnceLock<String> = OnceLock::new();

        STATUS.get_or_init(|| get_status_query($id))
    }};
}

impl From<SpotParam> for Spot {
    fn from(mut val: SpotParam) -> Self {
        match val.get_spot() {
            Location::Bradford => Spot {
                forecast_path: BRADFORD_PATH,
                realtime_path: BRADFORD_REALTIME_PATH,
                quality_query: quality_once_lock!(BRADFORD_QUALITY_ID),
                status_query: status_once_lock!(BRADFORD_QUALITY_ID),
                fallback_realtime_path: None,
                location: Location::Bradford,
                live_feed_url: None,
                name: "Bradford",
                has_bouy: false,
            },
            Location::PortWashington => Spot {
                forecast_path: PORT_WASHINGTON_PATH,
                realtime_path: PORT_WASHINGTON_REALTIME_PATH,
                quality_query: quality_once_lock!(PORT_WASHINGTON_QUALITY_ID),
                status_query: status_once_lock!(PORT_WASHINGTON_QUALITY_ID),
                fallback_realtime_path: None,
                location: Location::PortWashington,
                live_feed_url: None,
                name: "Port Washington",
                has_bouy: false,
            },
            Location::Sheboygan => Spot {
                forecast_path: SHEBOYGAN_PATH,
                realtime_path: SHEBOYGAN_REALTIME_PATH,
                quality_query: quality_once_lock!(SHEBOYGAN_NORTH_QUALITY_ID),
                status_query: status_once_lock!(SHEBOYGAN_NORTH_QUALITY_ID),
                fallback_realtime_path: Some(SHEBOYGAN_FALLBACK_REALTIME_PATH),
                location: Location::Sheboygan,
                live_feed_url: Some(
                    "https://www.youtube-nocookie.com/embed/p780CkCgNVE?si=qBa_a4twCnOprcG1&amp;controls=0",
                ),
                name: "Sheboygan - North",
                has_bouy: true,
            },
            Location::SheboyganSouth => Spot {
                forecast_path: SHEBOYGAN_SOUTH_PATH,
                realtime_path: SHEBOYGAN_REALTIME_PATH,
                quality_query: quality_once_lock!(SHEBOYGAN_SOUTH_QUALITY_ID),
                status_query: status_once_lock!(SHEBOYGAN_SOUTH_QUALITY_ID),
                fallback_realtime_path: Some(SHEBOYGAN_FALLBACK_REALTIME_PATH),
                location: Location::SheboyganSouth,
                live_feed_url: Some(
                    "https://www.youtube.com/embed/M0Ion4MpsgU?si=yCi2OVy3RIbY_5kC&amp;controls=0",
                ),
                name: "Sheboygan - South",
                has_bouy: true,
            },
            Location::Racine => Spot {
                forecast_path: RACINE_PATH,
                realtime_path: RACINE_REALTIME_PATH,
                quality_query: quality_once_lock!(RACINE_QUALITY_ID),
                status_query: status_once_lock!(RACINE_QUALITY_ID),
                fallback_realtime_path: Some(RACINE_FALLBACK_REALTIME_PATH),
                location: Location::Racine,
                live_feed_url: None,
                name: "Racine",
                has_bouy: true,
            },
            Location::Atwater => Spot {
                forecast_path: ATWATER_PATH,
                realtime_path: ATWATER_REALTIME_PATH,
                quality_query: quality_once_lock!(ATWATER_QUALITY_ID),
                status_query: status_once_lock!(ATWATER_QUALITY_ID),
                fallback_realtime_path: Some(BRADFORD_REALTIME_PATH),
                location: Location::Atwater,
                live_feed_url: None,
                name: "Atwater",
                has_bouy: true,
            },
        }
    }
}

// -- Forecast Paths --
pub const ATWATER_PATH: &str = "/gridpoints/MKX/90,67";
const BRADFORD_PATH: &str = "/gridpoints/MKX/90,66";
const SHEBOYGAN_PATH: &str = "/gridpoints/MKX/94,99";
const SHEBOYGAN_SOUTH_PATH: &str = "/gridpoints/MKX/94,98";
const PORT_WASHINGTON_PATH: &str = "/gridpoints/MKX/91,80";
const RACINE_PATH: &str = "/gridpoints/MKX/94,52";
// -- --

// -- Realtime Paths --
//
// -- Bouy Paths --
pub const ATWATER_REALTIME_PATH: &str = "/data/realtime2/45013.txt";
const SHEBOYGAN_REALTIME_PATH: &str = "/data/realtime2/45218.txt";
const RACINE_REALTIME_PATH: &str = "/data/realtime2/45199.txt";
// -- --

// -- Land Based Weather Station Paths --
const BRADFORD_REALTIME_PATH: &str = "/data/realtime2/MLWW3.txt";
const PORT_WASHINGTON_REALTIME_PATH: &str = "/data/realtime2/PWAW3.txt";

const SHEBOYGAN_FALLBACK_REALTIME_PATH: &str = "/data/realtime2/SGNW3.txt";
const RACINE_FALLBACK_REALTIME_PATH: &str = "/data/realtime2/KNSW3.txt";
// -- --
//
// -- Water Quality Queries --
// Base path for all spot queries
pub const QUALITY_PATH: &str = "/arcgis2/rest/services/OGW_Beach_Monitoring/OGW_Beach_Monitoring_Locations_Ext/MapServer/0/query";
//
const ATWATER_QUALITY_ID: &str = "171";
const RACINE_QUALITY_ID: &str = "204";
const BRADFORD_QUALITY_ID: &str = "192";
const SHEBOYGAN_NORTH_QUALITY_ID: &str = "170";
const SHEBOYGAN_SOUTH_QUALITY_ID: &str = "382";
const PORT_WASHINGTON_QUALITY_ID: &str = "100";
// -- --
//
// -- --

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Location {
    Atwater,
    Bradford,
    #[serde(rename = "Sheboygan - North")]
    Sheboygan,
    #[serde(rename = "Sheboygan - South")]
    SheboyganSouth,
    #[serde(rename = "Port Washington")]
    PortWashington,
    Racine,
}

impl Location {
    pub fn get_all() -> Vec<Location> {
        Self::into_iter().collect()
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
