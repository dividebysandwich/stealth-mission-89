//! The CBU-105 after its dispenser opens, as drawn (`stealth_sim::sfw`): the puff of its skin
//! cut away, the ten submunitions on their chutes, their rockets' flames, the skeets coming
//! down, and each skeet's penetrator: a white-hot streak down to the vehicle and the hit or
//! the kill there, or the little burst of a skeet that found nothing.
//!
//! The simulation steps the submunitions and skeets (sent to the other pilots with the moving
//! world); between its frames they are drawn on along their velocity.

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::SimEvent;
use stealth_sim::sfw::Phase;

use crate::combat::Combat;
use crate::explosions::{Blast, Calibre, Explosions};
use crate::models::ModelAssets;
use crate::palette_material::{FLAG_LINE, PaletteMaterial, PaletteParams};
use crate::sim_render::{FLARE, SimObject, map_point, spawn_canopy, spawn_dot, spawn_tracer};
use crate::world::WorldPos;

/// The chute's canopy against the pilot's (`spawn_canopy`): some 3 m across the skirt, a little
/// over the real one's size, to be made out from the weapon view.
const CANOPY_SCALE: f32 = 0.45;
/// Seconds a penetrator's streak shows.
const STREAK_SECS: f32 = 0.12;
/// Seconds at most a submunition is drawn on ahead of the last simulation frame.
const AHEAD_SECS: f64 = 0.25;
/// The dark grey of the submunitions and skeets.
const BODY: u8 = 239;

/// A submunition or skeet as drawn: its entities, and its last simulated position and
/// velocity (ft, ft/s) and when they came.
struct Body {
    dot: Entity,
    canopy: Option<Entity>,
    flame: Option<Entity>,
    p: [f64; 3],
    v: [f64; 3],
    at: f64,
}

/// A penetrator's streak.
struct Streak {
    line: Entity,
    from: DVec3,
    to: DVec3,
    age: f32,
}

#[derive(Resource, Default)]
pub struct SfwVisuals {
    blus: HashMap<u16, Body>,
    skeets: HashMap<u16, Body>,
    streaks: Vec<Streak>,
    /// Dot materials: the bodies', the rocket flames'.
    materials: Option<(Handle<PaletteMaterial>, Handle<PaletteMaterial>)>,
}

/// Bevy world point of a position in world units (ft: east, south, true altitude).
fn point(p: [f64; 3]) -> DVec3 {
    map_point(p[0] / 32.0, p[1] / 32.0, stealth_sim::math::view_z_f(p[2]))
}

/// Bevy world point of a fine position (`SimEvent`s').
fn fine_point(x: i32, y: i32, alt: i32) -> DVec3 {
    let f = stealth_sim::ROUND_FINE as f64;
    point([x as f64 / f, y as f64 / f, alt as f64 / f])
}

/// Where the player's own submunitions (else skeets) are, on average, as drawn: the weapon view
/// follows them down. None when none are in the air.
pub fn submunitions_centre(combat: &Combat) -> Option<DVec3> {
    let sfw = combat.sim.sfw();
    let me = combat.sim.active_pilot();
    let mine: Vec<[f64; 3]> =
        sfw.blus.iter().filter(|b| b.owner == me && b.t >= 0.0).map(|b| b.p).collect();
    let mine = if mine.is_empty() {
        sfw.skeets.iter().filter(|s| s.owner == me).map(|s| s.p).collect()
    } else {
        mine
    };
    if mine.is_empty() {
        return None;
    }
    let n = mine.len() as f64;
    Some(point(std::array::from_fn(|k| mine.iter().map(|p| p[k]).sum::<f64>() / n)))
}

