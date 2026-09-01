//! Integration and snapshot tests for User Story 1, User Story 7, and Phase 3.5 (Help Overlay).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::single_char_pattern,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use crossterm::event::{KeyCode, KeyEvent};
use leiden::{CsrGraph, LeidenEvent};
use leiden_tui::app::{App, AppState, FocusPanel};
use leiden_tui::ui;
use leiden_tui::ui::colors::{
    ACCENT_ERROR, ACCENT_INFO, ACCENT_PRIMARY, ACCENT_WARNING, BG_3, COMMUNITY_COLORS, FG_0, FG_1,
    FG_2, FG_3, community_color,
};
use leiden_tui::ui::styles::{
    LayoutMode, focused_border_style, header_style, layout_mode, log_debug_style, log_error_style,
    log_info_style, log_trace_style, log_warn_style, normal_row_style, selected_row_style,
    unfocused_border_style,
};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;

#[test]
fn test_status_bar_state_snapshots_all_four_variants() {
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    // 1. Idle
    let mut app = App::new_idle();
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw idle");
    let buf = terminal.backend().buffer().clone();
    let status_text = format!("{buf:?}");
    assert!(status_text.contains('○'), "Idle status missing ○ indicator");
    assert!(
        status_text.contains("Idle"),
        "Idle status missing Idle label"
    );

    // 2. Running
    app.state = AppState::Running { iteration: 2 };
    app.params.iteration_cap = 10;
    app.quality = 0.4231;
    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("draw running");
    let buf = terminal.backend().buffer().clone();
    let status_text = format!("{buf:?}");
    assert!(
        status_text.contains('●'),
        "Running status missing ● indicator"
    );
    assert!(
        status_text.contains("Running"),
        "Running status missing Running label"
    );
    assert!(
        status_text.contains("2/10"),
        "Running status missing 2/10 ratio"
    );
    assert!(
        status_text.contains("Q=0.4231"),
        "Running status missing Q=0.4231"
    );

    // 3. Done
    app.state = AppState::Done {
        iterations: 10,
        quality: 0.4500,
    };
    app.quality = 0.4500;
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw done");
    let buf = terminal.backend().buffer().clone();
    let status_text = format!("{buf:?}");
    assert!(status_text.contains('✓'), "Done status missing ✓ indicator");
    assert!(
        status_text.contains("Done"),
        "Done status missing Done label"
    );
    assert!(
        status_text.contains("Q=0.4500"),
        "Done status missing Q=0.4500"
    );

    // 4. Error
    app.state = AppState::Error("Graph file corrupt".to_string());
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw error");
    let buf = terminal.backend().buffer().clone();
    let status_text = format!("{buf:?}");
    assert!(
        status_text.contains('✗'),
        "Error status missing ✗ indicator"
    );
    assert!(
        status_text.contains("Error"),
        "Error status missing Error label"
    );
    assert!(
        status_text.contains("Graph file corrupt"),
        "Error status missing error message"
    );
}

#[test]
fn test_number_formatting_and_delta_q() {
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let mut app = App::new_idle();
    app.params.gamma = 1.5;
    app.params.iteration_cap = 10;
    app.state = AppState::Running { iteration: 3 };
    app.quality = 0.4231;
    app.push(LeidenEvent::IterationFinished {
        index: 2,
        quality: 0.4198,
    });
    app.push(LeidenEvent::IterationFinished {
        index: 3,
        quality: 0.4231,
    }); // delta_q = +0.0033

    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw frame");
    let buf = terminal.backend().buffer().clone();
    let text = format!("{buf:?}");
    assert!(
        text.contains("γ=1.50"),
        "Gamma must be formatted to 2dp: γ=1.50"
    );
    assert!(
        text.contains("3/10"),
        "Iteration progress gauge must show 3/10"
    );
    assert!(
        text.contains("Q=0.4231"),
        "Quality must be formatted to 4dp: Q=0.4231"
    );
    assert!(
        text.contains("ΔQ=+0.0033"),
        "Positive delta Q must be formatted signed to 4dp: ΔQ=+0.0033"
    );
}

