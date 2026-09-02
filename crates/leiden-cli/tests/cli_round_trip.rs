//! Integration tests: CLI round-trip and text format sorted output (T078, T079).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

#[test]
fn cli_round_trip() {
    let fixture = fixture_path("two_cliques.edg");
    let bin = env!("CARGO_BIN_EXE_leiden");

    let output = Command::new(bin)
        .arg("--format")
        .arg("json")
        .arg(fixture)
        .output()
        .expect("runs leiden CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("valid utf8 stdout");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    assert_eq!(json["gamma"], 1.0);
    assert_eq!(json["seed"], 0);
    assert_eq!(json["termination_reason"], "converged");
    assert_eq!(json["threading"], "SingleThreaded");
    assert!(json["quality"].as_f64().expect("quality is float") > 0.0);

    let assignments = json["assignments"].as_array().expect("assignments array");
    assert_eq!(assignments.len(), 9);

    let mut seen_nodes = HashSet::new();
    for assign in assignments {
        let node = assign["node"].as_str().expect("node is string");
        assert!(seen_nodes.insert(node));
    }
}

#[test]
fn cli_text_format_is_sorted() {
    let fixture = fixture_path("two_cliques.edg");
    let bin = env!("CARGO_BIN_EXE_leiden");

    let output = Command::new(bin)
        .arg("--format")
        .arg("text")
        .arg(fixture)
        .output()
        .expect("runs leiden CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("valid utf8 stdout");
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 9);

    let mut prev_node: Option<&str> = None;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 2);
        let node = parts[0];
        if let Some(prev) = prev_node {
            assert!(
                prev <= node,
                "lines must be sorted by node id: {prev} <= {node}"
            );
        }
        prev_node = Some(node);
    }
}
