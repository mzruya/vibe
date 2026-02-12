use anyhow::Result;
use std::path::Path;

use super::{AgentDyn, AgentResult};

/// A test agent that produces deterministic output for integration testing.
/// Creates a simple shell script binary that outputs a predictable message.
pub struct TestAgent;

/// Extract binary names from the prompt by looking for the binaries path.
/// The prompt contains lines like "- The final binary MUST be placed at: ./bin/hello"
/// or "- The final binary MUST be placed at: ./bin/foo, bar"
fn extract_binary_names(prompt: &str) -> Vec<String> {
    for line in prompt.lines() {
        let line = line.trim();
        if line.contains("MUST be placed at: ./bin/") {
            // Extract the part after "./bin/"
            if let Some(idx) = line.find("./bin/") {
                let names_part = &line[idx + 6..]; // Skip "./bin/"
                return names_part
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    // Fallback to default if not found
    vec!["test-binary".to_string()]
}

impl AgentDyn for TestAgent {
    fn generate_dyn(
        &self,
        prompt: &str,
        working_dir: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AgentResult>> + Send + '_>> {
        let working_dir = working_dir.to_path_buf();
        let binary_names = extract_binary_names(prompt);
        Box::pin(async move {
            // Create the bin directory
            let bin_dir = working_dir.join("bin");
            std::fs::create_dir_all(&bin_dir)?;

            // Create shell script binaries for each expected binary name
            for binary_name in &binary_names {
                let binary_path = bin_dir.join(binary_name);

                let script_content = format!(
                    r#"#!/bin/sh
echo "Hello from {}!"
exit 0
"#,
                    binary_name
                );

                std::fs::write(&binary_path, script_content)?;

                // Make it executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))?;
                }
            }

            Ok(AgentResult {
                success: true,
                cost_usd: Some(0.0),
                duration_secs: Some(0.1),
                session_id: Some("test-session-123".to_string()),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_binary_names_single() {
        let prompt = r#"
You are building a package called "hello".

YOUR TASK:
1. Write all source code needed
2. Build/compile it
3. Copy the final binary to ./bin/

REQUIREMENTS:
- The final binary MUST be placed at: ./bin/hello
- The binary must be executable and work correctly
"#;
        assert_eq!(extract_binary_names(prompt), vec!["hello"]);
    }

    #[test]
    fn test_extract_binary_names_multiple() {
        let prompt = r#"
REQUIREMENTS:
- The final binary MUST be placed at: ./bin/foo, bar, baz
"#;
        assert_eq!(extract_binary_names(prompt), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_extract_binary_names_fallback() {
        let prompt = "No binary names here";
        assert_eq!(extract_binary_names(prompt), vec!["test-binary"]);
    }
}
