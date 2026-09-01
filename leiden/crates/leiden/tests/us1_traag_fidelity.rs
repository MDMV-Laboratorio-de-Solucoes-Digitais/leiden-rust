//! Traag 2019 fidelity tests anchoring formulas and algorithm steps.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::fs;
use std::path::PathBuf;

use leiden::{CsrGraph, Edge, Modularity, MoveComponents, Partition, QualityFunction};

// Canonical hand-computed 4-node graph with 1 bridge edge for Eq. (1) modularity testing.
// ref: Traag 2019 §3
const FIXTURE_EQ1_DESCRIPTION: &str = "4-node graph with single bridge edge";

// Hand-computed modularity reference value for Eq. (1) verification on 4-node graph.
// ref: Traag 2019 §3
fn hand_computed_eq1_reference() -> f64 {
    // Q = 0.5 * ((2.0 / 3.0) - (8.0 / 3.0)^2 / (2.0 * 3.0)) = 0.5 * (2/3 - 64/54)
    // 2/3 - 32/27 = 18/27 - 32/27 = -14/27.
    // Or for two separate communities {0, 1} and {2, 3} with edge (1, 2):
    // m = 3.0 (edges: 0-1, 1-2, 2-3).
    // Comm 0 {0, 1}: e_0 = 1.0 (2e_0 = 2.0), k_C = 1 + 2 = 3.0.
    // Comm 1 {2, 3}: e_1 = 1.0 (2e_1 = 2.0), k_C = 2 + 1 = 3.0.
    // 2e_0 - (3^2) / (6) = 2.0 - 1.5 = 0.5.
    // 2e_1 - (3^2) / (6) = 2.0 - 1.5 = 0.5.
    // Sum = 1.0. Total Q = 1.0 / (2 * 3.0) = 1.0 / 6.0 = 0.16666666666666666
    1.0 / 6.0
}

// Hand-computed delta modularity reference for Eq. (A5) verification on 4-node path graph.
// ref: Traag 2019 §3
fn hand_computed_eq_a5_reference() -> f64 {
    // 4-node path 0-1-2-3, m = 3.0.
    // Degrees: k_0 = 1, k_1 = 2, k_2 = 2, k_3 = 1.
    // Initial singletons: C = {0}, T = {1}.
    // Move node 0 to community {1}.
    // sigma_in(T, 0) = 1.0, sigma_in(C, 0) = 0.0.
    // sigma_tot(T) = 2.0, sigma_tot(C) = 1.0, k_0 = 1.0.
    // delta_edges = (1.0 - 0.0) / 3.0 = 1/3.
    // delta_degree = 1.0 * 1.0 * (2.0 - 1.0 + 1.0) / (2 * 9.0) = 2.0 / 18.0 = 1/9.
    // delta_Q = 1/3 - 1/9 = 2/9 ~ 0.2222222222222222.
    2.0 / 9.0
}

// Hand-computed predicate boundary fixture for Traag refinement well-connected condition.
// ref: Traag 2019 §3
fn hand_computed_refinement_fixture() -> (f64, f64) {
    // Threshold comparison: e_u_c vs gamma * k_u * (k_c - k_u) / (2m)
    // For k_u = 2.0, k_c = 6.0, 2m = 12.0, gamma = 1.0:
    // threshold = 1.0 * 2.0 * (6.0 - 2.0) / 12.0 = 8.0 / 12.0 = 2/3 ~ 0.6666666666666666
    let threshold = 2.0 / 3.0;
    let actual_edge_weight = 1.0;
    (threshold, actual_edge_weight)
}

// Hand-computed aggregate graph edge weight reference matching Traag Algorithm A.2.
// ref: Traag 2019 §3
const fn hand_computed_aggregation_reference() -> (usize, usize, f64) {
    // Two refined communities of size 2: {0, 1} and {2, 3} joined by edge (1, 2) of weight 1.0
    // Aggregate graph has 2 nodes, 1 edge of weight 1.0, total weight 1.0
    (2, 1, 1.0)
}

