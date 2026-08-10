use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origin: String,
    pub tenant_id: String,
    pub static_dir: String,
    pub admin_username: String,
    pub admin_password: Option<String>,
    pub compose_apps_dir: String,
    pub cookie_secure: bool,
    pub base_domain: String,
    pub docker_socket: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://sam:sam@db:5432/sam".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "changeme-change-this-in-production".to_string()),
            cors_origin: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "https://app.example.com".to_string()),
            tenant_id: env::var("TENANT_ID")
                .unwrap_or_else(|_| "appforge".to_string()),
            static_dir: env::var("STATIC_DIR")
                .unwrap_or_else(|_| "static".to_string()),
            admin_username: env::var("ADMIN_USERNAME")
                .unwrap_or_else(|_| "admin".to_string()),
            admin_password: env::var("ADMIN_PASSWORD").ok(),
            compose_apps_dir: env::var("COMPOSE_APPS_DIR")
                .unwrap_or_else(|_| "/data/apps".to_string()),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true")
                .unwrap_or(false),
            base_domain: env::var("BASE_DOMAIN")
                .unwrap_or_else(|_| "app.example.com".to_string()),
            docker_socket: env::var("DOCKER_SOCKET").ok(),
        }
    }
}
