use serde_json::{json, Value};
use tokio::process::Command;

use crate::catalog::Catalog;
use crate::config::Config;
use crate::error::AppError;

pub async fn run(config: &Config, slug: &str) -> Result<Value, AppError> {
    let base = Catalog::base_dir(config);
    let enabled_dir = base.join("apps/enabled");
    let enabled_file = enabled_dir.join(format!("{}.yaml", slug));

    if !enabled_file.exists() {
        return Err(AppError::NotFound);
    }

    let project = format!("{}_{}", config.tenant_id, slug);
    let compose_dir = enabled_dir.join(slug);
    let compose_file = compose_dir.join("docker-compose.yml");

    if compose_file.exists() {
        let output = Command::new("docker")
            .args([
                "compose",
                "-p",
                &project,
                "-f",
                compose_file.to_str().unwrap(),
                "down",
                "--volumes",
                "--remove-orphans",
            ])
            .output()
            .await
            .map_err(|e| AppError::docker(format!("docker compose down: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::docker(format!("docker compose down failed: {stderr}")));
        }
    }

    let _ = std::fs::remove_dir_all(&compose_dir);
    std::fs::remove_file(&enabled_file)
        .map_err(|e| AppError::internal(format!("remove app yaml: {e}")))?;

    Ok(json!({
        "slug": slug,
        "status": "uninstalled",
    }))
}
