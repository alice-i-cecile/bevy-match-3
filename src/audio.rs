//! Sound effect playback.
//!
//! Reads [`SoundEffect`] messages and spawns self-despawning audio entities.

use crate::GameSystems;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SoundEffect>()
            .add_systems(Startup, load_audio_handles)
            .add_systems(Update, play_audio_cues.in_set(GameSystems::AudioVisual));
    }
}

/// The distinct audio events the game can trigger.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    Select,
    Swap,
    Match,
    Invalid,
}

impl SoundEffect {
    pub const ALL: [SoundEffect; 4] = [
        SoundEffect::Select,
        SoundEffect::Swap,
        SoundEffect::Match,
        SoundEffect::Invalid,
    ];

    pub fn asset_path(self) -> &'static str {
        match self {
            SoundEffect::Select => "audio/select.ogg",
            SoundEffect::Swap => "audio/swap.ogg",
            SoundEffect::Match => "audio/match.ogg",
            SoundEffect::Invalid => "audio/invalid.ogg",
        }
    }
}

/// Pre-loaded handles for each sound effect.
#[derive(Resource)]
pub struct AudioHandles(HashMap<SoundEffect, Handle<AudioSource>>);

fn load_audio_handles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = SoundEffect::ALL
        .iter()
        .map(|&effect| (effect, asset_server.load(effect.asset_path())))
        .collect();
    commands.insert_resource(AudioHandles(handles));
}

/// Spawns a one-shot audio entity for each [`SoundEffect`] message.
fn play_audio_cues(
    mut effects: MessageReader<SoundEffect>,
    handles: Res<AudioHandles>,
    mut commands: Commands,
) {
    for effect in effects.read() {
        commands.spawn((
            AudioPlayer::new(handles.0[&effect].clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}
