//! Error message templates — centralized for consistency.
//!
//! Functions instead of plain strings because messages include
//! dynamic data (names, IDs) that need formatting.

/// Species with given name was not found.
pub fn species_not_found(name: &str) -> String {
    format!("Species '{}' not found", name)
}

/// Biome with given name was not found.
pub fn biome_not_found(name: &str) -> String {
    format!("Biome '{}' not found", name)
}
