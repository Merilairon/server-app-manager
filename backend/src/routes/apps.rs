use axum::extract::{Extension, Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::auth::Claims;
use crate::catalog::Catalog;
use crate::error::AppError;
use crate::install::{run as install_app_engine, InstallPayload};
use crate::uninstall;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/apps", get(list_apps))
        .route("/apps/install", post(install_app))
        .route("/apps/{slug}", get(get_app))
        .route("/apps/{slug}/uninstall", post(uninstall_app))
}

fn require_permission(state: &AppState, claims: &Claims, permission: &str) -> Result<(), AppError> {
    if state.roles.has_permission(&claims.role, permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

async fn list_apps(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, AppError> {
    require_permission(&state, &claims, "read:apps")?;
    let catalog = Catalog::load(&state.config)?;
    Ok(Json(json!({ "apps": catalog.apps })))
}

async fn get_app(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    require_permission(&state, &claims, "read:apps")?;
    let catalog = Catalog::load(&state.config)?;
    let app = catalog.get(&slug).ok_or(AppError::NotFound)?;
    Ok(Json(json!({ "app": app })))
}

async fn install_app(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<InstallPayload>,
) -> Result<Json<Value>, AppError> {
    require_permission(&state, &claims, "install:apps")?;
    let catalog = Catalog::load(&state.config)?;
    let app = catalog
        .get(&body.slug)
        .ok_or_else(|| AppError::bad_request(format!("app '{}' not found", body.slug)))?
        .clone();
    let result = install_app_engine(&state.config, &app, &body.values).await?;
    Ok(Json(result))
}

async fn uninstall_app(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    require_permission(&state, &claims, "uninstall:own_apps")?;
    let result = uninstall::run(&state.config, &slug).await?;
    Ok(Json(result))
}
