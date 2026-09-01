//! Integration tests: Key binding state transitions (T116).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use leiden_tui::app::{App, AppState, FocusPanel};

#[test]
fn tui_key_bindings_state_transitions() {
    let mut app = App::new_idle();

    // 1. Toggle graph
    assert!(app.visibility.show_graph);
    app.handle_key(KeyEvent::from(KeyCode::Char('g')));
    assert!(!app.visibility.show_graph);
    app.handle_key(KeyEvent::from(KeyCode::Char('g')));
    assert!(app.visibility.show_graph);

    // 2. Toggle log
    assert!(app.visibility.show_log);
    app.handle_key(KeyEvent::from(KeyCode::Char('l')));
    assert!(!app.visibility.show_log);
    app.handle_key(KeyEvent::from(KeyCode::Char('l')));
    assert!(app.visibility.show_log);

    // 3. Tab focus rotation
    assert_eq!(app.focus, FocusPanel::CommunityList);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::GraphView);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::LogPane);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::CommunityList);

    // 4. Pause toggle
    assert!(!app.control.paused.load(std::sync::atomic::Ordering::SeqCst));
    app.handle_key(KeyEvent::from(KeyCode::Char('p')));
    assert!(app.control.paused.load(std::sync::atomic::Ordering::SeqCst));

    // 5. Help overlay
    assert!(!app.visibility.help_open);
    app.handle_key(KeyEvent::from(KeyCode::Char('?')));
    assert!(app.visibility.help_open);
    app.handle_key(KeyEvent::from(KeyCode::Char('?')));
    assert!(!app.visibility.help_open);

    // 6. Restart
    app.state = AppState::Done {
        iterations: 5,
        quality: 0.5,
    };
    app.handle_key(KeyEvent::from(KeyCode::Char('r')));
    assert_eq!(app.state, AppState::Running { iteration: 0 });

    // 7. Quit via 'q'
    assert!(!app.control.should_quit);
    app.handle_key(KeyEvent::from(KeyCode::Char('q')));
    assert!(!app.control.should_quit); // Needs confirmation because state is Running
    app.handle_key(KeyEvent::from(KeyCode::Char('y')));
    assert!(app.control.should_quit);

    // 8. Quit via Ctrl+C
    app.control.should_quit = false;
    app.state = AppState::Running { iteration: 0 };
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(!app.control.should_quit);
    app.handle_key(KeyEvent::from(KeyCode::Char('y')));
    assert!(app.control.should_quit);
}
