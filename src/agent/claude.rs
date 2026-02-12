use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;

use super::{AgentDyn, AgentResult};

pub struct ClaudeAgent;

/// Streaming display that shows rolling activity
struct StreamDisplay {
    lines: Vec<String>,
    max_lines: usize,
    last_height: usize,
}

impl StreamDisplay {
    fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            max_lines,
            last_height: 0,
        }
    }

    fn add_line(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > self.max_lines {
            self.lines.remove(0);
        }
        self.render();
    }

    fn update_last(&mut self, line: String) {
        if self.lines.is_empty() {
            self.lines.push(line);
        } else {
            *self.lines.last_mut().unwrap() = line;
        }
        self.render();
    }

    fn render(&mut self) {
        let mut stdout = std::io::stdout();

        // Move cursor up to clear previous output
        if self.last_height > 0 {
            print!("\x1b[{}A", self.last_height);
        }

        // Clear and print each line
        for line in &self.lines {
            // Truncate to terminal width (assume 80 for safety)
            let display_line = if line.len() > 78 {
                format!("{}...", &line[..75])
            } else {
                line.clone()
            };
            println!("\x1b[2K  \x1b[90m{}\x1b[0m", display_line);
        }

        self.last_height = self.lines.len();
        stdout.flush().ok();
    }

    fn clear(&self) {
        if self.last_height > 0 {
            print!("\x1b[{}A", self.last_height);
            for _ in 0..self.last_height {
                println!("\x1b[2K");
            }
            print!("\x1b[{}A", self.last_height);
            std::io::stdout().flush().ok();
        }
    }
}

impl ClaudeAgent {
    async fn run(&self, prompt: &str, working_dir: &Path) -> Result<AgentResult> {
        let start = std::time::Instant::now();

        let mut child = tokio::process::Command::new("claude")
            .args([
                "-p",
                prompt,
                "--output-format",
                "stream-json",
                "--verbose",
                "--dangerously-skip-permissions",
                "--no-session-persistence",
                "--allowed-tools",
                "Bash Edit Write Read",
            ])
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()
            .context("Failed to run 'claude'. Is Claude Code installed?")?;

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        let mut display = StreamDisplay::new(3);
        let mut cost_usd: Option<f64> = None;
        let mut session_id: Option<String> = None;
        let mut is_error = false;
        let mut current_text = String::new();

        // Process stream with Ctrl+C handling
        loop {
            tokio::select! {
                biased;
                _ = signal::ctrl_c() => {
                    display.clear();
                    bail!("Interrupted by user");
                }
                line = reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            if let Some(event) = parse_stream_event(&line) {
                                match event {
                                    StreamEvent::ToolUse { tool, input_preview } => {
                                        current_text.clear();
                                        display.add_line(format!("{}: {}", tool, input_preview));
                                    }
                                    StreamEvent::ToolResult { output_preview } => {
                                        if !output_preview.is_empty() {
                                            display.add_line(format!("  → {}", output_preview));
                                        }
                                    }
                                    StreamEvent::Text { text } => {
                                        current_text.push_str(&text);
                                        let preview = current_text.lines().last().unwrap_or("").to_string();
                                        if !preview.trim().is_empty() {
                                            display.update_last(preview);
                                        }
                                    }
                                    StreamEvent::Result { cost, session, error } => {
                                        cost_usd = cost;
                                        session_id = session;
                                        is_error = error;
                                    }
                                }
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            display.clear();
                            bail!("Error reading from claude: {}", e);
                        }
                    }
                }
            }
        }

        // Wait for process to finish
        let status = child.wait().await.context("Failed to wait for claude")?;
        display.clear();

        let duration = start.elapsed().as_secs_f64();

        if !status.success() || is_error {
            bail!("Claude Code exited with error");
        }

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

enum StreamEvent {
    ToolUse {
        tool: String,
        input_preview: String,
    },
    ToolResult {
        output_preview: String,
    },
    Text {
        text: String,
    },
    Result {
        cost: Option<f64>,
        session: Option<String>,
        error: bool,
    },
}

fn parse_stream_event(line: &str) -> Option<StreamEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let event_type = v.get("type")?.as_str()?;

    match event_type {
        "assistant" => {
            // Check for tool_use in content
            if let Some(content) = v.get("message")?.get("content")?.as_array() {
                for item in content {
                    if item.get("type")?.as_str()? == "tool_use" {
                        let tool = item.get("name")?.as_str()?.to_string();
                        let input = item.get("input")?;
                        let input_preview = format_tool_input(&tool, input);
                        return Some(StreamEvent::ToolUse {
                            tool,
                            input_preview,
                        });
                    }
                    if item.get("type")?.as_str()? == "text" {
                        let text = item.get("text")?.as_str()?.to_string();
                        return Some(StreamEvent::Text { text });
                    }
                }
            }
            None
        }
        "user" => {
            // Tool results come as user messages
            if let Some(content) = v.get("message")?.get("content")?.as_array() {
                for item in content {
                    if item.get("type")?.as_str()? == "tool_result" {
                        let output = item.get("content")?.as_str().unwrap_or("");
                        let output_preview = output.lines().next().unwrap_or("").to_string();
                        return Some(StreamEvent::ToolResult { output_preview });
                    }
                }
            }
            None
        }
        "result" => {
            let cost = v.get("total_cost_usd").and_then(|c| c.as_f64());
            let session = v
                .get("session_id")
                .and_then(|s| s.as_str())
                .map(String::from);
            let error = v.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
            Some(StreamEvent::Result {
                cost,
                session,
                error,
            })
        }
        _ => None,
    }
}

fn format_tool_input(tool: &str, input: &serde_json::Value) -> String {
    match tool {
        "Write" | "Edit" => input
            .get("file_path")
            .and_then(|p| p.as_str())
            .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
            .unwrap_or_default(),
        "Read" => input
            .get("file_path")
            .and_then(|p| p.as_str())
            .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
            .unwrap_or_default(),
        "Bash" => input
            .get("command")
            .and_then(|c| c.as_str())
            .map(|c| {
                let c = c.trim();
                if c.len() > 50 {
                    format!("{}...", &c[..47])
                } else {
                    c.to_string()
                }
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}
