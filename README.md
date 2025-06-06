<p align="center">
  <img src="/assets/images/rkit.png">
</p>

# rkit <a href="https://crates.io/crates/rkit"><img src="https://img.shields.io/crates/v/rkit" alt="Crates.io"></a> <img src="https://github.com/imthor/rkit/actions/workflows/publish.yml/badge.svg" alt="Publish Package">
A minimal, high-performance CLI suite written in Rust for organizing, scanning, and inspecting Git repositories under a user-defined workspace.

## Installation

```bash
cargo install rkit
```

## Features

- `clone`: Smart Git clone wrapper that organizes repositories by domain and organization
- `ls`: High-performance Git repository scanner with:
  - Parallel directory traversal
  - Configurable search depth and thread count
  - Progress bar and performance metrics
  - Symbolic link and filesystem boundary control
- `view`: View repository information and metadata

## Shell Extensions

rkit provides optional shell extensions that add useful functions and completions to your shell configuration.

### Prerequisites

- [rkit](https://github.com/imthor/rkit) - The main tool
- [fzf](https://github.com/junegunn/fzf) - Fuzzy finder for the terminal

### For Bash/Zsh Users

Install the shell extension:

```bash
# For zsh users (recommended)
zsh < <(curl -sSL https://raw.githubusercontent.com/imthor/rkit/main/install.sh)

# For bash users
curl -sSL https://raw.githubusercontent.com/imthor/rkit/main/install.sh | bash
```

The script will:
- Check for required dependencies
- Ask for confirmation before installing each function
- Create a backup of your shell config file
- Add the selected functions and completions

After installation, either:
- Restart your shell, or
- Run `source ~/.zshrc` (for Zsh) or `source ~/.bashrc` (for Bash)

### For Fish Users

Install the shell extension:

```bash
curl -sSL https://raw.githubusercontent.com/imthor/rkit/main/install.fish | fish
```

The script will:
- Check for required dependencies
- Ask for confirmation before installing each function
- Create a backup of your Fish config file
- Add the selected functions and completions

After installation, either:
- Restart your shell, or
- Run `source ~/.config/fish/config.fish`

### Available Extension Functions

After installing the shell extension, the following functions will be available:

#### `clone`

Clone a repository using rkit.

```bash
clone <repository>
```

#### `cdc`

Change directory to a repository using fuzzy search.

```bash
cdc [query]  # Optional query to pre-filter the list
```

#### `edit`

Open a repository in your default editor (VS Code) using fuzzy search.

```bash
edit [query]  # Optional query to pre-filter the list
```

### Extension Features

- **Fuzzy Search**: All functions use fzf for intuitive repository selection
- **Preview**: See repository details while searching
- **Shell Completions**: Tab completion for repository names
- **Query Support**: Pre-filter the repository list with a query
- **Safe Installation**: Creates backup of your config file before making changes

### Uninstalling Extensions

To remove the shell extension:

1. Open your shell config file:
   - Zsh: `~/.zshrc`
   - Bash: `~/.bashrc`
   - Fish: `~/.config/fish/config.fish`

2. Remove the section that starts with `# rkit functions`

3. Source your config file or restart your shell

### Extension Backup

The installation scripts create a backup of your config file before making any changes. The backup will be saved as:
- Zsh/Bash: `~/.zshrc.bak` or `~/.bashrc.bak`
- Fish: `~/.config/fish/config.fish.bak`

You can restore your original configuration by copying the backup file back if needed.

## Configuration

The configuration file is located at:
- Unix-like systems: `~/.config/rkit/config.yaml`
- Windows: `%APPDATA%\rkit\config.yaml`

It will be created automatically on first run with platform-specific default values.

### Default Configuration

The default configuration includes:

```yaml
# Linux/macOS
project_root: ~/projects
rview:
  - command: basename {REPO}
    label: Repo
  - command: git -C {REPO} rev-parse --abbrev-ref HEAD
    label: Active Branch
  - command: git -C {REPO} status
    label: Status
  - command: cat {REPO}/README.md
    label: README

# Windows
project_root: %USERPROFILE%\projects
rview:
  - command: powershell -Command "Split-Path -Leaf {REPO}"
    label: Repo
  - command: git -C {REPO} rev-parse --abbrev-ref HEAD
    label: Active Branch
  - command: git -C {REPO} status
    label: Status
  - command: type {REPO}\README.md
    label: README
```

## Usage

### Clone a repository

```bash
rkit clone https://github.com/imthor/rkit.git
```

This will clone the repository to `~/projects/github.com/imthor/rkit` (or `%USERPROFILE%\projects\github.com\username\repo` on Windows).

### List repositories

```bash
rkit ls [--full] [--max-depth <depth>] [--follow-links] [--same-file-system] [--threads <num>] [--max-repos <num>] [--no-stop-at-git]
```

Lists all Git repositories found under the configured project root.

Options:
- `--full`: Show absolute paths instead of relative paths
- `--max-depth <depth>`: Maximum depth to search for repositories [default: 4]
- `--follow-links`: Follow symbolic links [default: false]
- `--same-file-system`: Stay on the same filesystem [default: true]
- `--threads <num>`: Number of threads to use for searching [default: number of CPU cores]
- `--max-repos <num>`: Maximum number of repositories to find [default: no limit]
- `--no-stop-at-git`: Don't skip repositories that are inside other repositories [default: false]

The command will:
- Show directories as they are found, providing immediate feedback
- By default, skip repositories that are inside other repositories (e.g., if repo2 contains a .git directory, any repositories inside repo2 will be skipped)
- When `--no-stop-at-git` is used, it will find all repositories regardless of their location in the directory tree
- Include a progress bar showing the scanning status
- When run with debug logging enabled (`RUST_LOG=debug`), it will display performance metrics including:
  - Number of repositories found
  - Number of directories scanned
  - Total duration of the operation

### View repository information

```bash
rkit view path/to/repo
```

Displays repository information based on configured commands. The command will:
- Show the repository name
- Display the active branch
- Show the current git status
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
