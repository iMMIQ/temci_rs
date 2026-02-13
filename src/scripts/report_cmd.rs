//! Report generation command
//!
//! This module provides functionality to generate reports from saved benchmark results,
//! supporting various output formats including console, CSV, and JSON.

#![allow(dead_code)]

use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use tracing::{info, debug};

use crate::run::report::{Report, FormatType};
use crate::run::executor::BenchmarkSummary;
use crate::utils::time::{duration_as_ms, duration_to_ms};

/// Saved benchmark results that can be loaded from a file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SavedResults {
    /// Name of the benchmark suite
    name: String,
    /// Timestamp when the benchmarks were run
    timestamp: Option<String>,
    /// Individual benchmark results
    results: Vec<SavedBenchmarkResult>,
}

/// A single saved benchmark result
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct SavedBenchmarkResult {
    /// Command that was run
    command: String,
    /// Arguments to the command
    args: Vec<String>,
    /// Number of runs
    runs: usize,
    /// Successful runs
    successful: usize,
    /// Failed runs
    failed: usize,
    /// Minimum duration in milliseconds
    min_ms: Option<f64>,
    /// Maximum duration in milliseconds
    max_ms: Option<f64>,
    /// Average duration in milliseconds
    avg_ms: Option<f64>,
    /// Total time in milliseconds
    total_ms: f64,
}

impl From<SavedBenchmarkResult> for BenchmarkSummary {
    fn from(saved: SavedBenchmarkResult) -> Self {
        use std::time::Duration;

        // Create a minimal BenchmarkSummary from the saved data
        BenchmarkSummary {
            all_runs: Vec::new(), // Detailed run data not available in saved results
            benchmark_runs: Vec::new(),
            successful_runs: saved.successful,
            failed_runs: saved.failed,
            min_duration: saved.min_ms.map(|ms| Duration::from_secs_f64(ms / 1000.0)),
            max_duration: saved.max_ms.map(|ms| Duration::from_secs_f64(ms / 1000.0)),
            avg_duration: saved.avg_ms.map(|ms| Duration::from_secs_f64(ms / 1000.0)),
            total_time: Duration::from_secs_f64(saved.total_ms / 1000.0),
            command: saved.command,
            args: saved.args,
        }
    }
}

/// Generate benchmark report
///
/// # Arguments
///
/// * `format` - Output format (console, csv, json)
/// * `output` - Optional output file path
/// * `input` - Optional input data file path (defaults to looking for recent results)
pub async fn report(
    format: String,
    output: Option<String>,
    input: Option<String>,
) -> Result<()> {
    info!("Generating report in {} format", format);

    // Parse the format type
    let format_type = FormatType::from_str(&format)
        .ok_or_else(|| anyhow::anyhow!("Invalid format: {}", format))?;

    // Determine input source
    let input_path = input.unwrap_or_else(|| {
        // Look for default result files
        if Path::new("temci_results.json").exists() {
            "temci_results.json".to_string()
        } else if Path::new("temci_results.yaml").exists() {
            "temci_results.yaml".to_string()
        } else {
            "temci_results.json".to_string()
        }
    });

    debug!("Loading results from {}", input_path);

    // Load the saved results
    let saved_results = load_results(&input_path)?;

    // Convert saved results to BenchmarkSummaries
    let summaries: Vec<BenchmarkSummary> = saved_results.results
        .into_iter()
        .map(|r| r.into())
        .collect();

    if summaries.is_empty() {
        info!("No benchmark results found");
        return Ok(());
    }

    info!("Loaded {} benchmark results", summaries.len());

    // Create the report
    let report = Report::new(saved_results.name, summaries);

    // Generate the report content
    let report_content = report.generate(format_type);

    // Output to file or stdout
    if let Some(output_path) = output {
        fs::write(&output_path, report_content)
            .with_context(|| format!("Failed to write report to {}", output_path))?;
        info!("Report saved to {}", output_path);
    } else {
        println!("{}", report_content);
    }

    Ok(())
}

/// Load saved benchmark results from a file
///
/// Supports both JSON and YAML formats based on file extension
fn load_results(path: &str) -> Result<SavedResults> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read results from {}", path))?;

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "json" => {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON from {}", path))
        }
        "yaml" | "yml" => {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML from {}", path))
        }
        _ => {
            // Try JSON first, then YAML
            serde_json::from_str::<SavedResults>(&content)
                .or_else(|_| serde_yaml::from_str(&content))
                .with_context(|| format!("Failed to parse results from {}", path))
        }
    }
}

