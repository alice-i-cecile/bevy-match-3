//! Board cursor component and visual representation.
//!
//! This module owns the keyboard/gamepad cursor position ([`BoardCursor`]) and
//! the sprite that mirrors it in world space.

use bevy::prelude::*;

use crate::GameSystems;
use crate::ScreenState;
use crate::board::{GRID_COLS, GRID_ROWS, GridPos};
use crate::gems::GEM_SIZE;

/// The thickness of the cursor border in pixels.
const CURSOR_THICKNESS: f32 = 4.0;

/// The color of the cursor sprite.
const CURSOR_COLOR: Color = Color::hsla(54.0, 0.3, 0.90, 0.35);

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ScreenState::InGame), spawn_cursor)
            .add_systems(
                Update,
                sync_cursor_transform
                    .run_if(any_with_component::<BoardCursor>)
                    .run_if(in_state(ScreenState::InGame))
                    .in_set(GameSystems::AudioVisual),
            );
    }
}

/// Position of the keyboard/gamepad cursor on the board.
///
/// Stored directly on the singleton cursor entity so input and visuals read
/// and write the same source of truth.
#[derive(Component, Debug)]
#[require(Sprite, DespawnOnExit::<ScreenState>(ScreenState::InGame))]
pub struct BoardCursor {
    pub col: usize,
    pub row: usize,
}

impl Default for BoardCursor {
    fn default() -> Self {
        Self {
            col: GRID_COLS / 2,
            row: GRID_ROWS / 2,
        }
    }
}

/// Spawns the singleton cursor entity for a new run.
fn spawn_cursor(mut commands: Commands) {
    let board_cursor = BoardCursor::default();

    let initial_pos = GridPos::new(board_cursor.col, board_cursor.row)
        .to_world()
        .with_z(-0.5);
    commands.spawn((
        board_cursor,
        // These components are already implied by the required components on [`BoardCursor`],
        // but we need to set them up with the right data for the cursor sprite.
        Sprite::from_color(CURSOR_COLOR, Vec2::splat(GEM_SIZE + CURSOR_THICKNESS * 2.0)),
        // Transform is transitively required by Sprite!
        Transform::from_translation(initial_pos),
    ));
}

/// Moves the cursor sprite to track the [`BoardCursor`] component.
fn sync_cursor_transform(cursor: Single<(&BoardCursor, Ref<BoardCursor>, &mut Transform)>) {
    let (cursor, cursor_ref, mut transform) = cursor.into_inner();
    if !cursor_ref.is_changed() {
        return;
    }
    transform.translation = GridPos::new(cursor.col, cursor.row).to_world().with_z(-0.5);
}
