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
    pub description: String,
    /// Available versions, newest first
    pub versions: Vec<String>,
}

impl IndexEntry {
    pub fn latest_version(&self) -> Option<&str> {
        self.versions.first().map(|s| s.as_str())
    }
}

pub struct GitHubRegistry {
    client: Client,
    owner: String,
    repo: String,
    branch: String,
}

impl GitHubRegistry {
    pub fn new(owner: &str, repo: &str, branch: &str) -> Self {
        Self {
            client: Client::new(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: branch.to_string(),
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

    /// Fetch a formula. If version is None, fetches the latest version.
    pub async fn fetch_formula(
        &self,
        package: &str,
        version: Option<&str>,
    ) -> Result<FetchedFormula> {
        // If no version specified, look up latest from index
        let version = match version {
            Some(v) => v.to_string(),
            None => {
                let index = self.fetch_index().await?;
                let entry = index
                    .formulas
                    .iter()
                    .find(|f| f.name == package)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Package '{}' not found in registry", package)
                    })?;
                entry
                    .latest_version()
                    .ok_or_else(|| anyhow::anyhow!("No versions available for '{}'", package))?
                    .to_string()
            }
        };

        let formula_path = format!("formulas/{}/{}/formula.toml", package, version);
        let prompt_path = format!("formulas/{}/{}/prompt.md", package, version);

        let formula_content = self
            .fetch_file(&formula_path)
            .await
            .with_context(|| format!("Formula not found for '{}@{}'", package, version))?;

        let prompt = self
            .fetch_file(&prompt_path)
            .await
            .with_context(|| format!("Prompt not found for '{}@{}'", package, version))?;

        let formula: Formula =
            toml::from_str(&formula_content).context("Failed to parse formula.toml")?;

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

    pub async fn get_package_info(&self, package: &str) -> Result<IndexEntry> {
        let index = self.fetch_index().await?;
        index
            .formulas
            .into_iter()
            .find(|f| f.name == package)
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found", package))
    }
}

/// Parse a package spec like "hello" or "hello@1.0.0" into (name, optional version)
pub fn parse_package_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some(idx) = spec.find('@') {
        (&spec[..idx], Some(&spec[idx + 1..]))
    } else {
        (spec, None)
    }
}
