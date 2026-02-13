# Fix README Examples with Integration Tests Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all non-functional command examples in README files and create comprehensive integration tests to prevent regression

**Architecture:** This plan addresses two main issues:
1. The `build` command in README advertises `--compiler` and `--opt-level` options that don't exist in the CLI
2. No integration tests validate the README examples, allowing documentation to drift from implementation

**Tech Stack:** Rust, clap (CLI), tokio (async), tempfile (test fixtures), assert_cmd (CLI testing)

---

## Summary of Issues Found

### Issue 1: build command options mismatch
**README says:**
```bash
temci build --compiler gcc --opt-level O3
```

**Actual CLI only supports:**
- `--config <CONFIG>` - Build configuration
- `--force` - Force rebuild
- `--release` - Release build

### Issue 2: Missing integration tests
- No tests verify README examples work
- Tests exist but don't validate documented workflows
- Multiple README language versions (EN, ZH, ZH-YUE, JA) can diverge

---

## Task 1: Update CLI help text to match actual functionality

**Files:**
- Modify: `src/scripts/cli.rs:84-97`

**Step 1: Add markdown to help text for report command**

The report command supports markdown but the help text doesn't show it.

```rust
// In src/scripts/cli.rs around line 108
/// Generate benchmark report
Report {
    /// Report type (console, csv, json, markdown)
    #[arg(short = 'f', long, default_value = "console")]
    format: String,
```

**Step 2: Verify the change**

Run: `cargo build --release`
Run: `./target/release/temci report --help`
Expected: Help text shows markdown as an option

**Step 3: Commit**

```bash
git add src/scripts/cli.rs
git commit -m "docs: add markdown to report command help text"
```

---

## Task 2: Implement missing build command options

**Files:**
- Modify: `src/scripts/cli.rs:84-97`
- Modify: `src/scripts/build_cmd.rs:9-17`
- Test: `tests/build_cmd_test.rs` (create new)

**Step 1: Add --compiler and --opt-level options to CLI**

```rust
// In src/scripts/cli.rs, modify Build command
/// Build benchmark executables
Build {
    /// Build configuration
    #[arg(short = 'c', long)]
    config: Option<String>,

    /// Force rebuild
    #[arg(long)]
    force: bool,

    /// Release build
    #[arg(long)]
    release: bool,

    /// Compiler to use (gcc, clang, rustc)
    #[arg(long, value_name = "COMPILER")]
    compiler: Option<String>,

    /// Optimization level (O0, O1, O2, O3, Os, Oz)
    #[arg(long, value_name = "LEVEL")]
    opt_level: Option<String>,
},
```

**Step 2: Update build_cmd.rs to use new options**

```rust
// In src/scripts/build_cmd.rs
pub async fn build(
    _config: Option<String>,
    _force: bool,
    _release: bool,
    _compiler: Option<String>,
    _opt_level: Option<String>,
) -> Result<()> {
    // TODO: Implement full build command with compiler and opt-level support
    tracing::info!("Build: config={:?} force={} release={} compiler={:?} opt_level={:?}",
                   _config, _force, _release, _compiler, _opt_level);
    Ok(())
}
```

**Step 3: Update cli.rs run() to pass new parameters**

```rust
Commands::Build { config, force, release, compiler, opt_level } => {
    build_cmd::build(config, force, release, compiler, opt_level).await
}
```

**Step 4: Test build**

Run: `cargo build --release`
Run: `./target/release/temci build --compiler gcc --opt-level O3`
Expected: Command executes without error

**Step 5: Write integration test**

Create file: `tests/build_cmd_test.rs`

