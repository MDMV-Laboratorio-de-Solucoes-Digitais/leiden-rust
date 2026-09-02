# Phase 0: Research

## Research: Thread Synchronization
- **Decision:** Use an `Arc<AtomicBool>` for the `paused` state and an `Arc<AtomicBool>` for a `step` trigger. The worker thread will spin-wait or sleep briefly (`thread::sleep`) when `paused == true`, unless `step == true`. If `step == true`, it completes one iteration and flips `step` back to false. A `quit` signal can similarly be an `Arc<AtomicBool>` to abort the wait loop.
- **Rationale:** The Leiden algorithm's `orchestrator/mod.rs` outer loop currently doesn't hold heavy mutexes across iterations. Simple atomic booleans checked at the start/end of the iteration loop (and potentially during the loop) are lightweight and avoid deadlocks with the MPSC event channel.
- **Alternatives considered:** `std::sync::Condvar` with a `Mutex<State>`. This is more traditional for blocking but requires mutating the orchestrator to carry a `Condvar`. Atomics with short `sleep(10ms)` or `thread::yield_now()` are often sufficient for TUI tools and easier to integrate cleanly.

## Research: Ratatui Community-Clustered Grid Layout
- **Decision:** Use `ratatui::widgets::canvas::Canvas` with `Points` or `ratatui::text` block layout to group nodes into distinct spatial blocks. We can assign each community a bounding box on the canvas, and dynamically position nodes within their community's bounding box.
- **Rationale:** The `design-system.md` expects visual clarity and community separation. A true force-directed layout is computationally heavy. Placing nodes into pre-calculated community "grid cells" (e.g., if there are 4 communities, a 2x2 grid) allows for fast spatial grouping and immediate O(1) positioning.
- **Alternatives considered:** A fully interactive force-directed graph (rejected due to overhead constraint < 10%). A raw textual output (rejected by functional requirements).

## Research: Intermediate State Emission
- **Decision:** Add `partition: crate::Partition` to `LeidenEvent::IterationFinished`.
- **Rationale:** The TUI's Graph View needs the exact node-to-community mapping to render the layout. Cloning the partition (which is essentially a `Vec<u32>`) at the end of each iteration is extremely fast (microseconds for typical graphs) and easily fits within the <10% overhead constraint.
- **Alternatives considered:** Sending a delta (moved nodes). While cheaper, it requires the TUI to reconstruct the state, adding complexity. A full clone is simpler and perfectly acceptable for performance.
