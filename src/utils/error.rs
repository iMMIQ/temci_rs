use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemciError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Run error: {0}")]
    Run(String),

    #[error("Build error: {0}")]
    Build(String),

    #[error("Report error: {0}")]
    Report(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, TemciError>;
