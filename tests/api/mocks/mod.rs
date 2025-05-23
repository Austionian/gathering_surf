use serde_json::json;
use std::sync::OnceLock;

pub fn forecast_json() -> &'static serde_json::Value {
    static FORECAST_JSON: OnceLock<serde_json::Value> = OnceLock::new();
    FORECAST_JSON.get_or_init(|| {
        json!({
        "@context": [
            "https://geojson.org/geojson-ld/geojson-context.jsonld",
            {
                "@version": "1.1",
                "wmoUnit": "https://codes.wmo.int/common/unit/",
                "nwsUnit": "https://api.weather.gov/ontology/unit/"
            }
        ],
        "id": "https://api.weather.gov/gridpoints/MKX/90,67",
        "type": "Feature",
        "properties": {
            "@id": "https://api.weather.gov/gridpoints/MKX/90,67",
            "@type": "wx:Gridpoint",
            "updateTime": "2024-06-11T02:54:57+00:00",
            "validTimes": "2024-06-10T20:00:00+00:00/P7DT5H",
            "elevation": {
                "unitCode": "wmoUnit:m",
                "value": 175.86959999999999
            },
            "forecastOffice": "https://api.weather.gov/offices/MKX",
            "gridId": "MKX",
            "gridX": "90",
            "gridY": "67",
            "temperature": {
                "uom": "wmoUnit:degC",
                "values": [
                    {
                        "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                        "value": 15.555555555555555
                    },
                ]},
                 "waveHeight": {
            "uom": "wmoUnit:m",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 0.30480000000000002
                }]},
                "wavePeriod": {
            "uom": "nwsUnit:s",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 4
                }]},
                "waveDirection": {
            "uom": "wmoUnit:degree_(angle)",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 30
                }]},
                 "probabilityOfThunder": {
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 0
                }]},
                "probabilityOfPrecipitation": {
            "uom": "wmoUnit:percent",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 0
                }]},
                "windGust": {
            "uom": "wmoUnit:km_h-1",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 25.928000000000001
                }]},
                "windSpeed": {
            "uom": "wmoUnit:km_h-1",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 18.52
                }]},
                "windDirection": {
            "uom": "wmoUnit:degree_(angle)",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 30
                }]},
                "skyCover": {
            "uom": "wmoUnit:percent",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 4
                }]},
                    "dewpoint": {
            "uom": "wmoUnit:degC",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT20H",
                    "value": 6.666666666666667
                }]}
        }})
    })
}

