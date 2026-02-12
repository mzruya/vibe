use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::Deserialize;

use super::formula::{FetchedFormula, Formula};

/// Index of all formulas in the registry (from index.json)
#[derive(Debug, Deserialize)]
pub struct RegistryIndex {
    pub formulas: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    pub version: String,
    pub description: String,
}

pub struct GitHubRegistry {
    client: Client,
    owner: String,
    repo: String,
    branch: String,
}

impl GitHubRegistry {
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            client: Client::new(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: "main".to_string(),
        }
    }

    fn raw_url(&self, path: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, self.repo, self.branch, path
        )
    }

    async fn fetch_file(&self, path: &str) -> Result<String> {
        let url = self.raw_url(path);
        let resp = self
            .client
            .get(&url)
            .header("User-Agent", "vibe-package-manager")
            .send()
            .await
            .with_context(|| format!("Failed to fetch {}", path))?;

        if !resp.status().is_success() {
            bail!("Failed to fetch {} (status {})", path, resp.status());
        }

        resp.text().await.context("Failed to read response body")
    }

    pub async fn fetch_formula(&self, package: &str) -> Result<FetchedFormula> {
        let formula_path = format!("formulas/{}/formula.toml", package);
        let prompt_path = format!("formulas/{}/prompt.md", package);

        let formula_content = self
            .fetch_file(&formula_path)
            .await
            .with_context(|| format!("Formula not found for package '{}'", package))?;

        let prompt = self
            .fetch_file(&prompt_path)
            .await
            .with_context(|| format!("Prompt not found for package '{}'", package))?;

        let formula: Formula = toml::from_str(&formula_content)
            .context("Failed to parse formula.toml")?;

        Ok(FetchedFormula { formula, prompt })
    }

    async fn fetch_index(&self) -> Result<RegistryIndex> {
        let content = self
            .fetch_file("index.json")
            .await
            .context("Failed to fetch registry index")?;

        serde_json::from_str(&content).context("Failed to parse index.json")
    }

    pub async fn search(&self, query: &str) -> Result<Vec<IndexEntry>> {
        let index = self.fetch_index().await?;
        let query_lower = query.to_lowercase();

        Ok(index
            .formulas
            .into_iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&query_lower)
                    || f.description.to_lowercase().contains(&query_lower)
            })
            .collect())
    }

    pub async fn list_all(&self) -> Result<Vec<IndexEntry>> {
        let index = self.fetch_index().await?;
        Ok(index.formulas)
    }
}
