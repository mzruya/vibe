pub mod build;
pub mod link;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub package: String,
    pub version: String,
    pub installed_at: DateTime<Utc>,
    pub agent: String,
    pub cost_usd: Option<f64>,
    pub duration_secs: Option<f64>,
    pub binaries: Vec<String>,
    pub build_system: String,
}

pub struct Cellar;

impl Cellar {
    pub fn package_dir(name: &str, version: &str) -> PathBuf {
        config::cellar_dir().join(name).join(version)
    }

    pub fn src_dir(name: &str, version: &str) -> PathBuf {
        Self::package_dir(name, version).join("src")
    }

    pub fn bin_dir(name: &str, version: &str) -> PathBuf {
        Self::package_dir(name, version).join("bin")
    }

    pub fn receipt_path(name: &str, version: &str) -> PathBuf {
        Self::package_dir(name, version).join("receipt.json")
    }

    #[allow(dead_code)]
    pub fn is_installed(name: &str, version: &str) -> bool {
        Self::receipt_path(name, version).exists()
    }

    pub fn create_dirs(name: &str, version: &str) -> Result<()> {
        std::fs::create_dir_all(Self::src_dir(name, version))?;
        std::fs::create_dir_all(Self::bin_dir(name, version))?;
        Ok(())
    }

    pub fn save_receipt(receipt: &Receipt) -> Result<()> {
        let path = Self::receipt_path(&receipt.package, &receipt.version);
        let json = serde_json::to_string_pretty(receipt)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn load_receipt(name: &str, version: &str) -> Result<Receipt> {
        let path = Self::receipt_path(name, version);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("No receipt found for {}@{}", name, version))?;
        serde_json::from_str(&content).context("Failed to parse receipt")
    }

    pub fn list_installed() -> Result<Vec<Receipt>> {
        let cellar = config::cellar_dir();
        let mut receipts = Vec::new();

        if !cellar.exists() {
            return Ok(receipts);
        }

        for entry in std::fs::read_dir(&cellar)? {
            let entry = entry?;
            if !entry.path().is_dir() {
                continue;
            }
            let package_name = entry.file_name().to_string_lossy().to_string();

            for version_entry in std::fs::read_dir(entry.path())? {
                let version_entry = version_entry?;
                if !version_entry.path().is_dir() {
                    continue;
                }
                let version = version_entry.file_name().to_string_lossy().to_string();

                if let Ok(receipt) = Self::load_receipt(&package_name, &version) {
                    receipts.push(receipt);
                }
            }
        }

        Ok(receipts)
    }

    pub fn find_installed_version(name: &str) -> Result<Option<String>> {
        let package_dir = config::cellar_dir().join(name);
        if !package_dir.exists() {
            return Ok(None);
        }

        for entry in std::fs::read_dir(&package_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let version = entry.file_name().to_string_lossy().to_string();
                if Self::receipt_path(name, &version).exists() {
                    return Ok(Some(version));
                }
            }
        }

        Ok(None)
    }

    pub fn copy_binary(src: &Path, name: &str, version: &str, binary_name: &str) -> Result<PathBuf> {
        let dest = Self::bin_dir(name, version).join(binary_name);
        std::fs::copy(src, &dest)
            .with_context(|| {
                format!(
                    "Failed to copy binary from {} to {}",
                    src.display(),
                    dest.display()
                )
            })?;

        // Ensure executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;
        }

        Ok(dest)
    }

    pub fn remove(name: &str) -> Result<()> {
        let package_dir = config::cellar_dir().join(name);
        if package_dir.exists() {
            std::fs::remove_dir_all(&package_dir)
                .with_context(|| format!("Failed to remove {}", package_dir.display()))?;
        }
        Ok(())
    }
}
