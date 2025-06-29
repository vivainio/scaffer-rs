use clap::{Parser, Subcommand};
use anyhow::{Result, Context};

mod config;
mod template;
mod generator;
mod utils;

use config::ScafferConfig;
use generator::TemplateGenerator;

#[derive(Parser)]
#[command(name = "scaffer")]
#[command(about = "A scaffolding tool for generating code from templates")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate code from named or downloaded template
    #[command(name = "g")]
    Generate {
        /// Template to generate or a URL to template zip package
        template: Option<String>,
        /// Give value to variable
        #[arg(short = 'v', long = "var", value_name = "variable=value")]
        variables: Vec<String>,
        /// Overwrite files if needed
        #[arg(short, long)]
        force: bool,
        /// Dry run, do not create files
        #[arg(long)]
        dry: bool,
    },
    /// Add current directory as template root in user global scaffer.json
    Add,
    /// Create index.ts for current directory
    Barrel,
    /// Create .gitignore file
    Gitignore,
    /// Setup scaffer configuration
    Setup,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { template, variables, force, dry } => {
            let generator = TemplateGenerator::new();
            generator.generate(template, variables, force, dry)?;
        }
        Commands::Add => {
            add_current_directory_as_template()?;
        }
        Commands::Barrel => {
            create_barrel_file()?;
        }
        Commands::Gitignore => {
            create_gitignore_file()?;
        }
        Commands::Setup => {
            setup_scaffer_config()?;
        }
    }

    Ok(())
}

fn add_current_directory_as_template() -> Result<()> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    let mut global_config = ScafferConfig::load_global()?;
    global_config.add_template_path(current_dir.to_string_lossy().to_string());
    global_config.save_global()?;
    
    println!("Added current directory as template root");
    Ok(())
}

fn create_barrel_file() -> Result<()> {
    use std::fs;
    use walkdir::WalkDir;
    
    let mut exports = Vec::new();
    
    for entry in WalkDir::new(".")
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".ts") && file_name != "index.ts" {
                    let module_name = file_name.trim_end_matches(".ts");
                    exports.push(format!("export * from './{}';\n", module_name));
                }
            }
        } else if entry.file_type().is_dir() {
            if let Some(dir_name) = entry.file_name().to_str() {
                exports.push(format!("export * from './{}';\n", dir_name));
            }
        }
    }
    
    fs::write("index.ts", exports.join(""))?;
    println!("Created index.ts barrel file");
    Ok(())
}

fn create_gitignore_file() -> Result<()> {
    use std::fs;
    
    let gitignore_content = r#"# Dependencies
node_modules/
target/
dist/
build/

# Environment variables
.env
.env.local
.env.*.local

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Cache
.cache/
*.tmp
*.temp
"#;
    
    fs::write(".gitignore", gitignore_content)?;
    println!("Created .gitignore file");
    Ok(())
}

fn setup_scaffer_config() -> Result<()> {
    use dialoguer::{Input, Confirm};
    use std::fs;
    
    println!("Setting up scaffer configuration...");
    
    let template_dirs: String = Input::new()
        .with_prompt("Enter template directories (comma-separated)")
        .default("templates".to_string())
        .interact_text()?;
    
    let use_urls = Confirm::new()
        .with_prompt("Do you want to configure template URLs?")
        .default(false)
        .interact()?;
    
    let mut config = ScafferConfig::new();
    
    for dir in template_dirs.split(',') {
        config.add_template_path(dir.trim().to_string());
    }
    
    if use_urls {
        loop {
            let template_name: String = Input::new()
                .with_prompt("Template name (empty to finish)")
                .allow_empty(true)
                .interact_text()?;
            
            if template_name.is_empty() {
                break;
            }
            
            let template_url: String = Input::new()
                .with_prompt("Template URL")
                .interact_text()?;
            
            config.add_template_url(template_name, template_url);
        }
    }
    
    let config_content = serde_json::to_string_pretty(&config)?;
    fs::write("scaffer.json", config_content)?;
    
    println!("Created scaffer.json configuration file");
    Ok(())
} 