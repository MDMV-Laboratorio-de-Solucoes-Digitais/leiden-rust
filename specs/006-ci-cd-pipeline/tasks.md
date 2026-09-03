# Tasks: CI/CD Pipeline for Leiden-Rust

**Input**: Design documents from `/specs/006-ci-cd-pipeline/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: NOT explicitly requested — test tasks omitted per spec.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- CI workflow: `.github/workflows/ci.yml`
- Release workflow: `.github/workflows/release.yml`
- Config files: `deny.toml`, `clippy.toml`, `rust-toolchain.toml`
- Crates: `crates/leiden/`, `crates/leiden-cli/`, `crates/leiden-tui/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare CI workflow structure and shared configuration

- [ ] T001 Create `.github/workflows/` directory structure and backup existing `ci.yml`
- [ ] T002 [P] Add shared environment variables to ci.yml: `CARGO_TERM_COLOR: always`, `RUSTFLAGS: "-D warnings"`, `TERM: xterm-256color`
- [ ] T003 [P] Add concurrency group to ci.yml: `group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: true`
- [ ] T004 Add 30-minute `timeout-minutes` to all jobs in ci.yml (SC-010)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Create `detect-changes` job in ci.yml using `dorny/paths-filter@v3` with filters for: `leiden`, `leiden-cli`, `leiden-tui`, `workspace-config`, `any-code` (per Appendix A)
- [ ] T006 Add `lint-and-audit` job in ci.yml running: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo deny check`
- [ ] T007 Add Swatinem/rust-cache setup step to ci.yml with `save-if: ${{ github.ref == 'refs/heads/main' }}`
- [ ] T008 Add cargo-nextest installation step using `taiki-e/install-action@nextest`

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Automated Quality Gates (Priority: P1) 🎯 MVP

**Goal**: Every push/PR runs formatting, linting, security audits, and tests automatically

**Independent Test**: Push a commit with a formatting error and observe CI failure with clear error message

### Implementation for User Story 1

- [ ] T009 [P] [US1] Add formatting check step: `cargo fmt --all -- --check` in `.github/workflows/ci.yml`
- [ ] T010 [P] [US1] Add clippy step: `cargo clippy --workspace --all-targets --all-features` with `-D warnings` in `.github/workflows/ci.yml`
- [ ] T011 [P] [US1] Add cargo-deny step: `cargo deny check` in `.github/workflows/ci.yml`
- [ ] T012 [US1] Add test execution step: `cargo nextest run --workspace --all-features --no-fail-fast` in `.github/workflows/ci.yml`
- [ ] T013 [US1] Add documentation build step: `cargo doc --workspace --no-deps` enforcing `missing_docs = deny` in `.github/workflows/ci.yml`

**Checkpoint**: US1 complete — quality gates run on every push/PR

---

## Phase 4: User Story 2 - Workspace-Aware Test Execution (Priority: P1)

**Goal**: Detect changed crates and only run tests for affected crates

**Independent Test**: Make a documentation-only change and observe that only relevant test jobs execute

### Implementation for User Story 2

- [ ] T014 [US2] Wire `detect-changes` job outputs to test job conditions using `needs.changes.outputs.*` in `.github/workflows/ci.yml`
- [ ] T015 [US2] Add `test-core` job with condition: `needs.changes.outputs.core == 'true' || needs.changes.outputs.meta == 'true'` in `.github/workflows/ci.yml`
- [ ] T016 [US2] Add `test-cli` job with condition: `needs.changes.outputs.cli == 'true' || needs.changes.outputs.core == 'true' || needs.changes.outputs.meta == 'true'` in `.github/workflows/ci.yml`
- [ ] T017 [US2] Add `test-tui` job with condition: `needs.changes.outputs.tui == 'true' || needs.changes.outputs.core == 'true' || needs.changes.outputs.meta == 'true'` in `.github/workflows/ci.yml`
- [ ] T018 [US2] Verify dependency topology: changing `leiden` core triggers CLI + TUI tests (FR-001)

**Checkpoint**: US2 complete — path filtering respects workspace dependency topology

---

## Phase 5: User Story 3 - Headless TUI Testing (Priority: P2)

**Goal**: Ratatui TUI tests run reliably in CI without terminal hardware

**Independent Test**: Run TUI tests in CI without PTY and observe tests pass using in-memory rendering buffers

### Implementation for User Story 3

- [ ] T019 [P] [US3] Add TUI unit tests using `TestBackend::new(width, height)` for rendering tests in `crates/leiden-tui/tests/`
- [ ] T020 [P] [US3] Add geometry guard tests at 80x24 (minimum), 79x23 (below-minimum), 240x60 (ultrawide) in `crates/leiden-tui/tests/test_geometry_guard.rs`
- [ ] T021 [US3] Add PTY integration test using `portable-pty` crate (with `script` fallback) in `crates/leiden-tui/tests/pty_integration.rs`
- [ ] T022 [US3] Add `#[cfg(unix)]` gate to PTY tests with documented no-op for non-Unix platforms

