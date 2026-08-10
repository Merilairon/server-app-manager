use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not implemented")]
    NotImplemented,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("database error")]
    Sqlx(#[from] sqlx::Error),

    #[error("bcrypt error")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("jwt error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("docker error: {0}")]
    Docker(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn docker(msg: impl Into<String>) -> Self {
        Self::Docker(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotImplemented => (StatusCode::NOT_IMPLEMENTED, "not_implemented"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            AppError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            AppError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "hash_error"),
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "jwt_error"),
            AppError::Docker(_) => (StatusCode::INTERNAL_SERVER_ERROR, "docker_error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = json!({
            "code": code,
            "message": self.to_string(),
        });

        (status, Json(body)).into_response()
    }
}
