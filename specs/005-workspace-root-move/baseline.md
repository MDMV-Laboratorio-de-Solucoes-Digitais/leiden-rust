# Pre-Move Verification Baseline (SC-004 Anchor)

**Feature**: 005-workspace-root-move | **Date**: 2026-09-02
**Recorded from**: nested workspace `leiden/` (pre-move layout), branch `dev`
at `5573fa2` (Merge pull request #242), toolchain rustc 1.98.0 stable.

This is the comparison anchor for SC-004 / tasks.md T003 / T011 / T027: the
post-move suite must reproduce this pass/fail profile from the repository root.

## Results

| Suite | Command (cwd = `leiden/`) | Result | Notes |
|---|---|---|---|
| Check | `cargo check --workspace --all-targets` | PASS | clean, exit 0 |
| Format | `cargo fmt --all -- --check` | PASS | exit 0 |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | PASS | clean via clippy-sarif, exit 0 |
| Tests (debug) | `cargo nextest run --workspace` | PASS | 266 passed / 0 failed / 0 skipped |
| Tests (release) | `cargo nextest run --workspace --release` | PASS | 268 passed / 0 failed / 0 skipped |
| Docs | `cargo doc --workspace --no-deps` | PASS | exit 0 (missing_docs = deny) |
| Deny | `cargo deny --config deny.toml check` | PASS | advisories ok, bans ok, licenses ok, sources ok |

## Notes

- The debug/release test-count asymmetry (266 vs 268) is inherent to the
  suite: two perf/determinism tests are gated by
  `#[cfg(not(debug_assertions))]` (release gate per Constitution "`--release`
  test gate"), so the counts differ by profile. The profile to compare is
  per-suite pass/fail plus these counts.
- Baseline captured by /speckit-implement (tasks.md T003) prior to creating
  branch `005-workspace-root-move`.
