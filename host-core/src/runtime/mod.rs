//! Runtime adapter layer.
//!
//! Manages sidecar processes for backend language runtimes (Python, Go, .NET,
//! Swift, etc.). The host core spawns these processes, communicates over
//! stdio or local HTTP, and relays IPC messages between the frontend and
//! the runtime.

pub mod adapter;
pub mod process;
