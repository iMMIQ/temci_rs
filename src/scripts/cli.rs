//! Command-line interface for temci
//!
//! Provides the main CLI parser and command dispatching.

use clap::{Parser, Subcommand};
use anyhow::Result as AnyhowResult;
use crate::scripts::{exec, short_exec, build_cmd, clean, report_cmd as report, setup, completion};

/// Advanced benchmarking tool
#[derive(Parser, Debug)]
#[command(name = "temci")]
#[command(author = "Johannes Hofmeister <johannes.hofmeister@gmail.com>")]
#[command(version = "0.8.5")]
#[command(about = "Advanced benchmarking tool with multiple run drivers and statistical analysis", long_about = None)]
pub struct TemciCli {
    /// Increase verbosity (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute benchmarks with full configuration
    Exec {
        /// Benchmark suite configuration
        #[arg(short, long)]
        suite: Option<String>,

        /// Number of executions per benchmark
        #[arg(short = 'r', long)]
        runs: Option<usize>,

        /// Run driver to use (basic, perf, perf_stat, perf_record, valgrind)
        #[arg(long)]
        driver: Option<String>,

        /// Disable CPU affinity
        #[arg(long)]
        no_affinity: bool,

        /// Show only summary
        #[arg(short = 'S', long)]
        summary: bool,
    },

    /// Quick execution of commands
    ShortExec {
        /// Commands to execute
        #[arg(required = true)]
        commands: Vec<String>,

        /// Number of executions
        #[arg(short = 'r', long, default_value = "10")]
        runs: usize,

        /// Warmup runs
        #[arg(short = 'w', long, default_value = "0")]
        warmup: usize,

        /// Show summary
        #[arg(short = 'S', long)]
        summary: bool,
    },

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
    },

    /// Clean build artifacts
    Clean {
        /// Clean all artifacts
        #[arg(long)]
        all: bool,
    },

    /// Generate benchmark report
    Report {
        /// Report type (console, csv, json)
        #[arg(short = 'f', long, default_value = "console")]
        format: String,

        /// Output file
        #[arg(short = 'o', long)]
        output: Option<String>,

        /// Input data file
        #[arg(short = 'i', long)]
        input: Option<String>,
    },

    /// Setup initial configuration
    Setup {
        /// Configuration file to create
        #[arg(short = 'c', long, default_value = "temci.yaml")]
        config: String,

        /// Overwrite existing config
        #[arg(long)]
        overwrite: bool,
    },

    /// Generate shell completion
    Completion {
        /// Shell type (bash, zsh, fish, elvish)
        #[arg(short = 's', long)]
        shell: Option<String>,
    },
}

impl TemciCli {
    /// Run the CLI with the given arguments
    pub async fn run(self) -> AnyhowResult<()> {
        // Note: tracing is initialized in main.rs

        // Dispatch to command handler
        match self.command {
            Commands::Exec { suite, runs, driver, no_affinity, summary } => {
                Self::handle_exec(suite, runs, driver, no_affinity, summary).await
            }
            Commands::ShortExec { commands, runs, warmup, summary } => {
                Self::handle_short_exec(commands, runs, warmup, summary).await
            }
            Commands::Build { config, force, release } => {
                build_cmd::build(config, force, release).await
            }
            Commands::Clean { all } => {
                Self::handle_clean(all).await
            }
            Commands::Report { format, output, input } => {
                Self::handle_report(format, output, input).await
            }
            Commands::Setup { config, overwrite } => {
                Self::handle_setup(config, overwrite).await
            }
            Commands::Completion { shell } => {
                Self::handle_completion(shell).await
            }
        }
    }

    async fn handle_exec(
        suite: Option<String>,
        runs: Option<usize>,
        driver: Option<String>,
        no_affinity: bool,
        summary: bool,
    ) -> AnyhowResult<()> {
        exec::exec(suite, runs, driver, no_affinity, summary).await
    }

    async fn handle_short_exec(
        commands: Vec<String>,
        runs: usize,
        warmup: usize,
        summary: bool,
    ) -> AnyhowResult<()> {
        short_exec::short_exec(commands, runs, warmup, summary).await
    }

    async fn handle_build(
        config: Option<String>,
        force: bool,
        release: bool,
    ) -> AnyhowResult<()> {
        tracing::info!("Build: config={:?} force={} release={}",
                       config, force, release);
        // Placeholder - will be implemented by build_cmd module
        Ok(())
    }

    async fn handle_clean(all: bool) -> AnyhowResult<()> {
        clean::clean(all).await
    }

    async fn handle_report(
        format: String,
        output: Option<String>,
        input: Option<String>,
    ) -> AnyhowResult<()> {
        report::report(format, output, input).await
    }

    async fn handle_setup(config: String, overwrite: bool) -> AnyhowResult<()> {
        setup::setup(config, overwrite).await
    }

    async fn handle_completion(shell: Option<String>) -> AnyhowResult<()> {
        completion::completion(shell).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = TemciCli::try_parse_from(["temci", "--verbose", "exec"]).unwrap();
        assert_eq!(cli.verbose, 1);
    }

    #[test]
    fn test_exec_command() {
        let cli = TemciCli::try_parse_from([
            "temci", "exec",
            "--suite", "my_suite",
            "--runs", "10",
        ]).unwrap();
        match cli.command {
            Commands::Exec { suite, runs, .. } => {
                assert_eq!(suite, Some("my_suite".to_string()));
                assert_eq!(runs, Some(10));
            }
            _ => panic!("Expected Exec command"),
        }
    }

    #[test]
    fn test_short_exec_command() {
        let cli = TemciCli::try_parse_from([
            "temci", "short-exec",
            "echo hello",
            "echo world",
            "--runs", "5",
        ]).unwrap();
        match cli.command {
            Commands::ShortExec { commands, runs, .. } => {
                assert_eq!(commands.len(), 2);
                assert_eq!(runs, 5);
            }
            _ => panic!("Expected ShortExec command"),
        }
    }
}
