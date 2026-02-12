use anyhow::{Context, Result, bail};
use std::path::Path;

use super::{AgentDyn, AgentResult};

pub struct ClaudeAgent;

impl ClaudeAgent {
    async fn run(&self, prompt: &str, working_dir: &Path) -> Result<AgentResult> {
        let start = std::time::Instant::now();

        let output = tokio::process::Command::new("claude")
            .args([
                "-p",
                prompt,
                "--output-format",
                "json",
                "--dangerously-skip-permissions",
                "--no-session-persistence",
                "--allowed-tools",
                "Bash Edit Write Read",
            ])
            .current_dir(working_dir)
            .output()
            .await
            .context("Failed to run 'claude'. Is Claude Code installed?")?;

        let duration = start.elapsed().as_secs_f64();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Claude Code exited with error: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output for cost info
        let cost_usd = parse_cost_from_json(&stdout);
        let session_id = parse_session_id_from_json(&stdout);

        Ok(AgentResult {
            success: true,
            cost_usd,
            duration_secs: Some(duration),
            session_id,
        })
    }
}

impl AgentDyn for ClaudeAgent {
    fn generate_dyn(
        &self,
        prompt: &str,
        working_dir: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AgentResult>> + Send + '_>> {
        let prompt = prompt.to_string();
        let working_dir = working_dir.to_path_buf();
        Box::pin(async move { self.run(&prompt, &working_dir).await })
    }
}

fn parse_cost_from_json(output: &str) -> Option<f64> {
    let v: serde_json::Value = serde_json::from_str(output).ok()?;
    v.get("cost_usd")
        .and_then(|c| c.as_f64())
}

fn parse_session_id_from_json(output: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(output).ok()?;
    v.get("session_id")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
}
