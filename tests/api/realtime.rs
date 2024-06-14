use gathering_surf::ATWATER_REALTIME_PATH;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{start_integration_test_app, start_test_app};

#[tokio::test]
async fn integration_it_returns_the_latest_data_as_json() {
    let app = start_integration_test_app()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/api/realtime", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let data = response.text().await.unwrap();

    assert!(data.contains("as_of"));
    assert!(data.contains("wind_direction"));
    assert!(data.contains("air_temp"));
    assert!(data.contains("quality_color"));
    assert!(data.contains("quality_text"));
}

#[tokio::test]
async fn it_handles_a_non_200_response_from_realtime_client_and_retries_once() {
    let app = start_test_app()
        .await
        .expect("Unable to start test server.");

    let mock_client = &app.mock_client.unwrap();
    Mock::given(method("GET"))
        .and(path(ATWATER_REALTIME_PATH))
        .respond_with(ResponseTemplate::new(502).set_body_string("Bad gateway"))
        .expect(2)
        .mount(mock_client)
        .await;

    let response = reqwest::get(format!("http://{}/api/realtime", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 500);

    let data = response.text().await.unwrap();

    assert!(!data.contains("as_of"));
    assert!(data.contains("Something went wrong: Non 200 response from NOAA realtime"));
}
