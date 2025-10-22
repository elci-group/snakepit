# Contributing to Snakepit

Thank you for your interest in contributing to Snakepit! Here are some guidelines to help you get started.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/snakepit.git`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Test your changes: `cargo test`
6. Commit with descriptive messages: `git commit -am "Add feature: my feature"`
7. Push to your fork: `git push origin feature/my-feature`
8. Create a Pull Request to the main repository

## Development Setup

```bash
# Clone the repository
git clone https://github.com/adminx/snakepit.git
cd snakepit

# Build the project
cargo build --release

# Run tests
cargo test

# Run snakepit
./target/release/snakepit --help
```

## Code Style

- Use Rust conventions and idiomatic patterns
- Run `cargo fmt` to format code
- Run `cargo clippy` for linting suggestions
- Keep functions focused and well-documented

## Testing

- Add tests for new functionality
- Ensure all tests pass: `cargo test`
- Test across platforms if possible (Linux, macOS, Windows)

## Commit Messages

Use clear, descriptive commit messages:

- ✨ `feat:` New features
- 🐛 `fix:` Bug fixes
- 📚 `docs:` Documentation changes
- 🎨 `style:` Code style changes
- ♻️ `refactor:` Code refactoring
- ✅ `test:` Test additions or changes

Example:
```
feat: Add zsh shell integration for auto-install

- Support zsh completion
- Add auto-install hook for zsh
- Update shell detection logic
```

## Pull Request Guidelines

1. Update the README.md with new features or changes
2. Update the CHANGELOG.md
3. Ensure CI/CD passes
4. Request reviews from maintainers
5. Keep PRs focused on a single feature or fix

## Reporting Issues

When reporting bugs, please include:

- Snakepit version: `snakepit --version`
- Operating system and version
- Python version(s) used
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error logs (if applicable)

## Feature Requests

For new features:

1. Check existing issues and PRs
2. Describe the use case and expected behavior
3. Provide examples if possible
4. Discuss performance and compatibility implications

## License

By contributing to Snakepit, you agree that your contributions will be licensed under its MIT License.

## Questions?

- Open an issue with the `question` label
- Check existing documentation
- Review closed issues for similar questions

Thanks for contributing! 🐍
