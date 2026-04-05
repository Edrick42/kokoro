use rand::Rng;

use super::{Genome, Species};

impl Genome {
    /// Creates a child genome by crossing two parent genomes with mutation.
    #[allow(dead_code)]
    pub fn crossover(parent_a: &Genome, parent_b: &Genome, child_species: Species) -> Self {
        let mut rng = rand::rng();

        fn pick(rng: &mut impl Rng, a: f32, b: f32) -> f32 {
            if rng.random_bool(0.5) { a } else { b }
        }

        fn mutate(rng: &mut impl Rng, val: f32, min: f32, max: f32) -> f32 {
            if rng.random_range(0.0f32..1.0) < 0.15 {
                let shift = rng.random_range(-0.1f32..0.1);
                (val + shift).clamp(min, max)
            } else {
                val
            }
        }

        let c = pick(&mut rng, parent_a.curiosity, parent_b.curiosity);
        let curiosity = mutate(&mut rng, c, 0.0, 1.0);
        let l = pick(&mut rng, parent_a.loneliness_sensitivity, parent_b.loneliness_sensitivity);
        let loneliness_sensitivity = mutate(&mut rng, l, 0.0, 1.0);
        let a = pick(&mut rng, parent_a.appetite, parent_b.appetite);
        let appetite = mutate(&mut rng, a, 0.0, 1.0);
        let ci = pick(&mut rng, parent_a.circadian, parent_b.circadian);
        let circadian = mutate(&mut rng, ci, 0.0, 1.0);
        let r = pick(&mut rng, parent_a.resilience, parent_b.resilience);
        let resilience = mutate(&mut rng, r, 0.0, 1.0);
        let lr = pick(&mut rng, parent_a.learning_rate, parent_b.learning_rate);
        let learning_rate = mutate(&mut rng, lr, 0.0, 1.0);
        let h = pick(&mut rng, parent_a.hue, parent_b.hue);
        let hue = mutate(&mut rng, h / 360.0, 0.0, 1.0) * 360.0;

        Self {
            species: child_species,
            curiosity,
            loneliness_sensitivity,
            appetite,
            circadian,
            resilience,
            learning_rate,
            hue,
        }
    }
}
