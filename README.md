# rkit

A minimal, high-performance CLI suite written in Rust for organizing, scanning, and inspecting Git repositories under a user-defined workspace.

## Features

- `clone`: Smart Git clone wrapper that organizes repositories by domain and organization
- `ls`: List all Git repositories in your workspace
- `view`: View repository information and metadata

## Installation

```bash
cargo install --path .
```

## Configuration

The configuration file is located at:
- Unix-like systems: `~/.config/rkit/config.yaml`
- Windows: `%APPDATA%\rkit\config.yaml`

It will be created automatically on first run with default values.

Example configuration:

```yaml
project_root: "~/Projects"  # or "%USERPROFILE%\projects" on Windows

rview:
  - command: "git -C {REPO} rev-parse --abbrev-ref HEAD"
    label: "Branch"
  - command: "git -C {REPO} status --porcelain"
    label: "Uncommitted Changes"
  - command: "cat {REPO}/README.md"
    label: "README"
```

## Usage

### Clone a repository

```bash
rkit clone https://github.com/username/repo.git
```

This will clone the repository to `~/Projects/github.com/username/repo`.

### List repositories

```bash
rkit ls [--full]
```

Lists all Git repositories found under the configured project root. Use the `--full` flag to show absolute paths instead of relative paths.

### View repository information

```bash
rkit view path/to/repo
```

Displays repository information based on configured commands or falls back to showing README.md or directory listing. The command will:
- Show configured Git information (branch, status, etc.)
- Display the repository's README.md if available
- Fall back to directory listing if no README is found

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- ls

# Run with trace output
RUST_LOG=trace cargo run -- ls
```

The project uses structured logging with different verbosity levels:
- `info`: Default level, shows basic operations
- `debug`: Shows detailed operation information
- `trace`: Shows all internal operations

## License

MIT 