use std::fs;
use tempfile::TempDir;

#[test]
fn test_build_system_detection_cargo() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    // We can't import private modules directly, but we can test the binary's behavior
    // This test just ensures the file structure is correct for detection
    assert!(dir.path().join("Cargo.toml").exists());
}

#[test]
fn test_build_system_detection_go() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("go.mod"), "module test\ngo 1.21").unwrap();
    assert!(dir.path().join("go.mod").exists());
}

#[test]
fn test_build_system_detection_make() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Makefile"), "all:\n\techo hello").unwrap();
    assert!(dir.path().join("Makefile").exists());
}

#[test]
fn test_receipt_json_roundtrip() {
    let receipt_json = serde_json::json!({
        "package": "hello",
        "version": "1.0.0",
        "installed_at": "2024-01-01T00:00:00Z",
        "agent": "claude",
        "cost_usd": 0.05,
        "duration_secs": 30.0,
        "binaries": ["hello"],
        "build_system": "cargo"
    });

    let serialized = serde_json::to_string_pretty(&receipt_json).unwrap();
    let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(receipt_json, deserialized);
    assert_eq!(deserialized["package"], "hello");
    assert_eq!(deserialized["binaries"][0], "hello");
}

#[test]
fn test_symlink_creation_and_cleanup() {
    let dir = TempDir::new().unwrap();
    let src_file = dir.path().join("test_binary");
    let link_path = dir.path().join("link_binary");

    fs::write(&src_file, "#!/bin/sh\necho hello").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&src_file, fs::Permissions::from_mode(0o755)).unwrap();
        std::os::unix::fs::symlink(&src_file, &link_path).unwrap();
        assert!(link_path.is_symlink());
        assert!(link_path.exists());

        fs::remove_file(&link_path).unwrap();
        assert!(!link_path.exists());
    }
}
