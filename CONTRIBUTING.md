# Contributing to Xolariq

Thanks for your interest in contributing to Xolariq.

## Development Setup

### Requirements

- Rust (stable)
- Node.js
- pnpm or npm
- Tauri prerequisites
- Windows recommended for shell-extension development

Install Rust:

```bash
rustup default stable
````

## Clone the Repository

```bash
git clone https://github.com/neikiri/xolariq-dev.git
cd xolariq-dev
```

## Development Commands

### Rust checks

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

### Run the Tauri app

```bash
cd app
npm install
npm run tauri dev
```

## Pull Requests

Please:

* Keep pull requests focused and reasonably small
* Write clear commit messages
* Run formatting, clippy, and tests before submitting
* Update documentation when relevant

## Code Style

* Follow standard Rust formatting (`cargo fmt`)
* Avoid unnecessary dependencies
* Prefer explicit and readable code over clever abstractions

## Reporting Issues

When opening an issue, include:

* Operating system
* Steps to reproduce
* Expected behavior
* Actual behavior
* Logs or screenshots if available

## License

By contributing to this project, you agree that your contributions will be licensed under the Mozilla Public License 2.0 (MPL-2.0).

