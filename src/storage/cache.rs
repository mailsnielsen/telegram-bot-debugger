use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::models::CacheData;

/// Manages persistent storage of application data.
///
/// The CacheManager handles saving and loading data to/from disk,
/// including bot tokens, discovered chats, and analytics.
///
/// # Examples
///
/// ```no_run
/// use telegram_bot_debugger::storage::CacheManager;
///
/// let manager = CacheManager::new();
///
/// // Save a bot token
/// manager.save_token("123456:ABC-DEF".to_string())?;
///
/// // Load it back
/// let token = manager.load_token()?;
/// assert!(token.is_some());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct CacheManager {
    cache_path: PathBuf,
}

impl CacheManager {
    /// Creates a new CacheManager with the default cache path (`config/cache.json`).
    pub fn new() -> Self {
        Self {
            cache_path: PathBuf::from("config/cache.json"),
        }
    }

    /// Creates a new CacheManager with a custom cache file path.
    ///
    /// Useful for testing or alternative storage locations.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the cache file
    #[allow(dead_code)]
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            cache_path: path.as_ref().to_path_buf(),
        }
    }

    /// Loads cached data from disk.
    ///
    /// Returns empty CacheData if the cache file doesn't exist yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load(&self) -> Result<CacheData> {
        if !self.cache_path.exists() {
            return Ok(CacheData::default());
        }

        let content = fs::read_to_string(&self.cache_path).context("Failed to read cache file")?;

        let data = serde_json::from_str(&content).context("Failed to parse cache file")?;

        Ok(data)
    }

    /// Saves cache data to disk.
    ///
    /// Creates the parent directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `data` - The cache data to persist
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory cannot be created
    /// - The data cannot be serialized
    /// - The file cannot be written
    pub fn save(&self, data: &CacheData) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent).context("Failed to create cache directory")?;
        }

        let content =
            serde_json::to_string_pretty(data).context("Failed to serialize cache data")?;

        fs::write(&self.cache_path, content).context("Failed to write cache file")?;

        Ok(())
    }

    /// Saves a bot token to the cache.
    ///
    /// Loads existing cache data, updates the token, and saves it back.
    ///
    /// # Arguments
    ///
    /// * `token` - The bot token to save
    ///
    /// # Errors
    ///
    /// Returns an error if loading or saving fails.
    pub fn save_token(&self, token: String) -> Result<()> {
        let mut data = self.load()?;
        data.token = Some(token);
        self.save(&data)
    }

    /// Loads the bot token from the cache.
    ///
    /// # Returns
    ///
    /// `Some(String)` if a token was previously saved, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache file cannot be read or parsed.
    pub fn load_token(&self) -> Result<Option<String>> {
        let data = self.load()?;
        Ok(data.token)
    }

    /// Deletes the cache file from disk.
    ///
    /// Useful for testing or resetting the application state.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be deleted.
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path).context("Failed to remove cache file")?;
        }
        Ok(())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        let data = manager.load().unwrap();
        assert!(data.token.is_none());
        assert_eq!(data.chats.len(), 0);
    }

    #[test]
    fn test_save_and_load_token() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        let test_token = "123456:ABC-DEF".to_string();
        manager.save_token(test_token.clone()).unwrap();

        let loaded_token = manager.load_token().unwrap();
        assert_eq!(loaded_token, Some(test_token));
    }

    #[test]
    fn test_clear_removes_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        // Create a cache file
        manager.save_token("test_token".to_string()).unwrap();
        assert!(cache_path.exists());

        // Clear should remove it
        manager.clear().unwrap();
        assert!(!cache_path.exists());
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("nested/deep/cache.json");
        let manager = CacheManager::with_path(&cache_path);

        // Should create nested directories
        let data = CacheData::default();
        assert!(manager.save(&data).is_ok());
        assert!(cache_path.exists());
    }

    #[test]
    fn test_load_corrupted_json() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");

        // Write invalid JSON
        std::fs::write(&cache_path, "not valid json{{{").unwrap();

        let manager = CacheManager::with_path(&cache_path);
        let result = manager.load();

        // Should return error for corrupted JSON
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_complete_cache_data() {
        use crate::storage::models::CachedChat;

        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        let data = CacheData {
            token: Some("test_token".to_string()),
            chats: vec![CachedChat {
                chat_id: 100,
                chat_type: "private".to_string(),
                title: None,
                username: Some("testuser".to_string()),
                first_name: Some("Test".to_string()),
                last_name: None,
                last_seen: 1000,
                message_count: 5,
                topics: vec![],
            }],
            analytics: Default::default(),
        };

        manager.save(&data).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.token, Some("test_token".to_string()));
        assert_eq!(loaded.chats.len(), 1);
        assert_eq!(loaded.chats[0].chat_id, 100);
        assert_eq!(loaded.chats[0].message_count, 5);
    }

    #[test]
    fn test_save_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        // Save first token
        manager.save_token("token1".to_string()).unwrap();
        let first = manager.load_token().unwrap();
        assert_eq!(first, Some("token1".to_string()));

        // Overwrite with second token
        manager.save_token("token2".to_string()).unwrap();
        let second = manager.load_token().unwrap();
        assert_eq!(second, Some("token2".to_string()));
    }

    #[test]
    fn test_load_token_from_empty_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        // Write empty cache
        let data = CacheData::default();
        manager.save(&data).unwrap();

        let token = manager.load_token().unwrap();
        assert!(token.is_none());
    }

    #[test]
    fn test_save_preserves_existing_data() {
        use crate::storage::models::CachedChat;

        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let manager = CacheManager::with_path(&cache_path);

        // Save initial data with chats
        let data = CacheData {
            chats: vec![CachedChat {
                chat_id: 100,
                chat_type: "private".to_string(),
                title: None,
                username: None,
                first_name: Some("User".to_string()),
                last_name: None,
                last_seen: 1000,
                message_count: 10,
                topics: vec![],
            }],
            ..Default::default()
        };
        manager.save(&data).unwrap();

        // Update token using save_token (which loads, modifies, saves)
        manager.save_token("new_token".to_string()).unwrap();

        // Verify chats are still there
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.token, Some("new_token".to_string()));
        assert_eq!(loaded.chats.len(), 1);
        assert_eq!(loaded.chats[0].chat_id, 100);
    }
}
