//! The console's sounds (the counter-UAS simulator's recordings, `assets/sounds/station_*`):
//! the shelter's hum, the camera gimbal's motor while it slews (louder as it speeds up), a
//! Patriot launch from the battery (`launch.flac`), a Gepard burst, a launcher's refill
//! starting and done, and the explosions around (`sfx_05_explosion.flac`: a drone's warhead,
//! a wreck coming down, a kill), quieter further off and arriving at the speed of sound.

use bevy::audio::{AudioSink, AudioSinkPlayback, PlaybackSettings};
use bevy::prelude::*;

use super::{ConsoleEo, Manning};
use crate::combat::Combat;
use crate::sound::{Channel, Mixed};

/// Volumes: the hum, the gimbal at full slew, a launch, a burst, a refill, an explosion within
/// `BLAST_REF` m.
const HUM: f32 = 0.25;
const SLEW: f32 = 0.35;
const LAUNCH: f32 = 0.7;
const BURST: f32 = 0.8;
const RELOAD: f32 = 0.5;
const BLAST: f32 = 1.0;
/// An explosion is heard at full volume within this distance (m), then falling off with 1 /
/// distance, up to `BLAST_RANGE`; it arrives at the speed of sound (m/s).
const BLAST_REF: f64 = 300.0;
const BLAST_RANGE: f64 = 8000.0;
const SPEED_OF_SOUND: f64 = 340.0;

#[derive(Resource)]
pub(super) struct ConsoleSounds {
    hum: Handle<AudioSource>,
    slew: Handle<AudioSource>,
    launch: Handle<AudioSource>,
    burst: Handle<AudioSource>,
    reload: Handle<AudioSource>,
    blast: Handle<AudioSource>,
}

pub(super) fn load(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ConsoleSounds {
        hum: server.load("sounds/station_hum.flac"),
        slew: server.load("sounds/station_slew.flac"),
        launch: server.load("sounds/launch.flac"),
        burst: server.load("sounds/station_burst.flac"),
        reload: server.load("sounds/station_reload.flac"),
        blast: server.load("sounds/sfx_05_explosion.flac"),
    });
}

/// The loops playing while the console is manned.
#[derive(Component)]
pub(super) struct ConsoleLoop {
    slew: bool,
}

/// What was last heard: the battery's missiles in the air, the Gepard firing, the units
/// refilling.
#[derive(Default)]
pub(super) struct Heard {
    missiles: usize,
    firing: bool,
    refilling: Vec<bool>,
    /// Explosions on their way: seconds until heard, volume.
    blasts: Vec<(f32, f32)>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn play(
    mut commands: Commands,
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    eo: Option<Res<ConsoleEo>>,
    sounds: Option<Res<ConsoleSounds>>,
    settings: Res<crate::settings::Settings>,
    time: Res<Time<Real>>,
    mut loops: Query<(Entity, &ConsoleLoop, Option<&mut AudioSink>)>,
    mut heard: Local<Heard>,
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
    let firing = m.gepard() && sim.units().get(m.unit).is_some_and(|u| u.burst > 0);
    if firing && !heard.firing {
        one_shot(&sounds.burst, BURST);
    }
    heard.firing = firing;
    // A refill starting or done at the battery's launchers (the Gepard).
    let units: Vec<usize> = (0..sim.units().len())
        .filter(|&k| k == m.unit || (!m.gepard() && sim.launcher_radar(k) == Some(m.unit)))
        .collect();
    let now: Vec<bool> = units.iter().map(|&k| sim.refill_left(k) > 0.0).collect();
    if heard.refilling.len() == now.len() && heard.refilling != now {
        one_shot(&sounds.reload, RELOAD);
    }
    heard.refilling = now;
    // The explosions of this frame (not a gun round's hit or a flak burst), on their way here.
    if let Some(u) = sim.units().get(m.unit) {
        for &(x, y, _, kind) in &combat.explosions {
            if kind == -1 || kind == crate::explosions::FLAK {
                continue;
            }
            let d = ((x - u.x) as f64).hypot((y - u.y) as f64) * stealth_sim::aaa::M_PER_MAP;
            if d < BLAST_RANGE {
                let v = BLAST * (BLAST_REF / d.max(BLAST_REF)) as f32;
                heard.blasts.push(((d / SPEED_OF_SOUND) as f32, v));
            }
        }
    }
    let dt = time.delta_secs();
    for b in heard.blasts.iter_mut() {
        b.0 -= dt;
    }
    for &(_, v) in heard.blasts.iter().filter(|b| b.0 <= 0.0) {
        one_shot(&sounds.blast, v);
    }
    heard.blasts.retain(|b| b.0 > 0.0);
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
