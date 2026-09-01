//! UI layout, components, and rendering dispatcher.

pub mod colors;
pub mod community;
pub mod graph;
pub mod log_pane;
pub mod status_bar;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::App;
use crate::ui::community::render_community_panel;
use crate::ui::graph::render_graph_panel;
use crate::ui::log_pane::render_log_pane;
use crate::ui::status_bar::render_status_bar;

/// Render the complete TUI interface for the current application state.
pub fn render(frame: &mut Frame<'_>, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(size);

    let main_area = chunks[0];
    let status_area = chunks[1];

    let mut horiz_constraints = Vec::new();
    horiz_constraints.push(Constraint::Percentage(40)); // Community list
    if app.visibility.show_graph && app.visibility.show_log {
        horiz_constraints.push(Constraint::Percentage(30)); // Graph
        horiz_constraints.push(Constraint::Percentage(30)); // Logs
    } else if app.visibility.show_graph {
        horiz_constraints.push(Constraint::Percentage(60)); // Graph only
    } else if app.visibility.show_log {
        horiz_constraints.push(Constraint::Percentage(60)); // Logs only
    }

    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(horiz_constraints)
        .split(main_area);

    let mut chunk_idx = 0;

    // 1. Community panel
    if let Some(&area) = panel_chunks.get(chunk_idx) {
        render_community_panel(frame, app, area);
        chunk_idx += 1;
    }

    // 2. Graph panel (if visible)
    if app.visibility.show_graph
        && let Some(&area) = panel_chunks.get(chunk_idx)
    {
        render_graph_panel(frame, app, area);
        chunk_idx += 1;
    }

    // 3. Log pane (if visible)
    if app.visibility.show_log
        && let Some(&area) = panel_chunks.get(chunk_idx)
    {
        render_log_pane(frame, app, area);
    }

    // 4. Status bar
    render_status_bar(frame, app, status_area);

    // 5. Help overlay if open
    if app.visibility.help_open {
        render_help_modal(frame, size);
    }
}

fn render_help_modal(frame: &mut Frame<'_>, area: Rect) {
    let modal_area = centered_rect(60, 50, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Help / Key Bindings ");

    let text = vec![
        ratatui::text::Line::from(ratatui::text::Span::styled(
            "Leiden Interactive Community Detection TUI",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from("  q / Ctrl+C : Quit application"),
        ratatui::text::Line::from("  r          : Restart algorithm run"),
        ratatui::text::Line::from("  s          : Step single iteration (paused mode)"),
        ratatui::text::Line::from("  p          : Pause / resume auto-iteration"),
        ratatui::text::Line::from("  g          : Toggle graph view panel"),
        ratatui::text::Line::from("  l          : Toggle log pane"),
        ratatui::text::Line::from("  Tab        : Cycle focused panel"),
        ratatui::text::Line::from("  ↑ / ↓      : Select community in table"),
        ratatui::text::Line::from("  ?          : Toggle this help overlay"),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, modal_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
