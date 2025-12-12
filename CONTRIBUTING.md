# Contributing to UAIP

Thank you for your interest in contributing to UAIP (Universal AI Integration Protocol)!

**UAIP is created and owned by Hakille.** By contributing to this project, you agree that your contributions will be licensed under the Apache License 2.0.

This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Docker & Docker Compose
- Git
- Basic understanding of IoT protocols and authentication systems

### Setting Up Your Development Environment

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/UAIP.git
   cd UAIP
   ```

2. **Install Dependencies**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install development tools
   cargo install cargo-watch
   cargo install sqlx-cli --no-default-features --features postgres
   ```

3. **Start Infrastructure**
   ```bash
   docker-compose up -d
   ```

4. **Run Migrations**
   ```bash
   docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/001_initial_schema.sql
   docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/002_rbac_tables.sql
   ```

5. **Verify Setup**
   ```bash
   cargo test
   cargo build
   ```

## Development Workflow

### Branching Strategy

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - New features
- `bugfix/*` - Bug fixes
- `hotfix/*` - Critical fixes for production

### Creating a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### Making Changes

1. Write code following our [coding standards](#coding-standards)
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Run linter: `cargo clippy -- -D warnings`

## Coding Standards

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent formatting
- Run `clippy` and fix all warnings
- Write documentation comments (`///`) for public APIs
- Maintain test coverage above 80%

### Code Style

```rust
// Good: Clear, documented, tested
/// Validates a JWT token
///
/// # Arguments
/// * `token` - The JWT token to validate
///
/// # Returns
/// * `Result<Claims>` - The validated claims or an error
pub fn validate_token(&self, token: &str) -> Result<Claims> {
    // Implementation
}

// Tests in the same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token() {
        // Test implementation
    }
}
```

### Naming Conventions

- **Functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Traits**: `PascalCase` (prefer nouns or adjectives)

### Error Handling

- Use `Result<T, UaipError>` for recoverable errors
- Use `thiserror` for custom error types
- Provide meaningful error messages
- Document error conditions in function docs

### Testing

- Write unit tests for all public functions
- Write integration tests for cross-crate functionality
- Use descriptive test names: `test_validate_token_with_expired_token`
- Test both success and error cases
- Use `#[should_panic]` sparingly

### Documentation

- Document all public APIs with `///` doc comments
- Include examples in documentation when helpful
- Update README.md for significant changes
- Keep migration documentation up-to-date

## Submitting Changes

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```bash
feat(auth): add JWT refresh token support

Implements automatic token refresh with configurable expiry.
Includes tests for token rotation and validation.

Closes #123

---

fix(registry): correct device status update trigger

The trigger was not firing on heartbeat updates.
Fixed by adjusting the WHEN clause condition.

Fixes #456
```

### Before Submitting

- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation is updated
- [ ] Commit messages follow conventions
- [ ] Branch is up-to-date with main

## Pull Request Process

1. **Create Pull Request**
   - Use a clear, descriptive title
   - Reference related issues
   - Provide detailed description of changes
   - Include screenshots/demos if applicable

2. **PR Template**
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests added/updated
   - [ ] Manual testing performed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] No new warnings
   - [ ] Tests pass locally

   ## Related Issues
   Closes #123
   ```

3. **Code Review**
   - Address review comments promptly
   - Keep discussions professional and constructive
   - Request re-review after making changes

4. **Merging**
   - PRs require at least one approval
   - CI must pass (when set up)
   - Squash commits for cleaner history (optional)

## Reporting Issues

### Bug Reports

Use the bug report template and include:

- **Description**: Clear description of the bug
- **Steps to Reproduce**: Numbered steps to reproduce
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Environment**: OS, Rust version, Docker version
- **Logs**: Relevant error messages or logs
- **Screenshots**: If applicable

### Feature Requests

Use the feature request template and include:

- **Problem**: What problem does this solve?
- **Proposed Solution**: Your suggested approach
- **Alternatives**: Other solutions considered
- **Additional Context**: Any other relevant information

## Project Structure

```
uaip-hub/
├── crates/
│   ├── uaip-core/       # Core types and protocols
│   ├── uaip-auth/       # Authentication (JWT, X.509, RBAC)
│   ├── uaip-registry/   # Device registry
│   ├── uaip-router/     # Message routing
│   ├── uaip-security/   # Encryption and security
│   ├── uaip-orchestrator/ # AI orchestration
│   ├── uaip-adapters/   # Protocol adapters
│   └── uaip-hub/        # Main service
├── migrations/          # Database migrations
├── config/             # Configuration files
├── tests/              # Integration tests
└── docs/               # Documentation
```

## Development Tips

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p uaip-auth

# With output
cargo test -- --nocapture

# Watch mode (requires cargo-watch)
cargo watch -x test
```

### Database Development

```bash
# Connect to database
docker exec -it uaip-postgres psql -U uaip -d uaip

# Reset database
docker-compose down -v
docker-compose up -d
# Re-run migrations
```

### Performance Profiling

```bash
# Release build
cargo build --release

# With profiling
cargo flamegraph --bin uaip-hub
```

## Community

- **GitHub Discussions**: For questions and discussions
- **GitHub Issues**: For bugs and feature requests
- **Pull Requests**: For code contributions

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in commit messages (Co-Authored-By)

Thank you for contributing to UAIP! 🚀