pub const REALTIME_RESPONSE: &str = r#"#YY  MM DD hh mm WDIR WSPD GST  WVHT   DPD   APD MWD   PRES  ATMP  WTMP  DEWP  VIS PTDY  TIDE
#yr  mo dy hr mn degT m/s  m/s     m   sec   sec degT   hPa  degC  degC  degC  nmi  hPa    ft
2025 05 23 18 30  90  2.0  3.0   0.3     5    MM  24 1020.4   7.7   7.7    MM   MM   MM    MM
2025 05 23 18 00 100  4.0  4.0   0.3     5    MM  22 1020.2   7.7   7.7    MM   MM +0.5    MM
2025 05 23 17 30 140  3.0  3.0   0.2    MM    MM  19 1020.3   7.7   7.7    MM   MM   MM    MM
2025 05 23 17 00 120  3.0  4.0   0.2    MM    MM  18 1020.3   7.9   7.8    MM   MM +0.9    MM
2025 05 23 16 30 100  3.0  3.0   0.3     5    MM  23 1020.2   8.0   7.6    MM   MM   MM    MM
2025 05 23 16 00 110  2.0  3.0   0.3     5    MM  20 1020.3   7.9   7.6    MM   MM +1.4    MM
2025 05 23 15 30  80  4.0  5.0   0.3     5    MM  20 1019.8   7.9   7.5    MM   MM   MM    MM
2025 05 23 15 00  90  3.0  3.0   0.3     5    MM  21 1019.7   8.3   7.5    MM   MM +0.9    MM
2025 05 23 14 30  80  3.0  3.0   0.3     5    MM  22 1019.7   8.1   7.4    MM   MM   MM    MM
2025 05 23 14 00  10  2.0  2.0   0.3     5    MM  24 1019.4   8.2   7.2    MM   MM +1.0    MM
2025 05 23 13 30  20  3.0  3.0   0.3     5    MM  22 1019.3   7.8   7.1    MM   MM   MM    MM
2025 05 23 13 00  10  5.0  6.0   0.4     5    MM  21 1018.9   7.9   7.1    MM   MM +1.0    MM
2025 05 23 12 30 340  5.0  6.0   0.4     5    MM  22 1018.9   8.0   7.1    MM   MM   MM    MM
2025 05 23 12 00 320  2.0  3.0   0.4     5    MM  22 1018.8   7.6   7.1    MM   MM +1.0    MM
2025 05 23 11 30 330  3.0  3.0   0.4     5    MM  20 1018.6   7.2   7.1    MM   MM   MM    MM
2025 05 23 11 00 290  3.0  4.0   0.4     5    MM  19 1018.4   7.1   7.1    MM   MM +0.8    MM
2025 05 23 10 30 300  3.0  4.0   0.4     5    MM  20 1018.2   7.0   7.1    MM   MM   MM    MM
2025 05 23 10 00 290  3.0  3.0   0.4     5    MM  18 1017.9   7.0   7.1    MM   MM -0.0    MM
2025 05 23 09 30 290  2.0  3.0   0.4     5    MM  20 1017.8   6.9   7.1    MM   MM   MM    MM
2025 05 23 09 00 310  2.0  2.0   0.4     5    MM  20 1017.8   6.9   7.1    MM   MM -0.5    MM
2025 05 23 08 30 300  2.0  2.0   0.4     5    MM  21 1017.8   7.0   7.1    MM   MM   MM    MM
2025 05 23 08 00 320  2.0  2.0   0.4     5    MM  21 1017.6   6.9   7.1    MM   MM -1.3    MM
2025 05 23 07 30 320  1.0  2.0   0.4     5    MM  20 1017.7   7.0   7.1    MM   MM   MM    MM
2025 05 23 07 00 290  1.0  2.0   0.4     5    MM  22 1018.0   7.1   7.1    MM   MM -1.0    MM
2025 05 23 06 30 290  1.0  2.0   0.4     5    MM  21 1018.2   7.0   7.1    MM   MM   MM    MM
2025 05 23 06 00  10  1.0  2.0   0.4     5    MM  18 1018.3   6.8   7.2    MM   MM -1.0    MM
2025 05 23 05 30  MM  0.0  1.0   0.5     5    MM  18 1018.6   7.1   7.2    MM   MM   MM    MM
2025 05 23 05 00 300  1.0  2.0   0.5     5    MM  20 1018.9   7.3   7.2    MM   MM +0.3    MM
2025 05 23 04 30 260  1.0  1.0   0.5     5    MM  20 1019.0   7.1   7.2    MM   MM   MM    MM
2025 05 23 04 00 240  1.0  1.0   0.5     5    MM  19 1019.0   7.5   7.2    MM   MM +1.0    MM
2025 05 23 03 30  MM  0.0  1.0   0.5     5    MM  23 1019.2   7.5   7.2    MM   MM   MM    MM
2025 05 23 03 00  MM  0.0  1.0   0.5     5    MM  22 1019.3   7.7   7.2    MM   MM +1.7    MM
2025 05 23 02 30 270  1.0  2.0   0.6     5    MM  21 1018.9   7.4   7.2    MM   MM   MM    MM
2025 05 23 02 00 110  1.0  2.0   0.5     5    MM  23 1018.6   7.9   7.2    MM   MM +1.1    MM
2025 05 23 01 30 110  1.0  1.0   0.5     5    MM  23 1018.2   7.6   7.3    MM   MM   MM    MM
2025 05 23 01 00 120  1.0  2.0   0.5     5    MM  26 1018.0   8.1   7.4    MM   MM +0.8    MM
2025 05 23 00 30  MM  0.0  1.0   0.5     5    MM  29 1017.9   8.3   7.4    MM   MM   MM    MM
2025 05 23 00 00 170  1.0  2.0   0.5     5    MM  27 1017.6   8.4   7.5    MM   MM +0.7    MM
2025 05 22 23 30  MM  0.0  1.0   0.5     5    MM  28 1017.6   8.5   7.4    MM   MM   MM    MM
2025 05 22 23 00 140  1.0  2.0   0.5     5    MM  30 1017.5   8.4   7.4    MM   MM +0.6    MM
2025 05 22 22 30 120  1.0  2.0   0.5     5    MM  30 1017.3   8.3   7.4    MM   MM   MM    MM
2025 05 22 22 00 100  1.0  2.0   0.6     5    MM  30 1017.2   8.8   7.3    MM   MM +0.5    MM
2025 05 22 21 30 320  1.0  1.0   0.5     5    MM  28 1017.1   9.0   7.3    MM   MM   MM    MM
2025 05 22 21 00 280  2.0  3.0   0.5     5    MM  26 1016.9   8.5   7.2    MM   MM +0.6    MM
2025 05 22 20 30 310  2.0  3.0   0.5     5    MM  28 1017.1   7.9   7.2    MM   MM   MM    MM
2025 05 22 20 00 350  1.0  2.0   0.5     5    MM  26 1016.9   7.8   7.2    MM   MM +0.9    MM
2025 05 22 19 30  10  2.0  3.0   0.4     4    MM  28 1016.8   7.6   7.2    MM   MM   MM    MM
2025 05 22 19 00  30  1.0  3.0   0.4     4    MM  28 1016.7   7.4   7.0    MM   MM +1.3    MM
2025 05 22 18 30  80  2.0  3.0   0.4     4    MM  25 1016.3   7.5   7.0    MM   MM   MM    MM
2025 05 22 18 00  40  2.0  3.0   0.4     4    MM  25 1016.3   7.5   6.9    MM   MM +1.3    MM
2025 05 22 17 30  50  4.0  5.0   0.4     4    MM  24 1016.1   7.7   6.8    MM   MM   MM    MM
2025 05 22 17 00  50  5.0  7.0   0.4     4    MM  21 1016.0   7.6   6.8    MM   MM +1.5    MM"#;
