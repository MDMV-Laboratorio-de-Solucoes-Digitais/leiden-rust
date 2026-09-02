//! Performance benchmark: force-simulation physics tick budget (T039).
//!
//! Asserts a physics tick completes in ≤ 5 ms on a 50-node / 100-edge
//! fixture (SC-002, Contract §3.2) and measures throughput with Criterion.
//!
//! Running `cargo bench --bench simulation_perf` executes two phases:
//!
//! 1. **Budget gate** — 100 warmup ticks, then 500 ticks timed with
//!    [`Instant`]; the mean per-tick duration is asserted against the
//!    SC-002 budget (5 ms under release, 40 ms under debug — see
//!    [`DEBUG_TICK_BUDGET`] for why the debug budget is relaxed).
//! 2. **Criterion group** — the same tick is measured with Criterion for
//!    throughput statistics and regression tracking under
//!    `target/criterion`.

#![expect(
    clippy::expect_used,
    reason = "benchmark asserts the physics tick budget and may fail loudly"
)]
#![expect(
    clippy::print_stdout,
    reason = "bench binary prints its budget-gate summary line to stdout"
)]
#![expect(
    missing_docs,
    reason = "criterion_group! expands to an undocumented `pub fn benches` entry point"
)]

use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group};
use leiden_tui::simulation::ForceSimulation;

/// Node count of the SC-002 fixture (50 nodes / 100 edges).
const NODE_COUNT: usize = 50;

/// Nodes per community: 50 / 5 = 10 communities of 5 nodes.
const COMMUNITY_SIZE: usize = 5;

/// Warmup ticks executed before the timed budget window.
const WARMUP_TICKS: u32 = 100;

/// Ticks timed for the mean per-tick budget measurement.
const MEASURED_TICKS: u32 = 500;

/// Tick budget under a release-profile build (SC-002, Contract §3.2).
///
/// 5 ms per relaxation step keeps the physics workload at ≤ 10% of the
/// 20 FPS / 50 ms frame budget, leaving room for layout, explanation
/// rendering, and I/O in the same frame.
const RELEASE_TICK_BUDGET: Duration = Duration::from_millis(5);

/// Tick budget under a debug-profile build.
///
/// The contract gate itself runs under release (Constitution `--release`
/// test gate), where [`RELEASE_TICK_BUDGET`] applies. Debug builds execute
/// the O(n²) pair loop and `HashMap` traffic roughly an order of magnitude
/// slower, so this relaxed 40 ms smoke gate keeps debug-profile runs from
/// failing on interpreter-style slowdowns while still catching gross
/// regressions. The Criterion measurement itself is unchanged either way.
const DEBUG_TICK_BUDGET: Duration = Duration::from_millis(40);

/// Partition entries mapping each node id to its community id.
type Partition = Vec<(String, u32)>;

/// Undirected edge endpoints as node id pairs.
type EdgeList = Vec<(String, String)>;

/// Build the deterministic 50-node / 100-edge benchmark fixture.
///
/// Node ids are `node_0`..`node_49`. Edges form a ring (`node_i` ↔
/// `node_{(i + 1) % 50}`) plus chords (`node_i` ↔ `node_{(i + 17) % 50}`);
/// all 100 pairs are distinct, so no dedupe pass is needed. The partition
/// assigns `node_i` to community `i / 5` (10 communities of 5).
fn build_fixture() -> (ForceSimulation, Partition, EdgeList) {
    let nodes: Vec<String> = (0..NODE_COUNT).map(|i| format!("node_{i}")).collect();
    let sim = ForceSimulation::new(&nodes);

    let partition: Vec<(String, u32)> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let community = u32::try_from(i / COMMUNITY_SIZE)
                .expect("community id fits in u32 for the 50-node fixture");
            (node.clone(), community)
        })
        .collect();

    let mut edges: Vec<(String, String)> = Vec::with_capacity(2 * NODE_COUNT);
    for (i, node) in nodes.iter().enumerate() {
        edges.push((node.clone(), nodes[(i + 1) % NODE_COUNT].clone()));
        edges.push((node.clone(), nodes[(i + 17) % NODE_COUNT].clone()));
    }

    (sim, partition, edges)
}

/// Criterion benchmark: time one full force-relaxation step on the fixture.
fn bench_tick(c: &mut Criterion) {
    let _ = c.bench_function("force_simulation_tick_50_nodes", |b| {
        let (mut sim, partition, edges) = build_fixture();
        b.iter(|| {
            sim.tick(&partition, &edges);
        });
    });
}

/// Time the tick after warmup and assert the mean per-tick duration
/// against the SC-002 budget for the current build profile.
///
/// # Panics
///
/// Panics when the mean tick duration exceeds the budget; the panic
/// message includes the measured value.
fn assert_tick_budget() {
    let (mut sim, partition, edges) = build_fixture();

    for _ in 0..WARMUP_TICKS {
        sim.tick(&partition, &edges);
    }

    let start = Instant::now();
    for _ in 0..MEASURED_TICKS {
        sim.tick(&partition, &edges);
    }
    let mean = start.elapsed() / MEASURED_TICKS;

    let debug_build = cfg!(debug_assertions);
    let budget = if debug_build {
        DEBUG_TICK_BUDGET
    } else {
        RELEASE_TICK_BUDGET
    };

    assert!(
        mean <= budget,
        "physics tick mean {mean:?} exceeded the {} ms budget \
         (SC-002, Contract §3.2) on the 50-node/100-edge fixture",
        budget.as_millis()
    );

    println!(
        "physics tick mean: {mean:?} (budget {}ms, {} profile)",
        budget.as_millis(),
        if debug_build { "debug" } else { "release" }
    );
}

criterion_group!(benches, bench_tick);

fn main() {
    assert_tick_budget();
    benches();
}