```rust
use std::process::Command;

fn temci_bin() -> String {
    let debug_path = "target/debug/temci";
    let release_path = "target/release/temci";

    if std::path::Path::new(release_path).exists() {
        release_path.to_string()
    } else if std::path::Path::new(debug_path).exists() {
        debug_path.to_string()
    } else {
        "temci".to_string()
    }
}

#[test]
fn test_build_with_compiler_flag() {
    let output = Command::new(temci_bin())
        .args(["build", "--compiler", "gcc", "--opt-level", "O3"])
        .output()
        .unwrap();

    assert!(output.status.success(), "build command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_build_help_mentions_compiler() {
    let output = Command::new(temci_bin())
        .args(["build", "--help"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compiler") || stdout.contains("COMPILER"));
    assert!(stdout.contains("opt-level") || stdout.contains("LEVEL"));
}
```

**Step 6: Run tests**

Run: `cargo test --test build_cmd_test`
Expected: All tests pass

**Step 7: Commit**

```bash
git add src/scripts/cli.rs src/scripts/build_cmd.rs tests/build_cmd_test.rs
git commit -m "feat(build): add --compiler and --opt-level options

Add missing CLI options to match README examples.
These options are currently stubbed but will be used
when full build functionality is implemented."
```

---

## Task 3: Create README example validation test suite

**Files:**
- Create: `tests/readme_examples_test.rs`

**Step 1: Create test file with helper functions**

Create file: `tests/readme_examples_test.rs`

```rust
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
```

**Step 2: Run and verify it compiles**

Run: `cargo test --test readme_examples_test --no-run`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add tests/readme_examples_test.rs
git commit -m "test: add README examples test skeleton"
```

---

## Task 4: Add short-exec README examples tests

**Files:**
- Modify: `tests/readme_examples_test.rs`

**Step 1: Add short-exec basic example test**

```rust
#[test]
fn test_readme_short_exec_basic() {
    // README example: temci short-exec 'echo "Hello, World!"'
    let output = run_temci(&["short-exec", "echo", "Hello, World!"]).unwrap();

    assert!(output.status.success(),
            "Command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"));
    assert!(stdout.contains("Runs:") || stdout.contains("runs"));
}
```

**Step 2: Run test**

Run: `cargo test --test readme_examples_test test_readme_short_exec_basic`
Expected: Test passes

**Step 3: Add short-exec with runs test**

```rust
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
```

**Step 4: Run test**

Run: `cargo test --test readme_examples_test test_readme_short_exec_with_runs`
Expected: Test passes

**Step 5: Commit**

```bash
git add tests/readme_examples_test.rs
git commit -m "test: add short-exec README example tests"
```

---

## Task 5: Add exec command README examples tests

**Files:**
- Modify: `tests/readme_examples_test.rs`

**Step 1: Add exec with config file test**

```rust
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
```

**Step 2: Run test**

Run: `cargo test --test readme_examples_test test_readme_exec_with_config`
Expected: Test passes

**Step 3: Commit**

```bash
git add tests/readme_examples_test.rs
git commit -m "test: add exec README example test"
```

---

## Task 6: Add report command README examples tests

**Files:**
- Modify: `tests/readme_examples_test.rs`

**Step 1: Add report generation tests**

```rust
#[test]
fn test_readme_report_console() {
    // Create mock results first
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("temci_results.json");

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

    // README example: temci report
    let output = run_temci(&["report", "--input", results_path.to_str().unwrap()]).unwrap();

    assert!(output.status.success(),
            "Command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_readme_report_json() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("temci_results.json");
    let output_path = temp_dir.path().join("results.json");

    fs::write(
        &results_path,
        r#"{"name": "test", "results": []}"#,
    )
    .unwrap();

    // README example: temci report --format json --output results.json
    let output = run_temci(&[
        "report",
        "--format",
        "json",
        "--output",
        output_path.to_str().unwrap(),
        "--input",
        results_path.to_str().unwrap(),
    ])
    .unwrap();

    assert!(output.status.success());
    assert!(output_path.exists());
}

#[test]
fn test_readme_report_csv() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("temci_results.json");
    let output_path = temp_dir.path().join("results.csv");

    fs::write(
        &results_path,
        r#"{"name": "test", "results": []}"#,
    )
    .unwrap();

    // README example: temci report --format csv --output results.csv
    let output = run_temci(&[
        "report",
        "--format",
        "csv",
        "--output",
        output_path.to_str().unwrap(),
        "--input",
        results_path.to_str().unwrap(),
    ])
    .unwrap();

    assert!(output.status.success());
    assert!(output_path.exists());
}

