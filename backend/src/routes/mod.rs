pub mod auth;
pub mod apps;
pub mod containers;
pub mod users;
pub mod roles;
pub mod logs;

use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(apps::router())
        .merge(containers::router())
        .merge(users::router())
        .merge(roles::router())
        .merge(logs::router())
}
