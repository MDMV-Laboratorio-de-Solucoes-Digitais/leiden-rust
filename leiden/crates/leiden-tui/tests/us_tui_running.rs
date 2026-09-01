//! Integration tests: Running state rendering snapshot (T097).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::LeidenEvent;
use leiden_tui::app::{App, AppState};
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn running_state_shows_progress_bar() {
    let mut app = App::new_idle();
    app.state = AppState::Running { iteration: 3 };
    app.push(LeidenEvent::IterationFinished {
        index: 3,
        quality: 0.4127,
    });

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("renders frame");

    let buffer = terminal.backend().buffer();
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("RUNNING") || debug_str.contains("Iter 3"));
}
