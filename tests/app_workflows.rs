//! Integration tests for complete application workflows.
//!
//! Tests end-to-end scenarios that combine multiple components.

use telegram_bot_debugger::{App, Screen};
use telegram_bot_debugger::telegram::{UpdateProcessor, Update, Message, Chat, User};
use telegram_bot_debugger::analytics::Statistics;

#[test]
fn test_discovery_workflow() {
    // Workflow: Receive updates → Process → Discover chats
    let mut processor = UpdateProcessor::new();
    
    let updates = vec![
        Update {
            update_id: 1,
            message: Some(Message {
                message_id: 1,
                from: Some(User {
                    id: 100,
                    is_bot: false,
                    first_name: "Alice".to_string(),
                    last_name: None,
                    username: Some("alice".to_string()),
                }),
                chat: Chat {
                    id: 100,
                    chat_type: "private".to_string(),
                    title: None,
                    username: None,
                    first_name: Some("Alice".to_string()),
                    last_name: None,
                },
                date: 1000,
                text: Some("Hello".to_string()),
                message_thread_id: None,
                reply_to_message: None,
                other: std::collections::HashMap::new(),
            }),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        },
    ];
    
    processor.process_updates(updates);
    
    let chats = processor.get_discovered_chats();
    assert_eq!(chats.len(), 1);
    assert_eq!(chats[0].chat.id, 100);
    assert_eq!(chats[0].message_count, 1);
}

#[test]
fn test_analytics_workflow() {
    // Workflow: Discover chats → Compute statistics → Display
    let mut processor = UpdateProcessor::new();
    
    let updates = vec![
        Update {
            update_id: 1,
            message: Some(create_test_message(100, "private", 10)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        },
        Update {
            update_id: 2,
            message: Some(create_test_message(200, "group", 20)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        },
    ];
    
    processor.process_updates(updates);
    
    let chats = processor.get_discovered_chats();
    let stats = Statistics::from_chats(&chats);
    
    assert_eq!(stats.total_chats, 2);
    assert_eq!(stats.total_messages, 2);
    assert_eq!(stats.messages_per_chat.len(), 2);
}

#[test]
fn test_screen_navigation_workflow() {
    let mut app = App::new().unwrap();
    
    // App starts in TokenInput or Home depending on cached token
    let initial_screen = app.ui.current_screen;
    assert!(initial_screen == Screen::TokenInput || initial_screen == Screen::Home);
    
    // Navigate through screens
    app.switch_screen(Screen::Discovery);
    assert_eq!(app.ui.current_screen, Screen::Discovery);
    
    app.switch_screen(Screen::Analytics);
    assert_eq!(app.ui.current_screen, Screen::Analytics);
    
    app.switch_screen(Screen::RawJson);
    assert_eq!(app.ui.current_screen, Screen::RawJson);
    
    app.switch_screen(Screen::Home);
    assert_eq!(app.ui.current_screen, Screen::Home);
}

#[test]
fn test_app_initialization() {
    let app = App::new();
    assert!(app.is_ok());
    
    let app = app.unwrap();
    assert!(!app.ui.should_quit);
    assert!(app.ui.needs_render);
}

// Helper function
fn create_test_message(chat_id: i64, chat_type: &str, message_id: i64) -> Message {
    Message {
        message_id,
        from: Some(User {
            id: chat_id,
            is_bot: false,
            first_name: "User".to_string(),
            last_name: None,
            username: None,
        }),
        chat: Chat {
            id: chat_id,
            chat_type: chat_type.to_string(),
            title: if chat_type != "private" {
                Some(format!("Test {}", chat_type))
            } else {
                None
            },
            username: None,
            first_name: if chat_type == "private" {
                Some("User".to_string())
            } else {
                None
            },
            last_name: None,
        },
        date: 1000,
        text: Some("Test message".to_string()),
        message_thread_id: None,
        reply_to_message: None,
        other: std::collections::HashMap::new(),
    }
}

