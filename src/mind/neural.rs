//! Minimal MLP (Multi-Layer Perceptron) neural network in pure Rust.
//!
//! Architecture: **13 → 8 → 8**
//!
//! - **Input (13)**: 5 vital stats + 7 genome genes + 1 time-of-day
//! - **Hidden (8)**: ReLU activation
//! - **Output (8)**: One per mood state, softmax for probabilities
//!
//! Total parameters: (13×8 + 8) + (8×8 + 8) = 112 + 72 = **184 weights + biases**
//!
//! The network learns each Kobara's owner interaction patterns locally.
//! It does NOT replace the FSM — instead it *suggests* mood transitions
//! that the FSM can accept or veto.

use serde::{Deserialize, Serialize};
use rand::Rng;

/// Number of input neurons (5 stats + 7 genes + 1 time_of_day).
pub const INPUT_SIZE: usize = 13;
/// Number of hidden neurons.
pub const HIDDEN_SIZE: usize = 8;
/// Number of output neurons (one per MoodState).
pub const OUTPUT_SIZE: usize = 8;

/// A small feedforward neural network with one hidden layer.
///
/// All weights are stored as flat `Vec<f32>` for easy serialization
/// and cache-friendly access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLP {
    /// Weights from input → hidden (INPUT_SIZE × HIDDEN_SIZE, row-major).
    pub w1: Vec<f32>,
    /// Biases for hidden layer (HIDDEN_SIZE).
    pub b1: Vec<f32>,
    /// Weights from hidden → output (HIDDEN_SIZE × OUTPUT_SIZE, row-major).
    pub w2: Vec<f32>,
    /// Biases for output layer (OUTPUT_SIZE).
    pub b2: Vec<f32>,
}

impl MLP {
    /// Creates a new MLP with Xavier-initialized random weights.
    pub fn new() -> Self {
        let mut rng = rand::rng();

        // Xavier initialization: scale = sqrt(2 / (fan_in + fan_out))
        let scale1 = (2.0 / (INPUT_SIZE + HIDDEN_SIZE) as f32).sqrt();
        let scale2 = (2.0 / (HIDDEN_SIZE + OUTPUT_SIZE) as f32).sqrt();

        let w1: Vec<f32> = (0..INPUT_SIZE * HIDDEN_SIZE)
            .map(|_| rng.random_range(-scale1..scale1))
            .collect();
        let b1 = vec![0.0; HIDDEN_SIZE];

        let w2: Vec<f32> = (0..HIDDEN_SIZE * OUTPUT_SIZE)
            .map(|_| rng.random_range(-scale2..scale2))
            .collect();
        let b2 = vec![0.0; OUTPUT_SIZE];

        Self { w1, b1, w2, b2 }
    }

    /// Forward pass: input → hidden (ReLU) → output (softmax).
    ///
    /// Returns mood probabilities as a `[f32; OUTPUT_SIZE]` array summing to 1.0.
    pub fn forward(&self, input: &[f32; INPUT_SIZE]) -> [f32; OUTPUT_SIZE] {
        // Hidden layer: z1 = W1 * x + b1, h = ReLU(z1)
        let mut hidden = [0.0f32; HIDDEN_SIZE];
        for j in 0..HIDDEN_SIZE {
            let mut sum = self.b1[j];
            for i in 0..INPUT_SIZE {
                sum += input[i] * self.w1[i * HIDDEN_SIZE + j];
            }
            hidden[j] = sum.max(0.0); // ReLU
        }

        // Output layer: z2 = W2 * h + b2
        let mut logits = [0.0f32; OUTPUT_SIZE];
        for k in 0..OUTPUT_SIZE {
            let mut sum = self.b2[k];
            for j in 0..HIDDEN_SIZE {
                sum += hidden[j] * self.w2[j * OUTPUT_SIZE + k];
            }
            logits[k] = sum;
        }

        // Softmax
        softmax(&logits)
    }