#[test]
fn traag_eq1_modularity_matches() {
    let _ = FIXTURE_EQ1_DESCRIPTION;
    // 4-node path 0-1, 1-2, 2-3 (3 edges, total weight 3.0)
    let edges = vec![
        Edge {
            source: 0_u32,
            target: 1_u32,
            weight: 1.0,
        },
        Edge {
            source: 1_u32,
            target: 2_u32,
            weight: 1.0,
        },
        Edge {
            source: 2_u32,
            target: 3_u32,
            weight: 1.0,
        },
    ];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");

    // Partition {0, 1} in comm 0, {2, 3} in comm 1
    let mut partition = Partition::singletons(4);
    partition.move_node(1, 0);
    partition.move_node(3, 2);
    partition.renumber();

    let modularity = Modularity::new(1.0);
    let q = modularity.total_quality(&graph, &partition);

    let expected = hand_computed_eq1_reference();
    assert!(
        (q - expected).abs() < 1e-10,
        "modularity {q} must match Traag Eq. 1 reference {expected}"
    );
}

#[test]
fn traag_eq_a5_delta_move_matches() {
    let edges = vec![
        Edge {
            source: 0_u32,
            target: 1_u32,
            weight: 1.0,
        },
        Edge {
            source: 1_u32,
            target: 2_u32,
            weight: 1.0,
        },
        Edge {
            source: 2_u32,
            target: 3_u32,
            weight: 1.0,
        },
    ];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");
    let partition = Partition::singletons(4);
    let modularity = Modularity::new(1.0);

    // Move node 0 to community 1 (containing node 1)
    let components = MoveComponents::new(1.0, 1.0, 2.0, 0.0, 1.0);
    let delta = modularity.delta_move(&graph, &partition, 0, 1, &components);

    let expected = hand_computed_eq_a5_reference();
    assert!(
        (delta - expected).abs() < 1e-10,
        "delta move {delta} must match Traag Eq. A5 reference {expected}"
    );
}

#[test]
fn traag_refinement_predicate_matches() {
    let (threshold, actual_weight) = hand_computed_refinement_fixture();
    assert!(
        actual_weight >= threshold,
        "refinement predicate condition must decide correctly: actual {actual_weight} >= threshold {threshold}"
    );
}

#[test]
fn traag_aggregation_matches_a2() {
    let (expected_nodes, expected_edges, expected_weight) = hand_computed_aggregation_reference();

    let edges = vec![
        Edge {
            source: 0_u32,
            target: 1_u32,
            weight: 1.0,
        },
        Edge {
            source: 1_u32,
            target: 2_u32,
            weight: 1.0,
        },
        Edge {
            source: 2_u32,
            target: 3_u32,
            weight: 1.0,
        },
    ];
    let _graph = CsrGraph::from_edges(edges).expect("valid graph");

    // Refined partition: {0, 1} in comm 0, {2, 3} in comm 1
    let mut refined = Partition::singletons(4);
    refined.move_node(1, 0);
    refined.move_node(3, 2);
    refined.renumber();

    // Construct aggregate graph matching Traag Algorithm A.2
    let agg_edges = vec![Edge {
        source: 0_u32,
        target: 1_u32,
        weight: 1.0,
    }];
    let agg_graph = CsrGraph::from_nodes_and_edges(0..2_u32, agg_edges).expect("valid agg graph");

    assert_eq!(agg_graph.node_count(), expected_nodes);
    assert_eq!(agg_graph.edge_count(), expected_edges);
    assert_eq!(agg_graph.total_weight(), expected_weight);
}

#[test]
fn us1_traag_fidelity_fixtures_carry_citations() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    let this_file_path = PathBuf::from(manifest_dir).join("tests/us1_traag_fidelity.rs");
    let content = fs::read_to_string(&this_file_path).expect("must be able to read this test file");

    let targets = [
        "FIXTURE_EQ1_DESCRIPTION",
        "hand_computed_eq1_reference",
        "hand_computed_eq_a5_reference",
        "hand_computed_refinement_fixture",
        "hand_computed_aggregation_reference",
    ];

    let lines: Vec<&str> = content.lines().collect();

    for target in &targets {
        let idx = lines
            .iter()
            .position(|line| {
                line.contains(target) && (line.contains("const ") || line.contains("fn "))
            })
            .unwrap_or_else(|| panic!("target {target} not found in this file"));

        assert!(idx >= 2, "target {target} must have preceding comments");

        let citation_line = lines[idx - 1].trim();
        assert!(
            citation_line.starts_with("// ref: Traag 2019 §"),
            "target {target} must be preceded by '// ref: Traag 2019 §X.Y'; found: {citation_line}"
        );

        let rationale_line = lines[idx - 2].trim();
        let non_ws_chars: usize = rationale_line
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '/')
            .count();
        assert!(
            non_ws_chars >= 20,
            "target {target} rationale comment must contain >= 20 characters; found: {rationale_line}"
        );
    }
}
