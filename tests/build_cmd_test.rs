//! Integration tests for the build command

use std::process::Command;

fn temci_bin() -> String {
    let debug_path = "target/debug/temci";
    let release_path = "target/release/temci";
    if std::path::Path::new(release_path).exists() {
        release_path.to_string()
    } else if std::path::Path::new(debug_path).exists() {
        debug_path.to_string()
    } else {
        "temci".to_string()
    }
}

#[test]
fn test_build_with_compiler_and_opt_level_flags() {
    let output = Command::new(temci_bin())
        .args(["build", "--compiler", "gcc", "--opt-level", "O3"])
        .output()
        .unwrap();
    assert!(output.status.success(), "build command failed: {:?}",
            String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_build_help_mentions_compiler() {
    let output = Command::new(temci_bin())
        .args(["build", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compiler") || stdout.contains("COMPILER"));
    assert!(stdout.contains("opt-level") || stdout.contains("LEVEL"));
}
