# Leiden Algorithm in Rust

A panic-free, fully-documented Rust implementation of the **Leiden algorithm**
for community detection on graphs (Traag, Waltman, van Eck, 2019), structured
as a Cargo workspace with one library crate and two binaries:

| Crate | Purpose |
|---|---|
| [`leiden`](crates/leiden) | Library crate: the algorithm. Pure Rust, no I/O, no TUI deps. |
| [`leiden-cli`](crates/leiden-cli) | Non-interactive `leiden` binary: parse → run → serialize. |
| [`leiden-tui`](crates/leiden-tui) | Interactive `leiden-tui` binary (Ratatui). |

## Citation

This implementation follows the published Leiden algorithm:

> Traag, V. A., Waltman, L., & van Eck, N. J. (2019). **From Louvain to Leiden:
> guaranteeing well-connected communities.** *Scientific Reports*, 9, 5233.
> <https://doi.org/10.1038/s41598-019-41695-z>

Every algorithm seam (graph representation, local moving, refinement,
aggregation, orchestration) cites the corresponding paper section inline;
see [`tasks.md`](../specs/001-leiden-algorithm/tasks.md) (Phase 3, T043–T046)
and the FR-009 / T138a audit for the citation discipline.

## Scope

v1 implements a deterministic variant of Leiden (no stochastic refinement),
operating on graphs of ≤ ~100 000 nodes / ≤ ~1 000 000 edges in-memory on a
single CPU thread. Both the algorithm and the strict lint profile are
governed by the project constitution at
[`.specify/memory/constitution.md`](../.specify/memory/constitution.md).

## Quickstart (for users)

```sh
# Non-interactive: detect communities on a fixture and emit JSON to stdout
cargo run --release -p leiden-cli -- fixtures/two_cliques.edg | jq '.quality'

# Interactive: same algorithm, inspect the partition in a TUI
cargo run --release -p leiden-tui -- fixtures/karate.edg
```

See [`specs/001-leiden-algorithm/quickstart.md`](../specs/001-leiden-algorithm/quickstart.md)
for the full validation guide (US-1 acceptance scenarios, determinism,
SC-001 performance budget, proptest invariants).

## Development

### Minimum Supported Rust Version (MSRV)

**MSRV: `1.88.0`.** This floor is imposed by the transitive dependency
`ratatui = "0.30.2"` (declared in `crates/leiden-tui/Cargo.toml`), which
declares `rust_version = "1.88.0"` on crates.io. The toolchain pin lives
in [`rust-toolchain.toml`](rust-toolchain.toml); the active stable channel
satisfies this floor without manual `rustup` intervention.

Per Constitution Additional Constraints ("Language & edition"), any MSRV
bump above this floor requires a **PATCH amendment** to
`.specify/memory/constitution.md`. Lowering the MSRV below 1.88.0 requires
substituting the ratatui version whose `rust_version` is compatible, with
the change documented in `tasks.md` (T006) and approved via a Constitution
PATCH PR.

### Toolchain

The project uses the stable Rust toolchain pinned via `rust-toolchain.toml`,
with `rustfmt` and `clippy` components. No nightly features are required.

### Lint profile

The workspace enforces a strict `[workspace.lints]` profile copied verbatim
from [`../rust-code-rigor.md`](../rust-code-rigor.md). Concretely:

- All native `rustc` lints set to `deny`: `unsafe_code`, `missing_docs`,
  `missing_debug_implementations`, `unreachable_pub`, `unused_results`,
  `unused_qualifications`, `trivial_casts`, `trivial_numeric_casts`,
  `unused_extern_crates`.
- Clippy `all` and `pedantic` at `deny`; `nursery` at `warn`.
- Panic-free enforcement: `unwrap_used`, `expect_used`, `panic`, `todo`,
  `unimplemented`, `unreachable`, `dbg_macro` at `deny`.
- Hygiene: `wildcard_dependencies`, `multiple_crate_versions`, `use_self`,
  `clone_on_ref_ptr` at `deny`.
- `print_stdout` at `warn` (use `tracing` instead).

### Verifying a change

The TDD loop is enforced per logical component:

```sh
cargo check --workspace                 # must compile
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --workspace
```

All four commands MUST succeed before merging. The Constitution's
"Compliance One-liner" (`quickstart.md §9`) is required in every PR body.

### Layout

```text
leiden/
├── Cargo.toml                   # [workspace] + [workspace.lints]
├── rust-toolchain.toml          # Pin toolchain (channel = "stable")
├── clippy.toml                  # allowed-duplicate-crates (Constitution §VII)
├── README.md                    # This file
├── crates/
│   ├── leiden/                  # Library crate
│   ├── leiden-cli/              # Non-interactive CLI
│   └── leiden-tui/              # Interactive TUI (Ratatui)
└── fixtures/                    # Curated reference graphs (see quickstart.md §2)
```

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT) at your option.