use crate::helpers::start_test_app;

#[tokio::test]
async fn it_returns_the_latest_data_as_json() {
    let app = start_test_app()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/api/latest", &app.addr))
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
