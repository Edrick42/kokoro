//! Shop item definitions — shared between game and API.

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ShopItem {
    pub id: &'static str,
    pub name: &'static str,
    pub price: u32,
    pub category: &'static str,
}

/// All available shop items.
pub fn all_items() -> &'static [ShopItem] {
    &ITEMS
}

static ITEMS: [ShopItem; 8] = [
    ShopItem { id: "etharin_nectar",  name: "Etharin Nectar",  price: 50,  category: "food" },
    ShopItem { id: "crystal_bloom",   name: "Crystal Bloom",   price: 35,  category: "food" },
    ShopItem { id: "abyssal_truffle", name: "Abyssal Truffle", price: 40,  category: "food" },
    ShopItem { id: "resonance_berry", name: "Resonance Berry", price: 30,  category: "food" },
    ShopItem { id: "golden_crown",    name: "Golden Crown",    price: 80,  category: "cosmetic" },
    ShopItem { id: "verdance_scarf",  name: "Verdance Scarf",  price: 60,  category: "cosmetic" },
    ShopItem { id: "starlight_aura",  name: "Starlight Aura",  price: 100, category: "cosmetic" },
    ShopItem { id: "elders_mark",     name: "Elder's Mark",    price: 120, category: "cosmetic" },
];
