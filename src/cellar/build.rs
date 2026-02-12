use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum BuildSystem {
    Cargo,
    Go,
    Make,
    Npm,
    Python,
    Custom(String),
}

impl std::fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildSystem::Cargo => write!(f, "cargo"),
            BuildSystem::Go => write!(f, "go"),
            BuildSystem::Make => write!(f, "make"),
            BuildSystem::Npm => write!(f, "npm"),
            BuildSystem::Python => write!(f, "python"),
            BuildSystem::Custom(cmd) => write!(f, "custom ({})", cmd),
        }
    }
}

pub fn detect_build_system(src_dir: &Path) -> Result<BuildSystem> {
    if src_dir.join("Cargo.toml").exists() {
        return Ok(BuildSystem::Cargo);
    }
    if src_dir.join("go.mod").exists() {
        return Ok(BuildSystem::Go);
    }
    if src_dir.join("Makefile").exists() || src_dir.join("makefile").exists() {
        return Ok(BuildSystem::Make);
    }
    if src_dir.join("package.json").exists() {
        return Ok(BuildSystem::Npm);
    }
    if src_dir.join("setup.py").exists() || src_dir.join("pyproject.toml").exists() {
        return Ok(BuildSystem::Python);
    }

    bail!(
        "Could not detect build system in {}. No Cargo.toml, go.mod, Makefile, package.json, or Python project found.",
        src_dir.display()
    )
}

pub async fn build(src_dir: &Path, custom_command: Option<&str>) -> Result<BuildSystem> {
    let build_system = if let Some(cmd) = custom_command {
        run_custom_build(src_dir, cmd).await?;
        BuildSystem::Custom(cmd.to_string())
    } else {
        let system = detect_build_system(src_dir)?;
        match &system {
            BuildSystem::Cargo => run_cargo_build(src_dir).await?,
            BuildSystem::Go => run_go_build(src_dir).await?,
            BuildSystem::Make => run_make_build(src_dir).await?,
            BuildSystem::Npm => run_npm_build(src_dir).await?,
            BuildSystem::Python => {} // Python scripts don't need compilation
            BuildSystem::Custom(cmd) => run_custom_build(src_dir, cmd).await?,
        }
        system
    };
    Ok(build_system)
}

pub fn find_binaries(src_dir: &Path, build_system: &BuildSystem, binary_names: &[String]) -> Result<Vec<PathBuf>> {
    let mut binaries = Vec::new();

    // If specific binary names are provided, search for them
    if !binary_names.is_empty() {
        for name in binary_names {
            let found = find_binary_by_name(src_dir, name, build_system)?;
            binaries.push(found);
        }
        return Ok(binaries);
    }

    // Auto-detect binaries based on build system
    match build_system {
        BuildSystem::Cargo => {
            let release_dir = src_dir.join("target/release");
            if release_dir.exists() {
                binaries.extend(find_executables_in_dir(&release_dir)?);
            }
        }
        BuildSystem::Go => {
            binaries.extend(find_executables_in_dir(src_dir)?);
        }
        BuildSystem::Make | BuildSystem::Custom(_) => {
            // Check common output locations
            for dir_name in ["bin", "build", "dist", "."] {
                let dir = if dir_name == "." {
                    src_dir.to_path_buf()
                } else {
                    src_dir.join(dir_name)
                };
                if dir.exists() {
                    binaries.extend(find_executables_in_dir(&dir)?);
                }
            }
        }
        BuildSystem::Npm => {
            // Check for bin entries in package.json
            let pkg_path = src_dir.join("package.json");
            if pkg_path.exists() {
                let content = std::fs::read_to_string(&pkg_path)?;
                let pkg: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(bin) = pkg.get("bin") {
                    if let Some(obj) = bin.as_object() {
                        for (_name, path) in obj {
                            if let Some(p) = path.as_str() {
                                let bin_path = src_dir.join(p);
                                if bin_path.exists() {
                                    binaries.push(bin_path);
                                }
                            }
                        }
                    } else if let Some(path) = bin.as_str() {
                        let bin_path = src_dir.join(path);
                        if bin_path.exists() {
                            binaries.push(bin_path);
                        }
                    }
                }
            }
        }
        BuildSystem::Python => {
            // Look for scripts with shebangs
            for entry in std::fs::read_dir(src_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().is_none_or(|ext| ext == "py") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.starts_with("#!") {
                            binaries.push(path);
                        }
                    }
                }
            }
        }
    }

    if binaries.is_empty() {
        bail!("No binaries found after build in {}", src_dir.display());
    }

    Ok(binaries)
}

fn find_binary_by_name(src_dir: &Path, name: &str, build_system: &BuildSystem) -> Result<PathBuf> {
    let candidates: Vec<PathBuf> = match build_system {
        BuildSystem::Cargo => vec![
            src_dir.join("target/release").join(name),
            src_dir.join("target/debug").join(name),
        ],
        BuildSystem::Go => vec![src_dir.join(name)],
        _ => vec![
            src_dir.join("bin").join(name),
            src_dir.join("build").join(name),
            src_dir.join("dist").join(name),
            src_dir.join(name),
        ],
    };

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    bail!("Binary '{}' not found in expected locations", name)
}

fn find_executables_in_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    use std::os::unix::fs::PermissionsExt;
    let mut executables = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let metadata = std::fs::metadata(&path)?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 != 0 {
                // Skip common non-binary files
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if !name.ends_with(".d")
                    && !name.ends_with(".rmeta")
                    && !name.ends_with(".rlib")
                    && !name.starts_with("lib")
                    && !name.starts_with('.')
                    && !name.contains(".dSYM")
                {
                    executables.push(path);
                }
            }
        }
    }

    Ok(executables)
}

async fn run_cargo_build(src_dir: &Path) -> Result<()> {
    let output = tokio::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(src_dir)
        .output()
        .await
        .context("Failed to run cargo build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cargo build failed:\n{}", stderr);
    }
    Ok(())
}

async fn run_go_build(src_dir: &Path) -> Result<()> {
    let output = tokio::process::Command::new("go")
        .args(["build", "-o", "."])
        .current_dir(src_dir)
        .output()
        .await
        .context("Failed to run go build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("go build failed:\n{}", stderr);
    }
    Ok(())
}

async fn run_make_build(src_dir: &Path) -> Result<()> {
    let output = tokio::process::Command::new("make")
        .current_dir(src_dir)
        .output()
        .await
        .context("Failed to run make")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("make failed:\n{}", stderr);
    }
    Ok(())
}

async fn run_npm_build(src_dir: &Path) -> Result<()> {
    // Install dependencies first
    let install = tokio::process::Command::new("npm")
        .args(["install"])
        .current_dir(src_dir)
        .output()
        .await
        .context("Failed to run npm install")?;

    if !install.status.success() {
        let stderr = String::from_utf8_lossy(&install.stderr);
        bail!("npm install failed:\n{}", stderr);
    }

    // Then build
    let output = tokio::process::Command::new("npm")
        .args(["run", "build"])
        .current_dir(src_dir)
        .output()
        .await
        .context("Failed to run npm build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("npm build failed:\n{}", stderr);
    }
    Ok(())
}

async fn run_custom_build(src_dir: &Path, command: &str) -> Result<()> {
    let output = tokio::process::Command::new("sh")
        .args(["-c", command])
        .current_dir(src_dir)
        .output()
        .await
        .with_context(|| format!("Failed to run custom build: {}", command))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Custom build command failed:\n{}", stderr);
    }
    Ok(())
}
