use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/apps", get(list_apps))
        .route("/apps/install", post(install_app))
        .route("/apps/{slug}", get(get_app))
        .route("/apps/{slug}/uninstall", post(uninstall_app))
}

async fn list_apps(State(_state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!([])))
}

async fn get_app(
    State(_state): State<AppState>,
    Path(_slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotFound)
}

async fn install_app(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn uninstall_app(
    State(_state): State<AppState>,
    Path(_slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}
