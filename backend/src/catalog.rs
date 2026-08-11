use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Port {
    pub host: Option<i32>,
    pub container: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Placeholder {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub label: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub regex: Option<String>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Healthcheck {
    pub test: String,
    #[serde(default = "default_interval")]
    pub interval: String,
    #[serde(default = "default_timeout")]
    pub timeout: String,
    #[serde(default = "default_retries")]
    pub retries: i32,
}

fn default_interval() -> String {
    "30s".to_string()
}

fn default_timeout() -> String {
    "10s".to_string()
}

fn default_retries() -> i32 {
    3
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub mem_limit: Option<String>,
    pub cpu_quota: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppDefinition {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub image: String,
    pub service: Option<String>,
    #[serde(default)]
    pub ports: Vec<Port>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub healthcheck: Option<Healthcheck>,
    #[serde(default = "default_restart")]
    pub restart: String,
    #[serde(default)]
    pub placeholders: Vec<Placeholder>,
    pub resource_limits: Option<ResourceLimits>,
    #[serde(default)]
    pub status: String,
}

fn default_restart() -> String {
    "unless-stopped".to_string()
}

#[derive(Debug, Default)]
pub struct Catalog {
    pub apps: Vec<AppDefinition>,
    pub by_slug: HashMap<String, AppDefinition>,
}

impl Catalog {
    pub fn load(config: &Config) -> Result<Self, AppError> {
        let base = Self::base_dir(config);
        let mut catalog = Catalog::default();
        let dirs = vec![
            ("available", Path::new("apps/store")),
            ("installed", Path::new("apps/enabled")),
            ("disabled", Path::new("apps/disabled")),
        ];

        for (status, dir) in dirs {
            let full = base.join(dir);
            if !full.exists() {
                continue;
            }

            for entry in std::fs::read_dir(&full)
                .map_err(|e| AppError::internal(format!("read {dir:?}: {e}")))?
                .flatten()
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                    continue;
                }
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| AppError::internal(format!("read {}: {e}", path.display())))?;
                let mut app: AppDefinition = serde_yaml::from_str(&text).map_err(|e| {
                    AppError::internal(format!("parse {}: {e}", path.display()))
                })?;

                if app.slug.is_empty() || app.name.is_empty() || app.image.is_empty() {
                    continue;
                }

                if status == "installed" || status == "disabled" {
                    app.status = status.to_string();
                } else {
                    app.status = "available".to_string();
                }

                catalog.by_slug.insert(app.slug.clone(), app.clone());
                catalog.apps.push(app);
            }
        }

        catalog.apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(catalog)
    }

    pub(crate) fn base_dir(config: &Config) -> std::path::PathBuf {
        let configured = Path::new(&config.compose_apps_dir).to_path_buf();
        if configured.join("apps/store").exists() {
            return configured;
        }
        if Path::new("./apps/store").exists() {
            return Path::new(".").to_path_buf();
        }
        if Path::new("../apps/store").exists() {
            return Path::new("..").to_path_buf();
        }
        configured
    }

    pub fn get(&self, slug: &str) -> Option<&AppDefinition> {
        self.by_slug.get(slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config() -> (Config, std::path::PathBuf) {
        let base = std::env::temp_dir().join(format!("catalog-{}-base", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(base.join("apps/store")).unwrap();
        std::fs::write(
            base.join("apps/store/testapp.yaml"),
            r#"name: Test App
slug: testapp
description: A test app
category: Test
image: nginx:alpine
service: web
ports:
  - container: 80
placeholders:
  - name: DOMAIN
    type: string
    label: Domain
    required: true
    default: "example.com"
"#,
        )
        .unwrap();
        (
            Config {
                database_url: "postgres://test".to_string(),
                jwt_secret: "test".to_string(),
                cors_origin: "*".to_string(),
                tenant_id: "test".to_string(),
                static_dir: "static".to_string(),
                admin_username: "admin".to_string(),
                admin_password: None,
                compose_apps_dir: base.to_str().unwrap().to_string(),
                cookie_secure: false,
                base_domain: "app.local".to_string(),
                publish_app_ports: false,
                docker_socket: None,
            },
            base,
        )
    }

    #[test]
    fn loads_store_app() {
        let (config, _base) = temp_config();
        let catalog = Catalog::load(&config).unwrap();
        assert_eq!(catalog.apps.len(), 1);
        let app = catalog.get("testapp").unwrap();
        assert_eq!(app.name, "Test App");
        assert_eq!(app.status, "available");
        assert_eq!(app.placeholders.len(), 1);
    }

    #[test]
    fn base_dir_uses_configured_path() {
        let (config, _base) = temp_config();
        let base = Catalog::base_dir(&config);
        assert!(base.join("apps/store").exists());
    }
}
