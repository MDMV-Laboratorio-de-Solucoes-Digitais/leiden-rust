# Quickstart: Leiden Algorithm in Rust

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30
**Output of**: `$speckit-plan` Phase 1 (Design & Contracts)

This document is a **validation guide**, not an implementation tutorial. It
describes the runnable scenarios that prove the feature works end-to-end and
maps each to the spec's user stories and success criteria.

For implementation details, see `data-model.md`, `contracts/`, and (later)
`tasks.md`. For the strict lint profile and coding patterns, see
`rust-code-rigor.md` and `guide-to-strict-rust.md` at the repo root.

---

## 1. Prerequisites

- Rust stable toolchain pinned via `rust-toolchain.toml` at the workspace root.
- `cargo-deny` (CI advisory gate).
- `cargo-insta` for TUI snapshot review (`cargo install cargo-insta`).
- `jq` for shell pipeline examples (optional).

```sh
# Verify toolchain
cargo --version   # stable, edition 2024

# Build everything (lints + docs + tests)
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

All four commands MUST succeed before merging. They enforce:
- `cargo check` — workspace compiles.
- `cargo clippy … -D warnings` — strict lint profile is clean (Constitution §II).
- `cargo test` — unit, integration, and property tests pass (Constitution §V).
- `cargo doc … --no-deps` — every public item has `///` docs
  (`missing_docs = deny`).

---

## 2. Fixture Suite

Curated reference graphs live in `fixtures/`:

