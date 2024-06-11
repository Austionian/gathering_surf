use gathering_surf::startup;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use wiremock::MockServer;

pub(crate) struct TestApp {
    pub(crate) addr: SocketAddr,
    pub(crate) noaa_client: MockServer,
}

pub async fn start_test_app() -> Result<TestApp, String> {
    let noaa_client = MockServer::start().await;

    let app = startup(noaa_client.uri()).expect("Unable to start the server.");
    let listener = TcpListener::bind("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();

    let _ = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    Ok(TestApp { addr, noaa_client })
}
