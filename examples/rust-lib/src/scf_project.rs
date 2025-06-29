use std::collections::HashMap;

#[cfg(feature = "serde")]
use crate::serde_support::{Deserialize, Serialize};

/// Main ScfProject structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScfProject {
    name: String,
    config: HashMap<String, String>,
    created_at: std::time::SystemTime,
}

impl ScfProject {
    /// Create a new ScfProject instance
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: HashMap::new(),
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Get the name of the ScfProject
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set a configuration value
    pub fn set_config(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Get the creation time
    pub fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }

    /// Initialize the ScfProject with default configuration
    pub fn init(&mut self) {
        self.set_config("scf-project-version", env!("CARGO_PKG_VERSION"));
        self.set_config("scf-project-type", "library");
        println!("Initialized ScfProject: {}", self.name);
    }

    /// Convert to a configuration string
    pub fn to_config_string(&self) -> String {
        format!(
            "scf-project: {}\nscf-project-config: {:?}",
            self.name, self.config
        )
    }
}

impl Default for ScfProject {
    fn default() -> Self {
        Self::new("default-scf-project")
    }
}

impl std::fmt::Display for ScfProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScfProject({})", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scf_project_new() {
        let project = ScfProject::new("test-scf-project");
        assert_eq!(project.name(), "test-scf-project");
    }

    #[test]
    fn test_scf_project_config() {
        let mut project = ScfProject::new("config-test");
        project.set_config("test-key", "test-value");
        assert_eq!(project.get_config("test-key"), Some(&"test-value".to_string()));
    }

    #[test]
    fn test_scf_project_init() {
        let mut project = ScfProject::new("init-test");
        project.init();
        assert!(project.get_config("scf-project-version").is_some());
        assert!(project.get_config("scf-project-type").is_some());
    }

    #[test]
    fn test_scf_project_display() {
        let project = ScfProject::new("display-test");
        assert_eq!(format!("{}", project), "ScfProject(display-test)");
    }
} 