#[test]
fn test_negative_delta_q_formatting() {
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let mut app = App::new_idle();
    app.params.gamma = 1.0;
    app.params.iteration_cap = 20;
    app.state = AppState::Running { iteration: 2 };
    app.quality = 0.4100;
    app.push(LeidenEvent::IterationFinished {
        index: 1,
        quality: 0.4200,
    });
    app.push(LeidenEvent::IterationFinished {
        index: 2,
        quality: 0.4100,
    }); // delta_q = -0.0100

    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw frame");
    let buf = terminal.backend().buffer().clone();
    let text = format!("{buf:?}");
    assert!(
        text.contains("ΔQ=-0.0100"),
        "Negative delta Q must be formatted signed to 4dp: ΔQ=-0.0100"
    );
}

#[test]
fn test_help_overlay_structure_and_grouping() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let mut app = App::new_idle();
    app.visibility.help_open = true;

    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("draw help overlay");
    let buf = terminal.backend().buffer().clone();
    let text = format!("{buf:?}");

    // Title & structure
    assert!(
        text.contains("KEY BINDINGS"),
        "Help overlay must have title KEY BINDINGS"
    );
    assert!(
        text.contains("Navigation"),
        "Help overlay must contain Navigation group"
    );
    assert!(
        text.contains("Panels"),
        "Help overlay must contain Panels group"
    );
    assert!(
        text.contains("General"),
        "Help overlay must contain General group"
    );

    // Key bindings
    assert!(
        text.contains('q') && text.contains("Quit"),
        "Must list Quit"
    );
    assert!(
        text.contains('r') && text.contains("Restart"),
        "Must list Restart"
    );
    assert!(
        text.contains('s') && text.contains("Step"),
        "Must list Step"
    );
    assert!(
        text.contains('p') && text.contains("Pause"),
        "Must list Pause"
    );
    assert!(
        text.contains('g') && text.contains("Toggle graph"),
        "Must list Toggle graph"
    );
    assert!(
        text.contains('l') && text.contains("Toggle log"),
        "Must list Toggle log"
    );
    assert!(
        text.contains("Tab") && text.contains("Switch panel focus"),
        "Must list Switch panel focus"
    );
    assert!(
        text.contains("Select community"),
        "Must list Select community"
    );
    assert!(
        text.contains("Press any key to close"),
        "Must list footer hint Press any key to close"
    );
}

#[test]
fn test_style_presets_and_no_reversed() {
    let sel = selected_row_style();
    assert_eq!(sel.fg, Some(FG_0));
    assert_eq!(sel.bg, Some(BG_3));
    assert!(sel.add_modifier.contains(Modifier::BOLD));
    assert!(!sel.add_modifier.contains(Modifier::REVERSED));

    let hdr = header_style();
    assert_eq!(hdr.fg, Some(FG_1));
    assert!(hdr.add_modifier.contains(Modifier::BOLD));

    let norm = normal_row_style();
    assert_eq!(norm.fg, Some(FG_1));

    let foc = focused_border_style();
    assert_eq!(foc.fg, Some(ACCENT_PRIMARY));

    let unfoc = unfocused_border_style();
    assert_eq!(unfoc.fg, Some(FG_3));
}

#[test]
fn test_log_severity_styles() {
    let err = log_error_style();
    assert_eq!(err.fg, Some(ACCENT_ERROR));
    assert!(err.add_modifier.contains(Modifier::BOLD));

    let warn = log_warn_style();
    assert_eq!(warn.fg, Some(ACCENT_WARNING));
    assert!(warn.add_modifier.contains(Modifier::BOLD));

    let info = log_info_style();
    assert_eq!(info.fg, Some(ACCENT_INFO));
    assert!(!info.add_modifier.contains(Modifier::BOLD));

    let dbg = log_debug_style();
    assert_eq!(dbg.fg, Some(FG_2));

    let trace = log_trace_style();
    assert_eq!(trace.fg, Some(FG_3));
    assert!(trace.add_modifier.contains(Modifier::DIM));
}

