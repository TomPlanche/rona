use clap::{Parser, Subcommand, command};

use dialoguer::Select;
use glob::Pattern;

use crate::{
    git_related::{
        COMMIT_MESSAGE_FILE_PATH, COMMIT_TYPES, add_files, create_needed_files, get_status_files,
        git_commit, git_push, prepare_commit_msg,
    },
    my_clap_theme,
};

#[derive(Subcommand)]
enum Commands {
    /// Add and exclude subcommand
    /// Add all files to the git add command and exclude the files passed as positional arguments.
    #[command(short_flag = 'a', name = "add-with-exclude")]
    AddWithExclude {
        /// Patterns of files to exclude (supports glob patterns like "`node_modules`/*")
        #[arg(value_name = "PATTERNS")]
        exclude: Vec<String>,
    },

    /// Commit subcommand
    /// Directly commit the file with the text in `commit_message.md`.
    #[command(short_flag = 'c')]
    Commit {
        /// Additionnal arguments to pass to the commit command
        #[arg(value_name = "ARGS")]
        args: Vec<String>,

        /// Wheter to push the commit after committing
        #[arg(short = 'p', long = "push", default_value_t = false)]
        push: bool,
    },

    /// Generate subcommand
    /// Directly generate the `commit_message.md` file.
    #[command(short_flag = 'g')]
    Generate,

    /// List files from git status (for shell completion on the -a)
    #[command(short_flag = 'l')]
    ListStatus,

    /// Push subcommand
    /// Push to git repository.
    #[command(short_flag = 'p')]
    Push {
        /// Additionnal arguments to pass to the push command
        #[arg(value_name = "ARGS")]
        args: Vec<String>,
    },
}

#[derive(Parser)]
#[command(about = "Simple program that can:\n\
\t- Commit with the current 'commit_message.md' file text.\n\
\t- Generates the 'commit_message.md' file.")]
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

    match cli.command {
        Commands::AddWithExclude { exclude } => {
            let patterns: Vec<Pattern> = exclude
                .iter()
                .map(|p| Pattern::new(p).expect("Invalid glob pattern"))
                .collect();

            add_files(&patterns, cli.verbose)?;
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

            prepare_commit_msg(commit_type, cli.verbose)?;

            let editor = dotenv::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            // Open the commit message file in the editor of the user's choice
            let _ = std::process::Command::new(editor)
                .arg(COMMIT_MESSAGE_FILE_PATH)
                .spawn()
                .expect("Error opening the file in the editor")
                .wait();
        }
        Commands::Push { args } => {
            git_push(&args, cli.verbose)?;
        }
    }

    Ok(())
}
