mod config;
mod error;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::{HeaderName, HeaderValue, Method};
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = Config::from_env();
    let state = AppState {
        config: Arc::new(config.clone()),
    };

    let cors = if let Ok(origin) = HeaderValue::from_str(&config.cors_origin) {
        CorsLayer::new()
            .allow_origin(origin)
            .allow_credentials(true)
            .allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("x-csrf-token"),
            ])
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_headers(Any)
            .allow_methods(Any)
    };

    let api = routes::router();

    // SPA fallback: serve compiled Angular bundle from STATIC_DIR.
    // Unknown paths fall back to index.html so client-side routing works.
    let spa = ServeDir::new(&config.static_dir)
        .append_index_html_on_directories(true)
        .not_found_service(ServeDir::new(&config.static_dir).append_index_html_on_directories(true));

    let app = Router::<AppState>::new()
        .route("/healthz", get(healthz))
        .nest("/api/v1", api)
        .fallback_service(spa)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!(%addr, "server-app-manager backend listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn healthz() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
