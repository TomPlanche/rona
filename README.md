# 🔌 Rona

<h1 align="center">
    A simple CLI tool to help you with your git workflow.
</h1>

<p align="center">
  <a href="https://crates.io/crates/rona"><img src="https://img.shields.io/crates/v/rona.svg" alt="Crates.io Version"></a>
  <a href="https://docs.rs/rona"><img src="https://img.shields.io/docsrs/rona/latest" alt="Documentation"></a>
  <a href="https://github.com/TomPlanche/rona/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/rona" alt="License"></a>
  <a href="https://github.com/TomPlanche/rona/actions/workflows/rust.yaml"><img src="https://github.com/TomPlanche/rona/actions/workflows/rust.yaml/badge.svg" alt="Build Status"></a>
</p>

## Usage

### `add-exclude` (`-a`)

This command adds all files to the git add command and excludes the files that match the passed patterns.

Example:
```bash
rona add-exclude <pattern(s)>
# or
rona -a <pattern(s)>
```

Where `<pattern(s)>` are the patterns to exclude.

> [!CAUTION]
> The subtility of the patterns is that some terminals match the wildcard themselves.


Let's say you have this `git status --porcelain` output:

```
    M Cargo.lock
    M Cargo.toml
    M src/main.rs
?? LICENCE-APACHE
?? LICENCE-MIT
?? README.md
?? src/cli.rs
?? src/git_related.rs
?? src/lib.rs
?? src/utils.rs
```

For excluding all Rust files, you can use the pattern `*.rs`.

```bash
rona add-exclude "*.rs" # with quotes
# or use with wildcards
rona add-exclude **/*.rs # *.rs will not match any file and crash depending on the terminal, not my script's fault.
```

This will result in the following `git status --porcelain` output:

```
M  Cargo.lock
M  Cargo.toml
A  LICENCE-APACHE
A  LICENCE-MIT
A  README.md
    M src/main.rs
?? src/cli.rs
?? src/git_related.rs
?? src/lib.rs
?? src/utils.rs
```

### `commit` (`-c`)

This command commits all changes with a custom message from the `commit_message.md` file.

Example:
```bash
rona commit [extra args]
# or
rona -c [extra args]
```

This will commit all changes with the message from the `commit_message.md` file.

### `generate` (`-g`)

This command generates or updates the `commit_message.md` file with a template based on staged changes. It provides an interactive commit type selection and opens the file in your default editor.

Example:
```bash
rona generate
# or
rona -g
```

The command will:
1. Create `commit_message.md` and `.commitignore` files if they don't exist
2. Add both files to `.git/info/exclude`
3. Present an interactive selection of commit types (chore/feat/fix/test)
4. Generate commit message template with staged files
5. Open the message in your default editor (set via EDITOR env variable)
```

### `push` (`-p`)

This command pushes the committed changes to the remote repository.

Example:
```bash
rona push [extra args]
# or
rona -p [extra args]
```

This will push the committed changes to the remote repository.

### `list-status` (`-l`)

This command lists the status of the repository.

Example:
```bash
rona list-status
# or
rona -l
```

This is used with `fish` to autocomplete the `-a` command.
See the [`rona.fish`](./completions/rona.fish) fish completion script.
