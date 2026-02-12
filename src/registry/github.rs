use anyhow::{Context, Result, bail};
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;

use super::formula::{FetchedFormula, Formula};

#[derive(Debug, Deserialize)]
struct GitHubContent {
    content: Option<String>,
    encoding: Option<String>,
}

pub struct GitHubRegistry {
    client: Client,
    owner: String,
    repo: String,
    token: Option<String>,
}

impl GitHubRegistry {
    pub fn new(owner: &str, repo: &str) -> Self {
        let token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .or_else(|_| {
                // Fall back to `gh auth token` if available
                std::process::Command::new("gh")
                    .args(["auth", "token"])
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .ok_or(std::env::VarError::NotPresent)
            })
            .ok();

        Self {
            client: Client::new(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            token,
        }
    }

    fn contents_url(&self, path: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        )
    }

    fn authenticated_get(&self, url: &str) -> RequestBuilder {
        let mut req = self
            .client
            .get(url)
            .header("User-Agent", "vibe-package-manager")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req
    }

    async fn fetch_file(&self, path: &str) -> Result<String> {
        let url = self.contents_url(path);
        let resp = self
            .authenticated_get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch {}", path))?;

        if !resp.status().is_success() {
            bail!(
                "GitHub API returned {} for {}",
                resp.status(),
                path
            );
        }

        let content: GitHubContent = resp.json().await?;
        let encoded = content
            .content
            .context("No content in response")?;

        match content.encoding.as_deref() {
            Some("base64") => {
                let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
                let decoded = base64_decode(&cleaned)?;
                Ok(decoded)
            }
            _ => Ok(encoded),
        }
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

    pub async fn search(&self, query: &str) -> Result<Vec<Formula>> {
        let url = self.contents_url("formulas");
        let resp = self
            .authenticated_get(&url)
            .send()
            .await
            .context("Failed to list formulas")?;

        if !resp.status().is_success() {
            bail!("GitHub API returned {}", resp.status());
        }

        #[derive(Deserialize)]
        struct DirEntry {
            name: String,
            #[serde(rename = "type")]
            entry_type: String,
        }

        let entries: Vec<DirEntry> = resp.json().await?;
        let matching: Vec<&DirEntry> = entries
            .iter()
            .filter(|e| e.entry_type == "dir" && e.name.contains(query))
            .collect();

        let mut formulas = Vec::new();
        for entry in matching {
            match self.fetch_formula(&entry.name).await {
                Ok(fetched) => formulas.push(fetched.formula),
                Err(_) => continue,
            }
        }

        Ok(formulas)
    }

    #[allow(dead_code)]
    pub async fn list_all(&self) -> Result<Vec<String>> {
        let url = self.contents_url("formulas");
        let resp = self
            .authenticated_get(&url)
            .send()
            .await
            .context("Failed to list formulas")?;

        if !resp.status().is_success() {
            bail!("GitHub API returned {}", resp.status());
        }

        #[derive(Deserialize)]
        struct DirEntry {
            name: String,
            #[serde(rename = "type")]
            entry_type: String,
        }

        let entries: Vec<DirEntry> = resp.json().await?;
        Ok(entries
            .into_iter()
            .filter(|e| e.entry_type == "dir")
            .map(|e| e.name)
            .collect())
    }
}

fn base64_decode(input: &str) -> Result<String> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &b) in alphabet.iter().enumerate() {
        lookup[b as usize] = i as u8;
    }

    let input = input.trim_end_matches('=');
    let mut bytes = Vec::new();
    let chars: Vec<u8> = input.bytes().collect();

    for chunk in chars.chunks(4) {
        let mut buf: u32 = 0;
        let len = chunk.len();
        for (i, &b) in chunk.iter().enumerate() {
            let val = lookup[b as usize];
            if val == 255 {
                bail!("Invalid base64 character: {}", b as char);
            }
            buf |= (val as u32) << (6 * (3 - i));
        }

        bytes.push((buf >> 16) as u8);
        if len > 2 {
            bytes.push((buf >> 8) as u8);
        }
        if len > 3 {
            bytes.push(buf as u8);
        }
    }

    String::from_utf8(bytes).context("Invalid UTF-8 in decoded content")
}
