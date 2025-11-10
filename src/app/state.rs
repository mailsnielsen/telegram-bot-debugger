//! UI state management.
//!
//! Manages screen navigation and UI-specific state like selections and scroll positions.

use crate::analytics::Statistics;

/// Represents the current screen/mode of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    TokenInput,
    Home,
    Discovery,
    Messages,
    TestMessage,
    Monitor,
    Analytics,
    RawJson,
    Help,
}

/// Mode for sending test messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMessageMode {
    /// Send to the currently selected chat
    SelectedChat,
    /// Send to a manually entered chat ID
    ManualChatId,
}

/// Tracks which input field has focus in the Test Message screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFocus {
    /// Focus on Chat ID input field
    ChatId,
    /// Focus on Message Text input field
    MessageText,
}

/// Manages all UI-related state.
///
/// Handles screen navigation, item selection, and scroll positions.
#[derive(Debug)]
pub struct UiState {
    pub current_screen: Screen,
    pub previous_screen: Option<Screen>,
    pub should_quit: bool,
    pub needs_render: bool,

    // Navigation state
    pub selected_chat_index: usize,
    pub selected_message_index: usize,
    pub selected_update_index: usize,

    // Token input screen state
    pub token_input: String,
    pub token_error: Option<String>,

    // Test message screen state
    pub test_message_input: String,
    pub test_message_result: Option<String>,
    pub test_message_mode: TestMessageMode,
    pub manual_chat_id_input: String,
    pub test_message_input_focus: InputFocus,

    // Analytics cache
    pub statistics: Option<Statistics>,

    // Status messages
    pub status_message: Option<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Home,
            previous_screen: None,
            should_quit: false,
            needs_render: true,
            selected_chat_index: 0,
            selected_message_index: 0,
            selected_update_index: 0,
            token_input: String::new(),
            token_error: None,
            test_message_input: String::new(),
            test_message_result: None,
            test_message_mode: TestMessageMode::SelectedChat,
            manual_chat_id_input: String::new(),
            test_message_input_focus: InputFocus::MessageText, // Start with MessageText in SelectedChat mode
            statistics: None,
            status_message: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Marks that the UI needs to be redrawn.
    pub fn mark_dirty(&mut self) {
        self.needs_render = true;
    }

    /// Clears the dirty flag after rendering.
    pub fn clear_dirty(&mut self) {
        self.needs_render = false;
    }

    pub fn switch_screen(&mut self, screen: Screen) {
        self.previous_screen = Some(self.current_screen);
        self.current_screen = screen;
        self.clear_status();
        self.mark_dirty();
    }

