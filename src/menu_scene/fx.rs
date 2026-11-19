//! The scenes' weapons in flight: guided missiles trailing smoke to their targets, bombs
//! falling, gun rounds with their tracers, and the blasts where they end
//! (`explosions::Explosions`, which the menu draws too).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::stage::Stage;
use super::{SceneCamera, ShotEntity, ShotSet};
use crate::explosions::{Blast, Calibre, Explosions};
use crate::palette::Palette;
use crate::palette_material::{FLAG_LINE, PaletteMaterial, PaletteParams};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (missiles, bombs, shells, wrecks)
            .chain()
            .after(ShotSet)
            .run_if(resource_exists::<super::MenuScene>)
            .run_if(resource_exists::<super::director::Director>),
    );
}

const G: f64 = 9.81;

/// What a missile flies at: a point, or a thing of the scene (its place).
#[derive(Clone, Copy, Debug)]
pub enum Aim {
    Point(DVec3),
    Thing(Entity),
}

/// A guided missile (its model's root): its velocity (Bevy m/s), how hard it accelerates
/// (m/s^2) up to its top speed, how fast it turns (rad/s) after flying straight `straight`
/// seconds off the rail, what it flies at and the blast it ends in.
#[derive(Component)]
pub struct Missile {
    pub vel: Vec3,
    pub accel: f32,
    pub top: f32,
    pub turn: f32,
    pub straight: f32,
    pub aim: Aim,
    pub blast: Blast,
    pub age: f32,
}

/// A thing of the scene a missile hit: an aircraft (`Airborne`, flying at the velocity
/// given) comes apart, anything else is left a wreck.
#[derive(Component)]
pub struct Hit;

#[derive(Component)]
pub struct Airborne(pub Vec3);

/// A bomb falling (its model's root): its velocity, and how strongly the air holds it back
/// (1/m: a retarded bomb's chute or fins, `k |v| v`).
#[derive(Component)]
pub struct Bomb {
    pub vel: DVec3,
    pub drag: f64,
}

/// A gun round (its tracer line's entity): velocity, calibre, how long its tracer streak is
/// (seconds of its flight), its glow, and its age; it strikes the ground, or bursts in the air
/// after its fuse's seconds if it has one.
#[derive(Component)]
pub struct Shell {
    pub vel: DVec3,
    pub calibre: Calibre,
    pub streak: f64,
    pub glow: Option<(f32, f32)>,
    pub age: f64,
    pub fuse: Option<(f64, Blast)>,
}

/// Spawns a missile model `name` (a store of the aircraft file) at `at`, flying `vel`; its
/// smoke from its tail.
pub fn launch(
    stage: &mut Stage,
    name: &str,
    at: DVec3,
    vel: Vec3,
    missile: Missile,
) -> Option<Entity> {
    // The motor's flame on (`MOTOR_SLOT`).
    let mut state = crate::models::ModelState::default();
    state.0[14] = 1;
    let e = stage.model(super::stage::AIRCRAFT, name, 0.0, state)?;
    let rot = Transform::IDENTITY.looking_to(vel.normalize_or(Vec3::NEG_Z), Vec3::Y).rotation;
    stage.commands.entity(e).insert((
        WorldPos(at),
        Transform::from_rotation(rot),
        crate::trails::SceneSmoke(vec![Vec3::new(0.0, 0.0, 1.5)]),
        missile,
    ));
    Some(e)
}

/// Spawns a bomb model `name` at `at`, falling with `vel`.
pub fn drop(stage: &mut Stage, name: &str, at: DVec3, vel: DVec3, drag: f64) -> Option<Entity> {
    let e = stage.model(super::stage::AIRCRAFT, name, 0.0, crate::models::ModelState::default())?;
    stage.commands.entity(e).insert((WorldPos(at), Bomb { vel, drag }));
    Some(e)
}

