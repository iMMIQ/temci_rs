//! Short execution command for quick benchmarks
//!
//! This module provides a quick way to benchmark commands without creating
//! a configuration file. It's useful for ad-hoc benchmarking.

#![allow(dead_code)]

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::run::executor::{BenchmarkConfig, BenchmarkExecutor};
use crate::run::stats::{Sample, Statistics};
use crate::scripts::report_cmd::save_results;
use crate::utils::time::duration_as_ms;

/// Quick execution of commands with simple statistics
///
/// # Arguments
///
/// * `commands` - List of commands to execute (each as a full command string)
/// * `runs` - Number of benchmark runs (default 10)
/// * `warmup` - Number of warmup runs (default 0)
/// * `summary` - Whether to show only a summary (default detailed output)
/// * `output` - Optional output file path (default: temci_results.json)
/// * `no_save` - If true, don't save results to file
pub async fn short_exec(
    commands: Vec<String>,
    runs: usize,
    warmup: usize,
    summary: bool,
    output: Option<String>,
    no_save: bool,
) -> Result<()> {
    info!("Running short-exec with {} commands", commands.len());

    // Use default 20 runs if not specified, as per original spec
    let actual_runs = if runs == 10 { 20 } else { runs };
    let actual_warmup = if warmup == 0 && runs == 10 { 2 } else { warmup };

    debug!(
        "Configuration: runs={}, warmup={}",
        actual_runs, actual_warmup
    );

    // Determine output file path (default: temci_results.json)
    let output_path = if no_save {
        None
    } else {
        Some(output.unwrap_or_else(|| "temci_results.json".to_string()))
    };

    // Create executor with default settings
    let mut executor = BenchmarkExecutor::default_executor();

    // Results for all commands
    let mut all_results: Vec<CommandBenchmarkResult> = Vec::new();

    // Run each command
    for (idx, cmd_str) in commands.iter().enumerate() {
        info!(
            "Running command {}/{}: {}",
            idx + 1,
            commands.len(),
            cmd_str
        );

        // Parse the command string into command and args
        let (command, args) = parse_command_string(cmd_str)?;

        // Convert String args to &str for the executor
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        // Create benchmark config
        let config = BenchmarkConfig::new()
            .with_runs(actual_runs)
            .with_warmup(actual_warmup);

        // Run the benchmark
        match executor.run_benchmark(&command, &args_refs, &config).await {
            Ok(benchmark_summary) => {
                // Calculate additional statistics
                let durations: Vec<f64> = benchmark_summary
                    .benchmark_runs
                    .iter()
                    .filter_map(|r| {
                        if r.is_success() {
                            Some(duration_as_ms(&r.duration()))
                        } else {
                            None
                        }
                    })
                    .collect();

                let stats = if !durations.is_empty() {
                    Some(Statistics::from_sample(&Sample::new(durations)))
                } else {
                    None
                };

                let result = CommandBenchmarkResult {
                    command: cmd_str.clone(),
                    summary: benchmark_summary,
                    stats,
                };

                if !summary {
                    print_detailed_result(&result);
                }

                all_results.push(result);
            }
            Err(e) => {
                eprintln!("Command '{}' failed: {}", cmd_str, e);
            }
        }
    }

    // Print summary if requested or if multiple commands were run
    if summary || commands.len() > 1 {
        print_comparison_summary(&all_results);
    }

    // Save results to file if not disabled
    if let Some(path) = output_path {
        // Convert CommandBenchmarkResult to BenchmarkSummary for saving
        let summaries: Vec<crate::run::executor::BenchmarkSummary> =
            all_results.iter().map(|r| r.summary.clone()).collect();

        save_results("temci short-exec", &summaries, &path)
            .with_context(|| format!("Failed to save results to {}", path))?;
        info!("Results saved to {}", path);

        // Also print a message about using report command
        if !summary {
            println!(
                "\nResults saved to {}. Use 'temci report' to generate a formatted report.",
                path
            );
        }
    }

    Ok(())
}

/// Result of benchmarking a single command
struct CommandBenchmarkResult {
    command: String,
    summary: crate::run::executor::BenchmarkSummary,
    stats: Option<Statistics>,
}

/// Parse a command string into command and arguments
fn parse_command_string(cmd_str: &str) -> Result<(String, Vec<String>)> {
    let parts = shell_words::split(cmd_str)
        .with_context(|| format!("Failed to parse command: {}", cmd_str))?;

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    let command = parts[0].clone();
    let args = parts[1..].to_vec();

    Ok((command, args))
}

