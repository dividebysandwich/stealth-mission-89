//! The fleet's war at sea as heard, and its test aids (the sim's `naval`: the ships' Phalanx
//! mounts, the submarines' sea-skimmers, the attack boats; their models and tracers are drawn
//! by `sim_render`).
//!
//! Test aids:
//! - `--usv-swarm <n>,<km>[,<s>]`: `n` attack boats launched `km` out from the fleet's
//!   carrier `s` seconds into the flight (2 by default).
//! - `--sea-salvo <n>,<km>[,<s>]`: a salvo of `n` sea-skimmers fired from `km` out at it.
//! - `--sub-surface [<s>]`: the first submarine comes up to the surface (and fires MANPADS at
//!   helicopters low near it).

use bevy::prelude::*;

use crate::combat::Combat;
use crate::settings::AppState;

pub struct NavalPlugin;

impl Plugin for NavalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (test_aids, mount_sounds)
                .after(crate::combat::CombatSystems)
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// Seconds between the burst sounds of a mount that keeps firing (the sound's length).
const BURST_SOUND_EVERY: f32 = 1.0;

/// Each firing Phalanx mount is heard where it stands, a burst a second.
fn mount_sounds(
    combat: Option<Res<Combat>>,
    time: Res<Time>,
    mut reports: MessageWriter<crate::sound::CiwsReport>,
    mut since: Local<Vec<f32>>,
) {
    let Some(combat) = combat else { return };
    let mounts = &combat.sim.naval().mounts;
    since.resize(mounts.len(), f32::MAX);
    for (m, t) in mounts.iter().zip(since.iter_mut()) {
        if !m.firing {
            *t = f32::MAX;
            continue;
        }
        *t = if *t == f32::MAX { BURST_SOUND_EVERY } else { *t + time.delta_secs() };
        if *t >= BURST_SOUND_EVERY {
            *t = 0.0;
            reports.write(crate::sound::CiwsReport { at: crate::sim_render::metres_point(m.at) });
        }
    }
}

/// `n,km[,s]` of a test aid's argument.
fn count_km(arg: &str) -> Option<(usize, f64, f32)> {
    let mut f = arg.split(',');
    let n = f.next()?.parse().ok()?;
    let km = f.next()?.parse().ok()?;
    let at = f.next().and_then(|t| t.parse().ok()).unwrap_or(2.0);
    Some((n, km, at))
}

/// `--usv-swarm`, `--sea-salvo`, `--sub-surface` (see the module).
fn test_aids(combat: Option<ResMut<Combat>>, time: Res<Time>, mut done: Local<[bool; 3]>) {
    let Some(mut combat) = combat else { return };
    let arg = |name: &str| {
        let mut a = std::env::args().skip_while(|a| a != name);
        a.next().map(|_| a.next().unwrap_or_default())
    };
    let t = time.elapsed_secs();
    if !done[0]
        && let Some((n, km, at)) = arg("--usv-swarm").as_deref().and_then(count_km)
        && t >= at
    {
        done[0] = true;
        let placed = combat.sim.spawn_usv_swarm(n, km);
        info!("--usv-swarm: {placed} boats");
    }
    if !done[1]
        && let Some((n, km, at)) = arg("--sea-salvo").as_deref().and_then(count_km)
        && t >= at
    {
        done[1] = true;
        let fired = combat.sim.test_sea_salvo(n, km);
        info!("--sea-salvo: {fired} missiles");
    }
    if !done[2]
        && let Some(a) = arg("--sub-surface")
        && t >= a.parse().unwrap_or(2.0)
    {
        done[2] = true;
        combat.sim.surface_sub(0);
    }
}
