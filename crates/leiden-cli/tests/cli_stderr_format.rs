//! Integration tests: Canonical CLI stderr format verification (T084a, T094a).

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

const STDERR_REGEX: &str = r"^(loaded graph.*|iteration \d+(: quality=-?\d+(\.\d+)?)?|terminated after \d+ iterations(: (converged|iteration_cap|degenerate_input))?|(malformed|io):.*)$";

#[test]
fn cli_stderr_matches_cli_schema_1_5_spec() {
    let regex = regex::Regex::new(STDERR_REGEX).expect("valid regex");

    let canonical_lines = [
        "loaded graph: nodes=34 edges=78 total_weight=156.0",
        "iteration 1: quality=0.4198",
        "iteration 2: quality=0.4231",
        "terminated after 2 iterations: converged",
        "malformed: bad.edg:7: invalid weight `-1.0`: must be finite and ≥ 0",
        "io: fixtures/__missing__.edg: No such file or directory (os error 2)",
    ];

    for line in canonical_lines {
        assert!(
            regex.is_match(line),
            "canonical line '{line}' must match schema regex"
        );
    }
}

#[test]
fn cli_stderr_matches_cli_schema_1_5_at_runtime() {
    let regex = regex::Regex::new(STDERR_REGEX).expect("valid regex");
    let fixture = fixture_path("two_cliques.edg");
    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");

    let output = Command::new(&bin)
        .arg("--log-level")
        .arg("info")
        .arg(fixture)
        .output()
        .expect("runs CLI");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("valid utf8");
    let stdout = String::from_utf8(output.stdout).expect("valid utf8");

    // Partition on stdout, not stderr
    assert!(!stdout.is_empty());

    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        assert!(
            regex.is_match(trimmed),
            "stderr line '{trimmed}' must match schema regex"
        );
        assert!(!trimmed.contains("panicked at"));
        assert!(!trimmed.contains("thread 'main'"));
    }
}
