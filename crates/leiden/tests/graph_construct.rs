//! Unit and integration tests for `CsrGraph` construction.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::doc_markdown,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, LeidenError};

#[test]
fn graph_construct_two_cliques_offsets_and_counts() {
    // 4-clique (nodes 0..4) and 5-clique (nodes 4..9) joined by bridge (3, 4).
    let mut edges = Vec::new();
    for i in 0..4_u32 {
        for j in (i + 1)..4_u32 {
            edges.push(Edge {
                source: i,
                target: j,
                weight: 1.0,
            });
        }
    }
    for i in 4..9_u32 {
        for j in (i + 1)..9_u32 {
            edges.push(Edge {
                source: i,
                target: j,
                weight: 1.0,
            });
        }
    }
    edges.push(Edge {
        source: 3,
        target: 4,
        weight: 0.5,
    });

    let graph = CsrGraph::from_edges(edges).expect("valid graph edges must construct successfully");

    assert_eq!(graph.node_count(), 9);
    assert_eq!(graph.edge_count(), 17);
    assert_eq!(graph.total_weight(), 16.5);

    // Degree of node 0 (part of K4) is 3.0
    assert_eq!(graph.degree_of(0), 3.0);
    assert_eq!(graph.neighbours_of(0).len(), 3);
    assert_eq!(graph.weights_of(0).len(), 3);

    // Degree of node 3 (part of K4 + bridge 0.5) is 3.5
    assert_eq!(graph.degree_of(3), 3.5);

    // Degree of node 4 (part of K5 + bridge 0.5) is 4.5
    assert_eq!(graph.degree_of(4), 4.5);
}

#[test]
fn graph_construct_rejects_negative_weight() {
    let edges = vec![
        Edge {
            source: 0_u32,
            target: 1_u32,
            weight: 1.0,
        },
        Edge {
            source: 1_u32,
            target: 2_u32,
            weight: -0.5,
        },
    ];

    let result = CsrGraph::from_edges(edges);
    match result {
        Err(LeidenError::InvalidWeight { line: _, value }) => {
            assert_eq!(value, -0.5);
        }
        other => panic!("expected LeidenError::InvalidWeight, got {other:?}"),
    }
}

#[test]
fn graph_construct_rejects_self_loop() {
    let edges = vec![
        Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "c".to_string(),
            target: "c".to_string(),
            weight: 1.0,
        },
    ];

    let result = CsrGraph::from_edges(edges);
    match result {
        Err(LeidenError::SelfLoop { line, node }) => {
            assert_eq!(line, None);
            assert_eq!(node, "c");
        }
        other => panic!("expected LeidenError::SelfLoop, got {other:?}"),
    }
}

#[test]
fn selfloop_error_payload_matches_input_node() {
    let edges = vec![
        Edge {
            source: "node_alpha".to_string(),
            target: "node_beta".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "node_gamma".to_string(),
            target: "node_gamma".to_string(),
            weight: 2.5,
        },
    ];

    let result = CsrGraph::from_edges(edges);
    match result {
        Err(LeidenError::SelfLoop { line, node }) => {
            assert_eq!(line, None);
            assert_eq!(node, "node_gamma");
        }
        other => panic!("expected LeidenError::SelfLoop with node_gamma, got {other:?}"),
    }
}

#[test]
fn graph_construct_rejects_empty_input() {
    let edges: Vec<Edge<String>> = Vec::new();
    let result = CsrGraph::from_edges(edges);
    match result {
        Err(LeidenError::EmptyGraph) => {}
        other => panic!("expected LeidenError::EmptyGraph, got {other:?}"),
    }
}

#[test]
fn parallel_edges_summarised_into_single_csr_entry() {
    let edges = vec![
        Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            weight: 0.3,
        },
        Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            weight: 0.7,
        },
    ];

    let graph = CsrGraph::from_edges(edges).expect("parallel edges should be summarised");
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.total_weight(), 1.0);

    // Node 0 ('a') has 1 neighbour ('b', internal index 1) with weight 1.0
    assert_eq!(graph.neighbours_of(0), &[1]);
    assert_eq!(graph.weights_of(0), &[1.0]);
    assert_eq!(graph.degree_of(0), 1.0);

    // Node 1 ('b') has 1 neighbour ('a', internal index 0) with weight 1.0
    assert_eq!(graph.neighbours_of(1), &[0]);
    assert_eq!(graph.weights_of(1), &[1.0]);
    assert_eq!(graph.degree_of(1), 1.0);
}
