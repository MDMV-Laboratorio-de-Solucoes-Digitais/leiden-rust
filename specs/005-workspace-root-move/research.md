# Phase 0 Research: Flatten Workspace Into Repository Root

**Feature**: 005-workspace-root-move | **Date**: 2026-09-02

Every decision below was verified against the actual repository state
(exploration performed 2026-09-02 on branch `dev`). No NEEDS CLARIFICATION
items remained from the plan's Technical Context; this document records the
chosen mechanics for the restructuring and the evidence behind each choice.

## D1 — Move mechanics: `git mv` for all tracked files

**Decision**: Relocate every tracked file under `leiden/` (139 files) with
`git mv leiden/<path> <path>`, then verify rename detection with
`git status --find-renames` and `git diff --cached --stat -M90%` before
committing.

**Rationale**: `git mv` stages a rename explicitly, guaranteeing FR-002/SC-002
(100% rename recognition, ≥90% similarity) and keeping `git log --follow`
and blame intact (User Story 2). Since file *content* does not change during
the move, similarity is 100% for every moved file.

**Alternatives considered**:
- Plain `mv` + `git add -A`: relies on rename detection heuristics at commit
  time; works in practice for a content-identical move but is implicit and
  risks accidental content edits slipping into the same commit (FR-003).
- `git filter-repo` history rewrite: rewrites all history, breaks every
  existing clone and PR; rejected — the spec requires normal renames on a
  forward commit, and force-pushing rewritten history violates review
  integrity (User Story 2) and safe-execution practice.

## D2 — `.gitignore` collision: root copy wins, nested copy deleted

**Decision**: `git rm leiden/.gitignore`; keep the repository-root `.gitignore`
as the single ignore file. No content edits required.

**Rationale**: Verified line-by-line — the nested `leiden/.gitignore` (32
lines) is a strict subset of the root `.gitignore` (38 lines): the root copy
already contains every nested pattern (`/target/`, `/crates/*/target/`, IDE
junk, `*.snap.new`, OS junk, `.env*` handling, `*.log`, `.commandcode/`) plus
the root-only entries `.agents/`, `.omo/`, `graphify-out/`, `.opencode/`
(currently uncommitted working-tree additions that ship with PR 1). After the
move, `/target/` and `/crates/*/target/` match the regenerated build cache at
the root, so FR-005 holds: no previously-ignored file becomes tracked. This
is the spec's Edge Case "case/name collisions at the root" — for build/tool
files the workspace-side ignore rules are authoritative, and here they are
already identical, so the union equals the root file.

**Alternatives considered**:
- Merge both files into a combined list: redundant — the union was computed
  and equals the existing root file.
- Keep both `.gitignore` files (nested one moves up to a path where another
  `.gitignore` exists): impossible — two files cannot occupy one path; also
  violates FR-008's spirit of a single obvious root.

## D3 — CI cutover: same PR, `working-directory` removed

**Decision**: In PR 1, delete the six `working-directory: leiden` lines from
`.github/workflows/ci.yml` (all cargo steps then run from the checkout root,
which now *is* the workspace root). No other CI edits.

**Rationale**: The clarification session (2026-09-02) fixed the decision:
single atomic cutover — CI never runs against a stale path. All six re-pointed
steps (`cargo fmt --check`, `cargo clippy --workspace --all-targets`, debug
nextest, release nextest, `cargo doc`, `cargo deny`) resolve the workspace at
the checkout root after the move. `cargo deny --config deny.toml` still
resolves because `deny.toml` lands at the root.

**Alternatives considered**:
- Separate CI PR after the move: rejected — CI would fail (or silently test
  nothing) between the move commit and the CI fix; contradicts the clarified
  decision.
- Replace `working-directory: leiden` with `working-directory: .`: equivalent
  but noisier; plain removal is the minimal diff.

## D4 — Source-file path references: exactly one edit required

**Decision**: In PR 1, change one line in
`crates/leiden/tests/observability_checklist.rs`:
`workspace_root().join("../specs/001-leiden-algorithm/checklists/observability.md")`
→ `workspace_root().join("specs/001-leiden-algorithm/checklists/observability.md")`.

