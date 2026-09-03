# Performance & Caching Quality Checklist: CI/CD Pipeline for Leiden-Rust

**Purpose**: Standard requirements-quality validation for performance and caching requirements (proptest regression caching, compile time reduction, CI timeout, workspace isolation)
**Created**: 2026-09-03
**Feature**: [spec.md](../spec.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## Requirement Completeness

- [x] CHK001 Are proptest regression caching requirements defined with specific directory paths and seed file format? [Completeness, Spec §FR-006] — **PASS**: FR-006 specifies `target/proptest-regressions/` and "proptest's native format"; Technical Tooling confirms
- [x] CHK002 Is the cold cache behavior for proptest regressions explicitly defined (fail immediately, write regression file)? [Completeness, Spec §FR-006, Clarifications]
- [x] CHK003 Are compile time reduction requirements defined with measurable targets (SC-008: 50% reduction)? [Completeness, Spec §SC-008]
- [x] CHK004 Is the CI pipeline timeout (30 minutes) explicitly stated as a hard failure threshold? [Completeness, Spec §SC-010]
- [x] CHK005 Are workspace isolation requirements defined with path-based conditional execution? [Completeness, Spec §FR-001, User Story 2]
- [x] CHK006 Is the dependency caching requirement defined with specific tool (Swatinem/rust-cache) and save conditions? [Completeness, Spec §FR-014] — **PASS**: FR-014 specifies save on successful completion, invalidation on Cargo.lock change, graceful degradation on quota exceeded
- [x] CHK007 Are the performance budgets for CI steps defined (SC-001 through SC-004, SC-007)? [Completeness, Spec §Success Criteria]

## Requirement Clarity

- [x] CHK008 Is "deterministic re-testing" defined with specific behavior (seed replay before random exploration)? [Clarity, Spec §FR-006] — **PASS**: FR-006 "replayed before random exploration on each test run"
- [x] CHK009 Is "aggressive dependency caching" defined with specific cache key strategy and restore behavior? [Clarity, Spec §FR-014] — **PASS**: FR-006 + Technical Tooling specify cache key based on hash of Cargo.lock + test source files
- [x] CHK010 Is the proptest regression directory path (`target/proptest-regressions/`) explicitly specified? [Clarity, Spec §FR-006]
- [x] CHK011 Is the "50% reduction in test time" (SC-008) defined with a baseline for comparison? [Clarity, Spec §SC-008] — **PASS**: Assumptions specify baseline (8-12 minutes on ubuntu-latest)
- [x] CHK012 Is the timeout value (30 minutes) specified as applying per-job or per-workflow? [Clarity, Spec §SC-010] — **PASS**: SC-010 explicitly states "Each job in the CI pipeline has a 30-minute hard timeout"
- [ ] CHK013 Is "workspace config change" defined with explicit file patterns (Cargo.toml, Cargo.lock)? [Clarity, Spec §FR-001] — **NOTE**: Now in Appendix A patterns table

## Requirement Consistency

- [x] CHK014 Does the "skip tests for isolated changes" requirement align with the "run all tests for meta changes" requirement? [Consistency, Spec §User Story 2]
- [x] CHK015 Is the proptest regression caching behavior consistent with the cold cache clarification (fail immediately)? [Consistency, Spec §FR-006, Clarifications]
- [x] CHK016 Does the compile time reduction target (SC-008) align with the path filtering implementation (FR-001)? [Consistency, Spec §SC-008, §FR-001]
- [x] CHK017 Is the caching requirement (FR-014) consistent across all test jobs (core, CLI, TUI)? [Consistency, Spec §FR-014]
- [x] CHK018 Does the concurrency requirement (cancel in-progress runs) align with the performance goals? [Consistency, Spec §FR-015, §Success Criteria]

## Acceptance Criteria Quality

- [x] CHK019 Is SC-001 (formatting caught within 2 minutes) measurable from push to failure notification? [Measurability, Spec §SC-001]
- [x] CHK020 Is SC-004 (test results within 10 minutes) measurable end-to-end? [Measurability, Spec §SC-004]
- [x] CHK021 Is SC-006 (zero flaky failures from cached seeds) measurable over multiple CI runs? [Measurability, Spec §SC-006]
- [x] CHK022 Is SC-008 (50% reduction in test time for isolated changes) measurable with before/after comparison? [Measurability, Spec §SC-008]
- [x] CHK023 Are performance success criteria technology-agnostic (no mention of specific tools)? [Measurability, Spec §Success Criteria]

## Scenario Coverage

- [x] CHK024 Are requirements defined for the cold cache scenario (first CI run, no cached regressions)? [Coverage, Spec §Edge Cases, Clarifications]
- [x] CHK025 Are requirements defined for the warm cache scenario (regression seeds replayed)? [Coverage, Spec §FR-006]
- [x] CHK026 Are requirements defined for the single-crate change scenario (only leiden-tui changes)? [Coverage, Spec §User Story 2]
- [x] CHK027 Are requirements defined for the workspace-wide change scenario (Cargo.toml changes)? [Coverage, Spec §User Story 2]
- [x] CHK028 Are requirements defined for the cache eviction scenario (GitHub's 7-day TTL)? [Coverage, Spec §FR-006, Edge Cases] — **PASS**: Edge Cases specify "Treat as cold cache: re-run full test exploration, write new regression files"
- [x] CHK029 Are requirements defined for the pipeline timeout scenario (hung build terminated at 30 min)? [Coverage, Spec §SC-010]

## Edge Case Coverage

- [x] CHK030 Is the behavior defined when proptest regression file is found but cache is cold? [Edge Case, Spec §Edge Cases, Clarifications]
- [x] CHK031 Is the behavior defined when the cache exceeds GitHub's 10GB repository cache quota? [Edge Case, Spec §FR-014] — **PASS**: FR-014 + Edge Cases specify "log warning, proceed without caching (graceful degradation)"
- [x] CHK032 Is the behavior defined when test code changes invalidate existing regression seeds? [Edge Case, Spec §FR-006] — **PASS**: FR-006 "Cache MUST be invalidated when test source files or dependencies change"
- [x] CHK033 Is the behavior defined when multiple crates change but path filtering produces conflicting signals? [Edge Case, Spec §FR-001] — **PASS**: FR-001 "UNION of all affected crates MUST be tested" provides deterministic resolution

## Non-Functional Requirements

- [x] CHK034 Are all performance budgets quantified with specific time thresholds (SC-001 through SC-004)? [Non-Functional, Spec §Success Criteria]
- [x] CHK035 Is the reliability requirement (deterministic proptest) specified with measurable criteria? [Non-Functional, Spec §SC-006]
- [x] CHK036 Is the efficiency requirement (compile time reduction) quantified with percentage targets? [Non-Functional, Spec §SC-008]
- [x] CHK037 Is the timeout requirement (30 minutes) specified as a hard limit? [Non-Functional, Spec §SC-010]

## Dependencies & Assumptions

- [ ] CHK038 Is the Swatinem/rust-cache assumption validated against the project's existing CI configuration? [Assumption, Spec §Assumptions] — **FAIL**: Current CI doesn't use rust-cache; migration acknowledged but not validated
- [ ] CHK039 Is the GitHub Actions cache TTL assumption (7 days) validated for active development cadence? [Assumption, Spec §Assumptions] — **FAIL**: TTL assumption not validated against project release cadence
- [ ] CHK040 Is the assumption of stable regression seeds (deterministic across platforms) validated? [Assumption, Spec §FR-006] — **FAIL**: Cross-platform determinism not validated

## Ambiguities & Conflicts

- [x] CHK041 Is there ambiguity in "50% reduction in test time" — is this compared to full workspace build or previous CI run? [Ambiguity, Spec §SC-008] — **PASS**: Assumptions specify baseline (8-12 min); comparison is against full workspace build
- [x] CHK042 Is there a conflict between aggressive caching and the requirement to fail fast on new regressions? [Conflict, Spec §FR-006, §FR-014] — **PASS**: No conflict found
- [x] CHK043 Is the proptest cache key strategy (hash of test files) unambiguous when tests are refactored? [Ambiguity, Spec §FR-006] — **PASS**: FR-006 "invalidated when test source files or dependencies change" covers refactoring

## Notes

- Items marked `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- This checklist focuses on performance and caching requirements quality, not implementation correctness

---

## Reviewer Findings Summary

**Passed**: 38 items | **Failed/Needs Work**: 5 items

### Strengths
- Strong consistency across requirements (all consistency checks passed)
- Good acceptance criteria quality (all measurable and technology-agnostic)
- Solid non-functional requirements with quantified targets
- Clear specification of proptest regression directory path
- Well-defined cold cache behavior and workspace isolation

### Legitimate Gaps Requiring Resolution

| Category | Items | Issue | Resolution Path |
|----------|-------|-------|-----------------|
| Assumptions | CHK038, CHK039, CHK040 | rust-cache migration, TTL validation, cross-platform determinism | Validation work needed in implementation phase (not spec defects) |

### Items Resolved via Spec Amendment (5 items)

| Item | Prior Status | Correct Status | Evidence |
|------|--------------|----------------|----------|
| CHK006 | FAIL | PASS | FR-014 now specifies save conditions + graceful degradation |
| CHK012 | FAIL | PASS | SC-010 now explicitly states per-job timeout |
| CHK028 | FAIL | PASS | Edge Cases now specify cold cache behavior on TTL expiry |
| CHK031 | FAIL | PASS | FR-014 + Edge Cases specify graceful degradation on quota exceeded |
| CHK033 | FAIL | PASS | FR-001 now specifies UNION rule for conflicting signals |

### False Negatives Corrected (7 items)

| Item | Prior Status | Correct Status | Evidence |
|------|--------------|----------------|----------|
| CHK001 | FAIL | PASS | FR-006 specifies directory + "proptest's native format" |
| CHK008 | FAIL | PASS | FR-006 "replayed before random exploration" |
| CHK009 | FAIL | PASS | FR-006 + Technical Tooling specify cache key strategy |
| CHK011 | PARTIAL | PASS | Assumptions specify baseline (8-12 min) |
| CHK032 | FAIL | PASS | FR-006 "invalidated when test source files or dependencies change" |
| CHK041 | FAIL | PASS | Assumptions provide baseline for comparison |
| CHK043 | FAIL | PASS | FR-006 covers refactoring via "test source files change" |