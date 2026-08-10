use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Role {
    pub permissions: Option<Vec<String>>,
    pub inherits: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RolesFile {
    roles: HashMap<String, Role>,
}

#[derive(Debug, Clone)]
pub struct Roles {
    inner: HashMap<String, Role>,
}

impl Roles {
    pub fn from_yaml(path: &str) -> Result<Self, AppError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| AppError::internal(format!("roles file: {e}")))?;
        let file: RolesFile =
            serde_yaml::from_str(&content).map_err(|e| AppError::internal(format!("roles yaml: {e}")))?;
        Ok(Self { inner: file.roles })
    }

    pub fn permissions(&self, role: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut perms = HashSet::new();
        let mut stack = vec![role.to_string()];

        while let Some(name) = stack.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            if let Some(def) = self.inner.get(&name) {
                if let Some(list) = &def.permissions {
                    for p in list {
                        perms.insert(p.clone());
                    }
                }
                if let Some(parents) = &def.inherits {
                    for p in parents {
                        stack.push(p.clone());
                    }
                }
            }
        }

        perms.into_iter().collect()
    }

    pub fn has_permission(&self, role: &str, permission: &str) -> bool {
        let perms = self.permissions(role);
        perms.iter().any(|p| p == permission) || perms.iter().any(|p| p == "admin:all")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_roles() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("roles-{}.yaml", uuid::Uuid::new_v4()));
        std::fs::write(
            &dir,
            r#"
roles:
  user:
    permissions:
      - read:apps
      - install:apps
    inherits:
      - base
  base:
    permissions:
      - read:own_profile
  admin:
    permissions:
      - admin:all
    inherits:
      - user
"#,
        )
        .unwrap();
        dir
    }

    #[test]
    fn loads_roles_and_expands_permissions() {
        let path = write_temp_roles();
        let roles = Roles::from_yaml(path.to_str().unwrap()).unwrap();

        let user_perms = roles.permissions("user");
        assert!(user_perms.contains(&"read:apps".to_string()));
        assert!(user_perms.contains(&"install:apps".to_string()));
        assert!(user_perms.contains(&"read:own_profile".to_string()));
    }

    #[test]
    fn has_permission_checks_inheritance() {
        let path = write_temp_roles();
        let roles = Roles::from_yaml(path.to_str().unwrap()).unwrap();

        assert!(roles.has_permission("user", "read:apps"));
        assert!(roles.has_permission("user", "read:own_profile"));
        assert!(!roles.has_permission("user", "admin:all"));
        assert!(roles.has_permission("admin", "read:apps"));
        assert!(roles.has_permission("admin", "anything:goes"));
    }
}
