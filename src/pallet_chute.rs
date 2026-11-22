//! The cargo chutes: a supply pod or airdrop pallet (projectile spec 38, the C-5's pallets) falls
//! under a cluster of two identical canopies (the pilot's, `spawn_canopy`, scaled up, in olive
//! drab) a moment after it leaves the ramp. Drawn as a strapped crate (its type's model, the
//! classic supply pod, carries a canopy of its own, so it is hidden). The canopies blossom
//! streaming behind the load along the air flowing past it, the load swinging after them; as
//! they brake its speed over the ground they rise over it and spread side by side. The menu's
//! airdrop scene hangs its pallets under the same rig ([`CargoRig`]).

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::mesh::{MeshTag, PrimitiveTopology};
use bevy::prelude::*;

use crate::combat::Combat;
use crate::palette::Palette;
use crate::palette_material::{LEVEL_AIRCRAFT, mesh_tag};
use crate::scenery::{CRATE_SIZE, spawn_crate};
use crate::sim_render::{
    CANOPY_LINES, CANOPY_RISERS, RemoteVisuals, SimObject, SimVisuals, spawn_canopy_in,
};
use crate::world::WorldPos;

/// The projectile spec of the supply pod / pallet.
pub const PALLET_SPEC: i32 = 38;
/// Seconds after release before the static line pulls the canopies out.
pub const DEPLOY_SECS: f32 = 0.8;
/// Seconds the canopies take to blossom to their full size.
pub const OPEN_SECS: f32 = 1.2;
/// The cargo canopy against the pilot's: some 10 m across the skirt.
const CANOPY_SCALE: f32 = 1.5;
/// Olive drab gores.
const CARGO_COLOURS: [u8; 2] = [0xA9, 0xAB];
/// The strops from the crate's top corners meet this far over its top (m); the canopies'
/// rigging hangs from there.
const APEX: f32 = 2.0;
/// Each canopy leans this far out of the line the cluster pulls along (rad): streaming behind a
/// fast load, and side by side once the drift over the ground is gone (skirts just touching).
const LEAN_STREAMING: f32 = 0.20;
const LEAN_SPREAD: f32 = 0.42;
/// Speed over the ground (m/s) under which the canopies have spread out.
const SPREAD_BELOW: f32 = 4.0;
/// ... and over which they stream close together.
const STREAM_ABOVE: f32 = 25.0;
/// Seconds over which the drawn velocity is smoothed (its steps come from the sim's frames).
const VELOCITY_SMOOTHING: f32 = 0.3;
/// Steps faster than this (m/s) are jumps, not motion.
const MAX_SPEED: f32 = 400.0;
/// The crate swings at most this far (rad) out of hanging straight under its strops.
const MAX_SWING: f32 = 0.6;

/// A cargo load under its two canopies: the crate (with its strops) and the canopies, each
/// placed on its own with [`CargoRig::pose`].
#[derive(Clone, Copy)]
pub(crate) struct CargoRig {
    pub load: Entity,
    pub canopies: [Entity; 2],
}

/// Where a rig's parts are: the load's place and attitude, the canopies' place, attitude and
/// scale, and whether they are out.
pub(crate) struct RigPose {
    pub load: (DVec3, Quat),
    pub canopies: [(DVec3, Quat, Vec3); 2],
    pub open: bool,
}

