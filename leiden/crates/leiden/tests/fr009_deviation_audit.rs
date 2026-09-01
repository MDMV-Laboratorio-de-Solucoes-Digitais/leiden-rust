//! FR-009 Citation and Deviation Audit (T138a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions permitted per Constitution §III"
)]

use std::fs;
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

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn fr009_deviations_are_documented() {
    let root = workspace_root();
    let algorithm_dirs = [
        root.join("crates/leiden/src/local_moving"),
        root.join("crates/leiden/src/refinement"),
        root.join("crates/leiden/src/aggregation"),
        root.join("crates/leiden/src/quality"),
        root.join("crates/leiden/src/orchestrator"),
    ];

    let mut audited_files = Vec::new();
    for dir in &algorithm_dirs {
        if dir.exists() {
            collect_rs_files(dir, &mut audited_files);
        }
    }

    assert!(
        !audited_files.is_empty(),
        "Must find algorithm source files to audit citations"
    );

    for file in audited_files {
        let content = fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read file {file:?}: {err}"));

        let has_citation =
            content.contains("ref: Traag 2019 §") || content.contains("leiden-deviation:");
        assert!(
            has_citation,
            "FR-009 violation: algorithm file {:?} lacks `// ref: Traag 2019 §X.Y` citation or `// leiden-deviation:` marker",
            file
        );

        // Check citation format and rationale length
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find("// ref:") {
                let citation_part = &trimmed[pos + 7..].trim();
                assert!(
                    citation_part.contains("Traag 2019 §"),
                    "Citation in {:?} must match `Traag 2019 §`: {}",
                    file,
                    line
                );
                assert!(
                    citation_part.len() >= 15,
                    "Citation rationale in {:?} too short (< 15 chars): {}",
                    file,
                    line
                );
            }
        }
    }
}
