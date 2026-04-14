//! egui data panels — stats, genome, and neural network info.
//!
//! Renders a right-side panel with collapsible sections for each data
//! category. All data is read-only — the panels observe game state
//! without modifying it.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::creature::lifecycle::collection::CreatureCollection;
use crate::creature::interaction::physics::PhysicsBody;
use crate::creature::identity::species::CreatureRoot;
use crate::genome::Genome;
use crate::mind::{Mind, MoodState};
use crate::visuals::evolution::{GrowthState, GrowthStage};
use crate::creature::anatomy::AnatomyState;
use crate::creature::behavior::pose::PoseState;
use crate::creature::behavior::reactions::ExpressionOverride;
use crate::mind::hygiene::HygieneState;
use crate::world::environment::EnvironmentState;
use crate::mind::autonomic::AutonomicState as ANS;
use crate::creature::behavior::involuntary::InvoluntaryState;
use crate::mind::neural::{OUTPUT_SIZE, build_input, index_to_mood};
use crate::mind::plugin::NeuralMind;
use crate::visuals::breathing::{BreathingState, HeartbeatState};
use crate::visuals::resonance_glow::ResonanceGlow;

use super::DevModeState;

pub fn dev_panels_system(
    mut contexts: EguiContexts,
    mut dev_state: ResMut<DevModeState>,
    mut mind: Option<ResMut<Mind>>,
    genome: Option<Res<Genome>>,
    neural: Option<Res<NeuralMind>>,
    mut collection: Option<ResMut<CreatureCollection>>,
    mut growth: Option<ResMut<GrowthState>>,
    pose: Option<Res<PoseState>>,
    expression: Option<Res<ExpressionOverride>>,
    mut reaction_events: EventWriter<crate::creature::behavior::reactions::CreatureReaction>,
    anatomy: Option<Res<AnatomyState>>,
    hygiene: Option<Res<HygieneState>>,
    environment: Option<Res<EnvironmentState>>,
    ans: Option<Res<ANS>>,
    inv: Option<Res<InvoluntaryState>>,
    physics_q: Query<(&PhysicsBody, &BreathingState, &HeartbeatState, &ResonanceGlow), With<CreatureRoot>>,
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
            ui.checkbox(&mut dev_state.show_physics, "Physics Panel");
            ui.checkbox(&mut dev_state.show_cheats, "Cheats");
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
                if dev_state.show_physics {
                    draw_physics_panel(ui, &physics_q);
                }
                // Autonomic Nervous System
                if let Some(ref ans) = ans {
                    ui.collapsing("ANS", |ui| {
                        let label = if ans.level > 0.6 { "SYMPATHETIC" }
                            else if ans.level < 0.3 { "PARASYMPATHETIC" }
                            else { "neutral" };
                        let color = if ans.level > 0.6 { egui::Color32::from_rgb(220, 80, 80) }
                            else if ans.level < 0.3 { egui::Color32::from_rgb(80, 160, 220) }
                            else { egui::Color32::from_rgb(180, 180, 100) };
                        ui.colored_label(color, format!("{} ({:.0}%)", label, ans.level * 100.0));
                        ui.label(format!("Target: {:.0}%", ans.target * 100.0));
                        if ans.conflict > 0.3 {
                            ui.colored_label(egui::Color32::YELLOW, format!("Conflict: {:.0}%", ans.conflict * 100.0));
                        }
                    });

                    // Involuntary state (shown under ANS)
                    if let Some(ref inv) = inv {
                        ui.label(format!("Pupil: {:.0}%", inv.pupil * 100.0));
                        ui.label(format!("Tension: {:.0}%", inv.tension * 100.0));
                        if inv.blink_active > 0.0 {
                            ui.colored_label(egui::Color32::YELLOW, "BLINK");
                        }
                        ui.label(format!("Next blink: {:.1}s", inv.blink_timer));
                        if inv.startle_ticks > 0.0 {
                            ui.colored_label(egui::Color32::RED, format!("STARTLE {:.1}s", inv.startle_ticks));
                        }
                    }

                    ui.separator();
                }

                // Biology panel
                ui.collapsing("Biology", |ui| {
                    if let Some(ref anat) = anatomy {
                        ui.label(format!("Bone health: {:.0}%", anat.skeleton.bone_health * 100.0));
                        ui.label(format!("Muscle cond: {:.0}%", anat.muscles.condition * 100.0));
                        ui.label(format!("Fat level: {:.0}%", anat.fat.level * 100.0));
                        ui.label(format!("Skin integ: {:.0}%", anat.skin.integrity * 100.0));
                        ui.label(format!("Skin hydra: {:.0}%", anat.skin.hydration * 100.0));
                        ui.label(format!("Avg flex: {:.0}%", anat.avg_flexibility() * 100.0));
                        ui.label(format!("Avg fatigue: {:.0}%", anat.avg_fatigue() * 100.0));
                    }
                    if let Some(ref hyg) = hygiene {
                        ui.separator();
                        let color = if hyg.level < 30.0 { egui::Color32::RED } else { egui::Color32::GREEN };
                        ui.colored_label(color, format!("Hygiene: {:.0}", hyg.level));
                    }
                    // Disease display removed (param limit — see HUD vitals panel instead)
                    if let Some(ref env) = environment {
                        ui.separator();
                        ui.label(format!("Temp: {:.1}°C", env.temperature));
                        if env.warmth_buff > 0.0 {
                            ui.label(format!("Warmth buff: +{:.1}", env.warmth_buff));
                        }
                    }
                });
                ui.separator();

                // Pose & Reaction state (always visible when dev mode active)
                if let Some(ref pose) = pose {
                    ui.collapsing("Pose & Reactions", |ui| {
                        for (name, angle) in &pose.angles {
                            if angle.abs() > 0.1 {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{name}:"));
                                    ui.colored_label(
                                        egui::Color32::from_rgb(240, 200, 60),
                                        format!("{angle:.1}°"),
                                    );
                                });
                            }
                        }
                        if pose.angles.values().all(|a| a.abs() < 0.1) {
                            ui.label("All joints neutral");
                        }
                        if let Some(ref expr) = expression {
                            if expr.is_active() {
                                ui.separator();
                                ui.label(format!(
                                    "Expression: eyes={} mouth={} blush={:.1} ({}t)",
                                    expr.eyes, expr.mouth, expr.blush, expr.ticks
                                ));
                            }
                        }
                    });
                    ui.separator();
                }

                // Reaction test buttons
                if dev_state.show_cheats {
                    ui.label("Test reactions:");
                    ui.horizontal(|ui| {
                        use crate::creature::behavior::reactions::CreatureReaction;
                        use crate::config::nutrition::FoodType;
                        if ui.small_button("Eat").clicked() {
                            reaction_events.write(CreatureReaction::Eating {
                                food: FoodType::VerdanceBerry,
                                preferred: true,
                            });
                        }
                        if ui.small_button("Pet").clicked() {
                            reaction_events.write(CreatureReaction::Petted { pleasure: 0.8 });
                        }
                        if ui.small_button("Flinch").clicked() {
                            reaction_events.write(CreatureReaction::Flinched { pain: 0.9 });
                        }
                        if ui.small_button("Play").clicked() {
                            reaction_events.write(CreatureReaction::PlayStart);
                        }
                    });
                    ui.horizontal(|ui| {
                        use crate::creature::behavior::reactions::CreatureReaction;
                        if ui.small_button("Sleep").clicked() {
                            reaction_events.write(CreatureReaction::FallingAsleep);
                        }
                        if ui.small_button("Refuse").clicked() {
                            reaction_events.write(CreatureReaction::RefusingFood);
                        }
                        if ui.small_button("Sick").clicked() {
                            reaction_events.write(CreatureReaction::GotSick);
                        }
                        if ui.small_button("Recover").clicked() {
                            reaction_events.write(CreatureReaction::Recovered);
                        }
                    });
                    ui.add_space(4.0);
                }

                if dev_state.show_cheats {
                    draw_cheats_panel(
                        ui,
                        &mut dev_state,
                        mind.as_deref_mut(),
                        collection.as_deref_mut(),
                        growth.as_deref_mut(),
                    );
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
            stat_bar(ui, "Thirst", mind.stats.thirst, egui::Color32::from_rgb(60, 160, 220));
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
        MoodState::Thirsty  => egui::Color32::from_rgb(60, 160, 220),
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

            ui.label("Arch: 13 -> 8 -> 8 (184 params)");

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

// ---------------------------------------------------------------------------
// Physics & breathing panel
// ---------------------------------------------------------------------------

fn draw_physics_panel(
    ui: &mut egui::Ui,
    physics_q: &Query<(&PhysicsBody, &BreathingState, &HeartbeatState, &ResonanceGlow), With<CreatureRoot>>,
) {
    egui::CollapsingHeader::new("Physics & Vitals")
        .default_open(true)
        .show(ui, |ui| {
            let Ok((body, breathing, heartbeat, glow)) = physics_q.single() else {
                ui.label("No creature found");
                return;
            };

            // Physics
            ui.label(format!(
                "Vel: ({:.1}, {:.1})",
                body.velocity.x, body.velocity.y
            ));
            ui.horizontal(|ui| {
                ui.label(format!("Grounded: {}", if body.grounded { "yes" } else { "no" }));
                if body.buoyant {
                    ui.label("| Buoyant");
                }
            });

            ui.add_space(4.0);

            // Breathing
            ui.label(format!(
                "Breathing: {:.2} Hz | amp {:.3}",
                breathing.rate, breathing.amplitude
            ));

            // Heartbeat
            ui.label(format!("Heart: {:.0} BPM", heartbeat.bpm));
            if heartbeat.irregularity > 0.0 {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("Irregular ({:.0}%)", heartbeat.irregularity * 100.0),
                );
            }

            ui.add_space(4.0);

            // Resonance glow (kokoro-sac)
            ui.label(format!(
                "Kokoro-sac: {:.1} Hz | int {:.2}",
                glow.frequency, glow.intensity
            ));

            // Absence display removed (param limit reached)
            // TODO: restore when system params are refactored to use SystemParam bundle
        });

    ui.separator();
}

