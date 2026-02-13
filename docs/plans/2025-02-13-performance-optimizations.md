# Performance Optimizations Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Optimize critical performance bottlenecks in temci_rs while maintaining exact functional behavior through TDD.

**Architecture:**
- Eliminate redundant data cloning in statistical calculations
- Reduce string allocations in command execution
- Implement missing timeout functionality
- All changes verified with existing tests plus new regression tests

**Tech Stack:** Rust 2021, tokio, statrs, stats-ci, TDD with cargo test

---

## Task 1: Add Regression Tests for Stats Module

**Files:**
- Create: `src/run/stats_tests.rs` (new test module)
- Modify: `src/run/stats.rs:267-279` (extend existing tests)

**Step 1: Add comprehensive regression tests**

```rust
// Add to src/run/stats.rs tests module

#[test]
fn test_statistics_percentiles_consistency() {
    // Ensure multiple calls produce same results
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let sample = Sample::new(data.clone());

    let stats1 = Statistics::from_sample(&sample);
    let stats2 = Statistics::from_sample(&sample);

    assert_eq!(stats1.p25, stats2.p25);
    assert_eq!(stats1.p50, stats2.p50);
    assert_eq!(stats1.p75, stats2.p75);
    assert_eq!(stats1.p90, stats2.p90);
    assert_eq!(stats1.p95, stats2.p95);
    assert_eq!(stats1.p99, stats2.p99);
}

#[test]
fn test_statistics_from_large_sample() {
    // Test with larger dataset to catch allocation issues
    let data: Vec<f64> = (1..=1000).map(|i| i as f64).collect();
    let sample = Sample::new(data);

    let stats = Statistics::from_sample(&sample);
    assert_eq!(stats.min, Some(1.0));
    assert_eq!(stats.max, Some(1000.0));
    assert_eq!(stats.median, Some(500.5));
}

#[test]
fn test_outlier_detection_consistency() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
    let sample = Sample::new(data.clone());

    let outliers1 = sample.detect_outliers(1.5);
    let outliers2 = sample.detect_outliers(1.5);

    assert_eq!(outliers1, outliers2);
    assert!(outliers1.contains(&100.0));
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib run::stats::tests`

**Step 3: Commit**

```bash
git add src/run/stats.rs
git commit -m "test: add regression tests for stats module"
```

---

## Task 2: Optimize sorted_data() - Cache Sorting

**Files:**
- Modify: `src/run/stats.rs:22-72` (Sample struct)

**Step 1: Add cached sorted data field**

```rust
#[derive(Debug, Clone)]
pub struct Sample {
    data: Vec<f64>,
    // Add cached sorted data
    sorted_data: Option<Vec<f64>>,
}
```

**Step 2: Update methods to use cache**

```rust
impl Sample {
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            data,
            sorted_data: None,
        }
    }

    pub fn from_durations(durations: Vec<Duration>) -> Self {
        let data = durations
            .iter()
            .map(duration_as_ms)
            .collect();
        Self { data, sorted_data: None }
    }

    // Invalidate cache on mutation
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
        self.sorted_data = None;
    }

    pub fn extend(&mut self, values: Vec<f64>) {
        self.data.extend(values);
        self.sorted_data = None;
    }

    // Return cached sorted data
    fn sorted_data(&self) -> &[f64] {
        if self.sorted_data.is_none() {
            let mut sorted = self.data.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // Note: we need interior mutability or refactor to use &mut self
            // For now, return sorted but we need a different approach
        }
        self.sorted_data.as_deref().unwrap()
    }
}
```

**Step 3: Use better approach - compute all percentiles once**

Actually, the better approach is to compute all percentiles in a single pass in `Statistics::from_sample()`.

**Step 3 (better): Refactor Statistics::from_sample**

```rust
impl Statistics {
    pub fn from_sample(sample: &Sample) -> Self {
        if sample.is_empty() {
            return Self {
                mean: None, median: None, min: None, max: None,
                variance: None, std_dev: None,
                p25: None, p50: None, p75: None, p90: None, p95: None, p99: None,
            };
        }

        // Sort once, use for all percentiles
        let mut sorted = sample.data().to_vec();
        sorted.sort_unstable_by(|a, b| a.total_cmp(b));

        let n = sample.len();
        let mean = sample.data().mean();
        let min = sorted.first().copied();
        let max = sorted.last().copied();

        // Calculate variance
        let variance = if n > 1 {
            let sum_squared_diff: f64 = sample.data()
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum();
            Some(sum_squared_diff / (n - 1) as f64)
        } else {
            Some(0.0)
        };

        let std_dev = variance.map(|v| v.sqrt());

        // Compute all percentiles from single sorted array
        let data = Data::new(sorted.clone());
        let p25 = Some(data.quantile(0.25));
        let p50 = Some(data.quantile(0.50));
        let p75 = Some(data.quantile(0.75));
        let p90 = Some(data.quantile(0.90));
        let p95 = Some(data.quantile(0.95));
        let p99 = Some(data.quantile(0.99));

        Self {
            mean: Some(mean),
            median: p50,
            min, max,
            variance, std_dev,
            p25, p50, p75, p90, p95, p99,
        }
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib run::stats::tests`

