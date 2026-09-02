//! Status bar panel widget: playback state, progress, granularity, hints.

#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "ratio is clamped to 0.0..=1.0 and rounded before the f64 -> usize \
              cast, so truncation/sign loss cannot occur, and PROGRESS_BLOCKS \
              is a tiny constant whose usize -> f64 conversion is exact"
)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, AppState, GranularityMode};
use crate::ui::colors::{ACCENT_INFO, ACCENT_PRIMARY, ACCENT_WARNING, BG_2, FG_1, FG_2};
use crate::ui::styles::{key_hint_style, key_letter_style, state_color, state_indicator};

/// Number of blocks in the progress bar visualization (Contract §1.1).
const PROGRESS_BLOCKS: usize = 10;

/// Build the 10-block progress string (e.g., `[██████░░░░]`) for a ratio
/// in `[0.0, 1.0]`.
#[must_use]
fn progress_bar(ratio: f64) -> String {
    let clamped = ratio.clamp(0.0, 1.0);
    let filled = (clamped * PROGRESS_BLOCKS as f64).round() as usize;
    let empty = PROGRESS_BLOCKS.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Render the bottom status bar: playback state, progress bar, granularity
/// mode badge, and key hints (FR-005, Contract §1.1).
pub fn render_status_bar(frame: &mut Frame<'_>, app: &App, area: Rect) {
    // State indicator + label
    let sc = state_color(&app.state);
    let state_label = match &app.state {
        AppState::Idle => " IDLE ",
        AppState::Running { .. } => " RUNNING ",
        AppState::Done { .. } => " DONE ",
        AppState::Error(_) => " ERROR ",
        AppState::ConfirmQuit(_) => " QUIT? (y/n) ",
    };
    let state_span = Span::styled(
        format!("{}{state_label}", state_indicator(&app.state)),
        Style::new().fg(sc).add_modifier(Modifier::BOLD),
    );

    // Playback badge: Playing (info) vs Paused (warning)
    let playback_span = if app.playback.is_playing {
        Span::styled(" ▶ Playing", Style::new().fg(ACCENT_INFO).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" ⏸ Paused", Style::new().fg(ACCENT_WARNING))
    };

    // Progress: iteration/cap of the algorithm run
    let ratio =
        f64::from(app.iterations) / f64::from(app.params.iteration_cap.max(1));
    let progress_span = Span::styled(
        format!("  [{}] {:.0}%", progress_bar(ratio), ratio * 100.0),
        Style::new().fg(ACCENT_PRIMARY),
    );

    // Granularity badge: Mode: Phase (FG_1) vs Mode: Micro (ACCENT_INFO)
    let mode_span = match app.playback.granularity {
        GranularityMode::PhaseLevel => {
            Span::styled("  Mode: Phase", Style::new().fg(FG_1))
        }
        GranularityMode::MicroStep => {
            Span::styled("  Mode: Micro", Style::new().fg(ACCENT_INFO))
        }
    };

    // Dataset + quality info
    let info_span = Span::styled(
        format!("  Q={:.4}  γ={:.2}", app.quality, app.params.gamma),
        Style::new().fg(FG_2),
    );

    // Key hints
    let hint = |key: &str, desc: &str| {
        vec![
            Span::styled(format!(" {key}"), key_letter_style()),
            Span::styled(format!(":{desc}"), key_hint_style()),
        ]
    };
    let mut hints: Vec<Span<'_>> = vec![];
    let play_hint = if app.playback.is_playing {
        hint("Space", "Pause")
    } else {
        hint("Space", "Play")
    };
    hints.extend(play_hint);
    hints.extend(hint("n", "Step"));
    hints.extend(hint("t", "Mode"));
    hints.extend(hint("1-3", "Data"));
    hints.extend(hint("?", "Help"));
    hints.extend(hint("q", "Quit"));

    let mut line_spans = vec![
        state_span,
        playback_span,
        progress_span,
        mode_span,
        info_span,
    ];
    line_spans.push(Span::styled("  ", Style::new()));
    line_spans.extend(hints);

    let bg = Style::new().bg(BG_2);
    let line = Line::from(line_spans).style(bg);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_blocks() {
        assert_eq!(progress_bar(0.0), "░░░░░░░░░░");
        assert_eq!(progress_bar(1.0), "██████████");
        assert_eq!(progress_bar(0.5).chars().filter(|c| *c == '█').count(), 5);
        assert_eq!(progress_bar(0.6).chars().filter(|c| *c == '█').count(), 6);
    }
}
