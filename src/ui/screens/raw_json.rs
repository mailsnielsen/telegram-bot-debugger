use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph, Wrap},
    Frame,
};
use serde_json;

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    if app.telegram.raw_updates.is_empty() {
        let empty_message = Paragraph::new(vec![
            Line::from(""),
            Line::from("No updates received yet."),
            Line::from(""),
            Line::from("Updates will appear here as they are fetched."),
            Line::from(""),
            Line::from("To receive updates:"),
            Line::from("1. Press 'F5' to start Live Monitor"),
            Line::from("2. Send a message to your bot"),
            Line::from("3. The raw JSON will appear here"),
            Line::from(""),
            Line::from("This view shows the complete Telegram API response,"),
            Line::from("including all chat IDs, topic IDs, and other details."),
        ])
        .block(Block::bordered().title("Raw JSON Updates"));

        frame.render_widget(empty_message, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    // Update list
    let update_items: Vec<ListItem> = app
        .telegram.raw_updates
        .iter()
        .enumerate()
        .rev() // Show newest first
        .map(|(i, update)| {
            let indicator = if i == app.ui.selected_update_index {
                "→ "
            } else {
                "  "
            };

            let style = if i == app.ui.selected_update_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let update_type = update.get_update_type();

            let content = format!("{}Update {} ({})", indicator, update.update_id, update_type);

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let update_list = List::new(update_items).block(
        Block::bordered()
            .title("Updates (↑/↓ to navigate)"),
    );

    frame.render_widget(update_list, chunks[0]);

    // JSON details
    if let Some(selected_update) = app.telegram.get_selected_update(app.ui.selected_update_index) {
        let json_str = match serde_json::to_string_pretty(selected_update.as_ref()) {
            Ok(json) => json,
            Err(e) => format!("Error serializing JSON: {e}"),
        };

        // Split into lines for display
        let lines: Vec<Line> = json_str.lines().map(Line::from).collect();

        let json_paragraph = Paragraph::new(lines)
            .block(Block::bordered().title("JSON Details (Press 'e' to export)"))
            .wrap(Wrap { trim: false })
            .green();

        frame.render_widget(json_paragraph, chunks[1]);
    }
}

