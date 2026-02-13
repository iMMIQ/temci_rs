// Utility modules - public API
#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod registry;

// Re-exports for public API
#[allow(unused_imports)]
pub use config::*;
pub use error::{TemciError, Result};
// Don't export registry as it's unused internally
// pub use registry::{Registry, TypedRegistry};
