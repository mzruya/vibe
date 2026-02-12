use anyhow::{Result, bail};
use chrono::Utc;

use crate::agent;
use crate::cellar::build;
use crate::cellar::link;
use crate::cellar::{Cellar, Receipt};
use crate::config::Config;
use crate::registry::{GitHubRegistry, parse_package_spec};
use crate::ui::Ui;

pub async fn run(package_spec: &str, force: bool, agent_name: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let agent_name = agent_name.unwrap_or(&config.agent.default);
    let total_steps = 6;

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

    // Step 4: Generate code with AI agent
    Ui::step(4, total_steps, "Generating code with AI agent");
    let system_prompt = compose_prompt(&fetched.prompt, package);
    let ai_agent = agent::create_agent(agent_name)?;
    let spinner = Ui::spinner(&format!("Running {} agent...", agent_name));
    let agent_result = ai_agent.generate_dyn(&system_prompt, &src_dir).await?;
    spinner.finish_and_clear();

    if !agent_result.success {
        bail!("AI agent failed to generate code");
    }

    if let Some(cost) = agent_result.cost_usd {
        Ui::info(&format!("Agent cost: ${:.4}", cost));
    }
    if let Some(duration) = agent_result.duration_secs {
        Ui::info(&format!("Generation time: {:.1}s", duration));
    }
    Ui::success("Code generated successfully");

    // Step 5: Build
    Ui::step(5, total_steps, "Building generated code");
    let custom_build = fetched.formula.build.as_ref().and_then(|b| b.command.as_deref());
    let spinner = Ui::spinner("Building...");
    let build_system = build::build(&src_dir, custom_build).await?;
    spinner.finish_and_clear();
    Ui::success(&format!("Built with {}", build_system));

    // Step 6: Link binaries
    Ui::step(6, total_steps, "Installing binaries");
    let found_binaries = build::find_binaries(&src_dir, &build_system, &binary_names)?;
    let mut installed_binary_names = Vec::new();

    for binary_path in &found_binaries {
        let bin_name = binary_path
            .file_name()
            .expect("binary has no filename")
            .to_string_lossy()
            .to_string();

        let cellar_bin = Cellar::copy_binary(binary_path, package, &version, &bin_name)?;
        link::link_binary(&cellar_bin, &bin_name)?;
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
        build_system: format!("{}", build_system),
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

fn compose_prompt(raw_prompt: &str, package: &str) -> String {
    format!(
        r#"You are generating source code for a package called "{}".

IMPORTANT INSTRUCTIONS:
- Write ALL source code files needed for a complete, working project
- The code must compile and produce a working binary
- Write clean, production-quality code
- Include a proper build configuration (Cargo.toml for Rust, go.mod for Go, Makefile for C/C++, etc.)
- Do NOT explain the code, just write the files
- Do NOT run the build - just write the source files

PACKAGE SPECIFICATION:
{}
"#,
        package, raw_prompt
    )
}
