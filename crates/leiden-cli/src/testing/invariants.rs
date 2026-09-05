//! Shared assertion helpers for property-based tests in leiden-cli.

/// Floating-point comparison epsilon.
pub const EPSILON: f64 = 1e-9;

/// Assert two f64 values are equal within `EPSILON`.
///
/// # Panics
/// Panics if `|a - b| >= EPSILON`.
pub fn assert_eps_eq(a: f64, b: f64) {
    assert!(
        (a - b).abs() < EPSILON,
        "assertion failed: |{a} - {b}| < {EPSILON}"
    );
}
