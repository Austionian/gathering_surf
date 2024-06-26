use crate::helpers::start_test_app;

#[tokio::test]
async fn the_health_check_works() {
    let app = start_test_app()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/health_check", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}
