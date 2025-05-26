# Contributing to Rona

Thank you for your interest in contributing to Rona! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git 2.0 or later
- A text editor or IDE with Rust support

### Development Setup

1. **Clone the repository**:
```bash
git clone https://github.com/TomPlanche/rona.git
cd rona
```

2. **Install development tools**:
```bash
# Install cargo-audit for security checking
cargo install cargo-audit

# Install cargo-outdated for dependency management
cargo install cargo-outdated

# Install pre-commit hooks (optional but recommended)
cargo install hooksmith
hooksmith install
```

3. **Build and test**:
```bash
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## 📁 Project Structure

```
src/
├── main.rs              # Application entry point
├── cli.rs               # Command-line interface and argument parsing
├── config.rs            # Configuration management
├── errors.rs            # Error types and handling
├── git_related.rs       # Legacy git operations (being refactored)
├── git/                 # New modular git operations
│   ├── mod.rs          # Git module exports
│   ├── utils.rs        # Git utility functions
│   ├── commit.rs       # Commit operations (planned)
│   ├── operations.rs   # Git operations (planned)
│   └── status.rs       # Status operations (planned)
├── my_clap_theme.rs    # Custom CLI themes
├── performance.rs      # Performance utilities
└── utils.rs            # General utility functions
```

## 🛠 Development Guidelines

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` to catch common issues
- Write comprehensive documentation for public APIs
- Include examples in documentation where helpful

### Testing

- Write unit tests for all new functionality
- Include integration tests for CLI commands
- Ensure all tests pass before submitting PR
- Aim for good test coverage of critical paths

### Error Handling

- Use the custom error types defined in `errors.rs`
- Provide helpful error messages with context
- Include suggestions for fixing errors when possible
- Use `thiserror` for structured error handling

### Performance

- Minimize string allocations where possible
- Use `Cow<str>` for borrowed/owned string flexibility
- Batch operations when dealing with multiple files
- Profile performance-critical code paths

## 🔄 Development Workflow

### Making Changes

1. **Create a feature branch**:
```bash
git checkout -b feature/your-feature-name
```

2. **Make your changes**:
   - Write code following the guidelines above
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**:
```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

4. **Commit your changes**:
```bash
# Use rona itself for commits!
rona -a "*.rs"  # Add files excluding patterns
rona -g         # Generate commit message
rona -c         # Commit changes
```

### Pull Request Process

1. **Ensure CI passes**: All tests and checks must pass
2. **Update documentation**: Include relevant documentation updates
3. **Write clear PR description**: Explain what changes and why
4. **Request review**: Tag maintainers for review

### Commit Message Format

We follow conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(cli): add dry-run mode for all commands`
- `fix(git): handle empty repository error gracefully`
- `docs(readme): update installation instructions`

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test cli::tests
```

### Writing Tests

- Place unit tests in the same file as the code being tested
- Use descriptive test names that explain what is being tested
- Test both success and error cases
- Use `tempfile` for tests that need temporary files/directories

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_function_name_success_case() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = function_name(input);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "expected output");
    }
}
```

## 📚 Documentation

### Code Documentation

- Use `///` for public API documentation
- Include examples in doc comments when helpful
- Document error conditions and return values
- Use `#[must_use]` for functions whose return value should be used

### README Updates

- Keep installation instructions current
- Update feature list when adding new functionality
- Include new examples for significant features
- Maintain the command reference section

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment information**:
   - OS and version
   - Rust version (`rustc --version`)
   - Rona version (`rona --version`)

2. **Steps to reproduce**:
   - Exact commands run
   - Expected vs actual behavior
   - Any error messages

3. **Additional context**:
   - Git repository state
   - Configuration files
   - Relevant logs

## 💡 Feature Requests

For feature requests, please:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** and problem being solved
3. **Propose a solution** if you have ideas
4. **Consider implementation complexity** and maintenance burden

## 🔒 Security

- Report security vulnerabilities privately to the maintainers
- Run `cargo audit` regularly to check for known vulnerabilities
- Keep dependencies updated
- Follow secure coding practices

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Code Review**: For feedback on implementation approaches

## 🎉 Recognition

Contributors will be recognized in:
- The project README
- Release notes for significant contributions
- GitHub contributor graphs

Thank you for contributing to Rona! 🚀 