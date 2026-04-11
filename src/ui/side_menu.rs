//! Side menu — left drawer with Profile, Settings, Lore, and Shop tabs.
//!
//! Hamburger button (top-right area) toggles the drawer.
//! Each tab renders its own content in the scrollable panel.

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, buttons, PixelFont};
use crate::config::shop::{self, ShopCategory};
use crate::game::state::{AppState, GameplayEntity};
use crate::genome::Genome;
use crate::mind::Mind;
use crate::web::WebSession;
use super::style::*;

// ===================================================================
// Components
// ===================================================================

#[derive(Component)]
struct HamburgerBtn;

#[derive(Component)]
struct DrawerPanel;

#[derive(Component)]
struct DrawerContent;

#[derive(Component)]
struct TabBtn(SideMenuTab);

#[derive(Component)]
struct TabHighlight(SideMenuTab);

#[derive(Component)]
struct ShopBuyBtn(#[allow(dead_code)] &'static str);

#[derive(Component)]
struct ToastText;

// ===================================================================
// State
// ===================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SideMenuTab {
    Profile,
    Settings,
    Lore,
    Shop,
}

impl SideMenuTab {
    const ALL: [Self; 4] = [Self::Profile, Self::Settings, Self::Lore, Self::Shop];

    fn label(&self) -> &'static str {
        match self {
            Self::Profile  => "PROFILE",
            Self::Settings => "SETTINGS",
            Self::Lore     => "LORE",
            Self::Shop     => "SHOP",
        }
    }
}

#[derive(Resource)]
struct SideMenuState {
    open: bool,
    active_tab: SideMenuTab,
    needs_rebuild: bool,
    toast_timer: f32,
}

impl Default for SideMenuState {
    fn default() -> Self {
        Self { open: false, active_tab: SideMenuTab::Profile, needs_rebuild: false, toast_timer: 0.0 }
    }
}

#[derive(Resource)]
struct SoundEnabled(pub bool);

#[derive(Resource)]
struct MusicEnabled(pub bool);

// ===================================================================
// Plugin
// ===================================================================

pub struct SideMenuPlugin;

impl Plugin for SideMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SideMenuState::default())
           .insert_resource(SoundEnabled(true))
           .insert_resource(MusicEnabled(true))
           .add_systems(OnEnter(AppState::Gameplay), setup_side_menu)
           .add_systems(Update, (
               handle_hamburger_toggle,
               handle_tab_press,
               handle_shop_buy,
               sync_drawer_visibility,
               rebuild_content,
               fade_toast,
           ).run_if(in_state(AppState::Gameplay)));
    }
}

// ===================================================================
// Setup
// ===================================================================

fn setup_side_menu(mut commands: Commands, pixel_font: Res<PixelFont>, mut state: ResMut<SideMenuState>) {
    *state = SideMenuState::default();
    let font_lg = TextFont { font: pixel_font.0.clone(), font_size: fonts::LG, ..default() };

    // Hamburger button — top-right, below vitals panel
    commands.spawn((
        GameplayEntity, Button, AnimatedButton, HamburgerBtn,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            right: Val::Px(8.0),
            width: Val::Px(36.0),
            height: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            ..default()
        },
        BorderColor(NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(CREAM),
    )).with_child((
        Text::new("="),
        font_lg.clone(),
        TextColor(NEAR_BLACK),
    ));

    // Drawer panel — left side, full height
    commands.spawn((
        GameplayEntity, DrawerPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            bottom: Val::Px(0.0),
            width: Val::Px(210.0),
            flex_direction: FlexDirection::Column,
            border: UiRect::right(Val::Px(2.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(palette::CREAM),
        BorderColor(NEAR_BLACK),
    )).with_children(|drawer| {
        // Tab bar
        drawer.spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            padding: UiRect::all(Val::Px(4.0)),
            row_gap: Val::Px(2.0),
            column_gap: Val::Px(2.0),
            ..default()
        }).with_children(|tabs| {
            let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };
            for tab in SideMenuTab::ALL {
                let is_active = tab == SideMenuTab::Profile;
                let bg = if is_active { palette::NEAR_BLACK } else { palette::CREAM };
                let text_color = if is_active { palette::CREAM } else { palette::NEAR_BLACK };
                tabs.spawn((
                    Button, TabBtn(tab),
                    Node {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(NEAR_BLACK),
                    BorderRadius::all(Val::Px(0.0)),
                    BackgroundColor(bg),
                    TabHighlight(tab),
                )).with_child((
                    Text::new(tab.label()),
                    font_sm.clone(),
                    TextColor(text_color),
                ));
            }
        });

        // Separator line
        drawer.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(NEAR_BLACK),
        ));

        // Scrollable content area
        drawer.spawn((
            DrawerContent,
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(6.0),
                flex_grow: 1.0,
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ));
    });

    // Toast notification (hidden by default)
    commands.spawn((
        GameplayEntity, ToastText,
        Text::new(""),
        TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() },
        TextColor(palette::CREAM),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-60.0)),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(palette::NEAR_BLACK),
    ));
}

