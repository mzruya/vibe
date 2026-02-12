use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config;

pub fn link_binary(binary_path: &Path, name: &str) -> Result<PathBuf> {
    let bin_dir = config::bin_dir();
    std::fs::create_dir_all(&bin_dir)?;

    let link_path = bin_dir.join(name);

    // Remove existing symlink if present
    if link_path.exists() || link_path.is_symlink() {
        std::fs::remove_file(&link_path)
            .with_context(|| format!("Failed to remove existing link: {}", link_path.display()))?;
    }

    std::os::unix::fs::symlink(binary_path, &link_path)
        .with_context(|| {
            format!(
                "Failed to create symlink {} -> {}",
                link_path.display(),
                binary_path.display()
            )
        })?;

    Ok(link_path)
}

pub fn unlink_binary(name: &str) -> Result<bool> {
    let link_path = config::bin_dir().join(name);
    if link_path.exists() || link_path.is_symlink() {
        std::fs::remove_file(&link_path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn is_bin_in_path() -> bool {
    let bin_dir = config::bin_dir();
    if let Ok(path) = std::env::var("PATH") {
        path.split(':').any(|p| PathBuf::from(p) == bin_dir)
    } else {
        false
    }
}
