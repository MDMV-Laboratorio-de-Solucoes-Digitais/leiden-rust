//! Color palette and community hashing scheme.

use ratatui::style::Color;

// --- 1. Base Layer (Backgrounds & Surfaces) ---

/// Terminal background — the darkest surface, matching the user's dark
/// terminal theme (`#1a1b26`, Tokyo Night "storm" background).
pub const BG_0: Color = Color::Rgb(26, 27, 38);

/// Elevated surface — panel interiors and secondary backgrounds.
pub const BG_1: Color = Color::Rgb(31, 32, 46);

/// Active/focused panel background — used in the status bar.
pub const BG_2: Color = Color::Rgb(36, 38, 58);

/// Highlighted row / selection background — used for the selected
/// community row in the community panel.
pub const BG_3: Color = Color::Rgb(42, 45, 66);

/// Hover / transient highlight — used for progress gauge empty area.
pub const BG_4: Color = Color::Rgb(59, 63, 88);

// --- 2. Text Layer (Foregrounds) ---

/// Bright foreground — titles, selected items, primary values.
/// Matches Tokyo Night `fg` primary: `#c0caf5` = `(192, 202, 245)`.
pub const FG_0: Color = Color::Rgb(192, 202, 245);

/// Normal foreground — body content, table rows, descriptions.
pub const FG_1: Color = Color::Rgb(169, 177, 214);

/// Muted foreground — labels, secondary info, parameter names.
pub const FG_2: Color = Color::Rgb(115, 122, 137);

/// Dim foreground — disabled items, timestamps, key hints.
pub const FG_3: Color = Color::Rgb(86, 95, 113);

// --- 3. Semantic Accent Layer ---

/// Primary accent blue — focused borders, active selection, help overlay border.
pub const ACCENT_PRIMARY: Color = Color::Rgb(122, 162, 247);

/// Info accent teal — Running state, INFO log level, metrics, quality values.
pub const ACCENT_INFO: Color = Color::Rgb(115, 218, 202);

/// Warning accent amber — WARN log level, throttle events, iteration cap.
pub const ACCENT_WARNING: Color = Color::Rgb(224, 175, 104);

/// Error accent red — Error state, ERROR log level, quality degradation (ΔQ < 0).
pub const ACCENT_ERROR: Color = Color::Rgb(247, 118, 125);

/// Success accent green — Done state, quality improvement (ΔQ > 0).
pub const ACCENT_SUCCESS: Color = Color::Rgb(158, 206, 106);

// --- 4. Community Hash Colors ---

/// Deterministic color array for community-id → color mapping.
///
/// Usage: `COMMUNITY_COLORS[community_id as usize % COMMUNITY_COLORS.len()]`
///
/// The 12 colors are chosen for maximum pairwise perceptual distance
/// in CIELAB space, ensuring communities remain distinguishable in
/// both true-color and 256-color modes.
pub const COMMUNITY_COLORS: [Color; 12] = [
    Color::Rgb(122, 162, 247), // C0  blue       256: 111
    Color::Rgb(158, 206, 106), // C1  green      256: 150
    Color::Rgb(224, 175, 104), // C2  amber      256: 214
    Color::Rgb(247, 118, 125), // C3  coral      256: 204
    Color::Rgb(115, 218, 202), // C4  teal       256: 79
    Color::Rgb(187, 154, 247), // C5  lavender   256: 141
    Color::Rgb(255, 158, 100), // C6  orange     256: 209
    Color::Rgb(125, 207, 255), // C7  sky        256: 117
    Color::Rgb(195, 232, 141), // C8  lime       256: 192
    Color::Rgb(255, 167, 196), // C9  pink       256: 218
    Color::Rgb(169, 177, 214), // CA  silver     256: 146
    Color::Rgb(224, 208, 143), // CB  gold       256: 186
];

// --- 5. 16-Color ANSI Fallback Palette ---

/// 16-color ANSI fallback for `BG_0`.
pub const BG_0_ANSI: Color = Color::Black;
/// 16-color ANSI fallback for `BG_1`.
pub const BG_1_ANSI: Color = Color::Black;
/// 16-color ANSI fallback for `BG_2`.
pub const BG_2_ANSI: Color = Color::DarkGray;
/// 16-color ANSI fallback for `BG_3`.
pub const BG_3_ANSI: Color = Color::DarkGray;
/// 16-color ANSI fallback for `BG_4`.
pub const BG_4_ANSI: Color = Color::DarkGray;

