# Contributing

## Setup

Install Rust with support for edition 2024, then fork and clone the repository.

```sh
cargo build
cargo run
```

## Issues

Search existing issues before opening one. Use the bug report or feature request
template. For bugs, include reproduction steps, expected behavior, and your OS
and app version. Discuss major changes in an issue before implementing them.

## Pull Requests

1. Create a branch for your change.
2. Follow the [code organization guide](docs/develop/code_organization.md).
3. Keep changes focused; update relevant documentation and tests.
4. Run the checks below and manually verify affected UI behavior.
5. Open a pull request describing the change, related issues, and validation.

```sh
cargo fmt --check
cargo check
cargo test
```

See [Development](docs/develop/README.md) for implementation details.

## License

Contributions must be your own work or code you have permission to contribute
under the project's [PolyForm Noncommercial License 1.0.0](LICENSE). Preserve
third-party license notices.
