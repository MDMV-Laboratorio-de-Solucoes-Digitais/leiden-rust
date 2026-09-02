//! Graph visualization canvas widget with force-directed layout.
//!
//! Paints nodes as Unicode discs and edges as lines on a Ratatui `Canvas`,
//! using normalized virtual coordinates that re-project automatically on
//! terminal resize (FR-001, FR-002, FR-003, Contract §3.1).

use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::canvas::{Canvas, Line, Points};
use ratatui::widgets::{Block, Borders};

use crate::app::{App, FocusPanel};
use crate::ui::colors::{ACCENT_PRIMARY, FG_2, FG_3, community_color};

/// Node labels are displayed only when the dataset has at most 40 nodes
/// (Contract §3.1, CHK003).
const MAX_LABEL_NODES: usize = 40;

/// Resolve the color for one dataset edge: the shared community color when
/// both endpoints are clustered into the same community, dimmed `FG_3`
/// otherwise (unclustered graphs stay monochromatic, Contract §3.1).
fn edge_color(
    clustered: bool,
    community_of: &HashMap<&str, u32>,
    src: &str,
    tgt: &str,
) -> Color {
    if !clustered {
        return FG_3;
    }
    match (community_of.get(src), community_of.get(tgt)) {
        (Some(&c), Some(&d)) if c == d => community_color(c),
        _ => FG_3,
    }
}

/// Render the force-directed graph visualization canvas.
///
/// Nodes are painted as `●` discs in `FG_2` while unclustered (FR-002);
/// edges are painted as `FG_3` dimmed lines. Node ID labels appear only
/// when the total node count is ≤ 40. The canvas footer shows the active
/// dataset title with an `(Active)` badge in `ACCENT_PRIMARY`.
///
/// The simulation positions live in normalized `[0.05, 0.95]` virtual
/// space, and the canvas maps `[0.0, 1.0]` bounds onto the rendered
/// `Rect`, so terminal resizes re-project coordinates automatically
/// without any state mutation (Contract §4.2, CHK027).
pub fn render_graph_canvas(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::GraphView;
    let border_color = if is_focused { ACCENT_PRIMARY } else { FG_3 };

    let footer = format!(
        " Dataset: [{}] (Active) · {} nodes · {} edges ",
        app.dataset_title,
        app.partition.len(),
        app.dataset_edges.len()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(" GRAPH VISUALIZATION ")
        .title_style(Style::new().fg(FG_2))
        .title_bottom(ratatui::text::Line::from(Span::styled(
            footer,
            Style::new().fg(ACCENT_PRIMARY),
        )));

    if app.partition.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("No dataset loaded.")
            .block(block)
            .style(Style::new().fg(FG_2));
        frame.render_widget(empty, area);
        return;
    }

    let show_labels = app.partition.len() <= MAX_LABEL_NODES;

    // Clustering colors activate once the algorithm is running or done;
    // the initial state stays monochromatic `FG_2` (FR-002, Contract §3.1).
    let clustered = matches!(
        app.state,
        crate::app::AppState::Running { .. } | crate::app::AppState::Done { .. }
    );
    let community_of: HashMap<&str, u32> =
        app.partition.iter().map(|(n, c)| (n.as_str(), *c)).collect();

    let positions = &app.simulation.node_positions;

    let canvas = Canvas::default()
        .block(block)
        .x_bounds([0.0, 1.0])
        .y_bounds([0.0, 1.0])
        .paint(|ctx| {
            // Edges: intra-community edges colorize with the community
            // color; inter-community edges stay dimmed `FG_3` (Contract §3.1).
            for (src, tgt) in &app.dataset_edges {
                if let (Some(&p1), Some(&p2)) = (positions.get(src), positions.get(tgt)) {
                    let color = edge_color(clustered, &community_of, src, tgt);
                    ctx.draw(&Line {
                        x1: p1.x,
                        y1: p1.y,
                        x2: p2.x,
                        y2: p2.y,
                        color,
                    });
                }
            }

            // Nodes: monochromatic discs while unclustered (FR-002),
            // `COMMUNITY_COLORS[comm_id % 12]` once clustered — drawn as
            // one `Points` group per community so each gets its color.
            if clustered {
                let mut grouped: HashMap<u32, Vec<(f64, f64)>> = HashMap::new();
                for (node, comm) in &app.partition {
                    if let Some(p) = positions.get(node) {
                        grouped.entry(*comm).or_default().push((p.x, p.y));
                    }
                }
                for (comm, coords) in &grouped {
                    ctx.draw(&Points {
                        coords,
                        color: community_color(*comm),
                    });
                }
            } else {
                let points: Vec<(f64, f64)> = app
                    .partition
                    .iter()
                    .filter_map(|(node, _comm)| positions.get(node))
                    .map(|p| (p.x, p.y))
                    .collect();
                ctx.draw(&Points {
                    coords: &points,
                    color: FG_2,
                });
            }

            // Node ID labels (only for small graphs)
            if show_labels {
                for (node, comm) in &app.partition {
                    if let Some(p) = positions.get(node) {
                        let label_color = if clustered {
                            community_color(*comm)
                        } else {
                            FG_2
                        };
                        ctx.print(
                            p.x + 0.015,
                            p.y,
                            Span::styled(node.clone(), Style::new().fg(label_color)),
                        );
                    }
                }
            }
        });

    frame.render_widget(canvas, area);
}
