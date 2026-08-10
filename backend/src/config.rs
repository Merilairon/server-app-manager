use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origin: String,
    pub tenant_id: String,
    pub static_dir: String,
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
                .unwrap_or_else(|_| "tenant_default".to_string()),
            static_dir: env::var("STATIC_DIR")
                .unwrap_or_else(|_| "static".to_string()),
        }
    }
}
