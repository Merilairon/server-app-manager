use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/{name}", put(update_role))
}

async fn list_roles(State(_state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!([])))
}

async fn create_role(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn update_role(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}
