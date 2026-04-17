//! Window management abstractions.
//!
//! Provides types and traits for creating, configuring, and controlling
//! application windows. The concrete implementation will be backed by
//! the platform webview (via Tauri/wry).

use serde::{Deserialize, Serialize};

/// Unique identifier for a managed window.
pub type WindowId = String;

/// Configuration for creating a new window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub id: WindowId,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub decorations: bool,
    pub url: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            id: "main".into(),
            title: "Gateorix App".into(),
            width: 1200,
            height: 800,
            resizable: true,
            decorations: true,
            url: "index.html".into(),
        }
    }
}

/// Trait for a window manager backend.
///
/// Implementations handle the platform-specific details of creating
/// and controlling webview windows.
pub trait WindowManager {
    /// Create and show a new window with the given configuration.
    fn create_window(&mut self, config: WindowConfig) -> Result<(), WindowError>;

    /// Close the window with the given ID.
    fn close_window(&mut self, id: &WindowId) -> Result<(), WindowError>;

    /// Set the title of an existing window.
    fn set_title(&mut self, id: &WindowId, title: &str) -> Result<(), WindowError>;
}

/// Errors that can occur during window operations.
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("window not found: {0}")]
    NotFound(WindowId),

    #[error("window already exists: {0}")]
    AlreadyExists(WindowId),

    #[error("platform error: {0}")]
    Platform(String),
}
