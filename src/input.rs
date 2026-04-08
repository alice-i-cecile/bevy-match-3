//! Player input handling — keyboard, mouse, and gamepad.
//!
//! Each input handler translates raw device input into a [`SelectAction`]
//! message, which [`crate::selection`] consumes to update selection state and
//! emit a [`crate::selection::SwapMessage`] when a valid swap is ready.
//!
//! # Cursor vs. mouse
//!
//! The keyboard and gamepad navigate a visible [`BoardCursor`] around the grid.
//! Mouse clicks use the picking system to hit-test gem sprites directly.
//!
//! Clicks also update the cursor position so that switching between mouse
//! and keyboard feels natural.

use crate::GameState;
use crate::GameSystems;
use crate::ScreenState;
use crate::board::{GRID_COLS, GRID_ROWS, GridPos};
use crate::cursor::BoardCursor;
use crate::gems::GemType;
use bevy::picking::events::{Click, Pointer};
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SelectAction>()
            .add_systems(
                Update,
                (
                    move_cursor_with_keyboard,
                    move_cursor_with_gamepad,
                    move_cursor_with_mouse,
                )
                    .in_set(InputSystems::MoveCursor)
                    .in_set(GameSystems::Input),
            )
            .add_systems(
                Update,
                (
                    confirm_with_keyboard,
                    confirm_with_gamepad,
                    confirm_with_mouse,
                )
                    .in_set(InputSystems::ConfirmSelection)
                    .in_set(GameSystems::Input),
            )
            // Cursor movement is allowed during Playing and Animating for
            // responsiveness, but frozen during GameOver so the cursor doesn't
            // drift behind the overlay.
            //
            // This is a closure run condition: any `Fn(…) -> bool` that takes
            // only system parameters can be used as a run condition.
            // `in_state(…)` is just a helper that returns one of these.
            .configure_sets(
                Update,
                // This is a custom run condition, defined as a closure.
                // Any system that returns `bool` can be used as a run condition.
                // true means "run this system as normal", while false means "skip this system entirely".
                InputSystems::MoveCursor.run_if(|game_state: Option<Res<State<GameState>>>| {
                    if let Some(game_state) = game_state {
                        matches!(game_state.get(), GameState::Playing | GameState::Animating)
                    } else {
                        // GameState only exists while ScreenState::InGame is active,
                        // so `None` means we're on the main menu.

                        false
                    }
                }),
            )
            // Confirmation input is blocked during animation to prevent mid-animation state changes.
            .configure_sets(
                Update,
                InputSystems::ConfirmSelection
                    // We should move the cursor before confirming selection
                    .after(InputSystems::MoveCursor)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                return_to_main_menu
                    .run_if(in_state(ScreenState::InGame))
                    .in_set(GameSystems::Input),
            );
    }
}

/// System sets for input ordering.
///
/// [`MoveCursor`] can run during animation;
/// [`ConfirmSelection`] is blocked to prevent mid-animation state changes.
///
/// [`MoveCursor`]: InputSystems::MoveCursor
/// [`ConfirmSelection`]: InputSystems::ConfirmSelection
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum InputSystems {
    MoveCursor,
    ConfirmSelection,
}

/// Sent when the player confirms a grid cell.
#[derive(Message, Debug, Clone, Copy)]
pub struct SelectAction {
    pub pos: GridPos,
}

fn move_cursor_with_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor: Single<&mut BoardCursor>,
) {
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        cursor.col = cursor.col.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        cursor.col = (cursor.col + 1).min(GRID_COLS - 1);
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        cursor.row = cursor.row.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        cursor.row = (cursor.row + 1).min(GRID_ROWS - 1);
    }
}

fn move_cursor_with_gamepad(gamepads: Query<&Gamepad>, mut cursor: Single<&mut BoardCursor>) {
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::DPadLeft) {
            cursor.col = cursor.col.saturating_sub(1);
        }
        if gamepad.just_pressed(GamepadButton::DPadRight) {
            cursor.col = (cursor.col + 1).min(GRID_COLS - 1);
        }
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            cursor.row = cursor.row.saturating_sub(1);
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            cursor.row = (cursor.row + 1).min(GRID_ROWS - 1);
        }
    }
}

fn move_cursor_with_mouse(
    mut messages: MessageReader<Pointer<Click>>,
    gems: Query<&GridPos, With<GemType>>,
    mut cursor: Option<Single<&mut BoardCursor>>,
) {
    for message in messages.read() {
        let Ok(&pos) = gems.get(message.entity) else {
            continue;
        };

        if let Some(cursor) = cursor.as_mut() {
            cursor.col = pos.col;
            cursor.row = pos.row;
        }
    }
}

fn confirm_with_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    cursor: Single<&BoardCursor>,
    mut actions: MessageWriter<SelectAction>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        actions.write(SelectAction {
            pos: GridPos::new(cursor.col, cursor.row),
        });
    }
}

fn confirm_with_gamepad(
    gamepads: Query<&Gamepad>,
    cursor: Single<&BoardCursor>,
    mut actions: MessageWriter<SelectAction>,
) {
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            actions.write(SelectAction {
                pos: GridPos::new(cursor.col, cursor.row),
            });
        }
    }
}

fn confirm_with_mouse(
    mut messages: MessageReader<Pointer<Click>>,
    gems: Query<&GridPos, With<GemType>>,
    mut actions: MessageWriter<SelectAction>,
) {
    for message in messages.read() {
        let Ok(&pos) = gems.get(message.entity) else {
            continue;
        };

        actions.write(SelectAction { pos });
    }
}

fn return_to_main_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(ScreenState::MainMenu);
    }
}
