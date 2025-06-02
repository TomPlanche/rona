# 🔌 Rona

<h1 align="center">
    A powerful CLI tool to streamline your Git workflow
</h1>

<p align="center">
  <a href="https://crates.io/crates/rona"><img src="https://img.shields.io/crates/v/rona.svg" alt="Crates.io Version"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_rona"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_rona&metric=alert_status" alt="SonarCloud Status"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_rona"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_rona&metric=sqale_rating" alt="SonarCloud SQALE Rating"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_rona"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_rona&metric=security_rating" alt="SonarCloud Security Rating"></a>
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
- 🛠 Multi-shell completion support (Bash, Fish, Zsh, PowerShell)

## Installation

```bash
cargo install rona
rona init [editor] # The editor to use for commit messages [vim, zed] (default: nano)
```

## Usage Examples

### Basic Workflow

1. Initialize Rona with your preferred editor:
```bash
# Initialize with Vim
rona init vim

# Initialize with Zed
rona init zed

# Initialize with default editor (nano)
rona init
```

2. Stage files while excluding specific patterns:
```bash
# Exclude Rust files
rona -a "*.rs"

# Exclude multiple file types
rona -a "*.rs" "*.tmp" "*.log"

# Exclude directories
rona -a "target/" "node_modules/"

# Exclude files with specific patterns
rona -a "test_*.rs" "*.test.js"
```

3. Generate and edit commit message:
```bash
# Generate commit message template (opens editor)
rona -g

# Interactive mode (input directly in terminal)
rona -g -i

# This will:
# 1. Open an interactive commit type selector
# 2. Create/update commit_message.md
# 3. Either open your configured editor (default) or prompt for simple input (-i)
```

4. Commit and push changes:
```bash
# Commit with the prepared message
rona -c

# Commit and push in one command
rona -c -p

# Commit with additional Git arguments
rona -c --no-verify

# Commit and push with specific branch
rona -c -p origin main
```

### Advanced Usage

#### Working with Multiple Branches

```bash
# Create and switch to a new feature branch
git checkout -b feature/new-feature
rona -a "*.rs"
rona -g
rona -c -p

# Switch back to main and merge
git checkout main
git merge feature/new-feature
```

#### Handling Large Changes

```bash
# Stage specific directories
rona -a "src/" "tests/"

# Exclude test files while staging
rona -a "src/" -e "test_*.rs"

# Stage everything except specific patterns
rona -a "*" -e "*.log" "*.tmp"
```

#### Using with CI/CD

```bash
# In your CI pipeline
rona init
rona -a "*"
rona -g
rona -c -p --no-verify
```

#### Shell Integration

```bash
# Fish shell
echo "function rona
    command rona \$argv
end" >> ~/.config/fish/functions/rona.fish

# Bash
echo 'alias rona="command rona"' >> ~/.bashrc
```

### Common Use Cases

1. **Feature Development**:
```bash
# Start new feature
git checkout -b feature/new-feature
rona -a "src/" "tests/"
rona -g  # Select 'feat' type
rona -c -p
```

2. **Bug Fixes**:
```bash
# Fix a bug
git checkout -b fix/bug-description
rona -a "src/"
rona -g  # Select 'fix' type
rona -c -p
```

3. **Code Cleanup**:
```bash
# Clean up code
git checkout -b chore/cleanup
rona -a "src/" -e "*.rs"
rona -g  # Select 'chore' type
rona -c -p
```

4. **Testing**:
```bash
# Add tests
git checkout -b test/add-tests
rona -a "tests/"
rona -g  # Select 'test' type
rona -c -p
```

5. **Quick Commits (Interactive Mode)**:
```bash
# Fast workflow without opening editor
rona -a "src/"
rona -g -i  # Select type and input message directly
rona -c -p
```

## Command Reference

### `add-with-exclude` (`-a`)
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

### `commit` (`-c`)
Commit changes using prepared message.

```bash
rona commit [extra args]
# or
rona -c [-p | --push] [extra args]
```

### `completion`
Generate shell completion scripts.

```bash
rona completion <shell>
```

**Supported shells:** `bash`, `fish`, `zsh`, `powershell`

**Example:**
```bash
rona completion fish > ~/.config/fish/completions/rona.fish
```

### `generate` (`-g`)
Generate or update commit message template.

```bash
rona generate [--interactive]
# or
rona -g [-i | --interactive]
```

**Features:**
- Creates `commit_message.md` and `.commitignore`
- Interactive commit type selection
- Automatic file change tracking
- **Interactive mode:** Input commit message directly in terminal (`-i` flag)
- **Editor mode:** Opens in configured editor (default behavior)

**Examples:**

```bash
# Standard mode: Opens commit type selector, then editor
rona -g

# Interactive mode: Input message directly in terminal
rona -g -i
```

**Interactive Mode Usage:**
When using the `-i` flag, Rona will:
1. Show the commit type selector (chore, feat, fix, test)
2. Prompt for a single commit message input
3. Generate a clean format: `[commit_nb] (type on branch) message`
4. Save directly to `commit_message.md` without file details

This is perfect for quick, clean commits without the detailed file listing.

### `init` (`-i`)
Initialize Rona configuration.

```bash
rona init [editor] # The editor to use for commit messages [vim, zed] (default: nano)
```

### `list-status` (`-l`)
Display repository status (primarily for shell completion).

```bash
rona list-status
# or
rona -l
```

### `push` (`-p`)
Push committed changes to remote repository.

```bash
rona push [extra args]
# or
rona -p [extra args]
```

### `set-editor` (`-s`)
Set the default editor for commit messages.

```bash
rona set-editor <editor> # The editor to use for commit messages [vim, zed], no default here
```

### `help` (`-h`)
Display help information.

```bash
rona help
# or
rona -h
```

## Shell Completion

Rona supports auto-completion for multiple shells using [`clap_complete`](https://docs.rs/clap_complete/latest/clap_complete/index.html).

### Generate Completions

Generate completion files for your shell:

```bash
# Generate completions for specific shell
rona completion fish    # Fish shell
rona completion bash    # Bash
rona completion zsh     # Zsh  
rona completion powershell  # PowerShell

# Save to file
rona completion fish > ~/.config/fish/completions/rona.fish
```

### Installation by Shell

**Fish Shell:**
```fish
# Copy to Fish completions directory
rona completion fish > ~/.config/fish/completions/rona.fish
```

**Bash:**
```bash
# Add to your .bashrc
rona completion bash >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
# Add to your .zshrc or save to a completions directory
rona completion zsh >> ~/.zshrc
```

**PowerShell:**
```powershell
# Add to your PowerShell profile
rona completion powershell | Out-File -Append $PROFILE
```

### Features

The completions include:
- All command and flag completions
- Git status file completion for `add-with-exclude` command (Fish only)
- Context-aware suggestions

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
