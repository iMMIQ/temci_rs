//! Benchmark execution engine
//!
//! Provides a executor for running benchmarks with multiple executions
//! and collecting statistics.

use std::time::Duration;

use crate::run::runner::CommandRunner;

/// Configuration for running a benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub runs: usize,
    pub warmup_runs: usize,
    pub timeout: Option<Duration>,
    pub working_dir: Option<String>,
    pub env_vars: Vec<(String, String)>,
    pub max_concurrent: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            runs: 10,
            warmup_runs: 3,
            timeout: None,
            working_dir: None,
            env_vars: Vec::new(),
            max_concurrent: 1,
        }
    }
}

impl BenchmarkConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_runs(mut self, runs: usize) -> Self {
        self.runs = runs;
        self
    }

    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_runs = warmup;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
}

/// Result of a single benchmark run
#[derive(Debug, Clone)]
pub struct RunResult {
    pub result: crate::run::runner::CommandResult,
    pub run_number: usize,
    pub is_warmup: bool,
}

impl RunResult {
    pub fn new(result: crate::run::runner::CommandResult, run_number: usize, is_warmup: bool) -> Self {
        Self {
            result,
            run_number,
            is_warmup,
        }
    }

    pub fn duration(&self) -> Duration {
        self.result.duration
    }

    pub fn is_success(&self) -> bool {
        self.result.success
    }
}

/// Summary of a benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub all_runs: Vec<RunResult>,
    pub benchmark_runs: Vec<RunResult>,
    pub successful_runs: usize,
    pub failed_runs: usize,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub avg_duration: Option<Duration>,
    pub total_time: Duration,
    pub command: String,
    pub args: Vec<String>,
}

impl BenchmarkSummary {
    pub fn new(
        command: String,
        args: Vec<String>,
        all_runs: Vec<RunResult>,
        benchmark_runs: Vec<RunResult>,
    ) -> Self {
        let successful_runs = benchmark_runs.iter().filter(|r| r.is_success()).count();
        let failed_runs = benchmark_runs.len() - successful_runs;

        let durations: Vec<Duration> = benchmark_runs
            .iter()
            .filter_map(|r| if r.is_success() { Some(r.duration()) } else { None })
            .collect();

        let min_duration = durations.iter().copied().min();
        let max_duration = durations.iter().copied().max();
        let avg_duration = if !durations.is_empty() {
            let total: Duration = durations.iter().cloned().sum();
            Some(total / durations.len() as u32)
        } else {
            None
        };

        let total_time = all_runs
            .iter()
            .map(|r| r.result.duration)
            .sum::<Duration>();

        Self {
            all_runs,
            benchmark_runs,
            successful_runs,
            failed_runs,
            min_duration,
            max_duration,
            avg_duration,
            total_time,
            command,
            args,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.benchmark_runs.is_empty() {
            return 0.0;
        }
        self.successful_runs as f64 / self.benchmark_runs.len() as f64
    }
}

/// Benchmark executor for running benchmarks
pub struct BenchmarkExecutor {
    runner: CommandRunner,
    max_concurrent: usize,
}

impl BenchmarkExecutor {
    pub fn new(max_concurrent: usize) -> Self {
        let runner = CommandRunner::new();
        Self {
            runner,
            max_concurrent,
        }
    }

    pub fn default_executor() -> Self {
        Self::new(1)
    }

    /// Helper method to run a command using the runner
    async fn run_command(&mut self, command: &str, args: &[&str]) -> anyhow::Result<crate::run::runner::CommandResult> {
        use crate::run::runner::Runner;

        self.runner.run(command, args).await
            .map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))
    }

    /// Run a single benchmark with given configuration
    pub async fn run_benchmark(
        &mut self,
        command: &str,
        args: &[&str],
        config: &BenchmarkConfig,
    ) -> anyhow::Result<BenchmarkSummary> {
        let mut all_runs = Vec::new();
        let mut benchmark_runs = Vec::new();

        // Set working directory if specified
        if let Some(dir) = &config.working_dir {
            self.runner.set_working_dir(dir);
        }

        // Set timeout if specified
        if let Some(timeout) = config.timeout {
            self.runner.set_timeout(Some(timeout));
        }

        // Set environment variables
        for (key, value) in &config.env_vars {
            self.runner.set_env(key, value);
        }

        // Run warmup runs
        for i in 0..config.warmup_runs {
            let result = self.run_command(command, args).await?;
            all_runs.push(RunResult::new(result.clone(), i, true));
        }

        // Run actual benchmark runs
        let start_offset = config.warmup_runs;
        for i in 0..config.runs {
            let result = self.run_command(command, args).await?;
            let run_number = start_offset + i;
            all_runs.push(RunResult::new(result.clone(), run_number, false));
            benchmark_runs.push(RunResult::new(result, i, false));
        }

        Ok(BenchmarkSummary::new(
            command.to_string(),
            args.iter().map(|s| s.to_string()).collect(),
            all_runs,
            benchmark_runs,
        ))
    }
}
