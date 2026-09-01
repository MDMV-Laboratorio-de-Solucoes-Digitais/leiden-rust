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

## Library Usage

Add `leiden` to your `Cargo.toml`:

```toml
[dependencies]
leiden = "0.1.0"
```

### Basic Example

```rust
use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build an undirected weighted graph
    let edges = vec![
        Edge { source: "node1".to_string(), target: "node2".to_string(), weight: 1.0 },
        Edge { source: "node2".to_string(), target: "node3".to_string(), weight: 1.0 },
        Edge { source: "node3".to_string(), target: "node1".to_string(), weight: 1.0 },
        Edge { source: "node3".to_string(), target: "node4".to_string(), weight: 0.1 },
        Edge { source: "node4".to_string(), target: "node5".to_string(), weight: 1.0 },
    ];
    let graph = CsrGraph::from_edges(edges)?;

    // 2. Configure parameters
    let params = LeidenParameters {
        gamma: 1.0,
        seed: Some(42),
        iteration_cap: 10,
    };

    // 3. Run algorithm
    let result = Leiden::new()
        .with_parameters(params)
        .run(&graph)?;

    println!("Detected {} nodes across communities", result.partition.len());
    println!("Modularity Quality: {:.4}", result.quality);
    println!("Completed in {} iterations ({:?})", result.iterations, result.termination_reason);

    for (node, comm) in result.partition {
        println!("Node {node} -> Community {comm}");
    }

    Ok(())
}
```

## CLI Usage (`leiden`)

The non-interactive CLI binary accepts edge-list files (`.edg`, `.tsv`, `.csv`) or JSON adjacency input:

```sh
# Run on edge-list fixture and output JSON format
cargo run --release -p leiden-cli -- fixtures/two_cliques.edg --format json

# Run with custom resolution gamma and text format (default)
cargo run --release -p leiden-cli -- fixtures/karate.edg --gamma 1.2 --format text

# Read from stdin with JSON output redirected to file
cat fixtures/lfr_small.edg | cargo run --release -p leiden-cli -- --format json > partition.json
```

### CLI Arguments

- `<INPUT_FILE>`: Path to input edge-list or JSON graph file (reads from stdin if omitted).
- `--format <text|json>`: Output format (`text`: `<node>\t<comm>`, `json`: full partition output metadata).
- `--gamma <F>`: Resolution parameter $\gamma \ge 0$ (default: `1.0`).
- `--seed <U>`: Randomness seed for determinism (default: `0`).
- `--iteration-cap <N>`: Maximum Leiden outer-loop iterations (default: `10`).
- `--quiet`: Suppress stderr structured progress logs.
- `--log-level <LVL>`: Tracing log filter (`trace`, `debug`, `info`, `warn`, `error`).

## Interactive TUI Usage (`leiden-tui`)

An interactive terminal user interface built with Ratatui to visualize graph partitions, inspect community statistics, and monitor execution logs:

```sh
cargo run --release -p leiden-tui -- fixtures/karate.edg
```

### Key Bindings

| Key | Action |
|---|---|
| `q` / `Ctrl+C` | Quit application |
| `r` | Restart algorithm execution |
| `p` | Pause / resume automatic iteration |
| `g` | Toggle graph visualization panel |
| `l` | Toggle tracing log pane |
| `Tab` | Cycle keyboard focus across panels |
| `↑` / `↓` | Select and inspect community in table |
| `?` | Toggle help modal overlay |

## Development & Verification

### Minimum Supported Rust Version (MSRV)

**MSRV: `1.88.0`.** Pinned via [`rust-toolchain.toml`](rust-toolchain.toml).

### Full Verification Suite

```sh
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
ct --workspace
ct --workspace --release
cargo doc --workspace --no-deps
cargo deny --config deny.toml check
```

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT) at your option.
