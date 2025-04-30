use regex::Regex;
use std::{env, fs, path::PathBuf};

use crate::utils::print_error;

// Make this public so tests can use it directly
pub const CONFIG_FOLDER_NAME: &str = "rona-test-config";

/// Custom error type for configuration operations
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    RegexError(regex::Error),
    ConfigNotFound,
    ConfigAlreadyExists,
    InvalidConfig,
    HomeDirNotFound,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {e}"),
            ConfigError::RegexError(e) => write!(f, "Regex error: {e}"),
            ConfigError::ConfigNotFound => write!(f, "Configuration file not found"),
            ConfigError::ConfigAlreadyExists => write!(f, "Configuration file already exists"),
            ConfigError::InvalidConfig => write!(f, "Invalid configuration file"),
            ConfigError::HomeDirNotFound => write!(f, "Could not determine home directory"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<regex::Error> for ConfigError {
    fn from(error: regex::Error) -> Self {
        ConfigError::RegexError(error)
    }
}

/// Main configuration struct that handles all config operations
pub struct Config {
    root: PathBuf,
}

impl Config {
    /// `new`
    /// Creates a new Config instance with the default root
    ///
    /// ## Errors
    /// * When getting the config root fails
    pub fn new() -> Result<Self, ConfigError> {
        let root = Config::get_config_root()?;

        Ok(Config { root })
    }

    /// `with_root`
    /// Creates a new Config instance with a custom root path
    ///
    /// ## Arguments
    /// * `root` - The custom root path
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Config { root: root.into() }
    }

    /// # `get_editor`
    /// Retrieves the editor from the configuration file.
    ///
    /// ## Errors
    /// * If the configuration file cannot be read or if it is inexistent.
    /// * If the regex pattern fails to compile.
    /// * If the regex pattern fails to match the editor.
    pub fn get_editor(&self) -> Result<String, ConfigError> {
        let config_file = self.get_config_file_path()?;

        if !config_file.exists() {
            if !cfg!(test) {
                print_error(
                    "Configuration file not found",
                    "Please create a configuration file",
                    "Use the `rona init [editor]` to create a configuration file",
                );
            }

            return Err(ConfigError::ConfigNotFound);
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

    /// # `set_editor`
    /// Sets the editor in the configuration file.
    ///
    /// ## Arguments
    /// * `editor` - The editor to set.
    ///
    /// ## Errors
    /// Returns an error if the configuration file cannot be read or written.
    pub fn set_editor(&self, editor: &str) -> Result<(), ConfigError> {
        let config_file = self.get_config_file_path()?;

        if !config_file.exists() {
            if !cfg!(test) {
                print_error(
                    "Configuration file not found",
                    "Please create a configuration file first",
                    "Use the `rona init [editor]` command to create a new configuration file",
                );
            }

            return Err(ConfigError::ConfigNotFound);
        }

        let config_content = fs::read_to_string(&config_file)?;
        let regex = Config::get_regex_editor()?;

        let new_config_content = regex
            .replace(&config_content, &format!("editor = \"{editor}\""))
            .to_string();

        fs::write(&config_file, new_config_content)?;

        Ok(())
    }

    /// # `create_config_file`
    /// Creates a new configuration file
    ///
    /// ## Arguments
    /// * `editor` - The editor to use
    ///
    /// ## Errors
    /// * If an I/O error occurs while creating the configuration file
    /// * If the file already exists
    pub fn create_config_file(&self, editor: &str) -> Result<(), ConfigError> {
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

            return Err(ConfigError::ConfigAlreadyExists);
        }

        fs::write(&config_file, config_content)?;

        Ok(())
    }

    /// # `get_config_folder_path`
    /// Returns the path to the configuration folder.
    ///
    /// ## Errors
    /// * If the home directory cannot be determined
    ///
    /// ## Returns
    /// The path to the configuration folder.
    pub fn get_config_folder_path(&self) -> Result<PathBuf, ConfigError> {
        let config_folder_path = self.root.join(".config").join("rona");

        Ok(config_folder_path)
    }

    /// # `get_config_file_path`
    /// Returns the path to the configuration file
    ///
    /// ## Errors
    /// * If the home directory cannot be determined
    ///
    /// ## Returns
    /// The path to the configuration file
    pub fn get_config_file_path(&self) -> Result<PathBuf, ConfigError> {
        let config_folder_path = self.get_config_folder_path()?;

        Ok(config_folder_path.join("config.toml"))
    }

    /// # `get_config_root`
    /// Returns the root directory for the configuration files
    ///
    /// ## Errors
    /// * If the home directory cannot be determined
    ///
    /// ## Returns
    /// The root directory for the configuration files
    fn get_config_root() -> Result<PathBuf, ConfigError> {
        // Use environment variable for testing
        if env::var("RONA_TEST_DIR").is_ok() || cfg!(test) {
            Ok(PathBuf::from(CONFIG_FOLDER_NAME))
        } else {
            let root = env::var("HOME").or_else(|_| env::var("USERPROFILE"));

            if root.is_err() {
                return Err(ConfigError::HomeDirNotFound);
            }

            Ok(PathBuf::from(root.unwrap()))
        }
    }

    /// # `get_regex_editor`
    /// Returns the regex to match the editor in the configuration file
    ///
    /// ## Errors
    /// * If the regex cannot be compiled
    ///
    /// ## Returns
    /// The regex to match the editor in the configuration file
    fn get_regex_editor() -> Result<Regex, ConfigError> {
        let regex = Regex::new(r#"editor\s*=\s*"(.*?)""#);

        if regex.is_err() {
            return Err(ConfigError::RegexError(regex.err().unwrap()));
        }

        Ok(regex?)
    }
}

#[cfg(test)]
mod tests {
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
            Err(ConfigError::ConfigNotFound)
        ));
    }

    #[test]
    fn test_set_editor_error_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::with_root(temp_dir.path().to_path_buf());

        // Don't create a config file, verify we get an error
        assert!(matches!(
            config.set_editor("vim"),
            Err(ConfigError::ConfigNotFound)
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
            Err(ConfigError::InvalidConfig)
        ));
    }
}
