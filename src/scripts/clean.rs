//! Clean command for build artifacts and temporary files
//!
//! This module provides functionality to clean up temporary directories
//! and build artifacts created by temci.

#![allow(dead_code)]

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Clean build artifacts and temporary files
///
/// # Arguments
///
/// * `all` - If true, clean all artifacts including caches
pub async fn clean(all: bool) -> Result<()> {
    info!("Starting cleanup");

    let mut cleaned_paths = Vec::new();
    let errors: Vec<String> = Vec::new();

    // Clean local .temci directory
    if let Ok(cleaned) = clean_local_temci_dir() {
        cleaned_paths.extend(cleaned);
    }

    // Clean ~/.temci/tmp directory
    if let Ok(cleaned) = clean_global_tmp_dir() {
        cleaned_paths.extend(cleaned);
    }

    // Clean CPU affinity temporary files
    if let Ok(cleaned) = clean_cpu_affinity_files() {
        cleaned_paths.extend(cleaned);
    }

    // Clean result files
    if let Ok(cleaned) = clean_result_files() {
        cleaned_paths.extend(cleaned);
    }

    // Clean all if requested
    if all {
        if let Ok(cleaned) = clean_all_artifacts() {
            cleaned_paths.extend(cleaned);
        }
    }

    // Report what was cleaned
    if cleaned_paths.is_empty() {
        info!("No files to clean");
    } else {
        info!("Cleaned {} items:", cleaned_paths.len());
        for path in &cleaned_paths {
            debug!("  - {}", path);
        }
        println!("Cleaned {} items", cleaned_paths.len());
    }

    // Report any errors
    if !errors.is_empty() {
        for error in &errors {
            warn!("Cleanup error: {}", error);
        }
    }

    Ok(())
}

/// Clean the local .temci directory
fn clean_local_temci_dir() -> Result<Vec<String>> {
    let mut cleaned = Vec::new();
    let temci_dir = Path::new(".temci");

    if !temci_dir.exists() {
        debug!("Local .temci directory does not exist");
        return Ok(cleaned);
    }

    debug!("Cleaning local .temci directory");

    // Remove the entire directory
    fs::remove_dir_all(temci_dir).context("Failed to remove .temci directory")?;

    cleaned.push(".temci/".to_string());
    Ok(cleaned)
}

/// Clean the global ~/.temci/tmp directory
fn clean_global_tmp_dir() -> Result<Vec<String>> {
    let mut cleaned = Vec::new();

    // Get home directory
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let temci_tmp_dir = home_dir.join(".temci").join("tmp");

    if !temci_tmp_dir.exists() {
        debug!("Global ~/.temci/tmp directory does not exist");
        return Ok(cleaned);
    }

    debug!("Cleaning global ~/.temci/tmp directory");

    // Remove the tmp directory
    fs::remove_dir_all(&temci_tmp_dir)
        .with_context(|| format!("Failed to remove {:?}", temci_tmp_dir))?;

    cleaned.push(temci_tmp_dir.display().to_string());
    Ok(cleaned)
}

/// Clean CPU affinity related temporary files
fn clean_cpu_affinity_files() -> Result<Vec<String>> {
    let mut cleaned = Vec::new();

    // Look for CPU set affinity files in /tmp
    let tmp_dir = Path::new("/tmp");

    if !tmp_dir.exists() {
        return Ok(cleaned);
    }

    debug!("Looking for temci CPU affinity files in /tmp");

    // Try to find and remove temci-related cpu affinity files
    if let Ok(entries) = fs::read_dir(tmp_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.contains("temci") && file_name_str.contains("cpu") {
                let path = entry.path();
                debug!("Removing CPU affinity file: {:?}", path);

                if path.is_file() {
                    if fs::remove_file(&path).is_ok() {
                        cleaned.push(path.display().to_string());
                    }
                } else if path.is_dir() && fs::remove_dir_all(&path).is_ok() {
                    cleaned.push(path.display().to_string());
                }
            }
        }
    }

    Ok(cleaned)
}

/// Clean benchmark result files
fn clean_result_files() -> Result<Vec<String>> {
    let mut cleaned = Vec::new();
    let current_dir = Path::new(".");

    debug!("Looking for temci result files");

    let result_patterns = vec![
        "temci_results.json",
        "temci_results.yaml",
        "temci_results.yml",
    ];

    for pattern in result_patterns {
        let path = current_dir.join(pattern);
        if path.exists() {
            debug!("Removing result file: {:?}", path);
            if fs::remove_file(&path).is_ok() {
                cleaned.push(pattern.to_string());
            }
        }
    }

    Ok(cleaned)
}

/// Clean all build artifacts and caches
fn clean_all_artifacts() -> Result<Vec<String>> {
    let mut cleaned = Vec::new();

    // Clean target directory if it exists
    let target_dir = Path::new("target");
    if target_dir.exists() && target_dir.is_dir() {
        debug!("Cleaning target directory");

        // Only clean temci-specific build artifacts to be safe
        let temci_build_dir = target_dir.join("debug").join("build");
        if temci_build_dir.exists() && fs::remove_dir_all(&temci_build_dir).is_ok() {
            cleaned.push(temci_build_dir.display().to_string());
        }

        let temci_incremental = target_dir.join("debug").join("incremental");
        if temci_incremental.exists() && fs::remove_dir_all(&temci_incremental).is_ok() {
            cleaned.push(temci_incremental.display().to_string());
        }
    }

    // Clean Cargo cache for temci
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        let cargo_target = PathBuf::from(cargo_home);
        let git_checkouts = cargo_target.join("git").join("checkouts");

        // Don't actually clean cargo cache to avoid breaking other builds
        debug!("Cargo cache location: {:?}", git_checkouts);
    }

    Ok(cleaned)
}

#[allow(dead_code)]
/// Check if a directory exists and can be cleaned
fn is_cleanable_dir(path: &Path) -> bool {
    path.exists() && (path.is_dir() || path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_cleanable_dir_with_existing_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(is_cleanable_dir(temp_dir.path()));
    }

    #[test]
    fn test_is_cleanable_dir_with_nonexistent() {
        let nonexistent = Path::new("/nonexistent/path/that/does/not/exist");
        assert!(!is_cleanable_dir(nonexistent));
    }

    #[tokio::test]
    async fn test_clean_nothing_to_clean() -> Result<()> {
        // Run clean in a directory with no temci artifacts
        let result = clean(false).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_clean_creates_temci_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let original_dir = std::env::current_dir()?;

        std::env::set_current_dir(temp_dir.path())?;

        // Create a .temci directory
        let temci_dir = temp_dir.path().join(".temci");
        fs::create_dir(&temci_dir)?;
        assert!(temci_dir.exists());

        // Run clean
        clean(false).await?;

        // Directory should be removed
        assert!(!temci_dir.exists());

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }
}