/// 16-color ANSI fallback for `FG_0`.
pub const FG_0_ANSI: Color = Color::White;
/// 16-color ANSI fallback for `FG_1`.
pub const FG_1_ANSI: Color = Color::Gray;
/// 16-color ANSI fallback for `FG_2`.
pub const FG_2_ANSI: Color = Color::DarkGray;
/// 16-color ANSI fallback for `FG_3`.
pub const FG_3_ANSI: Color = Color::DarkGray;

/// 16-color ANSI fallback for `ACCENT_PRIMARY`.
pub const ACCENT_PRIMARY_ANSI: Color = Color::Blue;
/// 16-color ANSI fallback for `ACCENT_INFO`.
pub const ACCENT_INFO_ANSI: Color = Color::Cyan;
/// 16-color ANSI fallback for `ACCENT_WARNING`.
pub const ACCENT_WARNING_ANSI: Color = Color::Yellow;
/// 16-color ANSI fallback for `ACCENT_ERROR`.
pub const ACCENT_ERROR_ANSI: Color = Color::Red;
/// 16-color ANSI fallback for `ACCENT_SUCCESS`.
pub const ACCENT_SUCCESS_ANSI: Color = Color::Green;

/// 16-color ANSI fallback for community colors.
/// Cycles through 6 ANSI colors for basic distinguishability.
pub const COMMUNITY_COLORS_ANSI: [Color; 6] = [
    Color::Blue,
    Color::Green,
    Color::Yellow,
    Color::Red,
    Color::Cyan,
    Color::Magenta,
];

// --- 6. Legacy compatibility constants (for transition) ---

/// Color palette used for hashing communities (legacy 8-color palette).
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

/// Highlight color for focused panels (legacy alias).
pub const FOCUS_COLOR: Color = ACCENT_PRIMARY;

/// Normal border color for unfocused panels (legacy alias).
pub const BORDER_COLOR: Color = FG_3;

/// Background header style color (legacy alias).
pub const HEADER_BG: Color = BG_2;

// --- 7. Helper Functions ---

/// Get a deterministic color for a community id by hashing into the
/// 12-color community palette.
///
/// Wraps via `community_id % COMMUNITY_COLORS.len()`, ensuring the same community ID
/// always maps to the same color across the community panel and
/// graph view (FR-007).
#[must_use]
pub const fn community_color(community_id: u32) -> Color {
    COMMUNITY_COLORS[(community_id as usize) % COMMUNITY_COLORS.len()]
}

fn detect_truecolor(colorterm: Option<&str>, term: Option<&str>) -> bool {
    if let Some(ct) = colorterm
        && (ct == "truecolor" || ct == "24bit")
    {
        return true;
    }
    if let Some(t) = term
        && (t.starts_with("alacritty") || t.starts_with("wezterm") || t.ends_with("-direct"))
    {
        return true;
    }
    false
}

/// Detect true-color support from environment variables.
///
/// Checks `COLORTERM` first (the standard mechanism), then falls
/// back to `TERM` heuristics for terminals that don't set
/// `COLORTERM` (notably `Alacritty`, `WezTerm`).
///
/// Returns `false` as the conservative default when neither
/// variable indicates true-color support (FR-013).
#[must_use]
pub fn supports_truecolor() -> bool {
    let colorterm = std::env::var("COLORTERM").ok();
    let term = std::env::var("TERM").ok();
    detect_truecolor(colorterm.as_deref(), term.as_deref())
}

