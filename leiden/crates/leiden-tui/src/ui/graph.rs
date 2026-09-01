//! Graph topology panel widget.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, FocusPanel};
use crate::ui::colors::{BORDER_COLOR, FOCUS_COLOR, community_color};

/// Render the graph visualization panel.
pub fn render_graph_panel(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::GraphView;
    let border_color = if is_focused {
        FOCUS_COLOR
    } else {
        BORDER_COLOR
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Graph Topology ");

    if app.partition.is_empty() {
        let empty_msg = Paragraph::new("No active graph loaded or partition empty.")
            .block(block)
            .style(Style::default().fg(ratatui::style::Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_len = 0;
    let max_width = area.width.saturating_sub(4) as usize;

    for (node, comm) in &app.partition {
        let color = community_color(*comm);
        let node_str = format!("[{node}:{comm}] ");
        let node_len = node_str.len();

        if current_len + node_len > max_width && !current_line.is_empty() {
            lines.push(Line::from(std::mem::take(&mut current_line)));
            current_len = 0;
        }

        current_line.push(Span::styled(node_str, Style::default().fg(color)));
        current_len += node_len;
    }

    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
