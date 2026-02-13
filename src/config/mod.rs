use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_REGISTRY_OWNER: &str = "mzruya";
const DEFAULT_REGISTRY_REPO: &str = "vibe-registry";
const DEFAULT_AGENT: &str = "claude";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub registry: RegistryConfig,
    pub agent: AgentConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub owner: String,
    pub repo: String,
    #[serde(default = "default_branch")]
    pub branch: String,
}

fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub default: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry: RegistryConfig {
                owner: DEFAULT_REGISTRY_OWNER.to_string(),
                repo: DEFAULT_REGISTRY_REPO.to_string(),
                branch: "main".to_string(),
            },
            agent: AgentConfig {
                default: DEFAULT_AGENT.to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let contents =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            toml::from_str(&contents).context("Failed to parse config file")
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        vibe_home().join("config.toml")
    }
}

pub fn vibe_home() -> PathBuf {
    if let Ok(home) = std::env::var("VIBE_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".vibe")
}

pub fn bin_dir() -> PathBuf {
    vibe_home().join("bin")
}

pub fn cellar_dir() -> PathBuf {
    vibe_home().join("cellar")
}

pub fn cache_dir() -> PathBuf {
    vibe_home().join("cache")
}

pub fn ensure_dirs() -> Result<()> {
    for dir in [vibe_home(), bin_dir(), cellar_dir(), cache_dir()] {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }
    Ok(())
}
