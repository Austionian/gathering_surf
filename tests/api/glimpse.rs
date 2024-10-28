use crate::{helpers::TestApp, integration_test_app};

#[tokio::test]
async fn it_returns_the_glimpse_view() {
    let app = integration_test_app!();

    let response = reqwest::get(format!("http://{}/glimpse", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    let response = response.text().await.unwrap();

    assert!(response.contains("Waves"));
    assert!(!response.contains("Forecast"));
}
