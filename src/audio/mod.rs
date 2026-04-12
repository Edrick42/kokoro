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
#[allow(dead_code)]
pub mod voice_composer;

use std::collections::HashMap;
use bevy::prelude::*;
use bevy::audio::PlaybackSettings;

use crate::game::state::AppState;

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
            GrowthStage::Egg   => StageKey::Cub,  // egg uses cub sounds
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
    /// Multiple variants per sound key — picks one randomly for natural variety.
    /// e.g. happy.ogg, happy_2.ogg, happy_3.ogg → 3 variants for Happy
    sounds: HashMap<SoundKey, Vec<Handle<AudioSource>>>,
}

impl SoundBank {
    /// Returns a random variant for the given key, or None if no sounds loaded.
    pub fn get(&self, key: &SoundKey) -> Option<Handle<AudioSource>> {
        let variants = self.sounds.get(key)?;
        if variants.is_empty() { return None; }
        if variants.len() == 1 { return Some(variants[0].clone()); }
        // Pick a random variant
        use rand::Rng;
        let idx = rand::rng().random_range(0..variants.len());
        Some(variants[idx].clone())
    }

    pub fn get_heartbeat(&self) -> Option<Handle<AudioSource>> {
        self.get(&SoundKey::Heartbeat)
    }

    /// Adds a sound variant to a key. Multiple calls = multiple variants.
    fn add(&mut self, key: SoundKey, handle: Handle<AudioSource>) {
        self.sounds.entry(key).or_default().push(handle);
    }

    /// Total number of sound files loaded.
    #[allow(dead_code)]
    fn total_loaded(&self) -> usize {
        self.sounds.values().map(|v| v.len()).sum()
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
// Vocal Repertoire — sounds that unlock through interaction
// ===================================================================

/// Named vocal sounds that a creature can learn.
/// Each species has different sounds. Cubs start with only basic sounds
/// and unlock more through play, feeding, and interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VocalSound {
    // Basic (all species start with these)
    ContentHum,     // happy baseline
    HungerCall,     // asking for food
    SleepyMurmur,   // drowsy sound

    // Learned through play
    PlayfulChirp,   // unlocked by playing
    ExcitedTrill,   // unlocked by lots of play

    // Learned through feeding
    SatisfiedPurr,  // unlocked by good food
    FoodRequest,    // unlocked by learning preferences

    // Learned through touch
    PetResponse,    // unlocked by petting
    TrustSound,     // unlocked by consistent care

    // Learned through aging/experience
    GreetingCall,   // unlocked after reunion (Mirror Bond)
    WisdomDrone,    // elder-only, unlocked by age
}

/// Tracks which sounds the creature knows and how it's learning.
#[derive(Component, Debug, Clone)]
pub struct VocalRepertoire {
    /// Which sounds are unlocked.
    pub unlocked: Vec<VocalSound>,
    /// Maximum sounds this species can learn.
    pub capacity: usize,
    /// Learning speed multiplier (species-dependent).
    pub learning_speed: f32,
    /// Progress toward next sound (0.0-1.0).
    pub learning_progress: f32,
    /// Which mood the creature has been in most → determines which sound unlocks next.
    pub mood_affinity: MoodAffinity,
    /// Genome-driven pitch offset (-1.0 to 1.0). Makes each individual sound unique.
    pub pitch_offset: f32,
    /// Genome-driven volume intensity (0.5-1.0).
    pub intensity: f32,
}

/// Tracks which moods the creature spends most time in.
/// Determines what kind of sound unlocks next.
#[derive(Debug, Clone, Default)]
pub struct MoodAffinity {
    pub play_score: f32,
    pub feed_score: f32,
    pub touch_score: f32,
}

impl VocalRepertoire {
    pub fn new(species: &Species, genome: &crate::genome::Genome) -> Self {
        use crate::config::communication::vocal;

        // All creatures start with the 3 basic sounds
        let unlocked = vec![
            VocalSound::ContentHum,
            VocalSound::HungerCall,
            VocalSound::SleepyMurmur,
        ];

        // Genome drives voice uniqueness
        // hue → pitch offset (every creature sounds slightly different)
        let pitch_offset = (genome.hue / 360.0) * 0.4 - 0.2; // -0.2 to +0.2
        // resilience → intensity (tougher = louder)
        let intensity = 0.5 + genome.resilience * 0.5;

        Self {
            unlocked,
            capacity: vocal::max_capacity(species),
            learning_speed: vocal::learning_speed(species),
            learning_progress: 0.0,
            mood_affinity: MoodAffinity::default(),
            pitch_offset,
            intensity,
        }
    }

