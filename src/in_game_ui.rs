//! Heads-up display: score counter and game-state label for in-game UI.

use bevy::prelude::*;

use crate::GameSystems;
use crate::ScreenState;
use crate::game_logic::Score;

pub struct InGameUiPlugin;

impl Plugin for InGameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ScreenState::InGame), setup_in_game_ui)
            .add_systems(
                Update,
                update_score_text
                    .run_if(resource_changed::<Score>)
                    .in_set(GameSystems::AudioVisual),
            );
    }
}

const SCORE_FONT_SIZE: f32 = 28.0;
const SCORE_TOP: f32 = 14.0;
const SCORE_LEFT: f32 = 16.0;

#[derive(Component)]
struct ScoreText;

fn setup_in_game_ui(mut commands: Commands) {
    // Score — top-left corner.
    commands.spawn((
        ScoreText,
        Text::new(Score::default().to_string()),
        TextFont {
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(SCORE_TOP),
            left: Val::Px(SCORE_LEFT),
            ..default()
        },
        DespawnOnExit(ScreenState::InGame),
    ));
}

fn update_score_text(score: Res<Score>, mut text: Single<&mut Text, With<ScoreText>>) {
    text.0 = score.to_string();
}
