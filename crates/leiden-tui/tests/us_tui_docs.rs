//! Integration test: All public items in event and colors modules are documented (T105a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::process::Command;

#[test]
fn tui_public_items_documented() {
    let output = Command::new("cargo")
        .arg("doc")
        .arg("-p")
        .arg("leiden-tui")
        .arg("--no-deps")
        .output()
        .expect("runs cargo doc");

    assert!(
        output.status.success(),
        "cargo doc must succeed without missing_docs warnings: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
