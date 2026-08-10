use axum::http::{header, HeaderMap, HeaderValue};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub tenant_id: String,
    pub exp: usize,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub status: String,
}

pub fn issue_token(config: &Config, user: &User, tenant_id: &str) -> Result<String, AppError> {
    let exp = (Utc::now().timestamp() as usize) + 24 * 3600;
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        tenant_id: tenant_id.to_string(),
        exp,
    };
    let key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
    encode(&Header::new(Algorithm::HS256), &claims, &key).map_err(AppError::Jwt)
}

pub fn verify_token(config: &Config, token: &str) -> Result<Claims, AppError> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(AppError::Jwt)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::Bcrypt)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash).map_err(AppError::Bcrypt)
}

pub async fn login_user(db: &PgPool, username: &str, password: &str) -> Result<User, AppError> {
    let user: Option<User> = sqlx::query_as::<sqlx::Postgres, User>(
        "SELECT id, username, email, password_hash, role, status FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(db)
    .await
    .map_err(AppError::Sqlx)?;

    let user = user.ok_or(AppError::Unauthorized)?;
    if user.status != "active" {
        return Err(AppError::Unauthorized);
    }
    eprintln!("login attempt password_len={} hash_len={}", password.len(), user.password_hash.len());
    let ok = verify_password(password, &user.password_hash)?;
    eprintln!("verify result ok={} user={}", ok, user.username);
    if !ok {
        return Err(AppError::Unauthorized);
    }
    Ok(user)
}

pub fn new_csrf_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn set_auth_cookies(config: &Config, token: &str, csrf: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let secure = if config.cookie_secure { "; Secure" } else { "" };
    let token_cookie = format!(
        "token={token}; HttpOnly; Path=/; SameSite=Strict{secure}; Max-Age=86400"
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&token_cookie).unwrap(),
    );
    let csrf_cookie = format!("csrf={csrf}; Path=/; SameSite=Strict{secure}; Max-Age=86400");
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&csrf_cookie).unwrap(),
    );
    headers
}

pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                let key = parts.next()?.trim();
                let val = parts.next()?.trim();
                if key == name {
                    Some(val.to_string())
                } else {
                    None
                }
            })
        })
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub fn login_response(
    config: &Config,
    user: &User,
    tenant_id: &str,
) -> Result<(HeaderMap, Value), AppError> {
    let token = issue_token(config, user, tenant_id)?;
    let csrf = new_csrf_token();
    let headers = set_auth_cookies(config, &token, &csrf);
    let body = json!({
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "role": user.role,
        },
        "csrf": csrf,
    });
    Ok((headers, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderMap, HeaderValue};

    fn test_config() -> Config {
        Config {
            database_url: "postgres://test".to_string(),
            jwt_secret: "test-secret".to_string(),
            cors_origin: "*".to_string(),
            tenant_id: "test".to_string(),
            static_dir: "static".to_string(),
            admin_username: "admin".to_string(),
            admin_password: Some("pass".to_string()),
            compose_apps_dir: "/tmp".to_string(),
            cookie_secure: false,
            base_domain: "app.example.com".to_string(),
            docker_socket: None,
        }
    }

    fn other_config() -> Config {
        Config {
            jwt_secret: "other-secret".to_string(),
            ..test_config()
        }
    }

    fn test_user() -> User {
        User {
            id: uuid::Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            password_hash: "".to_string(),
            role: "admin".to_string(),
            status: "active".to_string(),
        }
    }

    #[test]
    fn issue_and_verify_token_roundtrip() {
        let token = issue_token(&test_config(), &test_user(), "tenant_test").unwrap();
        let claims = verify_token(&test_config(), &token).unwrap();
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.tenant_id, "tenant_test");
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let token = issue_token(&test_config(), &test_user(), "tenant_test").unwrap();
        assert!(verify_token(&other_config(), &token).is_err());
    }

    #[test]
    fn verify_rejects_malformed_token() {
        assert!(verify_token(&test_config(), "not-a-jwt").is_err());
    }

    #[test]
    fn hash_and_verify_password() {
        let hash = hash_password("password123").unwrap();
        assert!(verify_password("password123", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn hash_and_verify_your_password() {
        let hash = "$2b$12$7YIUiEWF4gLkvH7c1USVIOVLoGRX0EAB8uuamvZXTD2gpbwaybGYS";
        assert!(verify_password("your-password", hash).unwrap());
    }

    #[test]
    fn cookie_value_parses_names() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("token=abc; csrf=xyz; other=123"),
        );
        assert_eq!(cookie_value(&headers, "token"), Some("abc".to_string()));
        assert_eq!(cookie_value(&headers, "csrf"), Some("xyz".to_string()));
        assert_eq!(cookie_value(&headers, "missing"), None);
    }
}
