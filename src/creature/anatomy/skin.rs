//! Skin — protection and sensation.
//!
//! Species-specific covering protects the body and mediates sensation:
//! - **Fur** (Moluun) — insulating, soft, moderate protection
//! - **Plumage** (Pylum) — lightweight, temperature regulation
//! - **Scales** (Skael) — armored, mineralized plates
//! - **Membrane** (Nyxal) — thin, permeable, bioluminescent

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinLayer {
    /// Type of skin covering.
    pub covering: SkinCovering,
    /// 0.0 = damaged, 1.0 = healthy.
    pub integrity: f32,
    /// Protection amount. Higher = more damage absorption.
    pub thickness: f32,
    /// 0.0 = cracked/dry, 1.0 = healthy.
    pub hydration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinCovering {
    Fur,
    Plumage,
    Scales,
    Membrane,
}
