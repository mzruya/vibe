use anyhow::{Result, bail};
use chrono::Utc;

use crate::agent;
use crate::cellar::link;
use crate::cellar::{Cellar, Receipt};
use crate::config::Config;
use crate::registry::{GitHubRegistry, parse_package_spec};
use crate::ui::Ui;

pub async fn run(package_spec: &str, force: bool, agent_name: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let agent_name = agent_name.unwrap_or(&config.agent.default);
    let total_steps = 5;

    // Parse package@version syntax
    let (package, requested_version) = parse_package_spec(package_spec);

    // Step 1: Check if already installed
    Ui::step(1, total_steps, "Checking installation status");
    if let Some(installed_version) = Cellar::find_installed_version(package)? {
        if !force {
            Ui::warning(&format!(
                "{} v{} is already installed. Use --force to reinstall.",
                package, installed_version
            ));
            return Ok(());
        }
        Ui::info("Force reinstall requested, removing existing installation...");
        // Unlink existing binaries
        if let Ok(receipt) = Cellar::load_receipt(package, &installed_version) {
            for binary in &receipt.binaries {
                link::unlink_binary(binary)?;
            }
        }
        Cellar::remove(package)?;
    }

    // Step 2: Fetch formula from registry
    Ui::step(2, total_steps, "Fetching formula from registry");
    let spinner = Ui::spinner("Downloading formula...");
    let registry = GitHubRegistry::new(&config.registry.owner, &config.registry.repo);
    let fetched = registry.fetch_formula(package, requested_version).await?;
    spinner.finish_and_clear();
    Ui::success(&format!(
        "Found {} v{}: {}",
        fetched.formula.package.name,
        fetched.formula.package.version,
        fetched.formula.package.description
    ));

    let version = fetched.formula.package.version.clone();
    let binary_names = fetched.formula.package.binaries.clone();

    // Step 3: Create workspace
    Ui::step(3, total_steps, "Preparing workspace");
    Cellar::create_dirs(package, &version)?;
    let src_dir = Cellar::src_dir(package, &version);
    Ui::info(&format!("Workspace: {}", src_dir.display()));

    // Step 4: Generate and build with AI agent
    Ui::step(4, total_steps, "Generating and building with AI agent");
    let system_prompt = compose_prompt(&fetched.prompt, package, &binary_names);
    let ai_agent = agent::create_agent(agent_name)?;
    let agent_result = ai_agent.generate_dyn(&system_prompt, &src_dir).await?;

    if !agent_result.success {
        bail!("AI agent failed to generate code");
    }

    // Format success message with cost/duration metrics
    let metrics: Vec<String> = [
        agent_result.cost_usd.map(|c| format!("${:.2}", c)),
        agent_result.duration_secs.map(|d| format!("{:.1}s", d)),
    ]
    .into_iter()
    .flatten()
    .collect();

    let success_msg = if metrics.is_empty() {
        "Code generated and built successfully".to_string()
    } else {
        format!(
            "Code generated and built successfully ({})",
            metrics.join(", ")
        )
    };
    Ui::success(&success_msg);

    // Step 5: Link binaries
    Ui::step(5, total_steps, "Installing binaries");
    let bin_dir = src_dir.join("bin");
    if !bin_dir.exists() {
        bail!("AI agent did not create bin/ directory with binaries");
    }

    let mut installed_binary_names = Vec::new();
    for bin_name in &binary_names {
        let binary_path = bin_dir.join(bin_name);
        if !binary_path.exists() {
            bail!("Expected binary '{}' not found in bin/", bin_name);
        }

        let cellar_bin = Cellar::copy_binary(&binary_path, package, &version, bin_name)?;
        link::link_binary(&cellar_bin, bin_name)?;
        installed_binary_names.push(bin_name.clone());
        Ui::success(&format!("Linked: {}", bin_name));
    }

    // Save receipt
    let receipt = Receipt {
        package: package.to_string(),
        version: version.clone(),
        installed_at: Utc::now(),
        agent: agent_name.to_string(),
        cost_usd: agent_result.cost_usd,
        duration_secs: agent_result.duration_secs,
        binaries: installed_binary_names,
    };
    Cellar::save_receipt(&receipt)?;

    // Check PATH
    if !link::is_bin_in_path() {
        println!();
        Ui::warning("~/.vibe/bin is not in your PATH. Add it with:");
        Ui::info("  export PATH=\"$HOME/.vibe/bin:$PATH\"");
    }

    println!();
    Ui::success(&format!("{} v{} installed successfully!", package, version));

    Ok(())
}

fn compose_prompt(raw_prompt: &str, package: &str, binaries: &[String]) -> String {
    let binary_list = binaries.join(", ");
    format!(
        r#"You are building a package called "{package}".

YOUR TASK:
1. Write all source code needed
2. Build/compile it
3. Copy the final binary to ./bin/

REQUIREMENTS:
- The final binary MUST be placed at: ./bin/{binary_list}
- The binary must be executable and work correctly
- Test that it runs before finishing

PACKAGE SPECIFICATION:
{raw_prompt}
"#
    )
}
