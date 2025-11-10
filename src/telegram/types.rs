//! Telegram Bot API type definitions.
//!
//! This module contains all the data structures used by the Telegram Bot API,
//! including users, chats, messages, and updates.

use serde::{Deserialize, Serialize};

/// Represents a Telegram user or bot.
///
/// # Fields
///
/// * `id` - Unique identifier for this user or bot
/// * `is_bot` - True if this user is a bot
/// * `first_name` - User's or bot's first name
/// * `last_name` - Optional last name
/// * `username` - Optional username (without @)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

/// Represents a Telegram chat (private, group, supergroup, or channel).
///
/// # Fields
///
/// * `id` - Unique identifier for this chat
/// * `chat_type` - Type of chat: "private", "group", "supergroup", or "channel"
/// * `title` - Optional title for groups, supergroups, and channels
/// * `username` - Optional username for private chats, supergroups, and channels
/// * `first_name` - Optional first name for private chats
/// * `last_name` - Optional last name for private chats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl Chat {
    /// Returns a human-readable display name for the chat.
    ///
    /// Priority order:
    /// 1. Title (for groups/channels)
    /// 2. Username (prefixed with @)
    /// 3. First and last name (for private chats)
    /// 4. Fallback to "Chat {id}"
    ///
    /// # Examples
    ///
    /// ```
    /// # use telegram_bot_debugger::telegram::Chat;
    /// let chat = Chat {
    ///     id: 123,
    ///     chat_type: "group".to_string(),
    ///     title: Some("My Group".to_string()),
    ///     username: None,
    ///     first_name: None,
    ///     last_name: None,
    /// };
    /// assert_eq!(chat.display_name(), "My Group");
    /// ```
    pub fn display_name(&self) -> String {
        if let Some(title) = &self.title {
            title.clone()
        } else if let Some(username) = &self.username {
            format!("@{username}")
        } else if let Some(first_name) = &self.first_name {
            if let Some(last_name) = &self.last_name {
                format!("{first_name} {last_name}")
            } else {
                first_name.clone()
            }
        } else {
            let id = self.id;
            format!("Chat {id}")
        }
    }
}

/// Represents a message in a chat.
///
/// # Fields
///
/// * `message_id` - Unique message identifier inside this chat
/// * `from` - Optional sender (absent for channel posts)
/// * `chat` - Conversation the message belongs to
/// * `date` - Date the message was sent (Unix timestamp)
/// * `text` - Optional text content
/// * `message_thread_id` - Optional topic/thread ID for forum groups
/// * `reply_to_message` - Optional message being replied to
///
/// Note: Additional fields from Telegram API are preserved using flatten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub from: Option<User>,
    pub chat: Chat,
    pub date: i64,
    pub text: Option<String>,
    pub message_thread_id: Option<i64>,
    pub reply_to_message: Option<Box<Message>>,
    
    // Capture all other fields from the API that we don't explicitly handle
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}

/// Represents a channel post.
///
/// Similar to [`Message`] but specifically for channel posts without a sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPost {
    pub message_id: i64,
    pub chat: Chat,
    pub date: i64,
    pub text: Option<String>,
    
    // Capture all other fields from the API
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}

/// Represents an incoming update from Telegram.
///
/// Only one of the optional fields will be present in each update.
///
/// # Fields
///
/// * `update_id` - Unique update identifier (monotonically increasing)
/// * `message` - New incoming message of any kind
/// * `channel_post` - New incoming channel post of any kind
/// * `edited_message` - New version of a message that was edited
///
/// Note: Additional update types (callback_query, inline_query, etc.) are preserved using flatten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
    pub channel_post: Option<ChannelPost>,
    pub edited_message: Option<Message>,
    
    // Capture all other fields from the API (e.g., callback_query, inline_query, etc.)
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}

/// Response from the `getUpdates` API method.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUpdatesResponse {
    pub ok: bool,
    pub result: Vec<Update>,
}