/// Draws the CBU-105s' submunitions, skeets and their events this frame.
#[allow(clippy::too_many_arguments)]
pub fn sync_sfw(
    mut commands: Commands,
    combat: Res<Combat>,
    mut vis: ResMut<SfwVisuals>,
    mut ex: ResMut<Explosions>,
    mut reports: MessageWriter<crate::sound::GunReport>,
    time: Res<Time>,
    assets: ModelAssets,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), With<SimObject>>,
) {
    let vis = &mut *vis;
    let (body, flame) = vis
        .materials
        .get_or_insert_with(|| {
            let mut line = |colour: u8| {
                materials.add(PaletteMaterial {
                    params: PaletteParams {
                        word: 0xFF00 | colour as u32,
                        flags: FLAG_LINE,
                        ..default()
                    },
                    globals: assets.palette.globals.clone(),
                })
            };
            (line(BODY), line(FLARE))
        })
        .clone();

    for e in &combat.sfw_fx {
        match *e {
            SimEvent::SfwDispense { x, y, alt } => {
                ex.pending.push((fine_point(x, y, alt), Blast::Pop { size: 4.0, shade: 0.55 }));
            }
            SimEvent::SkeetBurst { x, y, alt } => {
                ex.pending.push((fine_point(x, y, alt), Blast::Pop { size: 1.5, shade: 0.15 }));
            }
            SimEvent::SkeetFire { from, to, hit, kill } => {
                let (a, b) = (fine_point(from.0, from.1, from.2), fine_point(to.0, to.1, to.2));
                let down = (b - a).as_vec3().normalize_or(Vec3::NEG_Y);
                ex.pending.push((a, Blast::Muzzle(Calibre::Small, down)));
                let blast = if kill {
                    Blast::Kill
                } else if hit {
                    Blast::ArmourHit(Calibre::Tank)
                } else {
                    Blast::Strike(Calibre::Tank)
                };
                ex.pending.push((b, blast));
                reports.write(crate::sound::GunReport { at: b, calibre: Calibre::Tank });
                let line = spawn_tracer(&mut commands, flame.clone());
                vis.streaks.push(Streak { line, from: a, to: b, age: 0.0 });
            }
            _ => {}
        }
    }

    let now = time.elapsed_secs_f64();
    let sfw = combat.sim.sfw();
    // Submunitions: a dot, the canopy while on the chute, the flame while the rocket burns.
    let live: Vec<u16> = sfw.blus.iter().map(|b| b.id).collect();
    vis.blus.retain(|id, b| {
        let keep = live.contains(id);
        if !keep {
            for e in [Some(b.dot), b.canopy, b.flame].into_iter().flatten() {
                commands.entity(e).try_despawn();
            }
        }
        keep
    });
    for s in &sfw.blus {
        if s.t < 0.0 {
            continue;
        }
        let b = vis.blus.entry(s.id).or_insert_with(|| {
            let canopy = spawn_canopy(&mut commands, &assets);
            commands.entity(canopy).insert(Transform::from_scale(Vec3::splat(CANOPY_SCALE)));
            let fl = spawn_dot(&mut commands, &assets, flame.clone(), &[Vec3::ZERO]);
            commands.entity(fl).insert(crate::lighting::LightEmitter {
                color: Vec3::new(2.0, 1.6, 1.2),
                range: 60.0,
            });
            Body {
                dot: spawn_dot(&mut commands, &assets, body.clone(), &[Vec3::ZERO]),
                canopy: Some(canopy),
                flame: Some(fl),
                p: s.p,
                v: s.v,
                at: now,
            }
        });
        if b.p != s.p {
            (b.p, b.v, b.at) = (s.p, s.v, now);
        }
        let ahead = (now - b.at).min(AHEAD_SECS);
        let at = point(std::array::from_fn(|k| b.p[k] + b.v[k] * ahead));
        let shown = [
            (Some(b.dot), true),
            (b.canopy, s.phase == Phase::Chute),
            (b.flame, s.phase == Phase::Rocket),
        ];
        for (e, on) in shown {
            let Some(Ok((mut pos, _, mut v))) = e.map(|e| q.get_mut(e)) else { continue };
            pos.0 = at;
            v.set_if_neq(if on { Visibility::Inherited } else { Visibility::Hidden });
        }
    }
    // Skeets: dots.
    let live: Vec<u16> = sfw.skeets.iter().map(|s| s.id).collect();
    vis.skeets.retain(|id, b| {
        let keep = live.contains(id);
        if !keep {
            commands.entity(b.dot).try_despawn();
        }
        keep
    });
    for s in &sfw.skeets {
        let b = vis.skeets.entry(s.id).or_insert_with(|| Body {
            dot: spawn_dot(&mut commands, &assets, body.clone(), &[Vec3::ZERO]),
            canopy: None,
            flame: None,
            p: s.p,
            v: s.v,
            at: now,
        });
        if b.p != s.p {
            (b.p, b.v, b.at) = (s.p, s.v, now);
        }
        let ahead = (now - b.at).min(AHEAD_SECS);
        if let Ok((mut pos, _, mut v)) = q.get_mut(b.dot) {
            pos.0 = point(std::array::from_fn(|k| b.p[k] + b.v[k] * ahead));
            v.set_if_neq(Visibility::Inherited);
        }
    }

    // The penetrators' streaks, from the skeet down to the target.
    let dt = time.delta_secs();
    vis.streaks.retain_mut(|s| {
        s.age += dt;
        if s.age > STREAK_SECS {
            commands.entity(s.line).try_despawn();
            return false;
        }
        if let Ok((mut pos, mut tf, mut v)) = q.get_mut(s.line) {
            let d = (s.to - s.from).as_vec3();
            pos.0 = s.from;
            tf.rotation = Quat::from_rotation_arc(Vec3::Z, d.normalize_or(Vec3::Z));
            tf.scale = Vec3::new(1.0, 1.0, d.length().max(0.01));
            v.set_if_neq(Visibility::Inherited);
        }
        true
    });
}
