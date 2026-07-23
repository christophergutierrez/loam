//! End-to-end CLI tests — drive the built `loam` binary against the fixture
//! bundle with an isolated spool (LOAM_SPOOL) so no state leaks into the tree.

use std::path::Path;
use std::process::Command;

fn fixture(sub: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_repo")
        .join(sub)
}

#[test]
fn cli_get_prints_tier_and_body_from_nested_cwd() {
    let bin = env!("CARGO_BIN_EXE_loam");
    let spool = std::env::temp_dir().join("loam-cli-test-a.sqlite");
    let _ = std::fs::remove_file(&spool);
    let out = Command::new(bin)
        .args(["get", "greeting-contract"])
        .current_dir(fixture("src/deep"))
        .env("LOAM_SPOOL", &spool)
        .env("LOAM_HARNESS", "test")
        .output()
        .expect("run loam");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("verified"), "expected trust tier in: {s}");
    assert!(s.contains("greet"), "expected body in: {s}");
}

#[test]
fn cli_get_json_flags_stale() {
    let bin = env!("CARGO_BIN_EXE_loam");
    let spool = std::env::temp_dir().join("loam-cli-test-b.sqlite");
    let _ = std::fs::remove_file(&spool);
    let out = Command::new(bin)
        .args(["get", "stale-example", "--json"])
        .current_dir(fixture(""))
        .env("LOAM_SPOOL", &spool)
        .output()
        .expect("run loam");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"stale\": true"), "expected stale=true in: {s}");
    assert!(s.contains("src/greeting.rs"), "expected changed anchor in: {s}");
}

#[test]
fn cli_errors_when_no_bundle() {
    let bin = env!("CARGO_BIN_EXE_loam");
    let out = Command::new(bin)
        .args(["get", "whatever"])
        .current_dir(std::env::temp_dir())
        .env("LOAM_SPOOL", std::env::temp_dir().join("loam-cli-test-c.sqlite"))
        .output()
        .expect("run loam");
    assert!(!out.status.success(), "should fail with no bundle");
}
