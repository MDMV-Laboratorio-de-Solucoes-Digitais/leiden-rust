//! Keyboard shortcut help overlay widget.
//!
//! Renders the modal overlay for interactive key bindings (FR-017).

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::ui::colors::{ACCENT_PRIMARY, BG_1, FG_0, FG_1, FG_2, FG_3};

/// Render the interactive help modal overlay.
pub fn render_help_modal(frame: &mut Frame<'_>, area: Rect) {
    let modal_area = centered_rect(64, 70, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_PRIMARY))
        .style(Style::default().bg(BG_1))
        .title(" KEY BINDINGS ")
        .title_style(Style::default().fg(FG_0).add_modifier(Modifier::BOLD));

    let key_style = Style::default()
        .fg(ACCENT_PRIMARY)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(FG_1);
    let section_style = Style::default().fg(FG_2).add_modifier(Modifier::BOLD);
    let footer_style = Style::default()
        .fg(FG_3)
        .add_modifier(Modifier::DIM | Modifier::UNDERLINED);

    let text = vec![
        // General category
        Line::from(Span::styled("General", section_style)),
        Line::from(vec![
            Span::styled("  q / Ctrl+C   ", key_style),
            Span::styled("Quit application", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  r            ", key_style),
            Span::styled("Restart algorithm run", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  s            ", key_style),
            Span::styled("Step single iteration (paused mode)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  p            ", key_style),
            Span::styled("Pause / Resume execution", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", key_style),
            Span::styled("Toggle help overlay", desc_style),
        ]),
        Line::from(""),
        // Panels category
        Line::from(Span::styled("Panels", section_style)),
        Line::from(vec![
            Span::styled("  g            ", key_style),
            Span::styled("Toggle graph view panel", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  l            ", key_style),
            Span::styled("Toggle log pane", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Tab          ", key_style),
            Span::styled("Switch panel focus", desc_style),
        ]),
        Line::from(""),
        // Navigation category
        Line::from(Span::styled("Navigation", section_style)),
        Line::from(vec![
            Span::styled("  ↑ / ↓        ", key_style),
            Span::styled("Select community in table", desc_style),
        ]),
        Line::from(""),
        // Footer hint
        Line::from(Span::styled("Press any key to close", footer_style)),
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
