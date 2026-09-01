//! Color palette and community hashing scheme.

use ratatui::style::Color;

/// Color palette used for hashing communities.
pub const COMMUNITY_PALETTE: [Color; 8] = [
    Color::Cyan,
    Color::Yellow,
    Color::Green,
    Color::Magenta,
    Color::Blue,
    Color::Red,
    Color::LightCyan,
    Color::LightYellow,
];

/// Get a deterministic color for a community id by hashing.
#[must_use]
pub const fn community_color(community_id: u32) -> Color {
    let index = (community_id as usize) % COMMUNITY_PALETTE.len();
    COMMUNITY_PALETTE[index]
}

/// Highlight color for focused panels.
pub const FOCUS_COLOR: Color = Color::Cyan;

/// Normal border color for unfocused panels.
pub const BORDER_COLOR: Color = Color::DarkGray;

/// Background header style color.
pub const HEADER_BG: Color = Color::Blue;
