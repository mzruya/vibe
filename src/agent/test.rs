use anyhow::Result;
use std::path::Path;

use super::{AgentDyn, AgentResult};

/// A test agent that produces deterministic output for integration testing.
/// Creates a simple shell script binary that outputs a predictable message.
pub struct TestAgent;

impl AgentDyn for TestAgent {
    fn generate_dyn(
        &self,
        _prompt: &str,
        working_dir: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AgentResult>> + Send + '_>> {
        let working_dir = working_dir.to_path_buf();
        Box::pin(async move {
            // Create the bin directory
            let bin_dir = working_dir.join("bin");
            std::fs::create_dir_all(&bin_dir)?;

            // Create a simple shell script as the "binary"
            // The binary name is extracted from the prompt or defaults to "test-binary"
            let binary_name = "test-binary";
            let binary_path = bin_dir.join(binary_name);

            let script_content = r#"#!/bin/sh
echo "Hello from test binary!"
exit 0
"#;

            std::fs::write(&binary_path, script_content)?;

            // Make it executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))?;
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
