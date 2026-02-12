use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn vibe() -> Command {
    Command::cargo_bin("vibe").unwrap()
}

#[test]
fn test_help() {
    vibe()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-powered package manager"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("uninstall"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn test_version() {
    vibe()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("vibe 0.1.0"));
}

#[test]
fn test_install_help() {
    vibe()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install a package"))
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("--agent"));
}

#[test]
fn test_doctor() {
    vibe()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vibe Doctor"))
        .stdout(predicate::str::contains("AI Agents"))
        .stdout(predicate::str::contains("Build Tools"));
}

#[test]
fn test_list_runs() {
    // Just verify list command runs without error
    // (can't assume empty cellar in dev environment)
    vibe()
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_uninstall_not_installed() {
    vibe()
        .args(["uninstall", "nonexistent-package-xyz"])
        .assert()
        .success()
        .stderr(predicate::str::contains("is not installed"));
}

#[test]
fn test_search_real_registry() {
    vibe()
        .args(["search", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_info_real_registry() {
    vibe()
        .args(["info", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello v1.0.0"))
        .stdout(predicate::str::contains("friendly hello world"));
}

#[test]
fn test_search_no_results() {
    vibe()
        .args(["search", "zzzznonexistent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No packages found"));
}

#[test]
fn test_info_nonexistent() {
    vibe()
        .args(["info", "zzzznonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_install_missing_package_arg() {
    vibe()
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_info_with_version() {
    vibe()
        .args(["info", "hello@1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello v1.0.0"));
}
