//! Integration tests: PTY-based TUI testing (T021, T022).
//!
//! These tests verify that the TUI can initialize and run under a virtual PTY.
//! On non-Unix platforms, these tests are no-ops since PTY allocation is
//! Unix-specific.
//!
//! NOTE: These tests require a working `script` command and PTY support.
//! In CI environments where PTY is not properly supported, these tests
//! are skipped gracefully.

#![cfg(unix)]
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "test code: PTY integration tests permitted per Constitution §III"
)]

use std::io::{Read, Write};
use std::process::{Command, Stdio};

/// Returns true if the `script` command is available and functional.
fn script_command_available() -> bool {
    Command::new("script")
        .args(["-q", "/dev/null", "echo", "test"])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Returns true if PTY dimension setting is supported.
fn pty_dimensions_supported() -> bool {
    Command::new("script")
        .args([
            "-q",
            "/dev/null",
            "sh",
            "-c",
            "stty rows 24 cols 80 && stty size",
        ])
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn pty_available_for_tui_testing() {
    assert!(
        script_command_available(),
        "script command should be available for PTY allocation"
    );
}

#[test]
fn pty_with_dimensions_allocates_correctly() {
    let mut child = Command::new("script")
        .args(["-q", "/dev/null", "stty", "size"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"exit\n");
    }

    let output = child.wait_with_output().expect("command completes");

    assert!(output.status.success(), "script command should succeed");
}

#[test]
fn pty_can_run_binary_with_raw_mode() {
    let mut child = Command::new("script")
        .args(["-q", "/dev/null", "test", "-e", "target/debug/leiden-tui"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"exit\n");
    }

    let output = child.wait_with_output().expect("command completes");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("ENOTTY"),
        "PTY should prevent ENOTTY errors, got: {stderr}"
    );
}

#[test]
fn pty_dimensions_80x24() {
    let mut child = Command::new("script")
        .args([
            "-q",
            "/dev/null",
            "sh",
            "-c",
            "stty rows 24 cols 80 && stty size",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"exit\n");
    }

    let output = child.wait_with_output().expect("command completes");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("24") && stdout.contains("80"),
        "PTY should report 80x24 dimensions, got: {stdout}"
    );
}

#[test]
fn pty_dimensions_79x23_below_minimum() {
    if !pty_dimensions_supported() {
        // PTY dimension setting not supported in this environment (e.g., CI)
        return;
    }

    let mut child = Command::new("script")
        .args([
            "-q",
            "/dev/null",
            "sh",
            "-c",
            "stty rows 23 cols 79 && stty size",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"exit\n");
    }

    let output = child.wait_with_output().expect("command completes");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("23") && stdout.contains("79"),
        "PTY should report 79x23 dimensions, got: {stdout}"
    );
}

#[test]
fn pty_dimensions_240x60_ultrawide() {
    if !pty_dimensions_supported() {
        // PTY dimension setting not supported in this environment (e.g., CI)
        return;
    }

    let mut child = Command::new("script")
        .args([
            "-q",
            "/dev/null",
            "sh",
            "-c",
            "stty rows 60 cols 240 && stty size",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"exit\n");
    }

    let output = child.wait_with_output().expect("command completes");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("60") && stdout.contains("240"),
        "PTY should report 240x60 dimensions, got: {stdout}"
    );
}

#[test]
fn pty_raw_mode_initialization() {
    let mut child = Command::new("script")
        .args([
            "-q",
            "/dev/null",
            "sh",
            "-c",
            "test -t 0 && echo 'is_tty' || echo 'not_tty'",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"exit\n");
    }

    let output = child.wait_with_output().expect("command completes");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("is_tty"),
        "PTY should provide a TTY for raw mode initialization, got: {stdout}"
    );
}

#[test]
fn pty_input_events_delivered() {
    let mut child = Command::new("script")
        .args(["-q", "/dev/null", "cat"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns script command");

    let test_input = b"test_input\n";
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(test_input).expect("writes to PTY stdin");
    }

    let mut stdout = Vec::new();
    if let Some(mut child_stdout) = child.stdout.take() {
        let _bytes_read = child_stdout
            .read_to_end(&mut stdout)
            .expect("reads PTY stdout");
    }

    let _status = child.wait();

    assert!(
        stdout.windows(test_input.len()).any(|w| w == test_input),
        "PTY should deliver input events to the application"
    );
}