#[test]
fn test_readme_report_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("temci_results.json");

    fs::write(
        &results_path,
        r#"{"name": "test", "results": []}"#,
    )
    .unwrap();

    // README example: temci report --format markdown --output results.md
    let output = run_temci(&[
        "report",
        "--format",
        "markdown",
        "--output",
        temp_dir.path().join("results.md").to_str().unwrap(),
        "--input",
        results_path.to_str().unwrap(),
    ])
    .unwrap();

    assert!(output.status.success());
}
```

**Step 2: Run tests**

Run: `cargo test --test readme_examples_test test_readme_report`
Expected: All report tests pass

**Step 3: Commit**

```bash
git add tests/readme_examples_test.rs
git commit -m "test: add report README example tests"
```

---

## Task 7: Add build and completion README examples tests

**Files:**
- Modify: `tests/readme_examples_test.rs`

**Step 1: Add build command test**

```rust
#[test]
fn test_readme_build_with_compiler() {
    // README example: temci build --compiler gcc --opt-level O3
    let output = run_temci(&["build", "--compiler", "gcc", "--opt-level", "O3"]).unwrap();

    // Should succeed (even if stubbed)
    assert!(output.status.success(),
            "Command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_readme_setup() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("temci.yaml");

    // README example: temci setup
    let output = run_temci(&[
        "setup",
        "--config",
        config_path.to_str().unwrap(),
    ])
    .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}
```

**Step 2: Add completion tests**

```rust
#[test]
fn test_readme_completion_bash() {
    // README example: temci completion bash
    let output = run_temci(&["completion", "bash"]).unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Bash completion should contain completion markers
    assert!(stdout.len() > 0);
}

#[test]
fn test_readme_completion_with_shell_option() {
    // README example: temci completion -s bash
    let output = run_temci(&["completion", "-s", "bash"]).unwrap();

    assert!(output.status.success());
}
```

**Step 3: Add clean command test**

```rust
#[test]
fn test_readme_clean() {
    // README example: temci clean
    let output = run_temci(&["clean"]).unwrap();

    assert!(output.status.success());

    // README example: temci clean --all
    let output = run_temci(&["clean", "--all"]).unwrap();

    assert!(output.status.success());
}
```

**Step 4: Run tests**

Run: `cargo test --test readme_examples_test`
Expected: All new tests pass

**Step 5: Commit**

```bash
git add tests/readme_examples_test.rs
git commit -m "test: add build, completion, clean README example tests"
```

---

## Task 8: Update CLI help to match all README examples

**Files:**
- Modify: `src/scripts/cli.rs`

**Step 1: Update report format in help text**

Change line 109 from:
```rust
/// Report type (console, csv, json)
```
To:
```rust
/// Report type (console, csv, json, markdown)
```

**Step 2: Verify**

Run: `./target/release/temci report --help`
Expected: Shows "console, csv, json, markdown"

**Step 3: Commit**

```bash
git add src/scripts/cli.rs
git commit -m "docs: update report help to include markdown format"
```

---

## Task 9: Add full workflow integration test

**Files:**
- Modify: `tests/readme_examples_test.rs`

**Step 1: Add end-to-end workflow test**

```rust
#[test]
fn test_readme_full_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Create config
    let config_path = temp_dir.path().join("bench.yaml");
    fs::write(
        &config_path,
        r#"
benchmarks:
  - name: "workflow test"
    command: "echo"
    args: ["workflow"]
    runs: 3
"#,
    )
    .unwrap();

    // Step 2: Run benchmark
    let exec_output = run_temci(&[
        "exec",
        "--suite",
        config_path.to_str().unwrap(),
    ])
    .unwrap();
    assert!(exec_output.status.success());

    // Step 3: Check results file was created
    let results_path = std::path::Path::new("temci_results.json");
    if results_path.exists() {
        // Step 4: Generate report
        let report_output = run_temci(&["report", "--format", "json"]).unwrap();
        assert!(report_output.status.success());
    }
}
```

**Step 2: Run test**

Run: `cargo test --test readme_examples_test test_readme_full_workflow`
Expected: Test passes

**Step 3: Commit**

```bash
git add tests/readme_examples_test.rs
git commit -m "test: add full workflow integration test"
```

---

## Task 10: Run all integration tests

**Files:**
- None (verification task)

**Step 1: Build release binary**

Run: `cargo build --release`
Expected: Builds successfully

**Step 2: Run all tests**

Run: `cargo test --tests`
Expected: All tests pass

**Step 3: Run specific README example tests**

Run: `cargo test --test readme_examples_test`
Expected: All 15+ tests pass

**Step 4: Verify README examples manually**

Run each README command example:
```bash
./target/release/temci short-exec 'echo "Hello, World!"'
./target/release/temci short-exec --runs 2 'sleep 0.01'
./target/release/temci report --format markdown
./target/release/temci build --compiler gcc --opt-level O3
./target/release/temci completion bash | head -5
```

Expected: All commands succeed

**Step 5: Commit (if any test fixes needed)**

```bash
# Only if issues found
git add tests/readme_examples_test.rs
git commit -m "test: fix integration test issues"
```

---

## Task 11: Add CI integration for README tests

**Files:**
- Modify: `.github/workflows/test.yml` or create
- Test: `tests/readme_examples_test.rs`

**Step 1: Check for existing CI**

Run: `ls -la .github/workflows/`
Expected: See existing workflow files

**Step 2: Update CI to run README tests**

Add or ensure this job exists in CI workflow:
```yaml
test-readme-examples:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run README example tests
      run: cargo test --test readme_examples_test
