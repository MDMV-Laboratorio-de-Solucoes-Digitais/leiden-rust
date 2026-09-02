//! Integration test: keybinding help overlay toggle (T035, Phase 6).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use crossterm::event::{KeyCode, KeyEvent};
use leiden_tui::app::App;
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Render `app` onto a 100×30 test terminal and return the buffer's debug
/// representation for substring assertions.
fn render_to_buffer(app: &App) -> String {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, app)).expect("renders frame");
    format!("{:?}", terminal.backend().buffer())
}

#[test]
fn help_closed_by_default() {
    let app = App::new_idle();
    assert!(
        !app.visibility.help_open,
        "a freshly created idle app must not open the help modal"
    );

    let debug = render_to_buffer(&app);
    assert!(
        !debug.contains("Help / Key Bindings"),
        "help modal title must not appear in the buffer when help is closed"
    );
}

#[test]
fn question_key_opens_help_modal() {
    let mut app = App::new_idle();
    app.handle_key(KeyEvent::from(KeyCode::Char('?')));
    assert!(
        app.visibility.help_open,
        "pressing '?' must toggle help_open to true"
    );

    let debug = render_to_buffer(&app);
    assert!(
        debug.contains("Help / Key Bindings"),
        "help modal title must be drawn while help_open is true"
    );
    assert!(
        debug.contains("Play / pause auto-stepping"),
        "help modal must list the Space play/pause binding"
    );
    assert!(
        debug.contains("Toggle granularity (Phase/Micro)"),
        "help modal must list the 't' granularity binding"
    );
}

#[test]
fn question_key_toggles_help_closed() {
    let mut app = App::new_idle();

    app.handle_key(KeyEvent::from(KeyCode::Char('?')));
    assert!(app.visibility.help_open, "first '?' press must open help");

    app.handle_key(KeyEvent::from(KeyCode::Char('?')));
    assert!(
        !app.visibility.help_open,
        "second '?' press must toggle help_open back to false"
    );

    let debug = render_to_buffer(&app);
    assert!(
        !debug.contains("Help / Key Bindings"),
        "help modal title must not appear in the buffer after toggling help closed"
    );
}

#[test]
fn help_modal_lists_core_bindings() {
    let mut app = App::new_idle();
    app.handle_key(KeyEvent::from(KeyCode::Char('?')));
    assert!(app.visibility.help_open);

    let debug = render_to_buffer(&app);

    let core_bindings = [
        "Load preset dataset",
        "Restart explanation run",
        "Quit application",
        "Cycle focused panel",
        "Advance one step",
    ];
    let listed = core_bindings
        .iter()
        .filter(|binding| debug.contains(*binding))
        .count();
    assert!(
        listed >= 4,
        "help modal must list at least 4 of the core bindings, found {listed}: {core_bindings:?}"
    );
}
