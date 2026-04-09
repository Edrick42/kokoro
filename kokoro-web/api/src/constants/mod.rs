//! API constants organized by domain.
//!
//! Each submodule groups constants for a specific concern.
//! When the API grows, add new submodules — don't bloat existing ones.

pub mod server;
pub mod routes;
pub mod errors;

// Re-export server config at top level for convenience.
pub use server::ADDR as SERVER_ADDR;
