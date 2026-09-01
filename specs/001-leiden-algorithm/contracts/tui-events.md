# TUI Events & Logging Contract: `leiden-tui`

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30

This document defines the event-channel contract between the library and the
TUI, and the tracing-subscriber setup that bridges them. It complements
`data-model.md §1.8` and `cli-schema.md §2`.

---

## 1. Channel Topology

```text
┌─────────────────────────┐                ┌─────────────────────────┐
│   leiden orchestrator   │  LeidenEvent   │   leiden-tui main       │
│   (worker thread)       │ ─────────────► │   (render thread)       │
│                         │   mpsc::Sender │                         │
│                         │                │   App { rx, … }         │
└─────────────────────────┘                └─────────────────────────┘
         │                                            │
         │ tracing::info!(…)                          │ ratatui::draw
         ▼                                            ▼
┌─────────────────────────┐                ┌─────────────────────────┐
│ tracing-subscriber      │                │   terminal raw mode     │
│ (global, library-owned) │                │                         │
└─────────────────────────┘                └─────────────────────────┘
         │
         ├─► stderr (only when --log-file unset AND not a TTY)
         ├─► log file (--log-file, JSON lines)
         └─► in-memory ring buffer (TUI log pane, 500 entries)
```

The library owns the global `tracing-subscriber`; the TUI attaches an
**additional** `Layer<S>` to the existing subscriber (rather than installing
its own) so the library's diagnostics are the single source of truth.

---

## 2. Channel Configuration

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use leiden::{Leiden, LeidenEvent};

let (tx, rx): (Sender<LeidenEvent>, Receiver<LeidenEvent>) = channel();
let worker = std::thread::spawn(move || {
    Leiden::new()
        .with_event_sink(tx)
        .with_parameters(params)
        .run(&graph)
});

let app = App::new(rx); // drains on each tick via try_recv()
```

- **Channel type**: `std::sync::mpsc`, **bounded** at 1024 messages. If the
  buffer is full, the worker sends `LeidenEvent::Throttled { dropped: u64 }`
  and drops further events; this satisfies `unused_results = deny` and never
  blocks the worker. The 1024-event buffer absorbs bursts up to ~20,000 events/sec
  across local-moving iterations at standard tick intervals.
- **Tick rate & event polling**: The render loop polls for terminal events via
  `crossterm::event::poll(Duration::from_millis(50))` (20 Hz tick rate) while active,
  draining the channel every 50ms.
- **Channel error handling**: `Sender::send` returns `Result`; on `Err` the
  worker logs `tracing::warn!` and continues (the TUI has exited).
- **Drain loop**: `while let Ok(event) = rx.try_recv() { app.push(event); }`
  per tick. `try_recv` not `recv` — render thread must never block.

---

## 3. `LeidenEvent` Variants (full)

```rust
#[derive(Debug, Clone)]
pub enum LeidenEvent {
    GraphLoaded { nodes: usize, edges: usize, total_weight: f64 },
    IterationStarted { iteration: u32, phase: Phase },
    LocalMovingProgress { iteration: u32, moved_nodes: u32 },
    LocalMovingDelta { iteration: u32, delta_q: f64 },
    RefinementMerged { iteration: u32, from: u32, to: u32 },
    Aggregation { iteration: u32, aggregate_nodes: usize },
    QualityComputed { iteration: u32, quality: f64 },
    IterationFinished { iteration: u32, quality: f64 },
    Terminated { iterations: u32, reason: TerminationReason, quality: f64 },
    Throttled { dropped: u64 },
}

#[derive(Debug, Clone, Copy)]
pub enum Phase { LocalMoving, Refinement, Aggregation }
```

The TUI panel rendering uses `LeidenEvent::IterationFinished` and
`LeidenEvent::Terminated` to update the community list and status bar.

---

## 4. Tracing Subscriber Setup

The TUI installs two layers:

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

let env_filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info"));

let fmt_layer = fmt::layer()
    .with_writer(std::io::stderr)
    .with_ansi(false) // ratatui owns the terminal
    .with_target(true);

let log_pane_layer = LogPaneLayer::new(app_log_sink.clone());

tracing_subscriber::registry()
    .with(env_filter)
    .with(fmt_layer)
    .with(log_pane_layer)
    .init();
```

`fmt_layer` is gated to `is_terminal() == false` (the TTY is owned by ratatui
once `ratatui::init()` is called); when the TUI exits, fmt_layer resumes.

`LogPaneLayer` is the custom in-memory layer; see `crates/leiden-tui/src/logging.rs`.

---

## 5. In-Memory Log Pane Layer

```rust
pub struct LogPaneLayer {
    sink: Arc<Mutex<RingBuffer<String>>>,
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for LogPaneLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut buf = String::with_capacity(128);
        let _ = write!(
            &mut buf,
            "[{}] {}: {}",
            event.metadata().level(),
            event.metadata().target(),
            event.metadata().name(),
        );
        // Field values:
        struct V<'a>(&'a mut String);
        impl<'a> tracing::field::Visit for V<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                use std::fmt::Write as _;
                let _ = write!(&mut self.0, " {field:?}={value:?}");
            }
        }
        event.record(&mut V(&mut buf));
        self.sink.lock().push_back(buf);
    }
}
```

- Ring buffer size: 500 entries. Older entries are evicted.
- The TUI's `LogPane` widget renders the buffer as a `Paragraph` with `wrap`
  off and `scroll_offset` controlled by `j`/`k` (not in the public key map;
  internal to the panel).

---

## 6. Test Contract

TUI tests use `ratatui::backend::TestBackend`:

```rust
#[test]
fn idle_renders_three_panels() {
    let backend = ratatui::backend::TestBackend::new(120, 40);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    let app = App::new_idle();
    terminal.draw(|f| ui::render(f, &app)).unwrap();
    insta::assert_snapshot!(format!("{:?}", terminal.backend().buffer()));
}

#[test]
fn running_state_shows_progress_bar() {
    let mut app = App::new_idle();
    app.state = AppState::Running { iteration: 3 };
    app.events.push(LeidenEvent::IterationFinished {
        iteration: 3, quality: 0.4127,
    });
    let backend = ratatui::backend::TestBackend::new(120, 40);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::render(f, &app)).unwrap();
    insta::assert_snapshot!(format!("{:?}", terminal.backend().buffer()));
}
```

The `insta` snapshots live in `crates/leiden-tui/tests/snapshots/` and are
updated via `cargo insta review` on intentional UI changes.

---

## 7. End-of-Run Cleanup & Panic Safety
 
When the orchestrator returns or terminates:
 
1. The worker thread `JoinHandle` is dropped (the thread is in a `Send` safe
   state because `Leiden::run` returns `Result` and never panics). If the worker thread
   terminates unexpectedly or disconnects, the render loop transitions to `AppState::Error`.
2. The TUI's main thread receives the `Terminated` event and transitions to
   `AppState::Done` (or `AppState::Error` on `LeidenError`).
3. `ratatui::restore()` is called from `main` (and also registered via a panic hook
   `std::panic::set_hook`), ensuring terminal raw mode and screen buffers are cleanly restored
   even on unexpected panics or signals.
4. The `tracing` subscriber is dropped in reverse order: `fmt_layer` resumes
   writing to stderr until process exit.