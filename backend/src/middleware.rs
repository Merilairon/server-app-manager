use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::cookie_value;
use crate::error::AppError;
use crate::AppState;

pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path();
    if path == "/auth/login" || path == "/auth/forgot-password" {
        return Ok(next.run(req).await);
    }

    let token = cookie_value(req.headers(), "token").ok_or(AppError::Unauthorized)?;
    let claims = crate::auth::verify_token(&state.config, &token)?;
    req.extensions_mut().insert(claims);

    if is_mutating(&req) {
        let csrf_cookie = cookie_value(req.headers(), "csrf").ok_or(AppError::Forbidden)?;
        let csrf_header = req
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if csrf_cookie != csrf_header {
            return Err(AppError::Forbidden);
        }
    }

    Ok(next.run(req).await)
}

fn is_mutating(req: &Request) -> bool {
    matches!(req.method().as_str(), "POST" | "PUT" | "PATCH" | "DELETE")
}
