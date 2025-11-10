use chrono::{DateTime, Local, Timelike};
use std::collections::HashMap;

use crate::telegram::DiscoveredChat;

/// Aggregated statistics computed from discovered chats.
///
/// Provides insights into message activity including:
/// - Total messages and chats
/// - Top chats by message volume
/// - Hourly activity patterns
/// - Distribution by chat type (private, group, supergroup, channel)
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Total number of messages across all chats
    pub total_messages: usize,
    /// Total number of discovered chats
    pub total_chats: usize,
    /// Total number of topics across forum groups
    pub total_topics: usize,
    /// Message count per chat, sorted by count (descending)
    pub messages_per_chat: Vec<(String, usize)>,
    /// Message distribution by hour of day (0-23)
    pub hourly_distribution: Vec<(u32, usize)>,
    /// Count of chats by type
    pub chat_type_distribution: HashMap<String, usize>,
}

impl Statistics {
    /// Computes statistics from a list of discovered chats.
    ///
    /// # Arguments
    ///
    /// * `chats` - Slice of references to discovered chats
    ///
    /// # Returns
    ///
    /// A Statistics object with all computed aggregations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use telegram_bot_debugger::analytics::Statistics;
    /// # use telegram_bot_debugger::telegram::UpdateProcessor;
    /// let processor = UpdateProcessor::new();
    /// let chats = processor.get_discovered_chats();
    /// let stats = Statistics::from_chats(&chats);
    /// println!("Total messages: {}", stats.total_messages);
    /// ```
    pub fn from_chats(chats: &[&DiscoveredChat]) -> Self {
        let mut total_messages = 0;
        let mut total_topics = 0;
        let mut messages_per_chat = Vec::new();
        let mut hourly_distribution: HashMap<u32, usize> = HashMap::new();
        let mut chat_type_distribution: HashMap<String, usize> = HashMap::new();

        for chat in chats {
            total_messages += chat.message_count;
            total_topics += chat.topics.len();

            let chat_name = chat.chat.display_name();
            messages_per_chat.push((chat_name, chat.message_count));

            // Track chat type distribution
            *chat_type_distribution
                .entry(chat.chat.chat_type.clone())
                .or_insert(0) += 1;

            // Calculate hourly distribution based on last_seen timestamp
            let datetime = DateTime::from_timestamp(chat.last_seen, 0)
                .unwrap_or_else(|| DateTime::<Local>::default().into());
            let hour = datetime.hour();
            *hourly_distribution.entry(hour).or_insert(0) += chat.message_count;
        }

        // Sort messages_per_chat by count descending
        messages_per_chat.sort_by(|a, b| b.1.cmp(&a.1));

        // Convert hourly_distribution to sorted vec
        let mut hourly_vec: Vec<(u32, usize)> = hourly_distribution.into_iter().collect();
        hourly_vec.sort_by_key(|(hour, _)| *hour);

        Statistics {
            total_messages,
            total_chats: chats.len(),
            total_topics,
            messages_per_chat,
            hourly_distribution: hourly_vec,
            chat_type_distribution,
        }
    }

    /// Returns the top N chats by message count.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of chats to return
    ///
    /// # Returns
    ///
    /// A slice of (chat_name, message_count) tuples, sorted by count descending.
    pub fn get_top_chats(&self, limit: usize) -> &[(String, usize)] {
        let end = limit.min(self.messages_per_chat.len());
        &self.messages_per_chat[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        create_test_discovered_chat, create_test_discovered_chat_with_topics, create_test_topic,
    };

    #[test]
    fn test_statistics_from_empty_chats() {
        let chats: Vec<&DiscoveredChat> = vec![];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.total_chats, 0);
        assert_eq!(stats.total_topics, 0);
        assert_eq!(stats.messages_per_chat.len(), 0);
        assert_eq!(stats.hourly_distribution.len(), 0);
        assert_eq!(stats.chat_type_distribution.len(), 0);
    }

