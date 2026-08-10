use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/containers/launch", post(launch))
        .route("/containers/start/{id}", post(start))
        .route("/containers/stop/{id}", post(stop))
        .route("/containers/{id}", get(inspect))
        .route("/containers/{id}/health", get(health))
        .route("/containers/{id}/logs", get(logs))
}

async fn launch(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn start(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn stop(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn inspect(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<Value>, AppError> {
    Err(AppError::NotFound)
}

async fn health(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "status": "unknown",
        "last_checked": null,
    })))
}

async fn logs(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}
