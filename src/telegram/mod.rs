//! Telegram Bot API integration module.
//!
//! This module provides a complete client implementation for the Telegram Bot API,
//! including type definitions, update processing, and chat discovery functionality.
//!
//! # Main Components
//!
//! - [`TelegramClient`] - HTTP client for making API requests
//! - [`UpdateProcessor`] - Processes updates and discovers chats/topics
//! - Type definitions for all Telegram API objects
//!
//! # Example
//!
//! ```no_run
//! use telegram_bot_debugger::telegram::TelegramClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = TelegramClient::new("YOUR_BOT_TOKEN".to_string());
//!     let me = client.get_me().await?;
//!     println!("Bot username: {:?}", me.result);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod types;
pub mod updates;

pub use client::{TelegramClient};
pub use types::*;
pub use updates::UpdateProcessor;

