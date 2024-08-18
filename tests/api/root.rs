use crate::helpers::TestApp;

#[tokio::test]
async fn it_returns_the_index() {
    let app = TestApp::try_new_mocked()
        .await
        .expect("Unable to start test server.");

    let response = reqwest::get(format!("http://{}/", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    let response = response.text().await.unwrap();
    assert!(response.contains("Gathering Surf"));
}
