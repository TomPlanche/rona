# Helper function to get git status files
function __rona_status_files
    rona -l
end

# Base command
complete -c rona -f

# Subcommands
complete -c rona -n "__fish_use_subcommand" -a "add-with-exclude commit generate init list-status push set-editor"

# Short flags for subcommands
complete -c rona -n "__fish_use_subcommand" -s a -l add-with-exclude -d "Add with exclude"
complete -c rona -n "__fish_use_subcommand" -s c -l commit -d "Commit"
complete -c rona -n "__fish_use_subcommand" -s g -l generate -d "Generate commit message"
complete -c rona -n "__fish_use_subcommand" -s i -l init -d "Initialize configuration"
complete -c rona -n "__fish_use_subcommand" -s l -l list-status -d "List files from git status"
complete -c rona -n "__fish_use_subcommand" -s p -l push -d "Push to repository"
complete -c rona -n "__fish_use_subcommand" -s s -l set-editor -d "Set editor"

# Global verbose option
complete -c rona -n "__fish_seen_subcommand_from add-with-exclude commit generate init list-status push set-editor" -s v -l verbose -d "Show verbose output"

# Command-specific completions
# add-with-exclude: Complete with git status files
complete -c rona -n '__fish_seen_subcommand_from add-with-exclude -a' -xa '(__rona_status_files)'

# commit: Add push option
complete -c rona -n '__fish_seen_subcommand_from commit -c' -l push -s p -d "Push after commit"
