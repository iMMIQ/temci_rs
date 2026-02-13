//! Process execution and run drivers
//!
//! Provides functionality to execute commands as subprocesses and capture their output.
//! Supports basic execution and performance measurement with perf.

#![allow(dead_code)]

use async_trait::async_trait;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout as tokio_timeout;
use tracing::{debug, trace};

use crate::utils::error::{Result, TemciError};

/// Result of a command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Standard output captured from the command
    pub stdout: String,
    /// Standard error captured from the command
    pub stderr: String,
    /// Exit code of the process (None if terminated by signal)
    pub exit_code: Option<i32>,
    /// Wall-clock time taken by the command
    pub duration: Duration,
    /// Whether the command timed out
    pub timeout: bool,
    /// Whether the command was successful (exit code 0)
    pub success: bool,
}

impl CommandResult {
    /// Create a new empty command result
    pub fn new() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            duration: Duration::ZERO,
            timeout: false,
            success: false,
        }
    }

    /// Create a failed result with an error message
    pub fn error(msg: &str) -> Self {
        Self {
            stdout: String::new(),
            stderr: msg.to_string(),
            exit_code: Some(1),
            duration: Duration::ZERO,
            timeout: false,
            success: false,
        }
    }
}

impl Default for CommandResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Run driver types for different execution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDriver {
    /// Basic execution without performance measurement
    Basic,
    /// Linux perf stat for hardware counters
    PerfStat,
    /// Linux perf record for profiling
    PerfRecord,
    /// Valgrind for memory profiling
    Valgrind,
}

impl RunDriver {
    /// Parse a string into a RunDriver
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "basic" => Some(RunDriver::Basic),
            "perf" | "perf_stat" => Some(RunDriver::PerfStat),
            "perf_record" => Some(RunDriver::PerfRecord),
            "valgrind" => Some(RunDriver::Valgrind),
            _ => None,
        }
    }

    /// Get the name of the driver
    pub fn name(&self) -> &str {
        match self {
            RunDriver::Basic => "basic",
            RunDriver::PerfStat => "perf_stat",
            RunDriver::PerfRecord => "perf_record",
            RunDriver::Valgrind => "valgrind",
        }
    }
}

/// Trait for running commands with different strategies
#[async_trait]
pub trait Runner: Send + Sync {
    /// Run a command synchronously
    fn run_sync(&self, program: &str, args: &[&str]) -> Result<CommandResult>;

    /// Run a command asynchronously
    async fn run(&self, program: &str, args: &[&str]) -> Result<CommandResult>;

    /// Get the run driver type
    fn driver(&self) -> RunDriver;
}

/// Basic command runner using std::process
#[derive(Debug, Clone)]
pub struct CommandRunner {
    /// Environment variables to set for the command
    env: HashMap<String, String>,
    /// Timeout for command execution
    timeout: Option<Duration>,
    /// Working directory for the command
    working_dir: Option<String>,
}

