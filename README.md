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

[1/6] Checking installation status
[2/6] Fetching formula from registry
✓ Found fizzbuzz v1.0.0: A colorful FizzBuzz CLI with customizable ranges and rules
[3/6] Preparing workspace
  Workspace: /Users/matan.zruya/.vibe/cellar/fizzbuzz/1.0.0/src
[4/6] Generating code with AI agent
  Generation time: 22.1s
✓ Code generated successfully
[5/6] Building generated code
✓ Built with cargo
[6/6] Installing binaries
✓ Linked: fizzbuzz

✓ fizzbuzz v1.0.0 installed successfully!
```

## How it works

1. **Fetch** - Downloads a `formula.toml` and `prompt.md` from the [formula registry](https://github.com/mzruya/vibe-registry)
2. **Generate** - Sends the prompt to an AI coding agent (Claude Code) which writes all the source files
3. **Build** - Auto-detects the build system (Cargo, Go, Make, npm) and compiles
4. **Link** - Copies binaries to the cellar and symlinks them to `~/.vibe/bin/`

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/mzruya/vibe/main/install.sh | bash
```

Then add to your PATH:

```sh
export PATH="$HOME/.vibe/bin:$PATH"
```

### From source

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

Formulas live in [mzruya/vibe-registry](https://github.com/mzruya/vibe-registry). Each formula is a directory with two files:

### `formula.toml`

```toml
[package]
name = "hello"
version = "1.0.0"
description = "A friendly hello world CLI"
license = "MIT"
binaries = ["hello"]

# Optional: override auto-detected build
# [build]
# command = "cargo build --release"
# binary_paths = ["target/release/hello"]
```

### `prompt.md`

The prompt sent to the AI agent. Write it like you're pair-programming - describe what the tool should do, its CLI interface, and technical requirements.

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

The prompt can specify any language. Vibe auto-detects the build system from whatever files the agent generates:

| File detected | Build command |
|---------------|---------------|
| `Cargo.toml` | `cargo build --release` |
| `go.mod` | `go build` |
| `Makefile` | `make` |
| `package.json` | `npm install && npm run build` |
| `setup.py` / `pyproject.toml` | (no build needed) |

## Architecture

```
src/
  main.rs                         # Entry point, command dispatch
  cli/
    mod.rs                        # Clap CLI definition
    commands/
      install.rs                  # Fetch -> generate -> build -> link pipeline
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
    build.rs                      # Build system detection and execution
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
