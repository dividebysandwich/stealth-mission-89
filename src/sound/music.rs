//! The main menu's music: looped from the menu's start, faded out over `FADE` seconds once the
//! menu is left (a flight, a campaign, a multiplayer game), under the master and music volumes.

use bevy::audio::{AudioSink, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::settings::Settings;

const TRACK: &str = "music/menu01.ogg";
/// The fade-out (s).
const FADE: f32 = 3.0;

/// The menu music playing: its fade (1 until the menu is left, then down to 0).
#[derive(Component)]
pub(super) struct MenuMusic {
    level: f32,
    fading: bool,
}

/// Into the menu: the music starts again (replacing one still fading out).
pub(super) fn start(
    mut commands: Commands,
    server: Res<AssetServer>,
    settings: Res<Settings>,
    playing: Query<Entity, With<MenuMusic>>,
) {
    for e in &playing {
        commands.entity(e).despawn();
    }
    let gain = settings.master_volume * settings.music_volume;
    commands.spawn((
        AudioPlayer::<AudioSource>(server.load(TRACK)),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(gain)),
        MenuMusic { level: 1.0, fading: false },
    ));
}

/// Out of the menu: the music fades out.
pub(super) fn leave(mut q: Query<&mut MenuMusic>) {
    for mut m in &mut q {
        m.fading = true;
    }
}

/// The music follows the volumes and fades out, gone when silent.
pub(super) fn follow(
    mut commands: Commands,
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    mut q: Query<(Entity, &mut MenuMusic, Option<&mut AudioSink>)>,
) {
    // A long frame (the flight loading) does not swallow the fade.
    let dt = time.delta_secs().min(0.05);
    for (e, mut m, sink) in &mut q {
        if m.fading {
            m.level -= dt / FADE;
            if m.level <= 0.0 {
                commands.entity(e).despawn();
                continue;
            }
        }
        if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(
                settings.master_volume * settings.music_volume * m.level,
            ));
        }
    }
}
