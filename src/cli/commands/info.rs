use anyhow::Result;

use crate::cellar::Cellar;
use crate::config::Config;
use crate::registry::{GitHubRegistry, parse_package_spec};
use crate::ui::Ui;

pub async fn run(package_spec: &str) -> Result<()> {
    let config = Config::load()?;
    let registry = GitHubRegistry::new(&config.registry.owner, &config.registry.repo);

    let (package, requested_version) = parse_package_spec(package_spec);

    // Get package info from index for version list
    let spinner = Ui::spinner("Fetching package info...");
    let index_entry = registry.get_package_info(package).await?;
    let fetched = registry.fetch_formula(package, requested_version).await?;
    spinner.finish_and_clear();

    let pkg = &fetched.formula.package;

    Ui::header(&format!("{} v{}", pkg.name, pkg.version));
    println!();
    Ui::label_value("Description", &pkg.description);

    if let Some(ref homepage) = pkg.homepage {
        Ui::label_value("Homepage", homepage);
    }
    if let Some(ref license) = pkg.license {
        Ui::label_value("License", license);
    }
    if !pkg.binaries.is_empty() {
        Ui::label_value("Binaries", &pkg.binaries.join(", "));
    }
    if let Some(cmd) = fetched
        .formula
        .build
        .as_ref()
        .and_then(|b| b.command.as_ref())
    {
        Ui::label_value("Build", cmd);
    }

    // Show available versions
    if index_entry.versions.len() > 1 {
        Ui::label_value("Versions", &index_entry.versions.join(", "));
    }

    // Check if installed locally
    if let Some(version) = Cellar::find_installed_version(package)? {
        println!();
        Ui::success(&format!("Installed: v{}", version));
        if let Ok(receipt) = Cellar::load_receipt(package, &version) {
            Ui::label_value("Agent", &receipt.agent);
            Ui::label_value(
                "Installed at",
                &receipt.installed_at.format("%Y-%m-%d %H:%M").to_string(),
            );
        }
    } else {
        println!();
        Ui::info(&format!("Not installed. Run: vibe install {}", package));
    }

    Ok(())
}
