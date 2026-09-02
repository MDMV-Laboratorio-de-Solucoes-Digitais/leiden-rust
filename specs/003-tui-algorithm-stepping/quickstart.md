# Quickstart Validation Guide

## 1. Prerequisites
- Compile the project: `cargo build --workspace`
- A sample graph file (e.g., `leiden/fixtures/zachary.txt` if one exists, or any edge list).

## 2. Interactive Stepping Scenario
1. Run the TUI with a sample graph:
   `cargo run --bin leiden-tui -- path/to/graph.txt`
2. Immediately press `p` to enter **paused** mode.
3. Switch to the **Graph View** using `Tab` (or ensure it's visible with `g`).
4. Press `s` repeatedly.
   - **Expected Outcome:** The log pane should show exactly one `IterationStarted` and `IterationFinished` event per `s` press. The Graph View should spatially group nodes by their new community assignment after each press.
5. Press `q` to quit.
   - **Expected Outcome:** The application immediately exits gracefully, even though the algorithm was in a paused state.

## 3. Continuous Observation Scenario
1. Run the TUI again:
   `cargo run --bin leiden-tui -- path/to/graph.txt`
2. Do not press `p`. 
3. Switch to the Graph View.
4. **Expected Outcome:** The graph automatically updates its layout in real-time as communities form, running until convergence or the iteration cap is reached.

## 4. Overhead Validation
1. Run the TUI on a large graph in release mode:
   `cargo run --release --bin leiden-tui -- path/to/large_graph.txt`
2. Observe the total completion time compared to running the base algorithm without the TUI stepping features.
3. **Expected Outcome:** The overhead is <10%.
