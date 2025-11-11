use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let [stats_area, menu_area] = Layout::vertical([
        Constraint::Length(6), // Stats box
        Constraint::Min(0),    // Navigation menu
    ])
    .areas(area);

    // Welcome section
    let chats = app.telegram.get_discovered_chats();
    let total_messages: usize = chats.iter().map(|c| c.message_count).sum();
    let total_topics: usize = chats.iter().map(|c| c.topics.len()).sum();

    let welcome_text = vec![
        Line::from(Span::styled(
            "📊 Statistics Overview".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "  Chats: {}  |  Messages: {}  |  Topics: {}",
            chats.len(),
            total_messages,
            total_topics
        )),
        Line::from(""),
    ];

    let welcome = Paragraph::new(welcome_text).block(Block::bordered().title("Dashboard"));

    frame.render_widget(welcome, stats_area);

    // Navigation menu
    let monitoring_status = if app.monitoring.is_active() {
        Span::styled(
            " [ACTIVE]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" [INACTIVE]", Style::default().fg(Color::DarkGray))
    };

    let menu_items = vec![
        ListItem::new(Line::from(vec![Span::styled(
            "Main Screens:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  1 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Discovery           - Browse chats & topics, view messages"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  2 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Live Monitor        - Real-time message monitoring"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  3 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Analytics           - View statistics and activity"),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![Span::styled(
            "Debug Tools:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  4 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Raw JSON Output     - View complete Telegram API responses"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  5 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Webhook Management  - Configure webhooks and polling mode"),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![Span::styled(
            "Monitoring:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  F5",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Toggle Monitoring"),
            monitoring_status,
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("Tip:", Style::default().fg(Color::Magenta)),
            Span::raw(" Press F5 to activate monitoring, it runs in background!"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::raw(" Press 'e' in various screens to export data as JSON"),
        ])),
    ];

    let menu = List::new(menu_items)
        .block(Block::bordered().title("Navigation"))
        .white();

    frame.render_widget(menu, menu_area);
}