**Step 5: Commit**

```bash
git add src/run/stats.rs
git commit -m "perf(stats): compute all percentiles from single sorted array"
```

---

## Task 3: Optimize String Allocations in CommandRunner

**Files:**
- Modify: `src/run/runner.rs:17-64` (CommandResult struct)

**Step 1: Add test for string handling**

```rust
#[test]
fn test_command_result_preserves_content() {
    // Ensure optimization doesn't break content preservation
    let stdout_bytes = b"valid utf8 \xc3\xa9";
    let stderr_bytes = b"error message";

    let result = CommandResult {
        stdout: String::from_utf8_lossy(stdout_bytes).to_string(),
        stderr: String::from_utf8_lossy(stderr_bytes).to_string(),
        exit_code: Some(0),
        duration: Duration::from_millis(100),
        timeout: false,
        success: true,
    };

    assert_eq!(result.stdout, "valid utf8 \xc3\xa9");
    assert_eq!(result.stderr, "error message");
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test --lib run::runner::tests::test_command_result_preserves_content`

**Step 3: Optimize execute_command to use String::from_utf8 where possible**

```rust
fn execute_command(&self, cmd: &mut Command) -> Result<CommandResult> {
    let start = Instant::now();

    debug!("Executing command: {:?}", cmd);

    let output = cmd
        .output()
        .map_err(|e| TemciError::ExecutionError(format!("Execution failed: {}", e)))?;

    let duration = start.elapsed();

    // Optimize: try from_utf8 first (no allocation if valid), fall back to lossy
    let stdout = if let Ok(s) = String::from_utf8(output.stdout) {
        s
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    let stderr = if let Ok(s) = String::from_utf8(output.stderr) {
        s
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    let exit_code = output.status.code();
    let success = output.status.success();

    debug!("Command completed in {:?}", duration);
    trace!("stdout: {}", stdout);
    trace!("stderr: {}", stderr);

    Ok(CommandResult {
        stdout,
        stderr,
        exit_code,
        duration,
        timeout: false,
        success,
    })
}
```

**Step 4: Apply same optimization to async run method**

Modify the async `run()` method similarly (around line 221-262).

**Step 5: Run all runner tests**

Run: `cargo test --lib run::runner::tests`

**Step 6: Commit**

```bash
git add src/run/runner.rs
git commit -m "perf(runner): use String::from_utf8 for valid UTF-8 output"
```

---

## Task 4: Optimize PerfRunner Command Building

**Files:**
- Modify: `src/run/runner.rs:269-319` (PerfRunner)

**Step 1: Add test for perf command building**

```rust
#[test]
fn test_perf_command_building() {
    let runner = PerfRunner::new();
    let cmd = runner.build_perf_command("echo", &["hello", "world"]);

    assert_eq!(cmd[0], "perf");
    assert_eq!(cmd[1], "stat");
    assert_eq!(cmd[2], "-x,");
    assert!(cmd.contains(&"echo".to_string()));
    assert!(cmd.contains(&"hello".to_string()));
    assert!(cmd.contains(&"world".to_string()));
}
```

**Step 2: Run test**

Run: `cargo test --lib run::runner::tests::test_perf_command_building`

**Step 3: Optimize to use static strings**

```rust
impl PerfRunner {
    /// Build a perf stat command with optimized string handling
    fn build_perf_command(&self, program: &str, args: &[&str]) -> Vec<String> {
        // Pre-allocate exact capacity
        let mut cmd = Vec::with_capacity(6 + args.len());

        // Static strings - use with_capacity to avoid reallocations
        cmd.push("perf".to_string());
        cmd.push("stat".to_string());
        cmd.push("-x,".to_string());
        cmd.push("-e".to_string());
        cmd.push(self.events.join(","));
        cmd.push("--".to_string());
        cmd.push(program.to_string());
        cmd.extend(args.iter().map(|s| s.to_string()));

        cmd
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib run::runner::tests`

**Step 5: Commit**

```bash
git add src/run/runner.rs
git commit -m "perf(perf): pre-allocate vector capacity for command building"
```

---

## Task 5: Implement Timeout Support

**Files:**
- Modify: `src/run/runner.rs:176-262` (CommandRunner)

