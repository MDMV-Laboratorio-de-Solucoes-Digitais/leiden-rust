# Tasks: Flatten Workspace Into Repository Root

**Input**: Design documents from `/specs/005-workspace-root-move/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: No new test tasks are generated. This feature introduces no new behavior — the existing verification suite (debug + release gates, fmt, clippy, doc, deny) is the acceptance gate per FR-004/SC-004, and the single permitted source edit is a path-reference fix explicitly allowed by FR-003. Constitution §V TDD gate therefore does not apply to new code; it applies to verification discipline (failing state must be observed before the fix commit where feasible).

**Organization**: Tasks are grouped by user story. PR 1 delivers US1 + US2 evidence on branch `005-workspace-root-move`; PR 2 delivers US3 on branch `005-workspace-root-move-docs` (FR-009 checkpoint gates it).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Repository root IS the workspace root after the move (see [contracts/repository-layout.md](contracts/repository-layout.md))
- Pre-move paths are relative to the nested workspace `leiden/`
- Post-move paths are relative to the repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Branch and artifact hygiene before any restructuring work

- [X] T001 Create feature branch `005-workspace-root-move` from `dev` (verify clean base: `git log --oneline -1` matches dev HEAD)
- [X] T002 [P] Commit the feature's speckit artifacts on the branch: `specs/005-workspace-root-move/` (spec, plan, research, data-model, contracts/, quickstart.md, checklists/) plus root guidance file `AGENTS.md` — commit message `docs(spec): add 005-workspace-root-move artifacts` (these are FR-007 artifacts written against the NEW layout; not part of the restructuring commit)
- [X] T003 Capture the pre-move verification baseline: run from `leiden/` the full suite (`cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo nextest run --workspace --release`, `cargo doc --workspace --no-deps`, `cargo deny --config deny.toml check`) and record the pass/fail profile (per-suite result + test counts) in `specs/005-workspace-root-move/baseline.md` — this is the SC-004 comparison anchor

**Checkpoint**: Branch ready, artifacts committed, baseline recorded — restructuring work can begin

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Nothing beyond Phase 1 — the restructuring has no shared infrastructure to build. Phase 1's baseline (T003) is the blocking prerequisite for all user stories.

**⚠️ CRITICAL**: No user story work begins until T001–T003 are complete.

**Checkpoint**: Foundation ready — US1 may begin

---

## Phase 3: User Story 1 - Work From the Repository Root (Priority: P1) 🎯 MVP

**Goal**: The entire workspace lives at the repository root and every command works from there with no pre-navigation (PR 1: pure move + minimal functional cutover).

**Independent Test**: Fresh clone + run the standard build/test/lint commands from the repository root — all succeed with zero `cd` steps (quickstart.md Scenarios 1, 2, 7; SC-001).

### Implementation for User Story 1

- [X] T004 [US1] Perform the pure move with `git mv`: relocate every tracked file from `leiden/` to the repository root exactly per [contracts/repository-layout.md](contracts/repository-layout.md) Moved table — `leiden/Cargo.toml` → `Cargo.toml`, `leiden/Cargo.lock` → `Cargo.lock`, `leiden/rust-toolchain.toml` → `rust-toolchain.toml`, `leiden/clippy.toml` → `clippy.toml`, `leiden/deny.toml` → `deny.toml`, `leiden/proptest.toml` → `proptest.toml`, `leiden/README.md` → `README.md`, `leiden/crates/` → `crates/`, `leiden/fixtures/` → `fixtures/` (139 tracked files; zero content edits; do NOT move or delete untracked `leiden/target/` or `leiden/.commandcode/` yet)
- [X] T005 [US1] Verify staged rename purity BEFORE committing: `git diff --cached -M --name-status --format=` MUST show only `R`-class entries for all 139 moved files and zero `A`/`D` pairs among source files (SC-002 pre-commit gate; FR-002)
- [X] T006 [US1] Commit the restructuring as a single pure commit: `refactor(workspace): relocate workspace to repository root` (FR-003: moves only; no config edits mixed in — those are T007–T010)
- [X] T007 [US1] Update `.github/workflows/ci.yml`: delete the six `working-directory: leiden` lines (fmt, clippy, debug nextest, release nextest, doc, deny steps) so every step runs from the checkout root — no other CI edits (clarified decision: atomic cutover; research D9's toolchain-pin observation stays untouched)
- [X] T008 [US1] Resolve the root `.gitignore` collision per the spec's union policy: `git rm leiden/.gitignore` (root copy is a strict superset — verify `graphify-out/`, `.opencode/`, `.agents/`, `.omo/` entries are present in root `.gitignore`); confirms FR-005
- [X] T009 [US1] Fix the single permitted test-harness path reference in `crates/leiden/tests/observability_checklist.rs`: change `workspace_root().join("../specs/001-leiden-algorithm/checklists/observability.md")` to `workspace_root().join("specs/001-leiden-algorithm/checklists/observability.md")` (FR-003 test-harness clause; FR-004; the ONLY source edit of the feature)
- [X] T010 [US1] Delete untracked leftovers so FR-008 holds: remove `leiden/target/` and `leiden/.commandcode/`, then confirm `test ! -e leiden` at the repository root (research D7; root scratch files `update_spec*.py`/`*.patch` are untouched)
- [X] T011 [US1] Run the full verification suite from the repository root (`cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo nextest run --workspace --release`, `cargo doc --workspace --no-deps`, `cargo deny --config deny.toml check`) and compare the profile against `specs/005-workspace-root-move/baseline.md` (FR-004, SC-004 — must match T003's baseline)
- [X] T012 [US1] Commit the minimal functional fixes as one logical change: `chore(workspace): repoint CI, ignore rules, and test path to repository root` (constitution atomic-commit rule; distinct from the restructuring commit)
- [X] T013 [US1] Run quickstart.md validation scenarios 1 (root inventory + no `leiden/`), 3 (fixture-path integrity incl. `cargo test -p leiden --test observability_checklist`), and 5 (ignore rules at new location) from the repository root
- [ ] T014 [US1] Open PR 1 (`005-workspace-root-move` → `dev`) with the FR-009 revertibility note and the baseline comparison in the description; maintainer review per Constitution §Review (pushing/PR creation requires explicit user approval)

**Checkpoint**: PR 1 open — repository root is the workspace root; suite green with baseline-identical profile; US1 independently validated

---

## Phase 4: User Story 2 - History and Review Integrity (Priority: P2)

**Goal**: Every moved file's history remains traceable; the restructuring commit is reviewable as a pure mechanical change.

**Independent Test**: Rename detection on the restructuring commit reports moves as renames (≥90% similarity, zero A/D source pairs) and the commit diff contains only moves plus permitted reference updates (quickstart.md Scenario 4; SC-002).

### Implementation for User Story 2

- [X] T015 [US2] Verify committed rename recognition: `git show -M --name-status --format= <restructuring-commit>` reports only `R`-class statuses for all 139 moved files with no `A`/`D` pairs among source files; record the count in the PR 1 description (SC-002)
- [X] T016 [P] [US2] Verify history tracing survives the move: `git log --follow --oneline README.md`, `git log --follow --oneline crates/leiden/src/lib.rs`, and `git log --follow --oneline crates/leiden-cli/tests/cli_format.rs` each reach pre-move commits (FR-002, US2-3)
- [X] T017 [P] [US2] Verify commit purity in review: the restructuring commit's full diff contains only renames plus the six `working-directory` deletions, the `.gitignore` removal, and the one-line test path fix — no feature or bug-fix edits (FR-003, US2-2)
- [X] T018 [US2] Document in the PR 1 description the rebase guidance for in-flight branches (path-level conflicts expected per Spec Edge Case + research D10: new files added under `leiden/` resolve by placing them at the new root locations)

**Checkpoint**: PR 1 carries complete rename/history evidence; US2 independently validated

---

## Phase 5: User Story 3 - Tooling and Documentation Point at the New Layout (Priority: P2)

**Goal**: Every live reference names the new top-level locations; nothing instructs "cd into leiden/ first" (PR 2).

**Independent Test**: Repository-wide search returns zero live references to the old nested prefix outside archived spec snapshots (quickstart.md Scenario 6; SC-003).

### Implementation for User Story 3

- [ ] T019 [US3] Execute the FR-009 post-merge validation checkpoint after PR 1 merges: on updated `dev`, run the full verification suite from the repository root plus quickstart.md scenarios 1 and 2 — this checkpoint MUST pass before any PR 2 work (fix-forward vs revert decision per FR-009 if it fails)
- [ ] T020 [P] [US3] Regenerate the codebase knowledge graph at the new root location (`graphify-out/` refresh per Constitution §VIII and AGENTS.md; gitignored, local maintenance action — not a PR artifact)
- [ ] T021 [US3] Create branch `005-workspace-root-move-docs` from updated `dev` (PR 2 requires the flattened layout to exist on its base)
- [ ] T022 [US3] Fix `README.md` relative links to the new layout: `../specs/001-leiden-algorithm/tasks.md` → `specs/001-leiden-algorithm/tasks.md` and `../.specify/memory/constitution.md` → `.specify/memory/constitution.md` (FR-006)
- [ ] T023 [P] [US3] Fix `design-system.md` absolute `file:///` links: update the three `file:///home/…/leiden/leiden/…` targets (Cargo.toml, leiden-tui/Cargo.toml ×2) to the post-move locations (FR-006)
- [ ] T024 [US3] Run the SC-003 stale-reference sweep exactly as operationalized in spec.md §SC-003 (`working-directory: leiden`, `leiden/leiden`, stale `../specs`/`../.specify` refs — excluding git history, gitignored dirs, archived `specs/001…004` snapshots): zero live hits required
- [ ] T025 [US3] Verify contributor setup works as written: execute the commands from the relocated `README.md` §Development & Verification and §CLI Usage from the repository root (US3-2)
- [ ] T026 [US3] Commit the documentation sweep: `docs: repoint workspace path references to repository root`, then open PR 2 (`005-workspace-root-move-docs` → `dev`); it is a normal PR subject to the unchanged CI pipeline (Spec §Clarifications)