    /// Forward pass that also returns the hidden activations (needed for backprop).
    fn forward_with_hidden(&self, input: &[f32; INPUT_SIZE]) -> ([f32; HIDDEN_SIZE], [f32; OUTPUT_SIZE]) {
        let mut hidden = [0.0f32; HIDDEN_SIZE];
        for j in 0..HIDDEN_SIZE {
            let mut sum = self.b1[j];
            for i in 0..INPUT_SIZE {
                sum += input[i] * self.w1[i * HIDDEN_SIZE + j];
            }
            hidden[j] = sum.max(0.0);
        }

        let mut logits = [0.0f32; OUTPUT_SIZE];
        for k in 0..OUTPUT_SIZE {
            let mut sum = self.b2[k];
            for j in 0..HIDDEN_SIZE {
                sum += hidden[j] * self.w2[j * OUTPUT_SIZE + k];
            }
            logits[k] = sum;
        }

        (hidden, softmax(&logits))
    }

    /// Single-sample backpropagation with cross-entropy loss.
    ///
    /// `target` is a one-hot encoded label (index of the correct mood).
    /// `lr` is the learning rate.
    ///
    /// Returns the loss for monitoring.
    pub fn train_step(&mut self, input: &[f32; INPUT_SIZE], target: usize, lr: f32) -> f32 {
        let (hidden, output) = self.forward_with_hidden(input);

        // Cross-entropy loss: -log(output[target])
        let loss = -output[target].max(1e-7).ln();

        // Output gradient: dL/d(logit_k) = output_k - target_k  (softmax + cross-entropy)
        let mut d_logits = [0.0f32; OUTPUT_SIZE];
        for k in 0..OUTPUT_SIZE {
            d_logits[k] = output[k] - if k == target { 1.0 } else { 0.0 };
        }

        // Gradient for w2, b2
        for j in 0..HIDDEN_SIZE {
            for k in 0..OUTPUT_SIZE {
                self.w2[j * OUTPUT_SIZE + k] -= lr * hidden[j] * d_logits[k];
            }
        }
        for k in 0..OUTPUT_SIZE {
            self.b2[k] -= lr * d_logits[k];
        }

        // Backprop through hidden layer
        let mut d_hidden = [0.0f32; HIDDEN_SIZE];
        for j in 0..HIDDEN_SIZE {
            let mut sum = 0.0;
            for k in 0..OUTPUT_SIZE {
                sum += self.w2[j * OUTPUT_SIZE + k] * d_logits[k];
            }
            // ReLU derivative: 0 if hidden was 0, else 1
            d_hidden[j] = if hidden[j] > 0.0 { sum } else { 0.0 };
        }

        // Gradient for w1, b1
        for i in 0..INPUT_SIZE {
            for j in 0..HIDDEN_SIZE {
                self.w1[i * HIDDEN_SIZE + j] -= lr * input[i] * d_hidden[j];
            }
        }
        for j in 0..HIDDEN_SIZE {
            self.b1[j] -= lr * d_hidden[j];
        }

        loss
    }

    /// Returns the mood index with the highest probability.
    #[allow(dead_code)]
    pub fn predict(&self, input: &[f32; INPUT_SIZE]) -> usize {
        let probs = self.forward(input);
        probs.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    /// Serializes weights to a byte vector (for SQLite storage).
    pub fn to_bytes(&self) -> Vec<u8> {
        let config = bincode::config::standard();
        bincode::serde::encode_to_vec(self, config)
            .expect("MLP serialization failed")
    }

    /// Deserializes weights from a byte slice. Validates that the loaded
    /// dimensions match the current INPUT_SIZE/HIDDEN_SIZE/OUTPUT_SIZE; if a
    /// previous version stored weights with different sizes (e.g. before a new
    /// input feature was added), returns `None` so the caller can spawn a
    /// fresh network instead of crashing on out-of-bounds indexing.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        let config = bincode::config::standard();
        let (mlp, _): (MLP, _) = bincode::serde::decode_from_slice(data, config).ok()?;
        if mlp.w1.len() != INPUT_SIZE * HIDDEN_SIZE
            || mlp.b1.len() != HIDDEN_SIZE
            || mlp.w2.len() != HIDDEN_SIZE * OUTPUT_SIZE
            || mlp.b2.len() != OUTPUT_SIZE
        {
            return None; // dimension mismatch — stored network is stale
        }
        Some(mlp)
    }
}