    pub fn go_back(&mut self) {
        // Hierarchical back navigation
        let target = match self.current_screen {
            Screen::Messages => Some(Screen::Discovery),
            Screen::Discovery
            | Screen::Monitor
            | Screen::Analytics
            | Screen::RawJson
            | Screen::TestMessage
            | Screen::Help => Some(Screen::Home),
            Screen::Home => {
                // On home, Esc quits
                self.should_quit = true;
                None
            }
            Screen::TokenInput => None, // Can't go back from token input
        };

        if let Some(screen) = target {
            self.switch_screen(screen);
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.mark_dirty();
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn next_chat(&mut self, chat_count: usize) {
        if chat_count > 0 {
            self.selected_chat_index = (self.selected_chat_index + 1) % chat_count;
            self.mark_dirty();
        }
    }

    pub fn previous_chat(&mut self, chat_count: usize) {
        if chat_count > 0 {
            if self.selected_chat_index == 0 {
                self.selected_chat_index = chat_count - 1;
            } else {
                self.selected_chat_index -= 1;
            }
            self.mark_dirty();
        }
    }

    pub fn next_message(&mut self, message_count: usize) {
        if message_count > 0 {
            self.selected_message_index = (self.selected_message_index + 1) % message_count;
            self.mark_dirty();
        }
    }

    pub fn previous_message(&mut self, message_count: usize) {
        if message_count > 0 {
            if self.selected_message_index == 0 {
                self.selected_message_index = message_count - 1;
            } else {
                self.selected_message_index -= 1;
            }
            self.mark_dirty();
        }
    }

    pub fn next_update(&mut self, update_count: usize) {
        if update_count > 0 {
            self.selected_update_index = (self.selected_update_index + 1) % update_count;
            self.mark_dirty();
        }
    }

    pub fn previous_update(&mut self, update_count: usize) {
        if update_count > 0 {
            if self.selected_update_index == 0 {
                self.selected_update_index = update_count - 1;
            } else {
                self.selected_update_index -= 1;
            }
            self.mark_dirty();
        }
    }

    pub fn toggle_test_message_mode(&mut self) {
        self.test_message_mode = match self.test_message_mode {
            TestMessageMode::SelectedChat => {
                // When switching to ManualChatId, start with ChatId field focused
                self.test_message_input_focus = InputFocus::ChatId;
                TestMessageMode::ManualChatId
            }
            TestMessageMode::ManualChatId => {
                // When switching to SelectedChat, focus on MessageText (only field)
                self.test_message_input_focus = InputFocus::MessageText;
                TestMessageMode::SelectedChat
            }
        };
        self.mark_dirty();
    }

    pub fn toggle_input_focus(&mut self) {
        self.test_message_input_focus = match self.test_message_input_focus {
            InputFocus::ChatId => InputFocus::MessageText,
            InputFocus::MessageText => InputFocus::ChatId,
        };
        self.mark_dirty();
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_creation() {
        let state = UiState::new();
        assert_eq!(state.current_screen, Screen::Home);
        assert!(!state.should_quit);
    }

    #[test]
    fn test_screen_switching() {
        let mut state = UiState::new();
        state.switch_screen(Screen::Discovery);
        assert_eq!(state.current_screen, Screen::Discovery);
        assert_eq!(state.previous_screen, Some(Screen::Home));
    }

    #[test]
    fn test_go_back_from_discovery() {
        let mut state = UiState::new();
        state.switch_screen(Screen::Discovery);
        state.go_back();
        assert_eq!(state.current_screen, Screen::Home);
    }

    #[test]
    fn test_go_back_from_messages_to_discovery() {
        let mut state = UiState::new();
        state.switch_screen(Screen::Discovery);
        state.switch_screen(Screen::Messages);
        state.go_back();
        assert_eq!(state.current_screen, Screen::Discovery);
    }

    #[test]
    fn test_go_back_from_home_quits() {
        let mut state = UiState::new();
        state.current_screen = Screen::Home;
        state.go_back();
        assert!(state.should_quit);
    }

    #[test]
    fn test_quit() {
        let mut state = UiState::new();
        assert!(!state.should_quit);
        state.quit();
        assert!(state.should_quit);
    }

    #[test]
    fn test_status_message() {
        let mut state = UiState::new();
        assert!(state.status_message.is_none());

        state.set_status("Test".to_string());
        assert_eq!(state.status_message, Some("Test".to_string()));

        state.clear_status();
        assert!(state.status_message.is_none());
    }

    #[test]
    fn test_chat_navigation() {
        let mut state = UiState::new();

        // Test with 3 chats
        state.next_chat(3);
        assert_eq!(state.selected_chat_index, 1);

        state.next_chat(3);
        assert_eq!(state.selected_chat_index, 2);

        state.next_chat(3); // Wrap around
        assert_eq!(state.selected_chat_index, 0);

        state.previous_chat(3); // Wrap back
        assert_eq!(state.selected_chat_index, 2);
    }

    // Boundary conditions
    #[test]
    fn test_navigation_with_zero_items() {
        let mut state = UiState::new();

        state.next_chat(0);
        assert_eq!(state.selected_chat_index, 0);

        state.previous_chat(0);
        assert_eq!(state.selected_chat_index, 0);

        state.next_message(0);
        assert_eq!(state.selected_message_index, 0);

        state.previous_message(0);
        assert_eq!(state.selected_message_index, 0);
    }

    #[test]
    fn test_navigation_with_single_item() {
        let mut state = UiState::new();

        state.next_chat(1);
        assert_eq!(state.selected_chat_index, 0); // Wraps to itself

        state.previous_chat(1);
        assert_eq!(state.selected_chat_index, 0);
    }

    #[test]
    fn test_previous_chat_wraparound_from_zero() {
        let mut state = UiState::new();
        state.selected_chat_index = 0;

        state.previous_chat(5);
        assert_eq!(state.selected_chat_index, 4);
    }

    #[test]
    fn test_screen_switching_clears_status() {
        let mut state = UiState::new();
        state.set_status("Test status".to_string());
        assert!(state.status_message.is_some());

        state.switch_screen(Screen::Discovery);
        assert!(state.status_message.is_none());
    }

    #[test]
    fn test_screen_switching_marks_dirty() {
        let mut state = UiState::new();
        state.needs_render = false;

        state.switch_screen(Screen::Analytics);
        assert!(state.needs_render);
    }

    #[test]
    fn test_set_status_marks_dirty() {
        let mut state = UiState::new();
        state.needs_render = false;

        state.set_status("Test".to_string());
        assert!(state.needs_render);
    }

    #[test]
    fn test_mark_dirty_and_clear_dirty() {
        let mut state = UiState::new();

        state.needs_render = false;
        state.mark_dirty();
        assert!(state.needs_render);

        state.clear_dirty();
        assert!(!state.needs_render);
    }

    #[test]
    fn test_test_message_mode_toggle() {
        let mut state = UiState::new();
        assert_eq!(state.test_message_mode, TestMessageMode::SelectedChat);
        assert_eq!(state.test_message_input_focus, InputFocus::MessageText);

        state.toggle_test_message_mode();
        assert_eq!(state.test_message_mode, TestMessageMode::ManualChatId);
        assert_eq!(state.test_message_input_focus, InputFocus::ChatId); // Focus reset to ChatId

        state.toggle_test_message_mode();
        assert_eq!(state.test_message_mode, TestMessageMode::SelectedChat);
        assert_eq!(state.test_message_input_focus, InputFocus::MessageText); // Focus back to MessageText
    }

    #[test]
    fn test_input_focus_toggle() {
        let mut state = UiState::new();
        state.test_message_mode = TestMessageMode::ManualChatId;
        state.test_message_input_focus = InputFocus::ChatId;

        state.toggle_input_focus();
        assert_eq!(state.test_message_input_focus, InputFocus::MessageText);

        state.toggle_input_focus();
        assert_eq!(state.test_message_input_focus, InputFocus::ChatId);
    }

    #[test]
    fn test_default_state() {
        let state = UiState::default();
        assert_eq!(state.current_screen, Screen::Home);
        assert!(!state.should_quit);
        assert!(state.needs_render);
        assert_eq!(state.selected_chat_index, 0);
    }
}
