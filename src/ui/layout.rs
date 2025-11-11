use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::app::{App, Screen};

pub fn render_frame(frame: &mut Frame, app: &App) {
    let [title_area, content_area, status_area] = Layout::vertical([
        Constraint::Length(3), // Title bar
        Constraint::Min(0),    // Content
        Constraint::Length(3), // Status bar
    ])
    .areas(frame.area());

    // Render title bar
    render_title_bar(frame, title_area, app);

    // Render content based on current screen
    match app.ui.current_screen {
        Screen::TokenInput => super::screens::token_input::render(frame, content_area, app),
        Screen::Home => super::screens::home::render(frame, content_area, app),
        Screen::Discovery => super::screens::discovery::render(frame, content_area, app),
        Screen::Messages => super::screens::messages::render(frame, content_area, app),
        Screen::TestMessage => super::screens::test_message::render(frame, content_area, app),
        Screen::Monitor => super::screens::monitor::render(frame, content_area, app),
        Screen::Analytics => super::screens::analytics::render(frame, content_area, app),
        Screen::RawJson => super::screens::raw_json::render(frame, content_area, app),
        Screen::WebhookManagement => {
            super::screens::webhook_management::render(frame, content_area, app)
        }
        Screen::Help => render_help_screen(frame, content_area),
    }

    // Render status bar
    render_status_bar(frame, status_area, app);
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.ui.current_screen {
        Screen::TokenInput => "Telegram Bot Debugger - Token Setup",
        Screen::Home => "Telegram Bot Debugger - Dashboard",
        Screen::Discovery => "Telegram Bot Debugger - Discovery (Chats & Topics)",
        Screen::Messages => "Telegram Bot Debugger - Messages",
        Screen::TestMessage => "Telegram Bot Debugger - Send Test Message",
        Screen::Monitor => "Telegram Bot Debugger - Live Monitor",
        Screen::Analytics => "Telegram Bot Debugger - Analytics",
        Screen::RawJson => "Telegram Bot Debugger - Raw JSON Debug",
        Screen::WebhookManagement => "Telegram Bot Debugger - Webhook Management",
        Screen::Help => "Telegram Bot Debugger - Help",
    };

    let title_paragraph = Paragraph::new(title)
        .block(Block::bordered().border_style(Style::new().cyan()))
        .white()
        .bold();

    frame.render_widget(title_paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let monitoring_indicator = if app.monitoring.is_active() {
        vec!["● ".green().bold(), "Monitor: ON".green()]
    } else {
        vec!["● ".red().bold(), "Monitor: OFF".dark_gray()]
    };

    let mut status_text = monitoring_indicator;
    status_text.push(" | ".into());
    status_text.push("Esc".yellow().bold());
    status_text.push(":Back ".into());
    status_text.push("h".yellow().bold());
    status_text.push(":Help ".into());
    status_text.push("m".yellow().bold());
    status_text.push(":Message ".into());
    status_text.push("1-5".yellow().bold());
    status_text.push(":Screens ".into());
    status_text.push("F5".yellow().bold());
    status_text.push(":Monitor".into());

    if let Some(status_msg) = &app.ui.status_message {
        status_text.push(" | ".into());
        status_text.push(Span::from(status_msg.as_str()).green());
    }

    let status_paragraph = Paragraph::new(Line::from(status_text)).block(Block::bordered());

    frame.render_widget(status_paragraph, area);
}

fn render_help_screen(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Keyboard Shortcuts:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" - Quit the application"),
        ]),
        Line::from(vec![
            Span::styled("h", Style::default().fg(Color::Yellow)),
            Span::raw(" - Show this help screen"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" - Go back to home/previous screen"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Main Navigation:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(" 1 - Discovery (chats, topics, view messages)"),
        Line::from(" 2 - Live Monitor (real-time updates)"),
        Line::from(" 3 - Analytics (statistics)"),
        Line::from(" 4 - Raw JSON Debug (API responses)"),
        Line::from(" 5 - Webhook Management (configure webhooks)"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "F5",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Toggle monitoring (works in background)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Webhook Management Screen:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("i", Style::default().fg(Color::Yellow)),
            Span::raw(" - Get webhook info"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw(" - Delete webhook (enable polling)"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" - Set webhook from entered URL"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Discovery Screen:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" - Navigate chats"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" - View messages for selected chat"),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Yellow)),
            Span::raw(" - Export chat details as JSON"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Messages Screen:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" - Navigate messages"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(Color::Yellow)),
            Span::raw(" - Send test message to this chat"),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Yellow)),
            Span::raw(" - Export message as JSON"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Test Message:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" - Switch between Selected Chat / Manual Chat ID"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" - Send message"),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(help_text).block(Block::bordered().title("Help"));

    frame.render_widget(paragraph, area);
}
