//! The console's sounds (the counter-UAS simulator's recordings, `assets/sounds/station_*`):
//! the shelter's hum, the camera gimbal's motor while it slews (louder as it speeds up), a
//! Patriot launch from the battery (`launch.flac`), a Gepard burst, a launcher's refill
//! starting and done.

use bevy::audio::{AudioSink, AudioSinkPlayback, PlaybackSettings};
use bevy::prelude::*;

use super::{ConsoleEo, Manning};
use crate::combat::Combat;
use crate::sound::{Channel, Mixed};

/// Volumes: the hum, the gimbal at full slew, a launch, a burst, a refill.
const HUM: f32 = 0.25;
const SLEW: f32 = 0.35;
const LAUNCH: f32 = 0.7;
const BURST: f32 = 0.8;
const RELOAD: f32 = 0.5;

#[derive(Resource)]
pub(super) struct ConsoleSounds {
    hum: Handle<AudioSource>,
    slew: Handle<AudioSource>,
    launch: Handle<AudioSource>,
    burst: Handle<AudioSource>,
    reload: Handle<AudioSource>,
}

pub(super) fn load(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ConsoleSounds {
        hum: server.load("sounds/station_hum.flac"),
        slew: server.load("sounds/station_slew.flac"),
        launch: server.load("sounds/launch.flac"),
        burst: server.load("sounds/station_burst.flac"),
        reload: server.load("sounds/station_reload.flac"),
    });
}

/// The loops playing while the console is manned.
#[derive(Component)]
pub(super) struct ConsoleLoop {
    slew: bool,
}

/// A Gepard firing on: its burst heard again this often (s).
const BURST_AGAIN: f32 = 1.4;

/// What was last heard: the battery's missiles in the air, the Gepard firing (for how long),
/// the units refilling.
#[derive(Default)]
pub(super) struct Heard {
    missiles: usize,
    firing: Option<f32>,
    refilling: Vec<bool>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn play(
    mut commands: Commands,
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    eo: Option<Res<ConsoleEo>>,
    sounds: Option<Res<ConsoleSounds>>,
    settings: Res<crate::settings::Settings>,
    mut loops: Query<(Entity, &ConsoleLoop, Option<&mut AudioSink>)>,
    mut heard: Local<Heard>,
    time: Res<Time<Real>>,
) {
    let (Some(m), Some(combat), Some(sounds)) = (manning, combat, sounds) else {
        for (e, _, _) in &loops {
            commands.entity(e).try_despawn();
        }
        *heard = Heard::default();
        return;
    };
    let mut one_shot = |h: &Handle<AudioSource>, v: f32| {
        let mix = Mixed(Channel::Effects, v);
        let p = mix.settings(&settings, PlaybackSettings::DESPAWN);
        commands.spawn((AudioPlayer(h.clone()), p, mix));
    };
    let sim = &combat.sim;
    // A launch from the battery, a burst starting.
    let missiles = sim.ai_missiles().iter().filter(|x| x.station == Some(m.unit as u16)).count();
    if missiles > heard.missiles {
        one_shot(&sounds.launch, LAUNCH);
    }
    heard.missiles = missiles;
    let firing = m.gepard && sim.aaa_units().get(m.unit).is_some_and(|u| u.burst > 0);
    let dt = time.delta_secs();
    heard.firing = match heard.firing {
        _ if !firing => None,
        Some(t) if t + dt < BURST_AGAIN => Some(t + dt),
        _ => {
            one_shot(&sounds.burst, BURST);
            Some(0.0)
        }
    };
    // A refill starting or done at the battery's launchers (the Gepard).
    let units: Vec<usize> = (0..sim.aaa_units().len())
        .filter(|&k| k == m.unit || (!m.gepard && sim.launcher_radar(k) == Some(m.unit)))
        .collect();
    let now: Vec<bool> = units.iter().map(|&k| sim.refill_left(k) > 0.0).collect();
    if heard.refilling.len() == now.len() && heard.refilling != now {
        one_shot(&sounds.reload, RELOAD);
    }
    heard.refilling = now;
    // The loops: started once, the gimbal's following the slew.
    if loops.is_empty() {
        for (h, slew) in [(&sounds.hum, false), (&sounds.slew, true)] {
            let mix = Mixed(Channel::Effects, if slew { 0.0 } else { HUM });
            let p = mix.settings(&settings, PlaybackSettings::LOOP);
            commands.spawn((AudioPlayer(h.clone()), p, mix, ConsoleLoop { slew }));
        }
        return;
    }
    let slew = eo.map_or(0.0, |e| e.slew);
    for (_, l, sink) in &mut loops {
        if let (true, Some(mut sink)) = (l.slew, sink) {
            sink.set_volume(bevy::audio::Volume::Linear(
                SLEW * slew * Channel::Effects.gain(&settings),
            ));
        }
    }
}
