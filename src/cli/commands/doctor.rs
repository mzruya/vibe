use anyhow::Result;

use crate::cellar::link;
use crate::config;
use crate::ui::Ui;

pub async fn run() -> Result<()> {
    Ui::header("Vibe Doctor");
    println!();

    let mut all_ok = true;

    // Check vibe home directory
    check("Vibe home directory", config::vibe_home().exists(), &mut all_ok);

    // Check bin directory
    check("Binary directory", config::bin_dir().exists(), &mut all_ok);

    // Check cellar directory
    check("Cellar directory", config::cellar_dir().exists(), &mut all_ok);

    // Check config file
    check(
        "Config file",
        config::Config::config_path().exists(),
        &mut all_ok,
    );

    // Check PATH
    let in_path = link::is_bin_in_path();
    if in_path {
        Ui::success("~/.vibe/bin is in PATH");
    } else {
        Ui::warning("~/.vibe/bin is NOT in PATH");
        Ui::info("  Add to your shell profile: export PATH=\"$HOME/.vibe/bin:$PATH\"");
        all_ok = false;
    }

    println!();
    Ui::header("AI Agents");

    // Check Claude Code
    check_command("Claude Code", "claude", &["--version"]).await;

    // Check Codex
    check_command("Codex", "codex", &["--version"]).await;

    println!();
    Ui::header("Build Tools");

    // Check common build tools
    check_command("Rust/Cargo", "cargo", &["--version"]).await;
    check_command("Go", "go", &["version"]).await;
    check_command("Make", "make", &["--version"]).await;
    check_command("Node/npm", "npm", &["--version"]).await;

    println!();
    if all_ok {
        Ui::success("Everything looks good!");
    } else {
        Ui::warning("Some issues found. See above for details.");
    }

    Ok(())
}

fn check(label: &str, ok: bool, all_ok: &mut bool) {
    if ok {
        Ui::success(label);
    } else {
        Ui::error(label);
        *all_ok = false;
    }
}

async fn check_command(label: &str, cmd: &str, args: &[&str]) {
    match which::which(cmd) {
        Ok(_) => {
            if let Ok(output) = tokio::process::Command::new(cmd)
                .args(args)
                .output()
                .await
            {
                let version = String::from_utf8_lossy(&output.stdout);
                let version = version.trim();
                if version.is_empty() {
                    Ui::success(&format!("{}: found", label));
                } else {
                    let first_line = version.lines().next().unwrap_or(version);
                    Ui::success(&format!("{}: {}", label, first_line));
                }
            } else {
                Ui::success(&format!("{}: found", label));
            }
        }
        Err(_) => {
            Ui::warning(&format!("{}: not found", label));
        }
    }
}
