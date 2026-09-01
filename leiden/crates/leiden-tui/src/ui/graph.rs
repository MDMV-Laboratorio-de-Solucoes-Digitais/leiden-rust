//! Graph topology panel widget.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, FocusPanel};
use crate::ui::colors::{FG_2, FG_3, community_color};
use crate::ui::styles::{GRAPH_NODE, panel_block};

/// Render the graph visualization panel.
pub fn render_graph_panel(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::GraphView;
    let block = panel_block("Graph Topology", is_focused);

    if app.partition.is_empty() {
        let empty_msg = Paragraph::new("No active graph loaded or partition empty.")
            .block(block)
            .style(Style::default().fg(FG_3));
        frame.render_widget(empty_msg, area);
        return;
    }

    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_len = 0;
    let max_width = area.width.saturating_sub(4) as usize;

    for (node, comm) in &app.partition {
        let color = community_color(*comm);
        let node_text = format!(" {node}:{comm} ");
        // Approximate width: 1 symbol (●) + node_text.len()
        let item_len = 1 + node_text.len();

        if current_len + item_len > max_width && !current_line.is_empty() {
            lines.push(Line::from(std::mem::take(&mut current_line)));
            current_len = 0;
        }

        // Render community circle ● in community color
        current_line.push(Span::styled(GRAPH_NODE, Style::default().fg(color)));
        current_line.push(Span::styled(node_text, Style::default().fg(FG_2)));
        current_len += item_len;
    }

    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
