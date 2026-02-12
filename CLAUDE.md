# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is this project?

Vibe is a Homebrew-like CLI written in Rust that uses AI coding agents to generate, compile, and install software from prompt-based "formulas". Instead of downloading pre-built binaries, `vibe install <package>` fetches a prompt from a GitHub registry and sends it to Claude Code, which writes and builds the source code. Vibe then links the resulting binary.

## Quick reference

```sh
cargo build                      # Build the project
cargo test                       # Run all tests
cargo run -- doctor              # Test the CLI
cargo run -- search hello        # Test registry access
```

## Project structure

```
src/
  main.rs                         # Entry point: parse CLI, dispatch commands
  cli/mod.rs                      # Clap derive types (Cli, Commands enum)
  cli/commands/install.rs         # Core pipeline: fetch -> AI generate+build -> link
  cli/commands/uninstall.rs       # Remove package + symlinks
  cli/commands/list.rs            # List installed packages from cellar receipts
  cli/commands/search.rs          # Search registry via index.json
  cli/commands/info.rs            # Show formula details + local install status
  cli/commands/doctor.rs          # System health: check agents, build tools, PATH
  registry/formula.rs             # Types: Formula, PackageMetadata, BuildConfig, FetchedFormula
  registry/github.rs              # GitHubRegistry: fetch files via raw.githubusercontent.com
  agent/mod.rs                    # AgentDyn trait, AgentResult, create_agent() factory
  agent/claude.rs                 # ClaudeAgent: runs `claude -p` with JSON output
  agent/codex.rs                  # CodexAgent: stub (not yet implemented)
  cellar/mod.rs                   # Cellar struct: package dirs, receipts, list/remove
  cellar/link.rs                  # Symlink binaries to ~/.vibe/bin/
  config/mod.rs                   # Config struct, ~/.vibe/ directory management
  ui/mod.rs                       # Ui helpers: header, success, error, warning, spinner
  ui/banner.rs                    # ASCII art banner
tests/
  cli_tests.rs                    # Integration tests using assert_cmd
  cellar_tests.rs                 # Cellar/receipt/symlink tests
```

## Key concepts

- **Formula**: A `formula.toml` (metadata) + `prompt.md` (AI prompt) pair, stored in the [mzruya/vibe-registry](https://github.com/mzruya/vibe-registry) GitHub repo
- **Cellar**: `~/.vibe/cellar/<package>/<version>/` - where generated source and compiled binaries live
- **Receipt**: `receipt.json` in each cellar entry - tracks install metadata (agent, cost, binaries, timestamp)

## Install pipeline (install.rs)

1. Check if already installed (skip unless `--force`)
2. Fetch `formula.toml` + `prompt.md` from raw.githubusercontent.com
3. Create workspace at `~/.vibe/cellar/<name>/<version>/src/`
4. Compose prompt with instructions to build and place binary in `./bin/`
5. AI agent writes source files, builds, and copies binary to `./bin/`
6. Vibe copies binaries from `./bin/` to cellar and symlinks to `~/.vibe/bin/`
7. Save `receipt.json`

## Config

`~/.vibe/config.toml` - defaults to `mzruya/vibe-registry` as the formula source and `claude` as the agent.

## Agent interface

`agent/mod.rs` defines the `AgentDyn` trait with a single method `generate_dyn(prompt, working_dir)`. The Claude backend (`agent/claude.rs`) runs:

```
claude -p "<prompt>" --output-format json --dangerously-skip-permissions --no-session-persistence --allowed-tools "Bash Edit Write Read"
```

The AI agent is responsible for:
- Writing source code
- Building/compiling
- Placing the final binary in `./bin/<binary_name>`

## Testing

- Integration tests use `assert_cmd` to test CLI behavior
- Tests hit the real GitHub registry for `search` and `info` commands
- Run with `cargo test`
