use gathering_surf::ATWATER_PATH;
use gathering_surf::{get_configuration, startup, Settings};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

pub(crate) struct TestApp {
    pub(crate) addr: SocketAddr,
    pub(crate) mock_client: Option<MockServer>,
}

pub async fn start_integration_test_app() -> Result<TestApp, String> {
    let config = Box::new(get_configuration().expect("Failed to get configuration."));

    let addr = init_app(config).await?;

    Ok(TestApp {
        addr,
        mock_client: None,
    })
}

pub async fn start_test_app() -> Result<TestApp, String> {
    let mock_client = MockServer::start().await;

    let config = Box::new({
        let mut config = get_configuration().expect("Failed to get configuration.");
        // Override the API urls with the mock servers' urls
        config.forecast_api.base_url = mock_client.uri();
        config.realtime_api.base_url = mock_client.uri();

        config
    });

    let addr = init_app(config).await?;

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
        .mount(&mock_client)
        .await;

    Ok(TestApp {
        addr,
        mock_client: Some(mock_client),
    })
}

async fn init_app(config: Box<Settings>) -> Result<SocketAddr, String> {
    let config: &'static Settings = Box::leak(config);

    let app = startup(&config).expect("Unable to start the server.");
    let listener = TcpListener::bind("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();

    let _ = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    Ok(addr)
}
