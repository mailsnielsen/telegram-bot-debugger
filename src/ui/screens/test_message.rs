use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::app::{App, InputFocus, TestMessageMode};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Mode selector
            Constraint::Length(6),  // Target info
            Constraint::Length(5),  // Message input
            Constraint::Min(0),     // Info section (no separate help section)
        ])
        .split(area);

    // Mode selector
    let mode_text = match app.ui.test_message_mode {
        TestMessageMode::SelectedChat => vec![
            Line::from(vec![
                Span::styled("[ Selected Chat ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("[ Manual Chat ID ]", Style::default().fg(Color::DarkGray)),
                Span::raw("  (Press Tab to switch)"),
            ]),
        ],
        TestMessageMode::ManualChatId => vec![
            Line::from(vec![
                Span::styled("[ Selected Chat ]", Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled("[ Manual Chat ID ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  (Press Tab to switch)"),
            ]),
        ],
    };

    let mode_paragraph = Paragraph::new(mode_text)
        .block(Block::bordered().title("Mode"));

    frame.render_widget(mode_paragraph, chunks[0]);

    // Target info based on mode
    let target_info = match app.ui.test_message_mode {
        TestMessageMode::SelectedChat => {
            if let Some(chat) = app.get_selected_chat() {
                vec![
                    Line::from(""),
                    Line::from(Span::styled("Target Chat:", Style::default().fg(Color::Cyan))),
                    Line::from(format!("  Name: {}", chat.chat.display_name())),
                    Line::from(format!("  Chat ID: {}", chat.chat.id)),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled("No chat selected", Style::default().fg(Color::Red))),
                    Line::from("Go to 'Discovery' screen (press 1) to select a chat first."),
                    Line::from("Or switch to Manual Chat ID mode (press Tab)."),
                ]
            }
        }
        TestMessageMode::ManualChatId => {
            // Show focus indicator for Chat ID field
            let chat_id_display = if app.ui.manual_chat_id_input.is_empty() { 
                "<type chat ID here>".to_string() 
            } else { 
                app.ui.manual_chat_id_input.clone() 
            };
            
            let focus_indicator = if app.ui.test_message_input_focus == InputFocus::ChatId {
                " «"
            } else {
                ""
            };

            vec![
                Line::from(""),
                Line::from(Span::styled("Enter Chat ID:", Style::default().fg(Color::Cyan))),
                Line::from(format!("  {}{}", chat_id_display, focus_indicator)),
                Line::from(""),
            ]
        }
    };

    // Target info with focus-based border styling
    let target_border_style = if app.ui.test_message_mode == TestMessageMode::ManualChatId 
        && app.ui.test_message_input_focus == InputFocus::ChatId {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    let info_paragraph = Paragraph::new(target_info)
        .block(Block::bordered().title("Target").border_style(target_border_style));

    frame.render_widget(info_paragraph, chunks[1]);

    // Message input with focus-based border styling
    let message_border_style = if app.ui.test_message_mode == TestMessageMode::SelectedChat 
        || app.ui.test_message_input_focus == InputFocus::MessageText {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    let message_title = if app.ui.test_message_mode == TestMessageMode::ManualChatId 
        && app.ui.test_message_input_focus == InputFocus::MessageText {
        "Message Text «"
    } else {
        "Message Text"
    };

    let input = Paragraph::new(app.ui.test_message_input.as_str())
        .block(Block::bordered().title(message_title).border_style(message_border_style))
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, chunks[2]);

    // Info section - shows current focus, available keys, and result
    let mut info_lines = vec![];

    // Show result if available
    if let Some(result) = &app.ui.test_message_result {
        let color = if result.starts_with("✓") {
            Color::Green
        } else {
            Color::Red
        };
        info_lines.push(Line::from(Span::styled(result.as_str(), Style::default().fg(color).add_modifier(Modifier::BOLD))));
        info_lines.push(Line::from(""));
    }

    // Show mode and controls
    match app.ui.test_message_mode {
        TestMessageMode::SelectedChat => {
            info_lines.push(Line::from(Span::styled("Mode: Selected Chat", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Controls:"));
            info_lines.push(Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" - Send message"),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("  Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" - Switch to Manual Chat ID mode"),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("  Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" - Go back"),
            ]));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Tip: Select a chat in Discovery (1) first."));
        }
        TestMessageMode::ManualChatId => {
            let current_focus = match app.ui.test_message_input_focus {
                InputFocus::ChatId => "Chat ID «",
                InputFocus::MessageText => "Message Text «",
            };

            info_lines.push(Line::from(Span::styled("Mode: Manual Chat ID", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            info_lines.push(Line::from(Span::styled(format!("Focus: {}", current_focus), Style::default().fg(Color::Green))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Controls:"));
            info_lines.push(Line::from(vec![
                Span::styled("  Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" - Switch focus between fields"),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("  Shift+Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" - Switch to Selected Chat mode"),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" - Send message"),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("  Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" - Go back"),
            ]));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Tip: Chat IDs can be negative (e.g., -1001234567890)"));
        }
    }

    let info_paragraph = Paragraph::new(info_lines)
        .block(Block::bordered().title("Info"));

    frame.render_widget(info_paragraph, chunks[3]);
}

