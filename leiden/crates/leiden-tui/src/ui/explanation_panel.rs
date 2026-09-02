//! 3-tier explanation panel widget: Step Headline, Plain-English Analogy,
//! and Live Stat Badges (FR-004, Contract explanation-content.md).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::ui::colors::{ACCENT_INFO, ACCENT_SUCCESS, BG_1, FG_0, FG_1, FG_2};
use crate::ui::styles::unfocused_block;

/// Render the 3-tier explanation panel for the current explanation state.
///
/// Tier 1 renders the step headline in bold `FG_0`; Tier 2 renders the
/// analogy word-wrapped to the panel width in `FG_1`; Tier 3 renders up
/// to three live stat badges (Phase, Communities, Progress) with `FG_2`
/// labels and `ACCENT_INFO` values (CHK020). When the algorithm has
/// finished, the phase badge is styled `ACCENT_SUCCESS` instead.
pub fn render_explanation_panel(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = unfocused_block("EXPLANATION");
    let inner = block.inner(area);

    let explanation = &app.explanation;

    // Tier 1 — bold step headline.
    let headline = Line::from(Span::styled(
        explanation.headline.clone(),
        Style::new().fg(FG_0).add_modifier(Modifier::BOLD),
    ));

    // Tier 2 — plain-English analogy, word-wrapped to the panel width.
    let analogy_lines: Vec<Line<'static>> = explanation
        .wrapped_analogy_lines(usize::from(inner.width))
        .into_iter()
        .map(|line| Line::from(Span::styled(line, Style::new().fg(FG_1))))
        .collect();

    // Tier 3 — live stat badges (Phase, Communities, Progress).
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "progress clamped to 0..100 before rounding"
    )]
    let progress = (explanation.phase_progress.clamp(0.0, 1.0) * 100.0).round() as u64;

    let phase_color = if explanation.phase_name == "Finished" {
        ACCENT_SUCCESS
    } else {
        ACCENT_INFO
    };

    let badges = Line::from(vec![
        Span::styled("Phase: ", Style::new().fg(FG_2)),
        Span::styled(explanation.phase_name.clone(), Style::new().fg(phase_color)),
        Span::styled("  Communities: ", Style::new().fg(FG_2)),
        Span::styled(
            explanation.community_count.to_string(),
            Style::new().fg(ACCENT_INFO),
        ),
        Span::styled("  Progress: ", Style::new().fg(FG_2)),
        Span::styled(format!("{progress}%"), Style::new().fg(ACCENT_INFO)),
    ]);

    let mut lines: Vec<Line<'static>> = Vec::with_capacity(analogy_lines.len() + 3);
    lines.push(headline);
    lines.push(Line::from(""));
    lines.extend(analogy_lines);
    lines.push(Line::from(""));
    lines.push(badges);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::new().bg(BG_1));
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn renders_headline_and_badges() {
        let app = App::new_idle();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let _ = terminal.draw(|f| render_explanation_panel(f, &app, f.area()));
        let buffer = terminal.backend().buffer();
        let debug = format!("{buffer:?}");
        assert!(debug.contains("EXPLANATION"));
        assert!(debug.contains("Phase:"));
    }
}
