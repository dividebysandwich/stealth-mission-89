//! The rockets (70 mm and Zuni) as drawn (`stealth_sim::rockets`): each a bright dot while its motor burns
//! (its smoke is a trail, `trails`), a dark one after; a puff off the pod as it leaves, and its
//! warhead's burst where it comes down.
//!
//! The simulation steps them (sent to the other pilots with the moving world); between its
//! frames they are drawn on along their velocity.

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::SimEvent;
use stealth_sim::rockets::Rocket;

use crate::combat::{Combat, FS};
use crate::explosions::{Blast, Calibre, Explosions};
use crate::models::ModelAssets;
use crate::palette_material::{FLAG_LINE, PaletteMaterial, PaletteParams};
use crate::sim_render::{SimObject, aaa_point, spawn_dot};
use crate::world::WorldPos;

/// The motor's glow and the spent rocket's dark body (palette indices).
const FLAME: u8 = 0x0F;
const BODY: u8 = 170;

#[derive(Resource, Default)]
pub struct RocketVisuals {
    rockets: HashMap<u16, (Entity, bool)>,
    materials: Option<[Handle<PaletteMaterial>; 2]>,
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

/// Where rocket `r` is drawn now: on from its last simulated position along its velocity.
pub fn drawn_at(combat: &Combat, r: &Rocket) -> DVec3 {
    let ahead = combat.alpha.clamp(0.0, 1.5) / FS as f32;
    aaa_point(std::array::from_fn(|k| (r.p[k] + r.v[k] * ahead) as f64))
}

/// Draws the rockets in flight and their launches and bursts this frame.
#[allow(clippy::too_many_arguments)]
pub fn sync_rockets(
    mut commands: Commands,
    combat: Res<Combat>,
    mut vis: ResMut<RocketVisuals>,
    mut ex: ResMut<Explosions>,
    mut reports: MessageWriter<crate::sound::GunReport>,
    assets: ModelAssets,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut q: Query<(&mut WorldPos, &mut Visibility), With<SimObject>>,
) {
    let vis = &mut *vis;
    if vis.materials.as_ref().is_some_and(|m| {
        m.iter().any(|m| !crate::palette_material::current(&materials, m, &assets.palette.globals))
    }) {
        vis.materials = None;
    }
    let [flame, body] = vis
        .materials
        .get_or_insert_with(|| {
            [FLAME, BODY].map(|c| {
                materials.add(PaletteMaterial {
                    params: PaletteParams {
                        word: 0xFF00 | c as u32,
                        flags: FLAG_LINE,
                        ..default()
                    },
                    globals: assets.palette.globals.clone(),
                })
            })
        })
        .clone();

    for e in &combat.rocket_fx {
        match *e {
            SimEvent::RocketFired { at, zuni, .. } => {
                let size = if zuni { 2.5 } else { 1.5 };
                ex.pending.push((fine_point(at), Blast::Pop { size, shade: 0.8 }));
            }
            SimEvent::RocketBurst { at, kill, zuni } => {
                let p = fine_point(at);
                // The Zuni's warhead is five times the 70 mm rocket's.
                let calibre = if zuni { Calibre::Mm105 } else { Calibre::Mm40 };
                let blast = if kill { Blast::Kill } else { Blast::Strike(calibre) };
                ex.pending.push((p, blast));
                reports.write(crate::sound::GunReport { at: p, calibre });
            }
            _ => {}
        }
    }

    let rockets = &combat.sim.rockets().rockets;
    let live: Vec<u16> = rockets.iter().map(|r| r.id).collect();
    vis.rockets.retain(|id, (e, _)| {
        let keep = live.contains(id);
        if !keep {
            commands.entity(*e).try_despawn();
        }
        keep
    });
    for r in rockets {
        let burning = r.burning();
        let (e, lit) = vis.rockets.entry(r.id).or_insert_with(|| {
            let m = if burning { flame.clone() } else { body.clone() };
            (spawn_dot(&mut commands, &assets, m, &[Vec3::ZERO]), burning)
        });
        // Burnt out: the dark body from now on.
        if *lit && !burning {
            commands.entity(*e).try_despawn();
            *e = spawn_dot(&mut commands, &assets, body.clone(), &[Vec3::ZERO]);
            *lit = false;
        }
        if let Ok((mut pos, mut v)) = q.get_mut(*e) {
            pos.0 = drawn_at(&combat, r);
            v.set_if_neq(Visibility::Inherited);
        }
    }
}
