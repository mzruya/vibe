use anyhow::Result;

use crate::cellar::Cellar;
use crate::ui::Ui;

pub async fn run() -> Result<()> {
    let receipts = Cellar::list_installed()?;

    if receipts.is_empty() {
        Ui::info("No packages installed. Try: vibe install <package>");
        return Ok(());
    }

    Ui::header(&format!("Installed packages ({})", receipts.len()));
    println!();

    for receipt in &receipts {
        let binaries = receipt.binaries.join(", ");
        println!(
            "  {} v{} ({})",
            console::style(&receipt.package).bold(),
            receipt.version,
            console::style(&binaries).dim(),
        );
        if let Some(cost) = receipt.cost_usd {
            println!("    Agent: {} (${:.4})", receipt.agent, cost);
        }
        println!(
            "    Installed: {}",
            receipt.installed_at.format("%Y-%m-%d %H:%M")
        );
    }

    Ok(())
}
