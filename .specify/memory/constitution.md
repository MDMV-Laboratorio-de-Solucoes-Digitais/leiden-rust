<!--
Sync Impact Report
- Version change: 1.0.2 → 1.1.0 (MINOR per amendment procedure: added Principle VIII and development workflow guidance for Knowledge Graph /graphify querying)
- Modified principles: none
- Added sections: Core Principle VIII (Knowledge Graph Context & Query-Driven Architecture)
- Removed sections: none
- Follow-up TODOs: none
-->

# Leiden Algorithm Constitution

This constitution governs the implementation of the Leiden Algorithm for community
detection on graphs in the Rust programming language. It is the single source of truth
for engineering principles and is enforced by the project's lint configuration in
`Cargo.toml`, by code review, and by the Spec Kit workflow that surrounds this file.

The detailed lint profile and methodology that operationalize the principles below
live in:

- `guide-to-strict-rust.md` — definitive guide to pedantic Rust lint configuration,
  panic-free error handling, visibility discipline, documentation requirements,
  exception justification, observability, unused-result handling, dependency rigor,
  and the micro-verification / atomic-commit methodology.
- `rust-code-rigor.md` — canonical `[workspace.lints.rust]` and
  `[workspace.lints.clippy]` configuration block that MUST be copied verbatim into
  the project's `Cargo.toml` `[workspace.lints]` table.

## Core Principles

### I. Library-First & Domain Modeling

Every component of the Leiden implementation MUST be designed as a self-contained,
independently testable library crate with a single, clear purpose. The algorithm is
decomposed along its natural seams (graph representation, local moving, refinement,
aggregation, partitioning, orchestration) before any code is written.

- Domain types (graphs, partitions, node/edge weights, quality functions) MUST use
  owned data (`String`, `Vec<T>`, typed ids) when crossing module or crate
  boundaries. Borrowed views are allowed only where lifetimes are mathematically
  required for performance and MUST be encapsulated behind a clean facade.
- Traits (`Graph`, `WeightModel`, `QualityFunction`, `PartitionStrategy`) define
  behavior; concrete structs implement them. Domain logic MUST NOT carry multi-trait
  generic bounds (`T: A + B + C`); it MUST depend on concrete implementations.
- No "organizational-only" crates. Every crate MUST have a real consumer or a
  documented, justified standalone purpose.

**Rationale:** Leiden is a multi-stage algorithm (local moving, refinement,
aggregation, projection). Strict separation by library boundary produces testable,
composable units and keeps the borrow checker from leaking across stages.

### II. Strict Lint Compliance (NON-NEGOTIABLE)

The `[workspace.lints.rust]` and `[workspace.lints.clippy]` blocks defined in
`rust-code-rigor.md` MUST be applied verbatim to the project's `Cargo.toml` and
MUST NOT be weakened without amending this constitution.

- Native `rustc` lints set to `deny`: `unsafe_code`, `missing_docs`,
  `missing_debug_implementations`, `unreachable_pub`, `unused_results`,
  `unused_qualifications`, `trivial_casts`, `trivial_numeric_casts`,
  `unused_extern_crates`.
- Clippy main groups: `all = deny`, `pedantic = deny`, `nursery = warn`.
- `allow_attributes = deny` and `allow_attributes_without_reason = deny` —
  exceptions require `#[expect(...)]` with an explicit `reason = "..."`.
- Panic/tech-debt prevention at `deny`: `unwrap_used`, `expect_used`, `panic`,
  `todo`, `unimplemented`, `unreachable`, `dbg_macro`. `print_stdout = warn`.
- Hygiene at `deny`: `fallible_impl_from`, `clone_on_ref_ptr`, `use_self`,
  `wildcard_dependencies`, `multiple_crate_versions`. `cargo = warn`.

**Rationale:** A strict, workspace-wide lint profile cannot be bypassed file by
file and produces memory-safe, panic-free, fully-typed code by construction.

### III. Panic-Free Error Propagation

`unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`,
and `dbg!()` are forbidden in non-test production code. Every fallible operation
MUST surface as data.

