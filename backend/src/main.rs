mod auth;
mod catalog;
mod config;
mod error;
mod install;
mod middleware;
mod rbac;
mod routes;
mod uninstall;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::{HeaderName, HeaderValue, Method};
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::rbac::Roles;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
    pub docker: Arc<bollard::Docker>,
    pub roles: Arc<Roles>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = Config::from_env();

    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .map_err(|e| format!("failed to connect to database: {e}"))?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| format!("failed to run migrations: {e}"))?;
    seed_admin(&pool, &config)
        .await
        .map_err(|e| format!("failed to seed admin user: {e}"))?;

    let docker = bollard::Docker::connect_with_local_defaults()
        .map_err(|e| format!("failed to connect to docker: {e}"))?;

    let roles = Arc::new(
        Roles::from_yaml("roles/roles.yaml").map_err(|e| format!("failed to load roles: {e}"))?,
    );

    let state = AppState {
        config: Arc::new(config.clone()),
        db: pool,
        docker: Arc::new(docker),
        roles,
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

    let security = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    let api = routes::router()
        .layer(security)
        .layer(from_fn_with_state(state.clone(), middleware::auth));

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

async fn seed_admin(
    pool: &sqlx::PgPool,
    config: &Config,
) -> Result<(), crate::error::AppError> {
    let password = match &config.admin_password {
        Some(p) => p,
        None => return Ok(()),
    };

    let count: i64 = sqlx::query_scalar::<sqlx::Postgres, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        sqlx::query::<sqlx::Postgres>(
            "INSERT INTO users (username, email, password_hash, role) \
             VALUES ($1, $2, $3, 'admin') \
             ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash",
        )
        .bind(&config.admin_username)
        .bind(format!("{}@example.com", config.admin_username))
        .bind(&hash)
        .execute(pool)
        .await?;
    }

    Ok(())
}
