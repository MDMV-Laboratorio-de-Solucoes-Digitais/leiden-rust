//! Proptest configuration constants and helpers for leiden-cli tests.

/// Base (maximum) test cases for local development.
pub const BASE_CASES: u32 = 1000;

/// Minimum test cases in local development (floor for adaptive formula).
pub const LOCAL_MIN_CASES: u32 = 100;

/// Test cases in CI environment.
pub const CI_TEST_CASES: u32 = 256;

/// Node count threshold for adaptive case reduction.
pub const ADAPTIVE_THRESHOLD_NODES: usize = 20;

/// Returns the adaptive case count based on graph size.
///
/// Scales down cases for larger graphs to avoid exceeding the 30-second timeout.
///
/// # Arguments
/// * `nodes` - Number of nodes in the graph
/// * `is_ci` - Whether running in CI environment (uses lower floor)
#[must_use]
pub fn adaptive_case_count(nodes: usize, is_ci: bool) -> u32 {
    let min_cases = if is_ci {
        CI_TEST_CASES
    } else {
        LOCAL_MIN_CASES
    };
    let node_diff = nodes.saturating_sub(ADAPTIVE_THRESHOLD_NODES);
    let adaptive = BASE_CASES.saturating_sub(u32::try_from(node_diff).unwrap_or(u32::MAX) * 10);
    std::cmp::max(min_cases, adaptive)
}

/// Returns the shared proptest configuration for property tests.
///
/// NOTE: Named `proptest_cfg` to avoid conflict with the `#[proptest_config]`
/// attribute macro from the proptest crate.
///
/// # Arguments
/// * `graph_nodes` - Optional node count for adaptive case calculation
/// * `is_ci` - Whether running in CI environment
#[must_use]
pub fn proptest_cfg(graph_nodes: Option<usize>, is_ci: bool) -> proptest::test_runner::Config {
    let cases = graph_nodes.map_or(if is_ci { CI_TEST_CASES } else { BASE_CASES }, |nodes| {
        adaptive_case_count(nodes, is_ci)
    });
    proptest::test_runner::Config {
        cases,
        ..Default::default()
    }
}
