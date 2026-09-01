//! Integration test: CLI default seed is 0 when flag is omitted (T066c).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
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
fn cli_default_seed_is_zero_when_flag_omitted() {
    let fixture = fixture_path("two_cliques.edg");
    let bin = env!("CARGO_BIN_EXE_leiden");

    let output = Command::new(bin)
        .arg(fixture)
        .output()
        .expect("runs leiden CLI binary");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("valid utf8 stdout");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    assert_eq!(json["seed"], 0);
}
