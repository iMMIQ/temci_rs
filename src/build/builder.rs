//! Build system for compiling benchmarks
//!
//! Provides functionality to build benchmark programs with various
//! compilers and build options.

#![allow(dead_code)]

use std::path::PathBuf;

/// Types of compilers supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerType {
    /// GNU Compiler Collection
    Gcc,
    /// LLVM Compiler Collection
    Clang,
    /// Rust Compiler
    Rustc,
}

impl CompilerType {
    /// Parse a string into a CompilerType
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gcc" => Some(CompilerType::Gcc),
            "clang" => Some(CompilerType::Clang),
            "rustc" => Some(CompilerType::Rustc),
            _ => None,
        }
    }

    /// Get the compiler executable name
    pub fn executable(&self) -> &str {
        match self {
            CompilerType::Gcc => "gcc",
            CompilerType::Clang => "clang",
            CompilerType::Rustc => "rustc",
        }
    }
}

impl std::fmt::Display for CompilerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerType::Gcc => write!(f, "gcc"),
            CompilerType::Clang => write!(f, "clang"),
            CompilerType::Rustc => write!(f, "rustc"),
        }
    }
}

/// Optimization levels for compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimization
    None = 0,
    /// Basic optimization
    Basic = 1,
    /// Standard optimization
    Standard = 2,
    /// High optimization
    High = 3,
}

impl OptimizationLevel {
    /// Parse a string into an OptimizationLevel
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(OptimizationLevel::None),
            "1" => Some(OptimizationLevel::Basic),
            "2" => Some(OptimizationLevel::Standard),
            "3" => Some(OptimizationLevel::High),
            _ => None,
        }
    }
}

impl std::fmt::Display for OptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

/// Configuration for building a benchmark
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Compiler type to use
    pub compiler: CompilerType,
    /// Optimization level
    pub opt_level: OptimizationLevel,
    /// Whether to include debug info
    pub debug_info: bool,
    /// Output directory for built artifacts
    pub output_dir: Option<PathBuf>,
    /// Preprocessor definitions
    pub defines: Vec<String>,
    /// Include paths
    pub include_paths: Vec<PathBuf>,
    /// Whether to enable warnings
    pub warnings: bool,
    /// Whether to treat warnings as errors
    pub warnings_as_errors: bool,
    /// Additional compiler flags
    pub extra_flags: Vec<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            compiler: CompilerType::Gcc,
            opt_level: OptimizationLevel::Standard,
            debug_info: false,
            output_dir: None,
            defines: Vec::new(),
            include_paths: Vec::new(),
            warnings: false,
            warnings_as_errors: false,
            extra_flags: Vec::new(),
        }
    }
}

impl BuildConfig {
    /// Create a new build configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the compiler type
    pub fn with_compiler(mut self, compiler: CompilerType) -> Self {
        self.compiler = compiler;
        self
    }

    /// Set the optimization level
    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.opt_level = level;
        self
    }

    /// Enable or disable debug info
    pub fn with_debug_info(mut self, debug: bool) -> Self {
        self.debug_info = debug;
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(dir.into());
        self
    }

    /// Add a preprocessor definition
    pub fn with_define(mut self, define: &str) -> Self {
        self.defines.push(define.to_string());
        self
    }

    /// Add an include path
    pub fn with_include(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    /// Enable or disable warnings
    pub fn with_warnings(mut self, warnings: bool) -> Self {
        self.warnings = warnings;
        self
    }

    /// Enable or disable warnings as errors
    pub fn with_warnings_as_errors(mut self, warnings_as_errors: bool) -> Self {
        self.warnings_as_errors = warnings_as_errors;
        self
    }

    /// Add extra compiler flags
    pub fn with_extra_flag(mut self, flag: &str) -> Self {
        self.extra_flags.push(flag.to_string());
        self
    }
}

/// Builder for compiling benchmark programs
#[derive(Debug, Clone)]
pub struct Builder {
    config: BuildConfig,
}

impl Builder {
    /// Create a new builder with the given configuration
    pub fn new(config: BuildConfig) -> Self {
        Self { config }
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &BuildConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut BuildConfig {
        &mut self.config
    }

    /// Determine the appropriate compiler for a file based on extension
    pub fn determine_compiler(&self, file: &str) -> CompilerType {
        let ext = std::path::Path::new(file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "rs" => CompilerType::Rustc,
            "c" | "cpp" | "cc" | "cxx" | "C" => self.config.compiler,
            _ => self.config.compiler,
        }
    }

    /// Get the command to compile a file
    pub fn get_command(&self, input: &str, extra_args: &[&str]) -> String {
        let compiler = self.determine_compiler(input);
        let mut cmd = String::new();

        cmd.push_str(compiler.executable());
        cmd.push(' ');

        // Optimization level
        let opt_flag = match compiler {
            CompilerType::Rustc => match self.config.opt_level {
                OptimizationLevel::None => "-C opt-level=0",
                OptimizationLevel::Basic => "-C opt-level=1",
                OptimizationLevel::Standard => "-C opt-level=2",
                OptimizationLevel::High => "-C opt-level=3",
            },
            _ => match self.config.opt_level {
                OptimizationLevel::None => "-O0",
                OptimizationLevel::Basic => "-O1",
                OptimizationLevel::Standard => "-O2",
                OptimizationLevel::High => "-O3",
            },
        };
        cmd.push_str(opt_flag);
        cmd.push(' ');

        // Debug info
        if self.config.debug_info {
            let debug_flag = match compiler {
                CompilerType::Rustc => "-g",
                _ => "-g",
            };
            cmd.push_str(debug_flag);
            cmd.push(' ');
        }

        // Warnings
        if self.config.warnings {
            match compiler {
                CompilerType::Rustc => {
                    cmd.push_str("-Wall");
                    cmd.push(' ');
                }
                _ => {
                    cmd.push_str("-Wall -Wextra");
                    cmd.push(' ');
                }
            }
        }

        // Warnings as errors
        if self.config.warnings_as_errors {
            cmd.push_str("-Werror");
            cmd.push(' ');
        }

        // Defines
        for define in &self.config.defines {
            cmd.push_str(&format!("-D{} ", define));
        }

        // Include paths
        for path in &self.config.include_paths {
            cmd.push_str(&format!("-I{} ", path.display()));
        }

        // Extra flags
        for flag in &self.config.extra_flags {
            cmd.push_str(flag);
            cmd.push(' ');
        }

        // Input file
        cmd.push_str(input);
        cmd.push(' ');

        // Extra args (libraries, etc.)
        for arg in extra_args {
            cmd.push_str(arg);
            cmd.push(' ');
        }

        // Output file
        if let Some(output_dir) = &self.config.output_dir {
            let input_stem = std::path::Path::new(input)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            let output = output_dir.join(input_stem);
            cmd.push_str(&format!("-o {}", output.display()));
        }

        // Clean up trailing space
        cmd.trim_end().to_string()
    }

    /// Create a default builder
    pub fn default_builder() -> Self {
        Self::new(BuildConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_executable() {
        assert_eq!(CompilerType::Gcc.executable(), "gcc");
        assert_eq!(CompilerType::Clang.executable(), "clang");
        assert_eq!(CompilerType::Rustc.executable(), "rustc");
    }
}