    pub fn total_sounds(&self) -> usize { self.unlocked.len() }
    pub fn can_learn(&self) -> bool { self.total_sounds() < self.capacity }

    /// Check if a specific sound is unlocked.
    pub fn knows(&self, sound: &VocalSound) -> bool {
        self.unlocked.contains(sound)
    }

    /// Returns the best sound for a mood, from what this creature knows.
    pub fn best_sound_for_mood(&self, mood: &MoodState) -> Option<MoodSound> {
        match mood {
            MoodState::Happy => {
                if self.knows(&VocalSound::SatisfiedPurr) { Some(MoodSound::Happy) }
                else if self.knows(&VocalSound::ContentHum) { Some(MoodSound::Happy) }
                else { None }
            }
            MoodState::Playful => {
                if self.knows(&VocalSound::ExcitedTrill) { Some(MoodSound::Happy) }
                else if self.knows(&VocalSound::PlayfulChirp) { Some(MoodSound::Happy) }
                else if self.knows(&VocalSound::ContentHum) { Some(MoodSound::Happy) }
                else { None }
            }
            MoodState::Hungry => {
                if self.knows(&VocalSound::FoodRequest) { Some(MoodSound::Hungry) }
                else if self.knows(&VocalSound::HungerCall) { Some(MoodSound::Hungry) }
                else { None }
            }
            MoodState::Sleeping | MoodState::Tired => {
                if self.knows(&VocalSound::SleepyMurmur) { Some(MoodSound::Sleepy) }
                else { None }
            }
            _ => None,
        }
    }

    /// Advance learning and potentially unlock a new sound.
    /// Returns the newly unlocked sound if one was learned.
    pub fn advance_learning(&mut self, amount: f32) -> Option<VocalSound> {
        if !self.can_learn() { return None; }
        self.learning_progress += amount * self.learning_speed;
        if self.learning_progress >= 1.0 {
            self.learning_progress = 0.0;
            let new_sound = self.next_sound_to_learn();
            if let Some(sound) = new_sound {
                self.unlocked.push(sound);
            }
            new_sound
        } else {
            None
        }
    }

    /// Determines which sound to unlock next based on mood affinity.
    fn next_sound_to_learn(&self) -> Option<VocalSound> {
        // Priority based on what the creature does most
        let candidates = if self.mood_affinity.play_score > self.mood_affinity.feed_score
            && self.mood_affinity.play_score > self.mood_affinity.touch_score
        {
            // Plays a lot → learns play sounds
            vec![VocalSound::PlayfulChirp, VocalSound::ExcitedTrill]
        } else if self.mood_affinity.feed_score > self.mood_affinity.touch_score {
            // Eats well → learns food sounds
            vec![VocalSound::SatisfiedPurr, VocalSound::FoodRequest]
        } else {
            // Petted a lot → learns trust sounds
            vec![VocalSound::PetResponse, VocalSound::TrustSound]
        };

        // Find first candidate not yet unlocked
        candidates.into_iter().find(|s| !self.unlocked.contains(s))
            .or_else(|| {
                // Fallback: any sound not yet learned
                let all = [
                    VocalSound::PlayfulChirp, VocalSound::ExcitedTrill,
                    VocalSound::SatisfiedPurr, VocalSound::FoodRequest,
                    VocalSound::PetResponse, VocalSound::TrustSound,
                    VocalSound::GreetingCall, VocalSound::WisdomDrone,
                ];
                all.into_iter().find(|s| !self.unlocked.contains(s))
            })
    }

