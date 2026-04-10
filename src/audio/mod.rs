//! Sound system — multi-layer audio driven by species, age, mood, and biome.
//!
//! ## Audio layers
//!
//! 1. **Vocalizations** — species × growth stage × mood (cub sounds high, elder sounds deep)
//! 2. **Action SFX** — eat, play, sleep, touch, hatch
//! 3. **Biome ambient** — looping environment per species (forest, highlands, caves, ocean)
//! 4. **Soundtrack** — lofi retro music per biome (looping)
//! 5. **UI sounds** — click, menu open/close
//! 6. **Heartbeat** — synced with BPM
//!
//! ## File structure
//!
//! ```text
//! assets/sounds/
//! ├── moluun/{cub,young,adult,elder}/  happy.ogg, hungry.ogg, sleepy.ogg
//! ├── pylum/...   skael/...   nyxal/...
//! ├── actions/    eat.ogg, play.ogg, sleep.ogg, touch.ogg, hatch.ogg
//! ├── biomes/     verdance.ogg, highlands.ogg, shallows.ogg, depths.ogg
//! ├── ui/         click.ogg, open.ogg, close.ogg
//! ├── heartbeat.ogg
//! └── music/      verdance.ogg, highlands.ogg, shallows.ogg, depths.ogg
//! ```
//!
//! Drop .ogg files — the system loads whatever exists, runs silent for the rest.

#[allow(dead_code)]
mod synth;
#[allow(dead_code)]
mod wav;
#[allow(dead_code)]
mod species_sounds;
pub mod heartbeat;
#[allow(dead_code)]
mod ui_sounds;

use std::collections::HashMap;
use bevy::prelude::*;
use bevy::audio::PlaybackSettings;

use crate::config;
use crate::creature::species::CreatureRoot;
use crate::creature::touch::TouchEvent;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};
use crate::visuals::evolution::{GrowthState, GrowthStage};

// ===================================================================
// Sound keys
// ===================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundKey {
    /// Vocalization: species × stage × mood
    Vocal(SpeciesKey, StageKey, MoodSound),
    /// Action sound effect
    Action(ActionSound),
    /// Looping biome ambient
    Biome(BiomeKey),
    /// Looping soundtrack per biome
    Music(BiomeKey),
    /// Heartbeat pulse
    Heartbeat,
    /// UI interaction
    Ui(UiSound),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpeciesKey { Moluun, Pylum, Skael, Nyxal }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageKey { Cub, Young, Adult, Elder }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoodSound { Happy, Hungry, Sleepy }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionSound { Eat, Play, Sleep, Touch, Hatch }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeKey { Verdance, Highlands, Shallows, Depths }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSound { Click, MenuOpen, MenuClose }

impl From<&Species> for SpeciesKey {
    fn from(s: &Species) -> Self {
        match s {
            Species::Moluun => SpeciesKey::Moluun,
            Species::Pylum  => SpeciesKey::Pylum,
            Species::Skael  => SpeciesKey::Skael,
            Species::Nyxal  => SpeciesKey::Nyxal,
        }
    }
}

impl From<&GrowthStage> for StageKey {
    fn from(s: &GrowthStage) -> Self {
        match s {
            GrowthStage::Cub   => StageKey::Cub,
            GrowthStage::Young => StageKey::Young,
            GrowthStage::Adult => StageKey::Adult,
            GrowthStage::Elder => StageKey::Elder,
        }
    }
}

impl From<&Species> for BiomeKey {
    fn from(s: &Species) -> Self {
        match s {
            Species::Moluun => BiomeKey::Verdance,
            Species::Pylum  => BiomeKey::Highlands,
            Species::Skael  => BiomeKey::Shallows,
            Species::Nyxal  => BiomeKey::Depths,
        }
    }
}

fn species_dir(s: &SpeciesKey) -> &'static str {
    match s { SpeciesKey::Moluun => "moluun", SpeciesKey::Pylum => "pylum",
              SpeciesKey::Skael => "skael", SpeciesKey::Nyxal => "nyxal" }
}

fn stage_dir(s: &StageKey) -> &'static str {
    match s { StageKey::Cub => "cub", StageKey::Young => "young",
              StageKey::Adult => "adult", StageKey::Elder => "elder" }
}

fn biome_file(b: &BiomeKey) -> &'static str {
    match b { BiomeKey::Verdance => "verdance", BiomeKey::Highlands => "highlands",
              BiomeKey::Shallows => "shallows", BiomeKey::Depths => "depths" }
}

// ===================================================================
// SoundBank
// ===================================================================

