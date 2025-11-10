//! Integration tests for storage workflows.
//!
//! Tests complete save/load cycles and cache persistence scenarios.

use telegram_bot_debugger::storage::{CacheManager, models::{CacheData, CachedChat}};
use tempfile::TempDir;

#[test]
fn test_complete_storage_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("cache.json");
    
    // Create manager
    let manager = CacheManager::with_path(&cache_path);
    
    // Initially empty
    let data = manager.load().unwrap();
    assert!(data.token.is_none());
    assert_eq!(data.chats.len(), 0);
    
    // Save token
    manager.save_token("test_token_123".to_string()).unwrap();
    
    // Verify persistence
    let loaded = manager.load().unwrap();
    assert_eq!(loaded.token, Some("test_token_123".to_string()));
    
    // Save complete data
    let full_data = CacheData {
        token: Some("test_token_123".to_string()),
        chats: vec![
            CachedChat {
                chat_id: 100,
                chat_type: "private".to_string(),
                title: None,
                username: Some("testuser".to_string()),
                first_name: Some("Test".to_string()),
                last_name: None,
                last_seen: 1234567890,
                message_count: 42,
                topics: vec![],
            },
        ],
        analytics: Default::default(),
    };
    
    manager.save(&full_data).unwrap();
    
    // Verify complete data persistence
    let reloaded = manager.load().unwrap();
    assert_eq!(reloaded.token, Some("test_token_123".to_string()));
    assert_eq!(reloaded.chats.len(), 1);
    assert_eq!(reloaded.chats[0].chat_id, 100);
    assert_eq!(reloaded.chats[0].message_count, 42);
    
    // Clear and verify
    manager.clear().unwrap();
    assert!(!cache_path.exists());
}

#[test]
fn test_multiple_save_cycles() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(&cache_path);

    for i in 0..10 {
        let token = format!("token_{}", i);
        manager.save_token(token.clone()).unwrap();
        
        let loaded = manager.load_token().unwrap();
        assert_eq!(loaded, Some(token));
    }
}

#[test]
fn test_concurrent_manager_instances() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("cache.json");
    
    let manager1 = CacheManager::with_path(&cache_path);
    let manager2 = CacheManager::with_path(&cache_path);
    
    manager1.save_token("token1".to_string()).unwrap();
    
    let loaded_from_manager2 = manager2.load_token().unwrap();
    assert_eq!(loaded_from_manager2, Some("token1".to_string()));
}

