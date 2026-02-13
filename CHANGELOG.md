# Changelog

All notable changes to temci will be documented in this file.

## [0.8.5-rs] - 2024-01-XX

### Added
- Initial Rust implementation of temci benchmarking tool
- Core benchmark execution engine with precise timing
- Statistical analysis module with mean, median, percentiles, confidence intervals
- Multi-format reporting (console/text, markdown, JSON, CSV)
- CPU affinity control for process isolation
- Worker pool for parallel benchmark execution
- Build system integration with compiler detection (GCC, Clang, Rustc)
- Shell completion support for bash
- Configuration file support (YAML format)
- Environment variable management
- Working directory specification
- Timeout and warmup run support
- Save and load benchmark results

### Commands
- `short-exec` - Quick command execution with timing
- `exec` - Full benchmark execution with configuration files
- `report` - Generate reports from saved results
- `build` - Compile programs for benchmarking
- `clean` - Remove benchmark artifacts and caches
- `setup` - Initial configuration and tool detection
- `completion` - Generate shell completion scripts

### Performance
- Low-overhead timing measurements using std::time::Instant
- Efficient worker pool implementation with semaphore-based concurrency control
- Parallel statistical analysis using rayon
- Optimized JSON/YAML parsing with serde

### Changes from Python Version
- Replaced Python logging with `tracing` crate for structured logging
- Implemented type-safe error handling with `thiserror` and `anyhow`
- Used `clap 4` for modern CLI with derive macros
- Replaced custom thread pools with Tokio async runtime
- Implemented proper module structure following Rust best practices

### Migration Notes
- Configuration files remain compatible with Python version (YAML format)
- Result file format is compatible (JSON structure preserved)
- CLI arguments follow similar patterns but with modern naming conventions
- Use `temci short-exec` for quick benchmarks (equivalent to Python's short_exec)

### Technical Details
- Minimum Rust version: 1.70+ (edition 2021)
- Dependencies: clap, serde, tokio, rayon, tracing, statrs
- Async runtime: Tokio
- Parallel processing: Rayon for CPU-bound tasks, Tokio for I/O

## [0.8.5] - Python Version (Original)
- Original Python implementation with similar feature set
- Based on subprocess, psutil, and custom logging
- YAML configuration and JSON result files

### Future Plans
- [ ] Perf stat integration for hardware counter measurements
- [ ] Network benchmarking support
- [ ] Distributed benchmark execution
- [ ] Web UI for result visualization
- [ ] Database backend for result storage