#[test]
fn test_cross_panel_color_parity() {
    let mut app = App::new_idle();
    app.partition = vec![
        ("node_a".to_string(), 0),
        ("node_b".to_string(), 0),
        ("node_c".to_string(), 1),
        ("node_d".to_string(), 13), // wraps to 13 % 12 = 1
    ];

    let graph = CsrGraph::from_edges([
        leiden::Edge {
            source: "node_a".to_string(),
            target: "node_b".to_string(),
            weight: 1.0,
        },
        leiden::Edge {
            source: "node_c".to_string(),
            target: "node_d".to_string(),
            weight: 2.0,
        },
    ])
    .expect("valid graph");
    app.graph = Some(graph);

    let summaries = app.community_summaries();
    assert_eq!(summaries.len(), 3);

    for s in &summaries {
        let expected_color = community_color(s.id);
        assert_eq!(expected_color, COMMUNITY_COLORS[(s.id as usize) % 12]);
    }

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");

    let buffer = terminal.backend().buffer();
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Graph Topology"));
    assert!(debug_str.contains("node_a"));
}

#[test]
fn test_log_pane_rendering_and_styling() {
    let app = App::new_idle();
    if let Ok(mut ring) = app.log_ring.lock() {
        ring.push_back("[ERROR] test_target: failed to converge".to_string());
        ring.push_back("[WARN] test_target: slow iteration".to_string());
        ring.push_back("[INFO] test_target: step completed".to_string());
        ring.push_back("[DEBUG] test_target: internal weight=10".to_string());
        ring.push_back("[TRACE] test_target: trace details".to_string());
    }

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");

    let buffer = terminal.backend().buffer();
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("Logs"));
    assert!(debug_str.contains("[ERROR]"));
    assert!(debug_str.contains("[WARN]"));
    assert!(debug_str.contains("[INFO]"));
    assert!(debug_str.contains("[DEBUG]"));
    assert!(debug_str.contains("[TRACE]"));
}

#[test]
fn test_focus_navigation_cycle_and_skipping() {
    let mut app = App::new_idle();
    assert_eq!(app.focus, FocusPanel::CommunityList);

    // Full 3 panels: Community -> Graph -> Log -> Community
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::GraphView);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::LogPane);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::CommunityList);

    // Hide graph: Community -> Log -> Community
    app.visibility.show_graph = false;
    app.visibility.show_log = true;
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::LogPane);
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::CommunityList);

    // Hide log too: only Community visible -> Tab is no-op
    app.visibility.show_log = false;
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::CommunityList);

    // Hide graph, show log; if currently at GraphView and graph is hidden, normalize focus
    app.visibility.show_graph = false;
    app.visibility.show_log = true;
    app.focus = FocusPanel::GraphView;
    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.focus, FocusPanel::LogPane);
}

#[test]
fn test_responsive_layout_modes_and_toggle_redistribution() {
    assert_eq!(layout_mode(140), LayoutMode::Full);
    assert_eq!(layout_mode(100), LayoutMode::Compact);
    assert_eq!(layout_mode(70), LayoutMode::Stacked);
    assert_eq!(layout_mode(50), LayoutMode::Minimal);

    let app = App::new_idle();

    // 1. Full mode
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Graph Topology"));
    assert!(debug_str.contains("Logs"));

    // 2. Compact mode
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Graph Topology"));
    assert!(debug_str.contains("Logs"));

    // 3. Stacked mode
    let backend = TestBackend::new(70, 35);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Graph Topology"));
    assert!(debug_str.contains("Logs"));

    // 4. Minimal mode (only focused panel visible)
    let backend = TestBackend::new(50, 24);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(!debug_str.contains("Graph Topology"));
    assert!(!debug_str.contains("Logs"));
}