    /// Applies playback settings modified by this creature's voice genetics.
    pub fn voice_settings(&self) -> PlaybackSettings {
        let speed = 1.0 + self.pitch_offset; // 0.8 to 1.2
        let vol = config::audio::VOCAL_VOLUME * self.intensity;
        PlaybackSettings::DESPAWN
            .with_speed(speed)
            .with_volume(bevy::audio::Volume::Linear(vol))
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
           ).run_if(in_state(AppState::Gameplay)));
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
        // Fallback: try species root (no stage subfolder)
        for (mood, file) in &mood_sounds {
            let key = SoundKey::Vocal(*sk, StageKey::Adult, *mood);
            if !bank.sounds.contains_key(&key) {
                let path = format!("sounds/{}/{}", species_dir(sk), file);
                if try_load(&mut bank, &asset_server, key, &path) { loaded += 1; }
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
        info!("SoundBank: no .ogg files found — drop files in assets/sounds/");
    }
}

/// Loads a sound file + all its variants.
///
/// Given path `sounds/moluun/adult/happy.ogg`, also scans for:
/// - `sounds/moluun/adult/happy_2.ogg`
/// - `sounds/moluun/adult/happy_3.ogg`
/// - ... up to `_9`
///
/// All variants are added to the same SoundKey — `bank.get()` picks one randomly.
fn try_load(bank: &mut SoundBank, asset_server: &AssetServer, key: SoundKey, path: &str) -> bool {
    let full_path = format!("assets/{path}");
    if !std::path::Path::new(&full_path).exists() {
        return false;
    }

    // Load the base file
    bank.add(key, asset_server.load(path));
    let mut count = 1u32;

    // Scan for variants: happy.ogg → happy_2.ogg, happy_3.ogg, ...
    let stem = path.trim_end_matches(".ogg");
    for i in 2..=9 {
        let variant_path = format!("{stem}_{i}.ogg");
        if std::path::Path::new(&format!("assets/{variant_path}")).exists() {
            bank.add(key, asset_server.load(&variant_path));
            count += 1;
        } else {
            break; // stop at first gap
        }
    }

    if count > 1 {
        info!("  {} → {} variants", path, count);
    }
    true
}

// ===================================================================
// Systems
// ===================================================================

/// Plays vocalization on mood change — only if the creature KNOWS that sound.
/// Pitch and volume are genome-driven (each individual sounds different).
fn sound_on_mood_change(
    mind: Res<Mind>,
    genome: Res<Genome>,
    growth: Res<GrowthState>,
    bank: Res<SoundBank>,
    repertoire_q: Query<&VocalRepertoire, With<CreatureRoot>>,
    mut commands: Commands,
) {
    if !mind.is_changed() { return; }
    let Ok(rep) = repertoire_q.single() else { return };

    // Can this creature vocalize for this mood?
    let Some(mood_sound) = rep.best_sound_for_mood(&mind.mood) else { return };

    let sk = SpeciesKey::from(&genome.species);
    let stk = StageKey::from(&growth.stage);

    // Try stage-specific, fall back to adult
    let key = SoundKey::Vocal(sk, stk, mood_sound);
    let fallback = SoundKey::Vocal(sk, StageKey::Adult, mood_sound);

    if let Some(handle) = bank.get(&key).or_else(|| bank.get(&fallback)) {
        // Use genome-driven pitch and volume
        commands.spawn((
            AudioPlayer::new(handle),
            rep.voice_settings(),
        ));
    }
}

/// Plays touch sound + boosts touch affinity for vocal learning.
fn sound_on_touch(
    mut events: EventReader<TouchEvent>,
    bank: Res<SoundBank>,
    mut repertoire_q: Query<&mut VocalRepertoire, With<CreatureRoot>>,
    mut commands: Commands,
) {
    for _event in events.read() {
        // Boost touch affinity
        if let Ok(mut rep) = repertoire_q.single_mut() {
            rep.mood_affinity.touch_score += 0.01;
        }

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

/// Expands vocal repertoire through interaction. The creature learns
/// different sounds depending on what it does most (play, eat, be petted).
fn vocal_learning_system(
    mind: Res<Mind>,
    mut repertoire_q: Query<&mut VocalRepertoire, With<CreatureRoot>>,
) {
    if mind.mood == MoodState::Sleeping { return; }
    let Ok(mut rep) = repertoire_q.single_mut() else { return };

    // Track mood affinity — what the creature experiences most
    match mind.mood {
        MoodState::Playful => rep.mood_affinity.play_score += 0.001,
        MoodState::Happy   => rep.mood_affinity.feed_score += 0.0005,
        _ => {}
    }

    let learn_amount = match mind.mood {
        MoodState::Playful => 0.002,  // learns fastest when playing
        MoodState::Happy   => 0.001,  // learns when content
        _                  => 0.0,
    };

    if learn_amount > 0.0 {
        if let Some(new_sound) = rep.advance_learning(learn_amount) {
            info!(
                "Creature learned: {:?}! ({}/{} sounds, pitch offset: {:.2})",
                new_sound, rep.total_sounds(), rep.capacity, rep.pitch_offset
            );
        }
    }
}