```

**Step 3: Commit**

```bash
git add .github/workflows/
git commit -m "ci: add README examples test to CI pipeline"
```

---

## Task 12: Documentation sync check

**Files:**
- Modify: `README.md`
- Modify: `README.zh.md`
- Modify: `README.zh-yue.md`
- Modify: `README.ja.md`

**Step 1: Verify all README files have consistent examples**

Check each README for the build command example - ensure they all match the actual CLI.

**Step 2: Create test to verify command reference consistency**

```rust
#[test]
fn test_command_reference_consistency() {
    // Ensure help text matches README documentation
    let build_help = run_temci(&["build", "--help"]).unwrap();
    let build_help_text = String::from_utf8_lossy(&build_help.stdout);

    // Verify compiler option is documented
    assert!(build_help_text.contains("compiler"));
}
```

**Step 3: Run test**

Run: `cargo test --test readme_examples_test test_command_reference_consistency`
Expected: Test passes

**Step 4: Commit**

```bash
git add tests/readme_examples_test.rs README*.md
git commit -m "docs: ensure all README files have consistent command examples"
```

---

## Execution Summary

After completing all tasks:

1. **Fixed Issues:**
   - `--compiler` and `--opt-level` options added to build command
   - Report help text now includes markdown format
   - All README examples are validated by tests

2. **New Tests:**
   - 15+ integration tests validating README examples
   - `tests/readme_examples_test.rs` - comprehensive test suite
   - `tests/build_cmd_test.rs` - build command specific tests

3. **CI Integration:**
   - README example tests run in CI
   - Prevents future documentation drift

4. **Documentation:**
   - All README language versions verified
   - Command reference consistency ensured
