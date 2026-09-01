//! Log viewer panel widget.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, FocusPanel};
use crate::ui::colors::FG_1;
use crate::ui::styles::{
    log_debug_style, log_error_style, log_info_style, log_trace_style, log_warn_style, panel_block,
};

#[expect(
    clippy::option_if_let_else,
    reason = "chained prefix matching is clearer with if let sequence than nested map_or_else"
)]
fn format_log_line(line: &str) -> Line<'_> {
    if let Some(rest) = line.strip_prefix("[ERROR]") {
        Line::from(vec![
            Span::styled("[ERROR]", log_error_style()),
            Span::styled(rest.to_string(), Style::default().fg(FG_1)),
        ])
    } else if let Some(rest) = line.strip_prefix("[WARN]") {
        Line::from(vec![
            Span::styled("[WARN]", log_warn_style()),
            Span::styled(rest.to_string(), Style::default().fg(FG_1)),
        ])
    } else if let Some(rest) = line.strip_prefix("[INFO]") {
        Line::from(vec![
            Span::styled("[INFO]", log_info_style()),
            Span::styled(rest.to_string(), Style::default().fg(FG_1)),
        ])
    } else if let Some(rest) = line.strip_prefix("[DEBUG]") {
        Line::from(vec![
            Span::styled("[DEBUG]", log_debug_style()),
            Span::styled(rest.to_string(), Style::default().fg(FG_1)),
        ])
    } else if let Some(rest) = line.strip_prefix("[TRACE]") {
        Line::from(vec![
            Span::styled("[TRACE]", log_trace_style()),
            Span::styled(rest.to_string(), Style::default().fg(FG_1)),
        ])
    } else {
        Line::from(Span::styled(line.to_string(), log_debug_style()))
    }
}

/// Render the log viewer pane.
pub fn render_log_pane(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::LogPane;
    let block = panel_block("Logs", is_focused);

    let entries = app.log_ring.lock().map_or_else(
        |_| std::collections::VecDeque::new(),
        |ring| ring.entries().clone(),
    );

    let available_lines = area.height.saturating_sub(2) as usize;
    let skip_count = entries.len().saturating_sub(available_lines);

    let lines: Vec<Line<'_>> = entries
        .iter()
        .skip(skip_count)
        .map(|entry| format_log_line(entry.as_str()))
        .collect();

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
