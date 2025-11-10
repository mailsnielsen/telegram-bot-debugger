//! Application state and business logic.
//!
//! This module is organized into focused sub-modules following Single Responsibility Principle:
//! - [`state`] - UI state management
//! - [`telegram_manager`] - Telegram API integration
//! - [`monitoring`] - Live monitoring service

pub mod monitoring;
pub mod state;
pub mod telegram_manager;

use anyhow::Result;
use chrono::Local;
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::analytics::Statistics;
use crate::storage::CacheManager;
use monitoring::MonitoringService;
use state::UiState;
use telegram_manager::TelegramManager;

pub use state::{InputFocus, Screen, TestMessageMode};

/// Main application facade.
///
/// Coordinates between UI state, Telegram integration, and monitoring services.
/// Follows the Facade pattern to provide a simple interface to complex subsystems.
pub struct App {
    pub ui: UiState,
    pub telegram: TelegramManager,
    pub monitoring: MonitoringService,
    pub cache_manager: CacheManager,
}

impl App {
    pub fn new() -> Result<Self> {
        let cache_manager = CacheManager::new();
        let token = cache_manager.load_token()?;

        let (telegram, initial_screen) = if let Some(token) = token {
            (TelegramManager::new_with_token(token), Screen::Home)
        } else {
            (TelegramManager::new(), Screen::TokenInput)
        };

        let mut ui = UiState::new();
        ui.current_screen = initial_screen;

        Ok(Self {
            ui,
            telegram,
            monitoring: MonitoringService::new(),
            cache_manager,
        })
    }

    // Delegate to UI state
    pub fn quit(&mut self) {
        self.ui.quit();
    }

    pub fn mark_dirty(&mut self) {
        self.ui.mark_dirty();
    }

    pub fn clear_dirty(&mut self) {
        self.ui.clear_dirty();
    }

