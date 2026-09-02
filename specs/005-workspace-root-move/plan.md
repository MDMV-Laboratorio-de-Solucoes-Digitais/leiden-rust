# Implementation Plan: Flatten Workspace Into Repository Root

**Branch**: `005-workspace-root-move` | **Date**: 2026-09-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-workspace-root-move/spec.md`

## Summary

Relocate the entire Cargo workspace from the nested `leiden/leiden/` directory to the
repository root (`/home/luis/development/leiden/`), eliminating the `repo/leiden/leiden/`
duplication. The move is executed as pure `git mv` rename operations in PR 1
(plus the minimal functional path fixes: CI `working-directory`, `.gitignore`
consolidation, and one test that walks `../specs/`), followed by PR 2, a
documentation sweep that rewrites every live reference to the old nested prefix.
No crate is renamed, split, or merged; the three-crate workspace
(`crates/leiden`, `crates/leiden-cli`, `crates/leiden-tui`), fixtures, toolchain
pin, and the verbatim constitution lint block all move unchanged and relative
paths inside the workspace keep working because the whole tree shifts uniformly.

## Technical Context

**Language/Version**: Rust, edition 2024; `rust-toolchain.toml` pins `channel = "stable"`
(local 1.98.0); MSRV floor 1.88.0 (constitution Additional Constraints).

**Primary Dependencies**: clap, serde, thiserror, tracing, ratatui 0.30.2, crossterm
0.29.0, criterion (benches), proptest; tooling: cargo-nextest (`ct`), cargo-deny,
rustfmt, clippy. No dependency changes in this feature.

**Storage**: N/A (filesystem + git restructuring; 27 flat fixture files under `fixtures/`).

**Testing**: `cargo nextest run --workspace` (debug correctness gate) +
`cargo nextest run --workspace --release` (release/perf gate) + `cargo fmt --check` +
`cargo clippy --workspace --all-targets -- -D warnings` + `cargo doc --workspace --no-deps`
+ `cargo deny --config deny.toml check`. Existing suite must pass with the same
profile as before the move (FR-004, SC-004).

**Target Platform**: Linux dev machines and GitHub Actions `ubuntu-latest` CI.

**Project Type**: Cargo workspace — one library crate + two binary crates (CLI, TUI).

**Performance Goals**: None new; existing release-mode perf contracts
(SC-001 ≤5 s on 100-node/500-edge fixture) must keep passing.

**Constraints**: FR-002/FR-003 — the restructuring commit must be rename-pure
(100% rename detection, no unrelated content edits); two-PR cutover with CI path
fix atomic in PR 1 (clarified 2026-09-02); FR-008 — no `leiden/` directory remains
in the repository tree.

**Scale/Scope**: 139 tracked files moved up one level; 3 crates; 27 fixture files;
6 CI steps re-pointed; 1 source-file path fix; ~2 live documentation files updated
in PR 2 (README.md relative links, design-system.md absolute `file://` links).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Library-First & Domain Modeling | PASS | No crate created/removed/renamed; workspace members `crates/{leiden,leiden-cli,leiden-tui}` keep identical relative member paths in `Cargo.toml`. |
| II. Strict Lint Compliance | PASS | `[workspace.lints.*]` block moves verbatim inside `Cargo.toml`; no weakening. `clippy.toml`/`deny.toml` move path-free. |
| III. Panic-Free Error Propagation | PASS | Only source edit is a path string constant in one integration test (`observability_checklist.rs`); no production code touched. |
| IV. Documentation & Visibility Discipline | PASS | No public API changes; `cargo doc` gate re-run from root must pass. |
| V. Test-First | PASS (N/A scope) | No new behavior to test-first; validation is the existing suite (incl. release gate) passing from the root. The single test path fix is a mechanical reference update, not a feature; its "test" is the suite itself. |
| VI. Observability & I/O Discipline | PASS | No logging/output code changes. |
| VII. Dependency & Build Rigor | PASS | `Cargo.lock` moves unchanged; no dependency edits; `cargo deny check` re-run from root. Micro-verification applies to the PR-1 commits (move commit → verify → config-fix commit(s)). |
| VIII. Knowledge Graph Context | PASS | Graph queried during planning (`graphify-out/GRAPH_REPORT.md`): 874 nodes / 1361 edges; three-crate + CI/spec communities confirmed — the move shifts paths uniformly and changes no crate boundary, so the graph stays structurally valid modulo paths. Per AGENTS.md and Principle VIII, `graphify-out/` MUST be regenerated after the move lands (local maintenance action; directory is gitignored). |

**Gate result**: No violations. Complexity Tracking table stays empty.

## Project Structure

### Documentation (this feature)

```text
specs/005-workspace-root-move/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── repository-layout.md   # Authoritative old→new file map (the layout contract)
│   └── ci-and-commands.md     # CI step + developer command surface contract
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root — POST-MOVE layout)

```text
/  (repository root = workspace root)
├── Cargo.toml              # workspace manifest (members = crates/*, [workspace.lints] verbatim)
├── Cargo.lock              # unchanged content, relocated
├── rust-toolchain.toml     # channel = stable, MSRV floor note
├── clippy.toml             # allowed-duplicate-crates
├── deny.toml               # cargo-deny config (advisories/licenses/bans/sources)
├── proptest.toml           # 1000 cases / 200 shrink iters
├── README.md               # relocated from leiden/ (relative links fixed in PR 2)
├── .gitignore              # merged: root copy already ⊇ nested copy; nested copy deleted
├── AGENTS.md               # stays (root governance)
├── design-system.md        # stays (file:// links updated in PR 2)
├── guide-to-strict-rust.md # stays
├── rust-code-rigor.md      # stays
├── crates/
│   ├── leiden/             # library crate (unchanged internals, incl. benches/, tests/)
│   ├── leiden-cli/         # CLI binary crate
│   └── leiden-tui/         # TUI binary crate
├── fixtures/               # 27 fixture files (.edg/.json), relocated unchanged
├── .github/workflows/ci.yml      # working-directory: leiden removed (PR 1)
├── .specify/  specs/  graphify-out/ (gitignored)
└── (no leiden/ directory — FR-008)
```

**Structure Decision**: Single-workspace-at-root layout (Option 1, Cargo workspace
variant). The repository root becomes the workspace root; the pre-existing root
governance files (`AGENTS.md`, the two rigor guides, `design-system.md`, `specs/`,
`.specify/`) remain in place and are joined by the relocated workspace files.
The only root-name collision is `.gitignore` (both levels have one); resolution:
the nested copy is a strict subset of the root copy, so the root copy wins and the
nested copy is deleted (`git rm leiden/.gitignore`) — see research.md D2.
There is no root-level `README.md`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`clippy.toml`, `deny.toml`, `proptest.toml`, `crates/`, or `fixtures/` today, so every
other relocated path lands collision-free.

## Complexity Tracking

> Empty — no constitution violations to justify.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none) | | |
