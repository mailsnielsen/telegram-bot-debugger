use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let Some(stats) = &app.ui.statistics else {
        let empty = Paragraph::new("No statistics available yet.")
            .block(Block::bordered().title("Analytics"));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    // Overview
    let overview_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "📊 Statistics Overview",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Total Chats: {}", stats.total_chats)),
        Line::from(format!("Total Messages: {}", stats.total_messages)),
        Line::from(format!("Total Topics: {}", stats.total_topics)),
        Line::from(""),
    ];

    let overview = Paragraph::new(overview_text)
        .block(Block::bordered().title("Analytics"));

    frame.render_widget(overview, chunks[0]);

    // Split bottom section into 3 columns
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    // Top chats
    let mut top_chats_items = vec![ListItem::new(Line::from(Span::styled(
        "Most Active Chats:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )))];

    for (i, (chat_name, count)) in stats.get_top_chats(10).iter().enumerate() {
        top_chats_items.push(ListItem::new(Line::from(format!(
            "{}. {} - {} messages",
            i + 1,
            chat_name,
            count
        ))));
    }

    let top_chats = List::new(top_chats_items)
        .block(Block::bordered().title("Top Chats"));

    frame.render_widget(top_chats, bottom_chunks[0]);

    // Chat type distribution
    let mut type_items = vec![ListItem::new(Line::from(Span::styled(
        "Chat Types:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )))];

    for (chat_type, count) in &stats.chat_type_distribution {
        let color = match chat_type.as_str() {
            "private" => Color::Green,
            "group" | "supergroup" => Color::Blue,
            "channel" => Color::Yellow,
            _ => Color::White,
        };

        type_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{chat_type}: "), Style::default().fg(color)),
            Span::raw(format!("{count}")),
        ])));
    }

    let type_list = List::new(type_items)
        .block(Block::bordered().title("Types"));

    frame.render_widget(type_list, bottom_chunks[1]);

    // Hourly distribution
    let mut hourly_items = vec![ListItem::new(Line::from(Span::styled(
        "Activity by Hour:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )))];

    // Show only hours with activity
    let active_hours: Vec<_> = stats
        .hourly_distribution
        .iter()
        .filter(|(_, count)| *count > 0)
        .take(10)
        .collect();

    if active_hours.is_empty() {
        hourly_items.push(ListItem::new("No data yet"));
    } else {
        for (hour, count) in active_hours {
            hourly_items.push(ListItem::new(Line::from(format!(
                "{hour:02}:00 - {count} msgs"
            ))));
        }
    }

    let hourly_list = List::new(hourly_items)
        .block(Block::bordered().title("Hourly"));

    frame.render_widget(hourly_list, bottom_chunks[2]);
}

