# CI Pipeline Quality Checklist: CI/CD Pipeline for Leiden-Rust

**Purpose**: Rigorous requirements-quality validation for CI pipeline requirements (path filtering, test orchestration, conditional docs, caching, quality gates)
**Created**: 2026-09-03
**Last Verified Against Spec**: 2026-09-03 (spec.md v1.0, 197 lines)
**Feature**: [spec.md](../spec.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## Requirement Completeness

- [x] CHK001 Are path-filtering rules defined for all three crates (leiden, leiden-cli, leiden-tui) with explicit file patterns? [Completeness, Spec §FR-001, Appendix A] — **PASS**: FR-001 + Appendix A explicitly list `crates/leiden/**`, `crates/leiden-cli/**`, `crates/leiden-tui/**`
- [x] CHK002 Is the behavior for workspace configuration changes (Cargo.toml, Cargo.lock) explicitly defined in path-filtering requirements? [Completeness, Spec §FR-001]
- [x] CHK003 Are documentation-only changes distinguished from code changes in the path-filtering logic? [Completeness, Spec §FR-001, Edge Case]
- [x] CHK004 Is the exact formatting check command (`cargo fmt --check`) and its failure behavior specified? [Completeness, Spec §FR-002]
- [x] CHK005 Are Clippy invocation parameters (workspace, all-targets, all-features, -D warnings) explicitly documented? [Completeness, Spec §FR-003]
- [x] CHK006 Is the `cargo deny check` command and its validation scope (advisories, licenses, duplicates) specified? [Completeness, Spec §FR-004] — **PASS**: FR-004 defines capability; tooling in Technical Tooling section
- [x] CHK007 Are all test categories (unit, integration, property-based) explicitly required in the spec? [Completeness, Spec §FR-005]
- [x] CHK008 Is the cargo-nextest runner specified as the required test execution tool? [Completeness, Spec §FR-005]
- [x] CHK009 Are proptest regression caching mechanics (directory path, seed format, replay behavior) defined? [Completeness, Spec §FR-006] — **PASS**: FR-006 specifies `target/proptest-regressions/`, native format, replay before exploration
- [x] CHK010 Is the cold cache behavior for proptest regressions explicitly defined (fail immediately, write regression file)? [Completeness, Spec §FR-006, Clarifications]
- [x] CHK011 Are TUI TestBackend requirements specified (no CrosstermBackend in unit tests, in-memory rendering)? [Completeness, Spec §FR-007]
- [x] CHK012 Is virtual PTY allocation for integration tests defined with specific mechanism (portable-pty or script)? [Completeness, Spec §FR-008] — **PASS**: portable-pty (primary) + script (fallback) in Technical Tooling; FR-008 defines capability
- [x] CHK013 Is the benchmark compile-check-only requirement distinguished from benchmark execution? [Completeness, Spec §FR-009]
- [x] CHK014 Is the timeout value (30 minutes) explicitly stated as a hard failure threshold? [Completeness, Spec §SC-010]
- [x] CHK015 Are all concurrency requirements defined (cancel in-progress runs on new commits)? [Completeness, Spec §FR-015]

## Requirement Clarity

- [x] CHK016 Is "affected crates" defined with explicit path patterns (e.g., `crates/leiden/**`)? [Clarity, Spec §FR-001, Appendix A] — **PASS**: Appendix A provides explicit path-to-crate mapping
- [x] CHK017 Is the threshold for "many modules changed" (≥2 crates) explicitly quantified? [Clarity, Spec §User Story 2] — **PASS**: FR-001 defines workspace config → full workspace; Appendix A clarifies thresholds
- [x] CHK018 Is the proptest regression directory path (`target/proptest-regressions/`) explicitly specified? [Clarity, Spec §FR-006] — **PASS**: FR-006 specifies directory path
- [x] CHK019 Is "deterministic re-testing" defined with specific behavior (seed replay before random exploration)? [Clarity, Spec §FR-006] — **PASS**: FR-006 "replayed before random exploration"
- [x] CHK020 Is "terminal interaction" for PTY tests distinguished from "raw mode initialization"? [Clarity, Spec §FR-008] — **PASS**: FR-008 mentions both raw mode and input events
- [x] CHK021 Are "virtual PTY dimensions" specified (width × height) for geometry-sensitive tests? [Clarity, Spec §User Story 3] — **PASS**: FR-008 specifies 80x24, 79x23, 240x60
- [x] CHK022 Is "below-minimum terminal dimensions" quantified with specific width/height values? [Clarity, Spec §User Story 3, Edge Cases] — **PASS**: FR-008 specifies 79x23; Edge Cases confirm overlay behavior
- [x] CHK023 Is the panic trace grep mechanism (workspace_panic_trace_grep.rs) referenced as an existing test? [Clarity, Spec §FR-009] — **PASS**: FR-009 references compile-check; test file exists in codebase
- [x] CHK024 Is "aggressive dependency caching" defined with specific tool (Swatinem/rust-cache) and behavior? [Clarity, Spec §FR-014] — **PASS**: Tool and behavior in Technical Tooling + Assumptions

## Requirement Consistency

- [x] CHK025 Do path-filtering requirements in FR-001 align with acceptance scenarios in User Story 2? [Consistency, Spec §FR-001 vs §User Story 2]
- [x] CHK026 Is the `-D warnings` requirement consistent across fmt, clippy, and doc generation steps? [Consistency, Spec §FR-002/003/FR-006] — **PASS**: FR-017 doc build enforces same lint strictness
- [x] CHK027 Does the proptest regression behavior (fail immediately on cold cache) align with the determinism requirement? [Consistency, Spec §FR-006 vs §Clarifications]
- [x] CHK028 Is the TestBackend requirement for unit tests consistent with the PTY requirement for integration tests? [Consistency, Spec §FR-007 vs §FR-008]
- [x] CHK029 Does the "skip tests for documentation-only changes" requirement align with FR-001's path filtering? [Consistency, Spec §Clarifications vs §FR-001]
- [x] CHK030 Is the concurrency requirement (cancel in-progress) consistent across push and pull_request triggers? [Consistency, Spec §FR-015]
- [x] CHK031 Does the TUI test skip behavior for non-TUI changes align with the workspace isolation goal? [Consistency, Spec §User Story 2 vs §User Story 3]

## Acceptance Criteria Quality

- [x] CHK032 Is SC-001 (formatting caught within 2 minutes) measurable from the user's perspective? [Measurability, Spec §SC-001]
- [x] CHK033 Is SC-004 (test results within 10 minutes) measurable and testable? [Measurability, Spec §SC-004]
- [x] CHK034 Is SC-008 (50% reduction in test time for isolated changes) quantified with baseline comparison? [Measurability, Spec §SC-008] — **PASS**: Assumptions state baseline (8-12 minutes)
- [x] CHK035 Is SC-005 (TUI tests without terminal initialization errors) objectively verifiable? [Measurability, Spec §SC-005]
- [x] CHK036 Is SC-006 (zero flaky failures from cached seeds) measurable over multiple CI runs? [Measurability, Spec §SC-006]
- [x] CHK037 Are success criteria technology-agnostic (no mention of specific tools like cargo-nextest in SC-001 through SC-010)? [Measurability, Spec §Success Criteria] — **PASS**: SC-001 through SC-010 mention no implementation technologies

## Scenario Coverage

- [x] CHK038 Are requirements defined for the single-crate change scenario (only leiden changes)? [Coverage, Spec §User Story 2]
- [x] CHK039 Are requirements defined for the multi-crate change scenario (2+ crates change)? [Coverage, Spec §User Story 2] — **PASS**: FR-001 dependency topology covers this
- [x] CHK040 Are requirements defined for the workspace-config-only change scenario? [Coverage, Spec §User Story 2, Edge Cases]
- [x] CHK041 Are requirements defined for the non-code-only change scenario (README, fixtures)? [Coverage, Spec §Edge Cases, Clarifications]
- [x] CHK042 Is the proptest cold cache scenario (first run, no cache) addressed in requirements? [Coverage, Spec §Edge Cases]
- [x] CHK043 Are TUI below-minimum-dimension scenarios (79x23) addressed with expected behavior? [Coverage, Spec §User Story 3, Edge Cases]
- [x] CHK044 Is the TUI minimal valid dimension scenario (80x24) addressed? [Coverage, Spec §User Story 3] — **PASS**: FR-008 specifies 80x24 as default; Edge Cases confirm normal rendering
- [x] CHK045 Is the TUI ultrawide scenario (240x60) addressed? [Coverage, Spec §User Story 3] — **PASS**: FR-008 specifies 240x60; Edge Cases confirm aspect ratio constraints
- [x] CHK046 Are dependency vulnerability detection scenarios defined (transitive vs direct)? [Coverage, Spec §User Story 1, Edge Cases] — **PASS**: Edge Cases state "same as direct dependency"

## Edge Case Coverage

- [x] CHK047 Is the behavior defined when proptest regression file is found but cache is cold? [Edge Case, Spec §Edge Cases]
- [x] CHK048 Is the behavior defined when TUI dimensions are exactly at minimum threshold vs below? [Edge Case, Spec §User Story 3] — **PASS**: Edge Cases specify 80x24 = normal, 79x23 = overlay
- [x] CHK049 Is the behavior defined when a cross-platform build fails for one target but succeeds for others? [Edge Case, Spec §Edge Cases, Clarifications]
- [x] CHK050 Is the behavior defined when only documentation or non-code files change? [Edge Case, Spec §Edge Cases, Clarifications]
- [x] CHK051 Is the behavior defined when a vulnerability is found in a transitive dependency? [Edge Case, Spec §Edge Cases] — **PASS**: Edge Cases state same handling as direct dependency
- [x] CHK052 Is the behavior defined when path filtering detects changes in fixtures directory? [Edge Case, Spec §FR-001] — **PASS**: Appendix A maps `fixtures/**` → core tests

## Non-Functional Requirements

- [x] CHK053 Are performance requirements (SC-001 through SC-004, SC-007) quantified with specific time thresholds? [Non-Functional, Spec §Success Criteria]
- [x] CHK054 Is the reliability requirement (deterministic proptest) specified with measurable criteria? [Non-Functional, Spec §SC-006]
- [x] CHK055 Is the security requirement (dependency audit) specified with specific validation scope? [Non-Functional, Spec §FR-004]
- [x] CHK056 Is the maintainability requirement (workspace isolation) quantified with expected time reduction? [Non-Functional, Spec §SC-008] — **PASS**: Assumptions provide baseline (8-12 min) for SC-008 measurement
- [x] CHK057 Are there explicit requirements for CI status communication method (GitHub native PR checks only)? [Non-Functional, Spec §FR-016]

## Dependencies & Assumptions

- [x] CHK058 Is the assumption of GitHub Actions as the CI platform validated against the existing project setup? [Assumption, Spec §Assumptions] — **PASS**: GitHub Actions confirmed
- [x] CHK059 Is the assumption of standard runner availability (ubuntu-latest, macos-13/14, windows-latest) validated? [Assumption, Spec §Assumptions] — **PASS**: Standard runners confirmed
- [ ] CHK060 Is the Swatinem/rust-cache assumption consistent with the project's existing CI configuration? [Assumption, Spec §Assumptions] — **GAP**: Current ci.yml doesn't use rust-cache; migration acknowledged in Assumptions but not yet implemented
- [x] CHK061 Is the cargo-nextest assumption consistent with the project's existing test infrastructure? [Assumption, Spec §Assumptions] — **PASS**: Documented in Technical Tooling
- [ ] CHK062 Is the cross tool assumption for musl targets validated against the existing release workflow? [Assumption, Spec §Assumptions] — **GAP**: Current ci.yml doesn't use cross; migration acknowledged in Assumptions but not yet implemented
- [x] CHK063 Is the assumption about TestBackend architecture (existing pattern) validated against the codebase? [Assumption, Spec §Assumptions] — **PASS**: Existing codebase uses TestBackend

## Ambiguities & Conflicts

- [x] CHK064 Is "affected crates" unambiguous when fixtures directory changes (is leiden crate "affected" by fixture changes)? [Ambiguity, Spec §FR-001] — **PASS**: Appendix A explicitly maps `fixtures/**` → leiden core tests
- [x] CHK065 Is there a conflict between "skip tests for doc-only changes" and "run tests when Cargo.toml changes"? [Conflict, Spec §Clarifications] — **PASS**: No conflict; different triggers
- [x] CHK066 Is "cargo doc --workspace" unambiguous given the existing binary name collision (leiden lib vs leiden-cli bin)? [Ambiguity, Spec §FR-017] — **PASS**: cargo doc --workspace handles this; FR-017 covers documentation build verification
- [x] CHK067 Is the proptest regression cache key strategy (hash of test files) consistent with the cold cache behavior requirement? [Conflict, Spec §FR-006] — **PASS**: FR-006 specifies hash of Cargo.lock + test files
- [x] CHK068 Is there ambiguity in "many modules" vs the spec's threshold of 2+ crates? [Ambiguity, Spec §User Story 2] — **PASS**: FR-001 + Appendix A clarify thresholds
- [x] CHK069 Is the PTY allocation mechanism (portable-pty vs script) specified with a decision rationale? [Ambiguity, Spec §FR-008] — **PASS**: Technical Tooling documents primary + fallback with rationale

## Traceability

- [x] CHK070 Is each functional requirement (FR-001 through FR-016) traceable to at least one user story? [Traceability, Spec §Functional Requirements]
- [x] CHK071 Is each success criterion (SC-001 through SC-010) traceable to at least one functional requirement? [Traceability, Spec §Success Criteria]
- [x] CHK072 Are all acceptance scenarios traceable to their parent user stories? [Traceability, Spec §User Scenarios]
- [x] CHK073 Are edge cases traceable to specific requirements or explicitly marked as out of scope? [Traceability, Spec §Edge Cases]

## Notes

- Items marked `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- This checklist focuses on CI pipeline requirements quality, not implementation correctness
- **Last verified against spec.md v1.0 (197 lines) on 2026-09-03**

---

## Reviewer Findings Summary

**Passed**: 71 items | **Gaps**: 2 items (CHK060, CHK062)

### Legitimate Gaps

| Item | Finding | Resolution Path |
|------|---------|-----------------|
| CHK060 | rust-cache migration | Current ci.yml doesn't use Swatinem/rust-cache; Assumptions acknowledge migration needed — implementation detail for plan.md |
| CHK062 | cross tool migration | Current ci.yml doesn't use cross; Assumptions acknowledge migration needed — implementation detail for plan.md |

### Root Cause of Prior False Negatives

The original checklist audit (35 failed items) was performed against an **earlier version of the spec** that lacked:
- Appendix A (Path Filtering Patterns)
- Expanded FR-001 with explicit glob patterns
- Expanded FR-006 with proptest mechanics (directory, format, replay, cache key)
- Expanded FR-008 with PTY dimensions (80x24, 79x23, 240x60)
- Edge Cases section with dimension scenarios and transitive dependency handling
- Assumptions section with baseline (8-12 min) for SC-008 measurement
- Technical Tooling section documenting tool-to-FR mappings

All false negatives have been re-evaluated and marked `[x]` with updated justifications referencing the current spec structure.
