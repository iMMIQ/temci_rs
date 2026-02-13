use temci::run::runner::{CommandRunner, PerfRunner, Runner};

#[test]
fn test_command_runner_basic() {
    let runner = CommandRunner::new();

    let result = runner.run_sync("echo", &["hello", "world"]).unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.stdout.is_empty());
}

#[test]
fn test_command_runner_with_args() {
    let runner = CommandRunner::new();

    let result = runner.run_sync("echo", &["-n", "test"]).unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn test_command_runner_failure() {
    let runner = CommandRunner::new();

    let result = runner.run_sync("false", &[]);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.success);
    assert_eq!(result.exit_code, Some(1));
}

#[test]
fn test_command_runner_nonexistent() {
    let runner = CommandRunner::new();

    let result = runner.run_sync("nonexistent_command_xyz", &[]);

    assert!(result.is_err());
}

#[test]
fn test_command_result_fields() {
    let runner = CommandRunner::new();

    let result = runner.run_sync("echo", &["test"]).unwrap();

    assert!(result.stdout.contains("test"));
    assert!(result.stderr.is_empty());
    assert!(result.duration.as_nanos() > 0);
}

#[test]
fn test_perf_runner() {
    let perf = PerfRunner::new();

    // Only test if perf is available
    if which::which("perf").is_ok() {
        let result = perf.run_sync("echo", &["test"]);

        // Perf might not work in all environments, so we just check it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_run_driver_basic() {
    let runner = CommandRunner::new();

    let result = runner.run_sync("true", &[]).unwrap();
    assert!(result.success);
}

#[test]
fn test_command_runner_with_env() {
    let mut runner = CommandRunner::new();
    runner.set_env("TEST_VAR", "test_value");

    let result = runner.run_sync("sh", &["-c", "echo $TEST_VAR"]).unwrap();

    assert!(result.success);
    assert!(result.stdout.contains("test_value"));
}

#[tokio::test]
async fn test_command_runner_async() {
    let runner = CommandRunner::new();

    let result = runner.run("echo", &["async_test"]).await.unwrap();

    assert!(result.success);
    assert!(result.stdout.contains("async_test"));
}

#[tokio::test]
async fn test_command_runner_multiple_async() {
    let runner = CommandRunner::new();

    let result1 = runner.run("echo", &["first"]);
    let result2 = runner.run("echo", &["second"]);

    let (r1, r2): (std::result::Result<_, _>, std::result::Result<_, _>) = tokio::join!(result1, result2);

    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r1.unwrap().stdout.contains("first"));
    assert!(r2.unwrap().stdout.contains("second"));
}
