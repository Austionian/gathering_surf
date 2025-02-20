use crate::{helpers::TestApp, integration_test_app};

#[tokio::test]
async fn an_invalid_url_returns_the_404_not_found() {
    let app = integration_test_app!();

    let response = reqwest::get(format!("http://{}/four-oh-four-please", &app.addr))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 404);
    assert!(
        response
            .text()
            .await
            .unwrap()
            .contains("Nothing to see here")
    );
}
