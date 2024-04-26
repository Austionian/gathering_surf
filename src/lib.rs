mod configuration;
mod forecast;
mod latest;
mod quality;
mod routes;
mod spot;
mod utils;

use axum::{routing::get, Router};
use lazy_static::lazy_static;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub use configuration::get_configuration;
pub use forecast::*;
pub use latest::Latest;
pub use quality::*;
pub use spot::*;
pub use utils::*;

lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        match tera::Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    };
}

#[derive(Clone, serde::Serialize)]
pub struct Break {
    name: &'static str,
}

#[derive(Clone, serde::Serialize)]
pub struct AppState {
    breaks: [Break; 2],
}

pub fn startup() -> Result<Router, String> {
    // Create an AppState that is shared across the app.
    let state = AppState {
        breaks: [Break { name: "Atwater" }, Break { name: "Bradford" }],
    };

    let api = Router::new()
        .route("/latest", get(routes::latest))
        .route("/forecast", get(routes::forecast));

    // Create the Axum router.
    Ok(Router::new()
        // this will serve everything in /assets, including your minified stylesheet, e.g.
        // /assets/styles.css.
        .nest_service("/assets", ServeDir::new("assets"))
        // attaches the root route to the root.
        .route("/", get(routes::root))
        .nest("/api", api)
        .fallback(routes::handle_404)
        // binds the telemetry.
        .layer(TraceLayer::new_for_http())
        // adds the app state that will be available across Axum routes.
        .with_state(Arc::new(state))
        // adds the health check after the tracing to exclude from logs.
        .route("/health_check", get(routes::health_check)))
}
