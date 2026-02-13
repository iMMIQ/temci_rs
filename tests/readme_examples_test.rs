//! Integration tests for README examples
//!
//! This module tests all command examples documented in README.md to ensure
//! they work as documented and prevent documentation drift.

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the temci binary
fn temci_bin() -> PathBuf {
    let release_path = PathBuf::from("target/release/temci");
    let debug_path = PathBuf::from("target/debug/temci");

    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        PathBuf::from("temci")
    }
}

/// Run temci with arguments and return output
fn run_temci(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new(&temci_bin()).args(args).output()
}
