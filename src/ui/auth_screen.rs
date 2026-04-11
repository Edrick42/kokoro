//! Auth screen — Login / Register / Play as Guest.
//!
//! Connected to the Kokoro web API. Login and Register make real
//! HTTP calls via reqwest. Guest mode plays offline (no sync).

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, buttons, PixelFont};
use crate::game::state::AppState;
use crate::ui::style::{AnimatedButton, ButtonRestColor};
use crate::ui::text_input::{self, TextInputField, FocusedInput};
use crate::web::{self, WebSession};

// ===================================================================
// Components
// ===================================================================

#[derive(Component)]
struct AuthScreenUI;

#[derive(Component)]
enum AuthAction { Login, Register, Guest }

#[derive(Component)]
struct AuthErrorText;

#[derive(Component)]
struct EmailField;

#[derive(Component)]
struct PasswordField;


// ===================================================================
// State
// ===================================================================

#[derive(Resource, Default)]
struct AuthScreenMode {
    /// true = register form, false = login form
    register: bool,
}

// ===================================================================
// Plugin
// ===================================================================

pub struct AuthScreenPlugin;

impl Plugin for AuthScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuthScreenMode::default())
           .insert_resource(FocusedInput::default())
           .add_systems(OnEnter(AppState::Auth), setup_auth_screen)
           .add_systems(OnExit(AppState::Auth), cleanup_auth_screen)
           .add_systems(Update, (
               text_input::handle_input_focus,
               text_input::handle_input_typing,
               handle_auth_input,
           ).run_if(in_state(AppState::Auth)));
    }
}

// ===================================================================
// Setup
// ===================================================================

fn setup_auth_screen(
    mut commands: Commands,
    pixel_font: Res<PixelFont>,
    mut mode: ResMut<AuthScreenMode>,
) {
    mode.register = false;

    let font_lg = TextFont { font: pixel_font.0.clone(), font_size: fonts::LG, ..default() };
    let font_md = TextFont { font: pixel_font.0.clone(), font_size: fonts::MD, ..default() };
    let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };

    let mut email_entity = Entity::PLACEHOLDER;
    let mut password_entity = Entity::PLACEHOLDER;

    commands.spawn((
        AuthScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            padding: UiRect::horizontal(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(palette::CREAM),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("KOKORO"),
            font_lg.clone(),
            TextColor(palette::NEAR_BLACK),
        ));

        parent.spawn(Node { height: Val::Px(16.0), ..default() });

        // Error message (hidden by default)
        parent.spawn((
            Text::new(""),
            font_sm.clone(),
            TextColor(palette::RED),
            AuthErrorText,
            Node { max_width: Val::Px(280.0), ..default() },
        ));

        // Email field
        parent.spawn((
            Text::new("EMAIL"),
            font_sm.clone(),
            TextColor(palette::NEAR_BLACK),
        ));
        email_entity = text_input::spawn_text_input(parent, &pixel_font, "your@email.com", false, 260.0);

        // Password field
        parent.spawn((
            Text::new("PASSWORD"),
            font_sm.clone(),
            TextColor(palette::NEAR_BLACK),
            Node { margin: UiRect::top(Val::Px(4.0)), ..default() },
        ));
        password_entity = text_input::spawn_text_input(parent, &pixel_font, "********", true, 260.0);

        parent.spawn(Node { height: Val::Px(8.0), ..default() });

        // Login button
        spawn_auth_btn(parent, &font_md, "LOGIN", AuthAction::Login, palette::TEAL);

        // Register button
        spawn_auth_btn(parent, &font_md, "REGISTER", AuthAction::Register, palette::GOLD);

        parent.spawn(Node { height: Val::Px(8.0), ..default() });

        // Guest button (subtle)
        spawn_auth_btn(parent, &font_sm, "PLAY AS GUEST", AuthAction::Guest, palette::CREAM);
    });

    // Insert markers after the closure (avoids double-borrow of commands)
    commands.entity(email_entity).insert(EmailField);
    commands.entity(password_entity).insert(PasswordField);
}

fn spawn_auth_btn(parent: &mut ChildSpawnerCommands, font: &TextFont, label: &str, action: AuthAction, bg: Color) {
    let text_color = if bg == palette::CREAM { palette::NEAR_BLACK } else { palette::CREAM };

    parent.spawn((
        Button, AnimatedButton, ButtonRestColor(bg), action,
        Node {
            width: Val::Px(260.0),
            height: Val::Px(buttons::HEIGHT + 4.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            ..default()
        },
        BorderColor(palette::NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(bg),
    )).with_child((
        Text::new(label.to_string()),
        font.clone(),
        TextColor(text_color),
    ));
}

fn cleanup_auth_screen(
    mut commands: Commands,
    query: Query<Entity, With<AuthScreenUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ===================================================================
// Systems
// ===================================================================

fn handle_auth_input(
    query: Query<(&Interaction, &AuthAction), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut web_session: ResMut<WebSession>,
    email_q: Query<&TextInputField, With<EmailField>>,
    password_q: Query<&TextInputField, With<PasswordField>>,
    mut error_q: Query<&mut Text, With<AuthErrorText>>,
) {
    for (interaction, action) in query.iter() {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            AuthAction::Login => {
                let email = email_q.single().map(|f| f.value.clone()).unwrap_or_default();
                let password = password_q.single().map(|f| f.value.clone()).unwrap_or_default();

                if email.is_empty() || password.is_empty() {
                    set_error(&mut error_q, "Enter email and password");
                    continue;
                }

                match web::auth::login(&email, &password) {
                    Ok(session) => {
                        info!("Logged in as {}", session.display_name);
                        web_session.active = Some(session);
                        next_state.set(AppState::Onboarding);
                    }
                    Err(msg) => {
                        warn!("Login failed: {msg}");
                        set_error(&mut error_q, &msg);
                    }
                }
            }
            AuthAction::Register => {
                let email = email_q.single().map(|f| f.value.clone()).unwrap_or_default();
                let password = password_q.single().map(|f| f.value.clone()).unwrap_or_default();

                if email.is_empty() || password.len() < 8 {
                    set_error(&mut error_q, "Email required, password min 8 chars");
                    continue;
                }

                // Use email prefix as display name for simplicity
                let name = email.split('@').next().unwrap_or("Player").to_string();

                match web::auth::register(&name, &email, &password) {
                    Ok(session) => {
                        info!("Registered as {}", session.display_name);
                        web_session.active = Some(session);
                        next_state.set(AppState::Onboarding);
                    }
                    Err(msg) => {
                        warn!("Register failed: {msg}");
                        set_error(&mut error_q, &msg);
                    }
                }
            }
            AuthAction::Guest => {
                info!("Playing as guest (offline mode)");
                web_session.active = None;
                next_state.set(AppState::Onboarding);
            }
        }
    }
}

fn set_error(error_q: &mut Query<&mut Text, With<AuthErrorText>>, msg: &str) {
    if let Ok(mut text) = error_q.single_mut() {
        *text = Text::new(msg);
    }
}
