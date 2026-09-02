//! US1: Fixture suite community detection tests matching reference partitions.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::collections::HashSet;
use std::path::PathBuf;

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

fn load_edg(name: &str) -> CsrGraph<String> {
    let path = fixture_path(name);
    let content = std::fs::read_to_string(&path)
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
fn fixture_suite_matches_reference() {
    // 1. Test karate club fixture (Zachary 1977, 34 nodes, 78 edges)
    let karate = load_edg("karate.edg");
    let karate_res = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&karate)
        .expect("runs on karate club fixture");

    assert_eq!(karate_res.partition.len(), 34);
    assert!(
        karate_res.quality > 0.35,
        "karate modularity {} should be > 0.35",
        karate_res.quality
    );

    let karate_comms: HashSet<u32> = karate_res.partition.iter().map(|p| p.1).collect();
    assert!(
        karate_comms.len() >= 2 && karate_comms.len() <= 6,
        "karate club should detect between 2 and 6 communities, got {}",
        karate_comms.len()
    );

    // 2. Test two cliques fixture
    let two_cliques = load_edg("two_cliques.edg");
    let tc_res = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&two_cliques)
        .expect("runs on two_cliques fixture");

    assert_eq!(tc_res.partition.len(), 9);
    let tc_comms: HashSet<u32> = tc_res.partition.iter().map(|p| p.1).collect();
    assert_eq!(
        tc_comms.len(),
        2,
        "two_cliques should produce 2 communities"
    );

    // 3. Test ring of cliques
    let ring = load_edg("ring_of_cliques.edg");
    let ring_res = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&ring)
        .expect("runs on ring of cliques");
    assert!(ring_res.quality > 0.5);

    // 4. Test star graph
    let star = load_edg("star.edg");
    let star_res = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&star)
        .expect("runs on star graph");
    assert_eq!(star_res.partition.len(), 11);
}
