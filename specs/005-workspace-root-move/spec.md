# Feature Specification: Flatten Workspace Into Repository Root

**Feature Branch**: `005-workspace-root-move`

**Created**: 2026-09-02

**Status**: Draft

**Input**: User description: "Help me move /home/luis/development/leiden/leiden to /home/luis/development/leiden"

## Clarifications

### Session 2026-09-02

- Q: How should the CI workflow be handled when the workspace moves to the repository root? → A: Update CI paths in the same restructuring PR (single atomic cutover; CI never runs against a stale path).
- Q: Should the documentation/reference updates ship in the same PR as the pure file move, or in a separate follow-up PR? → A: Two PRs — PR 1 is the pure move (plus the minimal manifest/config path fixes required to function, including CI), PR 2 sweeps all documentation and guidance references.
- Q: Does FR-004's documentation gate include markdown documentation links (e.g. README), which break between the two PRs? → A: No — the gate is the Rust API documentation build (`cargo doc --workspace --no-deps`), which does not validate markdown link targets; README/markdown link validity is restored by the FR-006 follow-up PR.
- Q: Does PR 2 (the documentation sweep) run the same CI pipeline? → A: Yes — it is a normal pull request against the flattened layout and must pass the unchanged CI pipeline from the repository root.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Work From the Repository Root (Priority: P1)

A developer (or AI agent) opens the repository and runs all project commands — build, test, lint, documentation, fixtures — directly from the repository root, without first descending into an inner workspace directory. The nested `leiden/leiden/` duplication disappears: there is exactly one obvious place to work.

**Why this priority**: This is the entire point of the move. Every other benefit (shorter paths in tooling output, fewer agent-navigation mistakes, no confusion between the repo root and the workspace root) follows from it.

**Independent Test**: Can be fully tested by cloning the repository fresh and running the standard build/test/lint commands from the repository root; they must succeed with no directory pre-navigation.

**Acceptance Scenarios**:

1. **Given** a fresh clone of the repository, **When** a developer runs the standard build, test, and lint commands from the repository root, **Then** all succeed with no additional directory navigation.
2. **Given** the restructured repository, **When** a developer lists the repository root, **Then** the workspace manifest, lockfile, toolchain pin, lint and deny configurations, property-test configuration, license-adjacent configs, fixtures directory, and the three crate directories are all visible at the top level alongside the existing `specs/`, `.specify/`, and documentation files.
3. **Given** the restructured repository, **When** a developer looks for a directory named `leiden/` inside the repository root, **Then** no such nested workspace directory exists.

---

### User Story 2 - History and Review Integrity (Priority: P2)

A reviewer examining the restructuring change can follow every moved file's history. The move must be performed in a way that version control recognizes as renames, so `git log --follow`, blame, and the review diff all remain meaningful. The restructuring PR should contain no content edits mixed with the pure moves.

**Why this priority**: Preserving history protects every future investigation ("when did this function change?") and keeps the restructuring reviewable as a single mechanical change.

**Independent Test**: Can be tested by running rename detection on the restructuring commit and confirming that moved source files are reported as renames (≥ 90% similarity), and that the commit contains no unrelated content changes.

**Acceptance Scenarios**:

1. **Given** the restructuring commit, **When** rename detection is applied, **Then** all moved files are recognized as renames rather than delete+add pairs.
2. **Given** the restructuring commit, **When** its diff is reviewed, **Then** it contains only moves/renames and reference-path updates, with no feature or bug-fix edits mixed in.
3. **Given** any pre-move source file, **When** a developer runs history tracing on its post-move path, **Then** the complete pre-move history is visible.

---

### User Story 3 - Tooling and Documentation Point at the New Layout (Priority: P2)

After the move, every reference that names a workspace path — contributor documentation, spec documents, agent guidance files, editor/tool configuration, and any scripted command — points at the new top-level locations. Nothing instructs a newcomer to "cd into leiden/ first".

**Why this priority**: A structurally correct but reference-stale repository actively misleads contributors and automated agents, recreating the confusion the move was meant to eliminate.

**Independent Test**: Can be tested by searching the entire repository for path references containing the old nested prefix and confirming none remain (excluding the merged history and intentionally-archived spec snapshots).

**Acceptance Scenarios**:

1. **Given** the restructured repository, **When** the repository is searched for references to the old nested workspace path, **Then** no live documentation, configuration, or guidance file still points there.
2. **Given** the contributor documentation at the top level, **When** a new contributor follows its setup instructions, **Then** every command works from the repository root as written.
3. **Given** the ignore files, **When** build artifacts are produced at the new root location, **Then** they are excluded from version control exactly as before.

---

### Edge Cases

