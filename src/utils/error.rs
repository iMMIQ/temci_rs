use thiserror::Error;

pub type Result<T> = std::result::Result<T, TemciError>;

#[derive(Error, Debug)]
pub enum TemciError {
    #[error("Unknown error: {0}")]
    Unknown(String),
}
