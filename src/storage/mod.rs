//! Persistent storage and caching functionality.
//!
//! This module handles:
//! - Bot token persistence
//! - Chat information caching
//! - Analytics data storage
//!
//! Data is stored in JSON format in the `config/` directory.

pub mod cache;
pub mod models;

pub use cache::CacheManager;
