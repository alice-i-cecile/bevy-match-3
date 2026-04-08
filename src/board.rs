//! The game board: grid data and entity management.
//!
//! - [`Grid`] — resource holding entity handles and gem kinds for every cell.
//! - [`GridPos`] — component on every gem entity giving its (col, row) address.

use crate::ScreenState;
use crate::gems::{GEM_SIZE, GemType};
use bevy::prelude::*;
use std::collections::HashSet;

pub const GRID_COLS: usize = 8;
pub const GRID_ROWS: usize = 8;

pub const GEM_GAP: f32 = 16.0;
pub const GEM_STEP: f32 = GEM_SIZE + GEM_GAP;

/// A gem's address on the board grid.
///
/// `col` increases to the right; `row` increases **downward** (row 0 is the
/// top).
///
/// As a component, this mirrors each gem's position for ECS queries and visual
/// sync; [`crate::gems::GemsPlugin`] keeps [`Transform`] in sync with it.
///
/// The authoritative logical board state lives in [`Grid`].
/// Whenever board logic mutates [`Grid`], matching systems also
/// update each gem entity's [`GridPos`] so both views stay aligned.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[require(Transform)]
pub struct GridPos {
    pub col: usize,
    pub row: usize,
}

impl GridPos {
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }

    pub fn is_adjacent(self, other: GridPos) -> bool {
        let col_diff = self.col.abs_diff(other.col);
        let row_diff = self.row.abs_diff(other.row);
        (col_diff == 1 && row_diff == 0) || (col_diff == 0 && row_diff == 1)
    }

    /// The board is centred at the world origin.  Row 0 is at the top
    /// (positive Y) and row `GRID_ROWS - 1` at the bottom (negative Y).
    pub fn to_world(self) -> Vec3 {
        let half_width = (GRID_COLS as f32 * GEM_STEP) / 2.0;
        let half_height = (GRID_ROWS as f32 * GEM_STEP) / 2.0;
        Vec3::new(
            self.col as f32 * GEM_STEP - half_width + GEM_STEP / 2.0,
            -(self.row as f32 * GEM_STEP - half_height + GEM_STEP / 2.0),
            0.0,
        )
    }
}

/// The full game board.
///
/// Maintains two parallel arrays so that game-logic systems (e.g. match
/// detection, gravity) can work directly on data without going through ECS
/// queries.
///
/// This resource is the source of truth for logical board state.
///
/// Both arrays are **private** to prevent them from being updated
/// independently, which would leave the board in an inconsistent state.
/// Use the provided methods to read and write:
#[derive(Resource)]
pub struct Grid {
    entities: [[Option<Entity>; GRID_COLS]; GRID_ROWS],
    kinds: [[Option<GemType>; GRID_COLS]; GRID_ROWS],
}

impl Grid {
    pub fn empty() -> Self {
        Self {
            entities: [[None; GRID_COLS]; GRID_ROWS],
            kinds: [[None; GRID_COLS]; GRID_ROWS],
        }
    }

    pub fn entity_at(&self, pos: GridPos) -> Option<Entity> {
        self.entities[pos.row][pos.col]
    }

    pub fn kind_at(&self, pos: GridPos) -> Option<GemType> {
        self.kinds[pos.row][pos.col]
    }

    pub fn set_cell(&mut self, pos: GridPos, entity: Entity, kind: GemType) {
        self.entities[pos.row][pos.col] = Some(entity);
        self.kinds[pos.row][pos.col] = Some(kind);
    }

    /// Clears both arrays at `pos` and returns the entity that was there.
    pub fn clear_cell(&mut self, pos: GridPos) -> Option<Entity> {
        self.kinds[pos.row][pos.col] = None;
        self.entities[pos.row][pos.col].take()
    }

