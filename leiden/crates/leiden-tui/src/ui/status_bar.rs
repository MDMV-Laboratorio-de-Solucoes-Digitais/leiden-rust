//! Status bar panel widget.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Gauge, Paragraph, Sparkline};

use crate::app::{App, AppState};
use crate::ui::colors::{ACCENT_ERROR, ACCENT_INFO, ACCENT_SUCCESS, BG_2, BG_4, FG_0, FG_1, FG_2};
use crate::ui::styles::{
    DELTA, GAMMA, key_hint_style, key_letter_style, state_color, state_indicator, state_label,
};

/// Compute the quality history for up to the last 20 iterations from events.
#[must_use]
pub fn collect_quality_history(app: &App) -> Vec<f64> {
    let mut qualities = Vec::new();
    for event in &app.events {
        if let leiden::LeidenEvent::IterationFinished { quality, .. }
        | leiden::LeidenEvent::QualityComputed { quality, .. } = event
        {
            qualities.push(*quality);
        }
    }
    if qualities.is_empty()
        && (app.quality.abs() > f64::EPSILON || matches!(app.state, AppState::Done { .. }))
    {
        qualities.push(app.quality);
    }
    // Fixed window of most recent 20 iterations
    if qualities.len() > 20 {
        let skip_count = qualities.len() - 20;
        qualities = qualities.into_iter().skip(skip_count).collect();
    }
    qualities
}

/// Compute modularity delta (ΔQ) between the latest and preceding quality values.
#[must_use]
pub fn compute_delta_q(app: &App) -> Option<f64> {
    let qualities = collect_quality_history(app);
    if qualities.len() >= 2 {
        let latest = qualities[qualities.len() - 1];
        let prev = qualities[qualities.len() - 2];
        Some(latest - prev)
    } else {
        None
    }
}

/// Build the quality sparkline widget and data.
#[must_use]
pub fn build_quality_sparkline(qualities: &[f64]) -> (Sparkline<'_>, Vec<u64>) {
    let max = qualities.iter().copied().fold(0.0_f64, f64::max).max(1e-9);
    let data: Vec<u64> = qualities
        .iter()
        .map(|q| {
            let clamped = q.clamp(0.0, max);
            let normalized = (clamped / max) * 100.0;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "sparkline normalized value bounded to 0..100"
            )]
            let val = normalized as u64;
            val
        })
        .collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(ACCENT_INFO));
    (sparkline, data)
}

/// Build standard status bar key hints span list.
fn build_key_hints() -> Vec<Span<'static>> {
    vec![
        Span::styled("q", key_letter_style()),
        Span::styled(":quit ", key_hint_style()),
        Span::styled("r", key_letter_style()),
        Span::styled(":restart ", key_hint_style()),
        Span::styled("p", key_letter_style()),
        Span::styled(":pause ", key_hint_style()),
        Span::styled("?", key_letter_style()),
        Span::styled(":help", key_hint_style()),
    ]
}

fn build_delta_q_span(delta: f64) -> Span<'static> {
    let (delta_color, sign_prefix) = if delta >= 0.0 {
        (ACCENT_SUCCESS, "+")
    } else {
        (ACCENT_ERROR, "")
    };
    Span::styled(
        format!("{DELTA}Q={sign_prefix}{delta:.4}  "),
        Style::default()
            .fg(delta_color)
            .add_modifier(Modifier::BOLD),
    )
}

fn render_running_with_gauge(
    frame: &mut Frame<'_>,
    app: &App,
    area: Rect,
    iteration: u32,
    gauge_width: u16,
) {
    let bar_style = Style::default().bg(BG_2);
    let s_color = state_color(&app.state);
    let s_indicator = state_indicator(&app.state);
    let s_label = state_label(&app.state);

    let cap = app.params.iteration_cap.max(1);
    let ratio = (f64::from(iteration) / f64::from(cap)).clamp(0.0, 1.0);
    let gauge_label = format!("{iteration}/{cap}  Q={:.4}", app.quality);
    let delta_q_opt = compute_delta_q(app);

    let badge_len = u16::try_from(s_indicator.len() + s_label.len() + 3).unwrap_or(12);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(badge_len),
            Constraint::Length(gauge_width),
            Constraint::Min(20),
        ])
        .split(area);

    let state_spans = vec![
        Span::styled(format!(" {s_indicator} "), Style::default().fg(s_color)),
        Span::styled(
            s_label,
            Style::default().fg(s_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(state_spans)).style(bar_style),
        chunks[0],
    );

    let gauge = Gauge::default()
        .ratio(ratio)
        .gauge_style(Style::default().fg(ACCENT_INFO).bg(BG_4))
        .label(gauge_label);
    frame.render_widget(gauge, chunks[1]);

    let mut right_spans = vec![Span::raw("  ")];
    if let Some(delta) = delta_q_opt {
        right_spans.push(build_delta_q_span(delta));
    }

    right_spans.push(Span::styled(format!("{GAMMA}="), Style::default().fg(FG_2)));
    right_spans.push(Span::styled(
        format!("{:.2}  ", app.params.gamma),
        Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
    ));

    right_spans.push(Span::styled("seed=", Style::default().fg(FG_2)));
    right_spans.push(Span::styled(
        format!("{}  ", app.params.seed.unwrap_or(0)),
        Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
    ));

    let qualities = collect_quality_history(app);
    if qualities.is_empty() {
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(34)])
            .split(chunks[2]);

        frame.render_widget(
            Paragraph::new(Line::from(right_spans)).style(bar_style),
            inner_chunks[0],
        );
        let hints_p = Paragraph::new(Line::from(build_key_hints()))
            .style(bar_style)
            .alignment(ratatui::layout::Alignment::Right);
        frame.render_widget(hints_p, inner_chunks[1]);
    } else {
        let (sparkline, _data) = build_quality_sparkline(&qualities);
        let spark_len = u16::try_from(qualities.len()).unwrap_or(10);
        let spark_width = spark_len.min(10);
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(20),
                Constraint::Length(spark_width),
                Constraint::Length(34),
            ])
            .split(chunks[2]);

        frame.render_widget(
            Paragraph::new(Line::from(right_spans)).style(bar_style),
            inner_chunks[0],
        );
        frame.render_widget(sparkline, inner_chunks[1]);
        let hints_p = Paragraph::new(Line::from(build_key_hints()))
            .style(bar_style)
            .alignment(ratatui::layout::Alignment::Right);
        frame.render_widget(hints_p, inner_chunks[2]);
    }
}

