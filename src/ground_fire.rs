//! Vehicles' gunfire as drawn (`SimEvent::GroundShot`): the muzzle flash, the round's tracer
//! flying to where it lands, and what it does there (a strike in the dirt beside the target,
//! sparks off its armour, or the blast that knocks it out).
//!
//! The simulation knows where each round lands and when: the tracer takes the round's flight
//! time ([`GroundGun::flight_time`]) over the way as drawn, so a knocked out vehicle turns into
//! a wreck as its round arrives.

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::aaa::GroundKind;
use stealth_sim::ground::GroundGun;

use crate::combat::Combat;
use crate::explosions::{Blast, Calibre, Explosions};
use crate::palette_material::{FLAG_LINE, PaletteMaterial, PaletteParams};
use crate::sim_render::{DrawnUnits, SimObject};
use crate::world::WorldPos;

/// Seconds of a round's flight its tracer streak shows.
const STREAK: f32 = 0.02;

/// A round on its way.
struct Round {
    from: DVec3,
    to: DVec3,
    age: f32,
    time: f32,
    calibre: Calibre,
    /// Where it lands: on the target (and knocks it out), or on the ground.
    hit: bool,
    kill: bool,
    /// The friendly side's (its tracer's colour).
    blue: bool,
}

/// The vehicles' rounds in flight and their tracer lines.
#[derive(Resource, Default)]
pub struct GroundRounds {
    live: Vec<Round>,
    /// Tracer lines and the side whose colour each has.
    tracers: Vec<(Entity, bool)>,
    /// Tracer materials: the friendly side's, the enemy's.
    materials: Option<(Handle<PaletteMaterial>, Handle<PaletteMaterial>)>,
}

/// The muzzle's height over the ground and its reach ahead of the hull's centre (m).
fn muzzle_of(kind: GroundKind) -> (f32, f32) {
    match kind {
        k if k.tank() => (2.2, 6.0),
        k if k.carrier() => (2.4, 3.0),
        _ => (2.6, 2.0),
    }
}

fn calibre(gun: GroundGun, kind: GroundKind) -> Calibre {
    match gun {
        GroundGun::Tank => Calibre::Tank,
        GroundGun::Cannon => Calibre::Cannon,
        // The Gepard's 35 mm; the Shilka's 23 mm.
        GroundGun::Flak if kind == GroundKind::Gepard => Calibre::Cannon,
        GroundGun::Flak => Calibre::Small,
    }
}

/// Starts this frame's rounds (the flash at the muzzle), flies them, and lands them.
#[allow(clippy::too_many_arguments)]
pub fn ground_fire(
    mut commands: Commands,
    combat: Res<Combat>,
    drawn: Res<DrawnUnits>,
    mut rounds: ResMut<GroundRounds>,
    mut ex: ResMut<Explosions>,
    time: Res<Time>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    palette: Option<Res<crate::palette::Palette>>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), With<SimObject>>,
) {
    let Some(palette) = palette else { return };
    let rounds = &mut *rounds;
    let sim = &combat.sim;
    let units = sim.units();
    for s in &combat.ground_shots {
        let (Some(&(a, heading)), Some(&(b, _))) = (drawn.0.get(s.from), drawn.0.get(s.to)) else {
            continue;
        };
        let (Some(ua), Some(ub)) = (units.get(s.from), units.get(s.to)) else { continue };
        let (up, reach) = muzzle_of(ua.kind);
        let flat = Vec3::new((b - a).x as f32, 0.0, (b - a).z as f32);
        let dir = flat.try_normalize().unwrap_or(Vec3::new(heading.sin(), 0.0, -heading.cos()));
        let from = a + (Vec3::Y * up + dir * reach).as_dvec3();
        let to = if s.hit {
            b + DVec3::Y * if ub.kind.tank() { 1.6 } else { 1.8 }
        } else {
            // Metres east and north: Bevy x and -z.
            b + DVec3::new(s.miss.0 as f64, 0.0, -s.miss.1 as f64)
        };
        let c = calibre(s.gun, ua.kind);
        ex.pending.push((from, Blast::Muzzle(c, dir)));
        let d = from.distance(to);
        rounds.live.push(Round {
            from,
            to,
            age: 0.0,
            time: s.gun.flight_time(d) as f32,
            calibre: c,
            hit: s.hit,
            kill: s.kill,
            blue: sim.aaa_blue(s.from),
        });
    }

    let dt = time.delta_secs();
    rounds.live.retain_mut(|r| {
        r.age += dt;
        if r.age < r.time {
            return true;
        }
        let blast = if r.kill {
            Blast::Kill
        } else if r.hit {
            Blast::ArmourHit(r.calibre)
        } else {
            Blast::Strike(r.calibre)
        };
        // A wreck's blast on the ground under it.
        let at = if r.kill { DVec3::new(r.to.x, r.to.y - 1.7, r.to.z) } else { r.to };
        ex.pending.push((at, blast));
        false
    });

    let (blue, red) = rounds
        .materials
        .get_or_insert_with(|| {
            let mut line = |colour: u8| {
                materials.add(PaletteMaterial {
                    params: PaletteParams {
                        word: 0xFF00 | colour as u32,
                        flags: FLAG_LINE,
                        ..default()
                    },
                    globals: palette.globals.clone(),
                })
            };
            (line(crate::sim_render::TRACER), line(crate::sim_render::ENEMY_TRACER))
        })
        .clone();
    while rounds.tracers.len() < rounds.live.len() {
        let e = crate::sim_render::spawn_tracer(&mut commands, blue.clone());
        rounds.tracers.push((e, true));
    }
    for (k, (e, colour)) in rounds.tracers.iter_mut().enumerate() {
        let e = *e;
        let Ok((mut pos, mut tf, mut vis)) = q.get_mut(e) else { continue };
        let Some(r) = rounds.live.get(k) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        let t = (r.age / r.time.max(1e-3)) as f64;
        let tail = ((r.age - STREAK).max(0.0) / r.time.max(1e-3)) as f64;
        let head = r.from.lerp(r.to, t);
        pos.0 = head;
        let streak = (r.from.lerp(r.to, tail) - head).as_vec3();
        tf.rotation = Quat::from_rotation_arc(Vec3::Z, streak.normalize_or(Vec3::Z));
        tf.scale = Vec3::new(1.0, 1.0, streak.length().max(0.01));
        if *colour != r.blue {
            *colour = r.blue;
            commands.entity(e).insert(MeshMaterial3d(if r.blue {
                blue.clone()
            } else {
                red.clone()
            }));
        }
        vis.set_if_neq(Visibility::Inherited);
    }
}
