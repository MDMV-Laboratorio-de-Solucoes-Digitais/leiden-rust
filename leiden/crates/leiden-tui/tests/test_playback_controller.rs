//! Integration tests: `PlaybackController` state machine (T020, US2).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent};
use leiden_tui::app::{App, GranularityMode, PlaybackController};

#[test]
fn default_controller_is_paused_phase_level() {
    let fresh = PlaybackController::new();
    assert!(!fresh.is_playing, "new() must start paused");
    assert_eq!(fresh.tick_speed_ms, 200, "new() tick speed must be 200ms");
    assert!(!fresh.step_requested, "new() must not have a step pending");
    assert_eq!(
        fresh.granularity,
        GranularityMode::PhaseLevel,
        "new() must default to PhaseLevel granularity"
    );

    let defaulted = PlaybackController::default();
    assert!(!defaulted.is_playing, "default() must start paused");
    assert_eq!(
        defaulted.tick_speed_ms, 200,
        "default() tick speed must be 200ms"
    );
    assert!(!defaulted.step_requested, "default() must not have a step pending");
    assert_eq!(
        defaulted.granularity,
        GranularityMode::PhaseLevel,
        "default() must default to PhaseLevel granularity"
    );
}

#[test]
fn toggle_play_flips_state() {
    let mut controller = PlaybackController::new();
    assert!(!controller.is_playing, "must start paused");

    controller.request_step();
    assert!(controller.step_requested, "request_step must set the flag");

    // Single toggle starts playing and clears step_requested.
    controller.toggle_play();
    assert!(controller.is_playing, "first toggle must start playback");
    assert!(
        !controller.step_requested,
        "starting playback must clear step_requested"
    );

    // Second toggle returns to paused.
    controller.toggle_play();
    assert!(!controller.is_playing, "second toggle must pause again");
}

#[test]
fn request_step_auto_pauses() {
    let mut controller = PlaybackController::new();
    controller.toggle_play();
    assert!(controller.is_playing, "playback must be running before the step");

    controller.request_step();
    assert!(!controller.is_playing, "request_step must auto-pause");
    assert!(controller.step_requested, "request_step must set step_requested");
}

#[test]
fn toggle_granularity_flips_mode() {
    let mut controller = PlaybackController::new();
    assert_eq!(controller.granularity, GranularityMode::PhaseLevel);

    controller.toggle_granularity();
    assert_eq!(
        controller.granularity,
        GranularityMode::MicroStep,
        "first toggle must switch to MicroStep"
    );

    controller.toggle_granularity();
    assert_eq!(
        controller.granularity,
        GranularityMode::PhaseLevel,
        "second toggle must return to PhaseLevel"
    );
}

#[test]
fn on_preset_switch_pauses_and_preserves_granularity() {
    let mut controller = PlaybackController::new();
    controller.granularity = GranularityMode::MicroStep;
    controller.toggle_play();
    assert!(controller.is_playing, "playback must be running before switch");

    controller.on_preset_switch();
    assert!(!controller.is_playing, "preset switch must pause playback");
    assert!(!controller.step_requested, "preset switch must clear step_requested");
    assert_eq!(
        controller.granularity,
        GranularityMode::MicroStep,
        "preset switch must preserve the user's granularity"
    );
}

#[test]
fn space_key_toggles_play_and_syncs_paused() {
    let mut app = App::new_idle();
    assert!(!app.playback.is_playing, "app must start paused");

    app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
    assert!(app.playback.is_playing, "Space must start playback");
    assert!(
        !app.control.paused.load(Ordering::SeqCst),
        "starting playback must set control.paused to false"
    );

    app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
    assert!(!app.playback.is_playing, "second Space must pause playback");
    assert!(
        app.control.paused.load(Ordering::SeqCst),
        "pausing must set control.paused to true"
    );
}

#[test]
fn n_key_steps_and_auto_pauses() {
    let mut app = App::new_idle();
    app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
    assert!(app.playback.is_playing, "Space must start playback before stepping");

    app.handle_key(KeyEvent::from(KeyCode::Char('n')));
    assert!(!app.playback.is_playing, "'n' must auto-pause playback");
    assert!(
        app.control.paused.load(Ordering::SeqCst),
        "'n' must set control.paused to true"
    );
    assert!(
        app.control.step.load(Ordering::SeqCst),
        "'n' must set control.step to true"
    );
}

#[test]
fn right_arrow_key_steps() {
    let mut app = App::new_idle();
    app.handle_key(KeyEvent::from(KeyCode::Right));
    assert!(
        app.playback.step_requested,
        "Right arrow must set step_requested"
    );
    assert!(!app.playback.is_playing, "Right arrow must keep playback paused");
}

#[test]
fn t_key_toggles_granularity() {
    let mut app = App::new_idle();
    assert_eq!(app.playback.granularity, GranularityMode::PhaseLevel);

    app.handle_key(KeyEvent::from(KeyCode::Char('t')));
    assert_eq!(
        app.playback.granularity,
        GranularityMode::MicroStep,
        "'t' must switch to MicroStep"
    );

    app.handle_key(KeyEvent::from(KeyCode::Char('t')));
    assert_eq!(
        app.playback.granularity,
        GranularityMode::PhaseLevel,
        "second 't' must return to PhaseLevel"
    );
}

#[test]
fn preset_key_preserves_granularity_and_pauses() {
    let mut app = App::new_idle();
    app.handle_key(KeyEvent::from(KeyCode::Char('t')));
    assert_eq!(
        app.playback.granularity,
        GranularityMode::MicroStep,
        "'t' must switch to MicroStep before the preset switch"
    );

    app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
    assert!(app.playback.is_playing, "Space must start playback before the switch");

    app.handle_key(KeyEvent::from(KeyCode::Char('2')));
    assert!(!app.playback.is_playing, "preset key must auto-pause playback");
    assert_eq!(
        app.playback.granularity,
        GranularityMode::MicroStep,
        "preset key must preserve the user's granularity"
    );
    assert!(
        app.control.paused.load(Ordering::SeqCst),
        "preset key must set control.paused to true"
    );
}

#[test]
fn step_requested_cleared_when_playback_resumes() {
    let mut controller = PlaybackController::new();
    controller.request_step();
    assert!(controller.step_requested, "request_step must set the flag");
    assert!(!controller.is_playing, "request_step must pause playback");

    controller.toggle_play();
    assert!(controller.is_playing, "toggle_play must resume playback");
    assert!(
        !controller.step_requested,
        "resuming playback must clear step_requested"
    );
}
