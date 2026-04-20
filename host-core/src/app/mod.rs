//! Application lifecycle management.
//!
//! Handles initialization, configuration loading, and graceful shutdown
//! of the Gateorix host process.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level application manifest loaded from `gateorix.config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
    /// Frontend configuration.
    pub frontend: FrontendConfig,
    /// Runtime adapter configuration.
    pub runtime: Option<RuntimeConfig>,
    /// Permission declarations.
    pub permissions: PermissionConfig,
    /// Window definitions.
    #[serde(default)]
    pub windows: Vec<WindowDef>,
}

/// Frontend asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// Dev server URL used during development.
    #[serde(rename = "devUrl")]
    pub dev_url: Option<String>,
    /// Path to the production entry HTML file.
    pub entry: String,
}

/// Runtime adapter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Adapter type identifier (e.g. "python", "go", "dotnet").
    #[serde(rename = "type")]
    pub runtime_type: String,
    /// Entry point for the runtime (e.g. "backend/main.py").
    pub entry: String,
}

/// Manifest-level permission declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// Allowed filesystem paths. Empty means no filesystem access.
    #[serde(default)]
    pub filesystem: Vec<String>,
    /// Whether process execution is allowed.
    #[serde(default)]
    pub process: bool,
    /// Whether notifications are allowed.
    #[serde(default)]
    pub notifications: bool,
    /// Whether clipboard access is allowed.
    #[serde(default)]
    pub clipboard: bool,
}

/// A window definition from the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowDef {
    /// Unique window identifier.
    pub id: String,
    /// Window title.
    pub title: String,
    /// Initial width in logical pixels.
    #[serde(default = "default_width")]
    pub width: u32,
    /// Initial height in logical pixels.
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_width() -> u32 {
    1200
}
fn default_height() -> u32 {
    800
}

/// Load the application manifest from the given project directory.
pub fn load_manifest(project_dir: &Path) -> Result<AppManifest, Box<dyn std::error::Error>> {
    let config_path = project_dir.join("gateorix.config.json");
    let content = std::fs::read_to_string(&config_path)?;
    let manifest: AppManifest = serde_json::from_str(&content)?;
    Ok(manifest)
}
