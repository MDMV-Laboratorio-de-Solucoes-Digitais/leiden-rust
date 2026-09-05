# Property Test Utilities

This module provides shared infrastructure for property-based testing across the leiden workspace.

## Module Overview

| File | Purpose |
|------|---------|
| `config.rs` | Proptest configuration constants and helpers |
| `graphs.rs` | Graph generation strategies |
| `invariants.rs` | Shared assertion helpers |

All modules are `#[cfg(test)]` — zero production code impact.

## Usage

### Graph Generators

```rust
use crate::testing::graphs::{ErdosRenyi, GraphGenerator};

let gen = ErdosRenyi::new(0.3);
let graph = gen.generate(&mut rng);
```

### Proptest Configuration

```rust
use crate::testing::config::proptest_config;

#[proptest_config(proptest_config(Some(50), cfg!(debug_assertions)))]
```

### Assertion Helpers

```rust
use crate::testing::invariants::{assert_eps_eq, assert_finite, assert_modularity_valid};

assert_finite(quality);
assert_modularity_valid(quality);
```

## Topology Coverage (FR-006)

Each `property_tests` module MUST use at least 3 distinct topologies:
1. 1 random (Erdős-Rényi)
2. 1 community-structured (StochasticBlock)
3. 1 edge-case topology (ParallelEdges or DisconnectedGraph)

## Doc Comment Convention (SC-007)

Each `#[cfg(test)] mod property_tests` MUST have a doc comment:
```rust
/// Verifies INV-XXX: Description of what this test verifies.
#[cfg(test)]
mod property_tests {
    // ...
}
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `DEFAULT_TIMEOUT_MS` | 30000 | Per-case timeout (FR-014) |
| `BASE_CASES` | 1000 | Local development cases |
| `CI_TEST_CASES` | 256 | CI environment cases |
| `LOCAL_MIN_CASES` | 100 | Local minimum floor |
| `MAX_SHRINK_ITERS` | 200 | Maximum shrink iterations |
| `MODULARITY_EPSILON` | 1e-9 | Float comparison tolerance (FR-009) |
| `MIN_NODES` | 5 | Minimum graph nodes |
| `MAX_NODES` | 100 | Maximum graph nodes |
| `MIN_WEIGHT` | 1e-6 | Minimum edge weight |
| `MAX_WEIGHT` | 100.0 | Maximum edge weight |

## Traceability Matrix

See tasks.md for the complete mapping of invariants to tests.
