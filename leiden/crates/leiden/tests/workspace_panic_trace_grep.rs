//! Workspace-wide panic-trace grep test.
//!
//! Per `spec.md` FR-008 and Constitution §III (Panic-Free Error Propagation):
//! production code in the `leiden` workspace MUST NOT contain any
//! panic-prone macro usage (`panic!`, `todo!`, `unimplemented!`, `dbg!`,
//! `unreachable!`) or the panic-prone methods `.unwrap(` / `.expect(`. The
//! `[workspace.lints]` block enforces this at compile time via
//! `clippy::unwrap_used = deny`, `clippy::panic = deny`, etc. This test is
//! the **source-grep** counterpart that catches the same intent at the
//! file-scan level so the contract is enforced even on paths that bypass
//! the linter (e.g. a `#[allow(...)]` block that slipped past review).
//!
//! It is the library-side companion to T095a (Phase 5 CLI panic-trace grep)
//! and complements the workspace lint gate by walking the crate's `src/`
//! tree at test time and asserting zero matches against the panic-pattern
//! regex set.
//!
//! The companion shell wrapper `tools/check_no_panic_traces.sh` runs this
//! same check at the workspace level via the standard `rg`/`grep` tools and
//! is wired into CI alongside `cargo test --workspace`.

// Test code may panic to report scan failures; the production-code lint ban
// does not apply here per Constitution §III.
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::manual_let_else,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    reason = "test code: panic-trace grep diagnostic; production-code panic-free \
              ban does not apply per Constitution §III"
)]

use std::fs;
use std::path::PathBuf;

const FORBIDDEN_MACROS: &[&str] = &["panic!", "todo!", "unimplemented!", "unreachable!", "dbg!"];

const FORBIDDEN_METHODS: &[&str] = &[".unwrap(", ".expect("];

/// Returns the absolute path to the `leiden` library crate's `src/` tree.
#[must_use]
fn workspace_src_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    PathBuf::from(manifest_dir).join("src")
}

/// Recursively scans `root` for `.rs` files and asserts no forbidden
/// macro / method usage is present in non-comment, non-doc-comment contexts.
#[test]
fn workspace_no_panic_traces_under_any_test() {
    let src_root = workspace_src_root();

    let mut violations: Vec<(PathBuf, usize, String)> = Vec::new();
    scan_dir(&src_root, &mut violations);

    assert!(
        violations.is_empty(),
        "Constitution §III / FR-008 violated: forbidden panic-prone macros or methods found in production code under {src_root:?}. Offences: {violations:#?}",
    );
}

fn scan_dir(dir: &std::path::Path, violations: &mut Vec<(PathBuf, usize, String)>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, violations);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        scan_file(&path, violations);
    }
}

fn scan_file(path: &std::path::Path, violations: &mut Vec<(PathBuf, usize, String)>) {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return,
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return;
    };
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        // Skip line comments and doc comments.
        if trimmed.starts_with("//") {
            continue;
        }
        for needle in FORBIDDEN_MACROS {
            if line.contains(needle) {
                violations.push((path.to_path_buf(), idx + 1, format!("macro: {needle}")));
            }
        }
        for needle in FORBIDDEN_METHODS {
            if line.contains(needle) {
                violations.push((path.to_path_buf(), idx + 1, format!("method: {needle}")));
            }
        }
    }
}
