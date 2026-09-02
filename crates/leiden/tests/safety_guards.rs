//! Safety guards and compile-time invariants (T121c, T129a).

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
fn parallel_feature_is_off_by_default() {
    const {
        assert!(
            !cfg!(feature = "parallel"),
            "The `parallel` Cargo feature MUST be off by default per SC-001"
        );
    }
}

#[test]
fn rayon_not_in_dependency_tree() {
    let lock_path = workspace_root().join("Cargo.lock");
    assert!(
        lock_path.exists(),
        "Cargo.lock must exist at workspace root {lock_path:?}"
    );

    let lock_content = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|err| panic!("failed to read Cargo.lock: {err}"));

    for line in lock_content.lines() {
        let trimmed = line.trim();
        assert!(
            !(trimmed.starts_with("name = \"rayon\"")
                || trimmed.starts_with("name = \"rayon-core\"")),
            "FR-012 violation: `rayon` found in Cargo.lock: {line}"
        );
    }
}
