//! Test utilities and fixtures for testing.
//!
//! This module provides common test helpers, mock data factories, and utilities
//! for testing Telegram bot functionality.

#![cfg(test)]

use crate::telegram::{Chat, DiscoveredChat, Message, TopicInfo, Update, User};
use std::sync::Arc;

/// Creates a test User with default values.
///
/// # Arguments
///
/// * `id` - User ID
/// * `username` - Optional username
pub fn create_test_user(id: i64, username: Option<String>) -> User {
    User {
        id,
        is_bot: false,
        first_name: "Test".to_string(),
        last_name: Some("User".to_string()),
        username,
    }
}

/// Creates a test Chat with default values.
///
/// # Arguments
///
/// * `id` - Chat ID
/// * `chat_type` - Type of chat ("private", "group", "supergroup", "channel")
pub fn create_test_chat(id: i64, chat_type: &str) -> Chat {
    Chat {
        id,
        chat_type: chat_type.to_string(),
        title: if chat_type != "private" {
            Some(format!("Test {chat_type}"))
        } else {
            None
        },
        username: None,
        first_name: if chat_type == "private" {
            Some("John".to_string())
        } else {
            None
        },
        last_name: if chat_type == "private" {
            Some("Doe".to_string())
        } else {
            None
        },
    }
}

/// Creates a test Message with default values.
///
/// # Arguments
///
/// * `chat_id` - ID of the chat
/// * `message_id` - Message ID
/// * `date` - Unix timestamp
pub fn create_test_message(chat_id: i64, message_id: i64, date: i64) -> Message {
    Message {
        message_id,
        from: Some(create_test_user(1, Some("testuser".to_string()))),
        chat: create_test_chat(chat_id, "private"),
        date,
        text: Some("Test message".to_string()),
        message_thread_id: None,
        reply_to_message: None,
        other: std::collections::HashMap::new(),
    }
}

/// Creates a test Message with a specific thread ID for forum groups.
///
/// # Arguments
///
/// * `chat_id` - ID of the chat
/// * `message_id` - Message ID
/// * `date` - Unix timestamp
/// * `thread_id` - Topic/thread ID
#[allow(dead_code)]
pub fn create_test_message_with_thread(
    chat_id: i64,
    message_id: i64,
    date: i64,
    thread_id: i64,
) -> Message {
    let mut message = create_test_message(chat_id, message_id, date);
    message.message_thread_id = Some(thread_id);
    message.chat = create_test_chat(chat_id, "supergroup");
    message
}


/// Creates a test DiscoveredChat with default values.
///
/// # Arguments
///
/// * `chat_id` - Chat ID
/// * `chat_type` - Type of chat
/// * `message_count` - Number of messages
pub fn create_test_discovered_chat(
    chat_id: i64,
    chat_type: &str,
    message_count: usize,
) -> DiscoveredChat {
    DiscoveredChat {
        chat: create_test_chat(chat_id, chat_type),
        last_seen: 1000,
        message_count,
        topics: Vec::new(),
    }
}

/// Creates a test DiscoveredChat with topics.
///
/// # Arguments
///
/// * `chat_id` - Chat ID
/// * `topics` - Vector of topic infos
pub fn create_test_discovered_chat_with_topics(
    chat_id: i64,
    topics: Vec<TopicInfo>,
) -> DiscoveredChat {
    DiscoveredChat {
        chat: create_test_chat(chat_id, "supergroup"),
        last_seen: 1000,
        message_count: topics.iter().map(|t| t.message_count).sum(),
        topics,
    }
}

/// Creates a test TopicInfo.
///
/// # Arguments
///
/// * `thread_id` - Thread ID
/// * `name` - Optional topic name
/// * `message_count` - Number of messages in this topic
pub fn create_test_topic(thread_id: i64, name: Option<String>, message_count: usize) -> TopicInfo {
    TopicInfo {
        thread_id,
        name,
        message_count,
        last_seen: 1000,
    }
}


/// Wraps an Update in an Arc for testing scenarios that use Arc<Update>.
#[allow(dead_code)]
pub fn arc_update(update: Update) -> Arc<Update> {
    Arc::new(update)
}

/// Creates a mock Telegram API JSON response for getMe.
#[allow(dead_code)]
pub fn mock_get_me_response(bot_id: i64, bot_username: &str) -> String {
    format!(
        r#"{{
            "ok": true,
            "result": {{
                "id": {bot_id},
                "is_bot": true,
                "first_name": "Test Bot",
                "username": "{bot_username}"
            }}
        }}"#
    )
}

/// Creates a mock Telegram API JSON response for getUpdates.
#[allow(dead_code)]
pub fn mock_get_updates_response(updates: Vec<Update>) -> String {
    let updates_json = serde_json::to_string(&updates).unwrap();
    format!(
        r#"{{
            "ok": true,
            "result": {updates_json}
        }}"#
    )
}

/// Creates a mock Telegram API JSON response for sendMessage.
#[allow(dead_code)]
pub fn mock_send_message_response(success: bool, message_id: Option<i64>) -> String {
    if success {
        let mid = message_id.unwrap_or(123);
        format!(
            r#"{{
                "ok": true,
                "result": {{
                    "message_id": {mid},
                    "from": {{
                        "id": 1,
                        "is_bot": true,
                        "first_name": "Bot"
                    }},
                    "chat": {{
                        "id": 100,
                        "type": "private",
                        "first_name": "User"
                    }},
                    "date": 1000,
                    "text": "Test"
                }}
            }}"#
        )
    } else {
        r#"{
            "ok": false,
            "description": "Bad Request: invalid chat_id"
        }"#
        .to_string()
    }
}

/// Creates a mock error response from the Telegram API.
#[allow(dead_code)]
pub fn mock_error_response(error_code: u16, description: &str) -> String {
    format!(
        r#"{{
            "ok": false,
            "error_code": {error_code},
            "description": "{description}"
        }}"#
    )
}

