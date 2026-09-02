# Worker Interface Contract

The boundary between `leiden-tui` and the `leiden` core crate will expand to include runtime control.

## `Leiden` Orchestrator Builder
The `Leiden` struct will accept a control struct:
```rust
pub fn with_control_flags(mut self, flags: ControlFlags) -> Self
```

## Control Loop Behavior
During `run_outer_loop` inside `leiden`, the orchestrator must poll the `ControlFlags`:
1. Check `abort`. If true, break and return `TerminationReason::DegenerateInput` (or a new `Aborted` variant).
2. If `paused` is true, spin-wait or sleep-wait. 
3. While waiting, continually check `abort` (to break) and `step` (to advance).
4. If `step` becomes true, set `step` to false and execute exactly one iteration, then loop back to the wait state.
