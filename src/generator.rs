use crate::config::ScafferConfig;
use crate::template::TemplateProcessor;
use crate::utils;

use anyhow::{Context, Result, bail};
use dialoguer::{Confirm, Input, Select};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;

pub struct TemplateGenerator {
    config: ScafferConfig,
}

impl TemplateGenerator {
    pub fn new() -> Self {
        let config = ScafferConfig::load().unwrap_or_default();

        Self { config }
    }

    pub fn generate(
        &self,
        template: Option<String>,
        variables: Vec<String>,
        force: bool,
        dry_run: bool,
    ) -> Result<()> {
        let template_name = match template {
            Some(name) => name,
            None => self.prompt_for_template()?,
        };

        // Check if it's a URL
        let template_path =
            if template_name.starts_with("http://") || template_name.starts_with("https://") {
                self.download_template(&template_name)?
            } else {
                self.find_template(&template_name)?
            };

        // Parse command-line variables
        let mut var_map = HashMap::new();
        for var_str in variables {
            if let Some((key, value)) = var_str.split_once('=') {
                var_map.insert(key.to_string(), value.to_string());
            }
        }

        // Scan template for variables
        let required_vars = self.scan_template_variables(&template_path)?;

        // Prompt for missing variables
        for var_name in &required_vars {
            if !var_map.contains_key(var_name) {
                let value: String = Input::new()
                    .with_prompt(format!("Enter value for '{var_name}'"))
                    .interact_text()?;
                var_map.insert(var_name.clone(), value);
            }
        }

        // Process the template
        self.process_template(&template_path, var_map, force, dry_run)?;

        Ok(())
    }

    fn prompt_for_template(&self) -> Result<String> {
        let templates = self.config.find_templates()?;

        if templates.is_empty() {
            bail!("No templates found. Run 'scaffer setup' to configure template directories.");
        }

        let selection = Select::new()
            .with_prompt("Select a template")
            .items(&templates)
            .interact()?;

        Ok(templates[selection].clone())
    }

    fn download_template(&self, url: &str) -> Result<PathBuf> {
        println!("Downloading template from {url}...");

        let response = reqwest::blocking::get(url)
            .with_context(|| format!("Failed to download template from {url}"))?;

        if !response.status().is_success() {
            bail!("Failed to download template: HTTP {}", response.status());
        }

        let bytes = response.bytes().context("Failed to read template data")?;

        // Create temporary directory
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

        let zip_path = temp_dir.path().join("template.zip");
        fs::write(&zip_path, bytes).context("Failed to write template zip file")?;

        // Extract zip file
        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir_all(&extract_dir).context("Failed to create extraction directory")?;

        utils::extract_zip(&zip_path, &extract_dir).context("Failed to extract template zip")?;

        // Find the actual template directory (might be nested)
        let template_dir = utils::find_template_root(&extract_dir)?;

        Ok(template_dir)
    }

    fn find_template(&self, template_name: &str) -> Result<PathBuf> {
        // First check if it's a direct path
        let direct_path = PathBuf::from(template_name);
        if direct_path.exists() {
            return Ok(direct_path);
        }

        // Check template URLs
        let template_urls = self.config.get_template_urls()?;
        if let Some(url) = template_urls.get(template_name) {
            return Ok(PathBuf::from(url));
        }

        // Search in template directories
        for template_dir in self.config.get_template_directories()? {
            let template_path = template_dir.join(template_name);
            if template_path.exists() {
                return Ok(template_path);
            }
        }

        bail!("Template '{}' not found", template_name);
    }

    fn scan_template_variables(&self, template_path: &Path) -> Result<HashSet<String>> {
        let mut variables = HashSet::new();
        let processor = TemplateProcessor::new();

        // Check if there's a scaffer_init.py file for custom logic
        let init_file = template_path.join("scaffer_init.py");
        if init_file.exists() {
            println!("Found scaffer_init.py - custom template initialization");
            // TODO: Implement Python script execution for advanced templates
        }

        // Scan all files in the template
        for entry in WalkDir::new(template_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Extract variables from file path
            if let Some(path_str) = path.to_str() {
                let path_vars = processor.extract_variables(path_str);
                variables.extend(path_vars);
            }

            // Extract variables from file contents
            if entry.file_type().is_file() {
                if let Ok(content) = fs::read_to_string(path) {
                    let content_vars = processor.extract_variables(&content);
                    variables.extend(content_vars);
                }
            }
        }

        Ok(variables)
    }

    fn process_template(
        &self,
        template_path: &Path,
        variables: HashMap<String, String>,
        force: bool,
        dry_run: bool,
    ) -> Result<()> {
        let mut processor = TemplateProcessor::new();
        processor.set_variables(variables);

        let current_dir = std::env::current_dir().context("Failed to get current directory")?;

        println!("Processing template from: {}", template_path.display());

        if dry_run {
            println!("DRY RUN - No files will be created");
        }

        let mut files_created = 0;
        let mut files_skipped = 0;

        for entry in WalkDir::new(template_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();

            // Skip the template root directory itself
            if src_path == template_path {
                continue;
            }

            // Calculate relative path from template root
            let rel_path = src_path
                .strip_prefix(template_path)
                .context("Failed to calculate relative path")?;

            // Process the path with variable substitution
            let processed_rel_path = processor.process_path(&rel_path.to_string_lossy());
            let dest_path = current_dir.join(&processed_rel_path);

            if entry.file_type().is_dir() {
                // Create directory
                if !dry_run {
                    fs::create_dir_all(&dest_path).with_context(|| {
                        format!("Failed to create directory: {}", dest_path.display())
                    })?;
                }
                println!("Created directory: {processed_rel_path}");
            } else if entry.file_type().is_file() {
                // Skip scaffer_init.py
                if src_path.file_name() == Some(std::ffi::OsStr::new("scaffer_init.py")) {
                    continue;
                }

                // Check if file already exists
                if dest_path.exists() && !force {
                    if dry_run {
                        println!("Would skip existing file: {processed_rel_path}");
                        files_skipped += 1;
                        continue;
                    }

                    let overwrite = Confirm::new()
                        .with_prompt(format!(
                            "File '{processed_rel_path}' already exists. Overwrite?"
                        ))
                        .default(false)
                        .interact()?;

                    if !overwrite {
                        println!("Skipped: {processed_rel_path}");
                        files_skipped += 1;
                        continue;
                    }
                }

                // Read and process file content
                let content = fs::read_to_string(src_path).with_context(|| {
                    format!("Failed to read template file: {}", src_path.display())
                })?;

                let processed_content = processor.process_text(&content);

                if !dry_run {
                    // Ensure parent directory exists
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("Failed to create parent directory: {}", parent.display())
                        })?;
                    }

                    // Write processed file
                    fs::write(&dest_path, processed_content).with_context(|| {
                        format!("Failed to write file: {}", dest_path.display())
                    })?;
                }

                println!("Created file: {processed_rel_path}");
                files_created += 1;
            }
        }

        println!("\nTemplate processing complete!");
        println!("Files created: {files_created}");

        if files_skipped > 0 {
            println!("Files skipped: {files_skipped}");
        }

        if dry_run {
            println!("This was a dry run - no files were actually created.");
        }

        Ok(())
    }
}
