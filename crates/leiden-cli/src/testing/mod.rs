/// Test-only utilities for property-based testing in leiden-cli.
///
/// This module is `#[cfg(test)]` — zero production code impact.
/// Mirrors the structure of `leiden::testing` for CLI-specific tests.

pub mod config;
pub mod invariants;
