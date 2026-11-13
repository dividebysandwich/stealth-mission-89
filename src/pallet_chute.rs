//! The cargo chutes: a supply pod or airdrop pallet (projectile spec 38, the C-5's pallets) falls
//! under a parachute. The canopy (the pilot's, `spawn_canopy`, scaled up) blossoms a moment
//! after the pallet leaves the ramp, and the pallet hangs level under it.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::combat::Combat;
use crate::sim_render::{RemoteVisuals, SimVisuals, spawn_canopy};
use crate::world::WorldPos;

/// The projectile spec of the supply pod / pallet.
pub const PALLET_SPEC: i32 = 38;
/// Seconds after release before the static line pulls the canopy out.
const DEPLOY_SECS: f32 = 0.8;
/// Seconds the canopy takes to blossom to its full size.
const OPEN_SECS: f32 = 1.2;
/// The cargo canopy against the pilot's: some 10 m across the skirt.
const CANOPY_SCALE: f32 = 1.5;

/// Whose pallet: the own simulation's slot, or another pilot's (pilot, slot).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Own(usize),
    Remote(u8, u8),
}

/// Each falling pallet's canopy: the pallet's model it hangs over, the canopy and its age.
#[derive(Resource, Default)]
pub(crate) struct PalletChutes(HashMap<Key, (Entity, Entity, f32)>);

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pallet_chutes(
    mut commands: Commands,
    combat: Res<Combat>,
    vis: Res<SimVisuals>,
    remote: Res<RemoteVisuals>,
    time: Res<Time>,
    assets: crate::models::ModelAssets,
    mut chutes: ResMut<PalletChutes>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility)>,
) {
    let sim = &combat.sim;
    let me = sim.active_pilot();
    let mut live: Vec<(Key, Entity)> = sim
        .projectiles()
        .iter()
        .enumerate()
        .filter(|(_, m)| m.ttl != 0 && m.spec == PALLET_SPEC)
        .filter_map(|(s, _)| vis.proj[s].map(|(e, ..)| (Key::Own(s), e)))
        .collect();
    for p in sim.remote_pilots().into_iter().filter(|p| p.id != me) {
        for &(s, _) in p.missiles.iter().filter(|(_, m)| m.spec == PALLET_SPEC) {
            if let Some(e) = remote.missile((p.id, s)) {
                live.push((Key::Remote(p.id, s), e));
            }
        }
    }
    chutes.0.retain(|key, (pallet, canopy, _)| {
        let keep = live.iter().any(|(k, e)| k == key && e == pallet);
        if !keep {
            commands.entity(*canopy).try_despawn();
        }
        keep
    });
    let dt = time.delta_secs();
    for (key, pallet) in live {
        let (_, canopy, age) = chutes.0.entry(key).or_insert_with(|| {
            let canopy = spawn_canopy(&mut commands, &assets);
            commands.entity(canopy).insert(Transform::from_scale(Vec3::splat(0.01)));
            (pallet, canopy, 0.0)
        });
        *age += dt;
        let Ok((pos, mut tf, _)) = q.get_mut(pallet) else { continue };
        let at = pos.0;
        // Level under the canopy, its heading kept.
        let ahead = tf.rotation * Vec3::NEG_Z;
        let yaw = if ahead.xz().length() > 1e-3 { (-ahead.x).atan2(-ahead.z) } else { 0.0 };
        let open = ((*age - DEPLOY_SECS) / OPEN_SECS).clamp(0.0, 1.0);
        if open > 0.0 {
            tf.rotation = tf.rotation.slerp(Quat::from_rotation_y(yaw), open);
        }
        let Ok((mut cpos, mut ctf, mut cv)) = q.get_mut(*canopy) else { continue };
        cpos.0 = at;
        // Blossoms quickly, then fills out.
        let size = 1.0 - (1.0 - open).powi(3);
        ctf.scale = Vec3::new(size, 0.4 + 0.6 * size, size) * CANOPY_SCALE;
        ctf.rotation = Quat::from_rotation_y(yaw);
        cv.set_if_neq(if open > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
    }
}
