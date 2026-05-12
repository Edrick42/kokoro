//! Nerves — the signal pathway from mind to muscle.
//!
//! A nerve carries a 0..1 activation intent from the mind to a specific
//! muscle. Real nerves have latency (signal travels at finite speed),
//! attenuation (some signal loss), and health (damaged or aged nerves
//! deliver weaker signals).
//!
//! Today nerves use a [`NervePathway::Direct`] model: a fixed latency
//! buffer plus a multiplicative `attenuation` factor. The enum holds a
//! slot for a future `Network(...)` variant that would simulate
//! individual neurons.

/// How a nerve delivers signals end-to-end.
#[derive(Clone, Debug, PartialEq)]
pub enum NervePathway {
    /// Lumped model: signals pass through a fixed-latency ring buffer
    /// and are multiplied by `attenuation` ∈ 0..1 on the way out.
    Direct {
        /// Conduction latency in milliseconds.
        latency_ms: f32,
        /// 1.0 = perfect transmission, 0.0 = total silence. Damaged or
        /// aged nerves drift toward 0.
        attenuation: f32,
    },
    // future:
    // Network { neurons: Vec<Neuron>, ... },
}

/// A signal pathway from mind to muscle.
#[derive(Clone, Debug)]
pub struct Nerve {
    pub name: &'static str,
    pub pathway: NervePathway,

    /// Health in 0..=1. Affects the effective attenuation. The genome
    /// initialises this near 1.0; injury / age subtract from it.
    pub health: f32,

    /// Most recently issued intents, oldest → newest. Used to implement
    /// latency: at time t the muscle reads from `buffer[t − latency]`.
    /// The buffer length is sized to `latency_ms / dt` at the first
    /// `deliver` call.
    buffer: Vec<f32>,
}

impl Nerve {
    pub fn new(name: &'static str, latency_ms: f32, attenuation: f32) -> Self {
        Self {
            name,
            pathway: NervePathway::Direct {
                latency_ms,
                attenuation,
            },
            health: 1.0,
            buffer: Vec::new(),
        }
    }

    /// Push an intent (0..1) into the nerve's input end, then pop the
    /// signal that arrived at the muscle end this frame. Latency is
    /// quantised to integer multiples of `dt`.
    pub fn deliver(&mut self, intent: f32, dt: f32) -> f32 {
        let intent = intent.clamp(0.0, 1.0);
        match self.pathway {
            NervePathway::Direct {
                latency_ms,
                attenuation,
            } => {
                let latency_s = latency_ms / 1000.0;
                let slots = ((latency_s / dt.max(1e-6)).round() as usize).max(1);
                if self.buffer.len() != slots {
                    self.buffer.resize(slots, 0.0);
                }
                // Shift: front = oldest (delivered now), back = newest.
                let delivered = self.buffer.remove(0);
                self.buffer.push(intent);
                let gain = attenuation.clamp(0.0, 1.0) * self.health.clamp(0.0, 1.0);
                (delivered * gain).clamp(0.0, 1.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_nerve_delivers_zero_until_buffer_fills() {
        let mut n = Nerve::new("test", 50.0, 1.0);
        let dt = 0.01; // 10ms × 5 slots = 50ms latency
        // First 5 calls deliver zeros while the buffer fills up.
        for _ in 0..5 {
            assert_eq!(n.deliver(1.0, dt), 0.0);
        }
        // On the 6th call the first pushed intent emerges.
        let out = n.deliver(1.0, dt);
        assert!((out - 1.0).abs() < 1e-6);
    }

    #[test]
    fn attenuation_scales_output() {
        let mut n = Nerve::new("test", 10.0, 0.5);
        let dt = 0.01; // 1 slot, signal needs one full tick to traverse
        let _ = n.deliver(1.0, dt); // queue intent
        let out = n.deliver(0.0, dt); // it emerges, scaled
        assert!((out - 0.5).abs() < 1e-6);
    }

    #[test]
    fn health_further_scales_output() {
        let mut n = Nerve::new("test", 10.0, 1.0);
        n.health = 0.4;
        let dt = 0.01;
        let _ = n.deliver(1.0, dt);
        let out = n.deliver(0.0, dt);
        assert!((out - 0.4).abs() < 1e-6);
    }

    #[test]
    fn output_is_clamped_to_unit_range() {
        let mut n = Nerve::new("test", 10.0, 2.0); // illegally high attenuation
        let dt = 0.01;
        let out = n.deliver(1.0, dt);
        assert!(out <= 1.0);
    }
}
