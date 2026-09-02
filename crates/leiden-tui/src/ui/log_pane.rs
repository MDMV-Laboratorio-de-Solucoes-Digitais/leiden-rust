//! Log viewer panel widget.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, FocusPanel};
use crate::ui::colors::{BORDER_COLOR, FOCUS_COLOR};

fn level_color(line: &str) -> Color {
    if line.contains("[ERROR]") {
        Color::Red
    } else if line.contains("[WARN]") {
        Color::Yellow
    } else if line.contains("[INFO]") {
        Color::Green
    } else if line.contains("[DEBUG]") {
        Color::Blue
    } else {
        Color::DarkGray
    }
}

/// Render the log viewer pane.
pub fn render_log_pane(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::LogPane;
    let border_color = if is_focused {
        FOCUS_COLOR
    } else {
        BORDER_COLOR
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Logs ");

    let entries = app.log_ring.lock().map_or_else(
        |_| std::collections::VecDeque::new(),
        |ring| ring.entries().clone(),
    );

    let available_lines = area.height.saturating_sub(2) as usize;
    let skip_count = entries.len().saturating_sub(available_lines);

    let lines: Vec<Line<'_>> = entries
        .iter()
        .skip(skip_count)
        .map(|entry| {
            let color = level_color(entry);
            Line::from(Span::styled(entry.as_str(), Style::default().fg(color)))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
