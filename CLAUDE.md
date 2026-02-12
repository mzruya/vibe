# CLAUDE.md

## What is this project?

Vibe is a Homebrew-like CLI written in Rust that uses AI coding agents to generate, compile, and install software from prompt-based "formulas". Instead of downloading pre-built binaries, `vibe install <package>` fetches a prompt from a GitHub registry and sends it to Claude Code, which writes the source code. Vibe then auto-detects the build system and compiles it.

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
  cli/commands/install.rs         # Core pipeline: fetch -> AI generate -> build -> link
  cli/commands/uninstall.rs       # Remove package + symlinks
  cli/commands/list.rs            # List installed packages from cellar receipts
  cli/commands/search.rs          # Search registry via GitHub Contents API
  cli/commands/info.rs            # Show formula details + local install status
  cli/commands/doctor.rs          # System health: check agents, build tools, PATH
  registry/formula.rs             # Types: Formula, PackageMetadata, BuildConfig, FetchedFormula
  registry/github.rs              # GitHubRegistry: fetch files via Contents API, base64 decode
  agent/mod.rs                    # AgentDyn trait, AgentResult, create_agent() factory
  agent/claude.rs                 # ClaudeAgent: runs `claude -p` with JSON output
  agent/codex.rs                  # CodexAgent: stub (not yet implemented)
  cellar/mod.rs                   # Cellar struct: package dirs, receipts, list/remove
  cellar/build.rs                 # BuildSystem enum, detect from files, run builds
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
- **Build auto-detection**: Looks for `Cargo.toml`, `go.mod`, `Makefile`, `package.json`, `setup.py` in the generated source dir

## Install pipeline (install.rs)

1. Check if already installed (skip unless `--force`)
2. Fetch `formula.toml` + `prompt.md` from GitHub Contents API
3. Create workspace at `~/.vibe/cellar/<name>/<version>/src/`
4. Wrap the prompt with system instructions and send to AI agent
5. AI agent writes source files into the workspace
6. Auto-detect build system from generated files, run the build
7. Find output binaries, copy to cellar `bin/`, symlink to `~/.vibe/bin/`
8. Save `receipt.json`

## Config

`~/.vibe/config.toml` - defaults to `mzruya/vibe-registry` as the formula source and `claude` as the agent.

## GitHub auth

The registry client (`registry/github.rs`) tries `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token` for API authentication.

## Agent interface

`agent/mod.rs` defines the `AgentDyn` trait with a single method `generate_dyn(prompt, working_dir)`. The Claude backend (`agent/claude.rs`) runs:

```
claude -p "<prompt>" --output-format json --dangerously-skip-permissions --no-session-persistence --allowed-tools "Bash Edit Write Read"
```

## Testing conventions

- Integration tests use `assert_cmd` to test CLI behavior
- Tests that hit the real GitHub registry exist for `search` and `info` commands
- Run with `cargo test`

## Dependencies

Key crates: `clap` (CLI), `tokio` (async), `reqwest` (HTTP), `serde`/`toml`/`serde_json` (serialization), `indicatif`/`console` (terminal UI), `anyhow`/`thiserror` (errors).
