use crate::{helpers::TestApp, mocked_happy_path_test_app, mocked_unhappy_path_test_app};

#[tokio::test]
async fn it_returns_the_forecast_data_as_json() {
    let app = mocked_happy_path_test_app!();

    let response = reqwest::get(format!("http://{}/api/forecast", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let data = response.text().await.unwrap();

    assert!(data.contains("current_wave_direction"));
    assert!(data.contains("current_wave_height"));
    assert!(data.contains("forecast_as_of"));
    assert!(data.contains("qualities"));

    insta::assert_snapshot!(data);
}

#[tokio::test]
async fn it_handles_a_non_200_response_from_forecast_client_and_retries_once() {
    let app = mocked_unhappy_path_test_app!(forecast);

    let response = reqwest::get(format!("http://{}/api/forecast", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 500);

    let response = response.text().await.unwrap();

    assert!(!response.contains("current_wave_direction"));
    assert!(response.contains("Something went wrong: Non 200 response from NOAA"));
}

#[tokio::test]
async fn forecast_integration_test() {
    let app = TestApp::try_new()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/api/forecast", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let response = response.text().await.unwrap();

    assert!(response.contains("current_wave_direction"));
    assert!(response.contains("current_wave_height"));
    assert!(response.contains("forecast_as_of"));
    assert!(response.contains("qualities"));
}
