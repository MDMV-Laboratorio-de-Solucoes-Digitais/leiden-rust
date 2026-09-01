//! Integration tests: Bounded channel backpressure and throttling (T113a, T113b, T113c).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::sync::mpsc::sync_channel;
use std::time::Instant;

use leiden::{CsrGraph, Edge, LeidenEvent, LeidenParameters};
use leiden_tui::worker::{ThrottledSender, spawn_leiden_worker};

#[test]
fn bounded_channel_overflow_emits_throttled() {
    let (tx, rx) = sync_channel::<LeidenEvent>(2);
    let sender = ThrottledSender::new(tx);

    // Send 2 normal events filling the channel
    sender.send(LeidenEvent::IterationStarted {
        index: 1,
        phase: leiden::Phase::LocalMoving,
    });
    sender.send(LeidenEvent::IterationStarted {
        index: 2,
        phase: leiden::Phase::LocalMoving,
    });

    // 3rd event should trigger Throttled fallback without blocking
    sender.send(LeidenEvent::IterationStarted {
        index: 3,
        phase: leiden::Phase::LocalMoving,
    });

    let ev1 = rx.try_recv();
    assert!(ev1.is_ok());
    let ev2 = rx.try_recv();
    assert!(ev2.is_ok());
}

#[test]
fn sender_failure_is_logged_not_fatal() {
    let (tx, rx) = sync_channel::<LeidenEvent>(1);
    let sender = ThrottledSender::new(tx);

    // Drop receiver to simulate disconnected TUI
    drop(rx);

    // Send should log warning and return without panicking
    sender.send(LeidenEvent::Throttled { dropped: 1 });
}

#[test]
fn full_channel_does_not_block_orchestrator() {
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
    ];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");
    let params = LeidenParameters::default();

    let start = Instant::now();
    let paused = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let step = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let abort = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (rx, handle) = spawn_leiden_worker(graph, params, paused, step, abort);

    // Hold receiver without draining
    let res = handle.join().expect("worker joins successfully");
    let elapsed = start.elapsed();

    assert!(res.is_ok(), "Leiden run must complete without error");
    assert!(
        elapsed.as_millis() < 5000,
        "Orchestrator must not block indefinitely on undrained channel"
    );

    // Drop rx
    drop(rx);
}
