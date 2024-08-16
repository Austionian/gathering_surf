use gathering_surf::ATWATER_PATH;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{start_integration_test_app, start_test_app};

#[tokio::test]
async fn it_returns_the_forecast_data_as_json() {
    let app = start_test_app()
        .await
        .expect("Unable to start test server.");

    Mock::given(method("GET"))
        .and(path(ATWATER_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
                        {
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
        }}
            )))
        .mount(&app.mock_client.unwrap())
        .await;

    let response = reqwest::get(format!("http://{}/api/forecast", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let data = response.text().await.unwrap();

    assert!(data.contains("current_wave_direction"));
    assert!(data.contains("current_wave_height"));
    assert!(data.contains("forecast_as_of"));
    assert!(data.contains("graph_max"));
    assert!(data.contains("qualities"));

    insta::assert_snapshot!(data);
}

#[tokio::test]
async fn it_handles_a_non_200_response_from_forecast_client_and_retries_once() {
    let app = start_test_app()
        .await
        .expect("Unable to start test server.");

    let mock_client = &app.mock_client.unwrap();
    Mock::given(method("GET"))
        .and(path(ATWATER_PATH))
        .respond_with(ResponseTemplate::new(502).set_body_string("Bad gateway"))
        .expect(2)
        .mount(mock_client)
        .await;

    let response = reqwest::get(format!("http://{}/api/forecast", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 500);

    let data = response.text().await.unwrap();

    assert!(!data.contains("current_wave_direction"));
    assert!(data.contains("Something went wrong: Non 200 response from NOAA"));
}

#[tokio::test]
async fn forecast_integration_test() {
    let app = start_integration_test_app()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/api/forecast", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let data = response.text().await.unwrap();

    assert!(data.contains("current_wave_direction"));
    assert!(data.contains("current_wave_height"));
    assert!(data.contains("forecast_as_of"));
    assert!(data.contains("graph_max"));
    assert!(data.contains("qualities"));
}
