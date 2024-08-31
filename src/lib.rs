mod configuration;
mod forecast;
mod quality;
mod realtime;
mod routes;
mod spot;
mod utils;
mod water_quality;

use axum::{routing::get, Router};
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::broadcast::Sender;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub use configuration::{get_configuration, Settings};
pub use forecast::*;
pub use quality::*;
pub use realtime::Realtime;
pub use spot::*;
pub use utils::*;
pub use water_quality::*;

#[cfg(not(debug_assertions))]
static TEMPLATES: LazyLock<tera::Tera> =
    LazyLock::new(|| match tera::Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    });

#[cfg(debug_assertions)]
use std::sync::RwLock;

#[cfg(debug_assertions)]
pub static TEMPLATES: LazyLock<Template> = LazyLock::new(|| Template::new());

#[cfg(debug_assertions)]
pub struct Template {
    pub tera: Arc<RwLock<tera::Tera>>,
}

#[cfg(debug_assertions)]
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

#[derive(Clone)]
pub struct AppState {
    breaks: Vec<Location>,
    forecast_url: &'static str,
    realtime_url: &'static str,
    quality_url: &'static str,
    #[cfg(debug_assertions)]
    event_stream: Sender<&'static str>,
}

pub fn startup(
    settings: &'static Settings,
) -> Result<(Option<Sender<&'static str>>, Router), String> {
    #[cfg(debug_assertions)]
    let (tx, _) = tokio::sync::broadcast::channel(10);

    #[cfg(not(debug_assertions))]
    let tx = None;

    // Create an AppState that is shared across the app.
    let state = AppState {
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
