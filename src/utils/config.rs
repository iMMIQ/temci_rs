// Configuration structures - public API
#![allow(dead_code)]

use crate::utils::{Result, TemciError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunConfig {
    pub suites: Vec<BenchmarkSuite>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkSuite {
    pub attributes: HashMap<String, String>,
    pub data: Option<BenchmarkData>,
    #[serde(default)]
    pub runs: Option<Vec<ExecRunEntry>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkData {
    #[serde(flatten)]
    pub metrics: HashMap<String, Vec<f64>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecRunEntry {
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub run: Option<RunSettings>,
    pub report: Option<ReportSettings>,
    pub build: Option<BuildSettings>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunSettings {
    pub iterations: Option<usize>,
    pub warmup: Option<usize>,
    pub show_stderr: Option<bool>,
    pub show_stdout: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportSettings {
    pub output_format: Option<String>,
    pub output_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildSettings {
    pub parallel: Option<bool>,
    pub jobs: Option<usize>,
}

impl RunConfig {
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: serde_yaml::Value = serde_yaml::from_str(&content)?;
        let suites = if config.is_sequence() {
            serde_yaml::from_value(config)?
        } else if config.is_mapping() {
            if let Some(suites) = config.get("suites") {
                serde_yaml::from_value(suites.clone())?
            } else {
                return Err(TemciError::Config("No 'suites' key in config".to_string()));
            }
        } else {
            return Err(TemciError::Config("Invalid config format".to_string()));
        };
        Ok(RunConfig { suites })
    }

    pub fn from_yaml_str(yaml: &str) -> Result<Self> {
        let config: serde_yaml::Value = serde_yaml::from_str(yaml)?;
        let suites = if config.is_sequence() {
            serde_yaml::from_value(config)?
        } else {
            return Err(TemciError::Config("Invalid config format".to_string()));
        };
        Ok(RunConfig { suites })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            run: Some(RunSettings {
                iterations: Some(10),
                warmup: Some(1),
                show_stderr: Some(false),
                show_stdout: Some(false),
            }),
            report: Some(ReportSettings {
                output_format: Some("console".to_string()),
                output_file: None,
            }),
            build: Some(BuildSettings {
                parallel: Some(true),
                jobs: None,
            }),
        }
    }
}

impl Settings {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut settings: Self = serde_yaml::from_str(&content)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to parse settings file, using defaults: {}", e);
                Self::default()
            });

        if settings.run.is_none() {
            settings.run = Self::default().run;
        }
        if settings.report.is_none() {
            settings.report = Self::default().report;
        }
        if settings.build.is_none() {
            settings.build = Self::default().build;
        }

        Ok(settings)
    }
}
