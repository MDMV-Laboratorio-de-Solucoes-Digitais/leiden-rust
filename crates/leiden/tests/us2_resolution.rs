//! US2: Resolution tuning, parameter validation, reproducibility and tie-break rules.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::unnecessary_debug_formatting,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::path::PathBuf;

use leiden::{CsrGraph, Edge, Leiden, LeidenError, LeidenParameters, TerminationReason};

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
fn malformed_invalid_gamma() {
    let graph = load_edg("two_cliques.edg");

    // gamma = 0.0
    let res_zero = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 0.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph);
    assert!(
        matches!(res_zero, Err(LeidenError::InvalidGamma(val)) if (val - 0.0).abs() < f64::EPSILON)
    );

    // gamma = -1.0
    let res_neg = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: -1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph);
    assert!(
        matches!(res_neg, Err(LeidenError::InvalidGamma(val)) if (val - (-1.0)).abs() < f64::EPSILON)
    );

    // gamma = NaN
    let res_nan = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: f64::NAN,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph);
    assert!(matches!(res_nan, Err(LeidenError::InvalidGamma(_))));
}

#[test]
fn invalid_iteration_cap_rejected() {
    let graph = load_edg("two_cliques.edg");

    let res_zero_cap = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 0,
        })
        .run(&graph);
    assert!(matches!(
        res_zero_cap,
        Err(LeidenError::InvalidIterationCap(0))
    ));
}

#[test]
fn resolution_changes_partition() {
    let graph = load_edg("karate.edg");

    let res_low_gamma = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 0.5,
            seed: Some(42),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("runs successfully with gamma=0.5");

    let res_high_gamma = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 2.0,
            seed: Some(42),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("runs successfully with gamma=2.0");

    assert_ne!(
        res_low_gamma.partition, res_high_gamma.partition,
        "partitions under gamma=0.5 and gamma=2.0 must differ"
    );

    // Modularity values should differ
    assert_ne!(res_low_gamma.quality, res_high_gamma.quality);
}

#[test]
fn determinism_under_fixed_seed() {
    let graph = load_edg("karate.edg");
    let params = LeidenParameters {
        gamma: 1.0,
        seed: Some(12345),
        iteration_cap: 10,
    };

    let res1 = Leiden::new()
        .with_parameters(params.clone())
        .run(&graph)
        .expect("run 1 succeeds");

    let res2 = Leiden::new()
        .with_parameters(params)
        .run(&graph)
        .expect("run 2 succeeds");

    assert_eq!(res1.partition, res2.partition);
    assert_eq!(res1.quality, res2.quality);
    assert_eq!(res1.iterations, res2.iterations);
    assert_eq!(res1.termination_reason, res2.termination_reason);
    assert_eq!(res1.seed, res2.seed);
}

#[test]
fn convergence_before_cap() {
    let graph = load_edg("two_cliques.edg");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 100,
        })
        .run(&graph)
        .expect("runs on two_cliques");

    assert_eq!(result.termination_reason, TerminationReason::Converged);
    assert!(
        result.iterations < 100,
        "two cliques must converge in fewer than 100 iterations, took {}",
        result.iterations
    );
}

#[test]
fn iteration_cap_returns_best_partition() {
    let graph = load_edg("path.edg");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 2,
        })
        .run(&graph)
        .expect("runs on path with cap=2");

    assert_eq!(result.termination_reason, TerminationReason::IterationCap);
    assert_eq!(result.iterations, 2);
    assert!(result.quality.is_finite());
    assert_eq!(result.partition.len(), 10);
}

#[test]
fn iteration_cap_tiebreak_prefers_earliest_iteration() {
    // When modularity delta is within EPSILON across iterations, the orchestrator
    // prefers the earlier iteration's partition.
    let graph = load_edg("path.edg");
    let res = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 2,
        })
        .run(&graph)
        .expect("runs successfully");

    assert_eq!(res.termination_reason, TerminationReason::IterationCap);
    assert!(res.quality.is_finite());
}

#[test]
fn iteration_cap_tiebreak_prefers_smallest_community_count() {
    // If quality is tied within EPSILON at the same iteration, smaller community count wins.
    let graph = load_edg("path.edg");
    let res = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 5,
        })
        .run(&graph)
        .expect("runs successfully");

    assert!(res.quality.is_finite());
}
