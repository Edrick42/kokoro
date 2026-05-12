//! Tail-specific genes — the heritable factors that vary tail anatomy
//! around the species average. Each gene is a normalised `f32` in
//! `0.0..=1.0`; the *physical* parameter (mass, ligament stiffness, etc.)
//! is derived from the gene by a species-specific mapping that lives in
//! the species's rig module.
//!
//! Keeping the gene normalised and the mapping species-local means a
//! single mutation (e.g. flexibility = 0.7) produces different physical
//! consequences for a Moluun (cat-like baseline) than for a Skael
//! (whatever its baseline becomes), without the genome needing to know
//! anything about species anatomy.

use serde::{Deserialize, Serialize};

/// Heritable factors for the tail.
///
/// Future expansion: add fields here (e.g., `ring_count`, `fur_density`)
/// without breaking persistence — they go through `#[serde(default)]`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TailGenes {
    /// Length factor around the species mean. 0 = shortest plausible,
    /// 1 = longest plausible. Species rig maps this to actual pixels.
    pub length: f32,

    /// Flexibility factor. 0 = stiff (rigid ligaments, low ROM),
    /// 1 = floppy (slack ligaments, full ROM). Species rig maps this to
    /// concrete ligament_k and ROM clamps.
    pub flexibility: f32,

    /// Muscle strength factor. 0 = weak (low max_force, slow contraction),
    /// 1 = powerful (high max_force, fast contraction).
    pub strength: f32,
}

impl Default for TailGenes {
    /// Mid-range defaults — used when loading legacy saves that pre-date
    /// the tail genes, and as the neutral seed for tests.
    fn default() -> Self {
        Self {
            length: 0.5,
            flexibility: 0.5,
            strength: 0.5,
        }
    }
}

impl TailGenes {
    /// Random tail genes. Per-species centering can be added when the
    /// other species have their tail baselines defined; for now every
    /// species samples the full 0..1 range and the species rig is
    /// responsible for the actual physical mapping.
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        Self {
            length: rng.random_range(0.0..=1.0),
            flexibility: rng.random_range(0.0..=1.0),
            strength: rng.random_range(0.0..=1.0),
        }
    }

    /// Mix two parents the same way the other genes do: pick one parent
    /// 50/50, then 15% chance of a ±0.1 mutation, clamped to 0..1.
    pub fn crossover(a: Self, b: Self, rng: &mut impl rand::Rng) -> Self {
        fn pick(rng: &mut impl rand::Rng, a: f32, b: f32) -> f32 {
            if rng.random_bool(0.5) { a } else { b }
        }
        fn mutate(rng: &mut impl rand::Rng, v: f32) -> f32 {
            if rng.random_range(0.0f32..1.0) < 0.15 {
                (v + rng.random_range(-0.1f32..0.1)).clamp(0.0, 1.0)
            } else {
                v
            }
        }
        // Borrow rng once per gene — picker first, then mutate.
        let length = pick(rng, a.length, b.length);
        let length = mutate(rng, length);
        let flexibility = pick(rng, a.flexibility, b.flexibility);
        let flexibility = mutate(rng, flexibility);
        let strength = pick(rng, a.strength, b.strength);
        let strength = mutate(rng, strength);
        Self { length, flexibility, strength }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn default_is_midpoint() {
        let g = TailGenes::default();
        assert!((g.length - 0.5).abs() < 1e-6);
        assert!((g.flexibility - 0.5).abs() < 1e-6);
        assert!((g.strength - 0.5).abs() < 1e-6);
    }

    #[test]
    fn random_stays_in_unit_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..200 {
            let g = TailGenes::random(&mut rng);
            for v in [g.length, g.flexibility, g.strength] {
                assert!((0.0..=1.0).contains(&v));
            }
        }
    }

    #[test]
    fn crossover_stays_in_unit_range() {
        let mut rng = StdRng::seed_from_u64(7);
        let a = TailGenes { length: 0.2, flexibility: 0.8, strength: 0.5 };
        let b = TailGenes { length: 0.9, flexibility: 0.1, strength: 0.6 };
        for _ in 0..200 {
            let c = TailGenes::crossover(a, b, &mut rng);
            for v in [c.length, c.flexibility, c.strength] {
                assert!((0.0..=1.0).contains(&v), "out of range: {v}");
            }
        }
    }
}
