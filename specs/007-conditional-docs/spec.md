# Feature Specification: Conditional Documentation Generation

**Feature Branch**: `007-conditional-docs`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Include a step to generate docs for the whole workspace if many modules were changed (`cargo doc --workspace`) or only the altered module(s) (`cargo doc --modulename`)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Workspace-Wide Doc Generation (Priority: P1)

As a developer, I want the CI pipeline to generate documentation for the entire workspace when many modules or workspace configuration changes so that comprehensive API docs are always up-to-date after significant modifications.

**Why this priority**: This ensures documentation completeness when broad changes occur—without it, large refactors or multi-crate updates could leave docs stale.

**Independent Test**: Can be fully tested by making changes across multiple crates and observing that `cargo doc --workspace` is triggered, producing docs for all crates.

**Acceptance Scenarios**:

1. **Given** changes span two or more crates or touch `Cargo.toml`/`Cargo.lock`, **When** the doc generation step runs, **Then** `cargo doc --workspace --no-deps` executes and produces documentation for every crate in the workspace
2. **Given** workspace-wide doc generation runs, **When** the step completes, **Then** the build fails if any crate has missing documentation (`missing_docs = deny`)

---

### User Story 2 - Targeted Doc Generation for Single-Crate Changes (Priority: P1)

As a developer, I want the CI pipeline to generate documentation only for the changed crate(s) when modifications are isolated to a single module so that CI time is minimized for small, focused changes.

**Why this priority**: This optimizes feedback loop speed—a single-line fix in `leiden-cli` shouldn't trigger documentation builds for `leiden` and `leiden-tui`.

**Independent Test**: Can be fully tested by making a change only in `crates/leiden-tui/` and observing that `cargo doc -p leiden-tui --no-deps` runs while docs for other crates are skipped.

**Acceptance Scenarios**:

1. **Given** changes are isolated to a single crate (e.g., only `crates/leiden/` files modified), **When** the doc generation step runs, **Then** only `cargo doc -p leiden --no-deps` executes
2. **Given** changes are isolated to `crates/leiden-cli/`, **When** the doc generation step runs, **Then** only `cargo doc -p leiden-cli --no-deps` executes

---

### User Story 3 - Documentation Failure Detection (Priority: P2)

As a maintainer, I want the CI pipeline to fail when documentation has errors (broken links, missing docs, compile failures in doc examples) so that published documentation quality is preserved.

**Why this priority**: Broken documentation erodes trust and signals neglected code. Catching doc errors in CI prevents them from reaching published artifacts.

**Independent Test**: Can be fully tested by introducing a broken doc link or missing doc comment and observing the CI pipeline fail with a clear error.

**Acceptance Scenarios**:

1. **Given** a public item lacks a doc comment, **When** `cargo doc` runs, **Then** the build fails with `missing_docs` error
2. **Given** a doc example contains a compilation error, **When** `cargo doc` runs, **Then** the build fails with the specific doctest failure
3. **Given** a doc comment contains a broken intra-doc link, **When** `cargo doc` runs, **Then** the build fails with a broken link warning (if `broken_intra_doc_links = deny` is set)

---

### Edge Cases

- What happens when a change touches only non-code files (e.g., README, fixtures)? → Doc generation is skipped entirely
- What happens when a change touches a crate that has no public API (internal-only crate)? → Doc generation still runs but produces minimal output
- What happens when workspace configuration changes but no source files change? → Workspace-wide doc generation runs
- What happens when changes span exactly two crates? → Workspace-wide doc generation runs (threshold is ≥2 crates)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST determine the scope of documentation generation based on which crates were modified in the push/PR
- **FR-002**: System MUST run `cargo doc --workspace --no-deps` when changes span two or more crates or touch workspace configuration files (`Cargo.toml`, `Cargo.lock`)
- **FR-003**: System MUST run `cargo doc -p <crate> --no-deps` for only the changed crate when modifications are isolated to a single crate
- **FR-004**: System MUST skip documentation generation entirely when only non-code files change (e.g., markdown, fixtures, CI config)
- **FR-005**: System MUST fail the build if documentation generation produces errors (missing docs, broken links, doctest failures)
- **FR-006**: System MUST run documentation generation with the same lint strictness as the build (`RUSTFLAGS: "-D warnings"`)

### Key Entities

- **Doc Generation Scope**: The determination of whether to build workspace-wide or crate-targeted documentation based on changed paths
- **Changed Crate Set**: The set of crates whose source files were modified in a given push/PR
- **Documentation Build**: The CI step that invokes `cargo doc` with appropriate scope flags

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Documentation generation completes within 5 minutes for single-crate changes
- **SC-002**: Documentation generation completes within 10 minutes for workspace-wide changes
- **SC-003**: CI pipeline skips doc generation entirely for non-code-only changes (zero doc build time)
- **SC-004**: Missing documentation on public items is caught and fails the build within the doc generation step
- **SC-005**: Single-crate changes do not trigger documentation builds for unrelated crates (at least 60% reduction in doc build time for isolated changes)

## Assumptions

- The existing path-filtering mechanism (dorny/paths-filter) from the CI pipeline can be reused to determine which crates changed
- `cargo doc --no-deps` is sufficient (documentation of dependencies is not required since they are external)
- The workspace constitution's `missing_docs = deny` lint will cause `cargo doc` to fail on undocumented public items
- Non-code files are defined as: `*.md`, `*.txt`, `*.json` (fixtures), `*.yaml`, `*.yml` (CI config), and files in `.github/`
- The threshold for "many modules" is 2 or more crates changed simultaneously