impl CargoRig {
    /// Spawns the crate and its canopies (hidden; their meshes are built a moment later).
    pub(crate) fn spawn(commands: &mut Commands, palette: &Palette) -> Self {
        let load = commands
            .spawn((SimObject, WorldPos::default(), Transform::default(), Visibility::Hidden))
            .id();
        let (l, w, h) = (CRATE_SIZE.x, CRATE_SIZE.y, CRATE_SIZE.z);
        let apex = Vec3::Y * (h * 0.5 + APEX);
        let strops = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .into_iter()
            .flat_map(|(x, z)| [Vec3::new(x * w * 0.5, h * 0.5, z * l * 0.5), apex])
            .collect();
        let strops = lines(commands, palette, strops);
        commands.entity(load).add_child(strops);
        // The crate's meshes need the mesh store, which the caller may be reading.
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, palette: Mut<Palette>| {
                world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                    let mut commands = world.commands();
                    let body = spawn_crate(&mut commands, &mut meshes, &palette);
                    // Placed by its parent, not in the world.
                    commands.entity(body).remove::<WorldPos>().insert(Visibility::Inherited);
                    if let Ok(mut l) = commands.get_entity(load) {
                        l.add_child(body);
                    } else {
                        commands.entity(body).despawn();
                    }
                });
            });
            world.flush();
        });
        let canopies = [(); 2].map(|_| {
            let canopy = spawn_canopy_in(commands, palette, CARGO_COLOURS);
            let riser = lines(commands, palette, vec![Vec3::ZERO, Vec3::Y * CANOPY_RISERS]);
            commands
                .entity(canopy)
                .insert(Transform::from_scale(Vec3::splat(0.01)))
                .add_child(riser);
            canopy
        });
        Self { load, canopies }
    }

    /// Puts the rig's parts where `pose` says.
    pub(crate) fn place<F: bevy::ecs::query::QueryFilter>(
        &self,
        pose: &RigPose,
        q: &mut Query<(&mut WorldPos, &mut Transform, &mut Visibility), F>,
    ) {
        if let Ok((mut p, mut tf, mut v)) = q.get_mut(self.load) {
            (p.0, tf.rotation) = pose.load;
            v.set_if_neq(Visibility::Inherited);
        }
        for (&e, &(at, rotation, scale)) in self.canopies.iter().zip(&pose.canopies) {
            if let Ok((mut p, mut tf, mut v)) = q.get_mut(e) {
                (p.0, tf.rotation, tf.scale) = (at, rotation, scale);
                v.set_if_neq(if pose.open { Visibility::Inherited } else { Visibility::Hidden });
            }
        }
    }

    pub(crate) fn entities(&self) -> [Entity; 3] {
        [self.load, self.canopies[0], self.canopies[1]]
    }

    /// The rig `age` seconds after release with its load at `at` moving at `velocity` (Bevy
    /// metres, m/s; no wind), tumbling as `tumble` until its canopies are out and hanging
    /// under them turned to `heading` (about the vertical).
    pub(crate) fn pose(
        at: DVec3,
        tumble: Quat,
        heading: Quat,
        velocity: Vec3,
        age: f32,
    ) -> RigPose {
        let open = ((age - DEPLOY_SECS) / OPEN_SECS).clamp(0.0, 1.0);
        // The cluster pulls up into the air flowing past the load (against its velocity),
        // straight up once it hangs still.
        let pull =
            (-velocity).try_normalize().filter(|_| velocity.length() > 0.5).unwrap_or(Vec3::Y);
        // The load swings after it, so far.
        let hang = {
            let tilt = Vec3::Y.angle_between(pull);
            let axis = Vec3::Y.cross(pull).try_normalize().unwrap_or(Vec3::X);
            Quat::from_axis_angle(axis, tilt.min(MAX_SWING))
        };
        let load = tumble.slerp(hang * heading, open);
        let apex = at + (load * Vec3::Y * (CRATE_SIZE.z * 0.5 + APEX)).as_dvec3();
        // Side by side across the drift (else across the load), leaning apart further as the
        // drift dies.
        let drift = Vec3::new(velocity.x, 0.0, velocity.z);
        let across = Vec3::Y.cross(drift).try_normalize().unwrap_or(heading * Vec3::X);
        let across = (across - pull * across.dot(pull)).try_normalize().unwrap_or(Vec3::X);
        let calm =
            1.0 - ((drift.length() - SPREAD_BELOW) / (STREAM_ABOVE - SPREAD_BELOW)).clamp(0.0, 1.0);
        let lean =
            LEAN_STREAMING + (LEAN_SPREAD - LEAN_STREAMING) * calm * calm * (3.0 - 2.0 * calm);
        // Turning the pull about this axis leans it towards `across`.
        let axis = pull.cross(across).normalize_or(Vec3::Z);
        // Blossoms quickly, then fills out.
        let size = 1.0 - (1.0 - open).powi(3);
        let scale = Vec3::new(size, 0.4 + 0.6 * size, size) * CANOPY_SCALE;
        let canopy = |side: f32| {
            let rotation = Quat::from_axis_angle(axis, side * lean * open)
                * Quat::from_rotation_arc(Vec3::Y, pull)
                * heading;
            (apex, rotation, scale)
        };
        RigPose { load: (at, load), canopies: [canopy(1.0), canopy(-1.0)], open: open > 0.0 }
    }
}

