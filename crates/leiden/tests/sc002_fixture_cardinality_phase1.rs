//! Phase 1 failing-test that asserts the SC-002 curated-suite cardinality
//! invariant is satisfied before any user-story work begins.
//!
//! Per `spec.md` SC-002 and `tasks.md` T007c / T116b: the curated reference
//! suite MUST contain at least 10 graphs (with reference partitions) so that
//! `fixture_suite_matches_reference` (T035 / T116a) exercises a
//! representative spread of topologies.
//!
//! This is the **write-time** counterpart to the Phase 7 cardinality
//! assertion in `tests/sc002_fixture_cardinality.rs` (T116b). Both tests share
//! the same body; the `[P] [T]` markers and distinct `tests/` locations
//! disambiguate the two enforcement points (Phase 1 vs. Phase 7 pre-merge).
//!
//! The test walks `leiden/fixtures/` (resolved at runtime via
//! `CARGO_MANIFEST_DIR`), excludes the metadata file `_classification.json`
//! and the per-fixture `*.partition.json` companion files, and counts the
//! remaining `*.expected.json` files. The assertion `file_count >= 10` is
//! the SC-002 cardinality contract.
//!
//! Format-level validation (every `*.expected.json` parses, has the required
//! top-level fields) lives in Phase 7 once the parser exists.

// Per Constitution §III, test code MAY use `unwrap`/`expect`/`panic` for
// assertion diagnostics; the production-code lint bans do not apply here.
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::manual_let_else,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    reason = "test code: SC-002 cardinality invariant assertion; production-code \
              panic-free ban does not apply per Constitution §III"
)]

use std::fs;
use std::path::{Path, PathBuf};

const MIN_FIXTURE_COUNT: usize = 10;

/// Returns the absolute path to the workspace `fixtures/` directory, resolved
/// from this crate's manifest directory (i.e. `leiden/crates/leiden`).
#[must_use]
fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    Path::new(&manifest_dir)
        .ancestors()
        .nth(2) // crates/leiden -> crates -> leiden (workspace root)
        .expect("workspace root resolution failed")
        .join("fixtures")
}

/// Counts `*.expected.json` files under `dir` (non-recursive; the fixtures
/// suite is a flat directory). Excludes the metadata file
/// `_classification.json`.
#[must_use]
fn count_expected_json(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => panic!("failed to read fixtures directory {:?}: {e}", dir),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('_') {
            // Reserved metadata files (e.g. `_classification.json`).
            continue;
        }
        if name.ends_with(".expected.json") {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[test]
fn sc002_curated_suite_has_at_least_10_fixtures_at_phase_1() {
    let dir = fixtures_dir();
    assert!(
        dir.exists(),
        "fixtures directory not found at {dir:?}; expected it to be a sibling of crates/",
    );

    let expected = count_expected_json(&dir);
    let count = expected.len();
    let names: Vec<&str> = expected
        .iter()
        .filter_map(|p| p.file_name())
        .filter_map(|n| n.to_str())
        .collect();

    assert!(
        count >= MIN_FIXTURE_COUNT,
        "SC-002 cardinality invariant violated: expected >= {want} `*.expected.json` fixtures under {dir:?}, found {got}. Add more fixtures (T007-T007c) before user-story work proceeds. Files present: {names:?}",
        want = MIN_FIXTURE_COUNT,
        got = count,
    );
}
