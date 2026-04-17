//! # Gateorix Host Core
//!
//! The native host runtime for the Gateorix desktop framework.
//! Manages the application lifecycle, windows, IPC bridge, permissions,
//! runtime adapters, and the plugin system.

pub mod app;
pub mod ipc;
pub mod permissions;
pub mod plugins;
pub mod runtime;
pub mod window;