/// Print detailed results for a single command
fn print_detailed_result(result: &CommandBenchmarkResult) {
    println!("\n{}", result.command);
    println!("{}", "=".repeat(60));

    let summary = &result.summary;
    println!("  Runs: {}", summary.benchmark_runs.len());
    println!("  Successful: {}", summary.successful_runs);
    println!("  Failed: {}", summary.failed_runs);

    if let Some(ref stats) = result.stats {
        println!("  Time Statistics:");
        println!("    Min:     {:.2} ms", stats.min.unwrap_or(0.0));
        println!("    Max:     {:.2} ms", stats.max.unwrap_or(0.0));
        println!("    Mean:    {:.2} ms", stats.mean.unwrap_or(0.0));
        println!("    Median:  {:.2} ms", stats.median.unwrap_or(0.0));
        println!("    Std Dev: {:.2} ms", stats.std_dev.unwrap_or(0.0));

        if let (Some(p25), Some(p75)) = (stats.p25, stats.p75) {
            println!("    IQR:     {:.2} ms", p75 - p25);
        }

        if let Some(p90) = stats.p90 {
            println!("    90th:    {:.2} ms", p90);
        }
        if let Some(p95) = stats.p95 {
            println!("    95th:    {:.2} ms", p95);
        }
        if let Some(p99) = stats.p99 {
            println!("    99th:    {:.2} ms", p99);
        }
    }

    println!("  Total:   {:.2} ms", duration_as_ms(&summary.total_time));
}

/// Print a comparison summary for all commands
#[allow(dead_code)]
fn print_comparison_summary(results: &[CommandBenchmarkResult]) {
    println!("\n{}", "=".repeat(60));
    println!("SUMMARY");
    println!("{}", "=".repeat(60));

    // Find the fastest command
    let fastest = results.iter().filter(|r| r.stats.is_some()).min_by(|a, b| {
        let a_mean = a.stats.as_ref().unwrap().mean.unwrap_or(f64::MAX);
        let b_mean = b.stats.as_ref().unwrap().mean.unwrap_or(f64::MAX);
        a_mean
            .partial_cmp(&b_mean)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, result) in results.iter().enumerate() {
        println!("\n{}) {}", idx + 1, result.command);

        if let Some(ref stats) = result.stats {
            let mean = stats.mean.unwrap_or(0.0);
            print!("  Mean: {:.2} ms", mean);

            // Show relative speed compared to fastest
            if let Some(fastest_result) = fastest {
                if result.command != fastest_result.command {
                    let fastest_mean = fastest_result.stats.as_ref().unwrap().mean.unwrap_or(1.0);
                    if fastest_mean > 0.0 {
                        let relative = mean / fastest_mean;
                        print!(" ({:.2}x slower)", relative);
                    }
                } else {
                    print!(" (fastest)");
                }
            }

            println!();
            println!("  Min:  {:.2} ms", stats.min.unwrap_or(0.0));
            println!("  Max:  {:.2} ms", stats.max.unwrap_or(0.0));
        } else {
            println!("  No valid results");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_simple() {
        let result = parse_command_string("echo hello").unwrap();
        assert_eq!(result.0, "echo");
        assert_eq!(result.1, vec!["hello"]);
    }

    #[test]
    fn test_parse_command_with_quotes() {
        let result = parse_command_string("echo \"hello world\"").unwrap();
        assert_eq!(result.0, "echo");
        assert_eq!(result.1, vec!["hello world"]);
    }

    #[test]
    fn test_parse_command_empty() {
        let result = parse_command_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_complex() {
        let result = parse_command_string("ls -la /tmp").unwrap();
        assert_eq!(result.0, "ls");
        assert_eq!(result.1, vec!["-la", "/tmp"]);
    }

    #[tokio::test]
    async fn test_short_exec_single_command() -> Result<()> {
        let result = short_exec(vec!["echo test".to_string()], 5, 1, true, None, true).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_short_exec_multiple_commands() -> Result<()> {
        let result = short_exec(
            vec!["echo test1".to_string(), "echo test2".to_string()],
            3,
            0,
            true,
            None,
            true,
        )
        .await;
        assert!(result.is_ok());
        Ok(())
    }
}
