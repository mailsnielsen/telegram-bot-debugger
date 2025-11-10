//! Keyboard input handling and common key bindings.
//!
//! This module implements a Chain of Responsibility pattern for input handling,
//! allowing screen-specific handlers to process keys first, with fallback to
//! common navigation keys.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, Screen};

/// Represents whether a key was handled by a handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// The key was handled by the handler
    Handled,
    /// The key was not handled, should try next handler
    NotHandled,
}

/// Handles global navigation keys that work across all screens.
///
/// These keys provide consistent navigation regardless of the current screen:
/// - Number keys 1-4 for screen switching
/// - 'm' to open test message screen (send messages to any chat ID)
/// - 'q' to go back home
/// - 'h' for help
/// - F5 to toggle monitoring
/// - Esc to quit application
///
/// # Returns
///
/// - `KeyAction::Handled` if the key was processed
/// - `KeyAction::NotHandled` if the key should be handled elsewhere
pub async fn try_handle_global_keys(
    app: &mut App,
    key: KeyCode,
    _modifiers: KeyModifiers,
) -> Result<KeyAction> {
    match key {
        KeyCode::Esc => {
            // Esc provides hierarchical back navigation
            app.go_back();
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.switch_screen(Screen::Help);
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('1') => {
            app.switch_screen(Screen::Discovery);
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('2') => {
            app.switch_screen(Screen::Monitor);
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('3') => {
            app.switch_screen(Screen::Analytics);
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('4') => {
            app.switch_screen(Screen::RawJson);
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.switch_screen(Screen::TestMessage);
            Ok(KeyAction::Handled)
        }
        KeyCode::F(5) => {
            app.toggle_monitoring().await;
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled),
    }
}

/// Handles RawJson screen-specific keys (arrow keys and export).
///
/// Returns `KeyAction::Handled` if processed, otherwise `KeyAction::NotHandled`.
pub fn try_handle_raw_json_keys(app: &mut App, key: KeyCode) -> Result<KeyAction> {
    if app.ui.current_screen != Screen::RawJson {
        return Ok(KeyAction::NotHandled);
    }

    match key {
        KeyCode::Up => {
            app.next_update();
            Ok(KeyAction::Handled)
        }
        KeyCode::Down => {
            app.previous_update();
            Ok(KeyAction::Handled)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.export_selected_update()?;
            Ok(KeyAction::Handled)
        }
        _ => Ok(KeyAction::NotHandled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        App::new().unwrap()
    }

    // Global key handler tests
    #[tokio::test]
    async fn test_esc_hierarchical_navigation_from_discovery() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::Discovery;

        let result = try_handle_global_keys(&mut app, KeyCode::Esc, KeyModifiers::empty()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
        assert_eq!(app.ui.current_screen, Screen::Home);
    }

    #[tokio::test]
    async fn test_esc_from_home_quits() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::Home;

        let result = try_handle_global_keys(&mut app, KeyCode::Esc, KeyModifiers::empty()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
        assert!(app.ui.should_quit);
    }

    #[tokio::test]
    async fn test_esc_from_messages_goes_to_discovery() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::Messages;

        let result = try_handle_global_keys(&mut app, KeyCode::Esc, KeyModifiers::empty()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
        assert_eq!(app.ui.current_screen, Screen::Discovery);
    }

    #[tokio::test]
    async fn test_help_key() {
        let mut app = create_test_app();

        let result =
            try_handle_global_keys(&mut app, KeyCode::Char('h'), KeyModifiers::empty()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
        assert_eq!(app.ui.current_screen, Screen::Help);
    }

    #[tokio::test]
    async fn test_number_key_1() {
        let mut app = create_test_app();

        let result =
            try_handle_global_keys(&mut app, KeyCode::Char('1'), KeyModifiers::empty()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
        assert_eq!(app.ui.current_screen, Screen::Discovery);
    }

    #[tokio::test]
    async fn test_unknown_key_not_handled() {
        let mut app = create_test_app();

        let result =
            try_handle_global_keys(&mut app, KeyCode::Char('x'), KeyModifiers::empty()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::NotHandled);
    }

    // RawJson screen-specific tests
    #[test]
    fn test_raw_json_up_key() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::RawJson;

        let result = try_handle_raw_json_keys(&mut app, KeyCode::Up);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
    }

    #[test]
    fn test_raw_json_down_key() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::RawJson;

        let result = try_handle_raw_json_keys(&mut app, KeyCode::Down);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
    }

    #[test]
    fn test_raw_json_export_key() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::RawJson;

        let result = try_handle_raw_json_keys(&mut app, KeyCode::Char('e'));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::Handled);
    }

    #[test]
    fn test_raw_json_keys_on_wrong_screen() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::Home; // Not RawJson

        let result = try_handle_raw_json_keys(&mut app, KeyCode::Up);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::NotHandled);
    }

    #[test]
    fn test_raw_json_unknown_key() {
        let mut app = create_test_app();
        app.ui.current_screen = Screen::RawJson;

        let result = try_handle_raw_json_keys(&mut app, KeyCode::Char('x'));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyAction::NotHandled);
    }

    // KeyAction enum tests
    #[test]
    fn test_key_action_equality() {
        assert_eq!(KeyAction::Handled, KeyAction::Handled);
        assert_eq!(KeyAction::NotHandled, KeyAction::NotHandled);
        assert_ne!(KeyAction::Handled, KeyAction::NotHandled);
    }
}
