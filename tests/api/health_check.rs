use crate::{helpers::TestApp, integration_test_app};

#[tokio::test]
async fn the_health_check_works() {
    let app = integration_test_app!();

    let response = reqwest::get(format!("http://{}/health_check", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}
