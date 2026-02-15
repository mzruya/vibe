use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use super::formula::{FetchedFormula, Formula};
use super::github::IndexEntry;

pub struct LocalRegistry {
    path: PathBuf,
}

impl LocalRegistry {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Fetch a formula from local filesystem. If version is None, finds the latest version.
    pub async fn fetch_formula(
        &self,
        package: &str,
        version: Option<&str>,
    ) -> Result<FetchedFormula> {
        let package_dir = self.path.join("formulas").join(package);

        if !package_dir.exists() {
            bail!("Package '{}' not found in local registry at {}", package, self.path.display());
        }

        // If no version specified, find the latest (highest semver)
        let version = match version {
            Some(v) => v.to_string(),
            None => {
                let mut versions: Vec<String> = std::fs::read_dir(&package_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect();

                if versions.is_empty() {
                    bail!("No versions found for package '{}'", package);
                }

                // Sort by semver descending
                versions.sort_by(|a, b| {
                    semver_cmp(b, a)
                });

                versions.into_iter().next().unwrap()
            }
        };

        let version_dir = package_dir.join(&version);
        if !version_dir.exists() {
            bail!("Version '{}' not found for package '{}'", version, package);
        }

        let formula_path = version_dir.join("formula.toml");
        let prompt_path = version_dir.join("prompt.md");

        let formula_content = std::fs::read_to_string(&formula_path)
            .with_context(|| format!("Failed to read {}", formula_path.display()))?;

        let prompt = std::fs::read_to_string(&prompt_path)
            .with_context(|| format!("Failed to read {}", prompt_path.display()))?;

        let formula: Formula =
            toml::from_str(&formula_content).context("Failed to parse formula.toml")?;

        Ok(FetchedFormula { formula, prompt })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<IndexEntry>> {
        let formulas_dir = self.path.join("formulas");

        if !formulas_dir.exists() {
            return Ok(vec![]);
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entry in std::fs::read_dir(&formulas_dir)? {
            let entry = entry?;
            if !entry.path().is_dir() {
                continue;
            }

            let name = entry.file_name().into_string().unwrap_or_default();

            // Get versions
            let versions: Vec<String> = std::fs::read_dir(entry.path())?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();

            if versions.is_empty() {
                continue;
            }

            // Read formula to get description
            let latest = versions.iter().max_by(|a, b| semver_cmp(a, b)).unwrap();
            let formula_path = entry.path().join(latest).join("formula.toml");

            let description = if let Ok(content) = std::fs::read_to_string(&formula_path) {
                if let Ok(formula) = toml::from_str::<Formula>(&content) {
                    formula.package.description
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            if name.to_lowercase().contains(&query_lower)
                || description.to_lowercase().contains(&query_lower)
            {
                results.push(IndexEntry {
                    name,
                    description,
                    versions,
                });
            }
        }

        Ok(results)
    }

    pub async fn get_package_info(&self, package: &str) -> Result<IndexEntry> {
        let package_dir = self.path.join("formulas").join(package);

        if !package_dir.exists() {
            bail!("Package '{}' not found", package);
        }

        let versions: Vec<String> = std::fs::read_dir(&package_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        if versions.is_empty() {
            bail!("No versions found for package '{}'", package);
        }

        let latest = versions.iter().max_by(|a, b| semver_cmp(a, b)).unwrap();
        let formula_path = package_dir.join(latest).join("formula.toml");

        let description = if let Ok(content) = std::fs::read_to_string(&formula_path) {
            if let Ok(formula) = toml::from_str::<Formula>(&content) {
                formula.package.description
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Ok(IndexEntry {
            name: package.to_string(),
            description,
            versions,
        })
    }
}

/// Simple semver comparison (major.minor.patch)
fn semver_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    parse(a).cmp(&parse(b))
}
