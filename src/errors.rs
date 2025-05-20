use thiserror::Error;

/// Main error type for the Rona application
#[derive(Error, Debug)]
pub enum RonaError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Configuration file not found")]
    ConfigNotFound,

    #[error("Configuration file already exists")]
    ConfigAlreadyExists,

    #[error("Invalid configuration file")]
    InvalidConfig,

    #[error("Could not determine home directory")]
    HomeDirNotFound,
}

/// Git-related errors
#[derive(Error, Debug)]
pub enum GitError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git repository not found")]
    RepositoryNotFound,

    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("Invalid git status output: {0}")]
    InvalidStatus(String),

    #[error("Commit message file not found")]
    CommitMessageNotFound,

    #[error("Failed to process gitignore: {0}")]
    GitignoreError(String),

    #[error("Failed to process commitignore: {0}")]
    CommitignoreError(String),
}

/// Type alias for Result using `RonaError`
pub type Result<T> = std::result::Result<T, RonaError>;
