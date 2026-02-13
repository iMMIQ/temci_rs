# Replace Custom Statistics with Established Crates Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace custom statistical implementations in `src/run/stats.rs` with the `statrs` crate and `stats-ci` for confidence intervals, reducing maintenance burden and leveraging well-tested libraries.

**Architecture:** This is a refactoring task. We'll replace internal implementations with crate equivalents while maintaining the exact same public API (`Sample` and `Statistics` structs). All existing tests must pass without modification.

**Tech Stack:**
- Rust 2021 edition
- `statrs` 0.16 (already in dependencies)
- `stats-ci` 0.1 (new dependency for confidence intervals)

---

## Task 1: Add stats-ci Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add stats-ci to Cargo.toml**

In the `[dependencies]` section, add the new dependency:

```toml
[dependencies]
# ... existing dependencies ...
statrs = "0.16"
stats-ci = "0.1"
# ... rest of dependencies ...
```

**Step 2: Verify Cargo.toml is valid**

Run: `cargo check --quiet`
Expected: No errors, dependency resolves successfully

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add stats-ci for confidence interval calculations"
```

---

## Task 2: Replace Mean, Variance, and Standard Deviation with statrs

**Files:**
- Modify: `src/run/stats.rs` (lines 177-247, the `Statistics::from_sample` method)

**Context:**
The current implementation manually calculates mean, variance, and std_dev:
```rust
let mean = sorted.iter().sum::<f64>() / n as f64;
let variance = if n > 1 {
    let sum_squared_diff: f64 = sorted
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum();
    Some(sum_squared_diff / (n - 1) as f64)
} else {
    Some(0.0)
};
let std_dev = variance.map(|v| v.sqrt());
```

**Step 1: Add statrs imports**

At the top of `src/run/stats.rs`, add the statrs imports:

```rust
use std::time::Duration;
use statrs::statistics::{Mean, Median, SortedData};
```

**Step 2: Run test to verify current state**

Run: `cargo test statistics_mean --quiet`
Expected: PASS (current implementation works)

**Step 3: Replace mean calculation in Statistics::from_sample**

Find the line:
```rust
let mean = sorted.iter().sum::<f64>() / n as f64;
```

Replace with:
```rust
let mean = Some(sorted.mean());
```

**Step 4: Run test to verify mean still works**

Run: `cargo test statistics_mean --quiet`
Expected: PASS

**Step 5: Replace variance and std_dev calculation**

Find the variance calculation block (around lines 204-213):
```rust
// Calculate variance (sample variance)
let variance = if n > 1 {
    let sum_squared_diff: f64 = sorted
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum();
    Some(sum_squared_diff / (n - 1) as f64)
} else {
    Some(0.0)
};

