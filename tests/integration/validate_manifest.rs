//! Tests for the `wss-mux validate-manifest <path>` subcommand.

use std::fs;
use std::process::Command;

fn binary() -> Command {
    // `env!` is resolved by Cargo at compile time and points at the
    // built `wss-mux` binary for the current test profile.
    Command::new(env!("CARGO_BIN_EXE_wss-mux"))
}

const VALID_MANIFEST: &str = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
  - stream: presence
    subscribe: ["*"]
"#;

const INVALID_MANIFEST_NOT_YAML: &str = "this: is: not [valid yaml\n";

#[test]
fn validate_manifest_with_ok_yaml_exits_0_and_prints_summary() {
    let tmp = tempdir();
    let path = tmp.path().join("ok.yaml");
    fs::write(&path, VALID_MANIFEST).unwrap();

    let out = binary()
        .args(["validate-manifest", path.to_str().unwrap()])
        .output()
        .expect("spawn");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstdout: {stdout}\nstderr: {stderr}",
        out.status.code()
    );
    assert!(
        stdout.contains("2 streams"),
        "expected stream count in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("chat_messages") && stdout.contains("presence"),
        "expected stream names in stdout, got: {stdout}"
    );
}

#[test]
fn validate_manifest_with_bad_yaml_exits_nonzero_and_prints_error() {
    let tmp = tempdir();
    let path = tmp.path().join("bad.yaml");
    fs::write(&path, INVALID_MANIFEST_NOT_YAML).unwrap();

    let out = binary()
        .args(["validate-manifest", path.to_str().unwrap()])
        .output()
        .expect("spawn");
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "expected non-zero exit on invalid manifest, got {:?}\nstderr: {stderr}",
        out.status.code()
    );
    assert!(
        stderr.contains("error"),
        "expected 'error' in stderr, got: {stderr}"
    );
}

#[test]
fn validate_manifest_with_missing_file_exits_nonzero() {
    let out = binary()
        .args(["validate-manifest", "/nonexistent/path/streams.yaml"])
        .output()
        .expect("spawn");
    assert!(
        !out.status.success(),
        "expected non-zero exit when manifest path doesn't exist"
    );
}

#[test]
fn validate_manifest_without_path_arg_exits_with_usage_error() {
    let out = binary()
        .args(["validate-manifest"])
        .output()
        .expect("spawn");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "missing path arg should be a usage error"
    );
    // Usage diagnostics should mention the subcommand or the missing arg.
    assert!(
        stderr.to_lowercase().contains("usage")
            || stderr.contains("<path>")
            || stderr.contains("validate-manifest"),
        "expected usage hint in stderr, got: {stderr}"
    );
}

// Minimal temp-dir helper — keeps a third-party dep off the test build.
fn tempdir() -> TempDir {
    let mut p = std::env::temp_dir();
    let nonce: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        ^ std::process::id() as u64;
    p.push(format!("wss-mux-validate-{nonce}"));
    std::fs::create_dir_all(&p).unwrap();
    TempDir(p)
}

struct TempDir(std::path::PathBuf);
impl TempDir {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
