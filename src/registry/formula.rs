use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    pub package: PackageMetadata,
    #[serde(default)]
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub binaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub command: Option<String>,
    #[serde(default)]
    pub binary_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FetchedFormula {
    pub formula: Formula,
    pub prompt: String,
}