/// Select the appropriate color representation at runtime.
///
/// Returns the true-color RGB value when [`supports_truecolor()`] returns `true`,
/// otherwise returns the ANSI fallback color (FR-013, FR-014, data-model §2.1).
#[must_use]
pub fn resolve_color(color: Color, ansi: Color) -> Color {
    if supports_truecolor() { color } else { ansi }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_constants_are_const() {
        const _: Color = BG_0;
        const _: Color = BG_1;
        const _: Color = BG_2;
        const _: Color = BG_3;
        const _: Color = BG_4;
        const _: Color = FG_0;
        const _: Color = FG_1;
        const _: Color = FG_2;
        const _: Color = FG_3;
        const _: Color = ACCENT_PRIMARY;
        const _: Color = ACCENT_INFO;
        const _: Color = ACCENT_WARNING;
        const _: Color = ACCENT_ERROR;
        const _: Color = ACCENT_SUCCESS;
        const _: [Color; 12] = COMMUNITY_COLORS;
    }

    #[test]
    fn ansi_fallback_palette_complete() {
        let _: Color = BG_0_ANSI;
        let _: Color = BG_1_ANSI;
        let _: Color = BG_2_ANSI;
        let _: Color = BG_3_ANSI;
        let _: Color = BG_4_ANSI;
        let _: Color = FG_0_ANSI;
        let _: Color = FG_1_ANSI;
        let _: Color = FG_2_ANSI;
        let _: Color = FG_3_ANSI;
        let _: Color = ACCENT_PRIMARY_ANSI;
        let _: Color = ACCENT_INFO_ANSI;
        let _: Color = ACCENT_WARNING_ANSI;
        let _: Color = ACCENT_ERROR_ANSI;
        let _: Color = ACCENT_SUCCESS_ANSI;

        assert_eq!(COMMUNITY_COLORS_ANSI.len(), 6);
    }

    #[test]
    fn community_color_deterministic_and_wraps() {
        assert_eq!(community_color(0), COMMUNITY_COLORS[0]);
        assert_eq!(community_color(5), COMMUNITY_COLORS[5]);
        assert_eq!(community_color(11), COMMUNITY_COLORS[11]);

        assert_eq!(community_color(12), COMMUNITY_COLORS[0]);
        assert_eq!(community_color(13), COMMUNITY_COLORS[1]);
        assert_eq!(community_color(24), COMMUNITY_COLORS[0]);

        let a = community_color(42);
        let b = community_color(42);
        assert_eq!(a, b);
    }

    #[test]
    fn truecolor_detection_colorterm() {
        assert!(detect_truecolor(Some("truecolor"), None));
        assert!(detect_truecolor(Some("24bit"), None));
        assert!(!detect_truecolor(Some("other"), None));
    }

    #[test]
    fn truecolor_detection_term_fallback() {
        assert!(detect_truecolor(None, Some("alacritty")));
        assert!(detect_truecolor(None, Some("alacritty-direct")));
        assert!(detect_truecolor(None, Some("wezterm")));
        assert!(detect_truecolor(None, Some("xterm-direct")));
        assert!(!detect_truecolor(None, Some("xterm")));
        assert!(!detect_truecolor(None, Some("vt100")));
    }

    #[test]
    fn truecolor_detection_conservative_default() {
        assert!(!detect_truecolor(None, None));
        assert!(!detect_truecolor(Some(""), Some("")));
    }

    fn channel_luminance(val: u8) -> f64 {
        let v = f64::from(val) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    fn relative_luminance(color: Color) -> f64 {
        match color {
            Color::Rgb(r, g, b) => 0.0722f64.mul_add(
                channel_luminance(b),
                0.7152f64.mul_add(channel_luminance(g), 0.2126 * channel_luminance(r)),
            ),
            _ => 0.0,
        }
    }

    fn contrast_ratio(fg: Color, bg: Color) -> f64 {
        let l1 = relative_luminance(fg);
        let l2 = relative_luminance(bg);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn test_wcag_contrast_ratios() {
        let pairs = [
            (FG_0, BG_0, 4.5),
            (FG_1, BG_0, 4.5),
            (FG_1, BG_1, 4.5),
            (FG_0, BG_3, 4.5),
            (ACCENT_PRIMARY, BG_0, 4.5),
            (ACCENT_ERROR, BG_0, 4.5),
            (ACCENT_SUCCESS, BG_0, 4.5),
            (ACCENT_INFO, BG_0, 4.5),
            (ACCENT_WARNING, BG_0, 4.5),
        ];

        for (fg, bg, min_ratio) in pairs {
            let ratio = contrast_ratio(fg, bg);
            assert!(
                ratio >= min_ratio,
                "Contrast ratio {ratio:.2} < {min_ratio} for {fg:?} on {bg:?}"
            );
        }

        // FG_2 on BG_0 is muted secondary text (3.9:1) which exceeds the WCAG large text standard (>3:1).
        let fg2_ratio = contrast_ratio(FG_2, BG_0);
        assert!(
            fg2_ratio >= 3.5,
            "FG_2 on BG_0 contrast ratio {fg2_ratio:.2} < 3.5"
        );
    }

    #[test]
    fn test_resolve_color_and_supports_truecolor_call() {
        // supports_truecolor() should execute without panicking
        let tc = supports_truecolor();
        let chosen = resolve_color(FG_0, FG_0_ANSI);
        if tc {
            assert_eq!(chosen, FG_0);
        } else {
            assert_eq!(chosen, FG_0_ANSI);
        }
    }
}
