# Specification: TUI Algorithm Stepping

## Purpose & Background
Users need a way to watch the drawn graph being iterated or step through the Leiden algorithm in the TUI. Currently, the interface has some stubs (like the `s` key for stepping and `p` for pausing), but they are unimplemented. The algorithm runs on a separate background thread without listening to pause signals, only emits metadata during iterations instead of the full partition state, and uses a basic text-based graph visualization. This feature will bridge these gaps to provide a fully interactive, visual algorithm stepping experience.

## Clarifications
### Session 2026-09-01
- Q: Since the input graph lacks (X,Y) coordinates, how should the TUI determine node positions for the spatial canvas to visualize communities? -> A: Community-clustered grid (Group nodes into spatial blocks by community)
- Q: What should the TUI do if the user presses `s` (step) while the algorithm is continuously running (not paused)? -> A: Option B - Automatically switch to "paused" mode, finish the current iteration, and wait
- Q: What should happen if the user attempts to quit the application (e.g., presses q) while the algorithm is paused and waiting for a step signal? -> A: Automatically switch to "paused" mode, finish the current iteration, informs the user that if he quits now, the algorithm will be aborted, and the application will exit cleanly, without producing any results.


## User Scenarios
1. **Interactive Stepping:** A user launches the TUI, loads a graph, and presses `p` to pause the algorithm. The user then repeatedly presses `s` to step through the algorithm one iteration at a time, watching the visual graph update its communities after each step.
2. **Visual Observation:** A user watches the algorithm run without pausing, but the visual graph continuously updates its spatial layout and community coloring in real-time as nodes move between communities during iterations.

## Functional Requirements
1. **Thread Synchronization:** The Leiden orchestrator must accept a synchronization primitive (e.g., an `AtomicBool` or command channel) to block/wait at the end of each iteration loop when the TUI is in a "paused" state.
2. **Intermediate State Emission:** The orchestrator must clone and include the current node-to-community `Partition` state inside the `LeidenEvent::IterationFinished` event (or a new dedicated event) to allow the TUI to reflect mid-run data.
3. **Key Bindings Implementation:** The TUI's event loop (`app.rs`) must explicitly handle the `s` key. If already paused, it sends a "step" signal to allow exactly one iteration. If pressed while the algorithm is running continuously, the app must automatically switch to "paused" mode, finish the current iteration, and then wait.

## Edge Cases & Failure Handling
- **Quitting while paused:** If the user presses `q` to quit while the algorithm is currently running or waiting for a step signal, the TUI must inform the user that quitting now will abort the algorithm. If they confirm or quit while paused, the application unblocks the worker thread with an abort signal and exits cleanly without producing final output results.

## Success Criteria
1. Pressing `p` successfully halts the algorithm at the end of the current iteration, and pressing `s` advances it by exactly one iteration.
2. During the run (whether stepping or running continuously), the graph visualization updates its state (communities) mid-run, rather than only at the end.
3. The visual graph view renders nodes spatially (using a canvas or block representation) rather than merely printing text-based IDs and community numbers.
4. The algorithm's asymptotic performance (when running continuously) is not unacceptably degraded by the state cloning and synchronization overhead (e.g., overhead is < 10% on large graphs or cloning is skipped when the graph is hidden).

## Assumptions & Dependencies
- The `ratatui` canvas or block layout will be used for the enhanced rendering to stay within the terminal boundaries.
- The state cloning overhead is acceptable per-iteration, as iterations are relatively coarse-grained. If performance becomes an issue, cloning can be skipped when not paused or not actively rendering the graph view.