// ===================================================================
// Systems
// ===================================================================

fn handle_hamburger_toggle(
    query: Query<&Interaction, (Changed<Interaction>, With<HamburgerBtn>)>,
    mut state: ResMut<SideMenuState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            state.open = !state.open;
            if state.open { state.needs_rebuild = true; }
        }
    }
}

fn handle_tab_press(
    query: Query<(&Interaction, &TabBtn), Changed<Interaction>>,
    mut state: ResMut<SideMenuState>,
) {
    for (interaction, tab_btn) in query.iter() {
        if *interaction == Interaction::Pressed && state.active_tab != tab_btn.0 {
            state.active_tab = tab_btn.0;
            state.needs_rebuild = true;
        }
    }
}

fn sync_drawer_visibility(
    state: Res<SideMenuState>,
    mut drawer_q: Query<&mut Node, With<DrawerPanel>>,
    mut tab_q: Query<(&TabHighlight, &mut BackgroundColor, &Children)>,
    mut text_q: Query<&mut TextColor>,
) {
    if !state.is_changed() { return; }

    // Show/hide drawer
    for mut node in drawer_q.iter_mut() {
        node.display = if state.open { Display::Flex } else { Display::None };
    }

    // Update tab highlights
    for (highlight, mut bg, children) in tab_q.iter_mut() {
        let is_active = highlight.0 == state.active_tab;
        bg.0 = if is_active { palette::NEAR_BLACK } else { palette::CREAM };
        for child in children.iter() {
            if let Ok(mut tc) = text_q.get_mut(child) {
                tc.0 = if is_active { palette::CREAM } else { palette::NEAR_BLACK };
            }
        }
    }
}

fn rebuild_content(
    mut commands: Commands,
    mut state: ResMut<SideMenuState>,
    content_q: Query<(Entity, &Children), With<DrawerContent>>,
    pixel_font: Res<PixelFont>,
    mind: Res<Mind>,
    genome: Res<Genome>,
    sound_enabled: Res<SoundEnabled>,
    music_enabled: Res<MusicEnabled>,
    web_session: Res<WebSession>,
) {
    if !state.needs_rebuild { return; }
    state.needs_rebuild = false;

    let Ok((content_entity, children)) = content_q.single() else { return; };

    // Clear existing content
    for child in children.iter() {
        commands.entity(child).despawn();
    }

    let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };
    let font_md = TextFont { font: pixel_font.0.clone(), font_size: fonts::MD, ..default() };

    commands.entity(content_entity).with_children(|panel| {
        match state.active_tab {
            SideMenuTab::Profile  => build_profile_tab(panel, &font_sm, &font_md, &mind, &genome, &web_session),
            SideMenuTab::Settings => build_settings_tab(panel, &font_sm, &font_md, &sound_enabled, &music_enabled),
            SideMenuTab::Lore     => build_lore_tab(panel, &font_sm, &font_md),
            SideMenuTab::Shop     => build_shop_tab(panel, &font_sm, &font_md),
        }
    });
}

// ===================================================================
// Tab builders
// ===================================================================

