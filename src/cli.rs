//! Command Line Interface (CLI) Module for Rona
//!
//! This module handles all command-line interface functionality for Rona, including
//! - Command parsing and execution
//! - Subcommand implementations
//! - CLI argument handling
//!
//! # Commands
//!
//! The CLI supports several commands:
//! - `add-with-exclude`: Add files to git while excluding specified patterns
//! - `commit`: Commit changes using the commit message from `commit_message.md`
//! - `generate`: Generate a new commit message file
//! - `init`: Initialize Rona configuration
//! - `list-status`: List git status files (for shell completion)
//! - `push`: Push changes to remote repository
//! - `set-editor`: Configure the editor for commit messages
//!
//! # Features
//!
//! - Supports verbose mode for detailed operation logging
//! - Supports dry-run mode for previewing changes
//! - Integrates with git commands
//! - Provides shell completion capabilities
//! - Handles configuration management
//!

use crate::{
    config::Config,
    errors::Result,
    git_related::{
        COMMIT_MESSAGE_FILE_PATH, COMMIT_TYPES, create_needed_files, generate_commit_message,
        get_status_files, git_add_with_exclude_patterns, git_commit, git_push,
    },
    my_clap_theme,
};
use clap::{Command as ClapCommand, CommandFactory, Parser, Subcommand, ValueHint, command};
use clap_complete::{Shell, generate};
use dialoguer::Select;
use glob::Pattern;
use std::{io, process::Command};

