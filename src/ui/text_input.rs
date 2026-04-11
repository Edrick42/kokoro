//! Minimal text input widget for retro UI.
//!
//! Bevy doesn't have built-in text fields. This provides a basic
//! clickable text input with cursor, backspace, and character input.

use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;

use crate::config::ui::{palette, fonts, buttons, PixelFont};

/// Marker for a text input field. Contains the current value.
#[derive(Component)]
pub struct TextInputField {
    pub value: String,
    pub placeholder: String,
    pub max_len: usize,
    pub masked: bool,
}

/// Marker for the text display child of a text input.
#[derive(Component)]
pub struct TextInputDisplay;

/// Which field is currently focused (receives keyboard input).
#[derive(Resource, Default)]
pub struct FocusedInput(pub Option<Entity>);

/// Spawns a text input field. Returns the entity ID.
pub fn spawn_text_input(
    parent: &mut ChildSpawnerCommands,
    pixel_font: &PixelFont,
    placeholder: &str,
    masked: bool,
    width: f32,
) -> Entity {
    let font = TextFont {
        font: pixel_font.0.clone(),
        font_size: fonts::SM,
        ..default()
    };

    let field_entity = parent.spawn((
        Button,
        TextInputField {
            value: String::new(),
            placeholder: placeholder.to_string(),
            max_len: 64,
            masked,
        },
        Node {
            width: Val::Px(width),
            height: Val::Px(28.0),
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(palette::NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(palette::CREAM),
    )).with_children(|field| {
        field.spawn((
            Text::new(placeholder),
            font,
            TextColor(palette::GRAY),
            TextInputDisplay,
        ));
    }).id();

    field_entity
}

/// System: focus a text input when clicked.
pub fn handle_input_focus(
    query: Query<(Entity, &Interaction), (Changed<Interaction>, With<TextInputField>)>,
    mut focused: ResMut<FocusedInput>,
    mut border_q: Query<&mut BorderColor, With<TextInputField>>,
) {
    for (entity, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            // Unfocus previous
            if let Some(prev) = focused.0 {
                if let Ok(mut border) = border_q.get_mut(prev) {
                    border.0 = palette::NEAR_BLACK;
                }
            }
            // Focus new
            focused.0 = Some(entity);
            if let Ok(mut border) = border_q.get_mut(entity) {
                border.0 = palette::TEAL;
            }
        }
    }
}

/// System: handle keyboard input for the focused text field.
pub fn handle_input_typing(
    mut events: EventReader<KeyboardInput>,
    focused: Res<FocusedInput>,
    mut field_q: Query<(&mut TextInputField, &Children)>,
    mut text_q: Query<(&mut Text, &mut TextColor), With<TextInputDisplay>>,
) {
    let Some(focused_entity) = focused.0 else { return };
    let Ok((mut field, children)) = field_q.get_mut(focused_entity) else { return };

    for event in events.read() {
        if event.state != ButtonState::Pressed { continue; }

        match &event.logical_key {
            bevy::input::keyboard::Key::Backspace => {
                field.value.pop();
            }
            bevy::input::keyboard::Key::Character(c) => {
                let ch = c.as_str();
                if field.value.len() < field.max_len && !ch.is_empty() {
                    // Filter control characters
                    for c in ch.chars() {
                        if !c.is_control() && field.value.len() < field.max_len {
                            field.value.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Update display text
    for child in children.iter() {
        if let Ok((mut text, mut color)) = text_q.get_mut(child) {
            if field.value.is_empty() {
                *text = Text::new(&field.placeholder);
                color.0 = palette::GRAY;
            } else if field.masked {
                *text = Text::new("*".repeat(field.value.len()));
                color.0 = palette::NEAR_BLACK;
            } else {
                *text = Text::new(&field.value);
                color.0 = palette::NEAR_BLACK;
            }
        }
    }
}
