// Utility modules - public API
#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod registry;
pub mod time;

// Re-exports for public API
#[allow(unused_imports)]
pub use config::*;
pub use error::{TemciError, Result};
// pub use time::{duration_as_ms, duration_to_ms};
// Don't export registry as it's unused internally
// pub use registry::{Registry, TypedRegistry};
