use std::path::PathBuf;
use temci::build::builder::{
    BuildConfig, Builder as BuildBuilder, CompilerType, OptimizationLevel,
};

#[test]
fn test_compiler_type_from_str() {
    assert_eq!(CompilerType::from_str("gcc"), Some(CompilerType::Gcc));
    assert_eq!(CompilerType::from_str("clang"), Some(CompilerType::Clang));
    assert_eq!(CompilerType::from_str("rustc"), Some(CompilerType::Rustc));
    assert_eq!(CompilerType::from_str("invalid"), None);
}

#[test]
fn test_compiler_type_display() {
    assert_eq!(CompilerType::Gcc.to_string(), "gcc");
    assert_eq!(CompilerType::Clang.to_string(), "clang");
    assert_eq!(CompilerType::Rustc.to_string(), "rustc");
}

#[test]
fn test_optimization_level_display() {
    assert_eq!(OptimizationLevel::None.to_string(), "0");
    assert_eq!(OptimizationLevel::Basic.to_string(), "1");
    assert_eq!(OptimizationLevel::Standard.to_string(), "2");
    assert_eq!(OptimizationLevel::High.to_string(), "3");
}

#[test]
fn test_build_config_default() {
    let config = BuildConfig::default();
    assert_eq!(config.compiler, CompilerType::Gcc);
    assert_eq!(config.opt_level, OptimizationLevel::Standard);
    assert!(!config.debug_info);
}

#[test]
fn test_build_config_builder() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Clang)
        .with_optimization(OptimizationLevel::High)
        .with_debug_info(true);

    assert_eq!(config.compiler, CompilerType::Clang);
    assert_eq!(config.opt_level, OptimizationLevel::High);
    assert!(config.debug_info);
}

#[test]
fn test_build_config_with_output_dir() {
    let config = BuildConfig::new().with_output_dir("/tmp/build");

    assert_eq!(config.output_dir, Some(PathBuf::from("/tmp/build")));
}

#[test]
fn test_build_config_with_defines() {
    let config = BuildConfig::new()
        .with_define("DEBUG")
        .with_define("VERSION=1.0");

    assert_eq!(config.defines.len(), 2);
    assert!(config.defines.contains(&"DEBUG".to_string()));
    assert!(config.defines.contains(&"VERSION=1.0".to_string()));
}

#[test]
fn test_builder_creation() {
    let config = BuildConfig::new();
    let builder = BuildBuilder::new(config);
    assert_eq!(builder.config().compiler, CompilerType::Gcc);
}

#[test]
fn test_builder_get_command_gcc() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Gcc)
        .with_optimization(OptimizationLevel::High);
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &["-lm"]);
    assert!(cmd.contains("gcc"));
    assert!(cmd.contains("-O3"));
    assert!(cmd.contains("-lm"));
}

#[test]
fn test_builder_get_command_clang() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Clang)
        .with_optimization(OptimizationLevel::None);
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &[]);
    assert!(cmd.contains("clang"));
    assert!(cmd.contains("-O0"));
}

#[test]
fn test_builder_get_command_rustc() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Rustc)
        .with_optimization(OptimizationLevel::Standard);
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("main.rs", &[]);
    assert!(cmd.contains("rustc"));
    assert!(cmd.contains("-C opt-level=2"));
}

#[test]
fn test_builder_get_command_with_debug() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Gcc)
        .with_debug_info(true);
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &[]);
    assert!(cmd.contains("-g"));
}

#[test]
fn test_builder_get_command_with_defines() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Gcc)
        .with_define("TEST")
        .with_define("VALUE=42");
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &[]);
    assert!(cmd.contains("-DTEST"));
    assert!(cmd.contains("-DVALUE=42"));
}

#[test]
fn test_builder_get_command_with_output_dir() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Gcc)
        .with_output_dir("/tmp/build");
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &[]);
    assert!(cmd.contains("-o"));
    assert!(cmd.contains("/tmp/build"));
}

#[test]
fn test_optimization_level_from_str() {
    assert_eq!(
        OptimizationLevel::from_str("0"),
        Some(OptimizationLevel::None)
    );
    assert_eq!(
        OptimizationLevel::from_str("1"),
        Some(OptimizationLevel::Basic)
    );
    assert_eq!(
        OptimizationLevel::from_str("2"),
        Some(OptimizationLevel::Standard)
    );
    assert_eq!(
        OptimizationLevel::from_str("3"),
        Some(OptimizationLevel::High)
    );
    assert_eq!(OptimizationLevel::from_str("4"), None);
}

#[test]
fn test_build_config_with_warnings() {
    let config = BuildConfig::new()
        .with_warnings(true)
        .with_warnings_as_errors(true);

    assert!(config.warnings);
    assert!(config.warnings_as_errors);
}

#[test]
fn test_builder_get_command_with_warnings() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Gcc)
        .with_warnings(true);
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &[]);
    assert!(cmd.contains("-Wall"));
    assert!(cmd.contains("-Wextra"));
}

#[test]
fn test_builder_get_command_with_warnings_as_errors() {
    let config = BuildConfig::new()
        .with_compiler(CompilerType::Gcc)
        .with_warnings(true)
        .with_warnings_as_errors(true);
    let builder = BuildBuilder::new(config);

    let cmd = builder.get_command("test.c", &[]);
    assert!(cmd.contains("-Werror"));
}

#[test]
fn test_builder_determine_compiler_c() {
    let config = BuildConfig::new();
    let builder = BuildBuilder::new(config);

    let compiler = builder.determine_compiler("test.c");
    assert_eq!(compiler, CompilerType::Gcc); // Default
}

#[test]
fn test_builder_determine_compiler_cpp() {
    let config = BuildConfig::new();
    let builder = BuildBuilder::new(config);

    let compiler = builder.determine_compiler("test.cpp");
    assert_eq!(compiler, CompilerType::Gcc); // Default
}

#[test]
fn test_builder_determine_compiler_rs() {
    let config = BuildConfig::new();
    let builder = BuildBuilder::new(config);

    let compiler = builder.determine_compiler("main.rs");
    assert_eq!(compiler, CompilerType::Rustc);
}
