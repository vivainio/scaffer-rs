use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

/// Extract a ZIP file to a destination directory
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)
        .with_context(|| format!("Failed to open zip file: {}", zip_path.display()))?;

    let mut archive = ZipArchive::new(file).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .with_context(|| format!("Failed to read file at index {}", i))?;

        let outpath = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&outpath)
                .with_context(|| format!("Failed to create directory: {}", outpath.display()))?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create parent directory: {}", parent.display())
                })?;
            }

            let mut outfile = fs::File::create(&outpath)
                .with_context(|| format!("Failed to create file: {}", outpath.display()))?;

            std::io::copy(&mut file, &mut outfile)
                .with_context(|| format!("Failed to extract file: {}", outpath.display()))?;
        }
    }

    Ok(())
}

/// Find the root directory of a template within an extracted archive
/// This handles cases where the template might be nested within subdirectories
pub fn find_template_root(extract_dir: &Path) -> Result<PathBuf> {
    // First check if the extract directory itself looks like a template
    if is_template_directory(extract_dir)? {
        return Ok(extract_dir.to_path_buf());
    }

    // Look for template directories in subdirectories
    for entry in fs::read_dir(extract_dir)
        .with_context(|| format!("Failed to read directory: {}", extract_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && is_template_directory(&path)? {
            return Ok(path);
        }
    }

    // If no obvious template directory found, use the first subdirectory
    // or the extract directory itself
    for entry in fs::read_dir(extract_dir)
        .with_context(|| format!("Failed to read directory: {}", extract_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            return Ok(path);
        }
    }

    Ok(extract_dir.to_path_buf())
}

/// Check if a directory looks like a template directory
/// A template directory should contain files or have template variables in names
fn is_template_directory(dir: &Path) -> Result<bool> {
    let entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()?;

    // Empty directory is not a template
    if entries.is_empty() {
        return Ok(false);
    }

    // Check for scaffer_init.py which indicates a template
    for entry in &entries {
        if entry.file_name() == "scaffer_init.py" {
            return Ok(true);
        }
    }

    // Check if any file or directory names contain template variables
    for entry in &entries {
        if let Some(name) = entry.file_name().to_str() {
            if contains_template_variables(name) {
                return Ok(true);
            }
        }
    }

    // Check file contents for template variables
    for entry in &entries {
        let path = entry.path();
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                if contains_template_variables(&content) {
                    return Ok(true);
                }
            }
        }
    }

    // If it contains multiple files/directories, assume it's a template
    Ok(entries.len() > 1)
}

/// Check if text contains scaffer template variables
fn contains_template_variables(text: &str) -> bool {
    use regex::Regex;

    let patterns = [
        r"\bScf[A-Z][a-zA-Z0-9]*\b",  // ScfMyvar
        r"\bSCF_[A-Z][A-Z0-9_]*\b",   // SCF_MYVAR
        r"\bSCF-[A-Z][A-Z0-9-]*\b",   // SCF-MYVAR
        r"\bSCF\.[A-Z][A-Z0-9\.]*\b", // SCF.MYVAR
        r"\bscf_[a-z][a-z0-9_]*\b",   // scf_myvar
        r"\bscf-[a-z][a-z0-9-]*\b",   // scf-myvar
        r"\bscf\.[a-z][a-z0-9\.]*\b", // scf.myvar
        r"\bscf[a-z][a-z0-9]*\b",     // scfmyvar
        r"\bSCF[A-Z][A-Z0-9]*\b",     // SCFMYVAR
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                return true;
            }
        }
    }

    false
}

/// Normalize a path string for cross-platform compatibility
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Check if a string is a valid URL
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Sanitize a filename by removing or replacing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            '/' | '\\' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_contains_template_variables() {
        assert!(contains_template_variables("ScfMyProject"));
        assert!(contains_template_variables("scf-my-project"));
        assert!(contains_template_variables("SCF_MY_PROJECT"));
        assert!(contains_template_variables("scf.my.project"));
        assert!(!contains_template_variables("regular text"));
        assert!(!contains_template_variables("scaffold"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("my<file>name"), "my_file_name");
        assert_eq!(sanitize_filename("path/to/file"), "path_to_file");
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
    }

    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://localhost:8080"));
        assert!(!is_url("file.zip"));
        assert!(!is_url("/path/to/file"));
    }
}