**Rationale**: Audited every filesystem-path construction in the workspace:

| Site | Resolution | After move |
|---|---|---|
| `crates/leiden-cli/tests/*.rs` (6 files): `CARGO_MANIFEST_DIR/../../fixtures` | crate dir → `crates/` → root, then `fixtures/` | Still correct (uniform shift) — no edit |
| `crates/leiden/tests/us1_*.rs`, `us2_*.rs`, `sc002_*.rs`: same `../../fixtures` pattern | same | Still correct — no edit |
| `crates/leiden/benches/{aggregation,local_moving,refinement}.rs`: `CARGO_MANIFEST_DIR` ancestors(2) + `join("fixtures")` (workspace-root-relative) | workspace root + `fixtures/` | Still correct — no edit |
| `crates/leiden/tests/observability_checklist.rs`: workspace root + `../specs/…` | workspace root's parent — intentionally *outside* the workspace, into repo-root `specs/` | **Breaks** (would resolve outside the repo) — one-line edit |
| `Cargo.toml` members `crates/leiden`, `crates/leiden-cli`, `crates/leiden-tui` | relative | Unaffected |
| `crates/*/Cargo.toml` intra-workspace `path = "../leiden"` deps | relative | Unaffected |
| `clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `proptest.toml` | no paths | Unaffected |

The one edit is a path-reference update required for FR-004 (full suite,
including this checklist-alignment test, passes from the root) and is
explicitly permitted by FR-003 ("minimal reference updates required for the
repository to function (path references inside manifests, configuration, CI
workflows, and fixture/spec-path references in test harnesses)").

**Alternatives considered**:
- Copy the checklist into the crate: duplicates governance content, creates a
  drift hazard; rejected.
- Make the test skip when the file is absent: hides a real integrity check;
  rejected.

## D5 — Two-PR split: minimal-functional vs documentation sweep

**Decision** (per clarified requirement FR-006):

*PR 1 — `005-workspace-root-move` (pure move + minimal functional fixes)*:
1. `git mv` all 139 tracked files from `leiden/` to the root
   (one rename-pure commit, e.g. `refactor(workspace): relocate workspace to repository root`).
2. `.github/workflows/ci.yml`: remove the six `working-directory: leiden` lines (atomic CI cutover).
3. `.gitignore`: delete nested copy (root copy already covers it, including the pending `graphify-out/`/`.opencode/` additions).
4. `observability_checklist.rs`: single path-reference fix (D4).
5. Local cleanup of untracked leftovers so FR-008 holds: delete `leiden/target/` (disposable build cache, regenerated at the root) and the session-local `leiden/.commandcode/` (untracked, gitignored).
6. Post-merge maintenance (not a PR artifact): regenerate `graphify-out/` (D8).

*PR 2 — docs sweep (`005-workspace-root-move-docs`, lands immediately after PR 1)*:
- `README.md` (relocated): fix `../specs/001-leiden-algorithm/tasks.md` → `specs/…` and `../.specify/memory/constitution.md` → `.specify/memory/constitution.md`.
- `design-system.md`: update the three `file:///home/…/leiden/leiden/…` absolute links to the new locations.
- Repository-wide grep for the old prefix (`leiden/leiden`, `working-directory: leiden`, `../specs` from root-relative contexts, `cd leiden`) to reach SC-003 zero live references.