// ---------------------------------------------------------------------------
// Cheats panel
// ---------------------------------------------------------------------------

fn draw_cheats_panel(
    ui: &mut egui::Ui,
    dev_state: &mut DevModeState,
    mind: Option<&mut Mind>,
    collection: Option<&mut CreatureCollection>,
    growth: Option<&mut GrowthState>,
) {
    ui.heading("Cheats");
    ui.separator();

    // --- Tick speed ---
    ui.horizontal(|ui| {
        ui.label("Speed:");
        ui.add(egui::Slider::new(&mut dev_state.tick_speed, 0.5..=10.0).text("x"));
    });

    ui.add_space(4.0);

    // --- Growth stage selector ---
    if let Some(growth) = growth {
        ui.label(format!("Stage: {:?} {}", growth.stage, if growth.dev_override { "(override)" } else { "" }));
        if ui.button("Egg").clicked() {
            growth.stage = GrowthStage::Egg;
            growth.target_scale = GrowthStage::Egg.target_scale_pub();
            growth.dev_override = true;
        }
        if ui.button("Cub").clicked() {
            growth.stage = GrowthStage::Cub;
            growth.target_scale = GrowthStage::Cub.target_scale_pub();
            growth.dev_override = true;
        }
        if ui.button("Young").clicked() {
            growth.stage = GrowthStage::Young;
            growth.target_scale = GrowthStage::Young.target_scale_pub();
            growth.dev_override = true;
        }
        if ui.button("Adult").clicked() {
            growth.stage = GrowthStage::Adult;
            growth.target_scale = GrowthStage::Adult.target_scale_pub();
            growth.dev_override = true;
        }
        if ui.button("Elder").clicked() {
            growth.stage = GrowthStage::Elder;
            growth.target_scale = GrowthStage::Elder.target_scale_pub();
            growth.dev_override = true;
        }
        if growth.dev_override && ui.button("Reset to age").clicked() {
            growth.dev_override = false;
        }
        ui.add_space(4.0);
    } else {
        ui.label("GrowthState: not available");
        ui.add_space(4.0);
    }

    if let Some(mind) = mind {
        // --- Skip time ---
        ui.horizontal(|ui| {
            if ui.button("Skip 1h").clicked() {
                mind.age_ticks += 3600;
            }
            if ui.button("Skip 1d").clicked() {
                mind.age_ticks += 86400;
            }
        });

        ui.add_space(4.0);

        // --- Max stats ---
        if ui.button("Max all stats").clicked() {
            mind.stats.hunger = 0.0;
            mind.stats.happiness = 100.0;
            mind.stats.energy = 100.0;
            mind.stats.health = 100.0;
            mind.pending_hunger = 0.0;
            mind.pending_happiness = 0.0;
            mind.pending_energy = 0.0;
        }

        ui.add_space(4.0);

        // --- Force mood ---
        ui.label("Force mood:");
        ui.horizontal(|ui| {
            for (label, mood) in [
                ("Happy", MoodState::Happy),
                ("Hungry", MoodState::Hungry),
                ("Tired", MoodState::Tired),
            ] {
                if ui.small_button(label).clicked() {
                    mind.mood = mood;
                    mind.mood_cooldown = 15;
                }
            }
        });
        ui.horizontal(|ui| {
            for (label, mood) in [
                ("Lonely", MoodState::Lonely),
                ("Playful", MoodState::Playful),
                ("Sick", MoodState::Sick),
                ("Sleep", MoodState::Sleeping),
            ] {
                if ui.small_button(label).clicked() {
                    mind.mood = mood;
                    mind.mood_cooldown = 15;
                }
            }
        });
    }

    ui.add_space(4.0);

    // --- Hatch egg ---
    if let Some(collection) = collection {
        let idx = collection.active_index;
        if let Some(creature) = collection.creatures.get_mut(idx) {
            if !creature.egg.hatched {
                if ui.button("Hatch egg now").clicked() {
                    creature.egg.progress = 1.0;
                    creature.egg.hatched = true;
                    collection.visuals_dirty = true;
                }
                ui.label(format!(
                    "Incubation: {:.0}%",
                    creature.egg.progress * 100.0
                ));
            }
        }
    }

    ui.separator();
}
