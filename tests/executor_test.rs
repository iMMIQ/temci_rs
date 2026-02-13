use std::time::Duration;
use temci::run::executor::{BenchmarkConfig, BenchmarkExecutor, RunResult};

#[tokio::test]
async fn test_benchmark_config_default() {
    let config = BenchmarkConfig::default();
    assert_eq!(config.runs, 10);
    assert_eq!(config.warmup_runs, 3);
    assert_eq!(config.timeout, None);
    assert_eq!(config.working_dir, None);
    assert!(config.env_vars.is_empty());
    assert_eq!(config.max_concurrent, 1);
}

#[tokio::test]
async fn test_benchmark_config_builder() {
    let config = BenchmarkConfig::new()
        .with_runs(5)
        .with_warmup(2)
        .with_timeout(Duration::from_secs(30))
        .with_working_dir("/tmp")
        .with_env("KEY", "value")
        .with_max_concurrent(4);

    assert_eq!(config.runs, 5);
    assert_eq!(config.warmup_runs, 2);
    assert_eq!(config.timeout, Some(Duration::from_secs(30)));
    assert_eq!(config.working_dir, Some("/tmp".to_string()));
    assert_eq!(config.env_vars, vec![("KEY".to_string(), "value".to_string())]);
    assert_eq!(config.max_concurrent, 4);
}

#[tokio::test]
async fn test_benchmark_executor_creation() {
    let _executor = BenchmarkExecutor::new(2);

    let _executor = BenchmarkExecutor::default_executor();
}

#[tokio::test]
async fn test_run_result_creation() {
    use temci::run::runner::CommandResult;
    use std::time::Duration;

    let cmd_result = CommandResult {
        stdout: "output".to_string(),
        stderr: String::new(),
        exit_code: Some(0),
        duration: Duration::from_millis(100),
        timeout: false,
        success: true,
    };

    let result = RunResult::new(cmd_result.clone(), 0, false);
    assert_eq!(result.run_number, 0);
    assert!(!result.is_warmup);
    assert_eq!(result.duration(), Duration::from_millis(100));
    assert!(result.is_success());
}

#[tokio::test]
async fn test_run_result_warmup() {
    use temci::run::runner::CommandResult;
    use std::time::Duration;

    let cmd_result = CommandResult {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: Some(0),
        duration: Duration::from_millis(50),
        timeout: false,
        success: true,
    };

    let result = RunResult::new(cmd_result, 0, true);
    assert!(result.is_warmup);
    assert_eq!(result.duration(), Duration::from_millis(50));
}

#[tokio::test]
async fn test_benchmark_summary_success_rate() {
    use temci::run::runner::CommandResult;
    use std::time::Duration;

    let success_run = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration: Duration::from_millis(100),
            timeout: false,
            success: true,
        },
        0,
        false,
    );

    let failed_run = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: "error".to_string(),
            exit_code: Some(1),
            duration: Duration::from_millis(50),
            timeout: false,
            success: false,
        },
        1,
        false,
    );

    let summary = temci::run::executor::BenchmarkSummary::new(
        "echo".to_string(),
        vec!["hello".to_string()],
        vec![success_run.clone(), failed_run.clone()],
        vec![success_run, failed_run],
    );

    assert_eq!(summary.successful_runs, 1);
    assert_eq!(summary.failed_runs, 1);
    assert!((summary.success_rate() - 0.5).abs() < 0.001);
}

#[tokio::test]
async fn test_benchmark_summary_duration_stats() {
    use temci::run::runner::CommandResult;
    use std::time::Duration;

    let run1 = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration: Duration::from_millis(100),
            timeout: false,
            success: true,
        },
        0,
        false,
    );

    let run2 = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration: Duration::from_millis(200),
            timeout: false,
            success: true,
        },
        1,
        false,
    );

    let run3 = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration: Duration::from_millis(300),
            timeout: false,
            success: true,
        },
        2,
        false,
    );

    let summary = temci::run::executor::BenchmarkSummary::new(
        "sleep".to_string(),
        vec!["0.1".to_string()],
        vec![run1.clone(), run2.clone(), run3.clone()],
        vec![run1, run2, run3],
    );

    assert_eq!(summary.min_duration, Some(Duration::from_millis(100)));
    assert_eq!(summary.max_duration, Some(Duration::from_millis(300)));
    assert_eq!(summary.avg_duration, Some(Duration::from_millis(200)));
    assert_eq!(summary.total_time, Duration::from_millis(600));
}

#[tokio::test]
async fn test_benchmark_summary_empty() {
    let summary = temci::run::executor::BenchmarkSummary::new(
        "test".to_string(),
        vec![],
        vec![],
        vec![],
    );

    assert_eq!(summary.successful_runs, 0);
    assert_eq!(summary.failed_runs, 0);
    assert_eq!(summary.success_rate(), 0.0);
    assert_eq!(summary.min_duration, None);
    assert_eq!(summary.max_duration, None);
    assert_eq!(summary.avg_duration, None);
}

#[tokio::test]
async fn test_benchmark_summary_total_time_includes_warmup() {
    use temci::run::runner::CommandResult;
    use std::time::Duration;

    let warmup = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration: Duration::from_millis(50),
            timeout: false,
            success: true,
        },
        0,
        true,
    );

    let benchmark = RunResult::new(
        CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration: Duration::from_millis(100),
            timeout: false,
            success: true,
        },
        1,
        false,
    );

    let summary = temci::run::executor::BenchmarkSummary::new(
        "test".to_string(),
        vec![],
        vec![warmup, benchmark.clone()],
        vec![benchmark],
    );

    // Total time should include warmup
    assert_eq!(summary.total_time, Duration::from_millis(150));
    // But only benchmark runs count for stats
    assert_eq!(summary.benchmark_runs.len(), 1);
    assert_eq!(summary.all_runs.len(), 2);
}

#[tokio::test]
async fn test_benchmark_executor_run_simple_command() {
    let mut executor = BenchmarkExecutor::default_executor();
    let config = BenchmarkConfig::new()
        .with_runs(2)
        .with_warmup(1);

    let summary = executor.run_benchmark("echo", &["hello"], &config).await.unwrap();

    assert_eq!(summary.command, "echo");
    assert_eq!(summary.args, vec!["hello"]);
    assert_eq!(summary.benchmark_runs.len(), 2);
    assert_eq!(summary.all_runs.len(), 3); // 1 warmup + 2 benchmark
    assert_eq!(summary.successful_runs, 2);
    assert_eq!(summary.failed_runs, 0);
    assert!((summary.success_rate() - 1.0).abs() < 0.001);
}

#[tokio::test]
async fn test_benchmark_executor_with_warmup() {
    let mut executor = BenchmarkExecutor::default_executor();
    let config = BenchmarkConfig::new()
        .with_runs(1)
        .with_warmup(2);

    let summary = executor.run_benchmark("true", &[], &config).await.unwrap();

    // Should have 2 warmup runs and 1 benchmark run
    assert_eq!(summary.all_runs.len(), 3);
    assert_eq!(summary.benchmark_runs.len(), 1);

    // Check that warmup runs are marked correctly
    assert!(summary.all_runs[0].is_warmup);
    assert!(summary.all_runs[1].is_warmup);
    assert!(!summary.all_runs[2].is_warmup);
}
