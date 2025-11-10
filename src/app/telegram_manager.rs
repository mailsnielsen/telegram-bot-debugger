//! Telegram API integration manager.
//!
//! Handles all Telegram Bot API interactions, update processing, and chat discovery.

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use super::state::TestMessageMode;
use crate::telegram::{DiscoveredChat, TelegramClient, Update, UpdateProcessor};

// Input validation constants
const MAX_TOKEN_LENGTH: usize = 256;
const MAX_CHAT_ID_LENGTH: usize = 20;
const MAX_MESSAGE_LENGTH: usize = 4096;

/// Result of token validation.
pub enum TokenValidationResult {
    Valid(TelegramClient),
    Empty,
    TooLong(usize),
    Invalid(String),
}

/// Result of sending a test message.
pub struct SendMessageResult {
    pub success: bool,
    pub message: String,
}

/// Manages Telegram API client and update processing.
pub struct TelegramManager {
    pub client: Option<TelegramClient>,
    pub update_processor: UpdateProcessor,
    pub raw_updates: Vec<Arc<Update>>,
    pub raw_json_updates: Vec<JsonValue>, // Complete raw JSON from API
    pub last_processed_update_id: i64,
}

impl TelegramManager {
    pub fn new() -> Self {
        Self {
            client: None,
            update_processor: UpdateProcessor::new(),
            raw_updates: Vec::new(),
            raw_json_updates: Vec::new(),
            last_processed_update_id: 0,
        }
    }

    pub fn new_with_token(token: String) -> Self {
        Self {
            client: Some(TelegramClient::new(token)),
            update_processor: UpdateProcessor::new(),
            raw_updates: Vec::new(),
            raw_json_updates: Vec::new(),
            last_processed_update_id: 0,
        }
    }

    pub async fn validate_token(&mut self, token_input: &str) -> Result<TokenValidationResult> {
        let token = token_input.trim().to_string();

        if token.is_empty() {
            return Ok(TokenValidationResult::Empty);
        }

        if token.len() > MAX_TOKEN_LENGTH {
            return Ok(TokenValidationResult::TooLong(MAX_TOKEN_LENGTH));
        }

        let client = TelegramClient::new(token);
        match client.get_me().await {
            Ok(response) => {
                if response.ok {
                    self.client = Some(client.clone());
                    Ok(TokenValidationResult::Valid(client))
                } else {
                    Ok(TokenValidationResult::Invalid("Invalid token".to_string()))
                }
            }
            Err(e) => Ok(TokenValidationResult::Invalid(format!("Error: {e}"))),
        }
    }