/// Fires a round from `at` with velocity `vel` (Bevy m/s; the gun's own velocity added),
/// its muzzle flash carried along at `carry`; its tracer in `material` (`tracer_material`).
pub fn fire(
    commands: &mut Commands,
    material: Handle<PaletteMaterial>,
    ex: &mut Explosions,
    at: DVec3,
    carry: Vec3,
    shell: Shell,
    flash: bool,
) {
    let dir = shell.vel.as_vec3().normalize_or(Vec3::NEG_Z);
    let calibre = shell.calibre;
    let e = crate::sim_render::spawn_tracer(commands, material);
    commands.entity(e).insert((ShotEntity, shell, WorldPos(at)));
    if flash {
        ex.pending.push((at, Blast::AirMuzzle(calibre, dir, carry)));
    }
}

/// A gun round fired with velocity `vel`, its tracer's streak `streak` seconds long.
pub fn round(vel: DVec3, calibre: Calibre, streak: f64) -> Shell {
    Shell { vel, calibre, streak, glow: None, age: 0.0, fuse: None }
}

/// The tracers' material for the palette in use.
pub fn tracer_material(
    cache: &mut Option<(Handle<PaletteMaterial>, AssetId<bevy::render::storage::ShaderBuffer>)>,
    materials: &mut Assets<PaletteMaterial>,
    palette: &Palette,
) -> Handle<PaletteMaterial> {
    if cache.as_ref().is_none_or(|(_, g)| *g != palette.globals.id()) {
        *cache = None;
    }
    let (m, _) = cache.get_or_insert_with(|| {
        let m = materials.add(PaletteMaterial {
            params: PaletteParams {
                word: 0xFF00 | crate::sim_render::TRACER as u32,
                flags: FLAG_LINE,
                ..default()
            },
            globals: palette.globals.clone(),
        });
        (m, palette.globals.id())
    });
    m.clone()
}

/// The ground's height under a point.
fn ground(theater: &Option<Res<crate::world::Theater>>, p: DVec3) -> f64 {
    theater.as_ref().map_or(0.0, |t| t.hills.ground_m(p.x, -p.z) as f64)
}

/// Flies the missiles: straight off the rail, then turning towards their aim, faster and
/// faster; a blast where they reach it (or the ground).
fn missiles(
    mut commands: Commands,
    time: Res<Time>,
    theater: Option<Res<crate::world::Theater>>,
    mut ex: ResMut<Explosions>,
    mut q: Query<(Entity, &mut Missile, &mut WorldPos, &mut Transform), Without<SceneCamera>>,
    things: Query<&WorldPos, (Without<Missile>, Without<SceneCamera>)>,
) {
    let dt = time.delta_secs().min(0.1);
    for (e, mut m, mut pos, mut tf) in &mut q {
        m.age += dt;
        let target = match m.aim {
            Aim::Point(p) => p,
            Aim::Thing(t) => things.get(t).map_or(pos.0, |p| p.0),
        };
        let to = (target - pos.0).as_vec3();
        let speed = (m.vel.length() + m.accel * dt).min(m.top);
        let mut dir = m.vel.normalize_or(Vec3::NEG_Z);
        if m.age > m.straight {
            let want = to.normalize_or(dir);
            let angle = dir.angle_between(want);
            let step = (m.turn * dt).min(angle);
            if angle > 1e-4 {
                let axis = dir.cross(want).normalize_or(Vec3::Y);
                dir = Quat::from_axis_angle(axis, step) * dir;
            }
        } else {
            // Off the rail it sinks a little.
            m.vel.y -= (G as f32) * dt * 0.5;
            dir = m.vel.normalize_or(dir);
        }
        m.vel = dir * speed;
        let step = (m.vel * dt).as_dvec3();
        let hit = to.length() <= step.length() as f32 * 1.5 + 2.0;
        pos.0 += step;
        tf.rotation = Transform::IDENTITY.looking_to(dir, Vec3::Y).rotation;
        let ground_y = ground(&theater, pos.0);
        if hit || pos.0.y < ground_y {
            let at = if hit { target } else { DVec3::new(pos.0.x, ground_y, pos.0.z) };
            if hit && let Aim::Thing(t) = m.aim {
                commands.entity(t).insert(Hit);
            }
            ex.pending.push((at, m.blast));
            commands.entity(e).remove::<crate::trails::SceneSmoke>().insert(Visibility::Hidden);
            commands.entity(e).remove::<Missile>();
        }
    }
}