/// Response from the `sendMessage` API method.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub ok: bool,
    pub result: Option<Message>,
    pub description: Option<String>,
}

impl Update {
    /// Returns a human-readable string describing the type of this update.
    ///
    /// Detects all known Telegram Bot API update types, including:
    /// - Standard message types (message, edited_message, channel_post, edited_channel_post)
    /// - Business-related updates (business_connection, business_message, etc.)
    /// - Interactive elements (callback_query, inline_query, poll, etc.)
    /// - User/chat membership updates (my_chat_member, chat_member, chat_join_request)
    /// - Other types (chat_boost, message_reaction, etc.)
    ///
    /// For unknown or future update types, returns "other_{key}" where {key} is the
    /// field name from the API response.
    ///
    /// # Examples
    ///
    /// ```
    /// # use telegram_bot_debugger::telegram::{Update, Message, Chat};
    /// # use std::collections::HashMap;
    /// let update = Update {
    ///     update_id: 123,
    ///     message: Some(Message {
    ///         message_id: 1,
    ///         from: None,
    ///         chat: Chat {
    ///             id: 100,
    ///             chat_type: "private".to_string(),
    ///             title: None,
    ///             username: None,
    ///             first_name: Some("Test".to_string()),
    ///             last_name: None,
    ///         },
    ///         date: 1000,
    ///         text: Some("Hello".to_string()),
    ///         message_thread_id: None,
    ///         reply_to_message: None,
    ///         other: HashMap::new(),
    ///     }),
    ///     channel_post: None,
    ///     edited_message: None,
    ///     other: HashMap::new(),
    /// };
    ///
    /// assert_eq!(update.get_update_type(), "message");
    /// ```
    pub fn get_update_type(&self) -> String {
        // Check explicit fields first
        if self.message.is_some() {
            return "message".to_string();
        }
        if self.edited_message.is_some() {
            return "edited_message".to_string();
        }
        if self.channel_post.is_some() {
            return "channel_post".to_string();
        }
        
        // Check for other known update types in the `other` HashMap
        // These are flattened fields from the API that we don't explicitly handle
        let known_types = [
            "edited_channel_post",
            "business_connection",
            "business_message",
            "edited_business_message",
            "deleted_business_messages",
            "message_reaction",
            "message_reaction_count",
            "inline_query",
            "chosen_inline_result",
            "callback_query",
            "shipping_query",
            "pre_checkout_query",
            "purchased_paid_media",
            "poll",
            "poll_answer",
            "my_chat_member",
            "chat_member",
            "chat_join_request",
            "chat_boost",
            "removed_chat_boost",
        ];
        
        for type_name in &known_types {
            if self.other.contains_key(*type_name) {
                return type_name.to_string();
            }
        }
        
        // If we have any other field, return it as "other_{field_name}"
        if let Some((key, _)) = self.other.iter().next() {
            return format!("other_{}", key);
        }
        
        // Fallback for completely empty updates
        "unknown".to_string()
    }
}

/// Response from the `getMe` API method.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetMeResponse {
    pub ok: bool,
    pub result: Option<User>,
}

/// Represents a discovered chat with aggregated statistics.
///
/// This is used by the application to track chats that the bot has interacted with,
/// along with message counts and topic information for forum groups.
///
/// # Fields
///
/// * `chat` - The chat information
/// * `last_seen` - Unix timestamp of the last message received from this chat
/// * `message_count` - Total number of messages seen in this chat
/// * `topics` - List of topics/threads for forum groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredChat {
    pub chat: Chat,
    pub last_seen: i64,
    pub message_count: usize,
    pub topics: Vec<TopicInfo>,
}

