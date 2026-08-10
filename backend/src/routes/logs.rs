use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub app_slug: Option<String>,
    pub date: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/logs", get(list_logs))
}

async fn list_logs(
    State(_state): State<AppState>,
    Query(_q): Query<LogsQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "lines": [],
        "app_slug": null,
        "date": null,
    })))
}
