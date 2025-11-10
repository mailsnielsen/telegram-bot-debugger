use std::collections::HashMap;

use super::types::{DiscoveredChat, TopicInfo, Update};

/// Processes Telegram updates and maintains discovered chat information.
///
/// This processor analyzes incoming updates to:
/// - Track all chats the bot has interacted with
/// - Count messages per chat
/// - Discover and track topics in forum groups
/// - Maintain the last seen timestamp for each chat
///
/// # Examples
///
/// ```
/// use telegram_bot_debugger::telegram::UpdateProcessor;
///
/// let mut processor = UpdateProcessor::new();
/// // Process updates as they come in
/// // processor.process_updates(updates);
/// let chats = processor.get_discovered_chats();
/// println!("Discovered {} chats", chats.len());
/// ```
pub struct UpdateProcessor {
    discovered_chats: HashMap<i64, DiscoveredChat>,
    last_update_id: i64,
}

impl UpdateProcessor {
    /// Creates a new UpdateProcessor with empty state.
    pub fn new() -> Self {
        Self {
            discovered_chats: HashMap::new(),
            last_update_id: 0,
        }
    }

    /// Processes a batch of updates from Telegram.
    ///
    /// This method:
    /// - Updates the last_update_id for offset tracking
    /// - Discovers new chats or updates existing ones
    /// - Tracks message counts and timestamps
    /// - Identifies topics in forum groups
    ///
    /// # Arguments
    ///
    /// * `updates` - Vector of Update objects from the Telegram API
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::telegram::{UpdateProcessor, TelegramClient};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut processor = UpdateProcessor::new();
    /// let client = TelegramClient::new("TOKEN".to_string());
    /// 
    /// let response = client.get_updates(None, None).await?;
    /// processor.process_updates(response.result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_updates(&mut self, updates: Vec<Update>) {
        for update in updates {
            if update.update_id > self.last_update_id {
                self.last_update_id = update.update_id;
            }

            // Process message
            if let Some(message) = &update.message {
                let chat_id = message.chat.id;
                let entry = self.discovered_chats.entry(chat_id).or_insert_with(|| {
                    DiscoveredChat {
                        chat: message.chat.clone(),
                        last_seen: message.date,
                        message_count: 0,
                        topics: Vec::new(),
                    }
                });

                entry.message_count += 1;
                entry.last_seen = message.date.max(entry.last_seen);

                // Handle topics for forum groups
                if let Some(thread_id) = message.message_thread_id {
                    if let Some(topic) = entry.topics.iter_mut().find(|t| t.thread_id == thread_id) {
                        topic.message_count += 1;
                        topic.last_seen = message.date.max(topic.last_seen);
                    } else {
                        entry.topics.push(TopicInfo {
                            thread_id,
                            name: None, // We can't extract topic name from regular messages
                            message_count: 1,
                            last_seen: message.date,
                        });
                    }
                }
            }

            // Process channel posts
            if let Some(channel_post) = &update.channel_post {
                let chat_id = channel_post.chat.id;
                let entry = self.discovered_chats.entry(chat_id).or_insert_with(|| {
                    DiscoveredChat {
                        chat: channel_post.chat.clone(),
                        last_seen: channel_post.date,
                        message_count: 0,
                        topics: Vec::new(),
                    }
                });

                entry.message_count += 1;
                entry.last_seen = channel_post.date.max(entry.last_seen);
            }

            // Process edited messages
            if let Some(edited_message) = &update.edited_message {
                let chat_id = edited_message.chat.id;
                if let Some(entry) = self.discovered_chats.get_mut(&chat_id) {
                    entry.last_seen = edited_message.date.max(entry.last_seen);
                }
            }
        }
    }

    /// Returns all discovered chats sorted by last activity (most recent first).
    ///
    /// # Returns
    ///
    /// A vector of references to DiscoveredChat objects, sorted by last_seen timestamp
    /// in descending order (newest first).
    pub fn get_discovered_chats(&self) -> Vec<&DiscoveredChat> {
        let mut chats: Vec<&DiscoveredChat> = self.discovered_chats.values().collect();
        chats.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
        chats
    }

}

