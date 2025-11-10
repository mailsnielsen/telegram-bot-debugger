use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let selected_chat = app.get_selected_chat();
    
    if selected_chat.is_none() {
        let empty_message = Paragraph::new(vec![
            Line::from(""),
            Line::from("No chat selected."),
            Line::from(""),
            Line::from("Please go back to Discovery screen and select a chat."),
            Line::from(""),
            Line::from("Press Esc to go back."),
        ])
        .block(Block::bordered().title("Messages"));

        frame.render_widget(empty_message, area);
        return;
    }

    let chat = selected_chat.unwrap();
    let messages = app.telegram.get_messages_for_chat(chat.chat.id);

    if messages.is_empty() {
        let empty_message = Paragraph::new(vec![
            Line::from(""),
            Line::from(format!("No messages found for: {}", chat.chat.display_name())),
            Line::from(""),
            Line::from("Messages are shown from the last 50 updates received."),
            Line::from("Use the Live Monitor (F5) to collect more messages."),
            Line::from(""),
            Line::from("Press Esc to go back."),
        ])
        .block(Block::bordered().title(format!("Messages - {}", chat.chat.display_name())));

        frame.render_widget(empty_message, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Message list
    let message_items: Vec<ListItem> = messages
        .iter()
        .enumerate()
        .map(|(i, update)| {
            let indicator = if i == app.ui.selected_message_index {
                "→ "
            } else {
                "  "
            };

            let style = if i == app.ui.selected_message_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Extract info based on update type
            let update_type = update.get_update_type();
            
            let (timestamp, sender, text_preview) = if let Some(message) = &update.message {
                let ts = chrono::DateTime::from_timestamp(message.date, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let sender_name = message.from.as_ref()
                    .map(|u| u.username.clone().unwrap_or_else(|| u.first_name.clone()))
                    .unwrap_or_else(|| "Unknown".to_string());
                let text = message.text.as_deref().unwrap_or("[No text]");
                let preview = if text.len() > 30 {
                    format!("{}...", &text[..30])
                } else {
                    text.to_string()
                };
                (ts, sender_name, preview)
            } else if let Some(channel_post) = &update.channel_post {
                let ts = chrono::DateTime::from_timestamp(channel_post.date, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let text = channel_post.text.as_deref().unwrap_or("[No text]");
                let preview = if text.len() > 30 {
                    format!("{}...", &text[..30])
                } else {
                    text.to_string()
                };
                (ts, "Channel".to_string(), preview)
            } else if let Some(edited_message) = &update.edited_message {
                let ts = chrono::DateTime::from_timestamp(edited_message.date, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let sender_name = edited_message.from.as_ref()
                    .map(|u| u.username.clone().unwrap_or_else(|| u.first_name.clone()))
                    .unwrap_or_else(|| "Edited".to_string());
                let text = edited_message.text.as_deref().unwrap_or("[No text]");
                let preview = if text.len() > 30 {
                    format!("{}...", &text[..30])
                } else {
                    text.to_string()
                };
                (ts, format!("{} (edited)", sender_name), preview)
            } else {
                // For other update types, show the type and update ID
                ("N/A".to_string(), update_type.clone(), format!("[{}]", update_type))
            };

            let content = format!("{indicator}{timestamp} | {sender} | {text_preview}");
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let message_list = List::new(message_items)
        .block(Block::bordered().title("Messages (↑/↓ to navigate | m to send message | e to export | Esc to go back)"));

    frame.render_widget(message_list, chunks[0]);

    // Message details
    if let Some(selected_message) = app.get_selected_message_for_current_chat() {
        let mut details = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Message Details:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // Show update type at the top
        let update_type = selected_message.get_update_type();
        details.push(Line::from(Span::styled(
            format!("Type: {}", update_type),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        details.push(Line::from(format!("Update ID: {}", selected_message.update_id)));
        details.push(Line::from(""));

        if let Some(message) = &selected_message.message {
            details.push(Line::from(format!("Message ID: {}", message.message_id)));
            details.push(Line::from(format!("Chat ID: {}", message.chat.id)));
            
            if let Some(thread_id) = message.message_thread_id {
                details.push(Line::from(format!("Thread ID: {thread_id}")));
            }

            details.push(Line::from(format!("Date: {}", 
                chrono::DateTime::from_timestamp(message.date, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            )));

            if let Some(from) = &message.from {
                details.push(Line::from(""));
                details.push(Line::from(Span::styled(
                    "From:",
                    Style::default().fg(Color::Yellow),
                )));
                details.push(Line::from(format!("  User ID: {}", from.id)));
                details.push(Line::from(format!("  First Name: {}", from.first_name)));
                if let Some(last_name) = &from.last_name {
                    details.push(Line::from(format!("  Last Name: {last_name}")));
                }
                if let Some(username) = &from.username {
                    details.push(Line::from(format!("  Username: @{username}")));
                }
                details.push(Line::from(format!("  Is Bot: {}", from.is_bot)));
            }

            if let Some(text) = &message.text {
                details.push(Line::from(""));
                details.push(Line::from(Span::styled(
                    "Text:",
                    Style::default().fg(Color::Green),
                )));
                details.push(Line::from(format!("  {text}")));
            }
        } else if let Some(channel_post) = &selected_message.channel_post {
            details.push(Line::from(format!("Message ID: {}", channel_post.message_id)));
            details.push(Line::from(format!("Chat ID: {}", channel_post.chat.id)));
            details.push(Line::from(format!("Date: {}", 
                chrono::DateTime::from_timestamp(channel_post.date, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            )));

            if let Some(text) = &channel_post.text {
                details.push(Line::from(""));
                details.push(Line::from(Span::styled(
                    "Text:",
                    Style::default().fg(Color::Green),
                )));
                details.push(Line::from(format!("  {text}")));
            }
        } else if let Some(edited_message) = &selected_message.edited_message {
            details.push(Line::from(format!("Message ID: {}", edited_message.message_id)));
            details.push(Line::from(format!("Chat ID: {}", edited_message.chat.id)));
            
            if let Some(thread_id) = edited_message.message_thread_id {
                details.push(Line::from(format!("Thread ID: {thread_id}")));
            }

            details.push(Line::from(format!("Date: {}", 
                chrono::DateTime::from_timestamp(edited_message.date, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            )));

            if let Some(from) = &edited_message.from {
                details.push(Line::from(""));
                details.push(Line::from(Span::styled(
                    "From:",
                    Style::default().fg(Color::Yellow),
                )));
                details.push(Line::from(format!("  User ID: {}", from.id)));
                details.push(Line::from(format!("  First Name: {}", from.first_name)));
                if let Some(last_name) = &from.last_name {
                    details.push(Line::from(format!("  Last Name: {last_name}")));
                }
                if let Some(username) = &from.username {
                    details.push(Line::from(format!("  Username: @{username}")));
                }
                details.push(Line::from(format!("  Is Bot: {}", from.is_bot)));
            }

            if let Some(text) = &edited_message.text {
                details.push(Line::from(""));
                details.push(Line::from(Span::styled(
                    "Text:",
                    Style::default().fg(Color::Green),
                )));
                details.push(Line::from(format!("  {text}")));
            }
        } else {
            // For other update types, show basic info and the raw data
            details.push(Line::from(Span::styled(
                "This update type doesn't have standard message fields.",
                Style::default().fg(Color::Yellow),
            )));
            details.push(Line::from(""));
            details.push(Line::from("Check the Raw JSON view (press 4) for complete details."));
        }

        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "Actions:",
            Style::default().fg(Color::Magenta),
        )));
        details.push(Line::from("  m - Send test message to this chat"));
        details.push(Line::from("  e - Export this message as JSON"));

        let details_paragraph = Paragraph::new(details)
            .block(Block::bordered().title("Details"))
            .wrap(Wrap { trim: false });

        frame.render_widget(details_paragraph, chunks[1]);
    }
}