- Libraries MUST define domain error enums using `thiserror` with `#[derive(Debug,
  Error)]` and `#[error(...)]` on every variant. Fallible conversions between
  domain types MUST use `TryFrom`, never `From`.
- Failures cross module boundaries with the `?` operator. `Option<T>` branches are
  handled with `let ... else { return Err(...) }` or `match`, never with
  `.unwrap()`.
- `Result`-returning calls MUST be either handled, propagated, or explicitly
  discarded with `let _ = result;`. Implicit discards are denied by
  `unused_results = deny`.
- Tests MAY use `unwrap`/`expect` internally; this exception is local to test
  modules and MUST NOT leak into library or binary crates.

**Rationale:** Leiden operates on graphs that may be empty, disconnected, or
contain self-loops. Domain modeling of every failure path is the only way to keep
the algorithm correct on adversarial input.

### IV. Documentation & Visibility Discipline

Every public item (struct, enum, trait, function, constant, type alias) MUST carry
a `///` doc comment that explains its purpose, contracts, and error conditions.
Every public struct and enum MUST `#[derive(Debug)]`.

- `pub` is reserved for items that are actually exported from the crate root.
  Internal items MUST use `pub(crate)` or `pub(super)` to keep
  `unreachable_pub = deny` clean.
- Doc comments MUST cover: what the item does, when it returns an error (for
  fallible functions), and any non-obvious panics, allocations, or asymptotic
  guarantees. Missing docs are a build failure, not a warning.
- `clone_on_ref_ptr = deny` forces `Arc::clone(&x)` over `x.clone()`; `use_self =
  deny` forces `Self` in impls so refactors stay mechanical.

**Rationale:** Leiden's algorithm description (Traag et al., 2019) has subtle
invariants — community connectedness, refinement monotonicity, the `γ` resolution
parameter. The only way those invariants survive review is to encode them in
type signatures and docs at the API boundary.

### V. Test-First (NON-NEGOTIABLE)

TDD is mandatory: failing tests are written first, reviewed, observed to fail,
and only then is the implementation written to make them pass. The red-green-
refactor cycle is enforced per logical component.

- Every public function or trait method MUST have at least one unit test covering
  the happy path and one covering the primary error path before its
  implementation lands.
- Integration tests MUST exist for: graph parsing/construction, the local-moving
  phase on a known small graph, the refinement phase, the aggregation step, and
  the end-to-end Leiden run on at least one fixture from a published reference
  implementation.
- Property-based tests (via `proptest` or equivalent) MUST cover invariants:
  non-decreasing modularity under local moving, well-defined communities after
  refinement, and partition refinement being a refinement of the partition
  passed in.
- Benchmarks (`criterion` or equivalent) MUST exist for the inner loops
  (neighborhood scan, move computation, refinement merge) so regressions in
  asymptotic behavior are caught.

**Rationale:** Leiden has known edge cases (empty graphs, single-node graphs,
perfectly modular graphs). Test-first is the only discipline that surfaces them
before they ship.

### VI. Observability & I/O Discipline

The `tracing` ecosystem is the only sanctioned logging mechanism. `println!`,
`print!`, and `eprintln!` are restricted to `print_stdout = warn`; `dbg!` is
denied.

- Library code MUST use `tracing::{trace, debug, info, warn, error}` with
  structured fields (`info!(node_id = id, "moved node")`). No stringly-typed
  log lines without context.
- The CLI binary MUST emit human-readable progress to stderr and machine-readable
  output (the resulting partition) to stdout. Errors MUST be returned via
  `Result` and translated to exit codes at the binary boundary; no panic-on-
  error shortcuts.
- Public APIs MUST expose the minimum surface needed to integrate them; debug
  hooks (`Debug` impls, `tracing` spans) MUST NOT leak into the type signature.

**Rationale:** Leiden runs can be long on large graphs. Operators need
structured logs to diagnose convergence, not a wall of `println!` noise.

### VII. Dependency & Build Rigor

`Cargo.toml` MUST be maintained to the same standard as source code.

