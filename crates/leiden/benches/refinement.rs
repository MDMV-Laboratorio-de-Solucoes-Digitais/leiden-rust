//! Criterion benchmark: Refinement phase on `lfr_small.edg` (T122).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    unused_results,
    reason = "bench code: assertions and diagnostics permitted per Constitution §III"
)]

use criterion::{Criterion, criterion_group, criterion_main};
use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};
use std::hint::black_box;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    Path::new(&manifest_dir)
        .ancestors()
        .nth(2)
        .unwrap_or_else(|| Path::new("."))
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

fn bench_refinement(c: &mut Criterion) {
    let graph = load_fixture_graph("lfr_small.edg");
    let params = LeidenParameters {
        gamma: 1.0,
        seed: Some(42),
        iteration_cap: 2,
    };

    let _ = c.bench_function("refinement_lfr_small", |b| {
        b.iter(|| {
            let res = Leiden::new()
                .with_parameters(params.clone())
                .run(black_box(&graph));
            let _ = black_box(res);
        });
    });
}

criterion_group!(benches, bench_refinement);
criterion_main!(benches);