/// Save benchmark results to a file for later reporting
pub fn save_results(
    name: &str,
    summaries: &[BenchmarkSummary],
    path: &str,
) -> Result<()> {
    let results: Vec<SavedBenchmarkResult> = summaries
        .iter()
        .map(|s| SavedBenchmarkResult {
            command: s.command.clone(),
            args: s.args.clone(),
            runs: s.benchmark_runs.len(),
            successful: s.successful_runs,
            failed: s.failed_runs,
            min_ms: s.min_duration.map(duration_to_ms),
            max_ms: s.max_duration.map(duration_to_ms),
            avg_ms: s.avg_duration.map(duration_to_ms),
            total_ms: duration_as_ms(&s.total_time),
        })
        .collect();

    let saved = SavedResults {
        name: name.to_string(),
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
        results,
    };

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("json");

    let content = match ext {
        "yaml" | "yml" => serde_yaml::to_string(&saved)
            .context("Failed to serialize results to YAML")?,
        _ => serde_json::to_string_pretty(&saved)
            .context("Failed to serialize results to JSON")?,
    };

    fs::write(path, content)
        .with_context(|| format!("Failed to write results to {}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use std::time::Duration;

    #[test]
    fn test_format_type_parsing() {
        assert!(matches!(FormatType::from_str("console"), Some(FormatType::Text)));
        assert!(matches!(FormatType::from_str("csv"), Some(FormatType::Csv)));
        assert!(matches!(FormatType::from_str("json"), Some(FormatType::Json)));
        assert!(matches!(FormatType::from_str("text"), Some(FormatType::Text)));
        assert!(matches!(FormatType::from_str("markdown"), Some(FormatType::Markdown)));
        assert!(FormatType::from_str("invalid").is_none());
    }

    #[test]
    fn test_load_results_json() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, r#"{{"name": "test", "timestamp": "2024-01-01T00:00:00Z", "results": []}}"#)?;

        let results = load_results(temp_file.path().to_str().unwrap())?;
        assert_eq!(results.name, "test");
        assert!(results.results.is_empty());

        Ok(())
    }

    #[test]
    fn test_load_results_yaml() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "name: test")?;
        writeln!(temp_file, "timestamp: '2024-01-01T00:00:00Z'")?;
        writeln!(temp_file, "results: []")?;

        let results = load_results(temp_file.path().to_str().unwrap())?;
        assert_eq!(results.name, "test");
        assert!(results.results.is_empty());

        Ok(())
    }

    #[test]
    fn test_load_results_missing_file() {
        let result = load_results("nonexistent.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_benchmark_result_to_summary() {
        let saved = SavedBenchmarkResult {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            runs: 10,
            successful: 10,
            failed: 0,
            min_ms: Some(1.0),
            max_ms: Some(10.0),
            avg_ms: Some(5.0),
            total_ms: 50.0,
        };

        let summary: BenchmarkSummary = saved.into();

        assert_eq!(summary.command, "echo");
        assert_eq!(summary.args, vec!["hello"]);
        assert_eq!(summary.successful_runs, 10);
        assert_eq!(summary.failed_runs, 0);
        assert_eq!(summary.min_duration, Some(Duration::from_secs_f64(0.001)));
        assert_eq!(summary.max_duration, Some(Duration::from_secs_f64(0.010)));
        assert_eq!(summary.avg_duration, Some(Duration::from_secs_f64(0.005)));
        assert_eq!(summary.total_time, Duration::from_secs_f64(0.050));
    }

    #[test]
    fn test_save_results_json() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();

        let summary = BenchmarkSummary {
            all_runs: vec![],
            benchmark_runs: vec![],
            successful_runs: 5,
            failed_runs: 0,
            min_duration: Some(Duration::from_millis(10)),
            max_duration: Some(Duration::from_millis(20)),
            avg_duration: Some(Duration::from_millis(15)),
            total_time: Duration::from_millis(75),
            command: "test".to_string(),
            args: vec![],
        };

        let result = save_results("test_suite", &[summary], temp_path);
        assert!(result.is_ok());

        // Verify the file was created and can be read back
        let loaded = load_results(temp_path)?;
        assert_eq!(loaded.name, "test_suite");
        assert_eq!(loaded.results.len(), 1);
        assert_eq!(loaded.results[0].successful, 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_report_empty_format() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, r#"{{"name": "test", "results": []}}"#)?;

        let result = report("console".to_string(), None, Some(temp_file.path().to_str().unwrap().to_string())).await;
        assert!(result.is_ok());

        Ok(())
    }
}
