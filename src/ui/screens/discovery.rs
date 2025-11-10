use ratatui::{
    layout::{Constraint, Layout, Rect},
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
        .block(Block::bordered().title("Discovery - Chats & Topics"));

        frame.render_widget(empty_message, area);
        return;
    }

    let [list_area, details_area] = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .areas(area);

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

            let indicator = if i == app.ui.selected_chat_index {
                "→ "
            } else {
                "  "
            };

            let style = if i == app.ui.selected_chat_index {
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
        .block(Block::bordered().title("Chats (↑/↓ to navigate | Enter to view messages | e to export)"));

    frame.render_widget(chat_list, list_area);

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
        ];

        if let Some(username) = &selected_chat.chat.username {
            details.push(Line::from(format!("Username: @{username}")));
        }

        if let Some(title) = &selected_chat.chat.title {
            details.push(Line::from(format!("Title: {title}")));
        }

        if let Some(first_name) = &selected_chat.chat.first_name {
            details.push(Line::from(format!("First Name: {first_name}")));
        }

        if let Some(last_name) = &selected_chat.chat.last_name {
            details.push(Line::from(format!("Last Name: {last_name}")));
        }

        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "Statistics:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        details.push(Line::from(format!("Messages Seen: {}", selected_chat.message_count)));
        details.push(Line::from(format!("Last Activity: {}", 
            chrono::DateTime::from_timestamp(selected_chat.last_seen, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        )));

        if !selected_chat.topics.is_empty() {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                "Topics (Forum Threads):",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            for topic in &selected_chat.topics {
                let topic_name = topic.name.as_deref().unwrap_or("Unknown Topic");
                details.push(Line::from(format!(
                    "  • Thread ID: {} - {} ({} messages)",
                    topic.thread_id, topic_name, topic.message_count
                )));
            }
        }

        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "Actions:",
            Style::default().fg(Color::Green),
        )));
        details.push(Line::from("  Enter - View messages for this chat"));
        details.push(Line::from("  e     - Export chat details as JSON"));

        let details_paragraph = Paragraph::new(details)
            .block(Block::bordered().title("Chat Details"))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(details_paragraph, details_area);
    }
}