**Out of scope for both PRs** (verified clean or archived):
- `AGENTS.md`, `guide-to-strict-rust.md`, `rust-code-rigor.md`: contain no old-prefix references (grep-verified).
- `.specify/` scripts, templates, workflows: grep shows zero `leiden/` path references; they resolve the repo root via git.
- Archived spec snapshots (`specs/001…004` tasks/plans referencing `leiden/crates/…`): historical records of work performed under the old layout — per the spec's Assumptions section they are rewritten only where actively misleading; these are completed task ledgers and stay as-is.
- `specs/005-workspace-root-move/*` (this feature's artifacts): already written against the new layout (FR-007).

**Alternatives considered**:
- One atomic PR including docs: rejected — contradicts the clarified two-PR decision and mixes mechanical moves with prose edits, weakening review of the rename purity (FR-003).
- Three PRs (move / CI / docs): rejected — the clarification explicitly bundles CI with the move.

## D6 — Branch strategy

**Decision**: Branch `005-workspace-root-move` from `dev` carries PR 1. PR 2
branches as `005-workspace-root-move-docs` from `dev` *after* PR 1 merges
(its edits assume the flattened layout).

**Rationale**: Keeps the rename-pure commit reviewable in isolation;
`git log --follow` on PR 2's doc edits remains meaningful.

**Alternatives considered**: Stacking PR 2 directly on PR 1's branch — forces
serial merge without review independence and couples the two changes.

## D7 — Untracked artifacts and local leftovers

**Decision**: `leiden/target/` is deleted, not moved (spec Edge Case: build
cache is disposable; `cargo` regenerates `target/` at the new root — root
`.gitignore` already covers `/target/`). `leiden/.commandcode/` (untracked
agent-session config, gitignored at both levels) is removed with the empty
`leiden/` shell once the tracked content is gone.

**Rationale**: FR-008 forbids any remaining `leiden/` directory. Neither item
is tracked, so deleting them changes no git content and keeps the
restructuring commit rename-pure. Untracked scratch files already at the
repository root (`update_spec*.py`, `*.patch`) are outside the move scope
entirely: `git mv` operates only on tracked paths, so they are neither moved
nor modified.

**Alternatives considered**: Moving `target/` up — wasteful (~GBs of cache
whose absolute paths are baked into artifacts; cargo would distrust and
rebuild anyway).

## D8 — Knowledge graph regeneration (Constitution VIII)

**Decision**: After PR 1 lands, regenerate `graphify-out/` (per AGENTS.md and
Constitution Principle VIII: "MUST be refreshed when significant structural …
boundaries are modified").

**Rationale**: During planning the graph was queried
(`graphify-out/GRAPH_REPORT.md`: 874 nodes / 1361 edges / 88 communities,
including "CI Pipeline & Strict Rust Rigor", "Workspace Cargo Packages", and
the three crate communities). It confirmed the move changes no crate boundary,
symbol relationship, or dependency edge — only file paths shift one level, so
symbol-level findings remain valid. The graph's stored paths nonetheless
become stale; regeneration is a local maintenance action (`graphify-out/` is
gitignored) and is listed as a task so it is not forgotten.

**Alternatives considered**: Regenerating pre-move (stale again immediately)
or never (violates Principle VIII and misleads future `/graphify` queries).

## D9 — Observed pre-existing CI toolchain inconsistency (out of scope)

**Decision**: Do not touch the `toolchain: "1.85.0"` pin in `ci.yml` during
the move. Record it as a follow-up candidate.

**Rationale**: The constitution's MSRV floor is 1.88.0 (imposed by ratatui
0.30.2) and `rust-toolchain.toml` pins `channel = "stable"`; CI's explicit
`1.85.0` input to `dtolnay/rust-toolchain` conflicts with both. Changing it in
PR 1 would be an unrelated content change (FR-003). The effective toolchain
selection inside the workspace is governed by `rust-toolchain.toml`
(`channel = "stable"`), which is why CI is currently green despite the stale
pin. Follow-up: align the CI input (or drop the pin to let `rust-toolchain.toml`
decide) in a separate PR.

**Alternatives considered**: Fixing it in PR 1 — violates rename-purity;
fixing it in PR 2 — PR 2 is scoped to path references only.

## D10 — Rebasing in-flight branches onto the new layout

**Decision**: No special tooling. Developers with branches based on the old
layout rebase normally; git's rename detection resolves the one-level shift
automatically. Expected conflicts are path-level only where a rebased branch
*adds* new files under `leiden/` (resolve by placing them under the new root)
or edits a moved file's content heavily.

**Rationale**: Spec Edge Case ("Existing local clones"): `git mv` renames are
recorded as ordinary rename entries, and three-way merge follows renames for
content merges. `git rebase` after the move PR lands will replay each commit
against the flattened tree with rename-aware diff application.

**Alternatives considered**: A compatibility shim (symlink `leiden` → `.`):
rejected — FR-008 forbids any `leiden/` directory existing, and a self-
referential symlink breaks tooling that enumerates the root.