let std_dev = variance.map(|v| v.sqrt());
```

Replace with:
```rust
// statrs doesn't provide sample variance directly, use population variance for now
// For sample variance, we'll calculate manually (n-1 denominator)
let mean_val = sorted.mean();
let variance = if n > 1 {
    let sum_squared_diff: f64 = sorted
        .iter()
        .map(|&x| (x - mean_val).powi(2))
        .sum();
    Some(sum_squared_diff / (n - 1) as f64)
} else {
    Some(0.0)
};
let std_dev = variance.map(|v| v.sqrt());
```

**Step 6: Run all statistics tests**

Run: `cargo test stats_test --quiet`
Expected: All PASS

**Step 7: Commit**

```bash
git add src/run/stats.rs
git commit -m "refactor(stats): use statrs for mean calculation"
```

---

## Task 3: Replace Percentile Calculation with statrs

**Files:**
- Modify: `src/run/stats.rs` (lines 114-134, the `percentile` method)

**Context:**
The current implementation manually calculates percentiles using linear interpolation. statrs provides `SortedData::quantile` but uses a different method. We need to verify compatibility or keep our custom implementation.

**Step 1: Check if statrs quantile is compatible**

First, let's understand what statrs provides. Create a small test:

Run: `cargo doc --open --no-deps statrs` (optional, to view docs)

Or check the documentation: statrs uses linear interpolation similar to our implementation.

**Step 2: Add a test to verify statrs quantile matches our implementation**

In `tests/stats_test.rs`, temporarily add:

```rust
#[test]
fn test_statrs_percentile_compatibility() {
    use statrs::statistics::SortedData;
    let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let p25 = data.quantile(0.25);
    let p50 = data.quantile(0.5);
    let p75 = data.quantile(0.75);

    // Our implementation gives: p25=3.25, p50=5.5, p75=7.75
    assert!((p25 - 3.25).abs() < 0.01);
    assert!((p50 - 5.5).abs() < 0.01);
    assert!((p75 - 7.75).abs() < 0.01);
}
```

**Step 3: Run the compatibility test**

Run: `cargo test test_statrs_percentile_compatibility --quiet`
Expected: If PASS, proceed. If FAIL, our algorithm differs significantly.

**Step 4: Replace percentile implementation with statrs**

If the test passes, replace the `percentile` method in `src/run/stats.rs`:

Find:
```rust
/// Calculate percentile using linear interpolation
fn percentile(sorted_data: &[f64], percentile: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    if sorted_data.len() == 1 {
        return sorted_data[0];
    }

    let n = sorted_data.len() as f64;
    let pos = (percentile / 100.0) * (n - 1.0);
    let lower = pos.floor() as usize;
    let upper = pos.ceil() as usize;
    let fraction = pos - lower as f64;

    if upper >= sorted_data.len() {
        return sorted_data[sorted_data.len() - 1];
    }

    sorted_data[lower] + fraction * (sorted_data[upper] - sorted_data[lower])
}
```

Replace with:
```rust
/// Calculate percentile using statrs
fn percentile(sorted_data: &[f64], percentile: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    // statrs quantile takes 0-1 range, our percentile is 0-100
    sorted_data.quantile(percentile / 100.0)
}
```

**Step 5: Update imports to include quantile**

Ensure the import at top includes what's needed:
```rust
use statrs::statistics::SortedData;
```

**Step 6: Run all percentile tests**

Run: `cargo test test_statistics_percentiles --quiet`
Expected: All PASS

**Step 7: Remove the compatibility test**

Run: `git checkout tests/stats_test.rs` (or manually remove the test)

**Step 8: Commit**

```bash
git add src/run/stats.rs tests/stats_test.rs
git commit -m "refactor(stats): use statrs for percentile calculations"
```

---

## Task 4: Replace Confidence Interval with stats-ci

**Files:**
- Modify: `src/run/stats.rs` (lines 82-112, the `confidence_interval` method)

**Context:**
The current implementation uses hardcoded t-values for common confidence levels. `stats-ci` provides proper t-distribution calculations.

**Step 1: Add stats-ci import**

At the top of `src/run/stats.rs`, add:

```rust
use stats_ci::CI;
```

**Step 2: Check the current confidence_interval signature**

The method returns a custom `ConfidenceInterval` struct. We need to adapt stats-ci's output to match.

**Step 3: Replace confidence_interval implementation**

Find the `confidence_interval` method in `impl Sample`:

```rust
pub fn confidence_interval(&self, confidence: f64) -> ConfidenceInterval {
    if self.len() < 2 {
        return ConfidenceInterval {
            confidence,
            lower: 0.0,
            upper: 0.0,
        };
    }

    let stats = Statistics::from_sample(self);
    let mean = stats.mean.unwrap_or(0.0);
    let std_dev = stats.std_dev.unwrap_or(0.0);
    let n = self.len() as f64;

    // Approximate t-value for common confidence levels
    // For larger samples, this approaches the normal distribution
    let t_value = match confidence {
        c if (c - 0.90).abs() < 0.01 => 1.645,
        c if (c - 0.95).abs() < 0.01 => 1.960,
        c if (c - 0.99).abs() < 0.01 => 2.576,
        _ => 1.96, // Default to 95%
    };

    let margin = t_value * std_dev / n.sqrt();
    ConfidenceInterval {
        confidence,
        lower: mean - margin,
        upper: mean + margin,
    }
}
```

Replace with:

```rust
pub fn confidence_interval(&self, confidence: f64) -> ConfidenceInterval {
    if self.len() < 2 {
        return ConfidenceInterval {
            confidence,
            lower: 0.0,
            upper: 0.0,
        };
    }

    // Use stats-ci for proper t-distribution based confidence intervals
    let ci = stats_ci::confidence_interval(self.data(), confidence);
    ConfidenceInterval {
        confidence,
        lower: ci.lower(),
        upper: ci.upper(),
    }
}
```

**Step 4: Handle potential API differences**

If stats-ci's API differs, adapt accordingly. Check the actual API:

Run: `cargo doc --open --no-deps stats-ci` to verify the exact API.

**Step 5: Run confidence interval tests**

Run: `cargo test test_confidence_interval --quiet`
Expected: All PASS

**Step 6: Run all stats tests**

Run: `cargo test stats_test --quiet`
Expected: All PASS

**Step 7: Commit**

```bash
git add src/run/stats.rs
git commit -m "refactor(stats): use stats-ci for confidence interval calculations"
```

---

## Task 5: Verify All Tests Pass

**Files:**
- Test: `tests/stats_test.rs`

**Step 1: Run all stats tests**

Run: `cargo test --test stats_test --quiet`
Expected: All 24 tests PASS

**Step 2: Run full test suite**

Run: `cargo test --quiet`
Expected: All tests across the project PASS

**Step 3: Run clippy**

Run: `cargo clippy --quiet -- -D warnings`
Expected: No warnings

**Step 4: Build release binary**

Run: `cargo build --release --quiet`
Expected: Clean build

**Step 5: Run a quick integration test**

Run: `./target/release/temci short-exec sleep 0.1`
Expected: Command executes and shows timing output

**Step 6: Commit if any adjustments were needed**

If changes were made:
```bash
git add src/run/stats.rs tests/stats_test.rs
git commit -m "fix(stats): adjust for statrs/stats-ci compatibility"
```

---

## Task 6: Clean Up Unused Code

**Files:**
- Modify: `src/run/stats.rs`

**Step 1: Check for any now-unused helper functions**

Review `src/run/stats.rs` for any functions that are no longer used after refactoring.

The `sorted_data()` helper is still used by `detect_outliers` and potentially other methods, so it should remain.

**Step 2: Remove any truly dead code**

If any functions are completely unused, remove them.

**Step 3: Verify no dead code warnings**

Run: `cargo clippy --quiet -- -D warnings`
Expected: No `dead_code` warnings

**Step 4: Final commit if cleanup needed**

```bash
git add src/run/stats.rs
git commit -m "refactor(stats): remove unused code after refactoring"
```

---

## Task 7: Update Documentation

**Files:**
- Modify: `src/run/stats.rs` (module-level documentation)

**Step 1: Update module doc to reflect dependencies**

Find the module-level doc:
```rust
//! Statistical analysis for benchmark results
//!
//! Provides statistical functions for analyzing benchmark data including
//! descriptive statistics, percentiles, outlier detection, and confidence intervals.
```

Update to:
```rust
//! Statistical analysis for benchmark results
//!
//! Provides statistical functions for analyzing benchmark data including
//! descriptive statistics, percentiles, outlier detection, and confidence intervals.
//!
//! # Implementation
//!
//! This module uses the following crates for statistical computations:
//! - [`statrs`](https://docs.rs/statrs) for mean, median, and percentile calculations
//! - [`stats-ci`](https://docs.rs/stats-ci) for confidence interval calculations
//!
//! Custom implementations are retained for:
//! - Outlier detection using IQR method
//! - Sample management utilities
```

**Step 2: Run cargo doc to verify**

Run: `cargo doc --no-deps --quiet`
Expected: Documentation builds without errors

**Step 3: Commit documentation update**

```bash
git add src/run/stats.rs
git commit -m "docs(stats): document use of statrs and stats-ci crates"
```

---

## Verification Checklist

After completing all tasks, verify:

- [ ] All existing tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Clean release build (`cargo build --release`)
- [ ] Binary still works correctly (`temci short-exec <command>`)
- [ ] No new dependencies added beyond `stats-ci`
- [ ] No increase in binary size (or minimal increase)
- [ ] No performance regression in statistical calculations

## Rollback Plan

If issues arise:

1. Revert to the commit before refactoring: `git reset --hard <commit-hash>`
2. Each task is independently commit-able for easy selective revert
3. Tests at each task ensure issues are caught early

## Notes

- The public API (`Sample`, `Statistics`, `ConfidenceInterval`) remains unchanged
- All existing tests should pass without modification
- The `statrs` crate was already in dependencies but unused
- The `stats_ci` crate is new but lightweight
- Outlier detection (IQR method) is custom and retained as statrs doesn't provide it
