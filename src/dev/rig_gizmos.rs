//! Rig skeleton visualization using Bevy Gizmos.
//!
//! TODO: ResolvedRig resource was refactored out. This module needs
//! updating to use the new spawn pipeline. Stubbed for now.

use bevy::prelude::*;
use super::DevModeState;

pub fn draw_rig_gizmos(
    dev_state: Res<DevModeState>,
) {
    if !dev_state.show_rig {
        return;
    }
    // Rig gizmos temporarily disabled — awaiting spawn pipeline update
}
