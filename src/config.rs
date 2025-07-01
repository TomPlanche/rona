//! Configuration Management Module for Rona
//!
//! This module handles all configuration-related functionality, including
//! - Reading and writing configuration files
//! - Managing editor preferences
//! - Handling configuration errors
//!
//! # Configuration Structure
//!
//! The configuration is stored in TOML format at `~/.config/rona/config.toml`
//! and contains settings such as
//! - Editor preferences
//! - Other configuration options
//!
//! # Error Handling
//!
//! The module provides a custom error type `ConfigError` that handles various
//! configuration-related errors including
//! - IO errors
//! - Missing configuration
//! - Invalid configuration format
//! - Home directory not found

use regex::Regex;
use std::{env, fs, path::PathBuf};

use crate::{
    errors::{ConfigError, GitError, Result},
    utils::print_error,
};

// Make this public so tests can use it directly
pub const CONFIG_FOLDER_NAME: &str = "rona-test-config";

/// Main configuration struct that handles all config operations.
/// This includes both persistent configuration (stored in config file)
/// and runtime configuration (command-line flags).
///
/// # Fields
/// * `root` - The root path for configuration files
/// * `verbose` - Whether to show detailed output
/// * `dry_run` - Whether to simulate operations without making changes
pub struct Config {
    root: PathBuf,
    pub(crate) verbose: bool,
    pub(crate) dry_run: bool,
}

impl Config {
    /// Creates a new Config instance with the default root path and default settings.
    ///
    /// # Errors
    /// * If getting the config root path fails
    /// * If the home directory cannot be determined
    ///
    /// # Returns
    /// * `Result<Config>` - A new Config instance with default settings
    pub fn new() -> Result<Self> {
        let root = Config::get_config_root()?;
        let config = Config {
            root,
            verbose: false,
            dry_run: false,
        };

        Ok(config)
    }