    pub async fn send_test_message(
        &self,
        message_input: &str,
        chat_id_input: &str,
        mode: TestMessageMode,
        selected_chat: Option<&DiscoveredChat>,
    ) -> Result<SendMessageResult> {
        let Some(client) = &self.client else {
            return Ok(SendMessageResult {
                success: false,
                message: "✗ Error: No client available".to_string(),
            });
        };

        let text = message_input.trim();

        // Validate message text
        if text.is_empty() {
            return Ok(SendMessageResult {
                success: false,
                message: "✗ Error: Message cannot be empty".to_string(),
            });
        }

        if text.len() > MAX_MESSAGE_LENGTH {
            return Ok(SendMessageResult {
                success: false,
                message: format!("✗ Error: Message too long (max {MAX_MESSAGE_LENGTH} characters)"),
            });
        }

        let chat_id = match mode {
            TestMessageMode::SelectedChat => {
                if let Some(chat) = selected_chat {
                    chat.chat.id
                } else {
                    return Ok(SendMessageResult {
                        success: false,
                        message: "✗ Error: No chat selected".to_string(),
                    });
                }
            }
            TestMessageMode::ManualChatId => {
                let chat_id_str = chat_id_input.trim();

                if chat_id_str.is_empty() {
                    return Ok(SendMessageResult {
                        success: false,
                        message: "✗ Error: Chat ID cannot be empty".to_string(),
                    });
                }

                if chat_id_str.len() > MAX_CHAT_ID_LENGTH {
                    return Ok(SendMessageResult {
                        success: false,
                        message: format!(
                            "✗ Error: Chat ID too long (max {MAX_CHAT_ID_LENGTH} chars)"
                        ),
                    });
                }

                match chat_id_str.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        return Ok(SendMessageResult {
                            success: false,
                            message: "✗ Error: Invalid chat ID format (must be a number)"
                                .to_string(),
                        });
                    }
                }
            }
        };

        let result = client.send_message(chat_id, text, None).await?;

        if result.ok {
            Ok(SendMessageResult {
                success: true,
                message: "✓ Message sent successfully!".to_string(),
            })
        } else {
            let error = result
                .description
                .unwrap_or_else(|| "Unknown error".to_string());
            Ok(SendMessageResult {
                success: false,
                message: format!("✗ Error: {error}"),
            })
        }
    }

    pub fn get_discovered_chats(&self) -> Vec<&DiscoveredChat> {
        self.update_processor.get_discovered_chats()
    }

    pub fn get_messages_for_chat(&self, chat_id: i64) -> Vec<&Arc<Update>> {
        self.raw_updates
            .iter()
            .filter(|update| {
                if let Some(message) = &update.message {
                    message.chat.id == chat_id
                } else if let Some(channel_post) = &update.channel_post {
                    channel_post.chat.id == chat_id
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_selected_message_for_chat(
        &self,
        chat_id: Option<i64>,
        message_index: usize,
    ) -> Option<&Arc<Update>> {
        if let Some(id) = chat_id {
            let messages = self.get_messages_for_chat(id);
            messages.get(message_index).copied()
        } else {
            None
        }
    }

    pub fn get_selected_update(&self, index: usize) -> Option<&Arc<Update>> {
        self.raw_updates.get(index)
    }

    pub fn process_updates_batch(
        &mut self,
        updates: Vec<Update>,
        monitor_messages: &mut Vec<super::monitoring::MonitorMessage>,
    ) {
        for update in &updates {
            if update.update_id > self.last_processed_update_id {
                self.last_processed_update_id = update.update_id;

                // Store raw update for debugging (rolling buffer of 50)
                self.raw_updates.push(Arc::new(update.clone()));
                if self.raw_updates.len() > 50 {
                    self.raw_updates.remove(0);
                }

                // Extract monitor message if applicable
                self.extract_monitor_message(update, monitor_messages);
            }
        }

        self.update_processor.process_updates(updates);
    }

    fn extract_monitor_message(
        &self,
        update: &Update,
        monitor_messages: &mut Vec<super::monitoring::MonitorMessage>,
    ) {
        use chrono::Local;

        if let Some(message) = &update.message {
            let chat_name = message.chat.display_name();
            let sender = message
                .from
                .as_ref()
                .map(|u| u.username.clone().unwrap_or_else(|| u.first_name.clone()));
            let text = message
                .text
                .clone()
                .unwrap_or_else(|| "[No text]".to_string());

            monitor_messages.push(super::monitoring::MonitorMessage {
                timestamp: Local::now().timestamp(),
                chat_name,
                sender,
                text,
            });

            // Keep only last 100 messages
            if monitor_messages.len() > 100 {
                monitor_messages.remove(0);
            }
        } else if let Some(channel_post) = &update.channel_post {
            let chat_name = channel_post.chat.display_name();
            let text = channel_post
                .text
                .clone()
                .unwrap_or_else(|| "[No text]".to_string());

            monitor_messages.push(super::monitoring::MonitorMessage {
                timestamp: Local::now().timestamp(),
                chat_name,
                sender: None,
                text,
            });

            // Keep only last 100 messages
            if monitor_messages.len() > 100 {
                monitor_messages.remove(0);
            }
        }
    }
}

impl Default for TelegramManager {
    fn default() -> Self {
        Self::new()
    }
}
