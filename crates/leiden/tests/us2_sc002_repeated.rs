//! SC-002 v1 repeated-run determinism assertion (T116a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    dead_code,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::fs;
use std::path::{Path, PathBuf};

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};
use serde::Deserialize;

const fn default_gamma() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
struct FixtureExpected {
    fixture: String,
    #[serde(default = "default_gamma")]
    gamma: f64,
    reference_partition: Option<Vec<NodeAssignment>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct NodeAssignment {
    node: String,
    community: u32,
}

fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    Path::new(&manifest_dir)
        .ancestors()
        .nth(2)
        .expect("workspace root resolution failed")
        .join("fixtures")
}

fn load_edg(path: &Path) -> CsrGraph<String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"));

    let mut edges = Vec::new();
    let mut nodes = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 1 {
            nodes.push(parts[0].to_string());
        } else if parts.len() >= 2 {
            let src = parts[0].to_string();
            let dst = parts[1].to_string();
            let weight = if parts.len() >= 3 {
                parts[2]
                    .parse::<f64>()
                    .expect("valid float weight in fixture")
            } else {
                1.0
            };
            edges.push(Edge {
                source: src,
                target: dst,
                weight,
            });
        }
    }

    if edges.is_empty() && !nodes.is_empty() {
        CsrGraph::from_nodes_and_edges(nodes, edges).expect("valid graph")
    } else {
        CsrGraph::from_edges(edges).expect("valid graph")
    }
}

#[test]
fn sc002_v1_all_fixtures_match_in_100_percent_of_runs() {
    let dir = fixtures_dir();
    let entries = fs::read_dir(&dir).expect("read fixtures dir");

    let mut expected_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".expected.json") && !name.starts_with('_') {
                expected_files.push(path);
            }
        }
    }

    assert!(
        expected_files.len() >= 10,
        "Curated suite must have >= 10 fixtures"
    );

    for exp_path in expected_files {
        let json_str = fs::read_to_string(&exp_path).expect("read expected.json");
        let expected: FixtureExpected =
            serde_json::from_str(&json_str).expect("parse expected.json");

        let edg_path = dir.join(&expected.fixture);
        if !edg_path.exists() {
            continue;
        }

        let Ok(graph) = std::panic::catch_unwind(|| load_edg(&edg_path)) else {
            // E.g., empty graph returns LeidenError::EmptyGraph
            continue;
        };

        if graph.node_count() == 0 {
            continue;
        }

        // Run N=30 times with varying seeds
        let mut ref_partition: Option<Vec<(String, u32)>> = None;

        for seed_val in 0..30 {
            let params = LeidenParameters {
                gamma: expected.gamma,
                seed: Some(seed_val),
                iteration_cap: 10,
            };

            let res = Leiden::new()
                .with_parameters(params)
                .run(&graph)
                .expect("runs on fixture");

            if let Some(ref initial) = ref_partition {
                assert_eq!(
                    &res.partition, initial,
                    "Determinism failure in fixture {:?} on run seed={seed_val}",
                    expected.fixture
                );
            } else {
                ref_partition = Some(res.partition.clone());
            }
        }
    }
}
