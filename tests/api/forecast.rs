use crate::helpers::start_test_app;

#[tokio::test]
async fn it_returns_the_forecast_data_as_json() {
    let addr = start_test_app()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/api/forecast", &addr))
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
