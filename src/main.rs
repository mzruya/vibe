mod agent;
mod cellar;
mod cli;
mod config;
mod registry;
mod ui;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    config::ensure_dirs()?;
    config::Config::load()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
            package,
            force,
            agent,
            debug,
        } => {
            ui::banner::print_banner();
            cli::commands::install::run(&package, force, agent.as_deref(), debug).await?;
        }
        Commands::Uninstall { package } => {
            cli::commands::uninstall::run(&package).await?;
        }
        Commands::List => {
            cli::commands::list::run().await?;
        }
        Commands::Search { query } => {
            cli::commands::search::run(&query).await?;
        }
        Commands::Info { package } => {
            cli::commands::info::run(&package).await?;
        }
        Commands::Doctor => {
            ui::banner::print_banner();
            cli::commands::doctor::run().await?;
        }
    }

    Ok(())
}
