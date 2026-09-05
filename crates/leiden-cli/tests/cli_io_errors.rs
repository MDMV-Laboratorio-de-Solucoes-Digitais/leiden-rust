//! Integration tests: I/O errors yield exit code 5 (T083a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

#[test]
fn io_error_yields_exit_code_5() {
    let bin = std::env::var("CARGO_BIN_EXE_leiden_cli").expect("binary");

    // 1. Missing path
    let missing_path = fixture_path("__does_not_exist__.edg");
    let out_missing = Command::new(&bin)
        .arg(&missing_path)
        .output()
        .expect("runs CLI");

    assert_eq!(out_missing.status.code(), Some(5));
    let stderr_missing = String::from_utf8(out_missing.stderr).expect("valid utf8");
    assert!(
        stderr_missing.starts_with("io: "),
        "stderr must start with 'io: ', got: {stderr_missing}"
    );

    // 2. Directory as file
    let dir_path = fixture_path("");
    let out_dir = Command::new(&bin)
        .arg(&dir_path)
        .output()
        .expect("runs CLI");

    assert_eq!(out_dir.status.code(), Some(5));
    let stderr_dir = String::from_utf8(out_dir.stderr).expect("valid utf8");
    assert!(
        stderr_dir.starts_with("io: "),
        "stderr must start with 'io: ', got: {stderr_dir}"
    );

    // 3. Permission denied (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let is_root = std::fs::read_to_string("/proc/self/status").is_ok_and(|status| {
            status
                .lines()
                .any(|l| l.starts_with("Uid:") && l.split_whitespace().nth(1) == Some("0"))
        });

        if !is_root {
            let temp = tempfile::NamedTempFile::new().expect("temp file");
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o000))
                .expect("set permissions 000");

            let out_perm = Command::new(&bin)
                .arg(temp.path())
                .output()
                .expect("runs CLI");

            assert_eq!(out_perm.status.code(), Some(5));
            let stderr_perm = String::from_utf8(out_perm.stderr).expect("valid utf8");
            assert!(
                stderr_perm.starts_with("io: "),
                "stderr must start with 'io: ', got: {stderr_perm}"
            );
        }
    }
}

#[cfg(not(unix))]
#[test]
fn io_error_unix_only_no_op_on_windows() {
    // Platform asymmetry documentation test for Windows
}
