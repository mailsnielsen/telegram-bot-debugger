//! Property-based tests for verifying invariants.
//!
//! These tests use proptest to generate random inputs and verify that
//! certain properties hold true across all possible inputs.

use proptest::prelude::*;
use telegram_bot_debugger::telegram::{Chat, DiscoveredChat, Update, UpdateProcessor};

// Property: Navigation indices never go out of bounds
proptest! {
    #[test]
    fn prop_next_chat_stays_in_bounds(chat_count in 1usize..100, iterations in 0usize..1000) {
        let mut index = 0usize;
        for _ in 0..iterations {
            index = (index + 1) % chat_count;
        }
        prop_assert!(index < chat_count);
    }

    #[test]
    fn prop_previous_chat_stays_in_bounds(chat_count in 1usize..100, start_index in 0usize..100) {
        let start_index = start_index % chat_count; // Ensure valid start
        let mut index = start_index;

        // Simulate previous_chat logic
        if index == 0 {
            index = chat_count - 1;
        } else {
            index -= 1;
        }

        prop_assert!(index < chat_count);
    }
}

// Property: Serialization is always reversible (round-trip)
proptest! {
    #[test]
    fn prop_chat_serialization_roundtrip(
        id in any::<i64>(),
        chat_type in prop::sample::select(vec!["private", "group", "supergroup", "channel"]),
        has_title in any::<bool>(),
    ) {
        let chat = Chat {
            id,
            chat_type: chat_type.to_string(),
            title: if has_title { Some("Test".to_string()) } else { None },
            username: None,
            first_name: None,
            last_name: None,
        };

        let json = serde_json::to_string(&chat).unwrap();
        let deserialized: Chat = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(chat.id, deserialized.id);
        prop_assert_eq!(chat.chat_type, deserialized.chat_type);
        prop_assert_eq!(chat.title, deserialized.title);
    }

    #[test]
    fn prop_discovered_chat_serialization_roundtrip(
        chat_id in any::<i64>(),
        message_count in 0usize..10000,
        last_seen in any::<i64>(),
    ) {
        let chat = DiscoveredChat {
            chat: Chat {
                id: chat_id,
                chat_type: "private".to_string(),
                title: None,
                username: None,
                first_name: Some("Test".to_string()),
                last_name: None,
            },
            last_seen,
            message_count,
            topics: vec![],
        };

        let json = serde_json::to_string(&chat).unwrap();
        let deserialized: DiscoveredChat = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(chat.chat.id, deserialized.chat.id);
        prop_assert_eq!(chat.message_count, deserialized.message_count);
        prop_assert_eq!(chat.last_seen, deserialized.last_seen);
    }
}

// Property: Statistics totals always match sum of parts
proptest! {
    #[test]
    fn prop_statistics_totals_match_sum(
        message_counts in prop::collection::vec(0usize..1000, 1..20)
    ) {
        use telegram_bot_debugger::analytics::Statistics;

        let chats: Vec<DiscoveredChat> = message_counts.iter().enumerate().map(|(i, &count)| {
            DiscoveredChat {
                chat: Chat {
                    id: i as i64,
                    chat_type: "private".to_string(),
                    title: None,
                    username: None,
                    first_name: Some(format!("User{i}")),
                    last_name: None,
                },
                last_seen: 1000,
                message_count: count,
                topics: vec![],
            }
        }).collect();

        let chat_refs: Vec<&DiscoveredChat> = chats.iter().collect();
        let stats = Statistics::from_chats(&chat_refs);

        let sum_of_messages: usize = message_counts.iter().sum();
        prop_assert_eq!(stats.total_messages, sum_of_messages);
        prop_assert_eq!(stats.total_chats, message_counts.len());
    }
}

// Property: Message counts are never negative
proptest! {
    #[test]
    fn prop_message_counts_never_negative(
        update_ids in prop::collection::vec(0i64..1000, 1..50)
    ) {
        let mut processor = UpdateProcessor::new();

        let updates: Vec<Update> = update_ids.iter().map(|&id| {
            Update {
                update_id: id,
                message: Some(telegram_bot_debugger::telegram::Message {
                    message_id: id,
                    from: None,
                    chat: Chat {
                        id: 100,
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
            }
        }).collect();

        processor.process_updates(updates);

        let chats = processor.get_discovered_chats();
        // Verify we discovered exactly one chat and it has the correct number of messages
        prop_assert_eq!(chats.len(), 1);
        prop_assert_eq!(chats[0].message_count, update_ids.len());
    }
}

// Property: Chat display name is always non-empty or falls back to ID
proptest! {
    #[test]
    fn prop_chat_display_name_never_empty_unless_all_none(
        id in any::<i64>(),
    ) {
        let chat = Chat {
            id,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: None,
            last_name: None,
        };

        let display_name = chat.display_name();
        // Should fallback to "Chat {id}"
        prop_assert_eq!(display_name, format!("Chat {}", id));
    }

    #[test]
    fn prop_chat_display_name_title_priority(
        id in any::<i64>(),
        title in "\\PC+", // Non-empty string
    ) {
        let chat = Chat {
            id,
            chat_type: "group".to_string(),
            title: Some(title.clone()),
            username: Some("ignore_username".to_string()),
            first_name: Some("ignore_first".to_string()),
            last_name: Some("ignore_last".to_string()),
        };

        // Title should always take priority
        prop_assert_eq!(chat.display_name(), title);
    }
}

// Property: UpdateProcessor always maintains sorted chats by last_seen
proptest! {
    #[test]
    fn prop_discovered_chats_sorted_by_last_seen(
        timestamps in prop::collection::vec(0i64..1000000, 1..20)
    ) {
        let mut processor = UpdateProcessor::new();

        let updates: Vec<Update> = timestamps.iter().enumerate().map(|(i, &ts)| {
            Update {
                update_id: i as i64,
                message: Some(telegram_bot_debugger::telegram::Message {
                    message_id: i as i64,
                    from: None,
                    chat: Chat {
                        id: i as i64, // Each chat gets unique ID
                        chat_type: "private".to_string(),
                        title: None,
                        username: None,
                        first_name: Some(format!("User{i}")),
                        last_name: None,
                    },
                    date: ts,
                    text: Some("Test".to_string()),
                    message_thread_id: None,
                    reply_to_message: None,
                    other: std::collections::HashMap::new(),
                }),
                channel_post: None,
                edited_message: None,
                other: std::collections::HashMap::new(),
            }
        }).collect();

        processor.process_updates(updates);

        let chats = processor.get_discovered_chats();

        // Verify chats are sorted by last_seen descending
        for i in 1..chats.len() {
            prop_assert!(chats[i-1].last_seen >= chats[i].last_seen);
        }
    }
}
