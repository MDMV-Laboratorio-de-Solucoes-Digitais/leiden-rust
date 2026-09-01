//! US3: Library API integration tests, builder patterns, and type polymorphism.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::unnecessary_debug_formatting,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::num::NonZeroU32;

use leiden::{CsrGraph, Edge, Leiden, LeidenEvent, LeidenParameters};

#[test]
fn library_api_smoke() {
    let edges = vec![
        Edge {
            source: "n1".to_string(),
            target: "n2".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "n2".to_string(),
            target: "n3".to_string(),
            weight: 1.0,
        },
    ];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");
    let params = LeidenParameters::default();
    let result = Leiden::new()
        .with_parameters(params.clone())
        .run(&graph)
        .expect("smoke test run succeeds");

    assert!(result.quality.is_finite());
    assert!(result.iterations <= 10);
    assert_eq!(result.seed, params.seed);
}

#[test]
fn library_accepts_ord_node_id_types() {
    // 1. u32 node IDs
    let edges_u32 = vec![Edge {
        source: 10_u32,
        target: 20_u32,
        weight: 1.0,
    }];
    let graph_u32 = CsrGraph::from_edges(edges_u32).expect("valid u32 graph");
    let res_u32 = Leiden::new().run(&graph_u32).expect("runs on u32 graph");
    assert_eq!(res_u32.partition.len(), 2);
    assert_eq!(res_u32.partition[0].0, 10);
    assert_eq!(res_u32.partition[1].0, 20);

    // 2. String node IDs
    let edges_str = vec![Edge {
        source: "alice".to_string(),
        target: "bob".to_string(),
        weight: 2.0,
    }];
    let graph_str = CsrGraph::from_edges(edges_str).expect("valid string graph");
    let res_str = Leiden::new().run(&graph_str).expect("runs on string graph");
    assert_eq!(res_str.partition.len(), 2);
    assert_eq!(res_str.partition[0].0, "alice");
    assert_eq!(res_str.partition[1].0, "bob");
}

#[test]
fn leiden_public_api_has_no_mut_methods() {
    // Assert builder methods take by value or immutable reference, not &mut self
    let orchestrator = Leiden::new();
    let _with_params = orchestrator
        .clone()
        .with_parameters(LeidenParameters::default());
    let _with_threads = orchestrator.with_threads(NonZeroU32::new(1).expect("non-zero"));
}

#[test]
fn fr012_with_threads_builder_compiles() {
    const _: () = {
        let _: fn(Leiden, NonZeroU32) -> Leiden = Leiden::with_threads;
    };

    let edges = vec![Edge {
        source: "0".to_string(),
        target: "1".to_string(),
        weight: 1.0,
    }];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");

    let result = Leiden::default()
        .with_threads(NonZeroU32::new(1).expect("valid non-zero"))
        .run(&graph);

    assert!(result.is_ok());
}

#[test]
fn with_event_sink_builder_compiles() {
    const _: () = {
        let _: fn(Leiden, std::sync::mpsc::Sender<LeidenEvent>) -> Leiden = Leiden::with_event_sink;
    };

    let edges = vec![Edge {
        source: "0".to_string(),
        target: "1".to_string(),
        weight: 1.0,
    }];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");
    let (tx, _rx) = std::sync::mpsc::channel();

    let result = Leiden::default().with_event_sink(tx).run(&graph);

    assert!(result.is_ok());
}
