//! The submarine hunt seen from above the sea: a sonobuoy's and a torpedo's splash as they go
//! in, the column of water a torpedo's warhead throws up over a sunk submarine (or a torpedoed
//! ship, with its explosion's sound), and the feather a periscope cuts while a submarine runs at
//! periscope depth.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::combat::Combat;
use crate::explosions::{Blast, Explosions};

/// The classic explosion sound.
const EXPLOSION_SOUND: u8 = 5;
/// The periscope's feather: puffs a second, seen within this far (m).
const WAKE_RATE: f32 = 4.0;
const WAKE_SEEN: f64 = 12_000.0;

/// Simulation metres (east, north) on the sea in the Bevy world.
fn on_sea(e: f64, n: f64) -> DVec3 {
    DVec3::new(e, 0.0, -n)
}

pub fn splashes(
    combat: Option<Res<Combat>>,
    mut ex: ResMut<Explosions>,
    mut sounds: MessageWriter<crate::sound::PlaySound>,
    camera: Query<&crate::world::WorldPos, With<crate::flight::Player>>,
    time: Res<Time>,
    mut since: Local<f32>,
) {
    let Some(combat) = combat else { return };
    for e in &combat.asw_fx {
        if let stealth_sim::SimEvent::AswSplash { at, kind } = *e {
            let p = on_sea(at[0] as f64 / 10.0, at[1] as f64 / 10.0);
            let height = match kind {
                0 => 3.0,
                1 => 6.0,
                2 => 70.0,
                _ => 45.0,
            };
            ex.pending.push((p, Blast::SeaPlume { height }));
            if kind >= 2 {
                sounds.write(crate::sound::PlaySound { id: EXPLOSION_SOUND, priority: 2 });
            }
        }
    }
    // The periscopes' feathers.
    *since += time.delta_secs();
    if *since < 1.0 / WAKE_RATE {
        return;
    }
    *since = 0.0;
    let here = camera.single().ok().map(|p| p.0);
    for w in &combat.sim.asw().wakes {
        let p = on_sea(w[0], w[1]);
        if here.is_some_and(|h| h.with_y(0.0).distance(p) < WAKE_SEEN) {
            ex.pending.push((p, Blast::SeaPlume { height: 0.8 }));
        }
    }
}
