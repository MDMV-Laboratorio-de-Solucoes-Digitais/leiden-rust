//! Integration tests: Idle state rendering snapshot (T096).

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
fn idle_renders_three_panels() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let app = App::new_idle();

    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("renders frame");

    let buffer = terminal.backend().buffer();
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Logs"));
    assert!(debug_str.contains("IDLE"));
}
