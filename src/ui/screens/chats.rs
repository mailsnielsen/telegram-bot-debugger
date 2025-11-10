use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chats = app.telegram.get_discovered_chats();

    if chats.is_empty() {
        let empty_message = Paragraph::new(vec![
            Line::from(""),
            Line::from("No chats discovered yet."),
            Line::from(""),
            Line::from("The bot will discover chats when:"),
            Line::from("1. Someone sends a message to the bot"),
            Line::from("2. The bot is added to a group or channel"),
            Line::from("3. You use the Live Monitor to fetch updates"),
            Line::from(""),
            Line::from("Press 'F5' to start monitoring for updates."),
        ])
        .block(Block::bordered().title("Discovered Chats"));

        frame.render_widget(empty_message, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Chat list
    let chat_items: Vec<ListItem> = chats
        .iter()
        .enumerate()
        .map(|(i, chat)| {
            let color = match chat.chat.chat_type.as_str() {
                "private" => Color::Green,
                "group" | "supergroup" => Color::Blue,
                "channel" => Color::Yellow,
                _ => Color::White,
            };

            let indicator = if i == app.selected_chat_index {
                "→ "
            } else {
                "  "
            };

            let style = if i == app.selected_chat_index {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            let content = format!(
                "{}{} ({})",
                indicator,
                chat.chat.display_name(),
                chat.chat.chat_type
            );

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let chat_list = List::new(chat_items)
        .block(Block::bordered().title("Chats (↑/↓ to navigate)"));

    frame.render_widget(chat_list, chunks[0]);

    // Chat details
    if let Some(selected_chat) = app.get_selected_chat() {
        let mut details = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Chat Details:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Chat ID: {}", selected_chat.chat.id)),
            Line::from(format!("Type: {}", selected_chat.chat.chat_type)),
            Line::from(format!("Messages: {}", selected_chat.message_count)),
            Line::from(format!("Topics: {}", selected_chat.topics.len())),
        ];

        if let Some(username) = &selected_chat.chat.username {
            details.push(Line::from(format!("Username: @{}", username)));
        }

        if !selected_chat.topics.is_empty() {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                "Topics:",
                Style::default().fg(Color::Yellow),
            )));
            for topic in &selected_chat.topics {
                details.push(Line::from(format!(
                    "  • Thread ID: {} ({} messages)",
                    topic.thread_id, topic.message_count
                )));
            }
        }

        let details_paragraph = Paragraph::new(details)
            .block(Block::bordered().title("Details"));

        frame.render_widget(details_paragraph, chunks[1]);
    }
}