impl Default for UpdateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegram::{ChannelPost, Chat, Message, User};
    use crate::test_utils::create_test_chat;

    fn create_test_message(chat_id: i64, message_id: i64, date: i64) -> Message {
        Message {
            message_id,
            from: Some(User {
                id: 123,
                is_bot: false,
                first_name: "Test".to_string(),
                last_name: None,
                username: Some("testuser".to_string()),
            }),
            chat: Chat {
                id: chat_id,
                chat_type: "private".to_string(),
                title: None,
                username: Some("testchat".to_string()),
                first_name: Some("Test".to_string()),
                last_name: None,
            },
            date,
            text: Some("Test message".to_string()),
            message_thread_id: None,
            reply_to_message: None,
            other: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_process_updates_discovers_chat() {
        let mut processor = UpdateProcessor::new();
        
        let updates = vec![
            Update {
                update_id: 1,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(create_test_message(100, 2, 1001)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ];

        processor.process_updates(updates);
        
        let chats = processor.get_discovered_chats();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].chat.id, 100);
    }

    #[test]
    fn test_process_updates_discovers_chats() {
        let mut processor = UpdateProcessor::new();
        
        let updates = vec![
            Update {
                update_id: 1,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(create_test_message(200, 2, 1001)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ];

        processor.process_updates(updates);
        
        let chats = processor.get_discovered_chats();
        assert_eq!(chats.len(), 2);
    }

    #[test]
    fn test_process_updates_counts_messages() {
        let mut processor = UpdateProcessor::new();
        
        let updates = vec![
            Update {
                update_id: 1,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(create_test_message(100, 2, 1001)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ];

        processor.process_updates(updates);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.message_count, 2);
    }

    // Edge cases
    #[test]
    fn test_process_empty_updates() {
        let mut processor = UpdateProcessor::new();
        processor.process_updates(vec![]);
        
        // Empty updates should not discover any chats
        assert_eq!(processor.get_discovered_chats().len(), 0);
    }

    #[test]
    fn test_duplicate_update_ids() {
        let mut processor = UpdateProcessor::new();
        
        let updates = vec![
            Update {
                update_id: 1,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 1, // Duplicate ID
                message: Some(create_test_message(200, 2, 1001)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ];

        processor.process_updates(updates);
        
        // Should still process both messages and discover both chats
        assert_eq!(processor.get_discovered_chats().len(), 2);
    }

    #[test]
    fn test_out_of_order_update_ids() {
        let mut processor = UpdateProcessor::new();
        
        let updates = vec![
            Update {
                update_id: 5,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 3, // Lower ID
                message: Some(create_test_message(100, 2, 1001)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ];

        processor.process_updates(updates);
        
        // Should process both messages for the same chat
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.message_count, 2);
    }

    // Topic handling tests
    #[test]
    fn test_process_messages_with_topics() {
        let mut processor = UpdateProcessor::new();
        
        let mut message = create_test_message(100, 1, 1000);
        message.message_thread_id = Some(42);
        message.chat.chat_type = "supergroup".to_string();
        
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(message),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.topics.len(), 1);
        assert_eq!(chat.topics[0].thread_id, 42);
        assert_eq!(chat.topics[0].message_count, 1);
    }

    #[test]
    fn test_existing_topic_message_count_increments() {
        let mut processor = UpdateProcessor::new();
        
        let mut message1 = create_test_message(100, 1, 1000);
        message1.message_thread_id = Some(42);
        
        let mut message2 = create_test_message(100, 2, 1001);
        message2.message_thread_id = Some(42);
        
        processor.process_updates(vec![
            Update {
                update_id: 1,
                message: Some(message1),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(message2),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.topics.len(), 1);
        assert_eq!(chat.topics[0].message_count, 2);
    }

    #[test]
    fn test_multiple_topics_in_same_chat() {
        let mut processor = UpdateProcessor::new();
        
        let mut message1 = create_test_message(100, 1, 1000);
        message1.message_thread_id = Some(1);
        
        let mut message2 = create_test_message(100, 2, 1001);
        message2.message_thread_id = Some(2);
        
        let mut message3 = create_test_message(100, 3, 1002);
        message3.message_thread_id = Some(3);
        
        processor.process_updates(vec![
            Update {
                update_id: 1,
                message: Some(message1),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(message2),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 3,
                message: Some(message3),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.topics.len(), 3);
    }

    #[test]
    fn test_topic_last_seen_updates() {
        let mut processor = UpdateProcessor::new();
        
        let mut message1 = create_test_message(100, 1, 1000);
        message1.message_thread_id = Some(42);
        
        let mut message2 = create_test_message(100, 2, 2000);
        message2.message_thread_id = Some(42);
        
        processor.process_updates(vec![
            Update {
                update_id: 1,
                message: Some(message1),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(message2),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.topics[0].last_seen, 2000);
    }

    // Channel post tests
    #[test]
    fn test_process_channel_posts() {
        let mut processor = UpdateProcessor::new();
        
        let channel_post = ChannelPost {
            message_id: 1,
            chat: create_test_chat(-1001234567890, "channel"),
            date: 1000,
            text: Some("Channel post".to_string()),
            other: std::collections::HashMap::new(),
        };
        
        processor.process_updates(vec![Update {
            update_id: 1,
            message: None,
            channel_post: Some(channel_post),
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].chat.chat_type, "channel");
        assert_eq!(chats[0].message_count, 1);
    }

    #[test]
    fn test_channel_posts_count_correctly() {
        let mut processor = UpdateProcessor::new();
        
        let chat_id = -1001234567890;
        let channel_post1 = ChannelPost {
            message_id: 1,
            chat: create_test_chat(chat_id, "channel"),
            date: 1000,
            text: Some("Post 1".to_string()),
            other: std::collections::HashMap::new(),
        };
        
        let channel_post2 = ChannelPost {
            message_id: 2,
            chat: create_test_chat(chat_id, "channel"),
            date: 1001,
            text: Some("Post 2".to_string()),
            other: std::collections::HashMap::new(),
        };
        
        processor.process_updates(vec![
            Update {
                update_id: 1,
                message: None,
                channel_post: Some(channel_post1),
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: None,
                channel_post: Some(channel_post2),
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == chat_id).unwrap();
        assert_eq!(chat.message_count, 2);
    }

    // Edited message tests
    #[test]
    fn test_process_edited_messages() {
        let mut processor = UpdateProcessor::new();
        
        // First, send an original message to create the chat
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(create_test_message(100, 1, 1000)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        // Now send an edited message
        let edited_message = create_test_message(100, 1, 1500);
        processor.process_updates(vec![Update {
            update_id: 2,
            message: None,
            channel_post: None,
            edited_message: Some(edited_message),
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        // Edited messages don't create new chats or increment counts
        assert_eq!(chat.message_count, 1); // Only the original message
        assert_eq!(chat.last_seen, 1500); // But last_seen is updated
    }

    #[test]
    fn test_edited_message_updates_last_seen() {
        let mut processor = UpdateProcessor::new();
        
        // Original message
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(create_test_message(100, 1, 1000)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        // Edited version with newer timestamp
        let edited = create_test_message(100, 1, 2000);
        processor.process_updates(vec![Update {
            update_id: 2,
            message: None,
            channel_post: None,
            edited_message: Some(edited),
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.last_seen, 2000);
        assert_eq!(chat.message_count, 1); // Only the original message counted
    }

    #[test]
    fn test_edited_message_without_existing_chat() {
        let mut processor = UpdateProcessor::new();
        
        // Send only an edited message without an original
        let edited_message = create_test_message(100, 1, 1000);
        processor.process_updates(vec![Update {
            update_id: 1,
            message: None,
            channel_post: None,
            edited_message: Some(edited_message),
            other: std::collections::HashMap::new(),
        }]);
        
        // Chat should not be created from edited message alone
        let chats = processor.get_discovered_chats();
        assert!(!chats.iter().any(|c| c.chat.id == 100));
    }

    // Last seen timestamp tests
    #[test]
    fn test_last_seen_updates_to_most_recent() {
        let mut processor = UpdateProcessor::new();
        
        processor.process_updates(vec![
            Update {
                update_id: 1,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(create_test_message(100, 2, 500)), // Older timestamp
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.last_seen, 1000); // Should keep the newer one
    }

    // Multiple chats in single batch
    #[test]
    fn test_multiple_chats_single_batch() {
        let mut processor = UpdateProcessor::new();
        
        let updates = vec![
            Update {
                update_id: 1,
                message: Some(create_test_message(100, 1, 1000)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 2,
                message: Some(create_test_message(200, 2, 1001)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
            Update {
                update_id: 3,
                message: Some(create_test_message(300, 3, 1002)),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            },
        ];

        processor.process_updates(updates);
        
        let chats = processor.get_discovered_chats();
        assert_eq!(chats.len(), 3);
    }

    // Chat info updates when details change
    #[test]
    fn test_chat_info_does_not_change() {
        let mut processor = UpdateProcessor::new();
        
        // First message
        let mut message1 = create_test_message(100, 1, 1000);
        message1.chat.first_name = Some("Alice".to_string());
        
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(message1),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        // Second message with different chat details (shouldn't override)
        let mut message2 = create_test_message(100, 2, 1001);
        message2.chat.first_name = Some("Bob".to_string());
        
        processor.process_updates(vec![Update {
            update_id: 2,
            message: Some(message2),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        // First chat info is preserved
        assert_eq!(chat.chat.first_name, Some("Alice".to_string()));
        assert_eq!(chat.message_count, 2);
    }

    #[test]
    fn test_negative_chat_ids() {
        let mut processor = UpdateProcessor::new();
        
        let mut message = create_test_message(-1001234567890, 1, 1000);
        message.chat.chat_type = "supergroup".to_string();
        
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(message),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == -1001234567890).unwrap();
        assert_eq!(chat.chat.id, -1001234567890);
    }

    #[test]
    fn test_message_with_no_text() {
        let mut processor = UpdateProcessor::new();
        
        let mut message = create_test_message(100, 1, 1000);
        message.text = None;
        
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(message),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        let chats = processor.get_discovered_chats();
        let chat = chats.iter().find(|c| c.chat.id == 100).unwrap();
        assert_eq!(chat.message_count, 1);
    }

    #[test]
    fn test_very_large_update_id() {
        let mut processor = UpdateProcessor::new();
        
        let large_id = i64::MAX - 1;
        processor.process_updates(vec![Update {
            update_id: large_id,
            message: Some(create_test_message(100, 1, 1000)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        // Verify chat was discovered even with large update ID
        let chats = processor.get_discovered_chats();
        assert_eq!(chats.len(), 1);
    }

    #[test]
    fn test_processing_updates_incrementally() {
        let mut processor = UpdateProcessor::new();
        
        // First batch
        processor.process_updates(vec![Update {
            update_id: 1,
            message: Some(create_test_message(100, 1, 1000)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        assert_eq!(processor.get_discovered_chats().len(), 1);
        
        // Second batch
        processor.process_updates(vec![Update {
            update_id: 2,
            message: Some(create_test_message(200, 2, 1001)),
            channel_post: None,
            edited_message: None,
            other: std::collections::HashMap::new(),
        }]);
        
        // Should discover both chats
        assert_eq!(processor.get_discovered_chats().len(), 2);
    }
}

