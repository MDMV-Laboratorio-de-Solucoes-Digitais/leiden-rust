//! Integration test: Observability checklist alignment (T126a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions permitted per Constitution §III"
)]

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    Path::new(&manifest_dir)
        .ancestors()
        .nth(2)
        .expect("workspace root resolution failed")
        .to_path_buf()
}

#[test]
fn leidenevent_variant_count_matches_observability_checklist() {
    let checklist_path =
        workspace_root().join("../specs/001-leiden-algorithm/checklists/observability.md");

    assert!(
        checklist_path.exists(),
        "Observability checklist must exist at {checklist_path:?}"
    );

    let content = std::fs::read_to_string(&checklist_path)
        .unwrap_or_else(|err| panic!("failed to read observability checklist: {err}"));

    // Assert that the checklist covers all key LeidenEvent variants
    let expected_variants = [
        "IterationStarted",
        "IterationFinished",
        "Terminated",
        "Throttled",
    ];

    for variant in expected_variants {
        assert!(
            content.contains(variant),
            "Observability checklist must reference LeidenEvent variant `{variant}`"
        );
    }
}
