//! The JP233's submunitions as drawn (`stealth_sim::jp233`): the bomblets and mines coming
//! down on their small chutes, the mines lying where they landed, and each going off: a
//! cratering bomblet's burst of earth, a mine's sharp charge, a burning kill.
//!
//! The simulation steps them (sent to the other pilots with the moving world); between its
//! frames they are drawn on along their velocity.

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::SimEvent;

use crate::combat::Combat;
use crate::explosions::{Blast, Calibre, Explosions};
use crate::models::ModelAssets;
use crate::palette_material::{FLAG_LINE, PaletteMaterial, PaletteParams};
use crate::sim_render::{SimObject, aaa_point, spawn_canopy, spawn_dot};
use crate::world::WorldPos;

/// The chute's canopy against the pilot's (`spawn_canopy`): a metre or so across.
const CANOPY_SCALE: f32 = 0.15;
/// Seconds at most a submunition is drawn on ahead of the last simulation frame.
const AHEAD_SECS: f64 = 0.25;
/// The dark olive of the submunitions and mines.
const BODY: u8 = 170;

/// A submunition as drawn: its entities, its last simulated position and velocity (m, m/s)
/// and when they came.
struct Body {
    dot: Entity,
    canopy: Entity,
    p: [f32; 3],
    v: [f32; 3],
    at: f64,
}

#[derive(Resource, Default)]
pub struct Jp233Visuals {
    bomblets: HashMap<u16, Body>,
    mines: HashMap<u16, Entity>,
    material: Option<Handle<PaletteMaterial>>,
}

/// Bevy world point of a fine position (`SimEvent`s': wu east, south, ft up).
fn fine_point(at: (i32, i32, i32)) -> DVec3 {
    let f = stealth_sim::ROUND_FINE as f64;
    crate::sim_render::map_point(
        at.0 as f64 / f / 32.0,
        at.1 as f64 / f / 32.0,
        stealth_sim::math::view_z_f(at.2 as f64 / f),
    )
}

fn metres(p: [f32; 3]) -> DVec3 {
    aaa_point(p.map(|c| c as f64))
}

/// Draws the JP233s' submunitions, the mines on the ground and their bursts this frame.
#[allow(clippy::too_many_arguments)]
pub fn sync_jp233(
    mut commands: Commands,
    combat: Res<Combat>,
    mut vis: ResMut<Jp233Visuals>,
    mut ex: ResMut<Explosions>,
    mut reports: MessageWriter<crate::sound::GunReport>,
    time: Res<Time>,
    assets: ModelAssets,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), With<SimObject>>,
) {
    let vis = &mut *vis;
    if vis
        .material
        .as_ref()
        .is_some_and(|m| !crate::palette_material::current(&materials, m, &assets.palette.globals))
    {
        vis.material = None;
    }
    let body = vis
        .material
        .get_or_insert_with(|| {
            materials.add(PaletteMaterial {
                params: PaletteParams { word: 0xFF00 | BODY as u32, flags: FLAG_LINE, ..default() },
                globals: assets.palette.globals.clone(),
            })
        })
        .clone();

    for e in &combat.jp233_fx {
        if let SimEvent::Jp233Burst { at, crater, kill } = *e {
            let p = fine_point(at);
            let (blast, calibre) = match (crater, kill) {
                (_, true) => (Blast::Kill, Calibre::Tank),
                (true, false) => (Blast::Strike(Calibre::Tank), Calibre::Tank),
                (false, false) => (Blast::Strike(Calibre::Cannon), Calibre::Cannon),
            };
            ex.pending.push((p, blast));
            if crater {
                ex.pending.push((p, Blast::Pop { size: 3.0, shade: 0.25 }));
            }
            reports.write(crate::sound::GunReport { at: p, calibre });
        }
    }

    let now = time.elapsed_secs_f64();
    let jp = combat.sim.jp233();
    // The submunitions on their way down: a dot, the canopy once the chute is out.
    let live: Vec<u16> = jp.bomblets.iter().map(|b| b.id).collect();
    vis.bomblets.retain(|id, b| {
        let keep = live.contains(id);
        if !keep {
            commands.entity(b.dot).try_despawn();
            commands.entity(b.canopy).try_despawn();
        }
        keep
    });
    for s in &jp.bomblets {
        let b = vis.bomblets.entry(s.id).or_insert_with(|| {
            let canopy = spawn_canopy(&mut commands, &assets);
            commands.entity(canopy).insert(Transform::from_scale(Vec3::splat(CANOPY_SCALE)));
            Body {
                dot: spawn_dot(&mut commands, &assets, body.clone(), &[Vec3::ZERO]),
                canopy,
                p: s.p,
                v: s.v,
                at: now,
            }
        });
        if b.p != s.p {
            (b.p, b.v, b.at) = (s.p, s.v, now);
        }
        let ahead = (now - b.at).min(AHEAD_SECS) as f32;
        let at = metres(std::array::from_fn(|k| b.p[k] + b.v[k] * ahead));
        for (e, on) in [(b.dot, true), (b.canopy, s.open())] {
            let Ok((mut pos, _, mut v)) = q.get_mut(e) else { continue };
            pos.0 = at;
            v.set_if_neq(if on { Visibility::Inherited } else { Visibility::Hidden });
        }
    }
    // The mines lying in wait.
    let live: Vec<u16> = jp.mines.iter().map(|m| m.id).collect();
    vis.mines.retain(|id, &mut e| {
        let keep = live.contains(id);
        if !keep {
            commands.entity(e).try_despawn();
        }
        keep
    });
    for m in &jp.mines {
        let e = *vis
            .mines
            .entry(m.id)
            .or_insert_with(|| spawn_dot(&mut commands, &assets, body.clone(), &[Vec3::ZERO]));
        if let Ok((mut pos, _, mut v)) = q.get_mut(e) {
            pos.0 = metres([m.p[0], m.p[1], m.p[2] + 0.2]);
            v.set_if_neq(Visibility::Inherited);
        }
    }
}
