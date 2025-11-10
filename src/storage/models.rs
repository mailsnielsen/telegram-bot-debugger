//! Data models for persistent storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::telegram::{DiscoveredChat, TopicInfo};

/// Root cache data structure.
///
/// Contains all data that needs to be persisted between application runs.
#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
pub struct CacheData {
    /// The bot token
    pub token: Option<String>,
    /// Cached information about discovered chats
    pub chats: Vec<CachedChat>,
    /// Aggregated analytics data
    pub analytics: AnalyticsData,
}


/// Serializable representation of a discovered chat for caching.
///
/// Similar to [`DiscoveredChat`] but optimized for JSON storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedChat {
    pub chat_id: i64,
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub last_seen: i64,
    pub message_count: usize,
    pub topics: Vec<TopicInfo>,
}

impl From<&DiscoveredChat> for CachedChat {
    fn from(dc: &DiscoveredChat) -> Self {
        Self {
            chat_id: dc.chat.id,
            chat_type: dc.chat.chat_type.clone(),
            title: dc.chat.title.clone(),
            username: dc.chat.username.clone(),
            first_name: dc.chat.first_name.clone(),
            last_name: dc.chat.last_name.clone(),
            last_seen: dc.last_seen,
            message_count: dc.message_count,
            topics: dc.topics.clone(),
        }
    }
}

/// Analytics data for persistent storage.
///
/// Tracks message statistics across all chats.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AnalyticsData {
    /// Total number of messages processed
    pub total_messages: usize,
    /// Message count per chat ID
    pub messages_per_chat: HashMap<i64, usize>,
    /// Message distribution by hour of day (0-23)
    pub hourly_distribution: HashMap<i64, usize>,
}


impl AnalyticsData {
    // All methods currently accessed directly through struct fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_discovered_chat, create_test_topic};

    #[test]
    fn test_cached_chat_from_discovered_chat() {
        let discovered = create_test_discovered_chat(100, "private", 10);
        let cached = CachedChat::from(&discovered);

        assert_eq!(cached.chat_id, 100);
        assert_eq!(cached.chat_type, "private");
        assert_eq!(cached.message_count, 10);
        assert_eq!(cached.last_seen, 1000);
    }

    #[test]
    fn test_cached_chat_preserves_all_fields() {
        let mut discovered = create_test_discovered_chat(200, "group", 20);
        discovered.chat.title = Some("Test Group".to_string());
        discovered.chat.username = Some("testgroup".to_string());
        discovered.topics = vec![create_test_topic(1, Some("Topic 1".to_string()), 5)];
        
        let cached = CachedChat::from(&discovered);

        assert_eq!(cached.chat_id, 200);
        assert_eq!(cached.title, Some("Test Group".to_string()));
        assert_eq!(cached.username, Some("testgroup".to_string()));
        assert_eq!(cached.topics.len(), 1);
        assert_eq!(cached.topics[0].thread_id, 1);
    }

    #[test]
    fn test_cache_data_default() {
        let data = CacheData::default();

        assert!(data.token.is_none());
        assert_eq!(data.chats.len(), 0);
        assert_eq!(data.analytics.total_messages, 0);
    }

    #[test]
    fn test_analytics_data_default() {
        let analytics = AnalyticsData::default();

        assert_eq!(analytics.total_messages, 0);
        assert_eq!(analytics.messages_per_chat.len(), 0);
        assert_eq!(analytics.hourly_distribution.len(), 0);
    }
}

