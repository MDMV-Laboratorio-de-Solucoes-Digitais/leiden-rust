//! Community table panel widget.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Cell, Row, Table};

use crate::app::{App, FocusPanel};
use crate::ui::colors::community_color;
use crate::ui::styles::{header_style, normal_row_style, panel_block, selected_row_style};

/// Render the community statistics table panel.
pub fn render_community_panel(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::CommunityList;
    let block = panel_block("Communities", is_focused);

    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("Community"),
        Cell::from("Size"),
        Cell::from("IntW"),
        Cell::from("TDeg"),
    ])
    .style(header_style());

    let summaries = app.community_summaries();
    let rows: Vec<Row<'_>> = summaries
        .iter()
        .enumerate()
        .map(|(idx, summary)| {
            let color = community_color(summary.id);
            let is_selected = idx == app.selected_community;
            let base_style = if is_selected {
                selected_row_style()
            } else {
                normal_row_style()
            };

            // Community color block ██ preserves its community color even when selected
            let color_block_cell = Cell::from("██").style(Style::default().fg(color));

            Row::new(vec![
                Cell::from(format!("{}", summary.id)),
                color_block_cell,
                Cell::from(format!("{}", summary.size)),
                Cell::from(format!("{:.1}", summary.internal_weight)),
                Cell::from(format!("{:.1}", summary.total_degree)),
            ])
            .style(base_style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(11),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    frame.render_widget(table, area);
}
