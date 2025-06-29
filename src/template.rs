use convert_case::{Case, Casing};
use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TemplateVariable {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateProcessor {
    variables: HashMap<String, String>,
    variable_patterns: Vec<Regex>,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        let variable_patterns = vec![
            // ScfMyvar - PascalCase with Scf prefix
            Regex::new(r"\bScf([A-Z][a-zA-Z0-9]*)\b").unwrap(),
            // SCF_MYVAR - UPPER_SNAKE_CASE with SCF prefix
            Regex::new(r"\bSCF_([A-Z][A-Z0-9_]*)\b").unwrap(),
            // SCF-MYVAR - UPPER-KEBAB-CASE with SCF prefix
            Regex::new(r"\bSCF-([A-Z][A-Z0-9-]*)\b").unwrap(),
            // SCF.MYVAR - UPPER.DOT.CASE with SCF prefix
            Regex::new(r"\bSCF\.([A-Z][A-Z0-9\.]*)\b").unwrap(),
            // scf_myvar - snake_case with scf prefix
            Regex::new(r"\bscf_([a-z][a-z0-9_]*)\b").unwrap(),
            // scf-myvar - kebab-case with scf prefix
            Regex::new(r"\bscf-([a-z][a-z0-9-]*)\b").unwrap(),
            // scf.myvar - dot.case with scf prefix
            Regex::new(r"\bscf\.([a-z][a-z0-9\.]*)\b").unwrap(),
            // scfmyvar - lowercase flat with scf prefix
            Regex::new(r"\bscf([a-z][a-z0-9]*)\b").unwrap(),
            // SCFMYVAR - uppercase flat with SCF prefix
            Regex::new(r"\bSCF([A-Z][A-Z0-9]*)\b").unwrap(),
        ];

        Self {
            variables: HashMap::new(),
            variable_patterns,
        }
    }

    pub fn set_variable(&mut self, name: String, value: String) {
        // Normalize the variable name to kebab-case
        let normalized_name = name.to_case(Case::Kebab);
        self.variables.insert(normalized_name, value);
    }

    pub fn set_variables(&mut self, variables: HashMap<String, String>) {
        for (name, value) in variables {
            self.set_variable(name, value);
        }
    }

    /// Extract all template variables from the given text
    pub fn extract_variables(&self, text: &str) -> HashSet<String> {
        let mut variables = HashSet::new();

        for pattern in &self.variable_patterns {
            for caps in pattern.captures_iter(text) {
                if let Some(var_match) = caps.get(1) {
                    let var_name = var_match.as_str();
                    // Convert to kebab-case for consistency
                    let normalized = var_name.to_case(Case::Kebab);
                    variables.insert(normalized);
                }
            }
        }

        variables
    }

    /// Process template text by replacing all variable placeholders
    pub fn process_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        for (var_name, var_value) in &self.variables {
            result = self.replace_variable_in_text(&result, var_name, var_value);
        }

        result
    }

    /// Replace all occurrences of a variable in different case formats
    fn replace_variable_in_text(&self, text: &str, var_name: &str, var_value: &str) -> String {
        let mut result = text.to_string();

        // Convert variable name and value to different cases
        let pascal_var = var_name.to_case(Case::Pascal);
        let upper_snake_var = var_name.to_case(Case::UpperSnake);
        let upper_kebab_var = var_name.to_case(Case::UpperKebab);
        let upper_flat_var = var_name.to_case(Case::UpperFlat);
        let snake_var = var_name.to_case(Case::Snake);
        let kebab_var = var_name.to_case(Case::Kebab);
        let flat_var = var_name.to_case(Case::Flat);

        let pascal_val = var_value.to_case(Case::Pascal);
        let upper_snake_val = var_value.to_case(Case::UpperSnake);
        let upper_kebab_val = var_value.to_case(Case::UpperKebab);
        let upper_flat_val = var_value.to_case(Case::UpperFlat);
        let snake_val = var_value.to_case(Case::Snake);
        let kebab_val = var_value.to_case(Case::Kebab);
        let flat_val = var_value.to_case(Case::Flat);

        // Create dot-separated versions
        let upper_dot_var = upper_kebab_var.replace('-', ".");
        let lower_dot_var = kebab_var.replace('-', ".");
        let upper_dot_val = upper_kebab_val.replace('-', ".");
        let lower_dot_val = kebab_val.replace('-', ".");

        // Replace patterns (order matters - more specific patterns first)
        let replacements = vec![
            // PascalCase with Scf prefix
            (
                format!(r"\bScf{}\b", pascal_var),
                format!("Scf{}", pascal_val),
            ),
            // UPPER_SNAKE_CASE with SCF prefix
            (
                format!(r"\bSCF_{}\b", upper_snake_var),
                format!("SCF_{}", upper_snake_val),
            ),
            // UPPER-KEBAB-CASE with SCF prefix
            (
                format!(r"\bSCF-{}\b", upper_kebab_var),
                format!("SCF-{}", upper_kebab_val),
            ),
            // UPPER.DOT.CASE with SCF prefix
            (
                format!(r"\bSCF\.{}\b", upper_dot_var),
                format!("SCF.{}", upper_dot_val),
            ),
            // snake_case with scf prefix
            (
                format!(r"\bscf_{}\b", snake_var),
                format!("scf_{}", snake_val),
            ),
            // kebab-case with scf prefix
            (
                format!(r"\bscf-{}\b", kebab_var),
                format!("scf-{}", kebab_val),
            ),
            // dot.case with scf prefix
            (
                format!(r"\bscf\.{}\b", lower_dot_var),
                format!("scf.{}", lower_dot_val),
            ),
            // lowercase flat with scf prefix
            (format!(r"\bscf{}\b", flat_var), format!("scf{}", flat_val)),
            // uppercase flat with SCF prefix
            (
                format!(r"\bSCF{}\b", upper_flat_var),
                format!("SCF{}", upper_flat_val),
            ),
        ];

        for (pattern, replacement) in replacements {
            if let Ok(re) = Regex::new(&pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }

        result
    }

    /// Process a file path by replacing variables in the path components
    pub fn process_path(&self, path: &str) -> String {
        let processed = self.process_text(path);

        // Clean up any invalid path characters that might result from replacement
        processed
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
                _ => c,
            })
            .collect()
    }

    /// Get list of variables that need values
    pub fn get_missing_variables(&self, required_vars: &HashSet<String>) -> Vec<String> {
        required_vars
            .iter()
            .filter(|var| !self.variables.contains_key(*var))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_extraction() {
        let processor = TemplateProcessor::new();
        let text = r#"
            ScfMyProject vs ScfOtherVar in pascalcase.
            Lower kebab scf-my-project scf-other-var
            Lower snake scf_my_project scf_other_var
            Lower dotted scf.my.project scf.other.var
            Lower flat scfmyproject scfothervar
            Upper dotted SCF.MY.PROJECT SCF.OTHER.VAR
            Upper kebab SCF-MY-PROJECT SCF-OTHER-VAR
            Upper snake SCF_MY_PROJECT SCF_OTHER_VAR
            Upper flat SCFMYPROJECT SCFOTHERVAR
        "#;

        let variables = processor.extract_variables(text);
        let mut vars: Vec<_> = variables.into_iter().collect();
        vars.sort();

        // The processor extracts all variations but normalizes them to kebab-case
        // We should see the normalized versions of the different variable patterns
        let expected = vec![
            "my-project",
            "my.project",
            "myproject",
            "other-var",
            "other.var",
            "othervar",
        ];
        assert_eq!(vars, expected);
    }

    #[test]
    fn test_variable_replacement() {
        let mut processor = TemplateProcessor::new();
        processor.set_variable("my-project".to_string(), "hello-world".to_string());

        let text = "ScfMyProject and scf-my-project and SCF_MY_PROJECT";
        let result = processor.process_text(text);

        assert!(result.contains("ScfHelloWorld"));
        assert!(result.contains("scf-hello-world"));
        assert!(result.contains("SCF_HELLO_WORLD"));
    }

    #[test]
    fn test_path_processing() {
        let mut processor = TemplateProcessor::new();
        processor.set_variable("project".to_string(), "my-app".to_string());

        let path = "src/ScfProject/scf-project.rs";
        let result = processor.process_path(path);

        assert_eq!(result, "src/ScfMyApp/scf-my-app.rs");
    }
}
