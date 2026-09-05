//! Integration test: CLI format validation (T077).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

#[test]
fn cli_rejects_unknown_format() {
    let fixture = fixture_path("two_cliques.edg");
    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");

    let output = Command::new(&bin)
        .arg("--format")
        .arg("yaml")
        .arg(fixture)
        .output()
        .expect("runs leiden CLI");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("valid utf8 stderr");
    assert!(
        stderr.contains("unsupported output format 'yaml'; expected 'json' or 'text'"),
        "stderr was: {stderr}"
    );
}