/// Numerically stable softmax.
fn softmax(logits: &[f32; OUTPUT_SIZE]) -> [f32; OUTPUT_SIZE] {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut exp = [0.0f32; OUTPUT_SIZE];
    let mut sum = 0.0f32;
    for i in 0..OUTPUT_SIZE {
        exp[i] = (logits[i] - max).exp();
        sum += exp[i];
    }
    for i in 0..OUTPUT_SIZE {
        exp[i] /= sum;
    }
    exp
}

/// Maps a `MoodState` to its neural network output index.
pub fn mood_to_index(mood: &crate::mind::MoodState) -> usize {
    use crate::mind::MoodState;
    match mood {
        MoodState::Happy    => 0,
        MoodState::Hungry   => 1,
        MoodState::Tired    => 2,
        MoodState::Lonely   => 3,
        MoodState::Playful  => 4,
        MoodState::Sick     => 5,
        MoodState::Sleeping => 6,
        MoodState::Thirsty  => 7,
    }
}

/// Maps a neural network output index back to a `MoodState`.
pub fn index_to_mood(idx: usize) -> crate::mind::MoodState {
    use crate::mind::MoodState;
    match idx {
        0 => MoodState::Happy,
        1 => MoodState::Hungry,
        2 => MoodState::Tired,
        3 => MoodState::Lonely,
        4 => MoodState::Playful,
        5 => MoodState::Sick,
        6 => MoodState::Sleeping,
        7 => MoodState::Thirsty,
        _ => MoodState::Happy,
    }
}

/// Builds the 12-element input vector from current game state.
///
/// All values are normalized to roughly [0, 1]:
/// - hunger, thirst, happiness, energy, health: divided by 100
/// - genome genes: already [0, 1] (except hue which is /360)
/// - time_of_day: hour / 24
pub fn build_input(
    stats: &crate::mind::VitalStats,
    genome: &crate::genome::Genome,
    hour: f32,
) -> [f32; INPUT_SIZE] {
    [
        // Vital stats (5)
        stats.hunger / 100.0,
        stats.thirst / 100.0,
        stats.happiness / 100.0,
        stats.energy / 100.0,
        stats.health / 100.0,
        // Genome genes (7)
        genome.curiosity,
        genome.loneliness_sensitivity,
        genome.appetite,
        genome.circadian,
        genome.resilience,
        genome.learning_rate,
        genome.hue / 360.0,
        // Time context (1)
        hour / 24.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_produces_valid_probabilities() {
        let mlp = MLP::new();
        let input = [0.5; INPUT_SIZE];
        let probs = mlp.forward(&input);

        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5, "Softmax should sum to 1.0, got {sum}");
        assert!(probs.iter().all(|&p| p >= 0.0), "All probabilities should be >= 0");
    }

    #[test]
    fn train_step_reduces_loss() {
        let mut mlp = MLP::new();
        let input = [0.3, 0.7, 0.5, 0.9, 0.4, 0.6, 0.2, 0.8, 0.5, 0.3, 0.5, 0.5, 0.4];
        let target = 2; // Tired

        let loss1 = mlp.train_step(&input, target, 0.01);
        // Run several more steps
        let mut loss = loss1;
        for _ in 0..50 {
            loss = mlp.train_step(&input, target, 0.01);
        }
        assert!(loss < loss1, "Loss should decrease after training: {loss1} → {loss}");
    }

    #[test]
    fn serialization_roundtrip() {
        let mlp = MLP::new();
        let bytes = mlp.to_bytes();
        let restored = MLP::from_bytes(&bytes).expect("Deserialization failed");
        assert_eq!(mlp.w1.len(), restored.w1.len());
        assert_eq!(mlp.w2.len(), restored.w2.len());
    }
}
