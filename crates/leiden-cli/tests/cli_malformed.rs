//! Integration tests: CLI error handling on malformed inputs (T080, T081, T082, T083).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::io::Write;
use std::process::Command;

#[test]
fn malformed_negative_weight() {
    let mut temp = tempfile::NamedTempFile::new().expect("temp file");
    writeln!(temp, "a\tb\t-1.0").expect("write temp");

    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");
    let output = Command::new(&bin)
        .arg(temp.path())
        .output()
        .expect("runs CLI");

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).expect("valid utf8 stderr");
    assert!(
        stderr.contains("invalid weight `-1.0`: must be finite and ≥ 0"),
        "stderr was: {stderr}"
    );
}

#[test]
fn malformed_self_loop() {
    let mut temp = tempfile::NamedTempFile::new().expect("temp file");
    writeln!(temp, "a\ta\t1.0").expect("write temp");

    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");
    let output = Command::new(&bin)
        .arg(temp.path())
        .output()
        .expect("runs CLI");

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).expect("valid utf8 stderr");
    assert!(
        stderr.contains("self-loop on node 'a': not permitted"),
        "stderr was: {stderr}"
    );
}

#[test]
fn malformed_dangling_node() {
    let mut temp = tempfile::NamedTempFile::new().expect("temp file");
    let json_doc = r#"{
        "nodes": ["a", "b"],
        "edges": [["a", "b"], ["a", "c"]],
        "weights": [1.0, 1.0]
    }"#;
    writeln!(temp, "{json_doc}").expect("write temp");

    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");
    let output = Command::new(&bin)
        .arg(temp.path())
        .output()
        .expect("runs CLI");

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).expect("valid utf8 stderr");
    assert!(
        stderr.contains("node id `c` appears in edges but not in any declared node set"),
        "stderr was: {stderr}"
    );
}

#[test]
fn malformed_invalid_gamma() {
    let mut temp = tempfile::NamedTempFile::new().expect("temp file");
    writeln!(temp, "a\tb\t1.0").expect("write temp");

    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");
    let output = Command::new(&bin)
        .arg("--gamma")
        .arg("0.0")
        .arg(temp.path())
        .output()
        .expect("runs CLI");

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8(output.stderr).expect("valid utf8 stderr");
    assert!(
        stderr.contains("resolution γ must be > 0; got 0"),
        "stderr was: {stderr}"
    );
}
