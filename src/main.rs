use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;
use tokio::time::sleep;

use telegram_bot_debugger::app::{App, Screen};
use telegram_bot_debugger::input::{
    KeyAction, try_handle_global_keys, try_handle_raw_json_keys, try_handle_webhook_keys,
};
use telegram_bot_debugger::ui::render_frame;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

/// Main application event loop.
///
/// Handles rendering, input processing, and background task coordination.
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Render only when needed (dirty flag pattern)
        // This significantly reduces CPU usage by avoiding unnecessary redraws
        if app.ui.needs_render {
            terminal.draw(|f| render_frame(f, app))?;
            app.clear_dirty();
        }

        // Event polling with longer timeout to reduce CPU usage
        // 250ms is responsive enough for user input while being CPU-friendly
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            // Two-phase handling: Screen-specific first, then global fallback
            let handled = match app.ui.current_screen {
                Screen::TokenInput => handle_token_input(app, key.code).await?,
                Screen::TestMessage => handle_test_message(app, key.code, key.modifiers).await?,
                Screen::Monitor => handle_monitor(app, key.code).await?,
                Screen::Discovery => handle_discovery(app, key.code, key.modifiers).await?,
                Screen::Messages => handle_messages(app, key.code, key.modifiers).await?,
                _ => KeyAction::NotHandled, // Home, Help, Analytics, RawJson fall through
            };

            // If not handled by screen-specific handler, try global keys
            if handled == KeyAction::NotHandled {
                try_handle_global_keys(app, key.code, key.modifiers).await?;
            }

            // Try RawJson-specific keys if applicable
            try_handle_raw_json_keys(app, key.code)?;

            // Try Webhook Management-specific keys if applicable
            try_handle_webhook_keys(app, key.code).await?;

            // Mark for re-render after input processing
            app.mark_dirty();
        }

        // Process any updates received from the background monitoring task
        // This needs to run regularly to integrate async updates
        if app.monitoring.is_active() {
            app.process_received_updates();
        }

        // Exit loop if quit was requested
        if app.ui.should_quit {
            break;
        }

        // Small sleep only when monitoring to check for updates
        // Otherwise the event poll provides the timing
        if app.monitoring.is_active() {
            sleep(Duration::from_millis(100)).await;
        }
    }

    // Cleanup: Stop background monitoring task if it's still running
    if app.monitoring.is_active() {
        app.stop_monitoring().await;
    }

    Ok(())
}

/// Handles input on the token input screen.
///
/// Returns `KeyAction::NotHandled` to allow global keys to work.
async fn handle_token_input(app: &mut App, key: KeyCode) -> Result<KeyAction> {
    match key {
        KeyCode::Enter => {
            app.validate_and_save_token().await?;
            Ok(KeyAction::Handled)
        }
        KeyCode::Char(c) => {
            app.ui.token_input.push(c);
            app.ui.token_error = None;
            app.mark_dirty();
            Ok(KeyAction::Handled)
        }
        KeyCode::Backspace => {
            app.ui.token_input.pop();
            app.ui.token_error = None;
            app.mark_dirty();
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled),
    }
}

/// Handles input on the test message screen.
///
/// Supports both selected chat mode and manual chat ID entry.
/// Returns `KeyAction::Handled` for screen-specific keys, `NotHandled` for global keys.
async fn handle_test_message(
    app: &mut App,
    key: KeyCode,
    modifiers: KeyModifiers,
) -> Result<KeyAction> {
    use telegram_bot_debugger::app::{InputFocus, TestMessageMode};

    match key {
        KeyCode::Enter => {
            // Send message from any field/focus
            app.send_test_message().await?;
            Ok(KeyAction::Handled)
        }
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+Tab switches between Selected/Manual mode
                app.toggle_test_message_mode();
            } else if app.ui.test_message_mode == TestMessageMode::ManualChatId {
                // Tab cycles focus between Chat ID ↔ Message Text fields
                app.toggle_input_focus();
            } else {
                // In SelectedChat mode, Tab switches to Manual mode
                app.toggle_test_message_mode();
            }
            Ok(KeyAction::Handled)
        }
        KeyCode::BackTab => {
            // BackTab is how many terminals send Shift+Tab
            app.toggle_test_message_mode();
            Ok(KeyAction::Handled)
        }
        KeyCode::Char(c) => {
            // Insert character into the currently focused field
            match app.ui.test_message_mode {
                TestMessageMode::ManualChatId => match app.ui.test_message_input_focus {
                    InputFocus::ChatId => {
                        app.ui.manual_chat_id_input.push(c);
                    }
                    InputFocus::MessageText => {
                        app.ui.test_message_input.push(c);
                    }
                },
                TestMessageMode::SelectedChat => {
                    // Only message field available
                    app.ui.test_message_input.push(c);
                }
            }
            Ok(KeyAction::Handled)
        }
        KeyCode::Backspace => {
            // Delete from the currently focused field
            match app.ui.test_message_mode {
                TestMessageMode::ManualChatId => match app.ui.test_message_input_focus {
                    InputFocus::ChatId => {
                        app.ui.manual_chat_id_input.pop();
                    }
                    InputFocus::MessageText => {
                        app.ui.test_message_input.pop();
                    }
                },
                TestMessageMode::SelectedChat => {
                    app.ui.test_message_input.pop();
                }
            }
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled), // q and Esc handled by global handler
    }
}

/// Handles input on the monitor screen (live updates view).
///
/// Handles monitor-specific keys (m, p, c), delegates global keys to common handler.
async fn handle_monitor(app: &mut App, key: KeyCode) -> Result<KeyAction> {
    match key {
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.toggle_monitoring().await;
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.toggle_monitor_pause();
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.monitoring.messages.clear();
            app.mark_dirty();
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled), // Let global handler process navigation keys
    }
}

/// Handles input on the discovery screen (chat list).
///
/// Handles navigation, export, and Enter to view messages. Global keys handled by common handler.
async fn handle_discovery(
    app: &mut App,
    key: KeyCode,
    _modifiers: KeyModifiers,
) -> Result<KeyAction> {
    match key {
        KeyCode::Up => {
            app.previous_chat();
            Ok(KeyAction::Handled)
        }
        KeyCode::Down => {
            app.next_chat();
            Ok(KeyAction::Handled)
        }
        KeyCode::Enter => {
            // Navigate to messages screen
            app.ui.selected_message_index = 0; // Reset message index
            app.switch_screen(Screen::Messages); // This calls mark_dirty internally
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.export_selected_chat()?;
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled), // Let global handler process navigation keys
    }
}

/// Handles input on the messages screen (viewing messages for a specific chat).
///
/// Handles message navigation, export, and 'm' to send test message. Global keys handled by common handler.
async fn handle_messages(
    app: &mut App,
    key: KeyCode,
    _modifiers: KeyModifiers,
) -> Result<KeyAction> {
    match key {
        KeyCode::Up => {
            app.previous_message();
            Ok(KeyAction::Handled)
        }
        KeyCode::Down => {
            app.next_message();
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.export_selected_message()?;
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled), // Let global handler process navigation keys (including Esc)
    }
}