| File | Nodes | Edges | Purpose |
|---|---|---|---|
| `single_node.edg` | 1 | 0 | US-1 acceptance scenario 2. |
| `empty.edg` | 0 | 0 | US-1 acceptance scenario 3. |
| `two_cliques.edg` | 9 | 17 (16 intra-clique + 1 bridge) | US-1 acceptance scenario 1; SC-002 reference. |
| `karate.edg` | 34 | 78 | SC-002 reference (Zachary's karate club). |
| `ring_of_cliques.edg` | 12 | 18 | SC-002 reference; tests aggregation. |
| `star.edg` | 11 | 10 | SC-002 reference; tests degenerate γ. |
| `path.edg` | 10 | 9 | SC-002 reference; tests disconnected components. |
| `lfr_small.edg` | 100 | 500 | SC-001 performance budget. |

Each fixture ships with a `*.expected.json` documenting the reference partition
under γ=1.0 and seed=0. A property test loads the fixture, runs the algorithm,
and asserts the returned partition matches the expected one within tie-breaking
tolerance (lowest-node-id wins).

---

## 3. User Story 1 — Detect Communities (P1)

**Scenario**: Two cliques connected by a single bridge.

```sh
$ leiden --gamma 1.0 --seed 0 --format text fixtures/two_cliques.edg
0	0
1	0
2	0
3	0
4	0
5	1
6	1
7	1
8	1
```

**Validation**:

```sh
# Acceptance scenario 1: two distinct communities, both internally connected
cargo test -p leiden two_cliques_yields_two_communities

# Acceptance scenario 2: single-node graph returns one community, no error
cargo test -p leiden single_node_returns_one_community

# Acceptance scenario 3: empty graph returns empty partition, typed "empty graph" indicator
cargo test -p leiden empty_graph_returns_empty_partition
```

**SC-002 cross-check**: the curated fixture suite achieves ≥ 90% match on
graphs with known unique optima.

```sh
cargo test -p leiden fixture_suite_matches_reference
```

---

## 4. User Story 2 — Tune Resolution and Reproducibility (P2)

**Scenario**: Resolution sensitivity on the karate graph.

```sh
$ leiden --gamma 0.5 --seed 0 --format json fixtures/karate.edg | jq '.quality, .iterations'
0.4012
4

$ leiden --gamma 2.0 --seed 0 --format json fixtures/karate.edg | jq '.quality, .iterations'
0.2455
3
```

**Validation**:

```sh
# Same inputs → byte-identical output (FR-004)
cargo test -p leiden determinism_under_fixed_seed

# Different γ → different partition on non-degenerate fixtures
cargo test -p leiden resolution_changes_partition
```

---

## 5. User Story 3 — Library & CLI Integration (P3)

### 5.1 Library API

```rust
use leiden::{CsrGraph, Edge, Leiden, LeidenParameters, NodeId};

let edges: Vec<Edge<String>> = vec![
    Edge { source: "a".into(), target: "b".into(), weight: 1.0 },
    Edge { source: "b".into(), target: "c".into(), weight: 1.0 },
    Edge { source: "d".into(), target: "e".into(), weight: 1.0 },
];

let graph = CsrGraph::from_edges(edges)?;
let result = Leiden::new()
    .with_parameters(LeidenParameters::default())
    .run(&graph)?;

assert!(result.quality.is_finite());
assert!(result.iterations <= 10);
```

**Validation**:

```sh
cargo test -p leiden library_api_smoke
cargo doc --workspace --no-deps
```

### 5.2 CLI Round-trip

```sh
# Write partition to stdout, parse, assert validity
leiden --format json fixtures/two_cliques.edg > /tmp/out.json
jq -e '.assignments | length == 9' /tmp/out.json
jq -e '.assignments | map(.node) | unique | length == 9' /tmp/out.json
```

**Validation**:

```sh
cargo test -p leiden-cli cli_round_trip
cargo test -p leiden-cli cli_text_format_is_sorted
cargo test -p leiden-cli cli_rejects_unknown_format
```

### 5.3 Malformed Input (FR-008)

```sh
$ cat > /tmp/bad.edg <<EOF
a	b	-1.0
EOF
$ leiden /tmp/bad.edg
/tmp/bad.edg:1: invalid weight `-1.0`: must be finite and ≥ 0
$ echo $?
4
```

**Validation**:

```sh
cargo test -p leiden-cli malformed_negative_weight
cargo test -p leiden-cli malformed_self_loop
cargo test -p leiden-cli malformed_dangling_node
cargo test -p leiden-cli malformed_invalid_gamma
```

---

## 6. TUI Smoke (interactive binary)

The TUI is exercised via `TestBackend` snapshot tests in CI; an interactive
session looks like:

```sh
$ leiden-tui fixtures/karate.edg
```

Expected panels:
- Community list (left), sorted by size descending.
- Graph view (centre), BFS-laid-out, communities coloured.
- Log pane (right, toggled with `l`).
- Status bar (bottom): `iter=3 quality=0.4012 γ=1.0 seed=0`.

**Validation**:

```sh
cargo test -p leiden-tui
cargo insta review   # after intentional UI changes
```

---

## 7. Performance Budget (SC-001)

```sh
$ cargo bench -p leiden --bench local_moving -- fixtures/lfr_small.edg
```

The benchmark asserts (and fails the CI budget on regression) that the
100-node / 500-edge fixture completes local moving in < 2 s on a single
modern CPU thread. End-to-end (3 phases × ≤ 10 iterations) is < 5 s as per
SC-001.

```sh
cargo bench -p leiden
```

---

## 8. Property Tests (Constitution §V)

```sh
cargo test -p leiden proptest
```

Asserts:
- Modularity never decreases across a local-moving pass.
- Refinement output is a refinement of the input partition.
- Quality output is finite (no NaN, no ±∞) across 1000 random weighted graphs
  with `nodes ∈ [10, 1000]`, `edges ∈ [20, 10_000]` (SC-003).
- Every returned community is internally connected.

---

## 9. Compliance One-liner

Every PR review MUST include a one-line "Constitution compliance" note:

> Constitution: lints unchanged; public items documented; TDD test commit
> precedes implementation commit; no panic-prone patterns introduced.

PRs without this note are returned to the author (Constitution §Compliance
review).

---

## 10. What is NOT in this quickstart

This document deliberately omits:

- Full implementation code for any phase. See `tasks.md` (Phase 2) for the
  ordered component breakdown.
- Inner-loop micro-benchmarks. Those live in `crates/leiden/benches/` and are
  covered by the CI budget.
- TUI widget styling. Snapshot tests are the source of truth.
- CI pipeline configuration. See the constitution §Development Workflow /
  CI pipeline for the required checks.