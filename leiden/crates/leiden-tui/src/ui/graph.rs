//! Graph topology panel widget.

#![expect(
    clippy::cast_precision_loss,
    reason = "TUI math requires casting usize to f64, precision loss is negligible"
)]
#![expect(
    clippy::explicit_counter_loop,
    reason = "Iterating hashmap and need index"
)]
#![expect(
    clippy::suboptimal_flops,
    reason = "Readability preferred over mul_add for TUI"
)]
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::canvas::Canvas;

use crate::app::{App, FocusPanel};
use crate::ui::colors::{BORDER_COLOR, FOCUS_COLOR, community_color};

use std::collections::HashMap;

/// A spatial grid that tracks community centers and node positions.
#[derive(Debug, Default)]
pub struct CommunityGrid {
    /// Central coordinates for each community.
    pub community_centers: HashMap<u32, (f64, f64)>,
    /// Current rendered position of each node to maintain visual stability.
    pub node_positions: HashMap<String, (f64, f64)>,
}

impl CommunityGrid {
    /// Creates a new, empty `CommunityGrid`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates spatial coordinates for nodes based on their community assignment.
    pub fn calculate_coordinates(&mut self, partition: &[(String, u32)], area: Rect) {
        self.community_centers.clear();
        self.node_positions.clear();

        let mut communities: HashMap<u32, Vec<String>> = HashMap::new();
        for (node, comm) in partition {
            communities.entry(*comm).or_default().push(node.clone());
        }

        let num_comms = communities.len();
        if num_comms == 0 {
            return;
        }

        // Near-square grid: ceil(sqrt(n)) columns, ceil(n / cols) rows.
        // Integer math keeps the result exact without f64 -> usize casts.
        let mut cols = 1;
        while cols * cols < num_comms {
            cols += 1;
        }
        let rows = num_comms.div_ceil(cols);

        let cell_w = f64::from(area.width) / (cols as f64).max(1.0);
        let cell_h = f64::from(area.height) / (rows as f64).max(1.0);

        let mut comm_idx = 0;
        for (comm, nodes) in &communities {
            let row = comm_idx / cols;
            let col = comm_idx % cols;

            let cx = f64::from(area.x) + ((col as f64) * cell_w) + (cell_w / 2.0);
            let cy = f64::from(area.y) + ((row as f64) * cell_h) + (cell_h / 2.0);

            let _ = self.community_centers.insert(*comm, (cx, cy));

            let num_nodes = nodes.len();
            let radius = f64::min(cell_w, cell_h) * 0.3;

            for (i, node) in nodes.iter().enumerate() {
                if num_nodes == 1 {
                    let _ = self.node_positions.insert(node.clone(), (cx, cy));
                } else {
                    let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_nodes as f64);
                    let nx = cx + radius * angle.cos();
                    let ny = cy + radius * angle.sin();
                    let _ = self.node_positions.insert(node.clone(), (nx, ny));
                }
            }
            comm_idx += 1;
        }
    }
}

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
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    let mut grid = CommunityGrid::new();
    let inner_area = block.inner(area);
    grid.calculate_coordinates(&app.partition, inner_area);

    let canvas = Canvas::default()
        .block(block)
        .x_bounds([f64::from(area.x), f64::from(area.x + area.width)])
        .y_bounds([f64::from(area.y), f64::from(area.y + area.height)])
        .paint(move |ctx| {
            for (node, comm) in &app.partition {
                if let Some(&(x, y)) = grid.node_positions.get(node) {
                    let color = community_color(*comm);
                    ctx.print(
                        x,
                        y,
                        ratatui::text::Span::styled(node.clone(), Style::default().fg(color)),
                    );
                }
            }
        });

    frame.render_widget(canvas, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_grid_coordinate_calculation() -> Result<(), String> {
        let mut grid = CommunityGrid::new();
        let area = Rect::new(0, 0, 100, 100);
        let partition = vec![
            ("A".to_string(), 1),
            ("B".to_string(), 1),
            ("C".to_string(), 2),
        ];

        grid.calculate_coordinates(&partition, area);

        let Some(pos_a) = grid.node_positions.get("A") else {
            return Err("Node A not found in positions".to_string());
        };
        let Some(pos_b) = grid.node_positions.get("B") else {
            return Err("Node B not found in positions".to_string());
        };
        let Some(pos_c) = grid.node_positions.get("C") else {
            return Err("Node C not found in positions".to_string());
        };

        let dist = |p: &(f64, f64), q: &(f64, f64)| (p.0 - q.0).hypot(p.1 - q.1);
        let dist_same_comm = dist(pos_a, pos_b);
        let dist_across_comms = dist(pos_a, pos_c);

        if dist_same_comm >= dist_across_comms {
            return Err(
                "Nodes in the same community should be closer than nodes in different communities"
                    .to_string(),
            );
        }

        if grid.community_centers.is_empty() {
            return Err("Community centers should be calculated".to_string());
        }

        Ok(())
    }
}
