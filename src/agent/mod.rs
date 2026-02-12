pub mod claude;
pub mod codex;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub success: bool,
    pub cost_usd: Option<f64>,
    pub duration_secs: Option<f64>,
    pub session_id: Option<String>,
}

pub fn create_agent(name: &str) -> Result<Box<dyn AgentDyn + Send + Sync>> {
    match name {
        "claude" => Ok(Box::new(claude::ClaudeAgent)),
        "codex" => Ok(Box::new(codex::CodexAgent)),
        _ => bail!("Unknown agent: {}. Supported: claude, codex", name),
    }
}

pub trait AgentDyn {
    fn generate_dyn(
        &self,
        prompt: &str,
        working_dir: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AgentResult>> + Send + '_>>;
}
