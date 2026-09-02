//! Integration tests: Idle state rendering snapshot (T096, updated for the
//! 004 visual-explanation layout).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::app::App;
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn idle_renders_explanation_and_canvas() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let app = App::new_idle();

    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("renders frame");

    let buffer = terminal.backend().buffer();
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("EXPLANATION"));
    assert!(debug_str.contains("GRAPH VISUALIZATION"));
    assert!(debug_str.contains("IDLE"));
}
