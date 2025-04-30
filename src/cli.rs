use std::process::Command;

use clap::{Parser, Subcommand, command};

use dialoguer::Select;
use glob::Pattern;

use crate::{
    config::Config,
    git_related::{
        COMMIT_MESSAGE_FILE_PATH, COMMIT_TYPES, create_needed_files, generate_commit_message,
        get_status_files, git_add_with_exclude_patterns, git_commit, git_push,
    },
    my_clap_theme,
};

/// CLI's commands
#[derive(Subcommand)]
enum Commands {
    /// Add all files to the `git add` command and exclude the patterns passed as positional arguments.
    #[command(short_flag = 'a', name = "add-with-exclude")]
    AddWithExclude {
        /// Patterns of files to exclude (supports glob patterns like `"node_modules/*"`)
        #[arg(value_name = "PATTERNS")]
        exclude: Vec<String>,
    },

    /// Directly commit the file with the text in `commit_message.md`.
    #[command(short_flag = 'c')]
    Commit {
        /// Additionnal arguments to pass to the commit command
        #[arg(value_name = "ARGS")]
        args: Vec<String>,

        /// Whether to push the commit after committing
        #[arg(short = 'p', long = "push", default_value_t = false)]
        push: bool,
    },

    /// Directly generate the `commit_message.md` file.
    #[command(short_flag = 'g')]
    Generate,

    /// Initialize the rona configuration file.
    #[command(short_flag = 'i', name = "init")]
    Initialize {
        /// Editor to use for the commit message.
        #[arg(short = 'e', long = "editor", default_value_t = String::from("nano"))]
        editor: String,
    },

    /// List files from git status (for shell completion on the -a)
    #[command(short_flag = 'l')]
    ListStatus,

    /// Push to a git repository.
    #[command(short_flag = 'p')]
    Push {
        /// Additionnal arguments to pass to the push command
        #[arg(value_name = "ARGS")]
        args: Vec<String>,
    },

    /// Set the editor to use for editing the commit message.
    #[command(short_flag = 's', name = "set-editor")]
    Set {
        /// The editor to use for the commit message
        #[arg(value_name = "EDITOR")]
        editor: String,
    },
}

#[derive(Parser)]
#[command(about = "Simple program that can:\n\
\t- Commit with the current 'commit_message.md' file text.\n\
\t- Generate the 'commit_message.md' file.\n\
\t- Push to git repository.\n\
\t- Push to git repository.")]
#[command(author = "Tom Planche <tomplanche@proton.me>")]
#[command(help_template = "{about}\nMade by: {author}\n\nUSAGE:\n{usage}\n\n{all-args}\n")]
#[command(name = "rona")]
pub struct Cli {
    /// Commands
    #[command(subcommand)]
    command: Commands,

    /// Verbose
    /// Optional 'verbose' argument. Only works if a subcommand is passed.
    /// If passed, it will print more information about the operation.
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

/// # `run`
/// Runs the program.
///
/// ## Panics
/// * If the given glob patterns are invalid.
///
/// ## Errors
/// Returns an error if the command fails.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = Config::new()?;

    match cli.command {
        Commands::AddWithExclude { exclude } => {
            let patterns: Vec<Pattern> = exclude
                .iter()
                .map(|p| Pattern::new(p).expect("Invalid glob pattern"))
                .collect();

            git_add_with_exclude_patterns(&patterns, cli.verbose)?;
        }
        Commands::Commit { args, push } => {
            git_commit(&args, cli.verbose)?;

            if push {
                git_push(&Vec::new(), cli.verbose)?;
            }
        }
        Commands::ListStatus => {
            let files = get_status_files()?;

            // Print each file on a new line for fish shell completion
            for file in files {
                println!("{file}");
            }
        }
        Commands::Generate => {
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
        Commands::Initialize { editor } => {
            config.create_config_file(&editor)?;
        }
        Commands::Push { args } => {
            git_push(&args, cli.verbose)?;
        }
        Commands::Set { editor } => {
            config.set_editor(&editor)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_add_with_exclude_command() {
        let args = vec!["rona", "-a", "*.txt"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::AddWithExclude { exclude } => {
                assert_eq!(exclude, vec!["*.txt"]);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_command() {
        let args = vec!["rona", "-c"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Commit { args, push } => {
                assert!(!push);
                assert!(args.is_empty());
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_command_with_push_no_args() {
        let args = vec!["rona", "-c", "--push"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Commit { args, push } => {
                assert!(push);
                assert!(args.is_empty());
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_command_with_args() {
        let args = vec!["rona", "-c", "message"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Commit { args, push } => {
                assert!(!push);
                assert_eq!(args, vec!["message"]);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_commit_command_with_push_and_args() {
        let args = vec!["rona", "-c", "--push", "message"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Commit { args, push } => {
                assert!(push);
                assert_eq!(args, vec!["message"]);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_generate_command() {
        let args = vec!["rona", "-g"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Generate => (),
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_list_status_command() {
        let args = vec!["rona", "-l"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ListStatus => (),
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_push_command() {
        let args = vec!["rona", "-p", "--", "--force"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Push { args } => {
                assert_eq!(args, vec!["--force"]);
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
    fn test_cli_parsing() {
        let args = vec!["rona", "-a", "*.rs"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::AddWithExclude { exclude } => {
                assert_eq!(exclude, vec!["*.rs"]);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let args = vec!["rona", "-v", "-a", "*.rs"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
    }
}
