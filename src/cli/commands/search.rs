use anyhow::Result;

use crate::config::Config;
use crate::registry::GitHubRegistry;
use crate::ui::Ui;

pub async fn run(query: &str) -> Result<()> {
    let config = Config::load()?;
    let registry = GitHubRegistry::new(&config.registry.owner, &config.registry.repo);

    let spinner = Ui::spinner(&format!("Searching for '{}'...", query));
    let formulas = registry.search(query).await?;
    spinner.finish_and_clear();

    if formulas.is_empty() {
        Ui::info(&format!("No packages found matching '{}'", query));
        return Ok(());
    }

    Ui::header(&format!("Found {} package(s)", formulas.len()));
    println!();

    for formula in &formulas {
        println!(
            "  {} v{}",
            console::style(&formula.package.name).bold().cyan(),
            formula.package.version,
        );
        println!("    {}", formula.package.description);
        if let Some(ref homepage) = formula.package.homepage {
            println!("    {}", console::style(homepage).dim());
        }
        println!();
    }

    Ok(())
}