- Wildcard dependencies (`crate = "*"`) are denied. Every dependency MUST pin a
  semver range; `cargo add` MUST be used instead of hand-editing where possible.
- `multiple_crate_versions = deny` is enforced. Transitive duplicates are resolved
  via `cargo update`, and only when a transitive requirement is genuinely
  incompatible are duplicates allowed via a documented `clippy.toml`
  `allowed-duplicate-crates` entry.
- The `[workspace.lints]` block is the single source of lint truth. Crate-level
  overrides MUST NOT relax lints; they may only tighten them, and only with a
  comment explaining the local reason.
- Builds use the micro-verification loop: implement → `cargo check --workspace`
  → atomic conventional commit (e.g. `feat(leiden-local-moving): implement
  greedy move selection`). No batch commits across components.

**Rationale:** A library that can be silently downgraded or duplicated is not a
library. Pinning, dedup, and atomic history keep the project auditable.

### VIII. Knowledge Graph Context & Query-Driven Architecture

Maintainers and AI coding agents MUST leverage knowledge graph queries (via `/graphify`
or `graphify query <topic>`) for structural discovery, community mapping, and dependency
analysis before modifying or creating specifications, architectural designs, and implementations.

- Knowledge graph queries MUST be used to inspect symbol relationships, module cohesion,
  and cross-crate dependencies prior to proposing major refactors or new components.
- During Spec Kit workflows (`/speckit-specify`, `/speckit-plan`, `/speckit-tasks`, `/speckit-implement`),
  contextual queries against the graph MUST guide architectural decisions to avoid duplication,
  respect component boundaries, and preserve domain invariants.
- The codebase knowledge graph (e.g. `graphify-out/graph.json` and `graphify-out/GRAPH_REPORT.md`)
  MUST be refreshed when significant structural or cross-crate API boundaries are modified.

**Rationale:** Leiden is a multi-crate workspace spanning core algorithms, CLI, and TUI
components. Querying the knowledge graph via `/graphify` provides deterministic structural
context, eliminates blind spot assumptions, and guarantees high-fidelity, cohesive results across
the development lifecycle.

## Additional Constraints

- **Language & edition:** Rust stable, edition 2024. `rust-toolchain.toml` pins
  the toolchain. **MSRV floor: 1.88.0**, imposed by the transitive dependency
  `ratatui = "0.30.2"` declared in `tasks.md T006`; the floor is documented
  in `README.md` (T009) and may be raised via a PATCH amendment to this
  constitution when changed. Lowering the MSRV below 1.88.0 requires
  substituting the ratatui version whose `rust_version` is compatible, with
  the change documented in `tasks.md T006` and approved via a Constitution
  PATCH PR.
- **Domain accuracy:** Implementation MUST follow the Leiden algorithm as
  published (Traag, Waltman, van Eck, 2019). Any deviation MUST be documented
  inline with a citation and a rationale; silent deviations are forbidden.
- **Determinism:** Ties in the local-moving phase MUST be broken deterministically
  (e.g. by node id). Stochastic Leiden variants are out of scope unless added
  explicitly via a feature flag and a separate principle amendment.
- **Numerical stability:** Modularity / quality computations MUST use `f64` and
  MUST guard against division by zero and `NaN` propagation. Property tests MUST
  assert no `NaN` in output quality values.
- **Unsafe code:** `unsafe_code = deny` applies to all production code, including
  performance-critical inner loops. Any future `unsafe` block requires a MAJOR
  amendment to this constitution plus a `// SAFETY:` comment with proof of
  invariants.

## Development Workflow

- **Spec Kit flow.** Every non-trivial change starts with `$speckit-specify`,
  proceeds to `$speckit-plan`, `$speckit-tasks`, and `$speckit-implement`. PRs
  that skip the spec are rejected.
- **Knowledge Graph exploration & querying.** Maintainers and autonomous agents MUST utilize
  `/graphify` queries (e.g., `graphify query <topic>` or slash commands) to inspect the project
  knowledge graph, community structure, callflows, and architectural cross-references before and
  during specification, planning, refactoring, and code review to ensure high contextual
  awareness, traceability, and improved results.
