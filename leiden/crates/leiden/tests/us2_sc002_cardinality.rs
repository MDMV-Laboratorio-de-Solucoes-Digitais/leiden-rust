//! Phase 7: SC-002 curated-suite cardinality assertion (T116b).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::manual_let_else,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    reason = "test code: SC-002 cardinality invariant assertion"
)]

use std::fs;
use std::path::{Path, PathBuf};

const MIN_FIXTURE_COUNT: usize = 10;

#[must_use]
fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    Path::new(&manifest_dir)
        .ancestors()
        .nth(2)
        .expect("workspace root resolution failed")
        .join("fixtures")
}

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
fn sc002_curated_suite_has_at_least_10_fixtures() {
    let dir = fixtures_dir();
    assert!(
        dir.exists(),
        "fixtures directory not found at {dir:?}; expected it to be a sibling of crates/",
    );

    let expected = count_expected_json(&dir);
    let count = expected.len();

    assert!(
        count >= MIN_FIXTURE_COUNT,
        "SC-002 cardinality invariant violated: expected >= {want} `*.expected.json` fixtures, found {got}",
        want = MIN_FIXTURE_COUNT,
        got = count,
    );
}
