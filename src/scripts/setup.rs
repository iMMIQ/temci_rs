//! Setup command for initial configuration
//!
//! This module provides functionality to set up temci for first-time use,
//! including checking for required tools and creating configuration files.

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tracing::{info, debug, warn};

use crate::build::builder::{CompilerType, OptimizationLevel};

/// Default configuration file name
const DEFAULT_CONFIG_NAME: &str = "temci.yaml";

/// Default temci directory name
const TEMCI_DIR_NAME: &str = ".temci";

/// Setup initial configuration
///
/// # Arguments
///
/// * `config` - Configuration file path (default: temci.yaml)
/// * `overwrite` - Whether to overwrite existing config
pub async fn setup(config: String, overwrite: bool) -> Result<()> {
    info!("Starting temci setup");

    // Check if config already exists
    let config_path = Path::new(&config);

    if config_path.exists() && !overwrite {
        return Err(anyhow::anyhow!(
            "Configuration file {} already exists. Use --overwrite to replace it.",
            config
        ));
    }

    // Create .temci directory
    create_temci_directory()?;

    // Check system requirements
    check_system_requirements()?;

    // Create configuration file
    create_config_file(&config, overwrite)?;

    // Print summary
    print_setup_summary(&config);

    info!("Setup complete");
    Ok(())
}

/// Create the .temci directory structure
fn create_temci_directory() -> Result<()> {
    let temci_dir = Path::new(TEMCI_DIR_NAME);

    if !temci_dir.exists() {
        fs::create_dir(&temci_dir)
            .context("Failed to create .temci directory")?;
        info!("Created .temci directory");
    } else {
        debug!(".temci directory already exists");
    }

    // Create subdirectories
    let subdirs = vec!["tmp", "cache", "results"];

    for subdir in subdirs {
        let path = temci_dir.join(subdir);
        if !path.exists() {
            fs::create_dir(&path)
                .with_context(|| format!("Failed to create {}/{}", TEMCI_DIR_NAME, subdir))?;
            debug!("Created {}/{}", TEMCI_DIR_NAME, subdir);
        }
    }

    Ok(())
}

/// Check system requirements and available tools
fn check_system_requirements() -> Result<SystemInfo> {
    info!("Checking system requirements");

    let mut sys_info = SystemInfo::new();

    // Check CPU information
    sys_info.cpus = num_cpus::get();
    debug!("Detected {} CPUs", sys_info.cpus);

    // Check for perf (Linux perf events)
    sys_info.has_perf = which::which("perf").is_ok();
    if sys_info.has_perf {
        debug!("perf is available");
    } else {
        debug!("perf is not available");
    }

    // Check for valgrind
    sys_info.has_valgrind = which::which("valgrind").is_ok();
    if sys_info.has_valgrind {
        debug!("valgrind is available");
    } else {
        debug!("valgrind is not available");
    }

    // Check for compilers
    sys_info.has_gcc = which::which("gcc").is_ok();
    if sys_info.has_gcc {
        debug!("gcc is available");
    }

    sys_info.has_clang = which::which("clang").is_ok();
    if sys_info.has_clang {
        debug!("clang is available");
    }

    sys_info.has_rustc = which::which("rustc").is_ok();
    if sys_info.has_rustc {
        debug!("rustc is available");
    }

    // Check for hyperfine (alternative benchmarking tool)
    sys_info.has_hyperfine = which::which("hyperfine").is_ok();

    sys_info.has_python3 = which::which("python3").is_ok();

    Ok(sys_info)
}

/// System information collected during setup
#[derive(Debug)]
struct SystemInfo {
    cpus: usize,
    has_perf: bool,
    has_valgrind: bool,
    has_gcc: bool,
    has_clang: bool,
    has_rustc: bool,
    has_hyperfine: bool,
    has_python3: bool,
}

impl SystemInfo {
    fn new() -> Self {
        Self {
            cpus: 0,
            has_perf: false,
            has_valgrind: false,
            has_gcc: false,
            has_clang: false,
            has_rustc: false,
            has_hyperfine: false,
            has_python3: false,
        }
    }

    /// Get a recommended compiler based on availability
    fn recommended_compiler(&self) -> Option<CompilerType> {
        if self.has_clang {
            Some(CompilerType::Clang)
        } else if self.has_gcc {
            Some(CompilerType::Gcc)
        } else if self.has_rustc {
            Some(CompilerType::Rustc)
        } else {
            None
        }
    }
}

