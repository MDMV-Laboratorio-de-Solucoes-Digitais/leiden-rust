//! Proptest configuration constants and helpers.
//!
//! Provides shared configuration for all property-based tests in the leiden crate.
#![cfg(test)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::uninlined_format_args,
    clippy::manual_range_contains,
    clippy::format_push_string,
    clippy::option_if_let_else,
    clippy::unreachable,
    clippy::redundant_pub_crate,
    unused_imports,
    clippy::doc_markdown,
    dead_code,
    unused_doc_comments,
    deprecated,
    reason = "test code"
)]

/// Default timeout per test case in milliseconds (FR-014).
pub(crate) const DEFAULT_TIMEOUT_MS: u32 = 30_000;

/// Base (maximum) test cases for local development (FR-014).
pub(crate) const BASE_CASES: u32 = 1000;

/// Minimum test cases in local development (floor for adaptive formula).
pub(crate) const LOCAL_MIN_CASES: u32 = 100;

/// Test cases in CI environment (FR-014).
pub(crate) const CI_TEST_CASES: u32 = 256;

/// Maximum shrink iterations (FR-006).
pub(crate) const MAX_SHRINK_ITERS: u32 = 200;

/// Node count threshold for adaptive case reduction (FR-014).
pub(crate) const ADAPTIVE_THRESHOLD_NODES: usize = 20;

/// Floating-point comparison epsilon (FR-009).
pub(crate) const MODULARITY_EPSILON: f64 = 1e-9;

/// Minimum nodes in generated graphs.
pub(crate) const MIN_NODES: usize = 5;

/// Maximum nodes in generated graphs.
pub(crate) const MAX_NODES: usize = 100;

/// Minimum edge weight (exclusive zero).
pub(crate) const MIN_WEIGHT: f64 = 1e-6;

/// Maximum edge weight.
pub(crate) const MAX_WEIGHT: f64 = 100.0;

/// Returns the adaptive case count based on graph size.
///
/// Scales down cases for larger graphs to avoid exceeding the 30-second timeout.
/// Formula: `max(min_cases, BASE_CASES - 10 * nodes.saturating_sub(ADAPTIVE_THRESHOLD_NODES))`
///
/// # Contract
/// - For graphs with ≤20 nodes: returns `BASE_CASES` or `CI_TEST_CASES`
/// - For graphs with 20+ nodes: returns `max(min_cases, BASE_CASES - 10 * (nodes - 20))`
/// - `saturating_sub` prevents underflow for small graphs
///
/// # Arguments
/// * `nodes` - Number of nodes in the graph
/// * `is_ci` - Whether running in CI environment (uses lower floor)
pub(crate) fn adaptive_case_count(nodes: usize, is_ci: bool) -> u32 {
    let min_cases = if is_ci {
        CI_TEST_CASES
    } else {
        LOCAL_MIN_CASES
    };
    let adaptive =
        BASE_CASES.saturating_sub(nodes.saturating_sub(ADAPTIVE_THRESHOLD_NODES) as u32 * 10);
    std::cmp::max(min_cases, adaptive)
}

/// Returns the shared proptest configuration for property tests.
///
/// # Contract
/// - Timeout: 30 seconds per test case (FR-014)
/// - Cases: adaptive based on graph size (FR-014)
/// - Failure persistence: SourceParallel to "proptest-regressions" (FR-012)
/// - Max shrink iterations: 200 (FR-006)
///
/// # Arguments
/// * `graph_nodes` - Optional node count for adaptive case calculation
/// * `is_ci` - Whether running in CI environment
pub(crate) fn proptest_config(
    graph_nodes: Option<usize>,
    is_ci: bool,
) -> proptest::test_runner::Config {
    let cases = match graph_nodes {
        Some(nodes) => adaptive_case_count(nodes, is_ci),
        None => {
            if is_ci {
                CI_TEST_CASES
            } else {
                BASE_CASES
            }
        }
    };
    proptest::test_runner::Config {
        timeout: DEFAULT_TIMEOUT_MS,
        cases,
        failure_persistence: Some(Box::new(
            proptest::test_runner::FileFailurePersistence::SourceParallel("proptest-regressions"),
        )),
        max_shrink_iters: MAX_SHRINK_ITERS,
        ..Default::default()
    }
}

/// Detects if running in CI environment.
pub(crate) fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}