/// CLI's commands
#[derive(Subcommand)]
pub(crate) enum CliCommand {
    /// Add all files to the `git add` command and exclude the patterns passed as positional arguments.
    #[command(short_flag = 'a', name = "add-with-exclude")]
    AddWithExclude {
        /// Patterns of files to exclude (supports glob patterns like `"node_modules/*"`)
        #[arg(value_name = "PATTERNS", value_hint = ValueHint::AnyPath)]
        to_exclude: Vec<String>,

        /// Show what would be added without actually adding files
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Directly commit the file with the text in `commit_message.md`.
    #[command(short_flag = 'c')]
    Commit {
        /// Whether to push the commit after committing
        #[arg(short = 'p', long = "push", default_value_t = false)]
        push: bool,

        /// Show what would be committed without actually committing
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Additional arguments to pass to the commit command
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Generate shell completions for your shell
    #[command(name = "completion")]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Directly generate the `commit_message.md` file.
    #[command(short_flag = 'g')]
    Generate {
        /// Show what would be generated without creating files
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Initialize the rona configuration file.
    #[command(short_flag = 'i', name = "init")]
    Initialize {
        /// Editor to use for the commit message.
        #[arg(default_value_t = String::from("nano"))]
        editor: String,

        /// Show what would be initialized without creating files
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// List files from git status (for shell completion on the -a)
    #[command(short_flag = 'l')]
    ListStatus,

    /// Push to a git repository.
    #[command(short_flag = 'p')]
    Push {
        /// Show what would be pushed without actually pushing
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Additionnal arguments to pass to the push command
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Set the editor to use for editing the commit message.
    #[command(short_flag = 's', name = "set-editor")]
    Set {
        /// The editor to use for the commit message
        #[arg(value_name = "EDITOR")]
        editor: String,

        /// Show what would be changed without modifying config
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Parser)]
#[command(about = "Simple program that can:\n\
\t- Commit with the current 'commit_message.md' file text.\n\
\t- Generate the 'commit_message.md' file.\n\
\t- Push to git repository.\n\
\t- Add files with pattern exclusion.\n\
\nAll commands support --dry-run to preview changes.")]
#[command(author = "Tom Planche <tomplanche@proton.me>")]
#[command(help_template = "{about}\nMade by: {author}\n\nUSAGE:\n{usage}\n\n{all-args}\n")]
#[command(name = "rona")]
#[command(version)]
pub(crate) struct Cli {
    /// Commands
    #[command(subcommand)]
    pub(crate) command: CliCommand,

    /// Verbose output - show detailed information about operations
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Use custom config file path instead of default
    #[arg(long, value_name = "PATH")]
    config: Option<String>,
}

/// Build the CLI command structure for generating completions
fn build_cli() -> ClapCommand {
    Cli::command()
}

/// Print custom fish shell completions that enhance the auto-generated ones
fn print_fish_custom_completions() {
    println!();
    println!("# === CUSTOM RONA COMPLETIONS ===");
    println!("# Helper function to get git status files");
    println!("function __rona_status_files");
    println!("    rona -l");
    println!("end");
    println!();
    println!("# Command-specific completions");
    println!("# add-with-exclude: Complete with git status files");
    println!(
        "complete -c rona -n '__fish_seen_subcommand_from add-with-exclude -a' -xa '(__rona_status_files)'"
    );
}

/// Runs the program.
///
/// # Panics
/// * If the given glob patterns are invalid.
///
/// # Errors
/// * Return an error if the command fails.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::new()?;

    match cli.command {
        CliCommand::AddWithExclude {
            to_exclude: exclude,
            dry_run,
        } => {
            let patterns: Vec<Pattern> = exclude
                .iter()
                .map(|p| Pattern::new(p).expect("Invalid glob pattern"))
                .collect();

            git_add_with_exclude_patterns(&patterns, cli.verbose, dry_run)?;
        }
        CliCommand::Commit {
            args,
            push,
            dry_run,
        } => {
            git_commit(&args, cli.verbose, dry_run)?;

            if push {
                git_push(&args, cli.verbose, dry_run)?;
            }
        }
        CliCommand::Completion { shell } => {
            let mut cmd = build_cli();
            generate(shell, &mut cmd, "rona", &mut io::stdout());

            // Add custom completions for fish shell
            if matches!(shell, Shell::Fish) {
                print_fish_custom_completions();
            }
        }
        CliCommand::Generate { dry_run } => {
            if dry_run {
                println!("Would create files: commit_message.md, .commitignore");
                println!("Would add files to .git/info/exclude");
                return Ok(());
            }

            create_needed_files()?;

            let commit_type =
                COMMIT_TYPES[Select::with_theme(&my_clap_theme::ColorfulTheme::default())
                    .default(0)
                    .items(&COMMIT_TYPES)
                    .interact()
                    .unwrap()];

            generate_commit_message(commit_type, cli.verbose)?;

            let editor = config.get_editor()?;

            Command::new(editor)
                .arg(COMMIT_MESSAGE_FILE_PATH)
                .spawn()
                .expect("Failed to spawn editor")
                .wait()
                .expect("Failed to wait for editor");
        }
        CliCommand::Initialize { editor, dry_run } => {
            if dry_run {
                println!("Would create config file with editor: {editor}");
                return Ok(());
            }
            config.create_config_file(&editor)?;
        }
        CliCommand::ListStatus => {
            let files = get_status_files()?;

            // Print each file on a new line for fish shell completion
            for file in files {
                println!("{file}");
            }
        }
        CliCommand::Push { args, dry_run } => {
            git_push(&args, cli.verbose, dry_run)?;
        }
        CliCommand::Set { editor, dry_run } => {
            if dry_run {
                println!("Would set editor to: {editor}");
                return Ok(());
            }
            config.set_editor(&editor)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    use clap::Parser;

    // === ADD COMMAND TESTS ===

    #[test]
    fn test_add_basic() {
        let args = vec!["rona", "-a"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::AddWithExclude {
                to_exclude: exclude,
                dry_run,
            } => {
                assert!(exclude.is_empty());
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_add_single_pattern() {
        let args = vec!["rona", "-a", "*.txt"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::AddWithExclude {
                to_exclude: exclude,
                dry_run,
            } => {
                assert_eq!(exclude, vec!["*.txt"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_add_multiple_patterns() {
        let args = vec!["rona", "-a", "*.txt", "*.log", "target/*"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::AddWithExclude {
                to_exclude: exclude,
                dry_run,
            } => {
                assert_eq!(exclude, vec!["*.txt", "*.log", "target/*"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_add_with_long_name() {
        let args = vec!["rona", "add-with-exclude", "*.txt"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::AddWithExclude {
                to_exclude: exclude,
                dry_run,
            } => {
                assert_eq!(exclude, vec!["*.txt"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    // === COMMIT COMMAND TESTS ===

    #[test]
    fn test_commit_basic() {
        let args = vec!["rona", "-c"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(!push);
                assert!(args.is_empty());
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_push_flag() {
        let args = vec!["rona", "-c", "--push"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(push);
                assert!(args.is_empty());
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_message() {
        let args = vec!["rona", "-c", "Regular commit message"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(!push);
                assert_eq!(args, vec!["Regular commit message"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_git_flag() {
        let args = vec!["rona", "-c", "--amend"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(!push);
                assert_eq!(args, vec!["--amend"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_multiple_git_flags() {
        let args = vec!["rona", "-c", "--amend", "--no-edit"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(!push);
                assert_eq!(args, vec!["--amend", "--no-edit"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_push_and_git_flags() {
        let args = vec!["rona", "-c", "--push", "--amend", "--no-edit"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(push);
                assert_eq!(args, vec!["--amend", "--no-edit"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_message_and_push() {
        let args = vec!["rona", "-c", "--push", "Commit message"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(push);
                assert_eq!(args, vec!["Commit message"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    // === PUSH COMMAND TESTS ===

    #[test]
    fn test_push_basic() {
        let args = vec!["rona", "-p"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Push { args, dry_run } => {
                assert!(args.is_empty());
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_push_with_force() {
        let args = vec!["rona", "-p", "--force"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Push { args, dry_run } => {
                assert_eq!(args, vec!["--force"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_push_with_multiple_args() {
        let args = vec!["rona", "-p", "--force", "--set-upstream", "origin", "main"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Push { args, dry_run } => {
                assert_eq!(args, vec!["--force", "--set-upstream", "origin", "main"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_push_with_remote_and_branch() {
        let args = vec!["rona", "-p", "origin", "feature/branch"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Push { args, dry_run } => {
                assert_eq!(args, vec!["origin", "feature/branch"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_push_with_upstream_tracking() {
        let args = vec!["rona", "-p", "-u", "origin", "main"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Push { args, dry_run } => {
                assert_eq!(args, vec!["-u", "origin", "main"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    // === GENERATE COMMAND TESTS ===

    #[test]
    fn test_generate_command() {
        let args = vec!["rona", "-g"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Generate { dry_run } => {
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    // === LIST STATUS COMMAND TESTS ===

    #[test]
    fn test_list_status_command() {
        let args = vec!["rona", "-l"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::ListStatus => (),
            _ => panic!("Wrong command parsed"),
        }
    }

    // === INITIALIZE COMMAND TESTS ===

    #[test]
    fn test_init_default_editor() {
        let args = vec!["rona", "-i"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Initialize { editor, dry_run } => {
                assert_eq!(editor, "nano");
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_init_custom_editor() {
        let args = vec!["rona", "-i", "zed"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Initialize { editor, dry_run } => {
                assert_eq!(editor, "zed");
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    // === SET EDITOR COMMAND TESTS ===

    #[test]
    fn test_set_editor() {
        let args = vec!["rona", "-s", "vim"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Set { editor, dry_run } => {
                assert_eq!(editor, "vim");
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_set_editor_with_spaces() {
        let args = vec!["rona", "-s", "\"Visual Studio Code\""];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Set { editor, dry_run } => {
                assert_eq!(editor, "\"Visual Studio Code\"");
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_set_editor_with_path() {
        let args = vec!["rona", "-s", "/usr/bin/vim"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Set { editor, dry_run } => {
                assert_eq!(editor, "/usr/bin/vim");
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    // === VERBOSE FLAG TESTS ===

    #[test]
    fn test_verbose_with_commit() {
        let args = vec!["rona", "-v", "-c"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_verbose_with_push() {
        let args = vec!["rona", "-v", "-p"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_verbose_long_form() {
        let args = vec!["rona", "--verbose", "-c"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
    }

    // === EDGE CASES AND ERROR TESTS ===

    #[test]
    fn test_commit_flag_order_sensitivity() {
        let args = vec!["rona", "-c", "--amend", "--push"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(!push); // --push should be treated as git arg
                assert_eq!(args, vec!["--amend", "--push"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_with_similar_looking_args() {
        let args = vec!["rona", "-c", "--push-to-upstream"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(!push);
                assert_eq!(args, vec!["--push-to-upstream"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let args = vec!["rona", "--invalid"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_missing_required_value() {
        let args = vec!["rona", "-s"]; // missing editor value
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_complex_command_combination() {
        let args = vec!["rona", "-v", "-c", "--push", "--amend", "--no-edit"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        match cli.command {
            CliCommand::Commit {
                args,
                push,
                dry_run,
            } => {
                assert!(push);
                assert_eq!(args, vec!["--amend", "--no-edit"]);
                assert!(!dry_run);
            }
            _ => panic!("Wrong command parsed"),
        }
    }
}
