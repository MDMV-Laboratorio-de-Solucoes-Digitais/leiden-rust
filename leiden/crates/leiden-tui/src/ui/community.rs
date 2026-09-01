//! Community table panel widget.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::app::{App, FocusPanel};
use crate::ui::colors::{BORDER_COLOR, FOCUS_COLOR, community_color};

/// Render the community statistics table panel.
pub fn render_community_panel(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::CommunityList;
    let border_color = if is_focused {
        FOCUS_COLOR
    } else {
        BORDER_COLOR
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Communities ");

    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("Size"),
        Cell::from("Int. Weight"),
        Cell::from("Total Degree"),
    ])
    .style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(ratatui::style::Color::White),
    );

    let summaries = app.community_summaries();
    let rows: Vec<Row<'_>> = summaries
        .iter()
        .enumerate()
        .map(|(idx, summary)| {
            let color = community_color(summary.id);
            let is_selected = idx == app.selected_community;
            let mut style = Style::default().fg(color);
            if is_selected {
                style = style.add_modifier(Modifier::REVERSED);
            }

            Row::new(vec![
                Cell::from(format!("{}", summary.id)),
                Cell::from(format!("{}", summary.size)),
                Cell::from(format!("{:.1}", summary.internal_weight)),
                Cell::from(format!("{:.1}", summary.total_degree)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(14),
        Constraint::Length(14),
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    frame.render_widget(table, area);
}
