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
                        "validTime": "2024-06-10T20:00:00+00:00/PT1H",
                        "value": 15.555555555555555
                    },
                ]},
                 "waveHeight": {
            "uom": "wmoUnit:m",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT3H",
                    "value": 0.30480000000000002
                }]},
                "wavePeriod": {
            "uom": "nwsUnit:s",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT2H",
                    "value": 4
                }]},
                "waveDirection": {
            "uom": "wmoUnit:degree_(angle)",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT3H",
                    "value": 30
                }]},
                 "probabilityOfThunder": {
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT22H",
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
                    "validTime": "2024-06-10T20:00:00+00:00/PT1H",
                    "value": 25.928000000000001
                }]},
                "windSpeed": {
            "uom": "wmoUnit:km_h-1",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT1H",
                    "value": 18.52
                }]},
                "windDirection": {
            "uom": "wmoUnit:degree_(angle)",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT1H",
                    "value": 30
                }]},
                "skyCover": {
            "uom": "wmoUnit:percent",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT4H",
                    "value": 4
                }]},
                    "dewpoint": {
            "uom": "wmoUnit:degC",
            "values": [
                {
                    "validTime": "2024-06-10T20:00:00+00:00/PT4H",
                    "value": 6.666666666666667
                }]}
        }})
    })
}
