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

## TODO:
- [ ] Add support for `git add` with exclude patterns.
- [ ] Add `commit_message.md` generation from git staged changes.
- [ ] Add support for `git commit` with custom message from the `commit_message.md` file.
- [ ] Add support for `git push` with passed arguments.

## Usage

### `add-exclude`

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