/// A line mesh's part, tagged and shadowless like the canopy's lines (its mesh is added a
/// moment later).
fn lines(commands: &mut Commands, palette: &Palette, pts: Vec<Vec3>) -> Entity {
    let e = commands
        .spawn((
            Transform::default(),
            Visibility::Inherited,
            MeshMaterial3d(palette.line_material(CANOPY_LINES)),
            MeshTag(mesh_tag(LEVEL_AIRCRAFT, 0, false)),
            bevy::light::NotShadowCaster,
        ))
        .id();
    commands.queue(move |world: &mut World| {
        let mut mesh =
            Mesh::new(PrimitiveTopology::LineList, bevy::asset::RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pts);
        let h = world.resource_mut::<Assets<Mesh>>().add(mesh);
        if let Ok(mut e) = world.get_entity_mut(e) {
            e.insert(Mesh3d(h));
        }
    });
    e
}

/// Whose pallet: the own simulation's slot, or another pilot's (pilot, slot).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Own(usize),
    Remote(u8, u8),
}

/// A falling pallet as drawn: the projectile's model it stands for (hidden), its rig, its age
/// and its velocity (Bevy metres; None before its second frame).
struct Chute {
    pallet: Entity,
    rig: CargoRig,
    age: f32,
    last: Option<DVec3>,
    velocity: Option<Vec3>,
}

/// Each falling pallet's chutes.
#[derive(Resource, Default)]
pub(crate) struct PalletChutes(HashMap<Key, Chute>);

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pallet_chutes(
    mut commands: Commands,
    combat: Res<Combat>,
    vis: Res<SimVisuals>,
    remote: Res<RemoteVisuals>,
    time: Res<Time>,
    palette: Res<Palette>,
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
    chutes.0.retain(|key, c| {
        let keep = live.iter().any(|(k, e)| k == key && *e == c.pallet);
        if !keep {
            for e in c.rig.entities() {
                commands.entity(e).try_despawn();
            }
        }
        keep
    });
    let dt = time.delta_secs();
    for (key, pallet) in live {
        let c = chutes.0.entry(key).or_insert_with(|| Chute {
            pallet,
            rig: CargoRig::spawn(&mut commands, &palette),
            age: 0.0,
            last: None,
            velocity: None,
        });
        c.age += dt;
        // The projectile's model (the classic pod under its own canopy) gives way to the crate.
        let Ok((pos, tf, mut v)) = q.get_mut(pallet) else { continue };
        v.set_if_neq(Visibility::Hidden);
        let (at, model_rotation) = (pos.0, tf.rotation);
        // Not placed yet (its first frame).
        if at == DVec3::ZERO {
            continue;
        }
        // Its velocity as drawn, smoothed over the sim's steps.
        if let Some(last) = c.last
            && dt > 0.0
        {
            let now = ((at - last) / dt as f64).as_vec3();
            let k = 1.0 - (-dt / VELOCITY_SMOOTHING).exp();
            if now.length() < MAX_SPEED {
                c.velocity = Some(c.velocity.map_or(now, |v| v.lerp(now, k)));
            }
        }
        c.last = Some(at);
        let ahead = model_rotation * Vec3::NEG_Z;
        let yaw = if ahead.xz().length() > 1e-3 { (-ahead.x).atan2(-ahead.z) } else { 0.0 };
        let pose = CargoRig::pose(
            at,
            model_rotation,
            Quat::from_rotation_y(yaw),
            c.velocity.unwrap_or(Vec3::ZERO),
            c.age,
        );
        c.rig.place(&pose, &mut q);
    }
}