**Checkpoint**: US3 complete — TUI tests run headlessly in CI

---

## Phase 6: User Story 4 - Deterministic Property-Based Testing (Priority: P2)

**Goal**: Property-based tests are deterministic across CI runs

**Independent Test**: Observe that a proptest failure in one CI run causes the same seed to be re-tested in the next run

### Implementation for User Story 4

- [ ] T023 [P] [US4] Add proptest regression cache step using `actions/cache@v4` with path `target/proptest-regressions/` and key based on `crates/leiden/tests/*.rs` hash in `.github/workflows/ci.yml`
- [ ] T024 [P] [US4] Add `restore-keys: proptest-regressions-${{ runner.os }}-` for partial cache restore in `.github/workflows/ci.yml`
- [ ] T025 [US4] Verify cold cache behavior: new failures write regression file, subsequent runs replay seeds

**Checkpoint**: US4 complete — proptest regression seeds cached and replayed deterministically

---

## Phase 7: User Story 5 - Cross-Platform Release Automation (Priority: P3)

**Goal**: Pushing semantic version tag builds, packages, publishes binaries for Linux/macOS/Windows

**Independent Test**: Push version tag and observe GitHub Release with all platform artifacts and SHA-256 checksums

### Implementation for User Story 5

- [ ] T026 [P] [US5] Create `.github/workflows/release.yml` with trigger: `push: tags: - 'v[0-9]+.[0-9]+.[0-9]+*'`
- [ ] T027 [P] [US5] Add `permissions: contents: write` to release workflow
- [ ] T028 [P] [US5] Add build matrix with 5 targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc` in `.github/workflows/release.yml`
- [ ] T029 [P] [US5] Add cross-compilation step using `cross` tool for musl targets in `.github/workflows/release.yml`
- [ ] T030 [US5] Add binary stripping step for Unix targets in `.github/workflows/release.yml`
- [ ] T031 [US5] Add SHA-256 checksum generation step for each artifact in `.github/workflows/release.yml`
- [ ] T032 [US5] Add artifact packaging: tar.gz for Unix, zip for Windows in `.github/workflows/release.yml`
- [ ] T033 [US5] Add `publish-release` job using `softprops/action-gh-release@v2` with `generate_release_notes: true` in `.github/workflows/release.yml`
- [ ] T034 [US5] Add SHA256SUMS.txt manifest aggregation step in `.github/workflows/release.yml`
- [ ] T035 [US5] Enforce "no partial releases": fail entire release if any target fails (FR-010)

**Checkpoint**: US5 complete — version tag triggers automated multi-platform release

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T036 [P] Add `--release` profile test execution step (FR-018) in `.github/workflows/ci.yml`
- [ ] T037 [P] Add bench compile-check step: `cargo check --benches` in `.github/workflows/ci.yml`
- [ ] T038 [P] Add conditional documentation job with `doc-scope` counting logic (workspace/crate/skip) in `.github/workflows/ci.yml`
- [ ] T039 Validate ci.yml against quickstart.md scenarios
- [ ] T040 Run full CI pipeline on test branch and verify all jobs execute correctly

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel (both P1)
  - US3 and US4 can proceed in parallel (both P2)
  - US5 (P3) depends on US1 for CI foundation
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — No dependencies on other stories
- **US2 (P1)**: Can start after Foundational — Depends on detect-changes (Phase 2)
- **US3 (P2)**: Can start after Foundational — Independent
- **US4 (P2)**: Can start after Foundational — Independent
- **US5 (P3)**: Can start after US1 complete — Depends on CI foundation

### Within Each User Story

- Tests (if included) must be written and fail before implementation
- Workflow modifications must preserve existing functionality (don't break current `check-and-test`)
- Each job must have explicit conditions to avoid unnecessary execution

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel
- US1 + US2 can run in parallel after Foundational
- US3 + US4 can run in parallel after Foundational
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all quality gate steps for US1 together (different step positions in workflow):
Task: "Add formatting check step in .github/workflows/ci.yml"
Task: "Add clippy step in .github/workflows/ci.yml"
Task: "Add cargo-deny step in .github/workflows/ci.yml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test quality gates with intentional formatting/lint errors
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 → Test independently → Merge (MVP!)
3. Add US2 → Test independently → Merge
4. Add US3 + US4 → Test independently → Merge
5. Add US5 → Test independently → Merge
6. Polish phase → Final validation

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 (quality gates)
   - Developer B: US2 (path filtering)
   - Developer C: US3 (TUI testing)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- **CRITICAL**: Preserve existing `check-and-test` job functionality — don't break current CI
- Existing workflow uses `toolchain: "1.85.0"` but MSRV floor is 1.88.0 — note this discrepancy for resolution

---

## Phase 9: Convergence

**Purpose**: Remaining work identified by `/speckit-converge` to close the gap between spec/plan/tasks and current implementation.

- [ ] T041 [CRITICAL] Create `.github/workflows/release.yml` with: trigger `push: tags: - 'v[0-9]+.[0-9]+.[0-9]+*'`, `permissions: contents: write`, build matrix for 5 targets (x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc), cross-compilation via `cross` tool for musl targets, binary stripping for Unix, SHA-256 checksum generation, tar.gz/zip packaging, `softprops/action-gh-release@v2` publishing with `generate_release_notes: true`, SHA256SUMS.txt manifest aggregation, and "no partial releases" enforcement per FR-010/FR-011/FR-012/FR-013 (missing)
- [ ] T042 Add `detect-changes` job to ci.yml using `dorny/paths-filter@v3` with filters: `leiden`, `leiden-cli`, `leiden-tui`, `workspace-config`, `any-code` per Appendix A of spec.md per FR-001 (missing)
- [ ] T043 Wire `detect-changes` outputs to test job conditions: `test-core` (core \|\| meta), `test-cli` (cli \|\| core \|\| meta), `test-tui` (tui \|\| core \|\| meta) per FR-001, T014-T017 (missing)
- [ ] T044 Add proptest regression cache step using `actions/cache@v4` with path `target/proptest-regressions/` and key based on `crates/leiden/tests/*.rs` hash; add `restore-keys` for partial restore per FR-006, T023-T024 (missing)
- [ ] T045 Add PTY integration test file `crates/leiden-tui/tests/pty_integration.rs` using `portable-pty` crate (with `script` fallback) and `#[cfg(unix)]` gate with documented no-op for non-Unix per FR-008, T021-T022 (missing)
- [ ] T046 Add `Swatinem/rust-cache` setup step to ci.yml with `save-if: ${{ github.ref == 'refs/heads/main' }}` per FR-014, T007 (missing)
- [ ] T047 Add concurrency group to ci.yml: `group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: true` per FR-015, T003 (missing)
- [ ] T048 Add `timeout-minutes: 30` to all jobs in ci.yml per SC-010, T004 (missing)
- [ ] T049 Add `--release` profile test execution step (`cargo nextest run --workspace --release`) to ci.yml per FR-018, T036 (missing)
- [ ] T050 Add bench compile-check step `cargo check --benches` to ci.yml per FR-009, T037 (missing)
- [ ] T051 Add conditional documentation job with `doc-scope` counting logic (workspace/crate/skip) to ci.yml per FR-017, T038 (missing)
- [ ] T052 Add `--all-features` flag to clippy step: `cargo clippy --workspace --all-targets --all-features` per FR-003, T010 (partial)
- [ ] T053 Add `--all` flag to fmt step: `cargo fmt --all -- --check` per FR-002, T009 (partial)
- [ ] T054 Add `TERM: xterm-256color` to shared environment variables in ci.yml per T002 (partial)
- [ ] T055 Fix toolchain version mismatch: change ci.yml from `toolchain: "1.85.0"` to `toolchain: "stable"` to align with rust-toolchain.toml per Constitution MSRV floor 1.88.0 (partial)
- [ ] T056 Validate ci.yml against quickstart.md scenarios and run full CI pipeline on test branch per T039-T040 (partial)