/// The bombs fall, nose along their path; a blast on the ground.
fn bombs(
    mut commands: Commands,
    time: Res<Time>,
    theater: Option<Res<crate::world::Theater>>,
    mut ex: ResMut<Explosions>,
    mut q: Query<(Entity, &mut Bomb, &mut WorldPos, &mut Transform), Without<SceneCamera>>,
) {
    let dt = time.delta_secs().min(0.1) as f64;
    for (e, mut b, mut pos, mut tf) in &mut q {
        let drag = -b.vel * b.vel.length() * b.drag;
        b.vel += (drag - DVec3::Y * G) * dt;
        pos.0 += b.vel * dt;
        tf.rotation = Transform::IDENTITY
            .looking_to(b.vel.as_vec3().normalize_or(Vec3::NEG_Y), Vec3::Y)
            .rotation;
        let ground_y = ground(&theater, pos.0);
        if pos.0.y <= ground_y {
            ex.pending.push((DVec3::new(pos.0.x, ground_y, pos.0.z), Blast::Bomb));
            commands.entity(e).despawn();
        }
    }
}

/// The rounds fly, their tracers streaking behind them; a strike where they reach the ground
/// (or their burst).
fn shells(
    mut commands: Commands,
    time: Res<Time>,
    theater: Option<Res<crate::world::Theater>>,
    mut ex: ResMut<Explosions>,
    mut q: Query<
        (Entity, &mut Shell, &mut WorldPos, &mut Transform, &mut Visibility),
        Without<SceneCamera>,
    >,
) {
    let dt = time.delta_secs().min(0.1) as f64;
    for (e, mut s, mut pos, mut tf, mut vis) in &mut q {
        pos.0 += s.vel * dt - DVec3::Y * (0.5 * G * dt * dt);
        s.vel.y -= G * dt;
        s.age += dt;
        let streak = (-s.vel * s.streak.min(s.age)).as_vec3();
        tf.rotation = Quat::from_rotation_arc(Vec3::Z, streak.normalize_or(Vec3::Z));
        tf.scale = Vec3::new(1.0, 1.0, streak.length().max(0.01));
        vis.set_if_neq(Visibility::Inherited);
        if let Some((size, life)) = s.glow {
            ex.pending.push((pos.0, Blast::Tracer { size, life }));
        }
        let ground_y = ground(&theater, pos.0);
        if let Some((fuse, b)) = s.fuse
            && s.age >= fuse
        {
            ex.pending.push((pos.0, b));
            commands.entity(e).despawn();
        } else if pos.0.y <= ground_y {
            ex.pending.push((DVec3::new(pos.0.x, ground_y, pos.0.z), Blast::Strike(s.calibre)));
            commands.entity(e).despawn();
        } else if s.age > 8.0 {
            commands.entity(e).despawn();
        }
    }
}

/// The things hit: an aircraft breaks up and is gone, a vehicle is a wreck.
fn wrecks(
    mut commands: Commands,
    mut ex: ResMut<Explosions>,
    hit: Query<(Entity, &WorldPos, &super::stage::Model, Option<&Airborne>), With<Hit>>,
    mut states: Query<&mut crate::models::ModelState>,
) {
    for (e, pos, model, air) in &hit {
        commands.entity(e).remove::<Hit>();
        if let Some(air) = air {
            ex.breakups.push(crate::explosions::Breakup {
                pos: pos.0,
                vel: air.0,
                size: crate::explosions::AircraftSize::Fighter,
            });
            commands.entity(e).despawn();
        } else if let Ok(mut s) = states.get_mut(model.0) {
            s.0[1] = 1;
        }
    }
}