/// Create a default configuration file
fn create_config_file(config_path: &str, overwrite: bool) -> Result<()> {
    info!("Creating configuration file: {}", config_path);

    let config_content = generate_default_config()?;

    // Write config file
    if overwrite {
        fs::write(config_path, config_content)
            .context("Failed to write config file")?;
    } else {
        fs::write(config_path, config_content)
            .context("Failed to write config file")?;
    }

    Ok(())
}

/// Generate default configuration content
fn generate_default_config() -> Result<String> {
    let config = r#"# temci configuration file
# This file contains default benchmarking settings

# Number of runs for each benchmark
runs: 10

# Number of warmup runs before measuring
warmup_runs: 3

# Run driver to use: basic, perf_stat, perf_record, valgrind
driver: basic

# Disable CPU affinity (set to true for more variable results)
no_affinity: false

# Output format: console, csv, json, markdown
output_format: console

# Save results to file (optional)
# output_file: temci_results.json

# Benchmark suite
benchmarks:
  # Example benchmarks
  - command: echo
    args: ["hello"]
    name: "Example echo benchmark"

  - command: sleep
    args: ["0.001"]
    name: "Example sleep benchmark"
"#;

    Ok(config.to_string())
}

/// Print a summary of the setup
fn print_setup_summary(config_path: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Setup Summary");
    println!("{}", "=".repeat(60));

    println!("\nConfiguration file: {}", config_path);
    println!("Temci directory: {}", TEMCI_DIR_NAME);

    // Print system info
    if let Ok(sys_info) = check_system_requirements() {
        println!("\nSystem Information:");
        println!("  CPUs: {}", sys_info.cpus);
        println!("  perf: {}", if sys_info.has_perf { "available" } else { "not available" });
        println!("  valgrind: {}", if sys_info.has_valgrind { "available" } else { "not available" });

        println!("\nCompilers:");
        println!("  gcc: {}", if sys_info.has_gcc { "available" } else { "not available" });
        println!("  clang: {}", if sys_info.has_clang { "available" } else { "not available" });
        println!("  rustc: {}", if sys_info.has_rustc { "available" } else { "not available" });

        if let Some(compiler) = sys_info.recommended_compiler() {
            println!("  Recommended: {}", compiler);
        }
    }

    println!("\nNext steps:");
    println!("  1. Edit {} to configure your benchmarks", config_path);
    println!("  2. Run: temci exec");
    println!("  3. For quick benchmarks: temci short-exec <command>");
    println!("\n{}", "=".repeat(60));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_default_config() {
        let config = generate_default_config().unwrap();
        assert!(config.contains("runs: 10"));
        assert!(config.contains("warmup_runs: 3"));
        assert!(config.contains("driver: basic"));
        assert!(config.contains("benchmarks:"));
    }

    #[tokio::test]
    async fn test_setup_creates_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("temci.yaml");
        let config_str = config_path.to_str().unwrap();

        std::env::set_current_dir(temp_dir.path())?;

        // Run setup
        let result = setup(config_str.to_string(), false).await;
        assert!(result.is_ok());

        // Check that config was created
        assert!(config_path.exists());

        // Read and verify content
        let content = fs::read_to_string(&config_path)?;
        assert!(content.contains("runs: 10"));

        // Running again without overwrite should fail
        let result2 = setup(config_str.to_string(), false).await;
        assert!(result2.is_err());

        // Running with overwrite should succeed
        let result3 = setup(config_str.to_string(), true).await;
        assert!(result3.is_ok());

        std::env::set_current_dir(std::env::current_dir()?.join("../"))?;
        Ok(())
    }

    #[test]
    fn test_system_info_new() {
        let sys_info = SystemInfo::new();
        assert_eq!(sys_info.cpus, 0);
        assert!(!sys_info.has_perf);
        assert!(!sys_info.has_valgrind);
    }

    #[test]
    fn test_system_info_recommended_compiler() {
        let mut sys_info = SystemInfo::new();

        // No compiler available
        assert!(sys_info.recommended_compiler().is_none());

        // Clang available
        sys_info.has_clang = true;
        assert_eq!(sys_info.recommended_compiler(), Some(CompilerType::Clang));

        // GCC available instead
        sys_info.has_clang = false;
        sys_info.has_gcc = true;
        assert_eq!(sys_info.recommended_compiler(), Some(CompilerType::Gcc));

        // Rust available
        sys_info.has_gcc = false;
        sys_info.has_rustc = true;
        assert_eq!(sys_info.recommended_compiler(), Some(CompilerType::Rustc));
    }
}
