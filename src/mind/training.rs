//! Training pipeline for the creature's neural network.
//!
//! Reads interaction history from the SQLite `event_log` table, converts
//! each event into a (input, target_mood) training sample, and runs
//! mini-batch gradient descent on the MLP.
//!
//! ## Training schedule
//!
//! Training runs periodically (every N ticks) as a Bevy system, not
//! continuously. Each session does a few epochs over recent events so
//! the creature gradually adapts without burning CPU.
//!
//! ## How events become training samples
//!
//! Each event in `event_log` has a `payload` JSON containing the snapshot
//! of vital stats + mood at the time of the event. We pair:
//! - **Input**: vital stats + genome + time_of_day at event time
//! - **Target**: the mood that resulted from the player's action
//!
//! This teaches the network: "when my owner feeds me at night with
//! these stats, I tend to become Happy".

use rusqlite::Connection;
use crate::genome::Genome;
use super::neural::{MLP, INPUT_SIZE, build_input};

/// A single training sample extracted from the event log.
#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub input: [f32; INPUT_SIZE],
    pub target: usize, // mood index
}

/// Extracts training samples from the event_log table.
///
/// Reads the most recent `limit` events that have payload data,
/// parses the JSON payload to reconstruct the game state at that moment,
/// and builds input/target pairs.
pub fn extract_samples(conn: &Connection, genome: &Genome, limit: usize) -> Vec<TrainingSample> {
    let mut stmt = conn.prepare(
        "SELECT payload, created_at FROM event_log
         WHERE payload IS NOT NULL
         ORDER BY id DESC
         LIMIT ?1"
    ).unwrap_or_else(|_| return conn.prepare("SELECT 1 WHERE 0").unwrap());

    let mut samples = Vec::new();

    let rows = stmt.query_map([limit as i64], |row| {
        let payload: String = row.get(0)?;
        let created_at: i64 = row.get(1)?;
        Ok((payload, created_at))
    });

    let rows = match rows {
        Ok(r) => r,
        Err(_) => return samples,
    };

    for row in rows.flatten() {
        let (payload, created_at) = row;
        if let Some(sample) = parse_payload(&payload, genome, created_at) {
            samples.push(sample);
        }
    }

    samples
}

/// Parses a JSON payload from the event log into a training sample.
///
/// Expected payload format (written by `log_event_with_state`):
/// ```json
/// {
///   "hunger": 45.0,
///   "happiness": 70.0,
///   "energy": 60.0,
///   "health": 100.0,
///   "mood": "Happy",
///   "action": "fed"
/// }
/// ```
fn parse_payload(payload: &str, genome: &Genome, created_at: i64) -> Option<TrainingSample> {
    // Simple JSON parsing without serde_json dependency
    let hunger = extract_f32(payload, "hunger")?;
    let happiness = extract_f32(payload, "happiness")?;
    let energy = extract_f32(payload, "energy")?;
    let health = extract_f32(payload, "health")?;
    let mood_str = extract_str(payload, "mood")?;

    // Approximate hour from Unix timestamp
    let hour = ((created_at % 86400) as f32 / 3600.0).clamp(0.0, 23.99);

    let stats = crate::mind::VitalStats { hunger, happiness, energy, health };
    let input = build_input(&stats, genome, hour);
    let target = mood_str_to_index(&mood_str)?;

    Some(TrainingSample { input, target })
}

/// Trains the MLP on extracted samples for the given number of epochs.
///
/// Returns the average loss of the last epoch (for monitoring).
pub fn train_on_samples(mlp: &mut MLP, samples: &[TrainingSample], epochs: usize, lr: f32) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let mut last_avg_loss = 0.0;

    for _epoch in 0..epochs {
        let mut total_loss = 0.0;
        for sample in samples {
            total_loss += mlp.train_step(&sample.input, sample.target, lr);
        }
        last_avg_loss = total_loss / samples.len() as f32;
    }

    last_avg_loss
}

/// Builds a JSON payload string capturing the current game state.
///
/// This is stored in event_log.payload so future training can
/// reconstruct the exact conditions at the time of the event.
pub fn build_event_payload(
    stats: &crate::mind::VitalStats,
    mood: &crate::mind::MoodState,
    action: &str,
) -> String {
    format!(
        r#"{{"hunger":{:.1},"happiness":{:.1},"energy":{:.1},"health":{:.1},"mood":"{}","action":"{}"}}"#,
        stats.hunger, stats.happiness, stats.energy, stats.health,
        mood.label(), action
    )
}

// ---------------------------------------------------------------------------
// Simple JSON helpers (avoids serde_json dependency)
// ---------------------------------------------------------------------------

fn extract_f32(json: &str, key: &str) -> Option<f32> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c == ' ')?;
    rest[..end].parse().ok()
}

fn extract_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')?;
    Some(json[start..start + end].to_string())
}

fn mood_str_to_index(s: &str) -> Option<usize> {
    match s {
        "Happy"    => Some(0),
        "Hungry"   => Some(1),
        "Tired"    => Some(2),
        "Lonely"   => Some(3),
        "Playful"  => Some(4),
        "Sick"     => Some(5),
        "Sleeping" => Some(6),
        _          => None,
    }
}
