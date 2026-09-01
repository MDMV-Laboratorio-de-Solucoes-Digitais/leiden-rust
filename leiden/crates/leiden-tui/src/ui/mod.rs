//! UI layout, components, and rendering dispatcher.

pub mod colors;
pub mod community;
pub mod graph;
pub mod help;
pub mod log_pane;
pub mod status_bar;
pub mod styles;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{App, FocusPanel};
use crate::ui::community::render_community_panel;
use crate::ui::graph::render_graph_panel;
use crate::ui::help::render_help_modal;
use crate::ui::log_pane::render_log_pane;
use crate::ui::status_bar::render_status_bar;
use crate::ui::styles::{LayoutMode, layout_mode};

/// Render the complete TUI interface for the current application state.
pub fn render(frame: &mut Frame<'_>, app: &App) {
    let size = frame.area();

    // 1. Root layout: main area + single line status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(1)])
        .split(size);

    let main_area = chunks[0];
    let status_area = chunks[1];

    let mode = layout_mode(size.width);

    match mode {
        LayoutMode::Full | LayoutMode::Compact => {
            render_full_compact(frame, app, main_area, mode);
        }
        LayoutMode::Stacked => {
            render_stacked(frame, app, main_area);
        }
        LayoutMode::Minimal => {
            render_minimal(frame, app, main_area);
        }
    }

    // Status bar at bottom
    render_status_bar(frame, app, status_area);

    // Help overlay if open
    if app.visibility.help_open {
        render_help_modal(frame, size);
    }
}

fn render_full_compact(frame: &mut Frame<'_>, app: &App, area: Rect, mode: LayoutMode) {
    let show_g = app.visibility.show_graph;
    let show_l = app.visibility.show_log;

    let (comm_pct, graph_pct) = if mode == LayoutMode::Full {
        (40, 60)
    } else {
        (50, 50)
    };

    match (show_g, show_l) {
        (true, true) => {
            // Default 3-panel: Top 65% (comm + graph side-by-side), Bottom 35% (log)
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(area);

            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(comm_pct),
                    Constraint::Percentage(graph_pct),
                ])
                .split(v_chunks[0]);

            render_community_panel(frame, app, h_chunks[0]);
            render_graph_panel(frame, app, h_chunks[1]);
            render_log_pane(frame, app, v_chunks[1]);
        }
        (false, true) => {
            // Community panel full width top (65%), log pane bottom (35%)
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(area);

            render_community_panel(frame, app, v_chunks[0]);
            render_log_pane(frame, app, v_chunks[1]);
        }
        (true, false) => {
            // Community + graph side-by-side, full height
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(comm_pct),
                    Constraint::Percentage(graph_pct),
                ])
                .split(area);

            render_community_panel(frame, app, h_chunks[0]);
            render_graph_panel(frame, app, h_chunks[1]);
        }
        (false, false) => {
            // Community panel full screen
            render_community_panel(frame, app, area);
        }
    }
}

fn render_stacked(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let show_g = app.visibility.show_graph;
    let show_l = app.visibility.show_log;

    match (show_g, show_l) {
        (true, true) => {
            // Community (40%) → Graph (30%) → Log (30%)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ])
                .split(area);

            render_community_panel(frame, app, chunks[0]);
            render_graph_panel(frame, app, chunks[1]);
            render_log_pane(frame, app, chunks[2]);
        }
        (false, true) => {
            // Community (60%) → Log (40%)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);

            render_community_panel(frame, app, chunks[0]);
            render_log_pane(frame, app, chunks[1]);
        }
        (true, false) => {
            // Community (50%) → Graph (50%)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            render_community_panel(frame, app, chunks[0]);
            render_graph_panel(frame, app, chunks[1]);
        }
        (false, false) => {
            // Community full screen
            render_community_panel(frame, app, area);
        }
    }
}

fn render_minimal(frame: &mut Frame<'_>, app: &App, area: Rect) {
    // Only the focused panel is visible. Community panel is fallback if focused panel is hidden.
    let effective_focus = match app.focus {
        FocusPanel::GraphView if app.visibility.show_graph => FocusPanel::GraphView,
        FocusPanel::LogPane if app.visibility.show_log => FocusPanel::LogPane,
        _ => FocusPanel::CommunityList,
    };

    match effective_focus {
        FocusPanel::CommunityList => render_community_panel(frame, app, area),
        FocusPanel::GraphView => render_graph_panel(frame, app, area),
        FocusPanel::LogPane => render_log_pane(frame, app, area),
    }
}
