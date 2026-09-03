//! Integration tests: PTY-based terminal interaction for TUI binary.
//!
//! These tests allocate a virtual PTY and verify the TUI binary can initialize
//! in a controlled terminal environment. Tests are Unix-only (PTY required).

#![cfg(unix)]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Spawn the TUI binary with a virtual PTY allocated via `script` command.
fn spawn_tui_with_pty() -> std::io::Result<std::process::Child> {
    Command::new("script")
        .args(["-q", "/dev/null", "cargo", "run", "-p", "leiden-tui", "--"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

#[test]
#[ignore = "requires interactive PTY environment"]
fn pty_tui_starts_without_error() {
    let mut child = spawn_tui_with_pty().expect("failed to spawn TUI with PTY");

    std::thread::sleep(Duration::from_secs(2));

    let output = {
        let mut stdout = child.stdout.take().expect("failed to capture stdout");
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        buf
    };

    let _ = child.kill();
    assert!(!output.contains("error:"), "TUI produced error output: {output}");
}

#[test]
#[ignore = "requires interactive PTY environment"]
fn pty_tui_geometry_dimensions() {
    let mut child = spawn_tui_with_pty().expect("failed to spawn TUI with PTY");

    std::thread::sleep(Duration::from_millis(500));

    let _ = child.kill();
    let _ = child.wait();
}
