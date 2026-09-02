//! Observability integration test: verifying structured tracing events (T126).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};
use std::sync::{Arc, Mutex};
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

#[derive(Default, Clone)]
struct EventCapture {
    events: Arc<Mutex<Vec<String>>>,
}

impl<S> Layer<S> for EventCapture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let name = event.metadata().name().to_string();
        if let Ok(mut lock) = self.events.lock() {
            lock.push(name);
        }
    }
}

#[test]
fn tracing_events_are_emitted_during_run() {
    let capture = EventCapture::default();
    let subscriber = tracing_subscriber::registry().with(capture.clone());

    let edges = vec![
        Edge {
            source: 1,
            target: 2,
            weight: 1.0,
        },
        Edge {
            source: 2,
            target: 3,
            weight: 1.0,
        },
    ];
    let graph = CsrGraph::from_edges(edges).expect("valid graph");

    let result = tracing::subscriber::with_default(subscriber, || {
        Leiden::new()
            .with_parameters(LeidenParameters {
                gamma: 1.0,
                seed: Some(0),
                iteration_cap: 5,
            })
            .run(&graph)
    });

    assert!(result.is_ok());

    let logged_events = capture.events.lock().expect("lock capture").clone();
    assert!(
        !logged_events.is_empty(),
        "Orchestrator must emit structured tracing events during execution"
    );
}