    /// Creates a new Config instance with a custom root path.
    /// This is primarily used for testing purposes.
    ///
    /// # Arguments
    /// * `root` - The custom root path for configuration files
    ///
    /// # Returns
    /// * `Config` - A new Config instance with the specified root and default settings
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Config {
            root: root.into(),
            verbose: false,
            dry_run: false,
        }
    }

    /// Sets the verbose flag which controls detailed output logging.
    ///
    /// # Arguments
    /// * `verbose` - Whether to enable verbose output
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Sets the `dry_run` flag which controls whether operations are simulated.
    /// When true, operations will print what would happen without making actual changes.
    ///
    /// # Arguments
    /// * `dry_run` - Whether to enable dry run mode
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Retrieves the editor from the configuration file.
    ///
    /// # Errors
    /// * If the configuration file cannot be read
    /// * If the configuration file does not exist
    /// * If the regex pattern fails to compile
    /// * If the editor setting is missing or invalid
    ///
    /// # Returns
    /// * `Result<String>` - The configured editor command
    pub fn get_editor(&self) -> Result<String> {
        let config_file = self.get_config_file_path()?;

        if !config_file.exists() {
            if !cfg!(test) {
                print_error(
                    "Configuration file not found",
                    "Please create a configuration file",
                    "Use the `rona init [editor]` to create a configuration file",
                );
            }

            return Err(ConfigError::ConfigNotFound.into());
        }

        let config_content = fs::read_to_string(&config_file)?;
        let regex = Config::get_regex_editor()?;

        let editor = regex
            .captures(config_content.trim())
            .and_then(|captures| captures.get(1))
            .map(|match_| match_.as_str().to_string())
            .ok_or(ConfigError::InvalidConfig)?;

        Ok(editor.trim().to_string())
    }

    /// Sets the editor in the configuration file.
    ///
    /// # Arguments
    /// * `editor` - The editor command to set
    ///
    /// # Errors
    /// * If the configuration file cannot be read or written
    /// * If the configuration file does not exist
    /// * If the regex pattern fails to compile
    pub fn set_editor(&self, editor: &str) -> Result<()> {
        let config_file = self.get_config_file_path()?;

        if !config_file.exists() {
            if !cfg!(test) {
                print_error(
                    "Configuration file not found",
                    "Please create a configuration file first",
                    "Use the `rona init [editor]` command to create a new configuration file",
                );
            }

            return Err(ConfigError::ConfigNotFound.into());
        }

        let config_content = fs::read_to_string(&config_file)?;
        let regex = Config::get_regex_editor()?;

        let new_config_content = regex
            .replace(&config_content, &format!("editor = \"{editor}\""))
            .to_string();

        fs::write(&config_file, new_config_content)?;

        Ok(())
    }

    /// Creates a new configuration file with the specified editor.
    ///
    /// # Arguments
    /// * `editor` - The editor command to configure
    ///
    /// # Errors
    /// * If creating the configuration directory fails
    /// * If writing the configuration file fails
    /// * If the configuration file already exists
    pub fn create_config_file(&self, editor: &str) -> Result<()> {
        let config_folder = self.get_config_folder_path()?;

        if !config_folder.exists() {
            fs::create_dir_all(config_folder)?;
        }

        let config_file = self.get_config_file_path()?;
        let config_content = format!("editor = \"{editor}\"");

        if config_file.exists() {
            if !cfg!(test) {
                print_error(
                    "Configuration file already exists.",
                    &format!(
                        "A configuration file already exists at {}",
                        config_file.display()
                    ),
                    "Use `rona --set-editor <editor>` (or `rona -s <editor>`) to change it.",
                );
            }

            return Err(ConfigError::ConfigAlreadyExists.into());
        }

        fs::write(&config_file, config_content)?;

        Ok(())
    }

    /// Returns the path to the configuration folder.
    ///
    /// # Errors
    /// * If the home directory cannot be determined
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The path to the configuration folder
    pub fn get_config_folder_path(&self) -> Result<PathBuf> {
        let config_folder_path = self.root.join(".config").join("rona");
        Ok(config_folder_path)
    }

    /// Returns the path to the configuration file.
    ///
    /// # Errors
    /// * If the home directory cannot be determined
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The path to the configuration file
    pub fn get_config_file_path(&self) -> Result<PathBuf> {
        let config_folder_path = self.get_config_folder_path()?;
        Ok(config_folder_path.join("config.toml"))
    }

    /// Returns the root directory for the configuration files.
    /// Uses the test directory if `RONA_TEST_DIR` is set or running tests.
    ///
    /// # Errors
    /// * If the home directory cannot be determined
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The root directory for configuration files
    fn get_config_root() -> Result<PathBuf> {
        // Use environment variable for testing
        if env::var("RONA_TEST_DIR").is_ok() || cfg!(test) {
            Ok(PathBuf::from(CONFIG_FOLDER_NAME))
        } else {
            let root = env::var("HOME").or_else(|_| env::var("USERPROFILE"));

            if root.is_err() {
                return Err(GitError::RepositoryNotFound.into());
            }

            Ok(PathBuf::from(root.unwrap()))
        }
    }

    /// Returns the regex pattern used to match the editor setting in the config file.
    ///
    /// # Errors
    /// * If the regex pattern fails to compile
    ///
    /// # Returns
    /// * `Result<Regex>` - The compiled regex pattern
    fn get_regex_editor() -> Result<Regex> {
        Regex::new(r#"editor\s*=\s*"(.*?)""#).map_err(|e| ConfigError::RegexError(e).into())
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::RonaError;

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());
        let editor = "test_editor";

        // Create a new config file with the temp directory as root
        assert!(config.create_config_file(editor).is_ok());

        // Check the file exists and has the correct content
        let config_file = config.get_config_file_path().unwrap();
        assert!(config_file.exists());

        let content = fs::read_to_string(&config_file).unwrap();
        assert_eq!(content, format!("editor = \"{editor}\""));

        // Test error when a file already exists
        assert!(config.create_config_file(editor).is_err());
    }

    #[test]
    fn test_get_editor() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());
        let editor = "nano";

        // Create a config file
        config.create_config_file(editor).unwrap();

        // Test getting the editor
        let result = config.get_editor();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), editor);
    }

    #[test]
    fn test_set_editor() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());
        let initial_editor = "vim";

        // Create a config file
        config.create_config_file(initial_editor).unwrap();

        // Test setting a new editor
        let new_editor = "emacs";
        assert!(config.set_editor(new_editor).is_ok());

        // Verify the editor was updated
        let result = config.get_editor();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), new_editor);
    }

    #[test]
    fn test_get_editor_error_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());

        // Don't create a config file, verify we get an error
        assert!(matches!(
            config.get_editor(),
            Err(RonaError::Config(ConfigError::ConfigNotFound))
        ));
    }

    #[test]
    fn test_set_editor_error_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());

        // Don't create a config file, verify we get an error
        assert!(matches!(
            config.set_editor("vim"),
            Err(RonaError::Config(ConfigError::ConfigNotFound))
        ));
    }

    #[test]
    fn test_malformed_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());

        // Create a config directory
        let config_folder = config.get_config_folder_path().unwrap();
        fs::create_dir_all(&config_folder).unwrap();

        // Create a malformed config file
        let config_file = config.get_config_file_path().unwrap();
        fs::write(&config_file, "editor = missing_quotes").unwrap();

        // Test that get_editor returns an error
        assert!(matches!(
            config.get_editor(),
            Err(RonaError::Config(ConfigError::InvalidConfig))
        ));
    }
}