- **Existing local clones**: a developer with an unmerged local branch based on the old layout must be able to rebase onto the new layout with only path-level conflicts (git handles renames; no manual content merging expected).
- **Untracked build artifacts**: the build cache directory left behind in the old nested location must not be moved with the source; it is disposable and regenerated at the new root.
- **Case/name collisions at the root**: any file at the repository root that would collide with a moved file (same name, different content) must be reconciled explicitly: build, tool, and ignore files are reconciled by the union of both contents (repository-wide root entries preserved) — for this restructuring the only collision is `.gitignore`, where the root copy is already a strict superset of the workspace copy; any reconciliation beyond the union rule is documented in the restructuring PR description.
- **Path length / tooling assumptions**: tools that assume the workspace sits one level below the repository root (scripts, editor workspaces, CI job working directories) must be identified and updated in the same change.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: All workspace content — the workspace manifest, dependency lockfile, toolchain pin, lint and deny configurations, property-test configuration, fixture files, and every crate under `crates/` — MUST reside directly in the repository root after the change.
- **FR-002**: The move MUST be performed with version-control rename operations so file histories remain traceable across the move.
- **FR-003**: The restructuring commit MUST be pure: it MUST contain only moves plus the minimal reference updates required for the repository to function (path references inside manifests, configuration, CI workflows, and fixture/spec-path references in test harnesses), with no unrelated content changes.
- **FR-004**: The repository MUST build, pass its full test suite (including the release-mode gate), pass lint and formatting checks, and generate documentation from the repository root immediately after the move. The documentation gate is the Rust API documentation build (`cargo doc --workspace --no-deps`), which does not validate README or other markdown link targets; markdown link validity is restored by the FR-006 follow-up PR. After the move, the codebase knowledge graph (`graphify-out/`) MUST be regenerated at the new root location per Constitution §VIII.
- **FR-005**: Ignore patterns MUST be updated so generated build artifacts, caches, and knowledge-graph output are excluded at the new locations, and no previously ignored file becomes tracked.
- **FR-006**: All live references to the old nested workspace path — in contributor docs, agent guidance, spec documents, configuration files, and scripts — MUST be updated in a dedicated follow-up PR that lands immediately after the pure-move PR (which itself contains only the minimal functional reference updates permitted by FR-003).
- **FR-007**: The feature specification, planning, and task artifacts of this feature MUST be written against the new layout conventions so subsequent features assume the flattened structure.
- **FR-008**: The nested `leiden/` directory MUST NOT exist in the repository tree after the change (the crate named `leiden` continues to exist, relocated under `crates/leiden/`).
- **FR-009**: The restructuring PR MUST be a single atomically revertible unit — a `git revert` of it MUST restore a functional nested layout. The documentation-sweep PR MUST NOT begin until the restructuring PR is merged and its post-merge validation checkpoint passes: the full verification suite (FR-004) plus the quickstart validation scenarios run from the repository root. A post-merge failure is remediated fix-forward when the cause is a stale path or CI configuration, and by revert when the cause is functional breakage. The restructuring is complete only when the documentation-sweep PR has also landed (SC-003 satisfied).

### Key Entities *(include if feature involves data)*

- **Repository root**: the single top-level directory containing both project governance artifacts (specs, guidance, ignore rules) and the entire workspace (manifest, lockfile, crates, fixtures, tool configuration).
- **Workspace manifest**: the build-orchestration file that must remain discoverable at the root and may require internal path adjustments (member paths, bench/test artifact references).
- **Crate directories**: the three relocated component directories, whose internal contents move unchanged.
- **Path references**: any textual occurrence of the old nested prefix in documentation, configuration, or guidance that must be rewritten to the new layout.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A fresh clone can be built and fully tested from the repository root in one session with zero `cd`-into-`leiden/` steps.
- **SC-002**: 100% of moved source files are recognized as renames by version control in the restructuring commit, each with ≥ 90% content similarity; verifiable with `git show -M --name-status --format= <restructuring-commit>`, which MUST report only `R`-class statuses for moved files and no `A`/`D` pairs among source files.
- **SC-003**: A repository-wide search over tracked files — e.g. `grep -rn` for `working-directory: leiden`, `leiden/leiden`, and stale `../specs` / `../.specify` references, excluding git history, gitignored output directories (e.g. `graphify-out/`), and archived spec snapshots under `specs/001…004` — returns zero live references to the old nested workspace path.
- **SC-004**: The full verification suite (build, tests, lint, format check, documentation) passes at the repository root with the same pass/fail profile as before the move, and the regenerated knowledge graph reflects the new root layout.

## Assumptions

- The move is a pure relocation: no crate is renamed, split, or merged; the crates keep their existing names and internal structure.
- The existing build cache in the old nested location is disposable and will be regenerated at the root, not migrated.
- Documentation-only files already at the repository root (design system, rigor guides) stay where they are; only the workspace's own README and configuration move up.
- CI job definitions that pin a workspace working directory MUST be updated in the same restructuring PR as the move (atomic cutover), so CI never executes against a stale path.
- Historical spec documents under `specs/` that reference old paths are archived records and are updated only where they would actively mislead (e.g., active guidance), not rewritten wholesale.
