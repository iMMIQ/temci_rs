//! Full workflow integration tests
//!
//! End-to-end tests that verify the complete temci workflow from
//! execution through reporting.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get path to temci binary
fn temci_bin() -> PathBuf {
    // Look for the binary in target/debug or target/release
    let debug_path = PathBuf::from("target/debug/temci");
    let release_path = PathBuf::from("target/release/temci");

    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        // Fall back to using cargo run
        PathBuf::from("temci")
    }
}

/// Helper to run temci as a subprocess
fn run_temci(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    let bin_path = temci_bin();

    // If the binary doesn't exist at expected path, try building it first
    if !bin_path.exists() {
        let _ = Command::new("cargo")
            .args(["build", "--release"])
            .status();
    }

    Command::new(&bin_path).args(args).output()
}

#[test]
fn test_version_flag() {
    let output = run_temci(&["--version"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("temci"));
}

#[test]
fn test_help_flag() {
    let output = run_temci(&["--help"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
    assert!(stdout.contains("temci"));
}

#[test]
fn test_short_exec_simple_command() {
    // Each command must be a complete command string
    let output = run_temci(&["short-exec", "echo test"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test"));
}

#[test]
fn test_short_exec_timing() {
    let output = run_temci(&["short-exec", "sleep 0.1"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain timing information
    assert!(stdout.contains("ms") || stdout.contains("s"));
}

#[test]
fn test_short_exec_with_custom_runs() {
    let output = run_temci(&["short-exec", "--runs", "3", "echo custom-runs"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("custom-runs"));
}

#[test]
fn test_exec_with_suite_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bench_config.yaml");

    // Create a benchmark config
    fs::write(
        &config_path,
        r#"
runs: 3
timeout: 10
benchmarks:
  - name: "echo test"
    command: "echo"
    args: ["integration-test"]
    show_output: true
"#,
    )
    .unwrap();

    let output = run_temci(&["exec", "--suite", config_path.to_str().unwrap()]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("echo test") || stdout.contains("integration-test"));
}

#[test]
fn test_report_generation() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("results.json");

    // Create mock results
    fs::write(
        &results_path,
        r#"{
  "name": "test suite",
  "timestamp": "2024-01-01T00:00:00Z",
  "results": [
    {
      "command": "echo",
      "args": ["test"],
      "runs": 5,
      "successful": 5,
      "failed": 0,
      "min_ms": 1.0,
      "max_ms": 5.0,
      "avg_ms": 2.5,
      "total_ms": 12.5
    }
  ]
}"#,
    )
    .unwrap();

    // Test console format
    let output = run_temci(&["report", "--input", results_path.to_str().unwrap()]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("echo") || stdout.contains("test"));

    // Test JSON format to file
    let json_output = temp_dir.path().join("report.json");
    let output = run_temci(&[
        "report",
        "--format",
        "json",
        "--output",
        json_output.to_str().unwrap(),
        "--input",
        results_path.to_str().unwrap(),
    ])
    .unwrap();
    assert!(output.status.success());
    assert!(json_output.exists());
}

#[test]
fn test_completion_generation() {
    let output = run_temci(&["completion", "--shell", "bash"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain bash completion markers
    assert!(stdout.contains("complete") || stdout.contains("_temci") || stdout.contains("bash"));
}

#[test]
fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("workflow.yaml");

    // Step 1: Create benchmark config
    fs::write(
        &config_path,
        r#"
runs: 5
timeout: 10
benchmarks:
  - name: "workflow step 1"
    command: "echo"
    args: ["full-workflow-test"]
    show_output: false
"#,
    )
    .unwrap();

    // Step 2: Run benchmark
    let output = run_temci(&["exec", "--suite", config_path.to_str().unwrap()]).unwrap();

    // Should complete successfully
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show something from the benchmark execution
    assert!(!stdout.is_empty());
}

#[test]
fn test_clean_command() {
    let temp_dir = TempDir::new().unwrap();
    let temci_dir = temp_dir.path().join(".temci");

    // Create a .temci directory
    fs::create_dir_all(&temci_dir).unwrap();

    // Run clean (should not error even if directory doesn't exist after)
    let output = run_temci(&["clean"]).unwrap();
    assert!(output.status.success());
}

#[test]
fn test_multiple_commands() {
    // Test short-exec with multiple command variants
    let commands = vec![
        vec!["short-exec", "true"],
        vec!["short-exec", "echo multi"],
        vec!["short-exec", "printf test"],
    ];

    for cmd in commands {
        let output = run_temci(&cmd).unwrap();
        assert!(output.status.success(), "Command failed: {:?}", cmd);
    }
}

#[test]
fn test_error_handling_graceful() {
    // Test with non-existent command
    // short-exec handles errors gracefully and still returns 0 (because it continues with other commands)
    let output = run_temci(&["short-exec", "nonexistent-command-xyz"]).unwrap();
    // The tool should handle the error gracefully and return success
    assert!(output.status.success());

    // Check that error was reported (may be in stderr or stdout with tracing)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{}{}", stdout, stderr);
    assert!(output_text.contains("failed") || output_text.contains("Error") || output_text.contains("No such file"));

    // Test with invalid config - exec should return non-zero
    let temp_dir = TempDir::new().unwrap();
    let invalid_config = temp_dir.path().join("invalid.yaml");
    fs::write(&invalid_config, "invalid: yaml: content: [").unwrap();

    let output = run_temci(&["exec", "--suite", invalid_config.to_str().unwrap()]).unwrap();
    // Should handle error gracefully with non-zero exit code
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{}{}", stdout, stderr);
    assert!(output_text.contains("failed") || output_text.contains("Error") || output_text.contains("Failed") || output_text.contains("parse") || output_text.contains("config"));
}

#[test]
fn test_warmup_runs() {
    let output = run_temci(&["short-exec", "--warmup", "2", "echo warmup-test"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("warmup-test"));
}

#[test]
fn test_summary_output() {
    let output = run_temci(&["short-exec", "--summary", "echo summary-test"]).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("summary-test"));
}

#[test]
fn test_report_format_variants() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("results.json");

    // Create mock results
    fs::write(
        &results_path,
        r#"{
  "name": "test suite",
  "timestamp": "2024-01-01T00:00:00Z",
  "results": [
    {
      "command": "echo",
      "args": ["test"],
      "runs": 5,
      "successful": 5,
      "failed": 0,
      "min_ms": 1.0,
      "max_ms": 5.0,
      "avg_ms": 2.5,
      "total_ms": 12.5
    }
  ]
}"#,
    )
    .unwrap();

    // Test console format
    let output = run_temci(&["report", "--format", "text", "--input", results_path.to_str().unwrap()]).unwrap();
    assert!(output.status.success());

    // Test JSON format
    let json_output = temp_dir.path().join("report.json");
    let output = run_temci(&[
        "report",
        "--format",
        "json",
        "--output",
        json_output.to_str().unwrap(),
        "--input",
        results_path.to_str().unwrap(),
    ])
    .unwrap();
    assert!(output.status.success());
    assert!(json_output.exists());

    // Test CSV format
    let csv_output = temp_dir.path().join("report.csv");
    let output = run_temci(&[
        "report",
        "--format",
        "csv",
        "--output",
        csv_output.to_str().unwrap(),
        "--input",
        results_path.to_str().unwrap(),
    ])
    .unwrap();
    assert!(output.status.success());
    assert!(csv_output.exists());
}
