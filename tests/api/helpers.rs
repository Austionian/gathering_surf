use gathering_surf::{get_configuration, startup, Settings};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use wiremock::MockServer;

pub(crate) struct TestApp {
    pub(crate) addr: SocketAddr,
    pub(crate) mock_client: Option<MockServer>,
}

pub async fn start_integration_test_app() -> Result<TestApp, String> {
    let config = Box::new(get_configuration().expect("Failed to get configuration."));

    let addr = init_app(config).await?;

    Ok(TestApp {
        addr,
        mock_client: None,
    })
}

pub async fn start_test_app() -> Result<TestApp, String> {
    let mock_client = MockServer::start().await;

    let config = Box::new({
        let mut config = get_configuration().expect("Failed to get configuration.");
        // Override the API urls with the mock servers' urls
        config.forecast_api.base_url = mock_client.uri();
        config.realtime_api.base_url = mock_client.uri();

        config
    });

    let addr = init_app(config).await?;

    Ok(TestApp {
        addr,
        mock_client: Some(mock_client),
    })
}

async fn init_app(config: Box<Settings>) -> Result<SocketAddr, String> {
    let config: &'static Settings = Box::leak(config);

    let app = startup(&config).expect("Unable to start the server.");
    let listener = TcpListener::bind("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();

    let _ = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    Ok(addr)
}
