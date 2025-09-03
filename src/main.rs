#[cfg(debug_assertions)]
use gathering_surf::init_watchers;

use gathering_surf::{Settings, get_configuration, startup};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, fmt};

#[warn(clippy::pedantic)]
#[allow(unused_variables)]
#[tokio::main]
async fn main() {
    let filter = filter::Targets::new()
        .with_target("tower_http::trace::on_request", Level::TRACE)
        .with_target("tower_http::trace::on_response", Level::TRACE)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_default(Level::INFO);

    let tracing_layer = fmt::layer();

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();

    let config = Box::new(get_configuration().expect("Failed to read configuration."));

    let config: &'static Settings = Box::leak(config);

    let (tx, app) = startup(config).await.expect("Unable to start the server.");

    let address = format!("{}:{}", config.application.host, config.application.port)
        .parse::<SocketAddr>()
        .unwrap();

    #[cfg(debug_assertions)]
    tracing::info!("listening on {}", address);

    #[cfg(debug_assertions)]
    init_watchers!(tx);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
