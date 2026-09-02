# Phase 1: Data Model

## Entities

### `ControlFlags`
A new state container shared between the TUI thread and the Leiden worker thread.
- **Fields:**
  - `paused`: `Arc<AtomicBool>` - True if the algorithm should halt at the end of the current iteration.
  - `step`: `Arc<AtomicBool>` - Set to true by the TUI to request exactly one iteration to advance.
  - `abort`: `Arc<AtomicBool>` - Set to true if the TUI is exiting and the algorithm should immediately return/terminate.

### `LeidenEvent::IterationFinished` (Modified)
The existing event needs to carry the partition state.
- **Fields (added):**
  - `partition: Option<Partition>` (or explicitly `Vec<u32>`) - The cloned node-to-community mapping at the end of the iteration.

### `CommunityGrid`
A structure in the TUI to track spatial layout state.
- **Fields:**
  - `community_centers`: `HashMap<u32, (f64, f64)>` - Central coordinates for each community.
  - `node_positions`: `HashMap<String, (f64, f64)>` - Current rendered position of each node to maintain visual stability across iterations.
