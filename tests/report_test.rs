use std::time::Duration;
use temci::run::executor::{BenchmarkSummary, RunResult};
use temci::run::report::{FormatType, Report};
use temci::run::runner::CommandResult;
use temci::run::stats::Sample;

#[test]
fn test_format_type_from_str() {
    assert_eq!(FormatType::from_str("text"), Some(FormatType::Text));
    assert_eq!(FormatType::from_str("markdown"), Some(FormatType::Markdown));
    assert_eq!(FormatType::from_str("json"), Some(FormatType::Json));
    assert_eq!(FormatType::from_str("csv"), Some(FormatType::Csv));
    assert_eq!(FormatType::from_str("invalid"), None);
}

#[test]
fn test_format_type_display() {
    assert_eq!(FormatType::Text.to_string(), "text");
    assert_eq!(FormatType::Markdown.to_string(), "markdown");
    assert_eq!(FormatType::Json.to_string(), "json");
    assert_eq!(FormatType::Csv.to_string(), "csv");
}

#[test]
fn test_report_empty() {
    let report = Report::new("test_command", vec![]);
    assert_eq!(report.command(), "test_command");
    assert!(report.entries().is_empty());
}

#[test]
fn test_report_with_entries() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("echo", vec![summary]);

    assert_eq!(report.command(), "echo");
    assert_eq!(report.entries().len(), 1);
}

fn create_test_summary(durations_ms: Vec<u64>) -> BenchmarkSummary {
    let runs: Vec<RunResult> = durations_ms
        .iter()
        .enumerate()
        .map(|(i, &d)| RunResult::new(
            CommandResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: Some(0),
                duration: Duration::from_millis(d),
                timeout: false,
                success: true,
            },
            i,
            false,
        ))
        .collect();

    BenchmarkSummary::new(
        "test".to_string(),
        vec!["arg1".to_string()],
        runs.clone(),
        runs,
    )
}

#[test]
fn test_report_generate_text() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Text);
    assert!(output.contains("test"));
    assert!(output.contains("Runs: 3"));
}

#[test]
fn test_report_generate_markdown() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Markdown);
    assert!(output.contains("#"));
    assert!(output.contains("test"));
}

#[test]
fn test_report_generate_json() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Json);
    assert!(output.contains("{"));
    assert!(output.contains("command"));
    assert!(output.contains("test"));
}

#[test]
fn test_report_generate_csv() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Csv);
    assert!(output.contains("command"));
    assert!(output.contains("test"));
    assert!(output.contains(","));
}

#[test]
fn test_report_text_includes_statistics() {
    let summary = create_test_summary(vec![100, 150, 200, 250, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Text);
    assert!(output.contains("min") || output.contains("Min"));
    assert!(output.contains("max") || output.contains("Max"));
}

#[test]
fn test_report_json_valid() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Json);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
    assert!(parsed.is_ok());
}

#[test]
fn test_report_json_structure() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Json);
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["command"], "test");
    assert!(value["summaries"].is_array());
    assert_eq!(value["summaries"].as_array().unwrap().len(), 1);
}

#[test]
fn test_report_multiple_summaries() {
    let summary1 = create_test_summary(vec![100, 200]);
    let summary2 = create_test_summary(vec![300, 400]);
    let report = Report::new("test_suite", vec![summary1, summary2]);

    assert_eq!(report.entries().len(), 2);

    let json_output = report.generate(FormatType::Json);
    let value: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    assert_eq!(value["summaries"].as_array().unwrap().len(), 2);
}

#[test]
fn test_report_csv_headers() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Csv);
    let lines: Vec<&str> = output.lines().collect();

    assert!(!lines.is_empty());
    // First line should contain headers
    assert!(lines[0].contains("command"));
}

#[test]
fn test_report_csv_data_row_count() {
    let summary = create_test_summary(vec![100, 200, 300]);
    let report = Report::new("test", vec![summary]);

    let output = report.generate(FormatType::Csv);
    let lines: Vec<&str> = output.lines().collect();

    // At least 1 header + data
    assert!(lines.len() >= 2);
}

#[test]
fn test_report_text_formatting() {
    let summary = create_test_summary(vec![100, 200]);
    let report = Report::new("my_command", vec![summary]);

    let output = report.generate(FormatType::Text);
    assert!(output.contains("my_command"));
}

#[test]
fn test_report_from_sample() {
    let sample = Sample::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let stats = temci::run::stats::Statistics::from_sample(&sample);

    assert_eq!(stats.mean, Some(3.0));
    assert_eq!(stats.median, Some(3.0));
}
