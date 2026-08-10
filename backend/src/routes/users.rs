use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/me", put(update_me))
        .route("/users/{id}/password", put(update_password))
}

async fn list_users(State(_state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!([])))
}

async fn create_user(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn update_me(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn update_password(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}
