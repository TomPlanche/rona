function __rona_status_files
    mygit -l
end

# Complete the -e/--exclude option for the -a/--add-and-exclude command
complete -c mygit -n '__fish_seen_subcommand_from -a'                 -s e -l exclude -xa '(__rona_status_files)'
complete -c mygit -n '__fish_seen_subcommand_from --add-with-exclude' -s e -l exclude -xa '(__rona_status_files)'
