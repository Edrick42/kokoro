//! Neural Mind Bevy plugin — integrates the MLP into the game loop.
//!
//! ## What it does
//!
//! 1. **On startup**: loads saved neural weights from SQLite (or creates a new MLP).
//! 2. **Every tick**: after the FSM sets a mood, the neural network can nudge
//!    non-critical moods based on learned owner patterns.
//! 3. **Every TRAIN_INTERVAL ticks**: trains the MLP on recent event_log data.
//! 4. **On autosave**: persists weights to SQLite.
//!
//! ## FSM veto
//!
//! Critical states (Sick, Sleeping) are never overridden by the network.
//! For non-critical states, the network's suggestion is blended with the FSM:
//! - If the NN is confident (>0.5) about a different mood AND the FSM is not
//!   in a critical state, the NN suggestion wins.
//! - Otherwise the FSM decision stands.
//! - The blend weight (`nn_influence`) grows with training maturity.

use bevy::prelude::*;
use crate::game::state::AppState;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::mind::neural::{MLP, build_input, index_to_mood, mood_to_index};
use crate::mind::training::{extract_samples, train_on_samples};
use crate::persistence::plugin::DbConnection;

/// How many ticks between neural network training sessions.
const TRAIN_INTERVAL: u64 = 120;

/// Minimum number of events needed before the NN starts influencing mood.
const MIN_TRAINING_EVENTS: usize = 20;

/// Maximum number of recent events to train on per session.
const TRAIN_SAMPLE_LIMIT: usize = 200;

/// Number of epochs per training session.
const TRAIN_EPOCHS: usize = 5;

/// The creature's neural network, stored as a Bevy resource.
#[derive(Resource)]
pub struct NeuralMind {
    pub mlp: MLP,
    /// How many training sessions have completed.
    pub sessions_completed: u32,
    /// Average loss from the last training session.
    pub last_loss: f32,
    /// Whether the network has enough data to influence mood.
    pub mature: bool,
}

impl NeuralMind {
    pub fn new() -> Self {
        Self {
            mlp: MLP::new(),
            sessions_completed: 0,
            last_loss: f32::MAX,
            mature: false,
        }
    }

    pub fn from_mlp(mlp: MLP) -> Self {
        Self {
            mlp,
            sessions_completed: 0,
            last_loss: f32::MAX,
            mature: false,
        }
    }

    /// How much influence the NN has (0.0 = none, 1.0 = full).
    /// Grows logarithmically with training sessions.
    pub fn influence(&self) -> f32 {
        if !self.mature {
            return 0.0;
        }
        // Caps at ~0.6 after ~50 sessions (never fully replaces FSM)
        (self.sessions_completed as f32 / 50.0).min(1.0) * 0.6
    }
}

pub struct NeuralMindPlugin;

impl Plugin for NeuralMindPlugin {
    fn build(&self, app: &mut App) {
        // PersistencePlugin runs in PreStartup, so DbConnection is available by Startup
        app.add_systems(Startup, load_neural_weights)
           .add_systems(Update, (neural_mood_system, neural_train_system).run_if(in_state(AppState::Gameplay)));
    }
}

/// Loads neural weights from SQLite at startup.
fn load_neural_weights(
    mut commands: Commands,
    db: Res<DbConnection>,
) {
    let conn = db.0.lock().expect("DB lock poisoned");

    // Ensure the neural_weights table exists
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS neural_weights (
            id    INTEGER PRIMARY KEY CHECK (id = 1),
            data  BLOB NOT NULL
        );"
    );

    let neural_mind = match conn.prepare("SELECT data FROM neural_weights WHERE id = 1") {
        Ok(mut stmt) => {
            match stmt.query_row([], |row| {
                let data: Vec<u8> = row.get(0)?;
                Ok(data)
            }) {
                Ok(data) => {
                    if let Some(mlp) = MLP::from_bytes(&data) {
                        info!("Neural weights loaded from database");
                        NeuralMind::from_mlp(mlp)
                    } else {
                        info!("Failed to deserialize neural weights — starting fresh");
                        NeuralMind::new()
                    }
                }
                Err(_) => {
                    info!("No saved neural weights — initializing new MLP");
                    NeuralMind::new()
                }
            }
        }
        Err(_) => NeuralMind::new(),
    };

    commands.insert_resource(neural_mind);
}

/// After the FSM sets a mood each tick, the neural network can suggest
/// a different mood for non-critical states.
fn neural_mood_system(
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    neural: Res<NeuralMind>,
) {
    let influence = neural.influence();
    if influence < 0.01 {
        return; // Network not mature enough
    }

    // Don't override critical states
    if mind.mood.is_critical() {
        return;
    }

    // Get current hour approximation
    let hour = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ((secs % 86400) as f32 / 3600.0).clamp(0.0, 23.99)
    };

    let input = build_input(&mind.stats, &genome, hour);
    let probs = neural.mlp.forward(&input);

    // Find the NN's top suggestion
    let (nn_mood_idx, nn_confidence) = probs.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    let nn_mood = index_to_mood(nn_mood_idx);

    // Only override if NN is confident and suggests something different
    let fsm_idx = mood_to_index(&mind.mood);
    if nn_mood_idx != fsm_idx && *nn_confidence > 0.4 && !nn_mood.is_critical() {
        // Probabilistic blend: higher influence = more likely to accept NN suggestion
        use rand::Rng;
        let mut rng = rand::rng();
        if rng.random_range(0.0f32..1.0) < influence * nn_confidence {
            debug!(
                "Neural network overrides FSM: {} → {} (confidence: {:.2}, influence: {:.2})",
                mind.mood.label(), nn_mood.label(), nn_confidence, influence
            );
            mind.mood = nn_mood;
        }
    }
}

/// Periodically trains the MLP on recent interaction history.
fn neural_train_system(
    mind: Res<Mind>,
    genome: Res<Genome>,
    db: Res<DbConnection>,
    mut neural: ResMut<NeuralMind>,
) {
    if mind.age_ticks == 0 || mind.age_ticks % TRAIN_INTERVAL != 0 {
        return;
    }

    let conn = db.0.lock().expect("DB lock poisoned");

    let samples = extract_samples(&conn, &genome, TRAIN_SAMPLE_LIMIT);

    if samples.len() < MIN_TRAINING_EVENTS {
        debug!(
            "Not enough training data yet ({}/{} events)",
            samples.len(), MIN_TRAINING_EVENTS
        );
        return;
    }

    let lr = genome.learning_rate * 0.01 + 0.005; // 0.005–0.015 range
    let avg_loss = train_on_samples(&mut neural.mlp, &samples, TRAIN_EPOCHS, lr);

    neural.sessions_completed += 1;
    neural.last_loss = avg_loss;
    neural.mature = true;

    info!(
        "Neural training session #{}: {} samples, avg loss: {:.4}, lr: {:.4}",
        neural.sessions_completed, samples.len(), avg_loss, lr
    );

    // Save weights after training
    let bytes = neural.mlp.to_bytes();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO neural_weights (id, data) VALUES (1, ?1)",
        rusqlite::params![bytes],
    );
}
