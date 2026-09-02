# Phase 1 Data Model: Flatten Workspace Into Repository Root

**Feature**: 005-workspace-root-move | **Date**: 2026-09-02

This feature restructures the repository rather than introducing runtime
domain types. The "entities" below are the filesystem, manifest, and git
constructs the change operates on, with the validation rules (drawn from the
spec's FRs) each must satisfy after the change.

## 1. Repository Root

The single top-level directory. Before the change it holds only governance
artifacts; after the change it is simultaneously repo root and workspace root.

| Aspect | Before | After |
|---|---|---|
| Workspace manifest | absent | `Cargo.toml` |
| Lockfile | absent | `Cargo.lock` |
| Toolchain pin | absent | `rust-toolchain.toml` |
| Lint/deny configs | absent | `clippy.toml`, `deny.toml`, `proptest.toml` |
| README | absent | `README.md` |
| Governance (stays) | `AGENTS.md`, `design-system.md`, `guide-to-strict-rust.md`, `rust-code-rigor.md`, `specs/`, `.specify/`, `.github/` | unchanged |
| Nested workspace dir | `leiden/` | **must not exist** (FR-008) |

**Validation**: root listing shows every artifact of the After column; `test ! -e leiden` passes.

## 2. Workspace Manifest (`Cargo.toml`)

- Fields: `resolver`, `members` (`crates/leiden`, `crates/leiden-cli`,
  `crates/leiden-tui`), `[workspace.lints.rust]`, `[workspace.lints.clippy]`
  (constitution §II block, verbatim), `[workspace.package]`
  (edition 2024, rust-version 1.88, MIT OR Apache-2.0, publish = false),
  `[profile.release]`, `[profile.dev]`.
- **Constraint**: member paths are workspace-relative and are *identical*
  before and after the move — the manifest requires **zero content edits**.
- **Validation**: `cargo metadata --no-deps` from the root resolves all three
  members; the lints block is byte-identical to `rust-code-rigor.md`'s canonical block.

## 3. Crate Directories

`crates/leiden` (library), `crates/leiden-cli` (binary), `crates/leiden-tui`
(binary). Internal contents (src, tests, benches, crate manifests) move
unchanged.

- Intra-workspace dependencies stay relative: `leiden-cli` → `path = "../leiden"`,
  `leiden-tui` → `path = "../leiden"`, `path = "../leiden-cli"` — **zero edits**.
- Path-sensitive code inside crates (audited in research.md D4):
  - `CARGO_MANIFEST_DIR/../../fixtures` in 6 CLI test files + 6 library test
    files: resolves to workspace root + `fixtures/` — correct after uniform shift, **no edit**.
  - `benches/*.rs` `fixtures_dir()` = `CARGO_MANIFEST_DIR` ancestors(2) + `join("fixtures")`:
    workspace-root-relative — **no edit**.
  - `crates/leiden/tests/observability_checklist.rs`: workspace root + `../specs/…`
    (escaped the workspace into repo `specs/`) — **one-line edit** to
    `join("specs/…")`.
- **Validation**: full suite green from root; the observability-checklist test
  finds the file at `specs/001-leiden-algorithm/checklists/observability.md`.

## 4. Path References

Any textual occurrence of the old nested prefix in live files.

| Reference class | Examples | PR | Treatment |
|---|---|---|---|
| CI working directories | `working-directory: leiden` ×6 in `ci.yml` | 1 | removed (lines deleted) |
| Ignore rules | `leiden/.gitignore` | 1 | deleted (root copy is a superset) |
| Test path constants | `../specs/…` in `observability_checklist.rs` | 1 | rewritten to root-relative `specs/…` |
| README relative links | `../specs/001-leiden-algorithm/tasks.md`, `../.specify/memory/constitution.md` | 2 | rewritten root-relative |
| Absolute doc links | 3 × `file:///…/leiden/leiden/…` in `design-system.md` | 2 | rewritten |
| Agent guidance | `AGENTS.md`, `guide-to-strict-rust.md`, `rust-code-rigor.md` | — | verified clean (no occurrences) |
| Speckit scripts/templates | `.specify/**` | — | verified clean |
| Archived spec snapshots | `specs/001…004` task ledgers | — | intentionally preserved (historical record) |

**Validation**: `grep -rn "leiden/"` over live (non-archived, non-ignored)
files returns only benign self-references (e.g. crate names `crates/leiden`,
this feature's spec text); zero `working-directory: leiden`; zero stale `../specs` from root contexts (SC-003).

## 5. Git Rename Records

The history-integrity entity. Each of the 139 moved tracked files must appear
in the restructuring commit as a rename (R100-class similarity), never as a
delete+add pair.

- **Validation**: `git show --stat -M --diff-filter=R` counts 139 renames;
  `git show --diff-filter=D --name-only` (excluding the intentionally-deleted
  `leiden/.gitignore`… note: that deletion *pairs* with the pre-existing root
  `.gitignore` edit, it is a removal, not a content edit) and
  `--diff-filter=A` lists no source files; `git log --follow README.md`
  reaches pre-move history.

## 6. Ignore Rules (`.gitignore`)

Single root-level file; union of both pre-move files (they were already
nested ⊂ root; verified line-by-line in research.md D2).

- Patterns of record: `/target/`, `/crates/*/target/` (build cache at the new
  location), `graphify-out/`, `.commandcode/`, `.agents/`, `.omo/`,
  `.opencode/`, IDE/OS junk, `.env*` (with `!.env.example`), `*.log`, `*.snap.new`.
- **Validation (FR-005)**: after `cargo build` at the root, `git status --porcelain`
  shows no untracked `target/` entries; `git check-ignore target graphify-out` succeeds.

## State Transitions

```text
BEFORE (branch dev)                          AFTER (PR 1 merged)
repo/                                        repo/
├── leiden/            ← workspace           ├── Cargo.toml         (moved)
│   ├── Cargo.toml                           ├── Cargo.lock         (moved)
│   ├── crates/{leiden,leiden-cli,leiden-tui}├── crates/…           (moved)
│   ├── fixtures/ (27 files)                 ├── fixtures/          (moved)
│   ├── target/ (untracked, disposable)      ├── README.md          (moved)
│   └── *.toml, README.md, .gitignore        ├── *.toml configs     (moved)
├── .gitignore (root, superset)              ├── .gitignore         (root copy kept)
├── .github/workflows/ci.yml                 ├── .github/…/ci.yml   (working-directory removed)
├── AGENTS.md, guides, specs/, .specify/     ├── unchanged
                                             └── (leiden/ gone; target/ regenerated on demand)

AFTER (PR 2 merged): README/design-system links rewritten; zero live old-prefix references.
```
