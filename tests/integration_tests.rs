use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to run vibe with a custom VIBE_HOME
fn vibe_with_home(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("vibe").unwrap();
    cmd.env("VIBE_HOME", home);
    cmd
}

/// Helper to create a temp vibe home with required directories
fn setup_vibe_home() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("bin")).unwrap();
    fs::create_dir_all(dir.path().join("cellar")).unwrap();
    fs::create_dir_all(dir.path().join("cache")).unwrap();
    dir
}

// ============================================================================
// Registry Integration Tests (hitting real GitHub registry)
// ============================================================================

#[test]
fn test_search_returns_multiple_results() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["search", ""])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("fizzbuzz"));
}

#[test]
fn test_search_case_insensitive() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["search", "HELLO"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_search_by_description() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["search", "countdown"])
        .assert()
        .success()
        .stdout(predicate::str::contains("countdown"));
}

#[test]
fn test_info_shows_all_versions() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["info", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("1.0.0"));
}

#[test]
fn test_info_specific_version() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["info", "hello@1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello v1.0.0"));
}

#[test]
fn test_info_nonexistent_version() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["info", "hello@99.99.99"])
        .assert()
        .failure();
}

// ============================================================================
// Cellar Integration Tests (using isolated temp directories)
// ============================================================================

#[test]
fn test_list_empty_cellar() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No packages installed"));
}

#[test]
fn test_uninstall_nonexistent_package() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["uninstall", "nonexistent"])
        .assert()
        .success()
        .stderr(predicate::str::contains("is not installed"));
}

#[test]
fn test_list_with_manually_created_package() {
    let home = setup_vibe_home();

    // Manually create a package in the cellar (simulating a previous install)
    let pkg_dir = home.path().join("cellar/test-pkg/1.0.0");
    fs::create_dir_all(&pkg_dir).unwrap();

    // Create a receipt
    let receipt = serde_json::json!({
        "package": "test-pkg",
        "version": "1.0.0",
        "installed_at": "2024-01-01T00:00:00Z",
        "agent": "test",
        "cost_usd": 0.0,
        "duration_secs": 1.0,
        "binaries": ["test-pkg"]
    });
    fs::write(pkg_dir.join("receipt.json"), serde_json::to_string_pretty(&receipt).unwrap()).unwrap();

    vibe_with_home(home.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-pkg"))
        .stdout(predicate::str::contains("1.0.0"));
}

#[test]
fn test_uninstall_removes_package_and_symlinks() {
    let home = setup_vibe_home();

    // Create a package with a binary
    let pkg_dir = home.path().join("cellar/uninstall-test/1.0.0");
    let bin_dir = pkg_dir.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    // Create a binary
    let binary_path = bin_dir.join("uninstall-test");
    fs::write(&binary_path, "#!/bin/sh\necho test").unwrap();

    // Create a symlink in bin dir
    let symlink_path = home.path().join("bin/uninstall-test");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&binary_path, &symlink_path).unwrap();

    // Create receipt
    let receipt = serde_json::json!({
        "package": "uninstall-test",
        "version": "1.0.0",
        "installed_at": "2024-01-01T00:00:00Z",
        "agent": "test",
        "binaries": ["uninstall-test"]
    });
    fs::write(pkg_dir.join("receipt.json"), serde_json::to_string_pretty(&receipt).unwrap()).unwrap();

    // Verify package exists
    assert!(pkg_dir.exists());
    assert!(symlink_path.is_symlink());

    // Run uninstall
    vibe_with_home(home.path())
        .args(["uninstall", "uninstall-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("uninstalled"));

    // Verify package and symlink are gone
    assert!(!pkg_dir.exists());
    assert!(!symlink_path.exists());
}

#[test]
fn test_list_multiple_packages() {
    let home = setup_vibe_home();

    // Create two packages
    for (name, version) in [("pkg-a", "1.0.0"), ("pkg-b", "2.0.0")] {
        let pkg_dir = home.path().join(format!("cellar/{}/{}", name, version));
        fs::create_dir_all(&pkg_dir).unwrap();

        let receipt = serde_json::json!({
            "package": name,
            "version": version,
            "installed_at": "2024-01-01T00:00:00Z",
            "agent": "test",
            "binaries": [name]
        });
        fs::write(pkg_dir.join("receipt.json"), serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    }

    vibe_with_home(home.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("pkg-a"))
        .stdout(predicate::str::contains("pkg-b"));
}

// ============================================================================
// Doctor Integration Tests
// ============================================================================

#[test]
fn test_doctor_with_custom_home() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vibe Doctor"))
        .stdout(predicate::str::contains("AI Agents"));
}

#[test]
fn test_doctor_warns_about_path() {
    let home = setup_vibe_home();
    // The bin dir won't be in PATH, so doctor should warn
    vibe_with_home(home.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("not in your PATH").or(predicate::str::contains("PATH")));
}

// ============================================================================
// Config Integration Tests
// ============================================================================

#[test]
fn test_config_created_on_first_run() {
    let home = setup_vibe_home();

    // Run any command to trigger config creation
    vibe_with_home(home.path())
        .arg("list")
        .assert()
        .success();

    // Verify config was created
    let config_path = home.path().join("config.toml");
    assert!(config_path.exists());

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("mzruya"));
    assert!(config_content.contains("vibe-registry"));
    assert!(config_content.contains("claude"));
}

// ============================================================================
// Install Integration Tests (without real AI agent)
// ============================================================================

#[test]
fn test_install_fetches_formula_from_registry() {
    let home = setup_vibe_home();

    // This test verifies the install command can fetch from the real registry
    // but will fail at the agent step (which is expected without a real agent)
    // We just want to verify it gets past the formula fetch step
    let output = vibe_with_home(home.path())
        .args(["install", "hello"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have fetched the formula successfully
    assert!(
        stdout.contains("Found hello") || stdout.contains("Fetching formula"),
        "Expected formula fetch output, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_install_nonexistent_package() {
    let home = setup_vibe_home();

    vibe_with_home(home.path())
        .args(["install", "zzzznonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Failed")));
}

#[test]
fn test_install_with_invalid_version() {
    let home = setup_vibe_home();

    vibe_with_home(home.path())
        .args(["install", "hello@99.99.99"])
        .assert()
        .failure();
}

#[test]
fn test_install_already_installed_no_force() {
    let home = setup_vibe_home();

    // Create an existing installation
    let pkg_dir = home.path().join("cellar/hello/1.0.0");
    fs::create_dir_all(&pkg_dir).unwrap();

    let receipt = serde_json::json!({
        "package": "hello",
        "version": "1.0.0",
        "installed_at": "2024-01-01T00:00:00Z",
        "agent": "test",
        "binaries": ["hello"]
    });
    fs::write(pkg_dir.join("receipt.json"), serde_json::to_string_pretty(&receipt).unwrap()).unwrap();

    // Try to install without --force
    vibe_with_home(home.path())
        .args(["install", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already installed"));
}

// ============================================================================
// CLI Argument Validation Tests
// ============================================================================

#[test]
fn test_install_requires_package_argument() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_uninstall_requires_package_argument() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("uninstall")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_info_requires_package_argument() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("info")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_search_requires_query_argument() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_unknown_command() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .arg("unknown-command")
        .assert()
        .failure();
}

#[test]
fn test_install_invalid_agent() {
    let home = setup_vibe_home();
    vibe_with_home(home.path())
        .args(["install", "hello", "--agent", "invalid-agent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown agent"));
}
