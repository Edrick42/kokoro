use serde::{Deserialize, Serialize};

/// Available species of Kobara. Each has different body shapes, rigs,
/// and gene ranges.
///
/// All creatures are Kobaras — species determines their physical form:
/// - **Moluun** — soft, round, forest-dwelling Kobaras from the Verdance
/// - **Pylum** — winged, curious Kobaras from the Veridian Highlands
/// - **Skael** — scaled, resilient Kobaras from the Abyssal Shallows
/// - **Nyxal** — tentacled, intelligent Kobaras from the Abyssal Depths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Species {
    /// Round, soft, mammal-like Kobara from the Verdance forests.
    Moluun,
    /// Bird-like Kobara with wings and a beak from the Veridian Highlands.
    Pylum,
    /// Reptile-like Kobara with scales from the Abyssal Shallows.
    Skael,
    /// Tentacled, intelligent Kobara from the Abyssal Depths.
    Nyxal,
}