#[derive(Resource, Default)]
pub struct SoundBank {
    sounds: HashMap<SoundKey, Handle<AudioSource>>,
}

impl SoundBank {
    pub fn get(&self, key: &SoundKey) -> Option<Handle<AudioSource>> {
        self.sounds.get(key).cloned()
    }

    pub fn get_heartbeat(&self) -> Option<Handle<AudioSource>> {
        self.get(&SoundKey::Heartbeat)
    }
}

/// Tracks current ambient/music entities so we can stop them on switch.
#[derive(Resource, Default)]
struct AmbientState {
    biome_entity: Option<Entity>,
    music_entity: Option<Entity>,
    current_biome: Option<BiomeKey>,
}

// ===================================================================
// Vocal Repertoire
// ===================================================================

#[derive(Component, Debug, Clone)]
pub struct VocalRepertoire {
    pub base_count: usize,
    pub learned_count: usize,
    pub capacity: usize,
    pub learning_progress: f32,
    pub learning_speed: f32,
}

impl VocalRepertoire {
    pub fn new(species: &Species) -> Self {
        use crate::config::communication::vocal;
        Self {
            base_count: vocal::base_sounds(species),
            learned_count: 0,
            capacity: vocal::max_capacity(species),
            learning_progress: 0.0,
            learning_speed: vocal::learning_speed(species),
        }
    }

    pub fn total_sounds(&self) -> usize { self.base_count + self.learned_count }
    pub fn can_learn(&self) -> bool { self.total_sounds() < self.capacity }

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

// ===================================================================
// Plugin
// ===================================================================

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoundBank::default())
           .insert_resource(heartbeat::HeartbeatAudioState::default())
           .insert_resource(AmbientState::default())
           .add_systems(Startup, init_sound_bank)
           .add_systems(Update, (
               sound_on_mood_change,
               sound_on_touch,
               vocal_learning_system,
               heartbeat::heartbeat_audio_system,
               manage_ambient_and_music,
           ));
    }
}

// ===================================================================
// Startup: scan and load .ogg files
// ===================================================================

fn init_sound_bank(mut bank: ResMut<SoundBank>, asset_server: Res<AssetServer>) {
    let mut loaded = 0u32;

    // Vocalizations: species × stage × mood
    let species_keys = [SpeciesKey::Moluun, SpeciesKey::Pylum, SpeciesKey::Skael, SpeciesKey::Nyxal];
    let stage_keys = [StageKey::Cub, StageKey::Young, StageKey::Adult, StageKey::Elder];
    let mood_sounds = [
        (MoodSound::Happy, "happy.ogg"),
        (MoodSound::Hungry, "hungry.ogg"),
        (MoodSound::Sleepy, "sleepy.ogg"),
    ];

    for sk in &species_keys {
        for stk in &stage_keys {
            for (mood, file) in &mood_sounds {
                let path = format!("sounds/{}/{}/{}", species_dir(sk), stage_dir(stk), file);
                if try_load(&mut bank, &asset_server, SoundKey::Vocal(*sk, *stk, *mood), &path) {
                    loaded += 1;
                }
            }
        }
        // Also try species root (no stage subfolder) as fallback
        for (mood, file) in &mood_sounds {
            let path = format!("sounds/{}/{}", species_dir(sk), file);
            // Only load if stage-specific doesn't exist for ANY stage
            let stage_key = SoundKey::Vocal(*sk, StageKey::Adult, *mood);
            if !bank.sounds.contains_key(&stage_key) {
                if try_load(&mut bank, &asset_server, stage_key, &path) {
                    loaded += 1;
                }
            }
        }
    }

    // Action SFX
    for (action, file) in [
        (ActionSound::Eat, "actions/eat.ogg"),
        (ActionSound::Play, "actions/play.ogg"),
        (ActionSound::Sleep, "actions/sleep.ogg"),
        (ActionSound::Touch, "actions/touch.ogg"),
        (ActionSound::Hatch, "actions/hatch.ogg"),
    ] {
        if try_load(&mut bank, &asset_server, SoundKey::Action(action), &format!("sounds/{file}")) {
            loaded += 1;
        }
    }

    // Biome ambients (looping)
    for bk in [BiomeKey::Verdance, BiomeKey::Highlands, BiomeKey::Shallows, BiomeKey::Depths] {
        let path = format!("sounds/biomes/{}.ogg", biome_file(&bk));
        if try_load(&mut bank, &asset_server, SoundKey::Biome(bk), &path) { loaded += 1; }
    }

    // Music/soundtrack (looping)
    for bk in [BiomeKey::Verdance, BiomeKey::Highlands, BiomeKey::Shallows, BiomeKey::Depths] {
        let path = format!("sounds/music/{}.ogg", biome_file(&bk));
        if try_load(&mut bank, &asset_server, SoundKey::Music(bk), &path) { loaded += 1; }
    }

    // Heartbeat
    if try_load(&mut bank, &asset_server, SoundKey::Heartbeat, "sounds/heartbeat.ogg") { loaded += 1; }

    // UI
    for (ui, file) in [(UiSound::Click, "click.ogg"), (UiSound::MenuOpen, "open.ogg"), (UiSound::MenuClose, "close.ogg")] {
        if try_load(&mut bank, &asset_server, SoundKey::Ui(ui), &format!("sounds/ui/{file}")) { loaded += 1; }
    }

    if loaded > 0 {
        info!("SoundBank: loaded {} .ogg files", loaded);
    } else {
        info!("SoundBank: no .ogg files found — running silent. Drop files in assets/sounds/");
    }
}

