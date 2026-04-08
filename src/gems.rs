//! Gem types, display colours, random generation, and visual sync systems.

use crate::GameSystems;
use crate::ScreenState;
use crate::board::GridPos;
use bevy::math::StableInterpolate;
use bevy::prelude::*;
use rand::{RngExt, rng};

pub const GEM_SIZE: f32 = 60.0;

/// The variant of a gem on the board.
///
/// Used as both a **component** (every gem entity carries one) and a
/// plain value type inside [`crate::board::Grid`] for fast match detection.
///
/// # About required components
///
/// The `#[require(...)]` attribute tells Bevy to automatically insert the
/// listed components whenever a `GemType` is added to an entity.  This is why
/// spawning `(gem_type, pos)` in [`crate::board::setup_board`] is enough to
/// produce a visible, pickable sprite that cleans itself up — [`Sprite`], [`GridPos`],
/// [`Pickable`], and [`DespawnOnExit`] are all inserted for us.
///
/// [`Pickable`] opts the sprite into Bevy's picking/hit-testing system so
/// that pointer events like [`Pointer<Click>`](bevy::picking::events::Click)
/// are emitted when the player clicks on a gem.
///
/// [`DespawnOnExit`] tells Bevy to automatically despawn the entity when the game leaves [`crate::ScreenState::InGame`].
/// Without this, gem entities would linger in the world across sessions.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[require(Sprite, GridPos, Pickable, DespawnOnExit::<ScreenState>(ScreenState::InGame))]
pub enum GemType {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    Orange,
    Pink,
}

impl GemType {
    pub const ALL: [GemType; 7] = [
        GemType::Red,
        GemType::Blue,
        GemType::Green,
        GemType::Yellow,
        GemType::Purple,
        GemType::Orange,
        GemType::Pink,
    ];

    pub fn color(self) -> Color {
        match self {
            GemType::Red => Color::hsl(0.0, 0.75, 0.56),
            GemType::Blue => Color::hsl(224.0, 0.72, 0.56),
            GemType::Green => Color::hsl(133.0, 0.63, 0.45),
            GemType::Yellow => Color::hsl(54.0, 0.78, 0.54),
            GemType::Purple => Color::hsl(276.0, 0.72, 0.56),
            GemType::Orange => Color::hsl(27.0, 0.90, 0.52),
            GemType::Pink => Color::hsl(348.0, 0.82, 0.68),
        }
    }

    /// A brightened version of the colour used when the gem is selected.
    pub fn selected_color(self) -> Color {
        self.color().lighter(0.15)
    }

    pub fn random() -> Self {
        Self::ALL[rng().random_range(0..Self::ALL.len())]
    }

    /// Returns a random gem type that would **not** immediately create a run
    /// of three when placed at `(col, row)`.  Used during board initialisation.
    pub fn random_no_match(
        col: usize,
        row: usize,
        kinds: &[[Option<GemType>; crate::board::GRID_COLS]; crate::board::GRID_ROWS],
    ) -> Self {
        let mut rng = rng();

        // Keep drawing a random gem until we find one that won't form a match-3
        // right away. Even in the worst case, at least one colour remains
        // legal, so this terminates quickly.
        loop {
            let candidate = Self::ALL[rng.random_range(0..Self::ALL.len())];

            // Would placing `candidate` here complete a horizontal run of 3?
            // We only look left: the board is filled left-to-right.
            if col >= 2
                && kinds[row][col - 1] == Some(candidate)
                && kinds[row][col - 2] == Some(candidate)
            {
                continue; // horizontal match — try again
            }

            // Same vertically: only look upward.
            if row >= 2
                && kinds[row - 1][col] == Some(candidate)
                && kinds[row - 2][col] == Some(candidate)
            {
                continue; // vertical match — try again
            }

            return candidate;
        }
    }
}

/// Exponential-decay rate for [`smooth_nudge`](StableInterpolate::smooth_nudge).
const FALL_DECAY_RATE: f32 = 12.0;
const FALL_SNAP_THRESHOLD: f32 = 0.05;

/// Marker component for gems animating toward their [`GridPos`] world position.
///
/// This is a common Bevy pattern: a **marker component** (with no data) restricts a
/// query so that a system only processes the entities it cares about.
/// [`animate_falling_gems`] queries `With<Falling>` to find gems mid-fall
/// and uses [`StableInterpolate::smooth_nudge`] for framerate-independent
/// exponential decay toward the target position.
///
/// Once the gem arrives, the marker is removed and the system stops animating it.
#[derive(Component)]
pub struct Falling;

pub struct GemsPlugin;

impl Plugin for GemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                snap_new_gem_transforms,
                animate_falling_gems.run_if(in_state(ScreenState::InGame)),
                sync_gem_colors,
            )
                .in_set(GameSystems::AudioVisual),
        );
    }
}

/// Snaps newly-spawned gems (without [`Falling`]) to their grid position.
/// Initial board setup goes through here; cascade refills use [`Falling`].
fn snap_new_gem_transforms(
    mut query: Query<(&GridPos, &mut Transform), (With<GemType>, Added<GridPos>, Without<Falling>)>,
) {
    for (pos, mut transform) in &mut query {
        transform.translation = pos.to_world();
    }
}

/// Nudges every [`Falling`] gem toward its target using
/// [`StableInterpolate::smooth_nudge`].  Snaps and removes the marker
/// when close enough.
fn animate_falling_gems(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &GridPos, &mut Transform), (With<GemType>, With<Falling>)>,
) {
    let dt = time.delta_secs();
    for (entity, pos, mut transform) in &mut query {
        let target = pos.to_world();
        transform
            .translation
            .smooth_nudge(&target, FALL_DECAY_RATE, dt);

        if transform.translation.distance(target) < FALL_SNAP_THRESHOLD {
            transform.translation = target;
            commands.entity(entity).remove::<Falling>();
        }
    }
}

/// Sets sprite colour and size for newly-spawned gems.
fn sync_gem_colors(mut query: Query<(&GemType, &mut Sprite), Added<GemType>>) {
    for (gem_type, mut sprite) in &mut query {
        *sprite = Sprite::from_color(gem_type.color(), Vec2::splat(GEM_SIZE));
    }
}
