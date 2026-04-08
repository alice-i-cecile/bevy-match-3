//! Bevy Match-3
//!
//! A fully-playable match-3 game built with Bevy 0.18.
//!
//! # Reading guide
//!
//! If this is your first time in the codebase, read the modules in this order:
//!
//! 1. **`main.rs`:** [`ScreenState`], [`GameState`], [`GameSystems`], and the overall
//!    plugin wiring. These define the core patterns of the game.
//!    Note that [`GameState`] is a *substate* of [`ScreenState`]: it only
//!    exists while [`ScreenState::InGame`] is active, and is automatically
//!    removed when the player returns to the main menu.
//!
//! 2. [`gems`]: the [`GemType`] enum, its colours, and the visual sync
//!    systems that keep sprite transforms and colours in step with the board.
//!
//! 3. [`board`]: [`Grid`] resource (the authoritative board state),
//!    [`GridPos`] and matching logic.  Understanding [`Grid`] is a prerequisite for everything else.
//!
//! 4. [`cursor`]: singleton [`BoardCursor`] entity and sprite management.
//!
//! 5. [`input`]: keyboard, mouse, and gamepad handlers.  Each produces a
//!    [`SelectAction`] message consumed by [`selection`].
//!
//! 6. [`selection`]: turns [`SelectAction`] messages into [`SwapMessage`]s.
//!    Owns selection state and selected-gem highlighting.
//!
//! 7. [`game_logic`]: swap resolution, cascade, gravity,
//!    refill, and scoring. The heart of the game.
//!
//! 8. [`audio`]: plays one-shot sound effects in response to game events.
//!
//! 9. [`in_game_ui`]: the in-game UI. Currently just a score counter.
//!
//! 10. [`menu`]: the title screen shown in [`ScreenState::MainMenu`].
//!
//! [`ScreenState`]: self::ScreenState
//! [`GameState`]: self::GameState
//! [`GameSystems`]: self::GameSystems
//! [`GemType`]: gems::GemType
//! [`Grid`]: board::Grid
//! [`GridPos`]: board::GridPos
//! [`SelectAction`]: input::SelectAction
//! [`BoardCursor`]: cursor::BoardCursor
//! [`SwapMessage`]: selection::SwapMessage

use bevy::prelude::*;
use bevy::window::WindowResolution;

mod audio;
mod board;
mod cursor;
mod game_logic;
mod gems;
mod in_game_ui;
mod input;
mod menu;
mod selection;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Match-3".into(),
                resolution: WindowResolution::new(640, 700),
                ..default()
            }),
            ..default()
        }))
        .init_state::<ScreenState>()
        .add_sub_state::<GameState>()
        // Systems are organised into three ordered sets so that input is read,
        // then logic runs, then visuals are synced — all within the same frame.
        .configure_sets(
            Update,
            (
                GameSystems::Input,
                GameSystems::Logic,
                GameSystems::AudioVisual,
            )
                .chain(),
        )
        .add_plugins((
            board::BoardPlugin,
            cursor::CursorPlugin,
            gems::GemsPlugin,
            selection::SelectionPlugin,
            input::InputPlugin,
            game_logic::GameLogicPlugin,
            audio::AudioPlugin,
            in_game_ui::InGameUiPlugin,
            menu::MenuPlugin,
        ))
        .run();
}

/// Top-level screens for the game.
///
/// While [`ScreenState::InGame`] is active,
/// a [`GameState`] substate exists and is used to drive game logic.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScreenState {
    /// Title screen shown before the game starts.
    #[default]
    MainMenu,
    /// The player is on the board or a gameplay overlay.
    InGame,
}

/// Gameplay substate that exists only while [`ScreenState::InGame`] is active.
///
/// Confirmation and swap systems run only in [`Playing`].
/// [`Animating`] is held while the board resolves a
/// cascade (clear → fall → refill), preventing the player from queuing moves
/// mid-cascade while still allowing the cursor to move.
///
/// If no valid swap remains after a cascade, the game moves to [`GameOver`]
/// instead of back to [`Playing`].
///
/// # About substates
///
/// The #[source(ScreenState = ScreenState::InGame)] attribute tells Bevy that
/// this substate is tied to a specific parent state value: GameState is automatically inserted
/// (starting at its #[default] variant) when ScreenState enters InGame,
/// and automatically removed when ScreenState leaves InGame.
///
/// [`Playing`]: GameState::Playing
/// [`Animating`]: GameState::Animating
/// [`GameOver`]: GameState::GameOver
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(ScreenState = ScreenState::InGame)]
pub enum GameState {
    #[default]
    /// The player can freely move the cursor and make swaps.
    Playing,
    /// The board is resolving matches.
    Animating,
    /// No valid moves remain; the game has ended.
    GameOver,
}

/// Broad system sets used across all plugins to control ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystems {
    Input,
    Logic,
    AudioVisual,
}
