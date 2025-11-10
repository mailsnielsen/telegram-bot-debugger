use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    // Status info
    let status_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Monitoring Status: "),
            if app.monitoring.is_active() {
                if app.monitoring.paused {
                    Span::styled("⏸ PAUSED", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled("● ACTIVE", Style::default().fg(Color::Green))
                }
            } else {
                Span::styled("○ INACTIVE", Style::default().fg(Color::Red))
            },
        ]),
        Line::from(format!(
            "Messages received: {}",
            app.monitoring.messages.len()
        )),
        Line::from(
            "Press 'm' to start/stop | 'p' to pause | 'c' to clear | F5 to exit | 1-5 to navigate",
        ),
    ];

    let status = Paragraph::new(status_text).block(Block::bordered().title("Live Monitor"));

    frame.render_widget(status, chunks[0]);

    // Message list
    if app.monitoring.messages.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from("No messages yet."),
            Line::from(""),
            Line::from("To receive messages:"),
            Line::from("1. Send a message to your bot"),
            Line::from("2. Post in a channel where the bot is admin"),
            Line::from("3. Send a message in a group with the bot"),
            Line::from(""),
            Line::from("New messages will appear here in real-time."),
        ];

        let empty_paragraph = Paragraph::new(empty_text).block(Block::bordered().title("Messages"));

        frame.render_widget(empty_paragraph, chunks[1]);
    } else {
        let messages: Vec<ListItem> = app
            .monitoring
            .messages
            .iter()
            .rev()
            .map(|msg| {
                let datetime = DateTime::from_timestamp(msg.timestamp, 0)
                    .unwrap_or_else(|| DateTime::<Local>::default().into());
                let time_str = datetime.format("%H:%M:%S").to_string();

                let sender_str = msg
                    .sender
                    .as_ref()
                    .map(|s| format!("from {s}"))
                    .unwrap_or_default();

                let line = Line::from(vec![
                    Span::styled(
                        format!("[{time_str}] "),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{} ", msg.chat_name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(sender_str, Style::default().fg(Color::Yellow)),
                    Span::raw(": "),
                    Span::raw(&msg.text),
                ]);

                ListItem::new(line)
            })
            .collect();

        let message_list = List::new(messages).block(Block::bordered().title("Messages"));

        frame.render_widget(message_list, chunks[1]);
    }
}
