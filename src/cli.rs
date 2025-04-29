use clap::{Parser, Subcommand, command};

use glob::Pattern;

use crate::git_related::{add_files, commit};

#[derive(Subcommand, Debug)] // TODO: Remove Debug
enum Commands {
    /// Add and exclude subcommand
    /// Add all files to the git add command and exclude the files passed as positional arguments.
    #[command(short_flag = 'a', name = "add-exclude")]
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
    },
}

#[derive(Parser)]
#[command(about = "Simple program that can:\n\
\t- Commit with the current 'commit_message.md' file text.\n\
\t- Generates the 'commit_message.md' file.")]
#[command(author = "Tom P. <tomplanche@icloud.com>")]
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
        Commands::Commit { args } => {
            commit(&args, cli.verbose)?;
        }
    }

    // match cli.command {
    //     Commands::AddAndExclude { exclude } => {
    //         println!("Adding all files and excluding: {:?}", exclude);
    //     }
    // }

    Ok(())
}
