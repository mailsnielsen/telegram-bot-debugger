//! Live monitoring service for real-time updates.
//!
//! Manages background task for fetching and processing Telegram updates in real-time.

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::telegram::{TelegramClient, Update};

/// A monitored message displayed in the live monitor screen.
#[derive(Debug, Clone)]
pub struct MonitorMessage {
    pub timestamp: i64,
    pub chat_name: String,
    pub sender: Option<String>,
    pub text: String,
}

/// Commands sent to the monitoring background task.
#[derive(Debug, Clone)]
pub enum MonitoringCommand {
    /// Stop the monitoring task
    Stop,
}

/// Manages the live monitoring background task.
///
/// Coordinates a background async task that continuously fetches updates
/// from Telegram and sends them to the main application via a channel.
pub struct MonitoringService {
    /// Whether monitoring is currently active
    active: bool,

    /// Whether new messages should be added to the monitor display
    pub paused: bool,

    /// Messages displayed in the monitor screen
    pub messages: Vec<MonitorMessage>,

    /// Background task handle
    task_handle: Option<JoinHandle<()>>,

    /// Channel for receiving updates from the background task
    update_receiver: Option<mpsc::Receiver<Vec<Update>>>,

    /// Channel for sending control commands to the background task
    control_sender: Option<mpsc::Sender<MonitoringCommand>>,
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            active: false,
            paused: false,
            messages: Vec::new(),
            task_handle: None,
            update_receiver: None,
            control_sender: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Starts the monitoring background task.
    pub fn start(&mut self, client: TelegramClient, last_update_id: i64) {
        if self.task_handle.is_some() {
            return; // Already running
        }

        let (update_tx, update_rx) = mpsc::channel::<Vec<Update>>(100);
        let (control_tx, mut control_rx) = mpsc::channel::<MonitoringCommand>(10);

        let mut current_update_id = last_update_id;

        let handle = tokio::spawn(async move {
            loop {
                // Check for control commands (non-blocking)
                if let Ok(cmd) = control_rx.try_recv() {
                    match cmd {
                        MonitoringCommand::Stop => break,
                    }
                }

                // Fetch updates with short timeout
                let offset = Some(current_update_id + 1);
                match client.get_updates(offset, Some(1)).await {
                    Ok(response) => {
                        if response.ok && !response.result.is_empty() {
                            // Update offset for next iteration
                            for update in &response.result {
                                if update.update_id > current_update_id {
                                    current_update_id = update.update_id;
                                }
                            }

                            // Send updates to main app
                            if update_tx.send(response.result).await.is_err() {
                                break; // Channel closed, stop task
                            }
                        }
                    }
                    Err(_) => {
                        // Continue on error, will retry after sleep
                    }
                }

                // Sleep between checks
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        self.task_handle = Some(handle);
        self.update_receiver = Some(update_rx);
        self.control_sender = Some(control_tx);
        self.active = true;
    }

    /// Stops the monitoring background task.
    pub async fn stop(&mut self) {
        if let Some(sender) = self.control_sender.take() {
            let _ = sender.send(MonitoringCommand::Stop).await;
        }

        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }

        self.update_receiver = None;
        self.active = false;
    }

    /// Toggles monitoring on/off.
    pub async fn toggle(&mut self, client: TelegramClient, last_update_id: i64) {
        if self.active {
            self.stop().await;
        } else {
            self.start(client, last_update_id);
        }
    }

    /// Receives updates from the background task without blocking.
    ///
    /// Returns `Some(Vec<Update>)` if updates are available, `None` if channel is empty.
    pub fn receive_updates(&mut self) -> Option<Vec<Vec<Update>>> {
        if let Some(receiver) = &mut self.update_receiver {
            let mut all_updates = Vec::new();
            while let Ok(updates) = receiver.try_recv() {
                all_updates.push(updates);
            }
            if !all_updates.is_empty() {
                return Some(all_updates);
            }
        }
        None
    }
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new()
    }
}
