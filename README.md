# temci

Advanced benchmarking tool - Rust implementation

**语言 / Language / 語言:**
[English](README.md) | [简体中文](README.zh.md) | [繁體粵語](README.zh-yue.md) | [日本語](README.ja.md)

## Description

**temci** (Timing Execution Measuring Control and Inspection) is a comprehensive benchmarking tool for measuring and analyzing command execution performance. This is the Rust implementation of the original Python tool, providing enhanced performance and reliability.

temci allows you to:
- Execute commands with precise timing measurements
- Run benchmarks with configurable parameters (runs, timeouts, CPU affinity)
- Generate detailed reports in multiple formats (text, markdown, JSON, CSV)
- Perform statistical analysis on benchmark results
- Control CPU affinity and process isolation
- Build and compile benchmark programs

## Features

### Core Benchmarking
- **Precise Timing**: High-precision timing measurements using wall-clock and CPU time
- **Multiple Runners**: Support for basic, performance counter, and custom run drivers
- **Parallel Execution**: Worker pool for concurrent benchmark execution
- **Configurable Parameters**: Runs, timeouts, warmup runs, show output options

### Statistical Analysis
- Mean, median, standard deviation calculations
- Percentile calculations (p50, p90, p95, p99)
- Outlier detection and filtering
- Confidence interval computation

### Reporting
- Multiple output formats: console/text, markdown, JSON, CSV
- Customizable report layouts
- Save and load benchmark results
- Comparison reports between runs

### Process Control
- CPU affinity control (pin to specific cores)
- Process isolation and priority control
- Environment variable management
- Working directory specification

### Build System
- Support for GCC, Clang, and Rust compiler detection
- Automatic compiler detection and optimization level selection
- Integration with benchmark execution

### Shell Integration
- Bash completion support
- Shell output formatting

## Installation

### From Source

```bash
git clone https://github.com/iMMIQ/temci_rs.git
cd temci_rs
cargo build --release
sudo install target/release/temci /usr/local/bin/
```

### Using Cargo

```bash
cargo install temci
```

## Usage

### Basic Execution

Execute a command and measure its execution time:

```bash
temci short-exec 'echo "Hello, World!"'
```

Run a benchmark with multiple executions:

```bash
temci short-exec --runs 20 'sleep 1'
```

### Benchmark Configuration

Create a YAML configuration file (`temci.yaml`):

```yaml
benchmarks:
  - name: "sleep test"
    command: "sleep"
    args: ["0.1"]
    runs: 10
    timeout: 5

  - name: "command test"
    command: "echo"
    args: ["test"]
    runs: 5
    show_output: true
```

Run benchmarks from configuration:

```bash
temci exec
```

### Report Generation

Generate a report from saved results:

```bash
# Console output (default)
temci report

# Save to file
temci report --output results.txt

# Different formats
temci report --format json --output results.json
temci report --format csv --output results.csv
temci report --format markdown --output results.md
```

### Build Integration

Compile and benchmark programs:

```bash
# Setup build environment
temci setup

# Build with detected compiler
temci build --compiler gcc --opt-level O3
```

### Shell Completion

Enable bash completion:

```bash
# Generate completion file
temci completion bash > /etc/bash_completion.d/temci

# Or source directly
source <(temci completion bash)
```

### Cleanup

Remove benchmark artifacts and cache:

```bash
# Clean default artifacts
temci clean

# Clean all artifacts including caches
temci clean --all
```

## Command Reference

### `short-exec`
Quick execution with timing measurement.
```bash
temci short-exec [OPTIONS] <COMMAND> [ARGS]...
```

### `exec`
Full benchmark execution with configuration file support.
```bash
temci exec [OPTIONS]
```

### `report`
Generate reports from saved benchmark results.
```bash
temci report [OPTIONS] [--format FORMAT] [--output FILE]
```

### `build`
Compile programs for benchmarking.
```bash
temci build [OPTIONS]
```

### `clean`
Remove benchmark artifacts.
```bash
temci clean [OPTIONS]
```

### `setup`
Initial configuration and tool detection.
```bash
temci setup [OPTIONS]
```

### `completion`
Generate shell completion scripts.
```bash
temci completion <SHELL>
```

## Configuration File Format

temci uses YAML configuration files for complex benchmark scenarios:

```yaml
# Global settings
runs: 100
timeout: 60
show_output: false
show_stderr: false
unscaled_stdout: true
work_dir: /path/to/benchmarks

# Environment variables
env:
  RUST_BACKTRACE: "1"
  CUSTOM_VAR: "value"

# CPU affinity (comma-separated list or range)
cpuset: "0-3"  # Use cores 0, 1, 2, 3

# Individual benchmarks
benchmarks:
  - name: "benchmark 1"
    command: "program"
    args: ["--arg1", "value"]
    runs: 50
    warmup: 5

  - name: "benchmark 2"
    command: "program"
    args: ["--arg2", "value"]
    env:
      SPECIAL_VAR: "special_value"
```

## Result Files

Benchmark results are saved in JSON or YAML format:

```json
{
  "name": "benchmark suite",
  "timestamp": "2024-01-01T00:00:00Z",
  "results": [
    {
      "command": "sleep",
      "args": ["0.1"],
      "runs": 10,
      "successful": 10,
      "failed": 0,
      "min_ms": 100.5,
      "max_ms": 105.2,
      "avg_ms": 102.3,
      "total_ms": 1023.0
    }
  ]
}
```

## License

GPL-3.0

## Contributing

Contributions are welcome! Please submit pull requests or open issues for bugs and feature requests.

## Migration from Python Version

This Rust implementation provides:
- **Better Performance**: Faster execution and lower overhead
- **Type Safety**: Compile-time guarantees prevent runtime errors
- **Modern CLI**: Enhanced command-line interface with better help text
- **Tracing**: Structured logging using the `tracing` crate

See COMPARISON.md for detailed feature comparison.
