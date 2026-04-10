//! Main menu screen.
//!
//! Spawned on entering [`ScreenState::MainMenu`] and torn down on exit.
//! The only interaction is the Play button, which transitions to
//! [`ScreenState::InGame`].

use bevy::prelude::*;
use bevy::{
    input_focus::{
        AutoFocus, InputDispatchPlugin, InputFocus, InputFocusVisible,
        directional_navigation::DirectionalNavigationPlugin,
    },
    math::{CompassOctant, Dir2},
    picking::events::{Out, Over, Pointer},
    ui::auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator},
    ui_widgets::{Activate, Button, UiWidgetsPlugins},
};

use crate::GameState;
use crate::GameSystems;
use crate::ScreenState;
use crate::game_logic::Score;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputDispatchPlugin,
            DirectionalNavigationPlugin,
            UiWidgetsPlugins,
        ))
        .insert_resource(InputFocusVisible(true))
        .add_observer(on_play_activate)
        .add_observer(on_quit_activate)
        .add_observer(on_main_menu_activate)
        .add_observer(on_pointer_over)
        .add_observer(on_pointer_out)
        .add_systems(OnEnter(ScreenState::MainMenu), setup_menu)
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_screen)
        .add_systems(
            Update,
            (navigate_menu_with_arrows, update_menu_focus_indicator).in_set(GameSystems::Logic),
        );
    }
}

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct MainMenuButton;

/// Stores the normal and hovered background colors for a menu button.
#[derive(Component)]
struct MenuButtonColors {
    normal: Color,
    hovered: Color,
}

const BUTTON_NORMAL: Color = Color::hsl(224.0, 0.72, 0.38);
const BUTTON_HOVERED: Color = Color::hsl(224.0, 0.72, 0.50);
const QUIT_BUTTON_NORMAL: Color = Color::hsl(0.0, 0.60, 0.35);
const QUIT_BUTTON_HOVERED: Color = Color::hsl(0.0, 0.60, 0.47);
const BUTTON_FOCUS_BORDER: Color = Color::hsl(52.0, 0.30, 0.68);

fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            // This will cause all child nodes to be despawned when we leave the main menu as well.
            DespawnOnExit(ScreenState::MainMenu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(28.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Bevy Match-3"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Play button
            parent
                .spawn((
                    PlayButton,
                    AutoFocus,
                    Button,
                    AutoDirectionalNavigation::default(),
                    MenuButtonColors {
                        normal: BUTTON_NORMAL,
                        hovered: BUTTON_HOVERED,
                    },
                    Node {
                        padding: UiRect::axes(Val::Px(52.0), Val::Px(18.0)),
                        margin: UiRect::top(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_NORMAL),
                    BorderColor::all(Color::NONE),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Play"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit button
            parent
                .spawn((
                    QuitButton,
                    Button,
                    AutoDirectionalNavigation::default(),
                    MenuButtonColors {
                        normal: QUIT_BUTTON_NORMAL,
                        hovered: QUIT_BUTTON_HOVERED,
                    },
                    Node {
                        padding: UiRect::axes(Val::Px(52.0), Val::Px(18.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(QUIT_BUTTON_NORMAL),
                    BorderColor::all(Color::NONE),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Quit"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Full-screen overlay: final score + button back to the main menu.
fn setup_game_over_screen(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn((
            DespawnOnExit(GameState::GameOver),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(28.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("No More Moves!"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(score.to_string()),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::hsl(54.0, 0.91, 0.70)),
            ));

            parent
                .spawn((
                    MainMenuButton,
                    AutoFocus,
                    Button,
                    AutoDirectionalNavigation::default(),
                    MenuButtonColors {
                        normal: BUTTON_NORMAL,
                        hovered: BUTTON_HOVERED,
                    },
                    Node {
                        padding: UiRect::axes(Val::Px(52.0), Val::Px(18.0)),
                        margin: UiRect::top(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_NORMAL),
                    BorderColor::all(Color::NONE),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Main Menu"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn on_play_activate(
    event: On<Activate>,
    buttons: Query<(), With<PlayButton>>,
    mut next_screen_state: ResMut<NextState<ScreenState>>,
) {
    if buttons.get(event.entity).is_ok() {
        next_screen_state.set(ScreenState::InGame);
    }
}

fn on_quit_activate(
    event: On<Activate>,
    buttons: Query<(), With<QuitButton>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if buttons.get(event.entity).is_ok() {
        app_exit.write(AppExit::Success);
    }
}

fn on_main_menu_activate(
    event: On<Activate>,
    buttons: Query<(), With<MainMenuButton>>,
    mut next_screen_state: ResMut<NextState<ScreenState>>,
) {
    if buttons.get(event.entity).is_ok() {
        next_screen_state.set(ScreenState::MainMenu);
    }
}

fn on_pointer_over(
    event: On<Pointer<Over>>,
    mut buttons: Query<(&MenuButtonColors, &mut BackgroundColor)>,
) {
    if let Ok((colors, mut bg)) = buttons.get_mut(event.entity) {
        *bg = BackgroundColor(colors.hovered);
    }
}

fn on_pointer_out(
    event: On<Pointer<Out>>,
    mut buttons: Query<(&MenuButtonColors, &mut BackgroundColor)>,
) {
    if let Ok((colors, mut bg)) = buttons.get_mut(event.entity) {
        *bg = BackgroundColor(colors.normal);
    }
}

fn navigate_menu_with_arrows(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut focus_visible: ResMut<InputFocusVisible>,
    mut auto_directional_navigator: AutoDirectionalNavigator,
    menu_buttons: Query<(), Or<(With<PlayButton>, With<QuitButton>, With<MainMenuButton>)>>,
) {
    if menu_buttons.is_empty() {
        return;
    }

    let net_east_west = keyboard.just_pressed(KeyCode::ArrowRight) as i8
        - keyboard.just_pressed(KeyCode::ArrowLeft) as i8;
    let net_north_south = keyboard.just_pressed(KeyCode::ArrowUp) as i8
        - keyboard.just_pressed(KeyCode::ArrowDown) as i8;

    if let Ok(direction) = Dir2::from_xy(net_east_west as f32, net_north_south as f32) {
        focus_visible.0 = true;
        let _ = auto_directional_navigator.navigate(CompassOctant::from(direction));
    }
}

fn update_menu_focus_indicator(
    input_focus: Res<InputFocus>,
    focus_visible: Res<InputFocusVisible>,
    mut buttons: Query<
        (Entity, &mut BorderColor),
        Or<(With<PlayButton>, With<QuitButton>, With<MainMenuButton>)>,
    >,
) {
    for (entity, mut border_color) in &mut buttons {
        if focus_visible.0 && input_focus.get() == Some(entity) {
            *border_color = BorderColor::all(BUTTON_FOCUS_BORDER);
        } else {
            *border_color = BorderColor::all(Color::NONE);
        }
    }
}
