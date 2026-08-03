//! Service layer — business logic orchestrating repositories + protocol.
//!
//! Each service abstracts a bounded domain, keeping `commands/` thin
//! (IPC param validation + response serialization only).

pub mod build_service;
pub mod stash_service;

pub use stash_service::StashService;