**Checkpoint**: PR 2 open — zero live old-prefix references; US3 independently validated

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final completion evidence across both PRs

- [ ] T027 Verify the FR-009 completion criterion after PR 2 merges: both PRs landed, SC-003 search returns zero live references, quickstart.md Scenario 6 passes, and the final suite profile still matches `specs/005-workspace-root-move/baseline.md`
- [ ] T028 [P] Run quickstart.md Scenario 7 (fresh-clone equivalence): clone the merged repository and run `cargo check --workspace` + `cargo nextest run --workspace` with zero manual path fixes (SC-001 end-to-end proof)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies — start immediately
- **Foundational (Phase 2)**: T003 baseline BLOCKS all user stories (SC-004 comparison anchor)
- **US1 (Phase 3)**: depends on Setup; produces the restructuring commit (T006) that US2 verifies
- **US2 (Phase 4)**: depends on US1 T006 (commit exists); T015–T017 run on the branch, T018 writes PR description content
- **US3 (Phase 5)**: BLOCKED on PR 1 merge + T019 checkpoint (FR-009 ordering requirement) — NOT parallel with US1/US2 in time
- **Polish (Phase 6)**: depends on PR 2 merge

### User Story Dependencies

- **US1 (P1)**: independent — starts after Foundational
- **US2 (P2)**: depends on US1's restructuring commit; verification-only, no edits
- **US3 (P2)**: depends on US1 merged + FR-009 checkpoint passed; doc edits assume the flattened layout