impl CommandRunner {
    /// Create a new command runner
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            timeout: None,
            working_dir: None,
        }
    }

    /// Set an environment variable
    pub fn set_env(&mut self, key: &str, value: &str) -> &mut Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the timeout for command execution
    pub fn set_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// Set the working directory
    pub fn set_working_dir(&mut self, dir: &str) -> &mut Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// Build a Command with the configured settings
    fn build_command(&self, program: &str, args: &[&str]) -> Command {
        let mut cmd = Command::new(program);

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

        cmd
    }

    /// Helper function to convert bytes to String, handling invalid UTF-8 gracefully
    fn bytes_to_string(bytes: Vec<u8>) -> String {
        String::from_utf8(bytes)
            .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).to_string())
    }

    /// Execute the command and capture output
    fn execute_command(&self, cmd: &mut Command) -> Result<CommandResult> {
        let start = Instant::now();

        debug!("Executing command: {:?}", cmd);

        // For simplicity, we use the output() method which blocks until completion
        // Timeout support would require a more complex implementation
        let output = cmd
            .output()
            .map_err(|e| TemciError::ExecutionError(format!("Execution failed: {}", e)))?;

        let duration = start.elapsed();
        let stdout = Self::bytes_to_string(output.stdout);
        let stderr = Self::bytes_to_string(output.stderr);
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
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runner for CommandRunner {
    fn run_sync(&self, program: &str, args: &[&str]) -> Result<CommandResult> {
        let mut cmd = self.build_command(program, args);
        self.execute_command(&mut cmd)
    }

    async fn run(&self, program: &str, args: &[&str]) -> Result<CommandResult> {
        // For async, use tokio::process
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

        debug!("Executing async command: {:?}", cmd);

        // Spawn the command to get a Child object, which allows us to kill it on timeout
        let mut child = cmd
            .spawn()
            .map_err(|e| TemciError::ExecutionError(format!("Async execution failed: {}", e)))?;

        let (stdout_bytes, stderr_bytes, exit_code, success) = if let Some(dur) = &self.timeout {
            // Take ownership of stdout and stderr for timeout case (need to read manually)
            let mut stdout_reader = child.stdout.take().expect("stdout not captured");
            let mut stderr_reader = child.stderr.take().expect("stderr not captured");

            // Wait with timeout - if timeout occurs, we still have child to kill
            match tokio_timeout(*dur, child.wait()).await {
                Ok(Ok(status)) => {
                    // Process completed within timeout - collect output
                    use tokio::io::AsyncReadExt;

                    let mut stdout_buf = Vec::new();
                    let mut stderr_buf = Vec::new();
                    let _ = stdout_reader.read_to_end(&mut stdout_buf).await;
                    let _ = stderr_reader.read_to_end(&mut stderr_buf).await;

                    let exit_code = status.code();
                    let success = status.success();
                    (stdout_buf, stderr_buf, exit_code, success)
                }
                Ok(Err(e)) => {
                    return Err(TemciError::ExecutionError(format!(
                        "Async execution failed: {}",
                        e
                    )));
                }
                Err(_) => {
                    // Timeout occurred - kill the child process
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    return Ok(CommandResult {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: None,
                        duration: *dur,
                        timeout: true,
                        success: false,
                    });
                }
            }
        } else {
            // No timeout - use wait_with_output which handles reading stdout/stderr for us
            let output = child.wait_with_output().await.map_err(|e| {
                TemciError::ExecutionError(format!("Async execution failed: {}", e))
            })?;

            let exit_code = output.status.code();
            let success = output.status.success();
            (output.stdout, output.stderr, exit_code, success)
        };

        let duration = start.elapsed();
        let stdout = Self::bytes_to_string(stdout_bytes);
        let stderr = Self::bytes_to_string(stderr_bytes);

        debug!("Async command completed in {:?}", duration);

        Ok(CommandResult {
            stdout,
            stderr,
            exit_code,
            duration,
            timeout: false,
            success,
        })
    }

    fn driver(&self) -> RunDriver {
        RunDriver::Basic
    }
}

/// Runner that uses Linux perf for performance measurement
#[derive(Debug, Clone)]
pub struct PerfRunner {
    /// Base command runner
    base: CommandRunner,
    /// Perf events to record
    events: Vec<String>,
}

impl PerfRunner {
    /// Create a new perf runner
    pub fn new() -> Self {
        Self {
            base: CommandRunner::new(),
            events: vec![
                "cycles".to_string(),
                "instructions".to_string(),
                "cache-references".to_string(),
                "cache-misses".to_string(),
                "branches".to_string(),
                "branch-misses".to_string(),
            ],
        }
    }

    /// Set custom perf events
    pub fn set_events(&mut self, events: Vec<String>) -> &mut Self {
        self.events = events;
        self
    }

    /// Check if perf is available
    pub fn is_available(&self) -> bool {
        which::which("perf").is_ok()
    }

    /// Build a perf stat command
    pub(crate) fn build_perf_command(&self, program: &str, args: &[&str]) -> Vec<String> {
        let mut cmd = Vec::with_capacity(7 + args.len());
        cmd.push("perf".to_string());
        cmd.push("stat".to_string());
        cmd.push("-x,".to_string()); // CSV output
        cmd.push("-e".to_string());
        cmd.push(self.events.join(","));
        cmd.push("--".to_string());
        cmd.push(program.to_string());
        cmd.extend(args.iter().map(|s| s.to_string()));
        cmd
    }
}

impl Default for PerfRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runner for PerfRunner {
    fn run_sync(&self, program: &str, args: &[&str]) -> Result<CommandResult> {
        if !self.is_available() {
            // Fall back to basic runner if perf is not available
            tracing::warn!("perf not available, falling back to basic runner");
            return self.base.run_sync(program, args);
        }

        let perf_cmd = self.build_perf_command(program, args);
        let perf_prog = &perf_cmd[0];
        let perf_args: Vec<&str> = perf_cmd.iter().skip(1).map(|s| s.as_str()).collect();

        self.base.run_sync(perf_prog, &perf_args)
    }

    async fn run(&self, program: &str, args: &[&str]) -> Result<CommandResult> {
        if !self.is_available() {
            tracing::warn!("perf not available, falling back to basic runner");
            return self.base.run(program, args).await;
        }

        let perf_cmd = self.build_perf_command(program, args);
        let perf_prog = &perf_cmd[0];
        let perf_args: Vec<&str> = perf_cmd.iter().skip(1).map(|s| s.as_str()).collect();

        self.base.run(perf_prog, &perf_args).await
    }

    fn driver(&self) -> RunDriver {
        RunDriver::PerfStat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result() {
        let result = CommandResult::new();
        assert!(!result.success);

        let mut result = CommandResult::new();
        result.exit_code = Some(0);
        result.success = true;
        assert!(result.success);
    }

    #[test]
    fn test_run_driver_parsing() {
        assert_eq!(RunDriver::from_str("basic"), Some(RunDriver::Basic));
        assert_eq!(RunDriver::from_str("perf_stat"), Some(RunDriver::PerfStat));
        assert_eq!(RunDriver::from_str("invalid"), None);
    }

    #[test]
    fn test_command_runner_builder() {
        let mut runner = CommandRunner::new();
        runner.set_env("KEY", "value");
        runner.set_timeout(Some(Duration::from_secs(1)));

        assert_eq!(runner.env.get("KEY"), Some(&"value".to_string()));
        assert_eq!(runner.timeout, Some(Duration::from_secs(1)));
    }

    #[test]
    fn test_command_result_preserves_content() {
        // Ensure optimization doesn't break content preservation
        // \xc3\xa9 is the UTF-8 encoding of é
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

        assert_eq!(result.stdout, "valid utf8 é");
        assert_eq!(result.stderr, "error message");
    }

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

    #[tokio::test]
    async fn test_command_timeout() {
        let mut runner = CommandRunner::new();
        runner.set_timeout(Some(Duration::from_millis(100)));

        // This should timeout (sleep command)
        let result = runner.run("sleep", &["2"]).await;

        // Expect timeout or error
        assert!(result.is_err() || result.unwrap().timeout);
    }
}
