/// Utilities for ScfProject
use crate::ScfProject;

/// Convert a string to ScfProject compatible format
pub fn sanitize_scf_project_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' => c,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase()
}

/// Create a ScfProject from configuration
pub fn create_scf_project_from_config(name: &str, config: &[(&str, &str)]) -> ScfProject {
    let mut project = ScfProject::new(name);
    
    for (key, value) in config {
        project.set_config(key, value);
    }
    
    project
}

/// Validate a ScfProject name
pub fn is_valid_scf_project_name(name: &str) -> bool {
    !name.is_empty() 
        && name.len() <= 100 
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// Generate a default ScfProject configuration
pub fn default_scf_project_config() -> Vec<(&'static str, &'static str)> {
    vec![
        ("scf-project-type", "library"),
        ("scf-project-version", "0.1.0"),
        ("SCF_PROJECT_AUTHOR", "Unknown Author"),
        ("scf.project.license", "MIT"),
    ]
}

/// Format a ScfProject for display
pub fn format_scf_project_info(project: &ScfProject) -> String {
    format!(
        r#"
ScfProject Information:
- Name: {}
- Created: {:?}
- Configuration: {}
        "#,
        project.name(),
        project.created_at(),
        project.to_config_string()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_scf_project_name() {
        assert_eq!(sanitize_scf_project_name("My Project!"), "my-project");
        assert_eq!(sanitize_scf_project_name("hello_world"), "hello-world");
        assert_eq!(sanitize_scf_project_name("123-test"), "123-test");
    }

    #[test]
    fn test_is_valid_scf_project_name() {
        assert!(is_valid_scf_project_name("valid-name"));
        assert!(is_valid_scf_project_name("valid_name"));
        assert!(is_valid_scf_project_name("validname123"));
        assert!(!is_valid_scf_project_name(""));
        assert!(!is_valid_scf_project_name("-invalid"));
        assert!(!is_valid_scf_project_name("invalid-"));
        assert!(!is_valid_scf_project_name("invalid name"));
    }

    #[test]
    fn test_create_scf_project_from_config() {
        let config = vec![
            ("type", "library"),
            ("version", "1.0.0"),
        ];
        
        let project = create_scf_project_from_config("test-project", &config);
        assert_eq!(project.name(), "test-project");
        assert_eq!(project.get_config("type"), Some(&"library".to_string()));
        assert_eq!(project.get_config("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_default_scf_project_config() {
        let config = default_scf_project_config();
        assert!(!config.is_empty());
        assert!(config.iter().any(|(k, _)| k == &"scf-project-type"));
    }
} 