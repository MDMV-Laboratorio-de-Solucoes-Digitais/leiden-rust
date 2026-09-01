//! Status bar panel widget.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::{App, AppState};

/// Render the bottom status bar and progress indicator.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "progress percentage bounded to 0..100"
)]
pub fn render_status_bar(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let state_str = match &app.state {
        AppState::Idle => Span::styled(
            " IDLE ",
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        AppState::Running { iteration } => Span::styled(
            format!(" RUNNING (Iter {iteration}) "),
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        AppState::Done {
            iterations,
            quality,
        } => Span::styled(
            format!(" DONE (Iters {iterations}, Q={quality:.4}) "),
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        AppState::Error(msg) => Span::styled(
            format!(" ERROR: {msg} "),
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let details_str = format!(
        " γ={:.2} | Seed={} | Cap={} | Q={:.4} | Iter={} | ?:Help | q:Quit",
        app.params.gamma,
        app.params.seed.unwrap_or(0),
        app.params.iteration_cap,
        app.quality,
        app.iterations,
    );

    let line = Line::from(vec![
        state_str,
        Span::styled(details_str, Style::default().fg(Color::White)),
    ]);

    let block = Block::default().borders(Borders::TOP);

    if let AppState::Running { iteration } = app.state {
        let percent = ((f64::from(iteration) / f64::from(app.params.iteration_cap)) * 100.0)
            .clamp(0.0, 100.0) as u16;
        let gauge = Gauge::default()
            .block(block)
            .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
            .percent(percent)
            .label(format!(
                "Iter {iteration}/{} (Q={:.4})",
                app.params.iteration_cap, app.quality
            ));
        frame.render_widget(gauge, area);
    } else {
        let paragraph = Paragraph::new(line).block(block);
        frame.render_widget(paragraph, area);
    }
}
