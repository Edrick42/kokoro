//! Route path constants — every endpoint URL defined in one place.
//!
//! When adding a new endpoint, define its path here first,
//! then reference it in router.rs.

// Health
pub const HEALTH: &str = "/health";

// Species
pub const SPECIES_LIST: &str = "/api/species";
pub const SPECIES_BY_NAME: &str = "/api/species/{name}";

// Biomes
pub const BIOME_LIST: &str = "/api/biomes";
pub const BIOME_BY_NAME: &str = "/api/biome/{name}";

// Foods
pub const FOOD_LIST: &str = "/api/foods";
