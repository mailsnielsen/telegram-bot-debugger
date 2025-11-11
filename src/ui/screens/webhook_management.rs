use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Webhook info display
            Constraint::Length(8),  // Set webhook section
            Constraint::Min(0),     // Action buttons and help
        ])
        .split(area);

    // Webhook Info Display
    render_webhook_info(frame, chunks[0], app);

    // Set Webhook Section
    render_set_webhook(frame, chunks[1], app);

    // Actions and Help
    render_actions_help(frame, chunks[2], app);
}

fn render_webhook_info(frame: &mut Frame, area: Rect, app: &App) {
    let info_lines = if let Some(cached_info) = &app.ui.webhook_info_cache {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Current Webhook Status:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(cached_info.as_str()),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'i' to refresh",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "No webhook information loaded",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("Press 'i' to fetch current webhook information"),
        ]
    };

    let info_paragraph = Paragraph::new(info_lines)
        .block(
            Block::bordered()
                .title("Webhook Information")
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(info_paragraph, area);
}

fn render_set_webhook(frame: &mut Frame, area: Rect, app: &App) {
    let url_display = if app.ui.webhook_url_input.is_empty() {
        "<enter HTTPS webhook URL>".to_string()
    } else {
        app.ui.webhook_url_input.clone()
    };

    let input_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Webhook URL:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(format!("  {url_display}")),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type URL and press ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::styled(" to set webhook", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let input_paragraph = Paragraph::new(input_lines).block(
        Block::bordered()
            .title("Set New Webhook")
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(input_paragraph, area);
}

fn render_actions_help(frame: &mut Frame, area: Rect, app: &App) {
    let mut help_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Actions:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  i ", Style::default().fg(Color::Yellow)),
            Span::raw("- Get webhook info      "),
            Span::styled("  d ", Style::default().fg(Color::Red)),
            Span::raw("- Delete webhook (enable polling)"),
        ]),
        Line::from(vec![
            Span::styled("  Enter ", Style::default().fg(Color::Green)),
            Span::raw("- Set webhook from URL above"),
        ]),
        Line::from(""),
    ];

    // Show operation result if available
    if let Some(result) = &app.ui.webhook_operation_result {
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(Span::styled(
            "Result:",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(result.as_str()));
    }

    let help_paragraph = Paragraph::new(help_lines)
        .block(
            Block::bordered()
                .title("Help & Results")
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help_paragraph, area);
}
