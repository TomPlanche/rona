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

use std::{env, path::PathBuf};

use crate::{
    errors::{ConfigError, GitError, Result},
    utils::print_error,
};

use crate::my_clap_theme;
use crate::utils::find_project_root;
use config as config_crate;
use dialoguer::Select;
use serde::{Deserialize, Serialize};

// Define your default commit types
const DEFAULT_COMMIT_TYPES: &[&str] = &["feat", "fix", "docs", "test", "chore"];

/// Project-specific configuration that can be defined in rona.toml
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectConfig {
    /// Editor command to use for commit messages
    pub editor: Option<String>,

    /// Custom commit types for this project
    pub commit_types: Option<Vec<String>>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            editor: Some("nano".to_string()),
            commit_types: Some(
                DEFAULT_COMMIT_TYPES
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            ),
        }
    }
}

impl ProjectConfig {
    /// Loads the project configuration, merging global and project config files.
    ///
    /// # Errors
    /// Returns `ConfigError::ConfigNotFound` if the config files cannot be found or read.
    /// Returns `ConfigError::InvalidConfig` if deserialization fails.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined (i.e., if `std::env::current_dir()` fails).
    pub fn load() -> Result<Self> {
        let mut builder = config_crate::Config::builder();

        // Support both old and new global config paths
        let home = dirs::home_dir().ok_or(ConfigError::ConfigNotFound)?;
        let old_global = home.join(".config/rona/config.toml");
        let new_global = home.join(".config/rona.toml");
        if old_global.exists() {
            builder =
                builder.add_source(config_crate::File::from(old_global.clone()).required(false));
        }
        if new_global.exists() {
            builder =
                builder.add_source(config_crate::File::from(new_global.clone()).required(false));
        }

        // Add project config if it exists
        let project_config_path = env::current_dir()?.join(".rona.toml");
        if project_config_path.exists() {
            builder = builder
                .add_source(config_crate::File::from(project_config_path.clone()).required(false));
        }

        // Build the config
        let settings = builder.build().map_err(|_| ConfigError::ConfigNotFound)?;
        match settings.try_deserialize() {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Failed to deserialize config: {e}");
                Err(ConfigError::InvalidConfig.into())
            }
        }
    }
}

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
    pub project_config: ProjectConfig,
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
        let project_config = ProjectConfig::load()?;
        let config = Config {
            root,
            verbose: false,
            dry_run: false,
            project_config,
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
        let root = root.into();
        let project_config = ProjectConfig::load().unwrap_or_default();

        Config {
            root,
            verbose: false,
            dry_run: false,
            project_config,
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
    /// * If the editor setting is missing or invalid
    ///
    /// # Returns
    /// * `Result<String>` - The configured editor command
    pub fn get_editor(&self) -> Result<String> {
        self.project_config
            .editor
            .clone()
            .ok_or(ConfigError::InvalidConfig.into())
    }

    /// Sets the editor in the configuration file.
    ///
    /// # Arguments
    /// * `editor` - The editor command to set
    ///
    /// # Errors
    /// * If the configuration file cannot be read or written
    /// * If the configuration file does not exist
    pub fn set_editor(&self, editor: &str) -> Result<()> {
        use dialoguer::Select;
        use std::io::Write;
        let options = ["Project (./.rona.toml)", "Global (~/.config/rona.toml)"];

        let selection = Select::with_theme(&my_clap_theme::ColorfulTheme::default())
            .with_prompt("Where do you want to set the editor?")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|_| ConfigError::InvalidConfig)?;

        let config_path = match selection {
            0 => find_project_root()
                .map(|root| root.join(".rona.toml"))
                .map_err(|_| ConfigError::ConfigNotFound)?,
            1 => {
                let home = dirs::home_dir().ok_or(ConfigError::ConfigNotFound)?;
                home.join(".config/rona.toml")
            }
            _ => unreachable!(),
        };

        let mut config = self.project_config.clone();
        config.editor = Some(editor.to_string());
        let toml_str = toml::to_string_pretty(&config).map_err(|_| ConfigError::InvalidConfig)?;
        let mut file = std::fs::File::create(&config_path)?;
        file.write_all(toml_str.as_bytes())?;
        println!("Editor set in: {}", config_path.display());
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
        let options = ["Project (.rona.toml)", "Global (~/.config/rona.toml)"];
        let selection = Select::with_theme(&my_clap_theme::ColorfulTheme::default())
            .with_prompt("Where do you want to initialize the config?")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|_| ConfigError::InvalidConfig)?;

        let config_path = match selection {
            0 => env::current_dir()?.join(".rona.toml"),
            1 => {
                let home = dirs::home_dir().ok_or(ConfigError::ConfigNotFound)?;
                home.join(".config/rona.toml")
            }
            _ => unreachable!(),
        };

        let config_folder = config_path.parent().ok_or(ConfigError::ConfigNotFound)?;
        if !config_folder.exists() {
            std::fs::create_dir_all(config_folder)?;
        }

        if config_path.exists() {
            if !cfg!(test) {
                print_error(
                    "Configuration file already exists.",
                    &format!(
                        "A configuration file already exists at {}",
                        config_path.display()
                    ),
                    "Use `rona --set-editor <editor>` (or `rona -s <editor>`) to change it.",
                );
            }
            return Err(ConfigError::ConfigAlreadyExists.into());
        }

        let mut config = self.project_config.clone();
        config.editor = Some(editor.to_string());

        let toml_str = toml::to_string_pretty(&config).map_err(|_| ConfigError::InvalidConfig)?;
        std::fs::write(&config_path, toml_str)?;

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
}

// Make this public so tests can use it directly
pub const CONFIG_FOLDER_NAME: &str = "rona-test-config";

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

        let content = std::fs::read_to_string(&config_file).unwrap();
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
            Err(RonaError::Config(ConfigError::InvalidConfig))
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
        std::fs::create_dir_all(&config_folder).unwrap();

        // Create a malformed config file
        let config_file = config.get_config_file_path().unwrap();
        std::fs::write(&config_file, "editor = missing_quotes").unwrap();

        // Test that get_editor returns an error
        assert!(matches!(
            config.get_editor(),
            Err(RonaError::Config(ConfigError::InvalidConfig))
        ));
    }
}