### Within Each User Story

- US1: move (T004) → purity gate (T005) → commit (T006) → config fixes (T007–T010) → suite (T011) → commit (T012) → scenarios (T013) → PR (T014)
- US2: commit verification (T015) → parallel inspections (T016, T017) → PR documentation (T018)
- US3: checkpoint (T019) ∥ graph refresh (T020) → branch (T021) → parallel doc edits (T022, T023) → sweeps (T024, T025) → commit/PR (T026)

### Parallel Opportunities

- T002 and T003 are independent (artifacts vs baseline run)
- US2's T016 and T017 inspect independent aspects of the same commit
- US3's T020 runs alongside T019; T023 alongside T022 (different files)
- No cross-user-story parallelism: US3 is time-gated by PR 1 merge (FR-009)

---

## Parallel Example: User Story 2

```bash
# After the restructuring commit exists (US1 T006):
Task: "Verify history tracing via git log --follow for README.md and crate files (T016)"
Task: "Verify restructuring commit diff purity (T017)"
```

## Parallel Example: User Story 3

```bash
# After PR 1 merges and T019 checkpoint passes:
Task: "Regenerate graphify-out/ at new root (T020)"
# After T021 branch:
Task: "Fix README.md relative links (T022)"
Task: "Fix design-system.md file:/// links (T023)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (branch, artifacts, baseline)
2. Complete Phase 3 (US1): move → purity gate → commit → config cutover → suite green vs baseline → PR 1
3. **STOP and VALIDATE**: US1 is the entire point of the feature (SC-001/SC-002/SC-004 evidence); US2 evidence is a byproduct of T005/T006 discipline
4. US2 (Phase 4) closes the review-integrity evidence on the same PR

### Incremental Delivery

1. Setup + US1 + US2 → PR 1 merged → repository works from root (MVP complete)
2. FR-009 checkpoint (T019) → known-good gate
3. US3 → PR 2 merged → zero stale references (SC-003)
4. Polish → fresh-clone proof (T028) → feature complete per FR-009

### Single-Developer Strategy

Strictly sequential T001 → T028 is the intended path: the two-PR structure and FR-009 checkpoint impose the only real ordering; every [P] pair may interleave freely within its window.

---

## Notes

- [P] tasks = different files or independent inspections, no dependencies
- [Story] label maps task to spec user story for traceability
- Commit messages follow Constitution §Atomic Commits (conventional prefixes, workspace scope)
- T005 is a hard gate: if rename purity fails, stop and fix staging before T006 — never amend content into the restructuring commit
- T014/T026 (push + open PR) require explicit user approval before execution
- Stop at any checkpoint to validate the story independently