fn render_running_bar(frame: &mut Frame<'_>, app: &App, area: Rect, iteration: u32) {
    let bar_style = Style::default().bg(BG_2);
    let s_color = state_color(&app.state);
    let s_indicator = state_indicator(&app.state);
    let s_label = state_label(&app.state);

    let iter = iteration;
    let cap = app.params.iteration_cap.max(1);
    let gauge_width = 24.min(area.width.saturating_sub(60));
    let has_gauge = gauge_width >= 10;
    let delta_q_opt = compute_delta_q(app);

    if has_gauge {
        render_running_with_gauge(frame, app, area, iteration, gauge_width);
    } else {
        let mut spans = vec![
            Span::styled(format!(" {s_indicator} "), Style::default().fg(s_color)),
            Span::styled(
                s_label,
                Style::default().fg(s_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{iter}/{cap}  Q={:.4}  ", app.quality),
                Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
            ),
        ];
        if let Some(delta) = delta_q_opt {
            spans.push(build_delta_q_span(delta));
        }
        spans.push(Span::styled(
            format!("{GAMMA}={:.2}  ", app.params.gamma),
            Style::default().fg(FG_2),
        ));
        frame.render_widget(Paragraph::new(Line::from(spans)).style(bar_style), area);
    }
}

fn render_idle_bar(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let bar_style = Style::default().bg(BG_2);
    let s_color = state_color(&app.state);
    let s_indicator = state_indicator(&app.state);
    let s_label = state_label(&app.state);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(34)])
        .split(area);

    let spans = vec![
        Span::styled(format!(" {s_indicator} "), Style::default().fg(s_color)),
        Span::styled(
            s_label,
            Style::default().fg(s_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(format!("{GAMMA}="), Style::default().fg(FG_2)),
        Span::styled(
            format!("{:.2}  ", app.params.gamma),
            Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
        ),
        Span::styled("seed=", Style::default().fg(FG_2)),
        Span::styled(
            format!("{}  ", app.params.seed.unwrap_or(0)),
            Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
        ),
        Span::styled("cap=", Style::default().fg(FG_2)),
        Span::styled(
            format!("{}", app.params.iteration_cap),
            Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
        ),
    ];

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(bar_style),
        chunks[0],
    );
    let hints_p = Paragraph::new(Line::from(build_key_hints()))
        .style(bar_style)
        .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(hints_p, chunks[1]);
}

fn render_done_bar(frame: &mut Frame<'_>, app: &App, area: Rect, iterations: u32, quality: f64) {
    let bar_style = Style::default().bg(BG_2);
    let s_color = state_color(&app.state);
    let s_indicator = state_indicator(&app.state);
    let s_label = state_label(&app.state);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(34)])
        .split(area);

    let delta_q_opt = compute_delta_q(app);

    let mut spans = vec![
        Span::styled(format!(" {s_indicator} "), Style::default().fg(s_color)),
        Span::styled(
            s_label,
            Style::default().fg(s_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("iter=", Style::default().fg(FG_2)),
        Span::styled(
            format!("{iterations}/{}  ", app.params.iteration_cap),
            Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Q=", Style::default().fg(FG_2)),
        Span::styled(
            format!("{quality:.4}  "),
            Style::default()
                .fg(ACCENT_SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(delta) = delta_q_opt {
        spans.push(build_delta_q_span(delta));
    }

    spans.push(Span::styled(format!("{GAMMA}="), Style::default().fg(FG_2)));
    spans.push(Span::styled(
        format!("{:.2}  ", app.params.gamma),
        Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("seed=", Style::default().fg(FG_2)));
    spans.push(Span::styled(
        format!("{}  ", app.params.seed.unwrap_or(0)),
        Style::default().fg(FG_0).add_modifier(Modifier::BOLD),
    ));

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(bar_style),
        chunks[0],
    );
    let hints_p = Paragraph::new(Line::from(build_key_hints()))
        .style(bar_style)
        .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(hints_p, chunks[1]);
}

/// Render the bottom status bar and progress indicator.
pub fn render_status_bar(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let bar_style = Style::default().bg(BG_2);
    let block = Block::default().style(bar_style);
    frame.render_widget(block, area);

    match &app.state {
        AppState::Running { iteration } => {
            render_running_bar(frame, app, area, *iteration);
        }
        AppState::Idle => {
            render_idle_bar(frame, app, area);
        }
        AppState::Done {
            iterations,
            quality,
        } => {
            render_done_bar(frame, app, area, *iterations, *quality);
        }
        AppState::Error(msg) => {
            let s_color = state_color(&app.state);
            let s_indicator = state_indicator(&app.state);
            let s_label = state_label(&app.state);
            let spans = vec![
                Span::styled(format!(" {s_indicator} "), Style::default().fg(s_color)),
                Span::styled(
                    s_label,
                    Style::default().fg(s_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(": {msg}"), Style::default().fg(FG_1)),
            ];
            frame.render_widget(Paragraph::new(Line::from(spans)).style(bar_style), area);
        }
    }
}
