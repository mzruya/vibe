# Vibe

AI-powered package manager. Instead of downloading pre-compiled binaries, Vibe fetches a prompt from a registry and uses an AI coding agent to generate, compile, and install the software from scratch.

```
$ vibe install fizzbuzz

        _ _
 __   _(_) |__   ___
 \ \ / / | '_ \ / _ \
  \ V /| | |_) |  __/
   \_/ |_|_.__/ \___|  v0.1.0

  AI-powered package manager

[1/5] Checking installation status
[2/5] Fetching formula from registry
✓ Found fizzbuzz v1.0.0: A colorful FizzBuzz CLI with customizable ranges and rules
[3/5] Preparing workspace
  Workspace: /Users/matan.zruya/.vibe/cellar/fizzbuzz/1.0.0/src
[4/5] Generating and building with AI agent
  Duration: 22.1s
✓ Code generated and built successfully
[5/5] Installing binaries
✓ Linked: fizzbuzz

✓ fizzbuzz v1.0.0 installed successfully!
```

## How it works

1. **Fetch** - Downloads a `formula.toml` and `prompt.md` from the formula registry
2. **Generate & Build** - Sends the prompt to an AI coding agent (Claude Code) which writes, compiles, and tests the code
3. **Link** - Copies binaries to the cellar and symlinks them to `~/.vibe/bin/`

## Formula Registry

**[mzruya/vibe-registry](https://github.com/mzruya/vibe-registry)** - The official formula registry containing all available packages.

Browse the registry to see what's available, or contribute your own formulas. Each formula is just a prompt that describes what the tool should do—the AI handles the implementation.

## Installation

```sh
cargo install --git https://github.com/mzruya/vibe.git
```

### Prerequisites

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (the AI agent that generates code)

## Usage

```sh
# Install a package
vibe install hello

# Force reinstall
vibe install hello --force

# Search the registry
vibe search countdown

# Show package info
vibe info hello

# List installed packages
vibe list

# Uninstall
vibe uninstall hello

# Check system health
vibe doctor
```

## Configuration

Vibe stores everything under `~/.vibe/`:

```
~/.vibe/
  config.toml           # registry URL, default agent
  bin/                   # symlinked binaries (add to PATH)
  cellar/                # installed packages
    <package>/<version>/
      src/               # AI-generated source code
      bin/               # compiled binaries
      receipt.json       # installation metadata
  cache/                 # cached formulas
```

Default `config.toml`:

```toml
[registry]
owner = "mzruya"
repo = "vibe-registry"

[agent]
default = "claude"
```

## Creating formulas

Want to add a package? Submit a PR to **[mzruya/vibe-registry](https://github.com/mzruya/vibe-registry)**. Each formula is a directory with two files:

### `formula.toml`

```toml
[package]
name = "hello"
version = "1.0.0"
description = "A friendly hello world CLI"
license = "MIT"
binaries = ["hello"]
```

### `prompt.md`

The prompt sent to the AI agent. Describe what the tool should do, its CLI interface, and technical requirements. The AI agent handles both writing and building the code.

```markdown
# Hello CLI

Write a Rust CLI tool called `hello` that greets users.

## Requirements
- When run with no arguments: print "Hello, world!"
- When run with a name argument: print "Hello, <name>!"
- Use `clap` for argument parsing

## Technical Requirements
- Language: Rust
- Create a complete Cargo.toml
- The code should compile with `cargo build --release`
```

## Architecture

```
src/
  main.rs                         # Entry point, command dispatch
  cli/
    mod.rs                        # Clap CLI definition
    commands/
      install.rs                  # Fetch -> generate+build -> link pipeline
      uninstall.rs                # Remove package and symlinks
      list.rs                     # List installed packages
      search.rs                   # Search registry
      info.rs                     # Show package details
      doctor.rs                   # System health check
  registry/
    formula.rs                    # Formula, PackageMetadata, BuildConfig types
    github.rs                     # Registry client (raw.githubusercontent.com)
  agent/
    mod.rs                        # Agent trait + factory
    claude.rs                     # Claude Code backend (claude -p)
    codex.rs                      # Codex stub
  cellar/
    mod.rs                        # Install state, receipts
    link.rs                       # Symlink management
  config/
    mod.rs                        # ~/.vibe/ directory and config.toml
  ui/
    mod.rs                        # Colored output, spinners, progress
    banner.rs                     # ASCII art banner
```

## Development

```sh
# Build
cargo build

# Run tests
cargo test

# Run directly
cargo run -- doctor
cargo run -- search hello
```

## License

MIT
