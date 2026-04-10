//! UI sound synthesis — button clicks, menu open/close.
//!
//! Short, crisp chiptune sounds that provide tactile feedback.

use super::synth;

/// Button click: sharp square pulse at 1000 Hz.
pub fn generate_click() -> Vec<f32> {
    let mut s = synth::square(1000.0, 0.03, 0.5);
    synth::apply_envelope(&mut s, 0.002, 0.01, 0.5, 0.01);
    s
}

/// Menu open: ascending two-tone (600 Hz → 900 Hz).
pub fn generate_menu_open() -> Vec<f32> {
    let mut low = synth::square(600.0, 0.04, 0.4);
    synth::apply_envelope(&mut low, 0.002, 0.01, 0.6, 0.01);

    let mut high = synth::square(900.0, 0.04, 0.4);
    synth::apply_envelope(&mut high, 0.002, 0.01, 0.6, 0.01);

    let mut result = low;
    result.extend(high);
    result
}

/// Menu close: descending two-tone (900 Hz → 600 Hz).
pub fn generate_menu_close() -> Vec<f32> {
    let mut high = synth::square(900.0, 0.04, 0.4);
    synth::apply_envelope(&mut high, 0.002, 0.01, 0.6, 0.01);

    let mut low = synth::square(600.0, 0.04, 0.4);
    synth::apply_envelope(&mut low, 0.002, 0.01, 0.6, 0.01);

    let mut result = high;
    result.extend(low);
    result
}
