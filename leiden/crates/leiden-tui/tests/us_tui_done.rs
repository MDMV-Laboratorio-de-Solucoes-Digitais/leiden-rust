//! Integration tests: Done state rendering snapshot (T098).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::{LeidenEvent, TerminationReason};
use leiden_tui::app::App;
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn done_state_shows_final_partition() {
    let mut app = App::new_idle();
    app.partition = vec![
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 1),
    ];
    app.push(LeidenEvent::Terminated {
        iterations: 3,
        reason: TerminationReason::Converged,
        quality: 0.4127,
    });

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("renders frame");

    let buffer = terminal.backend().buffer();
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("Done") || debug_str.contains("DONE"));
    assert!(debug_str.contains("0.4127"));
}
