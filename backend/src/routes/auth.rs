use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;

use crate::auth::{login_response, login_user, Claims, LoginPayload};
use crate::error::AppError;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/me", get(me))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginPayload>,
) -> Result<(HeaderMap, Json<Value>), AppError> {
    eprintln!(
        "login payload username_len={} password_len={}",
        body.username.len(),
        body.password.len()
    );
    let user = login_user(&state.db, &body.username, &body.password).await?;
    let (headers, body) = login_response(&state.config, &user, &state.config.tenant_id)?;
    Ok((headers, Json(body)))
}

async fn forgot_password(
    State(_state): State<AppState>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::NotImplemented)
}

async fn me(
    State(_state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(serde_json::json!({
        "user_id": claims.sub,
        "username": claims.username,
        "role": claims.role,
        "tenant_id": claims.tenant_id,
    })))
}
