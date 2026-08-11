use std::collections::HashMap;

use serde_json::Value;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::catalog::{AppDefinition, Catalog};
use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, serde::Deserialize)]
pub struct InstallPayload {
    pub slug: String,
    #[serde(default)]
    pub values: HashMap<String, Value>,
}

pub async fn run(
    config: &Config,
    app: &AppDefinition,
    values: &HashMap<String, Value>,
) -> Result<Value, AppError> {
    let mut resolved: HashMap<String, String> = HashMap::new();
    for p in &app.placeholders {
        let v = match values.get(&p.name) {
            Some(Value::String(s)) => s.clone(),
            Some(other) => other.to_string(),
            None => match &p.default {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => {
                    if p.required {
                        return Err(AppError::bad_request(format!(
                            "missing placeholder {}",
                            p.name
                        )));
                    }
                    continue;
                }
            },
        };
        resolved.insert(p.name.clone(), v);
    }

    let base = Catalog::base_dir(config);
    let enabled_dir = base.join("apps/enabled");
    let enabled_file = enabled_dir.join(format!("{}.yaml", app.slug));
    if enabled_file.exists() {
        return Err(AppError::bad_request(format!(
            "app '{}' is already installed",
            app.slug
        )));
    }

    for dep in &app.depends_on {
        let dep_file = enabled_dir.join(format!("{}.yaml", dep));
        if !dep_file.exists() {
            return Err(AppError::bad_request(format!("missing dependency: {}", dep)));
        }
    }

    let mut installed = app.clone();
    for (k, v) in &resolved {
        installed.env.insert(k.clone(), v.clone());
    }

    std::fs::create_dir_all(&enabled_dir)
        .map_err(|e| AppError::internal(format!("create enabled dir: {e}")))?;
    let def_yaml = serde_yaml::to_string(&installed)
        .map_err(|e| AppError::internal(format!("serialize app: {e}")))?;
    std::fs::write(&enabled_file, def_yaml)
        .map_err(|e| AppError::internal(format!("write app yaml: {e}")))?;

    let project = format!("{}_{}", config.tenant_id, app.slug);
    let network_name = format!("{}_{}", config.tenant_id, app.slug);
    let backend_network = format!("{}_backend", config.tenant_id);
    let service_name = app.service.as_deref().unwrap_or(&app.slug);
    let container_name = format!("{}_{}", config.tenant_id, app.slug);

    let mut labels = serde_json::Map::new();
    labels.insert("traefik.enable".to_string(), Value::String("true".to_string()));
    labels.insert(
        format!("traefik.http.routers.{}.rule", app.slug),
        Value::String(format!("Host(`{}.{}`)", app.slug, config.base_domain)),
    );
    labels.insert(
        format!("traefik.http.routers.{}.entrypoints", app.slug),
        Value::String("websecure".to_string()),
    );
    labels.insert(
        format!("traefik.http.routers.{}.tls.certresolver", app.slug),
        Value::String("letsencrypt".to_string()),
    );
    labels.insert(
        format!("traefik.http.services.{}.loadbalancer.server.port", app.slug),
        app.ports
            .first()
            .map(|p| Value::Number(serde_json::Number::from(p.container as i64)))
            .unwrap_or_else(|| Value::Number(serde_json::Number::from(80i64))),
    );

    let mut service = serde_json::Map::new();
    service.insert("image".to_string(), Value::String(app.image.clone()));
    service.insert(
        "container_name".to_string(),
        Value::String(container_name.clone()),
    );
    service.insert("restart".to_string(), Value::String(app.restart.clone()));
    service.insert(
        "environment".to_string(),
        Value::Object(serde_json::Map::from_iter(
            installed
                .env
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone()))),
        )),
    );
    service.insert(
        "networks".to_string(),
        Value::Array(vec![
            Value::String("app_network".to_string()),
            Value::String("backend".to_string()),
        ]),
    );
    service.insert("labels".to_string(), Value::Object(labels));

    if !app.volumes.is_empty() {
        service.insert(
            "volumes".to_string(),
            Value::Array(
                app.volumes
                    .iter()
                    .map(|v| Value::String(v.clone()))
                    .collect(),
            ),
        );
    }

    if config.publish_app_ports {
        let host_ports: Vec<String> = app
            .ports
            .iter()
            .filter_map(|p| p.host.map(|h| format!("{}:{}", h, p.container)))
            .collect();
        if !host_ports.is_empty() {
            service.insert(
                "ports".to_string(),
                Value::Array(host_ports.iter().map(|p| Value::String(p.clone())).collect()),
            );
        }
    }

    if let Some(hc) = &app.healthcheck {
        let mut h = serde_json::Map::new();
        h.insert(
            "test".to_string(),
            Value::Array(vec![
                Value::String("CMD-SHELL".to_string()),
                Value::String(hc.test.clone()),
            ]),
        );
        h.insert("interval".to_string(), Value::String(hc.interval.clone()));
        h.insert("timeout".to_string(), Value::String(hc.timeout.clone()));
        h.insert(
            "retries".to_string(),
            Value::Number(serde_json::Number::from(hc.retries as i64)),
        );
        service.insert("healthcheck".to_string(), Value::Object(h));
    }

    if let Some(lim) = &app.resource_limits {
        let mut limits = serde_json::Map::new();
        if let Some(mem) = &lim.mem_limit {
            limits.insert("memory".to_string(), Value::String(mem.clone()));
        }
        if let Some(cpu) = &lim.cpu_quota {
            limits.insert("cpus".to_string(), Value::String(cpu.clone()));
        }
        if !limits.is_empty() {
            let mut resources = serde_json::Map::new();
            resources.insert("limits".to_string(), Value::Object(limits));
            let mut deploy = serde_json::Map::new();
            deploy.insert("resources".to_string(), Value::Object(resources));
            service.insert("deploy".to_string(), Value::Object(deploy));
        }
    }

    let mut services = serde_json::Map::new();
    services.insert(service_name.to_string(), Value::Object(service));

    let mut networks = serde_json::Map::new();
    let mut app_net = serde_json::Map::new();
    app_net.insert("driver".to_string(), Value::String("bridge".to_string()));
    app_net.insert("name".to_string(), Value::String(network_name));
    networks.insert("app_network".to_string(), Value::Object(app_net));

    let mut backend = serde_json::Map::new();
    backend.insert("external".to_string(), Value::Bool(true));
    backend.insert("name".to_string(), Value::String(backend_network));
    networks.insert("backend".to_string(), Value::Object(backend));

    let mut top_volumes = serde_json::Map::new();
    for v in &app.volumes {
        if let Some(idx) = v.find(':') {
            let name = &v[..idx];
            if !name.is_empty()
                && !name.contains('/')
                && !name.contains('\\')
                && !name.starts_with('.')
            {
                top_volumes.insert(name.to_string(), Value::Object(serde_json::Map::new()));
            }
        }
    }

    let mut compose = serde_json::Map::new();
    compose.insert("name".to_string(), Value::String(project.clone()));
    compose.insert("services".to_string(), Value::Object(services));
    compose.insert("networks".to_string(), Value::Object(networks));
    if !top_volumes.is_empty() {
        compose.insert("volumes".to_string(), Value::Object(top_volumes));
    }
    let compose = Value::Object(compose);

    let compose_dir = enabled_dir.join(&app.slug);
    std::fs::create_dir_all(&compose_dir)
        .map_err(|e| AppError::internal(format!("create compose dir: {e}")))?;
    let compose_path = compose_dir.join("docker-compose.yml");
    let compose_yaml = serde_yaml::to_string(&compose)
        .map_err(|e| AppError::internal(format!("compose yaml: {e}")))?;
    std::fs::write(&compose_path, compose_yaml)
        .map_err(|e| AppError::internal(format!("write compose: {e}")))?;

    let output = Command::new("docker-compose")
        .args([
            "-p",
            &project,
            "-f",
            compose_path.to_str().unwrap(),
            "up",
            "-d",
        ])
        .output()
        .await
        .map_err(|e| AppError::docker(format!("docker compose up: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::docker(format!("docker compose up failed: {stderr}")));
    }

    let healthy = wait_healthy(&container_name, app.healthcheck.is_some()).await?;
    let url = format!("https://{}.{}", app.slug, config.base_domain);
    Ok(serde_json::json!({
        "install_id": project,
        "slug": app.slug,
        "status": if healthy { "healthy" } else { "starting" },
        "url": url,
    }))
}

async fn wait_healthy(container_name: &str, has_healthcheck: bool) -> Result<bool, AppError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while tokio::time::Instant::now() < deadline {
        if has_healthcheck {
            let out = Command::new("docker")
                .args([
                    "inspect",
                    "--format",
                    "{{.State.Health.Status}}",
                    container_name,
                ])
                .output()
                .await
                .map_err(|e| AppError::docker(format!("docker inspect: {e}")))?;
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "healthy" {
                return Ok(true);
            }
        } else {
            let out = Command::new("docker")
                .args(["inspect", "--format", "{{.State.Status}}", container_name])
                .output()
                .await
                .map_err(|e| AppError::docker(format!("docker inspect: {e}")))?;
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "running" {
                return Ok(true);
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
    Ok(false)
}
