use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScafferConfig {
    pub scaffer: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scaffer_template_urls: Option<HashMap<String, String>>,
}

impl ScafferConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_template_path(&mut self, path: String) {
        self.scaffer.push(path);
    }

    pub fn add_template_url(&mut self, name: String, url: String) {
        if self.scaffer_template_urls.is_none() {
            self.scaffer_template_urls = Some(HashMap::new());
        }
        self.scaffer_template_urls
            .as_mut()
            .unwrap()
            .insert(name, url);
    }

    /// Load scaffer configuration from current directory or parent directories
    pub fn load() -> Result<Self> {
        let mut current_dir = std::env::current_dir().context("Failed to get current directory")?;

        loop {
            // Try scaffer.json first
            let scaffer_json = current_dir.join("scaffer.json");
            if scaffer_json.exists() {
                let content = fs::read_to_string(&scaffer_json)
                    .with_context(|| format!("Failed to read {}", scaffer_json.display()))?;
                return serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse {}", scaffer_json.display()));
            }

            // Try package.json with scaffer key
            let package_json = current_dir.join("package.json");
            if package_json.exists() {
                let content = fs::read_to_string(&package_json)
                    .with_context(|| format!("Failed to read {}", package_json.display()))?;

                if let Ok(package_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(scaffer_config) = package_data.get("scaffer") {
                        if let Ok(config) = serde_json::from_value::<Self>(scaffer_config.clone()) {
                            return Ok(config);
                        }
                    }
                }
            }

            // Move to parent directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        // No configuration found, return default
        Ok(Self::default())
    }

    /// Load global scaffer configuration from user's home directory
    pub fn load_global() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let global_config_path = home_dir.join(".scaffer.json");

        if global_config_path.exists() {
            let content = fs::read_to_string(&global_config_path)
                .with_context(|| format!("Failed to read {}", global_config_path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", global_config_path.display()))
        } else {
            Ok(Self::default())
        }
    }

    /// Save global scaffer configuration to user's home directory
    pub fn save_global(&self) -> Result<()> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let global_config_path = home_dir.join(".scaffer.json");

        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize configuration")?;

        fs::write(&global_config_path, content)
            .with_context(|| format!("Failed to write {}", global_config_path.display()))?;

        Ok(())
    }

    /// Get all template directories, merging local and global configurations
    pub fn get_template_directories(&self) -> Result<Vec<PathBuf>> {
        let mut directories = Vec::new();

        // Add local template directories
        for path in &self.scaffer {
            directories.push(PathBuf::from(path));
        }

        // Add global template directories
        let global_config = Self::load_global()?;
        for path in &global_config.scaffer {
            directories.push(PathBuf::from(path));
        }

        Ok(directories)
    }

    /// Get all template URLs, merging local and global configurations
    pub fn get_template_urls(&self) -> Result<HashMap<String, String>> {
        let mut urls = HashMap::new();

        // Add global template URLs
        let global_config = Self::load_global()?;
        if let Some(global_urls) = &global_config.scaffer_template_urls {
            urls.extend(global_urls.clone());
        }

        // Add local template URLs (these override global ones with same name)
        if let Some(local_urls) = &self.scaffer_template_urls {
            urls.extend(local_urls.clone());
        }

        Ok(urls)
    }

    /// Find all available templates
    pub fn find_templates(&self) -> Result<Vec<String>> {
        let mut templates = Vec::new();

        // Find directory-based templates
        for dir in self.get_template_directories()? {
            if dir.exists() && dir.is_dir() {
                for entry in fs::read_dir(&dir)
                    .with_context(|| format!("Failed to read directory {}", dir.display()))?
                {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            templates.push(name.to_string());
                        }
                    }
                }
            }
        }

        // Add URL-based templates
        for name in self.get_template_urls()?.keys() {
            templates.push(name.clone());
        }

        templates.sort();
        templates.dedup();
        Ok(templates)
    }
}
