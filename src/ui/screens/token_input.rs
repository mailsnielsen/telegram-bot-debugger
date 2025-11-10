use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from("Welcome to Telegram Bot Debugger!"),
        Line::from(""),
        Line::from("Please enter your Telegram Bot Token:"),
    ])
    .block(Block::bordered().title("Setup"));

    frame.render_widget(instructions, chunks[0]);

    // Token input
    let input_style = if app.ui.token_error.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(app.ui.token_input.as_str())
        .block(Block::bordered().title("Bot Token"))
        .style(input_style);

    frame.render_widget(input, chunks[1]);

    // Error or instructions
    if let Some(error) = &app.ui.token_error {
        let error_paragraph = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::bordered().title("Error"));
        frame.render_widget(error_paragraph, chunks[2]);
    } else {
        let help = Paragraph::new("Press Enter to validate token | Type to input | Backspace to delete")
            .block(Block::bordered().title("Help"));
        frame.render_widget(help, chunks[2]);
    }
}

