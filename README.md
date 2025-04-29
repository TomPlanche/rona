# 🔌 Rona

<h1 align="center">
    A powerful CLI tool to streamline your Git workflow
</h1>

<p align="center">
  <a href="https://crates.io/crates/rona"><img src="https://img.shields.io/crates/v/rona.svg" alt="Crates.io Version"></a>
  <a href="https://docs.rs/rona"><img src="https://img.shields.io/docsrs/rona/latest" alt="Documentation"></a>
  <a href="https://github.com/TomPlanche/rona/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/rona" alt="License"></a>
  <a href="https://github.com/TomPlanche/rona/actions/workflows/rust.yaml"><img src="https://github.com/TomPlanche/rona/actions/workflows/rust.yaml/badge.svg" alt="Build Status"></a>
</p>

## Overview

Rona is a command-line interface tool designed to enhance your Git workflow with powerful features and intuitive commands. It simplifies common Git operations and provides additional functionality for managing commits, files, and repository status.

## Features

- 🚀 Intelligent file staging with pattern exclusion
- 📝 Structured commit message generation
- 🔄 Streamlined push operations
- 🎯 Interactive commit type selection
- 🛠 Fish shell completion support

## Installation

```bash
cargo install rona
```

## Quick Start

1. Add files excluding patterns:
```bash
rona -a "*.rs"  # Exclude all Rust files
```

2. Generate commit message:
```bash
rona -g  # Opens interactive commit type selector
```

3. Commit changes:
```bash
rona -c [ARGS] # Commits using message from commit_message.md
# Push changes to remote repository
rona -cp [ARGS] # here, the args will be passed to git commit
```

## Command Reference

### File Management

#### `add-with-exclude` (`-a`)
Add files to Git staging while excluding specified patterns.

```bash
rona add-with-exclude <pattern(s)>
# or
rona -a <pattern(s)>
```

**Example:**
```bash
rona -a "*.rs" "*.tmp"  # Exclude Rust and temporary files
```

### Commit Management

#### `generate` (`-g`)
Generate or update commit message template.

```bash
rona generate
# or
rona -g
```

**Features:**
- Creates `commit_message.md` and `.commitignore`
- Interactive commit type selection
- Automatic file change tracking
- Opens in default editor (set via EDITOR env variable)

#### `commit` (`-c`)
Commit changes using prepared message.

```bash
rona commit [extra args]
# or
rona -c [extra args]
```

### Repository Operations

#### `push` (`-p`)
Push committed changes to remote repository.

```bash
rona push [extra args]
# or
rona -p [extra args]
```

#### `list-status` (`-l`)
Display repository status (primarily for shell completion).

```bash
rona list-status
# or
rona -l
```

## Shell Integration

### Fish Shell Completion
Add the following to your Fish configuration:

```fish
source /path/to/rona/completions/rona.fish
```

## Development

### Requirements
- Rust 2021 edition or later
- Git 2.0 or later

### Building from Source
```bash
git clone https://github.com/TomPlanche/rona.git
cd rona
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Support

For bugs, questions, and discussions please use the [GitHub Issues](https://github.com/TomPlanche/rona/issues).
