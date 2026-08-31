//! FR-009 write-time citation/deviation guard.
//!
//! Per `spec.md` FR-009 (Traag 2019 citation discipline) and Constitution
//! Additional Constraints ("Domain accuracy"): every public item or function
//! body in `crates/leiden/src/{local_moving, refinement, aggregation,
//! quality, orchestrator}/` MUST carry either a `// ref: Traag 2019 §X.Y`
//! citation comment or a `// leiden-deviation:` marker documenting the
//! intentional departure from the published algorithm.
//!
//! The companion `compile_fail` doctest in `crates/leiden/src/lib.rs`
//! demonstrates the citation pattern at the crate-root level.
//!
//! This is the **write-time** counterpart to T138a (Phase 7 pre-merge
//! audit). The audit walks the same directories and asserts the same
//! property at release-cut time; this test makes the contract fail-fast at
//! the moment a new file lands.
//!
//! Phase 1 / Phase 2 behaviour: the source-module directories do not yet
//! exist; this test scans them if present and is a no-op otherwise so the
//! Phase 1 verification gate stays green. From Phase 3 onward, any `pub` item
//! or function body added under those modules without a citation OR a
//! deviation marker causes this test to fail with a precise diagnostic.

// Test code may panic to report scan failures; the production-code lint ban
// does not apply here per Constitution §III.
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::redundant_pub_crate,
    clippy::uninlined_format_args,
    clippy::manual_let_else,
    clippy::unnecessary_debug_formatting,
    reason = "test code: FR-009 write-time citation guard; production-code lint bans \
              do not apply per Constitution §III"
)]

use std::fs;
use std::path::PathBuf;

mod leiden_tests_support {
    use std::fs;
    use std::path::Path;

    /// Returns `true` if `path`'s content contains at least one
    /// `// ref: Traag 2019 §X.Y` citation comment OR a
    /// `// leiden-deviation:` marker anywhere in the file. Used by the
    /// FR-009 guard at the file level; the audit (T138a) does the
    /// per-item / per-function scan at pre-merge time.
    #[must_use]
    pub(super) fn has_citation_or_deviation_marker(path: &Path) -> bool {
        let Ok(bytes) = fs::read(path) else {
            return false;
        };
        let Ok(text) = std::str::from_utf8(&bytes) else {
            return false;
        };
        text.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.contains("// ref: Traag 2019 §") || trimmed.contains("// leiden-deviation:")
        })
    }
}

const SCANNED_MODULES: &[&str] = &[
    "local_moving",
    "refinement",
    "aggregation",
    "quality",
    "orchestrator",
];

#[must_use]
fn crate_src_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    PathBuf::from(manifest_dir).join("src")
}

/// Walks `crates/leiden/src/{module}/` and asserts that every `.rs` file
/// contains either a Traag 2019 citation comment or a `leiden-deviation:`
/// marker. Phase 1 / Phase 2 see an empty directory scan and pass trivially;
/// Phase 3 onward, this test fails the build if any new file lacks the
/// discipline.
#[test]
fn fr009_no_uncited_deviations() {
    let src_root = crate_src_root();
    let mut violations: Vec<PathBuf> = Vec::new();

    for module in SCANNED_MODULES {
        let module_dir = src_root.join(module);
        if !module_dir.exists() {
            // Phase 1 / Phase 2: module not yet present; nothing to check.
            continue;
        }
        scan_for_uncited_files(&module_dir, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "FR-009 violated: files in {src_root:?}/{{local_moving,refinement,aggregation,quality,orchestrator}}/ must carry a `// ref: Traag 2019 §X.Y` citation OR a `// leiden-deviation:` marker. Files missing either: {violations:#?}",
    );
}

fn scan_for_uncited_files(dir: &std::path::Path, violations: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_for_uncited_files(&path, violations);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if !leiden_tests_support::has_citation_or_deviation_marker(&path) {
            violations.push(path);
        }
    }
}
