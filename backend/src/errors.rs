use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("executable not found at '{path}'")]
    ExecutableNotFound { path: PathBuf },

    #[error("key pattern not found in executable '{name}'")]
    KeyPatternNotFound { name: String },

    #[error("missing required environment variable '{name}'")]
    MissingEnvVar { name: String },

    #[error("configuration error: {message}")]
    Config { message: String },
}
