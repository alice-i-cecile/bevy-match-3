//! Core game logic: swap resolution, match detection, cascade, gravity,
//! refill, and scoring.
//!
//! # Gameplay Flow
//!
//! ```text
//! (Playing state)
//!   input → SwapMessage
//!   process_swap reads SwapMessage
//!     ├─ no match  → revert swap, play Invalid cue, stay in Playing
//!     └─ match     → play Swap cue, transition to Animating
//!
//! (Animating state — waits for fall animations to settle)
//!   process_cascade waits for all Falling markers to clear, then:
//!     ├─ no matches in grid.kinds
//!     │   ├─ valid move exists → transition back to Playing
//!     │   └─ no valid moves    → transition to GameOver
//!     └─ matches found
//!         1. Despawn matched gem entities
//!         2. Award points, play Match cue
//!         3. Apply gravity  (compact each column downward)
//!         4. Refill top of each column with new random gems
//!         5. Stay in Animating — new Falling gems must settle before next check
//! ```

use crate::GameState;
use crate::GameSystems;
use crate::ScreenState;
use crate::audio::SoundEffect;
use crate::board::{GEM_STEP, GRID_COLS, GRID_ROWS, Grid, GridPos};
use crate::gems::{Falling, GemType};
use crate::selection::SwapMessage;
use bevy::prelude::*;
use std::fmt;

/// Points awarded per cleared gem.
const POINTS_PER_CLEARED_GEM: u32 = 10;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(OnEnter(ScreenState::InGame), reset_score)
            .add_systems(
                Update,
                (
                    process_swap.run_if(in_state(GameState::Playing)),
                    process_cascade.run_if(in_state(GameState::Animating)),
                    sync_gridpos_from_grid.run_if(in_state(ScreenState::InGame)),
                )
                    .chain()
                    .in_set(GameSystems::Logic),
            );
    }
}

#[derive(Resource, Default, Debug)]
pub struct Score {
    pub value: u32,
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Score: {}", self.value)
    }
}

fn reset_score(mut score: ResMut<Score>) {
    score.value = 0;
}

/// Reads [`SwapMessage`]s from input systems and attempts each swap.
///
/// After swapping the two cells in the grid, the board is scanned for matches:
/// - **Match found** → keep the swap, emit a `Swap` audio cue, and transition
///   to [`GameState::Animating`] so [`process_cascade`] can finish the job.
/// - **No match** → revert the swap (grid data only — the entities' `GridPos`
///   components were never changed) and emit an `Invalid` audio cue.
fn process_swap(
    mut swap_messages: MessageReader<SwapMessage>,
    mut grid: ResMut<Grid>,
    mut audio: MessageWriter<SoundEffect>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for msg in swap_messages.read() {
        let (pa, pb) = (msg.pos_a, msg.pos_b);

        grid.swap_cells(pa, pb);

        if !grid.swap_creates_match(pa, pb) {
            // Revert the swap in the authoritative grid data.
            // sync_gridpos_from_grid will reconcile entity GridPos
            // components from Grid at the end of the logic step.
            grid.swap_cells(pa, pb);
            audio.write(SoundEffect::Invalid);
        } else {
            audio.write(SoundEffect::Swap);
            next_state.set(GameState::Animating);
        }
    }
}

/// Resolves one cascade step while in [`GameState::Animating`].
///
/// Waits until all [`Falling`] animations have settled, then:
/// 1. Looks for matches in the kinds array (updated synchronously — no entity
///    query needed).
/// 2. If none remain, transitions back to [`GameState::Playing`].
/// 3. If matches are found: clears them, applies gravity, refills, and
///    stays in `Animating` for the next cascade check once the new gems land.
fn process_cascade(
    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut audio: MessageWriter<SoundEffect>,
    mut next_state: ResMut<NextState<GameState>>,
    falling: Query<(), With<Falling>>,
) {
    // Wait for all fall animations to finish before processing the next
    // cascade step, so the player can see each round of clears.
    if !falling.is_empty() {
        return;
    }

    let matched = grid.find_all_matches();

    if matched.is_empty() {
        if grid.has_any_valid_move() {
            next_state.set(GameState::Playing);
        } else {
            next_state.set(GameState::GameOver);
        }
        return;
    }

    for &pos in &matched {
        if let Some(entity) = grid.clear_cell(pos) {
            commands.entity(entity).despawn();
        }
    }

    score.value = score
        .value
        .saturating_add(matched.len() as u32 * POINTS_PER_CLEARED_GEM);
    audio.write(SoundEffect::Match);

    apply_gravity_and_refill(&mut grid, &mut commands);
}

/// Compacts surviving gems toward the bottom of each column and fills the
/// vacated top cells with new [`Falling`] gems spawned above the board.
fn apply_gravity_and_refill(grid: &mut Grid, commands: &mut Commands) {
    for col in 0..GRID_COLS {
        // Collect surviving gems bottom-to-top.
        let survivors: Vec<(Entity, GemType)> = (0..GRID_ROWS)
            .rev()
            .filter_map(|row| {
                let pos = GridPos::new(col, row);
                grid.kind_at(pos).map(|gem_type| {
                    let entity = grid
                        .entity_at(pos)
                        .expect("entity must exist when grid kind is Some");
                    (entity, gem_type)
                })
            })
            .collect();

        // Clear the whole column before repacking.
        for row in 0..GRID_ROWS {
            let _ = grid.clear_cell(GridPos::new(col, row));
        }

        for (i, &(entity, gem_type)) in survivors.iter().enumerate() {
            let row = GRID_ROWS - 1 - i;
            let pos = GridPos::new(col, row);
            grid.set_cell(pos, entity, gem_type);
        }

        let empty_rows = GRID_ROWS - survivors.len();
        for row in 0..empty_rows {
            // GemType::random() rather than random_no_match() is intentional:
            // newly fallen gems can immediately form matches, which is what
            // drives cascade chains.
            let gem_type = GemType::random();
            let pos = GridPos::new(col, row);
            // Spawn the gem above the visible board so it falls into place.
            // Each new gem is staggered one additional step higher so a
            // column with several empties looks like a stream pouring in.
            let target = pos.to_world();
            let above = Vec3::new(
                target.x,
                target.y + (empty_rows - row) as f32 * GEM_STEP,
                0.0,
            );
            let entity = commands
                .spawn((
                    gem_type,
                    pos,
                    Transform::from_translation(above),
                    Falling,
                ))
                .id();
            grid.set_cell(pos, entity, gem_type);
        }
    }
}

/// Reconciles entity [`GridPos`] components from the authoritative [`Grid`].
fn sync_gridpos_from_grid(
    mut commands: Commands,
    grid: Res<Grid>,
    mut gem_positions: Query<&mut GridPos, With<GemType>>,
) {
    for (pos, entity) in grid.occupied_cells() {
        match gem_positions.get_mut(entity) {
            Ok(mut grid_pos) => {
                if *grid_pos != pos {
                    *grid_pos = pos;
                    // The gem moved to a new cell — add Falling so the
                    // visual position lerps smoothly instead of snapping.
                    commands.entity(entity).insert(Falling);
                }
            }
            Err(_) => {
                warn!(
                    "Grid references entity {entity:?} at ({}, {}) but it has no GridPos component",
                    pos.col, pos.row
                );
            }
        }
    }
}
