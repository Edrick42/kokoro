//! Sound system — audio infrastructure with placeholder support.
//!
//! Handles creature vocalizations, UI sounds, and ambient audio.
//! Each species has a vocal repertoire that can grow through interaction.
//! Sound files are loaded from `assets/sounds/{species}/`.

use bevy::prelude::*;

use crate::creature::species::CreatureRoot;
use crate::creature::touch::TouchEvent;
use crate::genome::Genome;
use crate::mind::{Mind, MoodState};

/// Sound categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSound {
    // Creature vocalizations
    CreatureHappy,
    CreatureHungry,
    CreatureSleepy,
    CreaturePet,
    CreatureFlinch,
    CreatureGreeting,

    // UI
    ButtonClick,
    MenuOpen,
    MenuClose,
    FoodSelect,

    // Ambient
    Ambient,
}

/// Vocal repertoire — tracks what sounds the creature knows.
#[derive(Component, Debug, Clone)]
pub struct VocalRepertoire {
    pub base_count: usize,
    pub learned_count: usize,
    pub capacity: usize,
    pub learning_progress: f32,
    pub learning_speed: f32,
}

impl VocalRepertoire {
    pub fn new(species: &crate::genome::Species) -> Self {
        use crate::config::communication::vocal;
        Self {
            base_count: vocal::base_sounds(species),
            learned_count: 0,
            capacity: vocal::max_capacity(species),
            learning_progress: 0.0,
            learning_speed: vocal::learning_speed(species),
        }
    }

    pub fn total_sounds(&self) -> usize {
        self.base_count + self.learned_count
    }

    pub fn can_learn(&self) -> bool {
        self.total_sounds() < self.capacity
    }

    /// Advance learning progress. Returns true if a new sound was learned.
    pub fn advance_learning(&mut self, amount: f32) -> bool {
        if !self.can_learn() { return false; }
        self.learning_progress += amount * self.learning_speed;
        if self.learning_progress >= 1.0 {
            self.learning_progress = 0.0;
            self.learned_count += 1;
            true
        } else {
            false
        }
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            sound_on_mood_change,
            sound_on_touch,
            vocal_learning_system,
        ));
    }
}

/// Plays a sound (log-only for now) when mood changes.
fn sound_on_mood_change(
    mind: Res<Mind>,
    genome: Res<Genome>,
    repertoire_q: Query<&VocalRepertoire, With<CreatureRoot>>,
) {
    if !mind.is_changed() { return; }

    let Ok(rep) = repertoire_q.get_single() else { return };

    let sound = match mind.mood {
        MoodState::Happy | MoodState::Playful => GameSound::CreatureHappy,
        MoodState::Hungry => GameSound::CreatureHungry,
        MoodState::Sleeping | MoodState::Tired => GameSound::CreatureSleepy,
        _ => return,
    };

    debug!(
        "Sound: {:?} (repertoire: {}/{} sounds, species: {:?})",
        sound, rep.total_sounds(), rep.capacity, genome.species
    );
}

/// Plays appropriate sound on touch events.
fn sound_on_touch(
    mut events: EventReader<TouchEvent>,
) {
    for event in events.read() {
        let sound = if event.pleasure > 0.5 {
            GameSound::CreaturePet
        } else if event.pain > 0.5 {
            GameSound::CreatureFlinch
        } else {
            continue;
        };
        debug!("Sound: {:?} (touched {})", sound, event.slot);
    }
}

/// Slowly expands vocal repertoire through interaction.
fn vocal_learning_system(
    mind: Res<Mind>,
    mut repertoire_q: Query<&mut VocalRepertoire, With<CreatureRoot>>,
) {
    // Learn a tiny bit each tick when awake and happy
    if mind.mood == MoodState::Sleeping { return; }

    let Ok(mut rep) = repertoire_q.get_single_mut() else { return };

    let learn_amount = if mind.mood == MoodState::Playful {
        0.002 // learns fastest when playful
    } else if mind.mood == MoodState::Happy {
        0.001
    } else {
        0.0
    };

    if learn_amount > 0.0 && rep.advance_learning(learn_amount) {
        info!(
            "Creature learned a new sound! ({}/{} total)",
            rep.total_sounds(), rep.capacity
        );
    }
}
