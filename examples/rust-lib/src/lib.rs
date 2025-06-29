//! # ScfProject
//!
//! A ScfProject library for scf-project functionality.
//!
//! ## Examples
//!
//! ```rust
//! use scf_project::ScfProject;
//!
//! let scf_project = ScfProject::new("example");
//! println!("Created: {}", scf_project.name());
//! ```

pub mod scf_project;
pub mod utils;

pub use scf_project::ScfProject;

/// The version of this ScfProject library
pub const SCF_PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the ScfProject library
pub fn init() {
    println!("Initializing scf-project library v{}", SCF_PROJECT_VERSION);
}

#[cfg(feature = "serde")]
pub mod serde_support {
    //! Serde support for ScfProject types
    pub use serde::{Deserialize, Serialize};
}

#[cfg(feature = "async")]
pub mod async_support {
    //! Async support for ScfProject operations
    use tokio::time::{sleep, Duration};
    
    /// Async initialization of ScfProject
    pub async fn async_init() {
        sleep(Duration::from_millis(100)).await;
        println!("Async initialization complete for scf-project");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scf_project_creation() {
        let project = ScfProject::new("test-scf-project");
        assert_eq!(project.name(), "test-scf-project");
    }

    #[test]
    fn test_version() {
        assert!(!SCF_PROJECT_VERSION.is_empty());
    }
} 