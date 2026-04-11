//! Shop item definitions — premium foods and cosmetics.
//!
//! All items are priced in "etharin crystals" (virtual currency).
//! This is a scaffold — purchasing is not yet functional.

/// A purchasable item in the shop.
pub struct ShopItem {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: ShopCategory,
    pub price: u32,
}

/// Item categories.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShopCategory {
    PremiumFood,
    Cosmetic,
}

impl ShopCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PremiumFood => "Premium Foods",
            Self::Cosmetic => "Cosmetics",
        }
    }
}

/// All available shop items.
pub const ITEMS: &[ShopItem] = &[
    // Premium Foods
    ShopItem {
        id: "etharin_nectar",
        name: "Etharin Nectar",
        description: "Rare liquid distilled from pure etharin. Boosts all stats.",
        category: ShopCategory::PremiumFood,
        price: 50,
    },
    ShopItem {
        id: "crystal_bloom",
        name: "Crystal Bloom",
        description: "Flower that grows on resonance crystals. Restores energy fully.",
        category: ShopCategory::PremiumFood,
        price: 35,
    },
    ShopItem {
        id: "abyssal_truffle",
        name: "Abyssal Truffle",
        description: "Deep-sea fungus rich in nutrients. Maximum hunger satisfaction.",
        category: ShopCategory::PremiumFood,
        price: 40,
    },
    ShopItem {
        id: "resonance_berry",
        name: "Resonance Berry",
        description: "Berry that vibrates at kokoro-sac frequency. Doubles happiness gain.",
        category: ShopCategory::PremiumFood,
        price: 30,
    },
    // Cosmetics
    ShopItem {
        id: "golden_crown",
        name: "Golden Crown",
        description: "A tiny crown forged from Verdance gold. Pure style.",
        category: ShopCategory::Cosmetic,
        price: 80,
    },
    ShopItem {
        id: "verdance_scarf",
        name: "Verdance Scarf",
        description: "Woven from bioluminescent spores. Glows faintly at night.",
        category: ShopCategory::Cosmetic,
        price: 60,
    },
    ShopItem {
        id: "starlight_aura",
        name: "Starlight Aura",
        description: "Ethereal glow effect. Your Kobara shimmers like Ren's moonlight.",
        category: ShopCategory::Cosmetic,
        price: 100,
    },
    ShopItem {
        id: "elders_mark",
        name: "Elder's Mark",
        description: "Ancient pattern that appears on long-lived Kobaras. Wisdom made visible.",
        category: ShopCategory::Cosmetic,
        price: 120,
    },
];

/// Get all items of a given category.
pub fn items_by_category(category: ShopCategory) -> impl Iterator<Item = &'static ShopItem> {
    ITEMS.iter().filter(move |item| item.category == category)
}
