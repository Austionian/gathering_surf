/// Contains quality text and associated color.
pub struct Quality(pub &'static str, pub &'static str);

pub static GOOD: Quality = Quality("Good", "#0bd674");
pub static OK: Quality = Quality("Fair to Good", "#ffcd1e");
pub static POOR: Quality = Quality("Poor", "#ff9500");
pub static VERY_POOR: Quality = Quality("Very Poor", "#f4496d");

pub trait GetQuality {
    fn get_quality(wind_speed: f64, wind_direction: f64) -> &'static Quality;
}
