use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/me", get(me))
}

async fn login(State(_state): State<AppState>, Json(_body): Json<Value>) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn forgot_password(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn me(State(_state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "user_id": null,
        "role": null,
        "tenant_id": null,
    })))
}
