pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vibe", about = "AI-powered package manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a package using AI code generation
    Install {
        /// Package name to install
        package: String,
        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,
        /// AI agent to use (default: claude)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Uninstall a package
    Uninstall {
        /// Package name to uninstall
        package: String,
    },
    /// List installed packages
    List,
    /// Search the registry for packages
    Search {
        /// Search query
        query: String,
    },
    /// Show package details
    Info {
        /// Package name
        package: String,
    },
    /// Check system health
    Doctor,
}
