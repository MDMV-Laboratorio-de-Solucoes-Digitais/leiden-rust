//! Key-binding and terminal event definitions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions triggered by user key presses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    /// Exit the application.
    Quit,
    /// Restart execution from the beginning.
    Restart,
    /// Advance execution by a single step.
    Step,
    /// Pause or resume automatic execution.
    PauseResume,
    /// Toggle auto-play stepping (Space, Contract §2.1).
    PlayPause,
    /// Advance exactly one step forward, auto-pausing (n / Right Arrow).
    StepForward,
    /// Toggle `PhaseLevel` / `MicroStep` stepping granularity (t).
    ToggleGranularity,
    /// Load the Karate Club preset (key `1`).
    LoadPresetKarateClub,
    /// Load the Two Cliques preset (key `2`).
    LoadPresetTwoCliques,
    /// Load the Random Mess preset (key `3`).
    LoadPresetRandomMess,
    /// Toggle visibility of the graph view panel.
    ToggleGraph,
    /// Toggle visibility of the log pane.
    ToggleLog,
    /// Cycle keyboard focus to the next visible panel.
    FocusNext,
    /// Move selection up in the current list/table.
    SelectUp,
    /// Move selection down in the current list/table.
    SelectDown,
    /// Toggle the interactive help modal overlay.
    ToggleHelp,
    /// No-op or unrecognized key.
    None,
}

/// Map a raw `crossterm::event::KeyEvent` to an `AppAction`.
#[must_use]
pub fn map_key_event(key: KeyEvent) -> AppAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return AppAction::Quit;
    }

    match key.code {
        KeyCode::Char('q') => AppAction::Quit,
        KeyCode::Char('r') => AppAction::Restart,
        KeyCode::Char('s') => AppAction::Step,
        KeyCode::Char('p') => AppAction::PauseResume,
        KeyCode::Char(' ') => AppAction::PlayPause,
        KeyCode::Char('n') | KeyCode::Right => AppAction::StepForward,
        KeyCode::Char('t') => AppAction::ToggleGranularity,
        KeyCode::Char('1') => AppAction::LoadPresetKarateClub,
        KeyCode::Char('2') => AppAction::LoadPresetTwoCliques,
        KeyCode::Char('3') => AppAction::LoadPresetRandomMess,
        KeyCode::Char('g') => AppAction::ToggleGraph,
        KeyCode::Char('l') => AppAction::ToggleLog,
        KeyCode::Tab => AppAction::FocusNext,
        KeyCode::Up => AppAction::SelectUp,
        KeyCode::Down => AppAction::SelectDown,
        KeyCode::Char('?') => AppAction::ToggleHelp,
        _ => AppAction::None,
    }
}
