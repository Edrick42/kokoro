use bevy::prelude::Resource;
use serde::{Serialize, Deserialize};

#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct Pet {
    pub hunger: f32,
    pub happiness: f32,
    pub energy: f32,
    pub discipline: f32,
}

impl Pet {
    pub fn new() -> Self {
        Self {
            hunger: 50.0,
            happiness: 50.0,
            energy: 50.0,
            discipline: 50.0,
        }
    }

    pub fn feed(&mut self) {
        self.hunger = (self.hunger - 10.0).max(0.0);
        self.happiness = (self.happiness + 5.0).min(100.0);
    }

    pub fn play(&mut self) {
        self.happiness = (self.happiness + 10.0).min(100.0);
        self.energy = (self.energy - 5.0).max(0.0);
    }
}