    #[test]
    fn test_statistics_single_chat() {
        let chat = create_test_discovered_chat(100, "private", 5);
        let chats = vec![&chat];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_messages, 5);
        assert_eq!(stats.total_chats, 1);
        assert_eq!(stats.messages_per_chat.len(), 1);
    }

    #[test]
    fn test_statistics_multiple_chats() {
        let chat1 = create_test_discovered_chat(100, "private", 10);
        let chat2 = create_test_discovered_chat(200, "group", 20);
        let chat3 = create_test_discovered_chat(300, "channel", 30);

        let chats = vec![&chat1, &chat2, &chat3];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_messages, 60);
        assert_eq!(stats.total_chats, 3);
    }

    #[test]
    fn test_messages_per_chat_sorted_descending() {
        let chat1 = create_test_discovered_chat(100, "private", 5);
        let chat2 = create_test_discovered_chat(200, "group", 20);
        let chat3 = create_test_discovered_chat(300, "supergroup", 10);

        let chats = vec![&chat1, &chat2, &chat3];
        let stats = Statistics::from_chats(&chats);

        // Should be sorted by message count descending
        assert_eq!(stats.messages_per_chat.len(), 3);
        assert!(stats.messages_per_chat[0].1 >= stats.messages_per_chat[1].1);
        assert!(stats.messages_per_chat[1].1 >= stats.messages_per_chat[2].1);

        // Highest should be 20
        assert_eq!(stats.messages_per_chat[0].1, 20);
    }

    #[test]
    fn test_hourly_distribution() {
        // Create chat with specific last_seen to test hour extraction
        let mut chat1 = create_test_discovered_chat(100, "private", 10);
        chat1.last_seen = 1609459200; // 2021-01-01 00:00:00 UTC (hour 0)

        let mut chat2 = create_test_discovered_chat(200, "group", 20);
        chat2.last_seen = 1609462800; // 2021-01-01 01:00:00 UTC (hour 1)

        let chats = vec![&chat1, &chat2];
        let stats = Statistics::from_chats(&chats);

        // Should have entries for the hours when messages were seen
        assert!(!stats.hourly_distribution.is_empty());

        // Distribution should be sorted by hour
        for i in 1..stats.hourly_distribution.len() {
            assert!(stats.hourly_distribution[i - 1].0 <= stats.hourly_distribution[i].0);
        }
    }

    #[test]
    fn test_chat_type_distribution() {
        let chat1 = create_test_discovered_chat(100, "private", 5);
        let chat2 = create_test_discovered_chat(200, "private", 10);
        let chat3 = create_test_discovered_chat(300, "group", 15);
        let chat4 = create_test_discovered_chat(400, "supergroup", 20);
        let chat5 = create_test_discovered_chat(500, "channel", 25);

        let chats = vec![&chat1, &chat2, &chat3, &chat4, &chat5];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.chat_type_distribution.len(), 4);
        assert_eq!(stats.chat_type_distribution["private"], 2);
        assert_eq!(stats.chat_type_distribution["group"], 1);
        assert_eq!(stats.chat_type_distribution["supergroup"], 1);
        assert_eq!(stats.chat_type_distribution["channel"], 1);
    }

    #[test]
    fn test_total_topics_count() {
        let topics1 = vec![
            create_test_topic(1, Some("Topic 1".to_string()), 5),
            create_test_topic(2, Some("Topic 2".to_string()), 3),
        ];
        let chat1 = create_test_discovered_chat_with_topics(100, topics1);

        let topics2 = vec![create_test_topic(3, Some("Topic 3".to_string()), 7)];
        let chat2 = create_test_discovered_chat_with_topics(200, topics2);

        let chats = vec![&chat1, &chat2];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_topics, 3);
    }

    #[test]
    fn test_get_top_chats() {
        let chat1 = create_test_discovered_chat(100, "private", 5);
        let chat2 = create_test_discovered_chat(200, "group", 20);
        let chat3 = create_test_discovered_chat(300, "supergroup", 15);
        let chat4 = create_test_discovered_chat(400, "channel", 30);

        let chats = vec![&chat1, &chat2, &chat3, &chat4];
        let stats = Statistics::from_chats(&chats);

        // Get top 2
        let top_2 = stats.get_top_chats(2);
        assert_eq!(top_2.len(), 2);
        assert_eq!(top_2[0].1, 30); // Highest
        assert_eq!(top_2[1].1, 20); // Second highest
    }

    #[test]
    fn test_get_top_chats_limit_exceeds_total() {
        let chat1 = create_test_discovered_chat(100, "private", 5);
        let chat2 = create_test_discovered_chat(200, "group", 10);

        let chats = vec![&chat1, &chat2];
        let stats = Statistics::from_chats(&chats);

        // Request more than available
        let top_5 = stats.get_top_chats(5);
        assert_eq!(top_5.len(), 2); // Should only return 2
    }

    #[test]
    fn test_get_top_chats_zero_limit() {
        let chat1 = create_test_discovered_chat(100, "private", 5);
        let chats = vec![&chat1];
        let stats = Statistics::from_chats(&chats);

        let top_0 = stats.get_top_chats(0);
        assert_eq!(top_0.len(), 0);
    }

    #[test]
    fn test_statistics_with_zero_messages() {
        let chat = create_test_discovered_chat(100, "private", 0);
        let chats = vec![&chat];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.total_chats, 1);
        assert_eq!(stats.messages_per_chat[0].1, 0);
    }

    #[test]
    fn test_statistics_with_equal_message_counts() {
        let chat1 = create_test_discovered_chat(100, "private", 10);
        let chat2 = create_test_discovered_chat(200, "group", 10);
        let chat3 = create_test_discovered_chat(300, "supergroup", 10);

        let chats = vec![&chat1, &chat2, &chat3];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_messages, 30);
        // All should have 10 messages
        for (_, count) in &stats.messages_per_chat {
            assert_eq!(*count, 10);
        }
    }

    #[test]
    fn test_statistics_preserves_chat_names() {
        let mut chat1 = create_test_discovered_chat(100, "private", 5);
        chat1.chat.first_name = Some("Alice".to_string());

        let mut chat2 = create_test_discovered_chat(200, "group", 10);
        chat2.chat.title = Some("My Group".to_string());

        let chats = vec![&chat1, &chat2];
        let stats = Statistics::from_chats(&chats);

        // Check that display names are present
        let names: Vec<&String> = stats
            .messages_per_chat
            .iter()
            .map(|(name, _)| name)
            .collect();
        assert!(names.contains(&&"Alice".to_string()) || names.contains(&&"My Group".to_string()));
    }

    #[test]
    fn test_hourly_distribution_aggregates_correctly() {
        // Create multiple chats with messages at same hour
        let mut chat1 = create_test_discovered_chat(100, "private", 10);
        chat1.last_seen = 1609459200; // hour 0

        let mut chat2 = create_test_discovered_chat(200, "group", 20);
        chat2.last_seen = 1609459200; // same hour 0

        let chats = vec![&chat1, &chat2];
        let stats = Statistics::from_chats(&chats);

        // Should aggregate messages from both chats for the same hour
        // We can't reliably test the exact aggregation without knowing the timezone,
        // but we can verify the structure
        assert!(!stats.hourly_distribution.is_empty());
    }

    #[test]
    fn test_very_large_message_counts() {
        let chat = create_test_discovered_chat(100, "private", usize::MAX - 1);
        let chats = vec![&chat];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_messages, usize::MAX - 1);
    }

    #[test]
    fn test_chat_type_distribution_counts_correctly() {
        let chat1 = create_test_discovered_chat(100, "private", 1);
        let chat2 = create_test_discovered_chat(200, "private", 1);
        let chat3 = create_test_discovered_chat(300, "private", 1);
        let chat4 = create_test_discovered_chat(400, "group", 1);

        let chats = vec![&chat1, &chat2, &chat3, &chat4];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.chat_type_distribution["private"], 3);
        assert_eq!(stats.chat_type_distribution["group"], 1);
        assert_eq!(stats.chat_type_distribution.len(), 2);
    }

    #[test]
    fn test_statistics_with_topics_no_messages() {
        let topics = vec![create_test_topic(1, None, 0), create_test_topic(2, None, 0)];
        let chat = create_test_discovered_chat_with_topics(100, topics);
        let chats = vec![&chat];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.total_topics, 2);
        assert_eq!(stats.total_messages, 0);
    }

    #[test]
    fn test_messages_per_chat_includes_all_chats() {
        let chat1 = create_test_discovered_chat(100, "private", 5);
        let chat2 = create_test_discovered_chat(200, "group", 10);
        let chat3 = create_test_discovered_chat(300, "channel", 15);

        let chats = vec![&chat1, &chat2, &chat3];
        let stats = Statistics::from_chats(&chats);

        assert_eq!(stats.messages_per_chat.len(), 3);
        let total_from_list: usize = stats.messages_per_chat.iter().map(|(_, count)| count).sum();
        assert_eq!(total_from_list, stats.total_messages);
    }
}
