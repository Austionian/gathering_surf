use gathering_surf::{ATWATER_PATH, ATWATER_REALTIME_PATH, Settings, get_configuration, startup};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::mocks;

#[derive(Debug)]
pub(crate) struct TestApp {
    pub(crate) addr: SocketAddr,
    pub(crate) mock_client: Option<MockServer>,
}

impl TestApp {
    pub async fn try_new() -> Result<Self, String> {
        let config = Box::new(get_configuration().expect("Failed to get configuration."));

        let addr = init_app(config).await?;

        Ok(Self {
            addr,
            mock_client: None,
        })
    }

    pub async fn try_new_mocked() -> Result<Self, String> {
        let mock_client = MockServer::start().await;

        let config = Box::new({
            let mut config = get_configuration().expect("Failed to get configuration.");
            // Override the API urls with the mock servers' urls
            config.forecast_api.base_url = mock_client.uri();
            config.realtime_api.base_url = mock_client.uri();

            config
        });

        let addr = init_app(config).await?;

        Ok(Self {
            addr,
            mock_client: Some(mock_client),
        })
    }

    pub async fn attach_success_mocks(&self) {
        if let Some(client) = &self.mock_client {
            client
                .register(
                    Mock::given(method("GET"))
                        .and(path(ATWATER_PATH))
                        .respond_with(
                            ResponseTemplate::new(200).set_body_json(mocks::forecast_json()),
                        ),
                )
                .await;
        }
    }

    pub async fn attach_failed_realtime_request_mocks(&self) {
        if let Some(client) = &self.mock_client {
            client
                .register(
                    Mock::given(method("GET"))
                        .and(path(ATWATER_REALTIME_PATH))
                        .respond_with(ResponseTemplate::new(502).set_body_string("Bad gateway"))
                        .expect(2),
                )
                .await;
        }
    }

    pub async fn attach_failed_forecast_request_mocks(&self) {
        if let Some(client) = &self.mock_client {
            client
                .register(
                    Mock::given(method("GET"))
                        .and(path(ATWATER_PATH))
                        .respond_with(ResponseTemplate::new(502).set_body_string("Bad gateway"))
                        .expect(2),
                )
                .await;
        }
    }
}

async fn init_app(config: Box<Settings>) -> Result<SocketAddr, String> {
    let config: &'static Settings = Box::leak(config);

    let (_, app) = startup(&config).await.expect("Unable to start the server.");
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

#[macro_export]
macro_rules! mock_app {
    () => {
        TestApp::try_new_mocked()
            .await
            .expect("Unable to start test server.")
    };
}

#[macro_export]
macro_rules! mocked_happy_path_test_app {
    () => {{
        let app = crate::mock_app!();

        app.attach_success_mocks().await;

        app
    }};
}

#[macro_export]
macro_rules! mocked_unhappy_path_test_app {
    (realtime) => {{
        let app = crate::mock_app!();

        app.attach_failed_realtime_request_mocks().await;

        app
    }};
    (forecast) => {{
        let app = crate::mock_app!();

        app.attach_failed_forecast_request_mocks().await;

        app
    }};
    () => {{
        let app = crate::mock_app!();

        app.attach_failed_forecast_request_mocks().await;
        app.attach_failed_realtime_request_mocks().await;

        app
    }};
}

#[macro_export]
macro_rules! integration_test_app {
    () => {{
        let app = TestApp::try_new()
            .await
            .expect("Unable to start test server.");

        app
    }};
}