    pub fn switch_screen(&mut self, screen: Screen) {
        self.ui.switch_screen(screen);

        // Precompute analytics when switching to analytics screen
        if screen == Screen::Analytics {
            let chats = self.telegram.get_discovered_chats();
            self.ui.statistics = Some(Statistics::from_chats(&chats));
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.ui.set_status(message);
    }

    // Navigation delegates
    pub fn next_chat(&mut self) {
        let chat_count = self.telegram.get_discovered_chats().len();
        self.ui.next_chat(chat_count);
    }

    pub fn previous_chat(&mut self) {
        let chat_count = self.telegram.get_discovered_chats().len();
        self.ui.previous_chat(chat_count);
    }

    pub fn next_message(&mut self) {
        if let Some(chat) = self.get_selected_chat() {
            let message_count = self.telegram.get_messages_for_chat(chat.chat.id).len();
            self.ui.next_message(message_count);
        }
    }

    pub fn previous_message(&mut self) {
        if let Some(chat) = self.get_selected_chat() {
            let message_count = self.telegram.get_messages_for_chat(chat.chat.id).len();
            self.ui.previous_message(message_count);
        }
    }

    pub fn next_update(&mut self) {
        let update_count = self.telegram.raw_updates.len();
        self.ui.next_update(update_count);
    }

    pub fn previous_update(&mut self) {
        let update_count = self.telegram.raw_updates.len();
        self.ui.previous_update(update_count);
    }

    // Telegram delegates
    pub fn get_discovered_chats(&self) -> Vec<&crate::telegram::DiscoveredChat> {
        self.telegram.get_discovered_chats()
    }

    pub fn get_selected_chat(&self) -> Option<&crate::telegram::DiscoveredChat> {
        let chats = self.get_discovered_chats();
        chats.get(self.ui.selected_chat_index).copied()
    }

    pub fn get_selected_message_for_current_chat(
        &self,
    ) -> Option<&std::sync::Arc<crate::telegram::Update>> {
        self.telegram.get_selected_message_for_chat(
            self.get_selected_chat().map(|c| c.chat.id),
            self.ui.selected_message_index,
        )
    }

    pub async fn validate_and_save_token(&mut self) -> Result<()> {
        let validation_result = self.telegram.validate_token(&self.ui.token_input).await?;

        match validation_result {
            telegram_manager::TokenValidationResult::Valid(client) => {
                self.cache_manager
                    .save_token(client.get_token().to_string())?;
                self.ui.token_error = None;
                self.switch_screen(Screen::Home);
                self.set_status("Token validated successfully!".to_string());
            }
            telegram_manager::TokenValidationResult::Empty => {
                self.ui.token_error = Some("Token cannot be empty".to_string());
                self.ui.mark_dirty();
            }
            telegram_manager::TokenValidationResult::TooLong(max) => {
                self.ui.token_error = Some(format!("Token too long (max {max} characters)"));
                self.ui.mark_dirty();
            }
            telegram_manager::TokenValidationResult::Invalid(msg) => {
                self.ui.token_error = Some(msg);
                self.ui.mark_dirty();
            }
        }

        Ok(())
    }

    pub async fn send_test_message(&mut self) -> Result<()> {
        let result = self
            .telegram
            .send_test_message(
                &self.ui.test_message_input,
                &self.ui.manual_chat_id_input,
                self.ui.test_message_mode,
                self.get_selected_chat(),
            )
            .await?;

        self.ui.test_message_result = Some(result.message);
        if result.success {
            self.ui.test_message_input.clear();
        }
        self.ui.mark_dirty();

        Ok(())
    }

    // Export methods
    pub fn export_selected_chat(&mut self) -> Result<()> {
        let chat_option = self.get_selected_chat().cloned();
        self.export_selected_generic(
            chat_option,
            |chat| format!("chat_{}", chat.chat.id),
            "No chat selected to export",
        )
    }

    pub fn export_selected_message(&mut self) -> Result<()> {
        let message_option = self.telegram.get_selected_message_for_chat(
            self.get_selected_chat().map(|c| c.chat.id),
            self.ui.selected_message_index,
        );
        self.export_selected_generic(
            message_option.map(|arc| arc.as_ref().clone()),
            |message| format!("message_{}", message.update_id),
            "No message selected to export",
        )
    }

    pub fn export_selected_update(&mut self) -> Result<()> {
        let update_option = self
            .telegram
            .get_selected_update(self.ui.selected_update_index);
        self.export_selected_generic(
            update_option.map(|arc| arc.as_ref().clone()),
            |update| format!("update_{}", update.update_id),
            "No update selected to export",
        )
    }

    fn export_selected_generic<T: Serialize>(
        &mut self,
        item: Option<T>,
        name_fn: impl FnOnce(&T) -> String,
        none_message: &str,
    ) -> Result<()> {
        match item {
            Some(data) => {
                let name = name_fn(&data);
                let filepath = self.export_to_json(&data, &name)?;
                self.set_status(format!("Exported to: {filepath}"));
            }
            None => {
                self.set_status(none_message.to_string());
            }
        }
        Ok(())
    }

    fn export_to_json<T: Serialize>(&mut self, data: &T, base_name: &str) -> Result<String> {
        let export_dir = Path::new("exports");
        if !export_dir.exists() {
            fs::create_dir(export_dir)?;
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{base_name}_{timestamp}.json");
        let filepath = export_dir.join(&filename);

        let json = serde_json::to_string_pretty(data)?;
        fs::write(&filepath, json)?;

        Ok(filepath.to_string_lossy().to_string())
    }

    // Monitoring delegates
    pub async fn toggle_monitoring(&mut self) {
        if let Some(client) = &self.telegram.client {
            let last_update_id = self.telegram.last_processed_update_id;
            self.monitoring.toggle(client.clone(), last_update_id).await;

            let status = if self.monitoring.is_active() {
                "Monitoring started"
            } else {
                "Monitoring stopped"
            };
            self.set_status(status.to_string());
        }
    }

    pub async fn stop_monitoring(&mut self) {
        self.monitoring.stop().await;
    }

    pub fn toggle_test_message_mode(&mut self) {
        self.ui.toggle_test_message_mode();
    }

    pub fn toggle_input_focus(&mut self) {
        self.ui.toggle_input_focus();
    }

    pub fn go_back(&mut self) {
        self.ui.go_back();
    }

    pub fn toggle_monitor_pause(&mut self) {
        self.monitoring.toggle_pause();
        self.ui.mark_dirty();
    }

    pub fn process_received_updates(&mut self) {
        if let Some(updates_batch) = self.monitoring.receive_updates() {
            for updates in updates_batch {
                if self.monitoring.paused {
                    // Still process for update tracking, but don't add to monitor messages
                    self.telegram.update_processor.process_updates(updates);
                } else {
                    self.telegram
                        .process_updates_batch(updates, &mut self.monitoring.messages);
                }
            }
            self.ui.mark_dirty();
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
