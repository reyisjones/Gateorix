# Contributing to Gateorix

Thank you for your interest in contributing to Gateorix! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository.
2. Clone your fork locally.
3. Create a feature branch from `main`.
4. Make your changes.
5. Run tests and linting.
6. Submit a pull request.

## Development Setup

### Prerequisites

- **Rust** (stable, latest) — [rustup.rs](https://rustup.rs)
- **Node.js** (v18+) and npm
- **Python** (3.10+) — for the Python adapter SDK
- Platform-specific dependencies for Tauri (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

### Build from Source

```bash
# Clone the repository
git clone https://github.com/gateorix/gateorix.git
cd gateorix

# Build the Rust workspace
cargo build

# Install CLI dependencies
cd cli && npm install && cd ..

# Run tests
cargo test
cd cli && npm test && cd ..
```

## Code Style

- **Rust**: Follow standard `rustfmt` formatting. Run `cargo fmt` before committing.
- **TypeScript**: Follow the project ESLint configuration. Run `npm run lint` in the `cli/` directory.
- **Python**: Follow PEP 8. Use `ruff` or `black` for formatting.

## Commit Messages

Use clear, descriptive commit messages:

```
feat(host-core): add tray icon support
fix(ipc): handle malformed JSON in bridge messages
docs(adapter-protocol): add Python adapter examples
```

Prefixes: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`.

## Pull Requests

- Keep PRs focused on a single change.
- Include tests for new functionality.
- Update documentation if the public API changes.
- Reference related issues in the PR description.

## Reporting Issues

Use [GitHub Issues](https://github.com/gateorix/gateorix/issues) with the provided templates for bug reports and feature requests.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