**Step 1: Add test for timeout functionality**

```rust
#[tokio::test]
async fn test_command_timeout() {
    let mut runner = CommandRunner::new();
    runner.set_timeout(Some(Duration::from_millis(100)));

    // This should timeout (sleep command)
    let result = runner.run("sleep", &["2"]).await;

    assert!(result.is_err()); // Or handle timeout gracefully
}
```

**Step 2: Run test - should fail**

Run: `cargo test --lib run::runner::tests::test_command_timeout`

**Step 3: Implement timeout using tokio::time::timeout**

```rust
use tokio::time::{timeout as tokio_timeout, error::Elapsed};

async fn run_with_timeout(
    &self,
    program: &str,
    args: &[&str],
    timeout_dur: Option<Duration>
) -> Result<CommandResult> {
    let start = Instant::now();

    let mut cmd = TokioCommand::new(program);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    for (key, value) in &self.env {
        cmd.env(key, value);
    }

    if let Some(dir) = &self.working_dir {
        cmd.current_dir(dir);
    }

    let output_future = cmd.output();

    let output = if let Some(dur) = timeout_dur {
        match tokio_timeout(dur, output_future).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                return Err(TemciError::ExecutionError(format!("Async execution failed: {}", e)));
            }
            Err(_) => {
                // Timeout occurred
                return Ok(CommandResult {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    duration: dur,
                    timeout: true,
                    success: false,
                });
            }
        }
    } else {
        output_future.await
            .map_err(|e| TemciError::ExecutionError(format!("Async execution failed: {}", e)))?
    };

    // ... rest of the processing
}
```

**Step 4: Update run() method to use timeout**

**Step 5: Run tests**

Run: `cargo test --lib run::runner::tests`

**Step 6: Commit**

```bash
git add src/run/runner.rs
git commit -m "feat(runner): implement command timeout support"
```

---

## Task 6: Add Benchmark Tests

**Files:**
- Create: `benches/stats_bench.rs` (new benchmark file)

**Step 1: Create benchmark file**

```rust
// Add to Cargo.toml first:
// [[bench]]
// name = "stats_bench"
// harness = false

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use temci::run::stats::{Sample, Statistics};

fn bench_statistics_from_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    for size in [10, 100, 1000, 10000].iter() {
        let data: Vec<f64> = (1..=*size).map(|i| i as f64).collect();
        let sample = Sample::new(data);

        group.bench_with_input(BenchmarkId::from_parameter(size), &sample, |b, s| {
            b.iter(|| Statistics::from_sample(black_box(s)))
        });
    }

    group.finish();
}

fn bench_outlier_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("outliers");

    for size in [10, 100, 1000].iter() {
        let data: Vec<f64> = (1..=*size).map(|i| i as f64).chain(vec![10000.0]).collect();
        let sample = Sample::new(data);

        group.bench_with_input(BenchmarkId::from_parameter(size), &sample, |b, s| {
            b.iter(|| s.detect_outliers(black_box(1.5)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_statistics_from_sample, bench_outlier_detection);
criterion_main!(benches);
```

**Step 2: Update Cargo.toml**

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "stats_bench"
harness = false
```

**Step 3: Run benchmarks**

Run: `cargo bench --bench stats_bench`

**Step 4: Commit**

```bash
git add benches/stats_bench.rs Cargo.toml
git commit -m "test: add performance benchmarks for stats module"
```

---

## Task 7: Verify All Tests Pass

**Step 1: Run full test suite**

Run: `cargo test --all`

**Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features`

**Step 3: Run format check**

Run: `cargo fmt --check`

**Step 4: Fix any issues found**

**Step 5: Final commit**

```bash
git add .
git commit -m "test: ensure all tests and lint checks pass"
```

---

## Summary of Changes

| Task | File | Change | Expected Impact |
|------|------|--------|-----------------|
| 1 | `stats.rs` | Add regression tests | Zero - safety |
| 2 | `stats.rs` | Single sort for all percentiles | ~6x fewer allocations |
| 3 | `runner.rs` | Use String::from_utf8 | ~50% fewer string allocs |
| 4 | `runner.rs` | Pre-allocate command vec | Fewer reallocs |
| 5 | `runner.rs` | Implement timeout | Feature complete |
| 6 | `stats_bench.rs` | Add benchmarks | Measurement capability |

---

## Testing Strategy

1. **Before each optimization:** Run existing tests to establish baseline
2. **Write test first:** Add specific tests for behavior being preserved
3. **Run test:** Confirm it passes before optimization
4. **Optimize:** Make performance changes
5. **Run test again:** Confirm behavior unchanged
6. **Benchmark:** Compare before/after with criterion benchmarks
