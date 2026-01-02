# Contributing to AI Translate Tool

Thank you for your interest in contributing to AI Translate Tool! This document provides guidelines for contributing to the project.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- On Linux: X11 development libraries

#### Linux Setup

```bash
sudo apt-get update
sudo apt-get install -y \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev \
    libfontconfig1-dev
```

### Building the Project

```bash
# Clone the repository
git clone https://github.com/AnlangA/ai-T.git
cd ai-T

# Build the project
cargo build

# Run the application
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

## Development Workflow

### Code Style

We follow the official Rust style guidelines. Please run the following before submitting:

```bash
# Format your code
cargo fmt

# Check for common mistakes
cargo clippy

# Run all tests
cargo test
```

### Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting a PR
- Test coverage should not decrease with new changes

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test config
```

### Commit Messages

Write clear and descriptive commit messages:

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 50 characters
- Add a blank line before a detailed description
- Reference issue numbers when applicable

Example:
```
Add support for custom API endpoints

- Allow users to configure custom API base URLs
- Add validation for URL format
- Update configuration UI

Fixes #123
```

## Pull Request Process

1. **Fork the repository** and create a new branch from `main`
2. **Make your changes** following the code style guidelines
3. **Add or update tests** as needed
4. **Update documentation** if you're changing functionality
5. **Run all checks**:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```
6. **Submit a PR** with a clear description of your changes

### PR Guidelines

- One feature or fix per PR
- Include tests for new functionality
- Update README if adding user-facing features
- Keep PRs focused and reasonably sized
- Respond to review feedback promptly

## Code Review

All submissions require review. We use GitHub pull requests for this purpose.

Reviewers will check for:
- Code quality and style
- Test coverage
- Documentation updates
- Performance implications
- Security considerations

## Reporting Issues

### Bug Reports

Include:
- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- System information (OS, Rust version)
- Relevant logs (use `RUST_LOG=debug`)

### Feature Requests

Include:
- Clear description of the feature
- Use cases and benefits
- Potential implementation approach
- Any relevant examples or mockups

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Accept constructive criticism gracefully
- Focus on what's best for the community

### Unacceptable Behavior

- Harassment or discriminatory language
- Personal attacks or trolling
- Publishing others' private information
- Other conduct that would be inappropriate in a professional setting

## Questions?

Feel free to:
- Open an issue for discussion
- Ask questions in your PR
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
