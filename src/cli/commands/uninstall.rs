use anyhow::Result;

use crate::cellar::link;
use crate::cellar::Cellar;
use crate::ui::Ui;

pub async fn run(package: &str) -> Result<()> {
    let version = match Cellar::find_installed_version(package)? {
        Some(v) => v,
        None => {
            Ui::error(&format!("{} is not installed", package));
            return Ok(());
        }
    };

    Ui::header(&format!("Uninstalling {} v{}...", package, version));

    // Remove symlinks
    if let Ok(receipt) = Cellar::load_receipt(package, &version) {
        for binary in &receipt.binaries {
            if link::unlink_binary(binary)? {
                Ui::info(&format!("Unlinked: {}", binary));
            }
        }
    }

    // Remove cellar directory
    Cellar::remove(package)?;

    Ui::success(&format!("{} v{} uninstalled successfully", package, version));
    Ok(())
}
