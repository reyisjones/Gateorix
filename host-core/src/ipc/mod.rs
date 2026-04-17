//! IPC bridge — the secure communication layer between frontend, host core,
//! and runtime adapters.
//!
//! Submodules:
//! - `protocol` — message types, serialization, and contracts.
//! - `bridge` — the dispatcher that routes messages to the correct handler.

pub mod bridge;
pub mod protocol;