#[test]
fn test_panel_toggles_all_modes() {
    let mut app = App::new_idle();

    // Full mode with graph hidden
    app.visibility.show_graph = false;
    app.visibility.show_log = true;
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(!debug_str.contains("Graph Topology"));
    assert!(debug_str.contains("Logs"));

    // Full mode with log hidden
    app.visibility.show_graph = true;
    app.visibility.show_log = false;
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Graph Topology"));
    assert!(!debug_str.contains("Logs"));

    // Full mode with both hidden -> community only
    app.visibility.show_graph = false;
    app.visibility.show_log = false;
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(!debug_str.contains("Graph Topology"));
    assert!(!debug_str.contains("Logs"));

    // Stacked mode with graph hidden
    app.visibility.show_graph = false;
    app.visibility.show_log = true;
    let backend = TestBackend::new(70, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(!debug_str.contains("Graph Topology"));
    assert!(debug_str.contains("Logs"));

    // Stacked mode with log hidden
    app.visibility.show_graph = true;
    app.visibility.show_log = false;
    let backend = TestBackend::new(70, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders");
    let debug_str = format!("{:?}", terminal.backend().buffer());
    assert!(debug_str.contains("Communities"));
    assert!(debug_str.contains("Graph Topology"));
    assert!(!debug_str.contains("Logs"));
}

#[test]
fn test_ansi_16_color_state_indicators_distinguishable() {
    // T035, T036, SC-005: 16-color fallback distinguishable by symbol and color
    use leiden_tui::ui::colors::{
        ACCENT_ERROR, ACCENT_ERROR_ANSI, ACCENT_INFO, ACCENT_INFO_ANSI, ACCENT_PRIMARY_ANSI,
        ACCENT_SUCCESS, ACCENT_SUCCESS_ANSI, FG_2, FG_2_ANSI,
    };
    use leiden_tui::ui::styles::{state_color, state_indicator, state_label};

    // Verify all states return non-empty symbols and distinct ANSI-resolvable fallback styles
    let idle = AppState::Idle;
    assert_eq!(state_indicator(&idle), "○");
    assert_eq!(state_label(&idle), "Idle");
    assert_eq!(state_color(&idle), FG_2);

    let running = AppState::Running { iteration: 1 };
    assert_eq!(state_indicator(&running), "●");
    assert_eq!(state_label(&running), "Running");
    assert_eq!(state_color(&running), ACCENT_INFO);

    let done = AppState::Done {
        iterations: 5,
        quality: 0.5,
    };
    assert_eq!(state_indicator(&done), "✓");
    assert_eq!(state_label(&done), "Done");
    assert_eq!(state_color(&done), ACCENT_SUCCESS);

    let error = AppState::Error("failed".into());
    assert_eq!(state_indicator(&error), "✗");
    assert_eq!(state_label(&error), "Error");
    assert_eq!(state_color(&error), ACCENT_ERROR);

    // Verify indicators are distinct characters for color-vision deficiency
    let mut unique = vec![
        state_indicator(&idle),
        state_indicator(&running),
        state_indicator(&done),
        state_indicator(&error),
    ];
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        4,
        "All 4 state indicators must be unique chars"
    );

    // Check ANSI fallback color mappings
    assert_eq!(FG_2_ANSI, ratatui::style::Color::DarkGray);
    assert_eq!(ACCENT_PRIMARY_ANSI, ratatui::style::Color::Blue);
    assert_eq!(ACCENT_INFO_ANSI, ratatui::style::Color::Cyan);
    assert_eq!(ACCENT_SUCCESS_ANSI, ratatui::style::Color::Green);
    assert_eq!(ACCENT_ERROR_ANSI, ratatui::style::Color::Red);
}

#[test]
fn test_no_emoji_in_rendered_output() {
    // SC-009, T054, FR-016: All symbols drawn from documented Unicode BMP set, no emoji (U+1F000..=U+1FAFF, etc.)
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let mut app = App::new_idle();
    app.visibility.help_open = true;
    app.partition = vec![("node_1".to_string(), 0), ("node_2".to_string(), 1)];
    let graph = CsrGraph::from_edges([leiden::Edge {
        source: "node_1".to_string(),
        target: "node_2".to_string(),
        weight: 1.0,
    }])
    .expect("valid graph");
    app.graph = Some(graph);

    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw frame");
    let buf = terminal.backend().buffer();

    for cell in buf.content() {
        let symbol = cell.symbol();
        for ch in symbol.chars() {
            let code = ch as u32;
            // Check non-BMP emoji and known pictorial emoji codepoint ranges
            let is_emoji = (0x1F000..=0x1FAFF).contains(&code)
                || (0x2600..=0x26FF).contains(&code) && (ch == '\u{26a0}' || ch == '\u{2728}');
            assert!(
                !is_emoji,
                "Found forbidden emoji character '{ch}' (U+{code:04X}) in buffer"
            );
        }
    }
}

#[test]
fn test_partitions_up_to_100_communities_and_scrolling_beyond() {
    // SC-010, FR-018, T055: Partitions with up to 100 communities render; >100 scroll with deterministic color cycling
    let mut app = App::new_idle();

    // Generate 120 communities
    let mut partition = Vec::new();
    let mut edges = Vec::new();
    for i in 0..120 {
        let node_a = format!("node_{i}_a");
        let node_b = format!("node_{i}_b");
        partition.push((node_a.clone(), i));
        partition.push((node_b.clone(), i));
        edges.push(leiden::Edge {
            source: node_a,
            target: node_b,
            weight: 1.0,
        });
    }
    let graph = CsrGraph::from_edges(edges).expect("valid graph");
    app.graph = Some(graph);
    app.partition = partition;

    let summaries = app.community_summaries();
    assert_eq!(summaries.len(), 120);

    // Verify deterministic color cycling for all 120 communities
    for (i, s) in summaries.iter().enumerate() {
        let expected_color = community_color(s.id);
        assert_eq!(expected_color, COMMUNITY_COLORS[(s.id as usize) % 12]);
        assert_eq!(s.id, u32::try_from(i).expect("valid u32 index"));
    }

    // Render with 120 communities
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw frame");
    let text = format!("{:?}", terminal.backend().buffer());
    assert!(text.contains("Communities"));

    // Scroll down and test
    app.handle_key(KeyEvent::from(KeyCode::Down));
    app.handle_key(KeyEvent::from(KeyCode::Down));
    let _ = terminal
        .draw(|f| ui::render(f, &app))
        .expect("draw frame after scroll");
}

#[test]
fn test_render_loop_frame_budget_under_50ms() {
    // SC-007, T052: Render loop completes each frame in under 50 ms at 20 FPS poll rate
    let mut app = App::new_idle();
    app.state = AppState::Running { iteration: 5 };
    app.params.iteration_cap = 100;
    app.quality = 0.5234;

    let mut partition = Vec::new();
    let mut edges = Vec::new();
    for i in 0..50 {
        let node_a = format!("node_{i}_a");
        let node_b = format!("node_{i}_b");
        partition.push((node_a.clone(), i));
        partition.push((node_b.clone(), i));
        edges.push(leiden::Edge {
            source: node_a,
            target: node_b,
            weight: 1.0,
        });
    }
    let graph = CsrGraph::from_edges(edges).expect("valid graph");
    app.graph = Some(graph);
    app.partition = partition;

    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let start = std::time::Instant::now();
    let frames = 20;
    for _ in 0..frames {
        let _ = terminal.draw(|f| ui::render(f, &app)).expect("draw frame");
    }
    let elapsed = start.elapsed();
    let per_frame = elapsed / frames;

    assert!(
        per_frame < std::time::Duration::from_millis(50),
        "Render loop frame duration {per_frame:?} exceeds 50 ms budget"
    );
}
