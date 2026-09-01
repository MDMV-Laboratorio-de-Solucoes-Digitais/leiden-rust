//! SC-001 performance and determinism integration tests (T121a, T121b).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    dead_code,
    unused_imports,
    reason = "test code: assertions permitted per Constitution §III"
)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};

fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    Path::new(&manifest_dir)
        .ancestors()
        .nth(2)
        .expect("workspace root resolution failed")
        .join("fixtures")
}

fn load_fixture_graph(name: &str) -> CsrGraph<String> {
    let path = fixtures_dir().join(name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));

    let mut edges = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let src = parts[0].to_string();
            let dst = parts[1].to_string();
            let weight = if parts.len() >= 3 {
                parts[2].parse::<f64>().unwrap_or(1.0)
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
    CsrGraph::from_edges(edges).expect("valid graph")
}

#[test]
#[cfg(not(debug_assertions))]
fn sc001_under_5s_on_lfr_small_fixture() {
    let graph = load_fixture_graph("lfr_small.edg");
    let params = LeidenParameters {
        gamma: 1.0,
        seed: Some(0),
        iteration_cap: 10,
    };

    let start = Instant::now();
    let result = Leiden::new()
        .with_parameters(params)
        .run(&graph)
        .expect("runs on lfr_small fixture");
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "SC-001 budget violated: took {:?}, expected < 5s",
        elapsed
    );
    assert!(result.quality.is_finite());
}

#[test]
#[cfg(not(debug_assertions))]
fn sc001_byte_identical_on_lfr_small_fixture() {
    let graph = load_fixture_graph("lfr_small.edg");
    let params = LeidenParameters {
        gamma: 1.0,
        seed: Some(0),
        iteration_cap: 10,
    };

    let res1 = Leiden::new()
        .with_parameters(params.clone())
        .run(&graph)
        .expect("first run on lfr_small");

    let res2 = Leiden::new()
        .with_parameters(params)
        .run(&graph)
        .expect("second run on lfr_small");

    assert_eq!(res1.partition, res2.partition);
    assert_eq!(res1.iterations, res2.iterations);
    assert_eq!(res1.termination_reason, res2.termination_reason);
    assert!((res1.quality - res2.quality).abs() < 1e-12);
}
