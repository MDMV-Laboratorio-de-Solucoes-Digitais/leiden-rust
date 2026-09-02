//! Integration test: Verification of all CLI exit codes (T091a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

fn assert_no_panic(stderr: &str) {
    assert!(
        !stderr.contains("panicked at"),
        "stderr contained 'panicked at': {stderr}"
    );
    assert!(
        !stderr.contains("thread 'main'"),
        "stderr contained 'thread 'main'': {stderr}"
    );
}

#[test]
fn all_exit_codes_exercised() {
    let bin = env!("CARGO_BIN_EXE_leiden");

    // Exit 0: Success
    let out0 = Command::new(bin)
        .arg(fixture_path("two_cliques.edg"))
        .output()
        .expect("runs CLI");
    assert_eq!(out0.status.code(), Some(0));
    assert_no_panic(&String::from_utf8_lossy(&out0.stderr));

    // Exit 2: Unsupported Format
    let out2 = Command::new(bin)
        .arg("--format")
        .arg("invalid_fmt")
        .arg(fixture_path("two_cliques.edg"))
        .output()
        .expect("runs CLI");
    assert_eq!(out2.status.code(), Some(2));
    assert_no_panic(&String::from_utf8_lossy(&out2.stderr));

    // Exit 3: Invalid Gamma / Cap
    let out3 = Command::new(bin)
        .arg("--gamma")
        .arg("-1.0")
        .arg(fixture_path("two_cliques.edg"))
        .output()
        .expect("runs CLI");
    assert_eq!(out3.status.code(), Some(3));
    assert_no_panic(&String::from_utf8_lossy(&out3.stderr));

    // Exit 4: Malformed input (negative weight)
    let mut temp4 = tempfile::NamedTempFile::new().expect("temp file");
    writeln!(temp4, "a\tb\t-5.0").expect("write temp");
    let out4 = Command::new(bin)
        .arg(temp4.path())
        .output()
        .expect("runs CLI");
    assert_eq!(out4.status.code(), Some(4));
    assert_no_panic(&String::from_utf8_lossy(&out4.stderr));

    // Exit 5: I/O error
    let out5 = Command::new(bin)
        .arg(fixture_path("__nonexistent_file__.edg"))
        .output()
        .expect("runs CLI");
    assert_eq!(out5.status.code(), Some(5));
    assert_no_panic(&String::from_utf8_lossy(&out5.stderr));
}