fn build_profile_tab(panel: &mut ChildSpawnerCommands, font_sm: &TextFont, font_md: &TextFont, mind: &Mind, genome: &Genome, web_session: &WebSession) {
    let species_name = match &genome.species {
        crate::genome::Species::Moluun => "Moluun",
        crate::genome::Species::Pylum  => "Pylum",
        crate::genome::Species::Skael  => "Skael",
        crate::genome::Species::Nyxal  => "Nyxal",
    };

    let age_days = mind.age_ticks / 86400;

    // Account info
    if let Some(ref session) = web_session.active {
        section_title(panel, font_md, "ACCOUNT");
        info_row(panel, font_sm, "Name", &session.display_name);
        info_row(panel, font_sm, "Email", &session.email);
        info_row(panel, font_sm, "Sync", "Active");
        separator(panel);
    } else {
        section_title(panel, font_md, "GUEST MODE");
        panel.spawn((
            Text::new("Login to sync your Kobara"),
            font_sm.clone(),
            TextColor(palette::GRAY),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
        ));
        separator(panel);
    }

    // Title
    section_title(panel, font_md, "YOUR KOBARA");

    // Species + age
    info_row(panel, font_sm, "Species", species_name);
    info_row(panel, font_sm, "Age", &format!("{} days", age_days));
    info_row(panel, font_sm, "Mood", mind.mood.label());

    separator(panel);

    // Stats
    section_title(panel, font_md, "STATS");
    info_row(panel, font_sm, "Hunger", &format!("{:.0}", mind.stats.hunger));
    info_row(panel, font_sm, "Happiness", &format!("{:.0}", mind.stats.happiness));
    info_row(panel, font_sm, "Energy", &format!("{:.0}", mind.stats.energy));

    separator(panel);

    // Genome
    section_title(panel, font_md, "GENOME");
    gene_row(panel, font_sm, "Curiosity", genome.curiosity);
    gene_row(panel, font_sm, "Social need", genome.loneliness_sensitivity);
    gene_row(panel, font_sm, "Appetite", genome.appetite);
    gene_row(panel, font_sm, "Circadian", genome.circadian);
    gene_row(panel, font_sm, "Resilience", genome.resilience);
    gene_row(panel, font_sm, "Learning", genome.learning_rate);
    gene_row(panel, font_sm, "Hue", genome.hue / 360.0);
}

fn build_settings_tab(panel: &mut ChildSpawnerCommands, font_sm: &TextFont, font_md: &TextFont, sound: &SoundEnabled, music: &MusicEnabled) {
    section_title(panel, font_md, "AUDIO");

    let sound_label = if sound.0 { "Sound: ON" } else { "Sound: OFF" };
    let music_label = if music.0 { "Music: ON" } else { "Music: OFF" };

    panel.spawn((
        Text::new(sound_label),
        font_sm.clone(),
        TextColor(palette::NEAR_BLACK),
        TextLayout::new_with_linebreak(LineBreak::WordBoundary),
    ));
    panel.spawn((
        Text::new(music_label),
        font_sm.clone(),
        TextColor(palette::NEAR_BLACK),
        TextLayout::new_with_linebreak(LineBreak::WordBoundary),
    ));

    separator(panel);

    section_title(panel, font_md, "DISPLAY");
    panel.spawn((
        Text::new("More settings coming soon."),
        font_sm.clone(),
        TextColor(palette::GRAY),
        TextLayout::new_with_linebreak(LineBreak::WordBoundary),
    ));
}

fn build_lore_tab(panel: &mut ChildSpawnerCommands, font_sm: &TextFont, font_md: &TextFont) {
    section_title(panel, font_md, "ETHARA");
    lore_text(panel, font_sm,
        "A world on the other side of perception. \
         Not a place you travel to — a place you arrive at \
         when the boundary between your inner world \
         and the outer one grows thin enough."
    );

    separator(panel);

    section_title(panel, font_md, "MOLUUN");
    lore_text(panel, font_sm,
        "Forest species. Round, social, fur-covered. \
         Reflects emotional openness. The Verdance."
    );

    section_title(panel, font_md, "PYLUM");
    lore_text(panel, font_sm,
        "Highland species. Winged, curious. \
         Reflects the seeking spirit. Veridian Highlands."
    );

    section_title(panel, font_md, "SKAEL");
    lore_text(panel, font_sm,
        "Cave species. Scaled, resilient. \
         Reflects quiet strength. Abyssal Shallows."
    );

    section_title(panel, font_md, "NYXAL");
    lore_text(panel, font_sm,
        "Deep ocean species. Tentacled, bioluminescent. \
         Reflects pattern-thinking. Abyssal Depths."
    );

    separator(panel);

    section_title(panel, font_md, "KOKORO-SAC");
    lore_text(panel, font_sm,
        "A resonance organ in every Kobara's chest. \
         It vibrates at frequencies that mirror \
         the emotional state of the bonded spirit."
    );
}

fn build_shop_tab(panel: &mut ChildSpawnerCommands, font_sm: &TextFont, font_md: &TextFont) {
    section_title(panel, font_md, "PREMIUM FOODS");

    for item in shop::items_by_category(ShopCategory::PremiumFood) {
        shop_item_row(panel, font_sm, item);
    }

    separator(panel);

    section_title(panel, font_md, "COSMETICS");

    for item in shop::items_by_category(ShopCategory::Cosmetic) {
        shop_item_row(panel, font_sm, item);
    }
}

