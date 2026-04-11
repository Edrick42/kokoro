//! Server-side modules — auth API calls and session management.
//!
//! Server function types are visible to both SSR and hydrate.
//! Function bodies only compile on SSR (handled by `#[server]` macro).

pub mod auth;
