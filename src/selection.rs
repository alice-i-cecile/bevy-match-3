//! Player selection state and swap message emission.
//!
//! Tracks which gem (if any) the player has picked as the first of a swap pair.
//! [`crate::input::SelectAction`] flows in; [`SwapMessage`] flows out when two
//! adjacent gems are confirmed.

use bevy::prelude::*;

use crate::GameState;
use crate::GameSystems;
use crate::ScreenState;
use crate::audio::SoundEffect;
use crate::board::{Grid, GridPos};
use crate::gems::GemType;
use crate::input::SelectAction;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SwapMessage>()
            .init_resource::<Selection>()
            .add_systems(
                OnEnter(ScreenState::InGame),
                reset_selection_on_enter_in_game,
            )
            .add_systems(
                Update,
                process_selection
                    .run_if(in_state(GameState::Playing))
                    .in_set(GameSystems::Logic),
            )
            .add_systems(
                Update,
                highlight_selected_gem
                    .run_if(resource_changed::<Selection>)
                    .in_set(GameSystems::AudioVisual),
            );
    }
}

/// Sent when a valid adjacent swap is confirmed.
#[derive(Message, Debug, Clone, Copy)]
pub struct SwapMessage {
    pub pos_a: GridPos,
    pub pos_b: GridPos,
}

/// Reads [`SelectAction`]s and applies selection/swap logic.
///
/// - Nothing selected → select this gem, play Select cue.
/// - Same gem confirmed again → deselect.
/// - Adjacent gem confirmed → emit [`SwapMessage`], clear selection.
/// - Non-adjacent gem confirmed → move selection here.
pub fn process_selection(
    mut actions: MessageReader<SelectAction>,
    grid: Res<Grid>,
    mut selection: ResMut<Selection>,
    mut swap_messages: MessageWriter<SwapMessage>,
    mut audio: MessageWriter<SoundEffect>,
) {
    for action in actions.read() {
        let pos = action.pos;

        // Only act if there is actually a gem at this cell.
        if grid.entity_at(pos).is_none() {
            continue;
        }

        match selection.pos() {
            None => {
                selection.select(pos);
                audio.write(SoundEffect::Select);
            }
            Some(prev) if prev == pos => {
                selection.clear();
            }
            Some(prev) if prev.is_adjacent(pos) => {
                swap_messages.write(SwapMessage {
                    pos_a: prev,
                    pos_b: pos,
                });
                selection.clear();
            }
            Some(_) => {
                selection.select(pos);
                audio.write(SoundEffect::Select);
            }
        }
    }
}

/// Which gem the player has selected as the first of a swap pair.
#[derive(Resource, Default, Debug)]
pub struct Selection(Option<GridPos>);

impl Selection {
    pub fn select(&mut self, pos: GridPos) {
        self.0 = Some(pos);
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn pos(&self) -> Option<GridPos> {
        self.0
    }
}

fn reset_selection_on_enter_in_game(mut selection: ResMut<Selection>) {
    selection.clear();
}

/// Brightens the currently selected gem so the player can see their pick.
///
/// Iterates all gems to reset the previously-highlighted one.  Caching the
/// old entity would save work, but at 64 gems it's not worth the extra state.
fn highlight_selected_gem(
    selection: Res<Selection>,
    grid: Res<Grid>,
    mut gems: Query<(Entity, &GemType, &mut Sprite)>,
) {
    let selected_entity = selection.pos().and_then(|pos| grid.entity_at(pos));
    for (entity, gem_type, mut sprite) in &mut gems {
        sprite.color = if selected_entity == Some(entity) {
            gem_type.selected_color()
        } else {
            gem_type.color()
        };
    }
}