// ===================================================================
// UI helpers
// ===================================================================

fn section_title(panel: &mut ChildSpawnerCommands, font: &TextFont, title: &str) {
    panel.spawn((
        Text::new(title),
        font.clone(),
        TextColor(palette::NEAR_BLACK),
    ));
}

fn info_row(panel: &mut ChildSpawnerCommands, font: &TextFont, label: &str, value: &str) {
    panel.spawn(Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        width: Val::Percent(100.0),
        ..default()
    }).with_children(|row| {
        row.spawn((Text::new(label), font.clone(), TextColor(palette::GRAY)));
        row.spawn((Text::new(value), font.clone(), TextColor(palette::NEAR_BLACK)));
    });
}

fn gene_row(panel: &mut ChildSpawnerCommands, font: &TextFont, label: &str, value: f32) {
    let bar_width = (value.clamp(0.0, 1.0) * 60.0).max(2.0);

    panel.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(4.0),
        width: Val::Percent(100.0),
        ..default()
    }).with_children(|row| {
        // Label (fixed width)
        row.spawn((
            Text::new(label),
            font.clone(),
            TextColor(palette::GRAY),
            Node { width: Val::Px(70.0), ..default() },
        ));
        // Bar background
        row.spawn((
            Node {
                width: Val::Px(60.0),
                height: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(palette::GRAY),
        )).with_children(|bg| {
            bg.spawn((
                Node {
                    width: Val::Px(bar_width),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(palette::TEAL),
            ));
        });
    });
}

fn separator(panel: &mut ChildSpawnerCommands) {
    panel.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(palette::GRAY),
    ));
}

fn lore_text(panel: &mut ChildSpawnerCommands, font: &TextFont, text: &str) {
    panel.spawn((
        Text::new(text),
        font.clone(),
        TextColor(palette::NEAR_BLACK),
        TextLayout::new_with_linebreak(LineBreak::WordBoundary),
        Node { max_width: Val::Px(190.0), ..default() },
    ));
}

fn shop_item_row(panel: &mut ChildSpawnerCommands, font: &TextFont, item: &'static shop::ShopItem) {
    panel.spawn(Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(4.0)),
        border: UiRect::all(Val::Px(1.0)),
        margin: UiRect::bottom(Val::Px(2.0)),
        width: Val::Percent(100.0),
        ..default()
    }).with_children(|card| {
        // Name + price row
        card.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|row| {
            row.spawn((Text::new(item.name), font.clone(), TextColor(palette::NEAR_BLACK)));
            row.spawn((Text::new(format!("{} EC", item.price)), font.clone(), TextColor(palette::GOLD)));
        });

        // Description
        card.spawn((
            Text::new(item.description),
            font.clone(),
            TextColor(palette::GRAY),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
            Node { max_width: Val::Px(180.0), margin: UiRect::top(Val::Px(2.0)), ..default() },
        ));

        // Buy button
        card.spawn((
            Button, ShopBuyBtn(item.id),
            Node {
                margin: UiRect::top(Val::Px(3.0)),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                border: UiRect::all(Val::Px(1.0)),
                align_self: AlignSelf::FlexEnd,
                ..default()
            },
            BorderColor(palette::NEAR_BLACK),
            BorderRadius::all(Val::Px(0.0)),
            BackgroundColor(palette::TEAL),
        )).with_child((
            Text::new("BUY"),
            font.clone(),
            TextColor(palette::CREAM),
        ));
    });
}

// ===================================================================
// Shop buy handler + toast
// ===================================================================

fn handle_shop_buy(
    query: Query<&Interaction, (Changed<Interaction>, With<ShopBuyBtn>)>,
    mut state: ResMut<SideMenuState>,
    mut toast_q: Query<(&mut Text, &mut Node), With<ToastText>>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed { continue; }

        // Show "Coming Soon" toast
        if let Ok((mut text, mut node)) = toast_q.single_mut() {
            *text = Text::new("Coming Soon!");
            node.display = Display::Flex;
            state.toast_timer = 2.0;
        }
    }
}

fn fade_toast(
    time: Res<Time>,
    mut state: ResMut<SideMenuState>,
    mut toast_q: Query<&mut Node, With<ToastText>>,
) {
    if state.toast_timer <= 0.0 { return; }

    state.toast_timer -= time.delta_secs();
    if state.toast_timer <= 0.0 {
        if let Ok(mut node) = toast_q.single_mut() {
            node.display = Display::None;
        }
    }
}
