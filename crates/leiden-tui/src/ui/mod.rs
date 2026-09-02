//! UI layout, components, and rendering dispatcher.

pub mod colors;
pub mod community;
pub mod explanation_panel;
pub mod graph;
pub mod graph_canvas;
pub mod log_pane;
pub mod status_bar;
pub mod styles;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::{App, AppState};
use crate::ui::colors::{ACCENT_PRIMARY, ACCENT_WARNING, BG_0, FG_0, FG_1, FG_2};
use crate::ui::community::{render_community_panel, render_community_summary_table};
use crate::ui::explanation_panel::render_explanation_panel;
use crate::ui::graph::render_graph_panel;
use crate::ui::graph_canvas::render_graph_canvas;
use crate::ui::log_pane::render_log_pane;
use crate::ui::status_bar::render_status_bar;

/// Guards layout against undersized terminal viewports (FR-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalDimensionGuard {
    /// Minimum supported terminal columns (80).
    pub min_columns: u16,
    /// Minimum supported terminal rows (24).
    pub min_rows: u16,
}

impl Default for TerminalDimensionGuard {
    fn default() -> Self {
        Self::standard()
    }
}

impl TerminalDimensionGuard {
    /// Standard guard with 80×24 minimum requirement.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            min_columns: 80,
            min_rows: 24,
        }
    }

    /// Check if given terminal dimensions are sufficient.
    #[must_use]
    pub const fn is_valid(&self, width: u16, height: u16) -> bool {
        width >= self.min_columns && height >= self.min_rows
    }

    /// Check if given terminal area dimensions are sufficient.
    #[must_use]
    pub const fn is_area_valid(&self, area: Rect) -> bool {
        self.is_valid(area.width, area.height)
    }

    /// Render the "TERMINAL TOO SMALL" modal overlay (FR-007, Contract §4.1).
    pub fn render_dimension_overlay(frame: &mut Frame<'_>, area: Rect, guard: &Self) {
        let is_valid = guard.is_area_valid(area);
        if is_valid {
            return;
        }

        let modal_area = centered_rect(58, 29, area);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(ACCENT_WARNING))
            .title(" TERMINAL TOO SMALL ");

        let text = vec![
            ratatui::text::Line::from(ratatui::text::Span::styled(
                format!("Current size: {} × {}", area.width, area.height),
                Style::default().fg(FG_1),
            )),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(ratatui::text::Span::styled(
                format!(
                    "Minimum required: {} × {}",
                    guard.min_columns, guard.min_rows
                ),
                Style::default().fg(FG_1),
            )),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(ratatui::text::Span::styled(
                "Please expand your terminal window.",
                Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
            )),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(ratatui::text::Span::styled(
                "Resize back to ≥ 80×24 to restore the interactive UI.",
                Style::default().fg(FG_2),
            )),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().bg(BG_0));
        frame.render_widget(paragraph, modal_area);
    }
}

/// Standard dimension guard for the visual explanation layout.
const DIMENSION_GUARD: TerminalDimensionGuard = TerminalDimensionGuard::standard();

/// Render the complete TUI interface for the current application state.
///
/// Implements the two-stage layout split from Contract §1.1:
/// Stage 1 partitions the root into Main Content + Status Bar;
/// Stage 2 partitions the Main Content into the Explanation Panel (35%)
/// and the Graph Visualization Canvas (65%). When the terminal is below
/// the 80×24 minimum, normal rendering is replaced by the
/// "TERMINAL TOO SMALL" modal overlay (FR-007).
pub fn render(frame: &mut Frame<'_>, app: &App) {
    let size = frame.area();

    // Dimension guard: suspend normal rendering below 80×24
    if !DIMENSION_GUARD.is_area_valid(size) {
        TerminalDimensionGuard::render_dimension_overlay(frame, size, &DIMENSION_GUARD);
        return;
    }

    // Stage 1: Main content + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(23), Constraint::Length(1)])
        .split(size);

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Stage 2: Explanation panel (35%) + graph canvas (65%)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_area);

    render_explanation_panel(frame, app, main_chunks[0]);
    render_graph_canvas(frame, app, main_chunks[1]);
    render_status_bar(frame, app, status_area);

    // Completion summary overlay when the algorithm has finished (T033)
    if matches!(app.state, AppState::Done { .. }) {
        let summary_area = centered_rect_fixed(46, 12, size);
        frame.render_widget(Clear, summary_area);
        render_community_summary_table(frame, app, summary_area);
    }

    // Help overlay if open
    if app.visibility.help_open {
        render_help_modal(frame, size);
    }
}

/// Render the legacy 3-panel dashboard layout (community, graph, log).
///
/// Retained for the community summary table and diagnostics; the visual
/// explanation mode uses [`render`] instead.
pub fn render_dashboard(frame: &mut Frame<'_>, app: &App) {
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

/// Render the centered keybinding help modal (50×14, Contract §2.1).
pub fn render_help_modal(frame: &mut Frame<'_>, area: Rect) {
    let modal_area = centered_rect_fixed(50, 14, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_PRIMARY))
        .title(" Help / Key Bindings ");

    let key_style = Style::default()
        .fg(ACCENT_PRIMARY)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(FG_1);

    let bind = |key: &str, desc: &str| {
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(format!("  {key:<14}"), key_style),
            ratatui::text::Span::styled(desc.to_string(), desc_style),
        ])
    };

    let text = vec![
        ratatui::text::Line::from(ratatui::text::Span::styled(
            "Leiden Interactive Community Detection TUI",
            Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
        )),
        ratatui::text::Line::from(""),
        bind("Space", "Play / pause auto-stepping"),
        bind("n / Right", "Advance one step"),
        bind("t", "Toggle granularity (Phase/Micro)"),
        bind("1 / 2 / 3", "Load preset dataset"),
        bind("r", "Restart explanation run"),
        bind("g / l", "Toggle graph / log panels"),
        bind("Tab", "Cycle focused panel"),
        bind("? / Esc", "Toggle this help overlay"),
        bind("q / Ctrl+C", "Quit application"),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().bg(BG_0));
    frame.render_widget(paragraph, modal_area);
}

/// Build a centered `width × height` modal `Rect` inside `r`.
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
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
