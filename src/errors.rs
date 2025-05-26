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

    #[error("Operation cancelled by user")]
    UserCancelled,

    #[error("Command execution failed: {command}")]
    CommandFailed { command: String },
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error while accessing config: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Regex compilation error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Configuration file not found at expected location")]
    ConfigNotFound,

    #[error("Configuration file already exists - use 'rona set-editor' to modify")]
    ConfigAlreadyExists,

    #[error("Invalid configuration format - please check your config.toml syntax")]
    InvalidConfig,

    #[error("Could not determine home directory - please set HOME environment variable")]
    HomeDirNotFound,

    #[error("Unsupported editor: {editor}. Supported editors: vim, zed, nano")]
    UnsupportedEditor { editor: String },
}

/// Git-related errors
#[derive(Error, Debug)]
pub enum GitError {
    #[error("IO error during git operation: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not in a git repository - please run this command from within a git repository")]
    RepositoryNotFound,

    #[error("Git command failed: {command}\nOutput: {output}")]
    CommandFailed { command: String, output: String },

    #[error("Invalid git status output format: {output}")]
    InvalidStatus { output: String },

    #[error("Commit message file 'commit_message.md' not found - run 'rona generate' first")]
    CommitMessageNotFound,

    #[error("Failed to process .gitignore file: {reason}")]
    GitignoreError { reason: String },

    #[error("Failed to process .commitignore file: {reason}")]
    CommitignoreError { reason: String },

    #[error("No staged changes to commit - use 'rona add-with-exclude' to stage files")]
    NoStagedChanges,

    #[error("Working directory is not clean - commit or stash your changes first")]
    DirtyWorkingDirectory,

    #[error("Remote repository not configured - add a remote with 'git remote add origin <url>'")]
    NoRemoteConfigured,
}

/// Type alias for Result using `RonaError`
pub type Result<T> = std::result::Result<T, RonaError>;
