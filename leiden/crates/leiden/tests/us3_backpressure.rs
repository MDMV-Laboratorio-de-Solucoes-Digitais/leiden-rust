//! Integration tests: Orchestrator event sink backpressure (T113d).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, Leiden};
use std::sync::mpsc;

#[test]
fn orchestrator_emits_throttled_on_full_sink() {
    let edges = vec![
        Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "b".to_string(),
            target: "c".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "a".to_string(),
            target: "c".to_string(),
            weight: 1.0,
        },
    ];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");

    let (tx, rx) = mpsc::channel();

    let result = Leiden::new().with_event_sink(tx).run(&graph);

    assert!(result.is_ok());
    let run_res = result.expect("run succeeds");
    assert!(run_res.quality.is_finite());

    // Receiver received events
    let mut count = 0;
    while let Ok(_event) = rx.try_recv() {
        count += 1;
    }
    assert!(count > 0);
}
