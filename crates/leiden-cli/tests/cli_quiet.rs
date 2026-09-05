//! Integration tests: Log level progress line suppression (T095b, T095c).

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
fn cli_quiet_suppresses_progress_lines() {
    let fixture = fixture_path("two_cliques.edg");
    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");

    let output = Command::new(&bin)
        .arg("--log-level")
        .arg("error")
        .arg(&fixture)
        .output()
        .expect("runs CLI");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("valid utf8 stderr");
    let stdout = String::from_utf8(output.stdout).expect("valid utf8 stdout");

    assert!(!stdout.is_empty(), "stdout must contain partition");
    assert!(
        !stderr.contains("loaded graph:"),
        "stderr must not contain progress line under --log-level error: {stderr}"
    );
    assert!(
        !stderr.contains("iteration "),
        "stderr must not contain iteration line under --log-level error: {stderr}"
    );
    assert!(
        !stderr.contains("terminated after"),
        "stderr must not contain terminated line under --log-level error: {stderr}"
    );
}

#[test]
fn cli_warn_level_suppresses_info_progress_lines() {
    let fixture = fixture_path("two_cliques.edg");
    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");

    // 1. info level -> progress lines present
    let out_info = Command::new(&bin)
        .arg("--log-level")
        .arg("info")
        .arg(&fixture)
        .output()
        .expect("runs CLI with info");

    assert!(out_info.status.success());
    let stderr_info = String::from_utf8(out_info.stderr).expect("valid utf8");
    assert!(stderr_info.contains("loaded graph:"));
    assert!(stderr_info.contains("terminated after"));

    // 2. warn level -> progress lines suppressed
    let out_warn = Command::new(&bin)
        .arg("--log-level")
        .arg("warn")
        .arg(&fixture)
        .output()
        .expect("runs CLI with warn");

    assert!(out_warn.status.success());
    let stderr_warn = String::from_utf8(out_warn.stderr).expect("valid utf8");
    assert!(!stderr_warn.contains("loaded graph:"));
    assert!(!stderr_warn.contains("iteration "));
    assert!(!stderr_warn.contains("terminated after"));
}
