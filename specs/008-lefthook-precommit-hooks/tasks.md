# Tasks: Lefthook Pre-commit Hooks

**Input**: Design documents from `/specs/008-lefthook-precommit-hooks/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: No test tasks generated — hooks are validated via manual git operations per quickstart.md.

**Organization**: Tasks are grouped by user story where directly applicable. Pre-push, commit-msg, and cache-warming hooks are shared infrastructure benefiting all stories.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Repository root**: `lefthook.yml` — single configuration file
- **Scripts directory**: `.lefthook/` — custom scripts if needed

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify prerequisites and scaffolding for lefthook configuration

- [x] T001 Verify lefthook binary is available (`lefthook version`) and document installation path
- [x] T002 Verify cocogitto binary is available (`cocogitto --version`) for commit-msg hook
- [x] T003 Verify cargo-nextest, cargo-deny, cargo-audit, cargo-llvm-cov are available for pre-push hook
- [x] T004 Create `lefthook.yml` at repository root with empty hook structure (pre-commit, prepare-commit-msg, commit-msg, pre-push, post-merge, post-checkout)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core configuration structure that all user stories depend on

- [x] T005 Configure `pre-commit` hook skeleton: `parallel: true` + empty command keys (fmt, clippy, check) in `lefthook.yml` (T007/T009/T010 populate the commands)
- [x] T006 Configure `pre-push` hook with `parallel: true` and command structure (test, deny, audit, doc, coverage) in `lefthook.yml`

---

## Phase 3: User Story 1 - Automatic Code Formatting on Commit (Priority: P1) 🎯 MVP

**Goal**: `cargo fmt` auto-applies formatting on commit; `--no-verify` bypass is mitigated by `prepare-commit-msg` backstop

**Independent Test**: Commit unformatted code → formatting is auto-applied, commit succeeds with formatted code. Commit with `--no-verify` → `prepare-commit-msg` still applies formatting.

### Implementation for User Story 1

- [x] T007 [US1] Configure `pre-commit` → `fmt` command: `cargo fmt || true` with `stage_fixed: true` and `glob: "*.rs"` in `lefthook.yml` (the `|| true` ensures fmt failures emit output but never block the commit per FR-001; syntax errors are caught by clippy/check)
- [x] T008 [US1] Configure `prepare-commit-msg` → `fmt-backstop` command: `cargo fmt --all && git add -u` in `lefthook.yml` (explicit `git add -u` replaces `stage_fixed: true` which is pre-commit-only)

**Checkpoint**: Formatting hooks work end-to-end — normal commits auto-format, `--no-verify` commits still get formatted via backstop

---

## Phase 4: User Story 2 - Fast Local Quality Gates (Priority: P2)

**Goal**: Clippy and cargo check run in parallel on pre-commit, blocking commits with clear error messages

**Independent Test**: Introduce clippy warning → commit blocked with specific violation. Valid code → commit completes in <5s.

### Implementation for User Story 2

- [x] T009 [US2] Configure `pre-commit` → `clippy` command: `cargo clippy --workspace --all-targets -- -D warnings` with `glob: "*.rs"` in `lefthook.yml`
- [x] T010 [US2] Configure `pre-commit` → `check` command: `cargo check --workspace` with `glob: "*.rs"` in `lefthook.yml`
- [x] T025 [US2] Verify FR-006 output behavior: confirm each tool's native stdout/stderr is surfaced on failure with file/line info, and "all checks passed" summary displays on success. Validate via quickstart.md scenarios per T023

**Checkpoint**: All three pre-commit checks (fmt, clippy, check) run in parallel, block on failure with clear output, complete in <5s

---

## Phase 5: User Story 3 - Bypass Prevention (Priority: P2)

**Goal**: `--no-verify` bypass is mitigated by non-bypassable `prepare-commit-msg` hook; policy documented

**Independent Test**: Attempt commit with `--no-verify` → `prepare-commit-msg` backstop still applies formatting. Normal commits → pre-commit hooks run as expected.

### Implementation for User Story 3

- [x] T012 [US3] Document `LEFTHOOK=0` as known bypass vector in `lefthook.yml` comments and rely on CI/CD as ultimate gate per FR-003

**Checkpoint**: Bypass prevention in place — `prepare-commit-msg` is non-bypassable for `--no-verify` (configured in T008), `LEFTHOOK=0` documented as known residual vector (CI/CD is ultimate gate for this vector)

---

## Phase 6: Shared Infrastructure (Pre-push, Commit-msg, Cache Warming)

**Purpose**: Heavier checks (pre-push), Conventional Commits enforcement (commit-msg), and build cache warming (post-merge/post-checkout) — benefits all user stories

- [x] T013 [P] Configure `commit-msg` → `conventional` command: `cog verify --file {1}` in `lefthook.yml`
- [x] T014 [P] Configure `pre-push` → `test` command: `cargo nextest run` in `lefthook.yml`
- [x] T015 [P] Configure `pre-push` → `deny` command: `cargo deny check` in `lefthook.yml`
- [x] T016 [P] Configure `pre-push` → `audit` command: `cargo audit` in `lefthook.yml`
- [x] T017 [P] Configure `pre-push` → `doc` command: `cargo doc --workspace --no-deps` in `lefthook.yml`
- [x] T018 [P] Configure `pre-push` → `coverage` command: `cargo llvm-cov` in `lefthook.yml`
- [x] T019 [P] Configure `post-merge` → `cache-warm` command: `nohup cargo build > /dev/null 2>&1 &` (background, non-blocking) in `lefthook.yml`
- [x] T020 [P] Configure `post-checkout` → `cache-warm` command: `nohup cargo build > /dev/null 2>&1 &` (background, non-blocking) in `lefthook.yml`

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, documentation, and edge case handling

- [x] T021 Add inline comments in `lefthook.yml` documenting: `LEFTHOOK=0` bypass vector, `prepare-commit-msg` as secondary backstop, performance target (<5s; benchmark: 10 files / 500 lines per FR-004/SC-002)
- [x] T022 Validate `lefthook.yml` syntax with `lefthook validate` (or equivalent)
- [x] T023 Run quickstart.md scenarios 1-12 to verify all functional requirements pass
- [x] T024 Add commented-out opt-in commands in `lefthook.yml` per FR-007: include additional checks (e.g., `cargo audit` for vulnerability scanning) as commented examples under pre-commit, with a header comment explaining "uncomment to enable additional checks (note: some checks like deny/audit already run in pre-push — these are pre-commit-only opt-in suggestions)"

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — creates initial `lefthook.yml` structure
- **User Story 1 (Phase 3)**: Depends on Foundational — adds fmt + prepare-commit-msg
- **User Story 2 (Phase 4)**: Depends on Foundational — adds clippy + check to pre-commit
- **User Story 3 (Phase 5)**: Depends on US1 completion — documents bypass prevention
- **Shared Infrastructure (Phase 6)**: Depends on Foundational — can run in parallel with US1/US2/US3
- **Polish (Phase 7)**: Depends on all previous phases

### User Story Dependencies

- **US1 (P1)**: Depends on Foundational (Phase 2) — No dependencies on other stories
- **US2 (P2)**: Depends on Foundational (Phase 2) — No dependencies on other stories
- **US3 (P2)**: Depends on Foundational (Phase 2) + US1 completion (documents prepare-commit-msg backstop)

### Within Each Phase

- Foundational structure first (T004, T005, T006)
- Then story-specific commands (T007-T008, T009-T010, T012)
- Shared infrastructure can proceed in parallel with story work
- Validation last (T021-T024)

### Parallel Opportunities

- Phase 1 setup checks (T001-T003) can run in parallel
- Phase 6 pre-push commands (T014-T018) can be configured in parallel (different commands, same file — sequential edits but no logical dependencies)
- Phase 6 post-merge/post-checkout (T019-T020) can run in parallel

---

## Parallel Example: Pre-push Configuration

```text
# All pre-push commands can be added in any order (different keys, same file):
Task: "Configure pre-push → test command (T014): cargo nextest run"
Task: "Configure pre-push → deny command (T015): cargo deny check"
Task: "Configure pre-push → audit command (T016): cargo audit"
Task: "Configure pre-push → doc command (T017): cargo doc --workspace --no-deps"
Task: "Configure pre-push → coverage command (T018): cargo llvm-cov"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify tooling)
2. Complete Phase 2: Foundational (scaffold lefthook.yml)
3. Complete Phase 3: User Story 1 (fmt + prepare-commit-msg)
4. **STOP and Validate**: Test formatting hooks via quickstart.md scenarios 1, 4, 5
5. Deploy if ready

