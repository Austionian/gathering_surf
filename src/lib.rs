mod configuration;
mod forecast;
mod quality;
mod realtime;
mod routes;
mod spot;
mod utils;
mod water_quality;

use axum::{Router, routing::get};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub use configuration::{Settings, get_configuration};
pub use forecast::*;
pub use quality::*;
pub use realtime::Realtime;
pub use spot::*;
pub use utils::*;
pub use water_quality::*;

templates::init!();

#[derive(Clone)]
pub struct AppState {
    redis_pool: Pool<RedisConnectionManager>,
    breaks: Vec<Location>,
    forecast_url: &'static str,
    realtime_url: &'static str,
    quality_url: &'static str,
    #[cfg(debug_assertions)]
    event_stream: Sender<&'static str>,
}

pub async fn startup(
    settings: &'static Settings,
) -> Result<(Option<Sender<&'static str>>, Router), String> {
    #[cfg(debug_assertions)]
    let (tx, _) = tokio::sync::broadcast::channel(10);

    #[cfg(not(debug_assertions))]
    let tx = None;

    let redis_manager = RedisConnectionManager::new(format!(
        "redis://{}:{}",
        std::env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string()),
        std::env::var("REDIS_PORT").unwrap_or("6379".to_string())
    ))
    .unwrap();

    let redis_pool = bb8::Pool::builder().build(redis_manager).await.unwrap();

    // Create an AppState that is shared across the app.
    let state = AppState {
        redis_pool,
        breaks: Location::get_all(),
        forecast_url: &settings.forecast_api.base_url,
        realtime_url: &settings.realtime_api.base_url,
        quality_url: &settings.quality_api.base_url,
        #[cfg(debug_assertions)]
        event_stream: tx.clone(),
    };

    #[cfg(debug_assertions)]
    let watch_state = state.clone();

    let api = Router::new()
        .route("/realtime", get(routes::realtime))
        .route("/forecast", get(routes::forecast));

    #[cfg(debug_assertions)]
    let tx = Some(tx);

    let app = Router::new()
        // this will serve everything in /assets, including your minified stylesheet, e.g.
        // /assets/styles.css.
        .nest_service("/assets", ServeDir::new("assets"))
        // attaches the root route to the root.
        .route("/", get(routes::root))
        .route("/glimpse", get(routes::glimpse))
        .nest("/api", api)
        .fallback(routes::handle_404)
        // binds the telemetry.
        .layer(TraceLayer::new_for_http())
        // adds the app state that will be available across Axum routes.
        .with_state(Arc::new(state))
        // adds the health check after the tracing to exclude from logs.
        .route("/health_check", get(routes::health_check));

    #[cfg(debug_assertions)]
    let watch_router = Router::new()
        .route("/watch", get(routes::watch))
        .with_state(Arc::new(watch_state));

    #[cfg(debug_assertions)]
    let app: Router = Router::new().merge(app).merge(watch_router);

    // Create the Axum router.
    Ok((tx, app))
}

#[macro_export]
macro_rules! init_watchers {
    ($tx:expr) => {
        use gathering_surf::TEMPLATES;
        use notify::Watcher;

        let tx = $tx.expect("available only in debug.");
        let js_tx = tx.clone();

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

        tracing::info!("watching templates for changes");

        watcher
            .watch(
                &std::path::PathBuf::from("templates"),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();

        js_watcher
            .watch(
                &std::path::PathBuf::from("client"),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();
    };
}

mod templates {
    /// Creates a static TEMPLATES
    macro_rules! init {
        () => {
            use std::sync::LazyLock;

            #[cfg(debug_assertions)]
            templates::debug!();

            #[cfg(not(debug_assertions))]
            static TEMPLATES: LazyLock<tera::Tera> =
                LazyLock::new(|| match tera::Tera::new("templates/**/*") {
                    Ok(t) => t,
                    Err(e) => {
                        println!("Parsing error(s): {}", e);
                        ::std::process::exit(1);
                    }
                });
        };
    }

    /// Creates a TEMPLATES which reloads when /templates change so the
    /// whole binary doesn't need to be recompilied
    #[cfg(debug_assertions)]
    macro_rules! debug {
        () => {
            use std::sync::RwLock;

            pub static TEMPLATES: LazyLock<Template> = LazyLock::new(Template::new);

            pub struct Template {
                pub tera: Arc<RwLock<tera::Tera>>,
            }

            impl Template {
                fn new() -> Self {
                    match tera::Tera::new("templates/**/*") {
                        Ok(t) => Self {
                            tera: Arc::new(RwLock::new(t)),
                        },
                        Err(e) => {
                            println!("Parsing error(s): {}", e);
                            ::std::process::exit(1);
                        }
                    }
                }

                fn render(&self, path: &str, context: &tera::Context) -> tera::Result<String> {
                    let tera = self.tera.read().unwrap();

                    tera.render(path, context)
                }
            }
        };
    }

    #[cfg(debug_assertions)]
    pub(crate) use debug;

    pub(crate) use init;
}
