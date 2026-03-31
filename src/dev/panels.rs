//! egui data panels — stats, genome, and neural network info.
//!
//! Renders a right-side panel with collapsible sections for each data
//! category. All data is read-only — the panels observe game state
//! without modifying it.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::genome::Genome;
use crate::mind::Mind;
use crate::mind::neural::{OUTPUT_SIZE, build_input, index_to_mood};
use crate::mind::plugin::NeuralMind;

use super::DevModeState;

pub fn dev_panels_system(
    mut contexts: EguiContexts,
    mut dev_state: ResMut<DevModeState>,
    mind: Option<Res<Mind>>,
    genome: Option<Res<Genome>>,
    neural: Option<Res<NeuralMind>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::SidePanel::right("dev_panel")
        .default_width(210.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Dev Mode (F12)");
            ui.separator();

            // --- Master toggles ---
            ui.checkbox(&mut dev_state.show_rig, "Rig Overlay");
            ui.checkbox(&mut dev_state.show_stats, "Stats Panel");
            ui.checkbox(&mut dev_state.show_genome, "Genome Panel");
            ui.checkbox(&mut dev_state.show_neural, "Neural Panel");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if dev_state.show_stats {
                    draw_stats_panel(ui, mind.as_deref(), genome.as_deref());
                }
                if dev_state.show_genome {
                    draw_genome_panel(ui, genome.as_deref());
                }
                if dev_state.show_neural {
                    draw_neural_panel(ui, neural.as_deref(), mind.as_deref(), genome.as_deref());
                }
            });
        });
}

// ---------------------------------------------------------------------------
// Stats panel
// ---------------------------------------------------------------------------

fn draw_stats_panel(ui: &mut egui::Ui, mind: Option<&Mind>, genome: Option<&Genome>) {
    egui::CollapsingHeader::new("Stats")
        .default_open(true)
        .show(ui, |ui| {
            let Some(mind) = mind else {
                ui.label("No Mind resource");
                return;
            };

            let mood_color = mood_to_egui_color(&mind.mood);
            ui.horizontal(|ui| {
                ui.label("Mood:");
                ui.colored_label(mood_color, mind.mood.label());
            });

            // FSM vs actual comparison
            if let Some(genome) = genome {
                let fsm = mind.fsm_mood(genome);
                if fsm != mind.mood {
                    ui.horizontal(|ui| {
                        ui.label("FSM says:");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            fsm.label(),
                        );
                        ui.label("(NN override)");
                    });
                }
            }

            ui.add_space(4.0);

            stat_bar(ui, "Hunger", mind.stats.hunger, egui::Color32::from_rgb(230, 140, 40));
            stat_bar(ui, "Happiness", mind.stats.happiness, egui::Color32::from_rgb(80, 200, 80));
            stat_bar(ui, "Energy", mind.stats.energy, egui::Color32::from_rgb(80, 140, 230));
            stat_bar(ui, "Health", mind.stats.health, egui::Color32::from_rgb(220, 60, 60));

            ui.add_space(4.0);
            ui.label(format!("Age: {} ticks", mind.age_ticks));
        });

    ui.separator();
}

fn stat_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let bar = egui::ProgressBar::new(value / 100.0)
            .text(format!("{value:.1}"))
            .fill(color);
        ui.add(bar);
    });
}

fn mood_to_egui_color(mood: &crate::mind::MoodState) -> egui::Color32 {
    use crate::mind::MoodState;
    match mood {
        MoodState::Happy    => egui::Color32::from_rgb(100, 220, 100),
        MoodState::Hungry   => egui::Color32::from_rgb(230, 160, 50),
        MoodState::Tired    => egui::Color32::from_rgb(140, 140, 180),
        MoodState::Lonely   => egui::Color32::from_rgb(120, 120, 200),
        MoodState::Playful  => egui::Color32::from_rgb(240, 200, 60),
        MoodState::Sick     => egui::Color32::from_rgb(200, 80, 80),
        MoodState::Sleeping => egui::Color32::from_rgb(100, 100, 160),
    }
}

// ---------------------------------------------------------------------------
// Genome panel
// ---------------------------------------------------------------------------

fn draw_genome_panel(ui: &mut egui::Ui, genome: Option<&Genome>) {
    egui::CollapsingHeader::new("Genome")
        .default_open(true)
        .show(ui, |ui| {
            let Some(genome) = genome else {
                ui.label("No Genome resource");
                return;
            };

            ui.label(format!("Species: {:?}", genome.species));
            ui.add_space(4.0);

            gene_bar(ui, "Curiosity", genome.curiosity);
            gene_bar(ui, "Loneliness Sens.", genome.loneliness_sensitivity);
            gene_bar(ui, "Appetite", genome.appetite);
            gene_bar(ui, "Circadian", genome.circadian);
            gene_bar(ui, "Resilience", genome.resilience);
            gene_bar(ui, "Learning Rate", genome.learning_rate);

            ui.add_space(4.0);

            // Hue with color swatch
            ui.horizontal(|ui| {
                ui.label(format!("Hue: {:.0}", genome.hue));
                let (r, g, b) = hsl_to_rgb(genome.hue, 0.7, 0.75);
                let color = egui::Color32::from_rgb(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(20.0, 14.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 3.0, color);
            });
        });

    ui.separator();
}

fn gene_bar(ui: &mut egui::Ui, label: &str, value: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let bar = egui::ProgressBar::new(value)
            .text(format!("{value:.2}"));
        ui.add(bar);
    });
}

/// Simple HSL to RGB conversion for the hue swatch.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let h = h / 360.0;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

// ---------------------------------------------------------------------------
// Neural network panel
// ---------------------------------------------------------------------------

fn draw_neural_panel(
    ui: &mut egui::Ui,
    neural: Option<&NeuralMind>,
    mind: Option<&Mind>,
    genome: Option<&Genome>,
) {
    egui::CollapsingHeader::new("Neural Network")
        .default_open(true)
        .show(ui, |ui| {
            let Some(neural) = neural else {
                ui.label("No NeuralMind resource");
                return;
            };

            let influence = neural.influence();

            ui.horizontal(|ui| {
                ui.label("Influence:");
                let bar = egui::ProgressBar::new(influence / 0.6)
                    .text(format!("{:.0}%", influence * 100.0));
                ui.add(bar);
            });

            ui.label(format!(
                "Mature: {}",
                if neural.mature { "yes" } else { "no" }
            ));
            ui.label(format!("Sessions: {}", neural.sessions_completed));

            if neural.last_loss < f32::MAX {
                ui.label(format!("Last loss: {:.4}", neural.last_loss));
            } else {
                ui.label("Last loss: --");
            }

            ui.label("Arch: 12 -> 8 -> 7 (167 params)");

            // Live prediction
            if let (Some(mind), Some(genome)) = (mind, genome) {
                ui.add_space(4.0);
                ui.label("Live prediction:");

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let hour = ((now % 86400) / 3600) as f32;

                let input = build_input(&mind.stats, genome, hour);
                let output = neural.mlp.forward(&input);

                for i in 0..OUTPUT_SIZE {
                    let mood = index_to_mood(i);
                    let conf = output[i];
                    let is_current = mood == mind.mood;

                    ui.horizontal(|ui| {
                        let label_text = if is_current {
                            format!("> {}", mood.label())
                        } else {
                            format!("  {}", mood.label())
                        };
                        ui.label(label_text);
                        let bar = egui::ProgressBar::new(conf)
                            .text(format!("{:.0}%", conf * 100.0));
                        ui.add(bar);
                    });
                }
            }
        });
}