/// Information about a topic (thread) in a forum group.
///
/// # Fields
///
/// * `thread_id` - Unique identifier for this topic
/// * `name` - Optional topic name (may not be available from regular messages)
/// * `message_count` - Number of messages seen in this topic
/// * `last_seen` - Unix timestamp of the last message in this topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub thread_id: i64,
    pub name: Option<String>,
    pub message_count: usize,
    pub last_seen: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chat::display_name() tests
    #[test]
    fn test_chat_display_name_with_title() {
        let chat = Chat {
            id: 1,
            chat_type: "group".to_string(),
            title: Some("My Group".to_string()),
            username: Some("mygroup".to_string()),
            first_name: None,
            last_name: None,
        };
        assert_eq!(chat.display_name(), "My Group");
    }

    #[test]
    fn test_chat_display_name_with_username() {
        let chat = Chat {
            id: 2,
            chat_type: "private".to_string(),
            title: None,
            username: Some("john_doe".to_string()),
            first_name: None,
            last_name: None,
        };
        assert_eq!(chat.display_name(), "@john_doe");
    }

    #[test]
    fn test_chat_display_name_with_first_and_last_name() {
        let chat = Chat {
            id: 3,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
        };
        assert_eq!(chat.display_name(), "John Doe");
    }

    #[test]
    fn test_chat_display_name_with_first_name_only() {
        let chat = Chat {
            id: 4,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: Some("John".to_string()),
            last_name: None,
        };
        assert_eq!(chat.display_name(), "John");
    }

    #[test]
    fn test_chat_display_name_fallback_to_id() {
        let chat = Chat {
            id: 5,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: None,
            last_name: None,
        };
        assert_eq!(chat.display_name(), "Chat 5");
    }

    #[test]
    fn test_chat_display_name_priority_title_over_username() {
        let chat = Chat {
            id: 6,
            chat_type: "supergroup".to_string(),
            title: Some("Priority Group".to_string()),
            username: Some("prioritygroup".to_string()),
            first_name: None,
            last_name: None,
        };
        // Title should take priority over username
        assert_eq!(chat.display_name(), "Priority Group");
    }

    #[test]
    fn test_chat_display_name_priority_username_over_name() {
        let chat = Chat {
            id: 7,
            chat_type: "private".to_string(),
            title: None,
            username: Some("testuser".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
        };
        // Username should take priority over names
        assert_eq!(chat.display_name(), "@testuser");
    }

    // Serialization/Deserialization tests
    #[test]
    fn test_user_serialization_roundtrip() {
        let user = User {
            id: 123,
            is_bot: false,
            first_name: "Test".to_string(),
            last_name: Some("User".to_string()),
            username: Some("testuser".to_string()),
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.is_bot, deserialized.is_bot);
        assert_eq!(user.first_name, deserialized.first_name);
        assert_eq!(user.last_name, deserialized.last_name);
        assert_eq!(user.username, deserialized.username);
    }

    #[test]
    fn test_chat_serialization_roundtrip() {
        let chat = Chat {
            id: -100123456789,
            chat_type: "supergroup".to_string(),
            title: Some("Test Group".to_string()),
            username: Some("testgroup".to_string()),
            first_name: None,
            last_name: None,
        };

        let json = serde_json::to_string(&chat).unwrap();
        let deserialized: Chat = serde_json::from_str(&json).unwrap();

        assert_eq!(chat.id, deserialized.id);
        assert_eq!(chat.chat_type, deserialized.chat_type);
        assert_eq!(chat.title, deserialized.title);
        assert_eq!(chat.username, deserialized.username);
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let message = Message {
            message_id: 42,
            from: Some(User {
                id: 1,
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
                first_name: Some("Bob".to_string()),
                last_name: None,
            },
            date: 1234567890,
            text: Some("Hello, World!".to_string()),
            message_thread_id: None,
            reply_to_message: None,
            other: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(message.message_id, deserialized.message_id);
        assert_eq!(message.date, deserialized.date);
        assert_eq!(message.text, deserialized.text);
        assert!(deserialized.from.is_some());
    }

    #[test]
    fn test_update_serialization_roundtrip() {
        let update = Update {
            update_id: 999,
            message: Some(Message {
                message_id: 1,
                from: None,
                chat: Chat {
                    id: 1,
                    chat_type: "private".to_string(),
                    title: None,
                    username: None,
                    first_name: Some("Test".to_string()),
                    last_name: None,
                },
                date: 1000,
                text: Some("Test".to_string()),
                message_thread_id: None,
                reply_to_message: None,
                other: std::collections::HashMap::new(),
            }),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&update).unwrap();
        let deserialized: Update = serde_json::from_str(&json).unwrap();

        assert_eq!(update.update_id, deserialized.update_id);
        assert!(deserialized.message.is_some());
        assert!(deserialized.channel_post.is_none());
        assert!(deserialized.edited_message.is_none());
    }

    #[test]
    fn test_discovered_chat_serialization_roundtrip() {
        let discovered_chat = DiscoveredChat {
            chat: Chat {
                id: 123,
                chat_type: "group".to_string(),
                title: Some("Test".to_string()),
                username: None,
                first_name: None,
                last_name: None,
            },
            last_seen: 1234567890,
            message_count: 42,
            topics: vec![
                TopicInfo {
                    thread_id: 1,
                    name: Some("General".to_string()),
                    message_count: 10,
                    last_seen: 1000,
                },
            ],
        };

        let json = serde_json::to_string(&discovered_chat).unwrap();
        let deserialized: DiscoveredChat = serde_json::from_str(&json).unwrap();

        assert_eq!(discovered_chat.chat.id, deserialized.chat.id);
        assert_eq!(discovered_chat.last_seen, deserialized.last_seen);
        assert_eq!(discovered_chat.message_count, deserialized.message_count);
        assert_eq!(discovered_chat.topics.len(), deserialized.topics.len());
    }

    // Edge cases
    #[test]
    fn test_chat_with_empty_strings() {
        let chat = Chat {
            id: 1,
            chat_type: "".to_string(),
            title: Some("".to_string()),
            username: Some("".to_string()),
            first_name: Some("".to_string()),
            last_name: Some("".to_string()),
        };

        // Empty title should still be displayed
        assert_eq!(chat.display_name(), "");
    }

    #[test]
    fn test_chat_with_special_characters() {
        let chat = Chat {
            id: 1,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: Some("Test 🚀".to_string()),
            last_name: Some("User 😀".to_string()),
        };

        assert_eq!(chat.display_name(), "Test 🚀 User 😀");
    }

    #[test]
    fn test_chat_with_very_long_name() {
        let long_name = "A".repeat(1000);
        let chat = Chat {
            id: 1,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: Some(long_name.clone()),
            last_name: None,
        };

        assert_eq!(chat.display_name(), long_name);
    }

    #[test]
    fn test_message_with_no_text() {
        let message = Message {
            message_id: 1,
            from: None,
            chat: Chat {
                id: 1,
                chat_type: "private".to_string(),
                title: None,
                username: None,
                first_name: None,
                last_name: None,
            },
            date: 1000,
            text: None,
            message_thread_id: None,
            reply_to_message: None,
            other: std::collections::HashMap::new(),
        };

        assert!(message.text.is_none());
    }

    #[test]
    fn test_message_with_thread_id() {
        let message = Message {
            message_id: 1,
            from: None,
            chat: Chat {
                id: 1,
                chat_type: "supergroup".to_string(),
                title: Some("Forum".to_string()),
                username: None,
                first_name: None,
                last_name: None,
            },
            date: 1000,
            text: Some("Forum message".to_string()),
            message_thread_id: Some(42),
            reply_to_message: None,
            other: std::collections::HashMap::new(),
        };

        assert_eq!(message.message_thread_id, Some(42));
    }

    #[test]
    fn test_update_with_channel_post() {
        let update = Update {
            update_id: 1,
            message: None,
            channel_post: Some(ChannelPost {
                message_id: 100,
                chat: Chat {
                    id: -1001234567890,
                    chat_type: "channel".to_string(),
                    title: Some("My Channel".to_string()),
                    username: Some("mychannel".to_string()),
                    first_name: None,
                    last_name: None,
                },
                date: 1000,
                text: Some("Channel post".to_string()),
                other: std::collections::HashMap::new(),
            }),
            edited_message: None,
            other: std::collections::HashMap::new(),
        };

        assert!(update.message.is_none());
        assert!(update.channel_post.is_some());
        assert!(update.edited_message.is_none());
    }

    #[test]
    fn test_update_with_edited_message() {
        let update = Update {
            update_id: 1,
            message: None,
            channel_post: None,
            edited_message: Some(Message {
                message_id: 1,
                from: None,
                chat: Chat {
                    id: 1,
                    chat_type: "private".to_string(),
                    title: None,
                    username: None,
                    first_name: None,
                    last_name: None,
                },
                date: 1000,
                text: Some("Edited".to_string()),
                message_thread_id: None,
                reply_to_message: None,
                other: std::collections::HashMap::new(),
            }),
            other: std::collections::HashMap::new(),
        };

        assert!(update.message.is_none());
        assert!(update.channel_post.is_none());
        assert!(update.edited_message.is_some());
    }

    #[test]
    fn test_negative_chat_ids() {
        // Groups and channels have negative IDs
        let chat = Chat {
            id: -1001234567890,
            chat_type: "supergroup".to_string(),
            title: Some("Supergroup".to_string()),
            username: None,
            first_name: None,
            last_name: None,
        };

        assert!(chat.id < 0);
        assert_eq!(chat.display_name(), "Supergroup");
    }

    #[test]
    fn test_topic_info_with_no_name() {
        let topic = TopicInfo {
            thread_id: 123,
            name: None,
            message_count: 5,
            last_seen: 1000,
        };

        assert!(topic.name.is_none());
        assert_eq!(topic.message_count, 5);
    }

    #[test]
    fn test_topic_info_with_name() {
        let topic = TopicInfo {
            thread_id: 456,
            name: Some("General Discussion".to_string()),
            message_count: 100,
            last_seen: 2000,
        };

        assert_eq!(topic.name, Some("General Discussion".to_string()));
        assert_eq!(topic.message_count, 100);
    }

    #[test]
    fn test_discovered_chat_empty_topics() {
        let chat = DiscoveredChat {
            chat: Chat {
                id: 1,
                chat_type: "private".to_string(),
                title: None,
                username: None,
                first_name: Some("User".to_string()),
                last_name: None,
            },
            last_seen: 1000,
            message_count: 10,
            topics: Vec::new(),
        };

        assert_eq!(chat.topics.len(), 0);
        assert_eq!(chat.message_count, 10);
    }

    #[test]
    fn test_get_me_response_deserialization() {
        let json = r#"{
            "ok": true,
            "result": {
                "id": 123456,
                "is_bot": true,
                "first_name": "Test Bot",
                "username": "testbot"
            }
        }"#;

        let response: GetMeResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());
        
        let user = response.result.unwrap();
        assert_eq!(user.id, 123456);
        assert!(user.is_bot);
        assert_eq!(user.username, Some("testbot".to_string()));
    }

    #[test]
    fn test_get_updates_response_deserialization() {
        let json = r#"{
            "ok": true,
            "result": []
        }"#;

        let response: GetUpdatesResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok);
        assert_eq!(response.result.len(), 0);
    }

    #[test]
    fn test_send_message_response_success() {
        let json = r#"{
            "ok": true,
            "result": {
                "message_id": 42,
                "from": {
                    "id": 1,
                    "is_bot": true,
                    "first_name": "Bot"
                },
                "chat": {
                    "id": 100,
                    "type": "private",
                    "first_name": "User"
                },
                "date": 1000,
                "text": "Test message"
            }
        }"#;

        let response: SendMessageResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok);
        assert!(response.result.is_some());
        assert!(response.description.is_none());
    }

    #[test]
    fn test_send_message_response_error() {
        let json = r#"{
            "ok": false,
            "description": "Bad Request: chat not found"
        }"#;

        let response: SendMessageResponse = serde_json::from_str(json).unwrap();
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert!(response.description.is_some());
    }
}