- **TDD gate.** No implementation commit lands without its corresponding failing-
  test commit immediately preceding it (or co-located in the same atomic commit).
- **CI pipeline.** CI MUST run, at minimum: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `ct --workspace`, `cargo doc --workspace --no-deps` (which fails on
  `missing_docs = deny`), and `cargo deny check` for advisories and licenses.
- **Platform-conditional tests.** Tests gated by `#[cfg(unix)]` (e.g., POSIX `chmod 000` for permission-denied per `tasks.md` T083a) MUST have a documented no-op counterpart under `#[cfg(not(unix))]` so the platform asymmetry is explicit in the test suite. Suppressing a test on a platform is acceptable only when an equivalent assertion is impossible.
- **`--release` test gate.** Performance contracts (e.g., SC-001's ≤5 s budget on
  a 100-node/500-edge fixture) MUST be exercised under
  `ct --workspace --release` so that `#[cfg(not(debug_assertions))]`-gated
  perf tests actually execute; the debug build's `ct --workspace` remains
  the correctness gate.
- **Review.** Every PR MUST be reviewed by at least one other maintainer. The
  reviewer MUST verify constitution compliance, not just correctness.
- **Atomic commits.** One logical change per commit. Conventional Commit prefixes
  (`feat`, `fix`, `refactor`, `docs`, `test`, `chore`) are required; the scope
  names the affected crate or module.

## Governance

- **Supremacy.** This constitution supersedes all other practices, READMEs,
  inline docs, and verbal conventions. Where it conflicts with anything else,
  this document wins until amended.
- **Amendment procedure.** Proposed amendments are submitted as PRs titled
  `docs: amend constitution to vX.Y.Z`. The PR MUST include: the proposed change,
  the rationale, the version bump type (MAJOR / MINOR / PATCH) with justification,
  and a migration plan if the change is MAJOR or MINOR. Amendments require
  approval from a project maintainer and a 24-hour comment window.
- **Versioning policy.** `MAJOR.MINOR.PATCH`:
  - **MAJOR** — backward-incompatible governance or principle removals or
    redefinitions (e.g. lifting `unsafe_code = deny`, dropping TDD as
    non-negotiable, weakening visibility rules).
  - **MINOR** — new principle or section added, or materially expanded guidance
    on an existing principle.
  - **PATCH** — clarifications, wording fixes, typo fixes, non-semantic
    refinements.
- **Compliance review.** Every PR review MUST include a one-line
  "Constitution compliance" note confirming the lint profile is unchanged,
  documentation is present, tests exist, and no panic-prone patterns were
  introduced. PRs without this note are returned to the author.
- **Guidance files.** Runtime development guidance — the exact `Cargo.toml`
  lint block, the panic-free error patterns, the visibility examples, the
  observability patterns, and the micro-verification methodology — lives in
  `rust-code-rigor.md` and `guide-to-strict-rust.md`. Those files are
  operational companions to this constitution; amendments to them are PATCH-level
  unless they materially change a principle, in which case this constitution is
  amended in the same PR.

**Version**: 1.1.0 | **Ratified**: 2026-08-30 | **Last Amended**: 2026-09-01

**Amendment 2026-09-01 (MINOR 1.0.2 → 1.1.0)**: Added Principle VIII (Knowledge Graph
Context & Query-Driven Architecture) and updated Development Workflow to mandate
the use of `/graphify` queries for Knowledge Graph exploration, ensuring enhanced
architectural context, consistency, and improved results across the Spec Kit workflow.

**Amendment 2026-08-31 (PATCH 1.0.1 → 1.0.2)**: Ratified MSRV floor of 1.88.0
in Additional Constraints "Language & edition", aligning the constitution
with `plan.md §Constraints (MSRV floor CRITICAL)` and `tasks.md T001/T009`
where the ratatui 0.30.2 `rust_version = "1.88.0"` requirement was already
mandated. No semantic principle change; purely a clarifying ratification.
