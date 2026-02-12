use anyhow::{Result, bail};
use std::path::Path;

use super::{AgentDyn, AgentResult};

pub struct CodexAgent;

impl AgentDyn for CodexAgent {
    fn generate_dyn(
        &self,
        _prompt: &str,
        _working_dir: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AgentResult>> + Send + '_>> {
        Box::pin(async { bail!("Codex agent is not yet implemented. Use 'claude' for now.") })
    }
}