fn try_load(bank: &mut SoundBank, asset_server: &AssetServer, key: SoundKey, path: &str) -> bool {
    if std::path::Path::new(&format!("assets/{path}")).exists() {
        bank.sounds.insert(key, asset_server.load(path));
        true
    } else {
        false
    }
}

// ===================================================================
// Systems
// ===================================================================

/// Plays vocalization on mood change (species × stage aware).
fn sound_on_mood_change(
    mind: Res<Mind>,
    genome: Res<Genome>,
    growth: Res<GrowthState>,
    bank: Res<SoundBank>,
    mut commands: Commands,
) {
    if !mind.is_changed() { return; }

    let sk = SpeciesKey::from(&genome.species);
    let stk = StageKey::from(&growth.stage);
    let mood = match mind.mood {
        MoodState::Happy | MoodState::Playful => MoodSound::Happy,
        MoodState::Hungry => MoodSound::Hungry,
        MoodState::Sleeping | MoodState::Tired => MoodSound::Sleepy,
        _ => return,
    };

    // Try stage-specific first, fall back to adult
    let key = SoundKey::Vocal(sk, stk, mood);
    let fallback = SoundKey::Vocal(sk, StageKey::Adult, mood);

    if let Some(handle) = bank.get(&key).or_else(|| bank.get(&fallback)) {
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(config::audio::VOCAL_VOLUME)),
        ));
    }
}

/// Plays touch sound + action SFX.
fn sound_on_touch(
    mut events: EventReader<TouchEvent>,
    bank: Res<SoundBank>,
    mut commands: Commands,
) {
    for _event in events.read() {
        if let Some(handle) = bank.get(&SoundKey::Action(ActionSound::Touch)) {
            commands.spawn((
                AudioPlayer::new(handle),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(config::audio::VOCAL_VOLUME)),
            ));
        }
    }
}

/// Manages looping biome ambient and music — switches when species changes.
fn manage_ambient_and_music(
    genome: Res<Genome>,
    bank: Res<SoundBank>,
    mut ambient: ResMut<AmbientState>,
    mut commands: Commands,
) {
    if !genome.is_changed() { return; }

    let biome = BiomeKey::from(&genome.species);

    // Already playing the right biome
    if ambient.current_biome == Some(biome) { return; }

    // Stop old ambient/music
    if let Some(entity) = ambient.biome_entity.take() {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = ambient.music_entity.take() {
        commands.entity(entity).despawn();
    }

    // Start new biome ambient (looping, quiet)
    if let Some(handle) = bank.get(&SoundKey::Biome(biome)) {
        let entity = commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(config::audio::AMBIENT_VOLUME)),
        )).id();
        ambient.biome_entity = Some(entity);
    }

    // Start new music (looping)
    if let Some(handle) = bank.get(&SoundKey::Music(biome)) {
        let entity = commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(config::audio::AMBIENT_VOLUME * 2.0)),
        )).id();
        ambient.music_entity = Some(entity);
    }

    ambient.current_biome = Some(biome);
}

/// Slowly expands vocal repertoire through interaction.
fn vocal_learning_system(
    mind: Res<Mind>,
    mut repertoire_q: Query<&mut VocalRepertoire, With<CreatureRoot>>,
) {
    if mind.mood == MoodState::Sleeping { return; }
    let Ok(mut rep) = repertoire_q.single_mut() else { return };

    let learn_amount = match mind.mood {
        MoodState::Playful => 0.002,
        MoodState::Happy   => 0.001,
        _                  => 0.0,
    };

    if learn_amount > 0.0 && rep.advance_learning(learn_amount) {
        info!("Creature learned a new sound! ({}/{} total)", rep.total_sounds(), rep.capacity);
    }
}
