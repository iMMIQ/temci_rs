//! Report generation for benchmark results
//!
//! Provides formatters for generating reports in various formats
//! including text, markdown, JSON, and CSV.

#![allow(dead_code)]

use crate::run::executor::BenchmarkSummary;
use crate::run::stats::{Sample, Statistics};
use crate::utils::time::duration_as_ms;
use serde::Serialize;

/// Output format types for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    /// Plain text format
    Text,
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

impl FormatType {
    /// Parse a string into a FormatType
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" | "console" => Some(FormatType::Text),
            "markdown" | "md" => Some(FormatType::Markdown),
            "json" => Some(FormatType::Json),
            "csv" => Some(FormatType::Csv),
            _ => None,
        }
    }
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatType::Text => write!(f, "text"),
            FormatType::Markdown => write!(f, "markdown"),
            FormatType::Json => write!(f, "json"),
            FormatType::Csv => write!(f, "csv"),
        }
    }
}

/// A report containing one or more benchmark summaries
#[derive(Debug, Clone)]
pub struct Report {
    /// Command that was benchmarked
    command: String,
    /// Benchmark summaries
    summaries: Vec<BenchmarkSummary>,
}

impl Report {
    /// Create a new report
    pub fn new(command: impl Into<String>, summaries: Vec<BenchmarkSummary>) -> Self {
        Self {
            command: command.into(),
            summaries,
        }
    }

    /// Get the command name
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Get the benchmark summaries
    pub fn entries(&self) -> &[BenchmarkSummary] {
        &self.summaries
    }

    /// Add a summary to the report
    pub fn add_summary(&mut self, summary: BenchmarkSummary) {
        self.summaries.push(summary);
    }

    /// Generate a report in the specified format
    pub fn generate(&self, format: FormatType) -> String {
        match format {
            FormatType::Text => self.generate_text(),
            FormatType::Markdown => self.generate_markdown(),
            FormatType::Json => self.generate_json(),
            FormatType::Csv => self.generate_csv(),
        }
    }

    /// Generate plain text report
    fn generate_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Benchmark Report: {}\n", self.command));
        output.push_str(&"=".repeat(60));
        output.push('\n');

        for (i, summary) in self.summaries.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }

            output.push_str(&format!("Summary {}:\n", i + 1));
            output.push_str(&format!("  Command: {}\n", summary.command));
            output.push_str(&format!("  Args: {:?}\n", summary.args));
            output.push_str(&format!("  Runs: {}\n", summary.benchmark_runs.len()));
            output.push_str(&format!("  Successful: {}\n", summary.successful_runs));
            output.push_str(&format!("  Failed: {}\n", summary.failed_runs));
            output.push_str(&format!(
                "  Success Rate: {:.2}%\n",
                summary.success_rate() * 100.0
            ));

            if let Some(min) = summary.min_duration {
                output.push_str(&format!("  Min Duration: {:?}\n", min));
            }
            if let Some(max) = summary.max_duration {
                output.push_str(&format!("  Max Duration: {:?}\n", max));
            }
            if let Some(avg) = summary.avg_duration {
                output.push_str(&format!("  Avg Duration: {:?}\n", avg));
            }
            output.push_str(&format!("  Total Time: {:?}\n", summary.total_time));
        }

        output
    }

    /// Generate markdown report
    fn generate_markdown(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# Benchmark Report: {}\n\n", self.command));

        for (i, summary) in self.summaries.iter().enumerate() {
            if i > 0 {
                output.push_str("\n---\n\n");
            }

            output.push_str(&format!("## Summary {}\n\n", i + 1));
            output.push_str(&format!("- **Command**: `{}`\n", summary.command));
            output.push_str(&format!("- **Args**: `{:?}`\n", summary.args));
            output.push_str(&format!("- **Runs**: {}\n", summary.benchmark_runs.len()));
            output.push_str(&format!("- **Successful**: {}\n", summary.successful_runs));
            output.push_str(&format!("- **Failed**: {}\n", summary.failed_runs));
            output.push_str(&format!(
                "- **Success Rate**: {:.2}%\n",
                summary.success_rate() * 100.0
            ));

            if let Some(min) = summary.min_duration {
                output.push_str(&format!("- **Min Duration**: `{:?}`\n", min));
            }
            if let Some(max) = summary.max_duration {
                output.push_str(&format!("- **Max Duration**: `{:?}`\n", max));
            }
            if let Some(avg) = summary.avg_duration {
                output.push_str(&format!("- **Avg Duration**: `{:?}`\n", avg));
            }
            output.push_str(&format!("- **Total Time**: `{:?}`\n", summary.total_time));
        }

        output
    }

    /// Generate JSON report
    fn generate_json(&self) -> String {
        #[derive(Serialize)]
        struct JsonReport {
            command: String,
            summaries: Vec<JsonSummary>,
        }

        #[derive(Serialize)]
        struct JsonSummary {
            command: String,
            args: Vec<String>,
            runs: usize,
            successful_runs: usize,
            failed_runs: usize,
            success_rate: f64,
            min_duration_ms: Option<f64>,
            max_duration_ms: Option<f64>,
            avg_duration_ms: Option<f64>,
            total_time_ms: f64,
        }

        let summaries: Vec<JsonSummary> = self
            .summaries
            .iter()
            .map(|s| {
                let durations: Vec<f64> = s
                    .benchmark_runs
                    .iter()
                    .filter_map(|r| {
                        if r.is_success() {
                            Some(duration_as_ms(&r.result.duration))
                        } else {
                            None
                        }
                    })
                    .collect();

                let stats = Statistics::from_sample(&Sample::new(durations));

                JsonSummary {
                    command: s.command.clone(),
                    args: s.args.clone(),
                    runs: s.benchmark_runs.len(),
                    successful_runs: s.successful_runs,
                    failed_runs: s.failed_runs,
                    success_rate: s.success_rate(),
                    min_duration_ms: stats.min,
                    max_duration_ms: stats.max,
                    avg_duration_ms: stats.mean,
                    total_time_ms: duration_as_ms(&s.total_time),
                }
            })
            .collect();

        let report = JsonReport {
            command: self.command.clone(),
            summaries,
        };

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    /// Generate CSV report
    fn generate_csv(&self) -> String {
        let mut output = String::new();
        output.push_str(
            "command,runs,successful,failed,success_rate,min_ms,max_ms,avg_ms,total_ms\n",
        );

        for summary in &self.summaries {
            let durations: Vec<f64> = summary
                .benchmark_runs
                .iter()
                .filter_map(|r| {
                    if r.is_success() {
                        Some(duration_as_ms(&r.result.duration))
                    } else {
                        None
                    }
                })
                .collect();

            let stats = Statistics::from_sample(&Sample::new(durations));

            output.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                escape_csv(&summary.command),
                summary.benchmark_runs.len(),
                summary.successful_runs,
                summary.failed_runs,
                summary.success_rate(),
                stats.min.map_or("".to_string(), |v| v.to_string()),
                stats.max.map_or("".to_string(), |v| v.to_string()),
                stats.mean.map_or("".to_string(), |v| v.to_string()),
                duration_as_ms(&summary.total_time),
            ));
        }

        output
    }
}

/// Escape a value for CSV output
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace("\"", "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_simple() {
        assert_eq!(escape_csv("hello"), "hello");
    }

    #[test]
    fn test_escape_csv_comma() {
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_quote() {
        assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
