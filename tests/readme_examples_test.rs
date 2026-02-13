//! Integration tests for README examples
//!
//! This module tests all command examples documented in README.md to ensure
//! they work as documented and prevent documentation drift.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the temci binary
fn temci_bin() -> PathBuf {
    let release_path = PathBuf::from("target/release/temci");
    let debug_path = PathBuf::from("target/debug/temci");

    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        PathBuf::from("temci")
    }
}

/// Run temci with arguments and return output
fn run_temci(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new(&temci_bin()).args(args).output()
}

#[test]
fn test_readme_short_exec_basic() {
    // README example: temci short-exec 'echo "Hello, World!"'
    let output = run_temci(&["short-exec", "echo \"Hello, World!\""]).unwrap();

    assert!(output.status.success(),
            "Command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The command is echoed at the start (with escaped characters)
    assert!(stdout.contains("echo") && stdout.contains("Hello"));
    assert!(stdout.contains("Runs:") || stdout.contains("runs"));
}

#[test]
fn test_readme_short_exec_with_runs() {
    // README example: temci short-exec --runs 20 'sleep 1'
    // Using shorter sleep for test speed
    let output = run_temci(&["short-exec", "--runs", "3", "sleep", "0.01"]).unwrap();

    assert!(output.status.success(),
            "Command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Runs: 3") || stdout.contains("runs: 3"));
}

#[test]
fn test_readme_exec_with_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("temci.yaml");

    // README example config
    fs::write(
        &config_path,
        r#"
benchmarks:
  - name: "sleep test"
    command: "sleep"
    args: ["0.01"]
    runs: 3
    timeout: 5

  - name: "command test"
    command: "echo"
    args: ["test"]
    runs: 2
    show_output: true
"#,
    )
    .unwrap();

    // README example: temci exec (with suite option)
    let output = run_temci(&["exec", "--suite", config_path.to_str().unwrap()]).unwrap();

    assert!(output.status.success(),
            "Command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sleep test") || stdout.contains("command test"));
}
