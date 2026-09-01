//! US1: Single node graph community detection tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::unnecessary_debug_formatting,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::path::PathBuf;

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters, TerminationReason};

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

fn load_single_node_edg() -> CsrGraph<String> {
    let path = fixture_path("single_node.edg");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"));

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

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
                parts[2].parse::<f64>().expect("valid float weight")
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

    CsrGraph::from_nodes_and_edges(nodes, edges).expect("valid graph")
}

#[test]
fn single_node_returns_one_community() {
    let graph = load_single_node_edg();
    let result = Leiden::new()
        .with_parameters(LeidenParameters::default())
        .run(&graph)
        .expect("algorithm runs without error on single node graph");

    assert_eq!(result.partition.len(), 1);
    assert_eq!(result.partition[0].0, "0");
    assert_eq!(result.partition[0].1, 0);
    assert_eq!(result.quality, 0.0);
    assert_eq!(result.termination_reason, TerminationReason::Converged);
}

#[test]
fn single_node_zero_edges_is_not_degenerate() {
    let nodes = vec!["isolated_node".to_string()];
    let edges: Vec<Edge<String>> = vec![];
    let graph =
        CsrGraph::from_nodes_and_edges(nodes, edges).expect("valid graph with 1 node 0 edges");

    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(42),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("runs on single-node-zero-edge graph");

    assert_eq!(result.partition.len(), 1);
    assert_eq!(result.partition[0].0, "isolated_node");
    assert_eq!(result.partition[0].1, 0);
    assert_eq!(result.quality, 0.0);
    assert_eq!(result.iterations, 0);
    assert_eq!(
        result.termination_reason,
        TerminationReason::Converged,
        "single-node zero-edge must be Converged, NOT DegenerateInput"
    );
}
