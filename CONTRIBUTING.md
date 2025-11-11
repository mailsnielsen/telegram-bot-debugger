# Contributing to Telegram Bot Debugger

Thank you for your interest in contributing to Telegram Bot Debugger! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Documentation](#documentation)
- [Issue Guidelines](#issue-guidelines)

## Code of Conduct

- Be respectful and constructive in all interactions
- Focus on what is best for the project and community
- Show empathy towards other community members
- Accept constructive criticism gracefully

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report:
- Check the [existing issues](https://github.com/mailsnielsen/telegram-bot-debugger/issues) to avoid duplicates
- Try the latest release to see if the bug is already fixed
- Collect as much information as possible about the problem

When submitting a bug report, include:
- **Description**: Clear and concise description of the bug
- **Steps to Reproduce**: Detailed steps to reproduce the behavior
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Rust version (`rustc --version`), terminal emulator
- **Logs/Screenshots**: If applicable, add terminal output or screenshots

### Suggesting Features

Feature requests are welcome! When suggesting a feature:
- Check if it's already been suggested in existing issues
- Clearly describe the problem this feature would solve
- Explain how it would benefit users
- Consider if it fits the project's scope and goals

### Code Contributions

We welcome pull requests for:
- Bug fixes
- New features
- Performance improvements
- Documentation improvements
- Test coverage improvements
- Code refactoring

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git
- A Telegram bot token (from [@BotFather](https://t.me/botfather))

### Setup Steps

1. **Fork the repository** on GitHub

2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/telegram-bot-debugger.git
   cd telegram-bot-debugger
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/mailsnielsen/telegram-bot-debugger.git
   ```

4. **Build the project**:
   ```bash
   cargo build
   ```

5. **Run tests** to ensure everything works:
   ```bash
   cargo test
   ```

### Project Structure

```
src/
├── main.rs           # Entry point
├── app/              # Application state management
├── telegram/         # Telegram API client, types, and updates
├── ui/               # Terminal UI components (ratatui)
├── storage/          # Cache and persistence
└── analytics/        # Statistics and analytics
```

## Code Standards

### Rust Style Guidelines

We follow standard Rust conventions and best practices:

#### Formatting

- **Always run `cargo fmt` before committing**
- The project uses default `rustfmt` settings
- No manual formatting adjustments needed

```bash
cargo fmt
```

#### Linting

- **Run `cargo clippy` and fix all warnings**
- Clippy helps catch common mistakes and improve code quality
- Only use `#[allow(clippy::lint_name)]` with clear justification in comments

```bash
cargo clippy -- -D warnings
```

#### Naming Conventions

- `snake_case` for functions, variables, and modules
- `CamelCase` for types, traits, and enums
- `SCREAMING_SNAKE_CASE` for constants
- Use descriptive names: `is_message_from_forum` not `check()`

#### Code Quality

- **DRY and SOLID principles**: Reuse existing code where applicable
- **Error handling**: Use `Result<T, E>` and avoid `.unwrap()` in production code
- **Documentation**: Document public APIs with `///` doc comments
- **Ownership**: Prefer borrowing over cloning when possible
- **Iterators**: Use iterator methods over manual loops when appropriate

### Example

```rust
/// Parses a Telegram chat ID from a string.
///
/// # Arguments
///
/// * `input` - The string to parse (e.g., "-1001234567890")
///
/// # Returns
///
/// Returns `Ok(i64)` if parsing succeeds, or `Err(ParseError)` on failure.
///
/// # Examples
///
/// ```
/// use telegram_bot_debugger::parse_chat_id;
///
/// let chat_id = parse_chat_id("-1001234567890")?;
/// assert_eq!(chat_id, -1001234567890);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_chat_id(input: &str) -> Result<i64, ParseError> {
    input.trim()
        .parse::<i64>()
        .map_err(|_| ParseError::InvalidChatId)
}
```

## Testing

### Running Tests

The project maintains 125+ tests covering unit tests, integration tests, and property-based tests.

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests for a specific module
cargo test telegram::
```

### Writing Tests

- **Add tests for all new features**
- **Test edge cases and error conditions**
- **Keep tests isolated and deterministic**
- **Use descriptive test names** that explain what is being tested

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chat_id_valid_private() {
        let result = parse_chat_id("123456789");
        assert_eq!(result.unwrap(), 123456789);
    }

    #[test]
    fn test_parse_chat_id_valid_group() {
        let result = parse_chat_id("-1001234567890");
        assert_eq!(result.unwrap(), -1001234567890);
    }

    #[test]
    fn test_parse_chat_id_invalid() {
        let result = parse_chat_id("not_a_number");
        assert!(result.is_err());
    }
}
```

### Test Coverage

- Aim to maintain or improve test coverage
- Critical paths must be tested: happy path, error cases, edge cases
- Integration tests should cover main user workflows

## Submitting Changes

### Branch Strategy

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. **Keep your branch up to date**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

### Commit Guidelines

Write clear, concise commit messages:

```
<type>: <short summary in present tense>

<optional detailed description>

<optional footer with issue references>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring without behavior change
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples**:
```
feat: add support for inline keyboard buttons

Implements parsing and display of inline keyboard buttons
in the Raw JSON view. Handles all button types including
URL, callback_data, and switch_inline_query.

Fixes #42
```

```
fix: handle empty update arrays in monitor

Prevents panic when getUpdates returns empty array.
```

### Pull Request Process

1. **Ensure your code passes all checks**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   cargo build --release
   ```

2. **Update documentation** if needed:
   - Update README.md for user-facing changes
   - Add/update doc comments for code changes
   - Update CHANGELOG.md (if present)

3. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request** on GitHub:
   - Use a clear, descriptive title
   - Reference related issues (e.g., "Fixes #123")
   - Describe what changes were made and why
   - Add screenshots for UI changes
   - List any breaking changes

5. **Respond to review feedback**:
   - Address all comments constructively
   - Push additional commits to the same branch
   - Mark conversations as resolved when addressed

### Pull Request Template

When creating a PR, include:

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update

## Testing
Describe the tests you ran and how to reproduce them.

## Checklist
- [ ] Code follows project style guidelines (`cargo fmt`)
- [ ] Linting passes (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated (if needed)
- [ ] No new warnings introduced
```

## Documentation

### Code Documentation

- **Public APIs** must have doc comments (`///`)
- Include **Examples** in doc comments when helpful
- Document **errors** that functions can return
- Explain **complex algorithms** or non-obvious design decisions

### README Updates

Update the README.md when adding:
- New features that users should know about
- Changes to installation or usage instructions
- New dependencies or system requirements
- Changes to configuration options

## Issue Guidelines

### Before Creating an Issue

- Search existing issues (open and closed)
- Check the README and documentation
- Try the latest release

### Issue Templates

**Bug Report**:
- Description of the bug
- Steps to reproduce
- Expected vs. actual behavior
- Environment details
- Logs/screenshots

**Feature Request**:
- Problem statement
- Proposed solution
- Alternative solutions considered
- Additional context

## Questions?

If you have questions about contributing:
- Check existing issues and discussions
- Open a new issue with the "question" label
- Reach out to the maintainers

## Recognition

Contributors will be recognized in:
- Release notes
- Project documentation
- GitHub contributors page

Thank you for contributing to Telegram Bot Debugger! 🚀

