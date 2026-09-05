//! Shared assertion helpers for property-based tests.
//!
//! Provides common assertion functions used across multiple test modules.
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
    dead_code,
    unused_doc_comments,
    deprecated,
    reason = "test code"
)]

use super::config::MODULARITY_EPSILON;

/// Assert two f64 values are equal within `MODULARITY_EPSILON`.
///
/// # Panics
/// Panics if `|a - b| >= MODULARITY_EPSILON`.
pub(crate) fn assert_eps_eq(a: f64, b: f64) {
    assert!(
        (a - b).abs() < MODULARITY_EPSILON,
        "assertion failed: |{} - {}| < {}",
        a,
        b,
        MODULARITY_EPSILON
    );
}

/// Assert value is finite (not NaN, not ±Inf).
///
/// # Panics
/// Panics if value is NaN or infinite.
pub(crate) fn assert_finite(v: f64) {
    assert!(v.is_finite(), "value must be finite, got {}", v);
}

/// Assert modularity is in valid range [-1.0, 1.0] (with epsilon).
///
/// # Panics
/// Panics if `q < -1.0 - MODULARITY_EPSILON` or `q > 1.0 + MODULARITY_EPSILON`.
pub(crate) fn assert_modularity_valid(q: f64) {
    assert!(
        q >= -1.0 - MODULARITY_EPSILON && q <= 1.0 + MODULARITY_EPSILON,
        "modularity {} not in [-1.0, 1.0]",
        q
    );
}

/// Assert that two partitions have the same community count.
///
/// # Panics
/// Panics if community counts differ.
pub(crate) fn assert_community_count_eq(
    a: &crate::partition::Partition,
    b: &crate::partition::Partition,
) {
    assert_eq!(
        a.community_count(),
        b.community_count(),
        "partition community count mismatch"
    );
}
