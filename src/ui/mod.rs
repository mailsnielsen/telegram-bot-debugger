//! Terminal user interface (TUI) components.
//!
//! This module contains the UI rendering logic using the `ratatui` library.
//! It includes:
//! - Main layout and frame rendering
//! - Individual screens for each application mode
//! - UI components and widgets

pub mod layout;
pub mod screens;

pub use layout::render_frame;