    pub fn occupied_cells(&self) -> impl Iterator<Item = (GridPos, Entity)> + '_ {
        (0..GRID_ROWS).flat_map(move |row| {
            (0..GRID_COLS).filter_map(move |col| {
                self.entities[row][col].map(|entity| (GridPos::new(col, row), entity))
            })
        })
    }

    /// Swaps the contents of two cells in both arrays.
    pub fn swap_cells(&mut self, a: GridPos, b: GridPos) {
        // Rust won't allow two mutable borrows of the same slice, so we
        // read-then-write using temporaries.
        let (ea, eb) = (self.entities[a.row][a.col], self.entities[b.row][b.col]);
        self.entities[a.row][a.col] = eb;
        self.entities[b.row][b.col] = ea;

        let (ka, kb) = (self.kinds[a.row][a.col], self.kinds[b.row][b.col]);
        self.kinds[a.row][a.col] = kb;
        self.kinds[b.row][b.col] = ka;
    }

    /// Returns `true` if swapping `pa` and `pb` created at least one match.
    pub fn swap_creates_match(&self, pa: GridPos, pb: GridPos) -> bool {
        self.pos_in_match(pa) || self.pos_in_match(pb)
    }

    /// Returns `true` if `pos` is part of a horizontal or vertical run of 3+,
    /// reading gem kinds from the actual board state.
    fn pos_in_match(&self, pos: GridPos) -> bool {
        let Some(kind) = self.kinds[pos.row][pos.col] else {
            return false;
        };
        let kinds = &self.kinds;
        Self::pos_in_match_impl(pos, kind, |r, c| kinds[r][c])
    }

    /// Core run-detection logic.
    ///
    /// Checks whether `pos`, treated as holding `kind`, belongs to a horizontal
    /// or vertical run of 3+.  Neighbouring cells are looked up via `kind_at`,
    /// which lets callers substitute a different kind for one cell without
    /// mutating the board (used by [`Self::has_any_valid_move`]).
    fn pos_in_match_impl(
        pos: GridPos,
        kind: GemType,
        kind_at: impl Fn(usize, usize) -> Option<GemType>,
    ) -> bool {
        // Horizontal run through pos.col
        let mut run = 1;
        let mut c = pos.col;
        while c > 0 && kind_at(pos.row, c - 1) == Some(kind) {
            c -= 1;
            run += 1;
        }
        c = pos.col;
        while c + 1 < GRID_COLS && kind_at(pos.row, c + 1) == Some(kind) {
            c += 1;
            run += 1;
        }
        if run >= 3 {
            return true;
        }

        // Vertical run through pos.row
        run = 1;
        let mut r = pos.row;
        while r > 0 && kind_at(r - 1, pos.col) == Some(kind) {
            r -= 1;
            run += 1;
        }
        r = pos.row;
        while r + 1 < GRID_ROWS && kind_at(r + 1, pos.col) == Some(kind) {
            r += 1;
            run += 1;
        }
        run >= 3
    }

    /// Returns `true` if there is at least one swap on the board that would
    /// create a match of 3+.
    ///
    /// Every adjacent pair (horizontal and vertical) is tested by checking
    /// whether either cell would form a run of 3+ under its post-swap kind,
    /// without mutating the board.  O(GRID_COLS × GRID_ROWS).
    pub fn has_any_valid_move(&self) -> bool {
        let kinds = &self.kinds;
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let pa = GridPos::new(col, row);
                let Some(kind_a) = kinds[pa.row][pa.col] else {
                    continue;
                };

                // Horizontal neighbour
                if col + 1 < GRID_COLS {
                    let pb = GridPos::new(col + 1, row);
                    if let Some(kind_b) = kinds[pb.row][pb.col] {
                        // After swapping pa↔pb: pa gets kind_b, pb gets kind_a.
                        // Pass the other cell's post-swap kind as an override so
                        // the run check reads the hypothetical board correctly.
                        if Self::pos_in_match_impl(pa, kind_b, |r, c| {
                            if r == pb.row && c == pb.col {
                                Some(kind_a)
                            } else {
                                kinds[r][c]
                            }
                        }) || Self::pos_in_match_impl(pb, kind_a, |r, c| {
                            if r == pa.row && c == pa.col {
                                Some(kind_b)
                            } else {
                                kinds[r][c]
                            }
                        }) {
                            return true;
                        }
                    }
                }

                // Vertical neighbour
                if row + 1 < GRID_ROWS {
                    let pb = GridPos::new(col, row + 1);
                    if let Some(kind_b) = kinds[pb.row][pb.col] {
                        if Self::pos_in_match_impl(pa, kind_b, |r, c| {
                            if r == pb.row && c == pb.col {
                                Some(kind_a)
                            } else {
                                kinds[r][c]
                            }
                        }) || Self::pos_in_match_impl(pb, kind_a, |r, c| {
                            if r == pa.row && c == pa.col {
                                Some(kind_b)
                            } else {
                                kinds[r][c]
                            }
                        }) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Returns the set of all grid positions that are part of a match of 3+.
    pub fn find_all_matches(&self) -> HashSet<GridPos> {
        let mut matched = HashSet::new();
        let kinds = &self.kinds;

        for row in 0..GRID_ROWS {
            for col in find_run_indices(&kinds[row]) {
                matched.insert(GridPos::new(col, row));
            }
        }

        for col in 0..GRID_COLS {
            let column: [Option<GemType>; GRID_ROWS] = std::array::from_fn(|row| kinds[row][col]);
            for row in find_run_indices(&column) {
                matched.insert(GridPos::new(col, row));
            }
        }

        matched
    }
}

/// Returns the indices within `line` that belong to a run of 3+ identical
/// non-empty kinds.
fn find_run_indices(line: &[Option<GemType>]) -> Vec<usize> {
    let mut result = Vec::new();
    let mut run_start = 0;

    // Iterate one past the end so the final run is always flushed.
    for i in 1..=line.len() {
        let prev = line[i - 1];
        let cur = line.get(i).copied().flatten();

        if cur != prev || prev.is_none() {
            if prev.is_some() && i - run_start >= 3 {
                result.extend(run_start..i);
            }
            run_start = i;
        }
    }

    result
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid::empty())
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(ScreenState::InGame), setup_board);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawns the initial gems and registers them in the [`Grid`].
///
/// Registered on [`OnEnter(ScreenState::InGame)`] so it runs exactly once per
/// new game — never on [`crate::GameState::Animating`] ↔ [`crate::GameState::Playing`]
/// round-trips during a cascade. Entities carry [`DespawnOnExit`] so they are
/// cleaned up automatically when gameplay is exited.
///
/// The board is regenerated (extremely rarely) if the random layout happens to
/// have no valid swap — a condition that [`process_cascade`] would never be able
/// to report because it is only reached from [`crate::GameState::Animating`].
fn setup_board(mut commands: Commands, mut grid: ResMut<Grid>) {
    loop {
        // Reset grid data so stale entity handles from the previous attempt
        // (or previous session) don't linger.
        *grid = Grid::empty();

        // Build the kind array incrementally so random_no_match can look back
        // at already-placed gems.
        let mut kind_buf = [[None::<GemType>; GRID_COLS]; GRID_ROWS];
        let mut spawned: Vec<Entity> = Vec::with_capacity(GRID_COLS * GRID_ROWS);

        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let gem_type = GemType::random_no_match(col, row, &kind_buf);
                kind_buf[row][col] = Some(gem_type);

                let pos = GridPos::new(col, row);
                let entity = commands.spawn((gem_type, pos)).id();
                grid.set_cell(pos, entity, gem_type);
                spawned.push(entity);
            }
        }

        if grid.has_any_valid_move() {
            break;
        }

        // No valid swap on this layout — despawn the batch and try again.
        // Spawn + despawn in the same command queue nets to no entity in the world.
        for entity in spawned {
            commands.entity(entity).despawn();
        }
    }
}
