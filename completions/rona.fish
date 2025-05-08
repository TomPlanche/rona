# Helper function to get git status files
function __rona_status_files
    rona -l
end

# Helper function to get git commit arguments
function __rona_commit_args
    echo --amend\t"Amend the previous commit"
    echo --no-edit\t"Reuse the previous commit message"
    echo --allow-empty\t"Allow empty commits"
    echo --no-verify\t"Skip pre-commit and commit-msg hooks"
    echo --signoff\t"Add Signed-off-by line"
    echo --reset-author\t"Reset author"
    echo --squash\t"Squash with previous commit"
    echo --fixup\t"Fixup with previous commit"
end

# Helper function to get git push arguments
function __rona_push_args
    echo --force\t"Force push"
    echo --force-with-lease\t"Force push if remote was not modified"
    echo -u\t"Set upstream tracking"
    echo --set-upstream\t"Set upstream tracking"
    echo --tags\t"Push tags"
    echo --delete\t"Delete remote branch"
    echo --all\t"Push all branches"
    echo --prune\t"Prune remote branches"
    echo --no-verify\t"Skip pre-push hook"
end

# Helper function to get git remotes
function __rona_git_remotes
    git remote
end

# Helper function to get git branches
function __rona_git_branches
    git branch --format "%(refname:short)"
end

# Base command
complete -c rona -f

# Global flags
complete -c rona -s v -l version -d "Print version information"
complete -c rona -s h -l help -d "Print help information"

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

# commit: Add push option and git commit arguments
complete -c rona -n '__fish_seen_subcommand_from commit -c' -l push -s p -d "Push after commit"
complete -c rona -n '__fish_seen_subcommand_from commit -c' -xa '(__rona_commit_args)'

# push: Add git push arguments, remotes, and branches
complete -c rona -n '__fish_seen_subcommand_from push -p' -xa '(__rona_push_args)'
complete -c rona -n '__fish_seen_subcommand_from push -p' -a '(__rona_git_remotes)' -d "Remote repository"
complete -c rona -n '__fish_seen_subcommand_from push -p' -a '(__rona_git_branches)' -d "Branch name"
