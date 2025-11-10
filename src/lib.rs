//! Telegram Bot Debugger Library
//!
//! This library provides the core functionality for the Telegram Bot Debugger TUI application.
//! It can also be used as a library for building custom Telegram bot debugging tools.

pub mod analytics;
pub mod app;
pub mod input;
pub mod storage;
pub mod telegram;
pub mod ui;

#[cfg(test)]
mod test_utils;

// Re-export commonly used types
pub use app::{App, Screen};
pub use telegram::{TelegramClient, UpdateProcessor};
pub use storage::CacheManager;

