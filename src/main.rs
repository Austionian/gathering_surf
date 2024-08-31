use gathering_surf::{get_configuration, startup, Settings};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, fmt};

#[cfg(debug_assertions)]
use gathering_surf::TEMPLATES;

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

    let (tx, app) = startup(config).expect("Unable to start the server.");

    let address = format!("{}:{}", config.application.host, config.application.port)
        .parse::<SocketAddr>()
        .unwrap();

    #[cfg(debug_assertions)]
    tracing::info!("listening on {}", address);

    #[cfg(debug_assertions)]
    use notify::Watcher;

    #[cfg(debug_assertions)]
    let tx = tx.expect("available only in debug.");
    #[cfg(debug_assertions)]
    let js_tx = tx.clone();

    #[cfg(debug_assertions)]
    #[allow(unused_must_use)]
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(_) => {
            // Get a writer lock to the templates struct
            let mut tera = TEMPLATES.tera.write().unwrap();

            // Reload tera with template changes
            match tera.full_reload() {
                Ok(_) => tracing::trace!("templates reloaded"),
                Err(e) => tracing::error!("failed to reload templates: {e}"),
            }

            // Notify browswer to reload.
            //
            // The channel is thrown away after this so no need to
            // unwrap as it will cause a poision lock error.
            tx.send("reload");
        }
        Err(e) => tracing::warn!("watch error: {e:?}"),
    })
    .unwrap();

    #[cfg(debug_assertions)]
    #[allow(unused_must_use)]
    let mut js_watcher = notify::recommended_watcher(move |res| match res {
        Ok(_) => {
            // Notify browswer to reload.
            //
            // The channel is thrown away after this so no need to
            // unwrap as it will cause a poision lock error.
            js_tx.send("reload");
        }
        Err(e) => tracing::warn!("watch error: {e:?}"),
    })
    .unwrap();

    #[cfg(debug_assertions)]
    tracing::info!("watching templates for changes");

    #[cfg(debug_assertions)]
    watcher
        .watch(
            &std::path::PathBuf::from("templates"),
            notify::RecursiveMode::Recursive,
        )
        .unwrap();

    #[cfg(debug_assertions)]
    js_watcher
        .watch(
            &std::path::PathBuf::from("client"),
            notify::RecursiveMode::Recursive,
        )
        .unwrap();

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
