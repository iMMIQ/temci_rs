//! Benchmark execution command
//!
//! This module provides the main benchmark execution functionality, including:
//! - Loading run configuration from YAML files
//! - Running benchmarks with the BenchmarkExecutor
//! - Generating reports in console or CSV format
//! - Saving results to files

#![allow(dead_code)]

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::run::executor::{BenchmarkConfig, BenchmarkExecutor};
use crate::run::report::{FormatType, Report};
use crate::utils::time::duration_as_ms;

/// Configuration loaded from a YAML file for benchmark execution
#[derive(Debug, serde::Deserialize)]
struct RunConfig {
    /// Number of runs per benchmark
    runs: Option<usize>,
    /// Number of warmup runs
    warmup_runs: Option<usize>,
    /// Timeout in seconds
    timeout_sec: Option<u64>,
    /// Working directory
    working_dir: Option<String>,
    /// Environment variables
    env: Option<Vec<(String, String)>>,
    /// Max concurrent executions
    max_concurrent: Option<usize>,
    /// Output format (console, csv, json)
    output_format: Option<String>,
    /// Output file path
    output_file: Option<String>,
    /// Benchmarks to run
    benchmarks: Vec<BenchmarkSpec>,
}

/// Specification for a single benchmark
#[derive(Debug, serde::Deserialize, Clone)]
struct BenchmarkSpec {
    /// Command to execute
    command: String,
    /// Arguments for the command
    args: Vec<String>,
    /// Optional name for the benchmark
    name: Option<String>,
    /// Optional override for runs
    runs: Option<usize>,
    /// Optional override for warmup runs
    warmup_runs: Option<usize>,
}

impl BenchmarkSpec {
    /// Get the display name for this benchmark
    fn display_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}

/// Execute benchmarks with full configuration
///
/// # Arguments
///
/// * `suite` - Optional path to benchmark suite YAML file
/// * `runs` - Override for number of runs
/// * `driver` - Optional run driver (basic, perf, perf_stat, perf_record, valgrind)
/// * `no_affinity` - Disable CPU affinity
/// * `summary` - Show only summary (not detailed output)
pub async fn exec(
    suite: Option<String>,
    runs: Option<usize>,
    driver: Option<String>,
    no_affinity: bool,
    summary: bool,
) -> Result<()> {
    info!("Starting benchmark execution");

    // Determine configuration source
    let config_path = suite.unwrap_or_else(|| "temci.yaml".to_string());

    // Load configuration from file
    let run_config = load_config(&config_path)?;

    debug!(
        "Loaded configuration with {} benchmarks",
        run_config.benchmarks.len()
    );

    // Check driver availability if specified
    if let Some(driver_name) = &driver {
        validate_driver(driver_name)?;
    }

    // Set up CPU affinity if not disabled
    if !no_affinity {
        setup_cpu_affinity()?;
    }

    // Build benchmark configuration from loaded config
    let benchmark_config = build_benchmark_config(&run_config, runs)?;

    // Create executor
    let max_concurrent = benchmark_config.max_concurrent;
    let mut executor = BenchmarkExecutor::new(max_concurrent);

    // Run all benchmarks
    let mut all_summaries = Vec::new();
    for (idx, spec) in run_config.benchmarks.iter().enumerate() {
        info!(
            "Running benchmark {}/{}: {}",
            idx + 1,
            run_config.benchmarks.len(),
            spec.display_name()
        );

        // Apply per-benchmark overrides
        let mut spec_config = benchmark_config.clone();
        if let Some(spec_runs) = spec.runs {
            spec_config.runs = spec_runs;
        }
        if let Some(spec_warmup) = spec.warmup_runs {
            spec_config.warmup_runs = spec_warmup;
        }

        // Convert args to &str for executor
        let args: Vec<&str> = spec.args.iter().map(|s| s.as_str()).collect();

        match executor
            .run_benchmark(&spec.command, &args, &spec_config)
            .await
        {
            Ok(benchmark_summary) => {
                all_summaries.push(benchmark_summary.clone());

                if !summary {
                    // Print quick summary for each benchmark if not in summary-only mode
                    print_quick_summary(&spec.display_name(), &benchmark_summary);
                }
            }
            Err(e) => {
                warn!("Benchmark {} failed: {}", spec.display_name(), e);
            }
        }
    }

    // Generate and output report
    let output_format = run_config.output_format.as_deref().unwrap_or("console");

    let format_type = FormatType::from_str(output_format)
        .ok_or_else(|| anyhow::anyhow!("Invalid output format: {}", output_format))?;

    // Create report from all summaries
    let config_path_buf = PathBuf::from(&config_path);
    let command_name = config_path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("benchmarks")
        .to_string();

    let report = Report::new(command_name, all_summaries);

    // Generate report content
    let report_content = report.generate(format_type);

    // Output to file or stdout
    if let Some(output_path) = run_config.output_file {
        save_report(&output_path, &report_content)?;
        info!("Report saved to {}", output_path);
    } else {
        // Print to stdout
        if summary {
            print_summary_report(&report);
        } else {
            println!("{}", report_content);
        }
    }

    info!("Benchmark execution completed");
    Ok(())
}

/// Load run configuration from a YAML file
fn load_config(path: &str) -> Result<RunConfig> {
    let file = File::open(path).with_context(|| format!("Failed to open config file: {}", path))?;

    serde_yaml::from_reader(file).with_context(|| format!("Failed to parse config file: {}", path))
}

