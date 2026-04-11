//! Kokoro Shared — single source of truth for game + web API.
//!
//! All species data, food types, gene ranges, biome info, and lore text
//! live here. Both the Bevy game and the Axum API import from this crate.
//! Change once, update everywhere.

pub mod species;
pub mod food;
pub mod biomes;
pub mod genes;
pub mod shop;

/// Convenience: all shop items.
pub fn shop_items() -> &'static [shop::ShopItem] {
    shop::all_items()
}
