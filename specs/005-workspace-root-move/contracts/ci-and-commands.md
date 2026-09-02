# Contract: CI Steps & Developer Command Surface

**Feature**: 005-workspace-root-move

The interface this feature exposes to CI and to every contributor/agent is the
command surface below: after PR 1 each command runs from the repository root
with no pre-navigation. CI (`/.github/workflows/ci.yml`) runs exactly these
steps; the `working-directory: leiden` entries are deleted, so every step's
cwd is the checkout root — which now is the workspace root.

## CI contract (post-move)

| Step | Command (cwd = repo root) | Pass criterion |
|---|---|---|
| Rustfmt Check | `cargo fmt --check` | exit 0 |
| Clippy Workspace Check | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 (lint block unchanged) |
| Unit & Integration Tests (Debug) | `cargo nextest run --workspace` | all pass |
| Release Tests (perf/determinism gate) | `cargo nextest run --workspace --release` | all pass |
| Documentation Build | `cargo doc --workspace --no-deps` | exit 0 (`missing_docs = deny`) |
| Cargo Deny Check | `cargo deny --config deny.toml check` | advisories/licenses/bans clean |

Untouched and out of scope: checkout action, toolchain install step
(see research.md D9 for the pre-existing `1.85.0` pin observation),
nextest/cargo-deny installation steps, `RUSTFLAGS: "-D warnings"`.

## Developer command surface (post-move)

```sh
cargo check --workspace                                  # micro-verification loop
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace                            # "ct" alias
cargo nextest run --workspace --release                  # release gate (SC-001 perf)
cargo doc --workspace --no-deps
cargo deny --config deny.toml check
cargo run --release -p leiden-cli -- fixtures/karate.edg --format json   # fixture paths stay root-relative
cargo bench --bench local_moving -p leiden               # benches resolve fixtures via workspace root
```

All fixture references (`fixtures/*.edg`) resolve from the root; test binaries
resolve them via `CARGO_MANIFEST_DIR/../../fixtures` and benches via the
workspace root — both unchanged by the move.

## Failure contract

Any step failing after PR 1 that passed before the move (same pass/fail
profile, SC-004) is a defect of the restructuring, to be fixed within PR 1
before merge — with the exception of toolchain/CI-infra issues already
flagged as out-of-scope follow-ups (D9).