### Incremental Delivery

1. Complete Setup + Foundational → Config scaffold ready
2. Add US1 → Test formatting independently → Deploy (MVP!)
3. Add US2 → Test quality gates independently
4. Add US3 → Test bypass prevention independently
5. Add Shared Infrastructure → Pre-push, commit-msg, cache warming
6. Polish → Validate all quickstart scenarios

### Sequential Strategy (Single Developer)

Since all tasks modify the same file (`lefthook.yml`), sequential execution is recommended:

1. Setup → verify tooling (T001-T003)
2. Scaffold → create lefthook.yml with empty hook structure (T004)
3. Pre-commit foundation → parallel: true, command structure (T005)
4. US1 → fmt + prepare-commit-msg (T007-T008)
5. US2 → clippy + check (T009-T010)
6. US3 → bypass documentation (T012)
7. Pre-push foundation → parallel: true (T006)
8. Pre-push commands → test, deny, audit, doc, coverage (T014-T018)
9. Commit-msg → cocogitto (T013)
10. Cache warming → post-merge, post-checkout (T019-T020)
11. Polish → comments, validation, quickstart, opt-in docs (T021-T024)

---

## Notes

- [P] tasks = different files, no dependencies (not applicable here — single file)
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All tasks modify `lefthook.yml` at repository root — no other files created
- `stage_fixed: true` only valid for `pre-commit` and `prepare-commit-msg` hooks
- `cog verify --file {1}` receives commit message file path via `{1}` placeholder