/// Validate that the specified driver is available
fn validate_driver(driver_name: &str) -> Result<()> {
    match driver_name {
        "basic" => Ok(()), // Always available
        "perf" | "perf_stat" | "perf_record" => {
            if which::which("perf").is_ok() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("perf is not available on this system"))
            }
        }
        "valgrind" => {
            if which::which("valgrind").is_ok() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("valgrind is not available on this system"))
            }
        }
        _ => Err(anyhow::anyhow!("Unknown driver: {}", driver_name)),
    }
}

/// Set up CPU affinity for consistent benchmarking
fn setup_cpu_affinity() -> Result<()> {
    // Try to set CPU affinity to a single CPU core
    // This is a placeholder - actual implementation would use the cpuset module
    debug!("CPU affinity setup requested (not fully implemented yet)");
    Ok(())
}

/// Build a BenchmarkConfig from the loaded RunConfig
fn build_benchmark_config(
    config: &RunConfig,
    runs_override: Option<usize>,
) -> Result<BenchmarkConfig> {
    let runs = runs_override.or(config.runs).unwrap_or(10);
    let warmup_runs = config.warmup_runs.unwrap_or(3);

    let mut benchmark_config = BenchmarkConfig::new()
        .with_runs(runs)
        .with_warmup(warmup_runs)
        .with_max_concurrent(config.max_concurrent.unwrap_or(1));

    // Set timeout if specified
    if let Some(timeout_sec) = config.timeout_sec {
        use std::time::Duration;
        benchmark_config = benchmark_config.with_timeout(Duration::from_secs(timeout_sec));
    }

    // Set working directory if specified
    if let Some(dir) = &config.working_dir {
        benchmark_config = benchmark_config.with_working_dir(dir);
    }

    // Set environment variables
    if let Some(env_vars) = &config.env {
        for (key, value) in env_vars {
            benchmark_config = benchmark_config.with_env(key, value);
        }
    }

    Ok(benchmark_config)
}

/// Print a quick summary for a single benchmark
fn print_quick_summary(name: &str, summary: &crate::run::executor::BenchmarkSummary) {
    println!("\n{}", name);
    println!("{}", "=".repeat(60));
    println!("  Runs: {}", summary.benchmark_runs.len());
    println!("  Successful: {}", summary.successful_runs);
    println!("  Failed: {}", summary.failed_runs);

    if let Some(avg) = summary.avg_duration {
        println!("  Avg: {:.2} ms", duration_as_ms(&avg));
    }
    if let Some(min) = summary.min_duration {
        println!("  Min: {:.2} ms", duration_as_ms(&min));
    }
    if let Some(max) = summary.max_duration {
        println!("  Max: {:.2} ms", duration_as_ms(&max));
    }
}

/// Print a summary report (aggregated results)
fn print_summary_report(report: &Report) {
    println!("\nBenchmark Summary");
    println!("{}", "=".repeat(60));

    for (idx, summary) in report.entries().iter().enumerate() {
        println!("\n{}:", idx + 1);
        if let Some(avg) = summary.avg_duration {
            println!("  Avg: {:.2} ms", duration_as_ms(&avg));
        }
        println!("  Runs: {} successful", summary.successful_runs);
        if summary.failed_runs > 0 {
            println!("  Failed: {}", summary.failed_runs);
        }
    }
}

/// Save report content to a file
fn save_report(path: &str, content: &str) -> Result<()> {
    let mut file =
        File::create(path).with_context(|| format!("Failed to create output file: {}", path))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to output file: {}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_driver_basic() {
        assert!(validate_driver("basic").is_ok());
    }

    #[test]
    fn test_validate_driver_invalid() {
        assert!(validate_driver("invalid_driver").is_err());
    }

    #[test]
    fn test_load_config_missing_file() {
        let result = load_config("nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_valid_yaml() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "runs: 20")?;
        writeln!(temp_file, "warmup_runs: 2")?;
        writeln!(temp_file, "benchmarks: []")?;

        let config = load_config(temp_file.path().to_str().unwrap())?;
        assert_eq!(config.runs, Some(20));
        assert_eq!(config.warmup_runs, Some(2));
        assert!(config.benchmarks.is_empty());

        Ok(())
    }

    #[test]
    fn test_benchmark_spec_display_name() {
        let spec = BenchmarkSpec {
            command: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            name: None,
            runs: None,
            warmup_runs: None,
        };

        assert_eq!(spec.display_name(), "echo hello world");

        let spec_with_name = BenchmarkSpec {
            name: Some("my_benchmark".to_string()),
            ..spec.clone()
        };

        assert_eq!(spec_with_name.display_name(), "my_benchmark");
    }

    #[test]
    fn test_build_benchmark_config_defaults() {
        let run_config = RunConfig {
            runs: None,
            warmup_runs: None,
            timeout_sec: None,
            working_dir: None,
            env: None,
            max_concurrent: None,
            output_format: None,
            output_file: None,
            benchmarks: vec![],
        };

        let config = build_benchmark_config(&run_config, None).unwrap();
        assert_eq!(config.runs, 10); // Default
        assert_eq!(config.warmup_runs, 3); // Default
        assert_eq!(config.max_concurrent, 1); // Default
    }

    #[test]
    fn test_build_benchmark_config_with_overrides() {
        let run_config = RunConfig {
            runs: Some(20),
            warmup_runs: Some(5),
            timeout_sec: Some(60),
            working_dir: Some("/tmp".to_string()),
            env: Some(vec![("KEY".to_string(), "value".to_string())]),
            max_concurrent: Some(4),
            output_format: None,
            output_file: None,
            benchmarks: vec![],
        };

        let config = build_benchmark_config(&run_config, Some(30)).unwrap();
        assert_eq!(config.runs, 30); // Override takes precedence
        assert_eq!(config.warmup_runs, 5);
        assert_eq!(config.max_concurrent, 4);
    }
}
