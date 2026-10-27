//! The Tornado's terrain following radar: with the autopilot engaged (AUTO) it flies as low as
//! it safely can, at `CLEARANCE` over whatever lies ahead (the mountains and the theater's
//! objects by their collision hulls, the generated buildings, trees and power lines) instead
//! of holding an altitude, still steering to the waypoint.
//!
//! As a real terrain following radar does, it compares the terrain ahead with a `template`
//! ("ski toe"): `CLEARANCE` under the flight path, level for the reaction distance and then
//! curving up as a pull-up of `PULL_G`. Terrain above the template asks for a climb, terrain all
//! below it lets the aircraft descend; `command` turns the largest difference into a flight
//! path angle, which the flight models fly (`Aircraft::tf_gamma`).
//!
//! Every frame while the Tornado flies, the radar scans the track ahead (curving with the turn)
//! for the highest obstacle at each distance (`TerrainRadar`, also for the cockpit's E-scope,
//! `crate::tfr`), whether the autopilot follows it or not.

use bevy::math::DVec2;
use bevy::prelude::*;

use super::{Aircraft, FlightConstants, Player};
use crate::world::{FloatingOrigin, WU, WorldPos};

/// Clearance over the obstacles (metres): 65 ft in altitude units.
pub const CLEARANCE: f32 = 65.0 * WU as f32;
/// Extra load factor the pull-up template assumes (the control law may pull more).
const PULL_G: f32 = 1.5;
/// Reaction time (s): obstacles count as this much flight time nearer.
const REACTION: f32 = 1.5;
/// Time (s) over which the height is brought to the clearance; more than `MARGIN` (metres)
/// below it twice as fast.
const CAPTURE: f32 = 3.0;
const MARGIN: f32 = CLEARANCE * 0.3;
/// Steepest commanded dive and climb (rad).
const MAX_DIVE: f32 = 0.3;
const MAX_CLIMB: f32 = 0.6;
/// Look-ahead: this much flight time, at least `MIN_AHEAD` metres.
const AHEAD_SECS: f32 = 20.0;
const MIN_AHEAD: f32 = 3000.0;
/// Sample spacing (metres) at the aircraft, growing with the distance.
const SPACING: f32 = 20.0;
const SPACING_GROWTH: f32 = 0.02;
/// Half width of the scanned corridor (metres) at the aircraft, growing with the distance.
const HALF_WIDTH: f32 = 20.0;
const WIDTH_GROWTH: f32 = 0.04;
/// The trees (low, and many to look through) count this far ahead, within this reach of the
/// track (metres).
const TREES_AHEAD: f32 = 1500.0;
const TREE_REACH: f64 = 30.0;
/// The radar beam's half width (rad).
const HALF_BEAM: f32 = 0.0175;
const G: f32 = 9.81;

/// The template's height `d` metres ahead at `speed` (m/s), relative to a level flight path
/// through the aircraft (metres): `CLEARANCE` under it for the reaction distance, then rising
/// as the pull-up.
pub fn template(d: f32, speed: f32) -> f32 {
    let v = speed.max(50.0);
    let x = (d - v * REACTION).max(0.0);
    x * x * G * PULL_G / (2.0 * v * v) - CLEARANCE
}

/// The flight path angle (rad) to fly at `speed` (m/s) over a profile of (distance ahead,
/// obstacle height relative to the aircraft) in metres: the climb that puts the template over
/// the highest obstacle above it, or the descent towards the nearest under it. Past the
/// reaction distance the template's own curve makes up the climb; nearer obstacles are closed
/// in on over `CAPTURE` seconds (smoothly holding the clearance; far under it faster, which
/// keeps a sudden obstacle from being cleared late).
pub fn command(profile: &[(f32, f32)], speed: f32) -> f32 {
    let v = speed.max(50.0);
    let lag = v * REACTION;
    profile
        .iter()
        .map(|&(d, h)| {
            let x = (d - lag).max(0.0);
            let above = h - template(d, v);
            let urgent = (above - MARGIN).max(0.0) * (1.0 - x / (v * CAPTURE)).max(0.0);
            ((above + urgent) / x.max(v * CAPTURE)).atan()
        })
        .fold(-MAX_DIVE, f32::max)
        .min(MAX_CLIMB)
}

/// What the terrain following radar sees, for the E-scope (`crate::tfr`).
#[derive(Resource, Default)]
pub struct TerrainRadar {
    /// The radar is scanning (the Tornado in the air).
    pub on: bool,
    /// The autopilot follows the terrain.
    pub following: bool,
    /// Speed (m/s) and flight path angle (rad).
    pub speed: f32,
    pub gamma: f32,
    /// The returns: distance ahead and obstacle height relative to the aircraft (metres), and
    /// whether the beam reaches it (not in the shadow of nearer terrain).
    pub returns: Vec<(f32, f32, bool)>,
}

/// Scans the track ahead (`TerrainRadar`), measures the height of what lies below
/// (`Aircraft::radar_ground`) and, with the autopilot engaged, sets the terrain following
/// command (both None unless an aircraft with the radar flies).
#[allow(clippy::type_complexity)]
pub(super) fn scan(
    k: Res<FlightConstants>,
    origin: Option<Res<FloatingOrigin>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    mut aircraft: Query<(&mut Aircraft, &WorldPos), With<Player>>,
    hulls: Query<(
        &crate::models::CollisionHull,
        &crate::models::LodLevel,
        &crate::models::LodShown,
        &GlobalTransform,
        &InheritedVisibility,
    )>,
    tile_roots: Query<(), With<crate::world::TileObject>>,
    mut radar: ResMut<TerrainRadar>,
) {
    radar.on = false;
    radar.returns.clear();
    let Ok((mut a, pos)) = aircraft.single_mut() else { return };
    a.tf_gamma = None;
    a.radar_ground = None;
    if !k.terrain_following || a.on_ground || a.ended.is_some() {
        return;
    }
    let following = a.autopilot();
    let origin = origin.map_or(bevy::math::DVec3::ZERO, |o| o.origin);
    let here = (pos.0 - origin).as_vec3();
    let path = a.path_dir();
    let speed = (a.velocity / 16.0 * WU as f32).max(50.0);
    let flat = Vec2::new(path.x, path.z).normalize_or(Vec2::NEG_Y);
    // The track curves with the turn, the autopilot's only as far as the waypoint's bearing.
    let (heading, _, roll) = a.euler();
    let turn_rate = G * roll.tan().clamp(-2.0, 2.0) / speed;
    let to_go = super::bam(super::bam_diff(a.waypoint_bearing, super::to_bam(heading))).abs();
    let ahead = (speed * AHEAD_SECS).max(MIN_AHEAD);

    // The tile objects' hulls near the corridor (a circle round its middle).
    let start = Vec2::new(here.x, here.z);
    let mid = start + flat * ahead * 0.5;
    let reach = ahead * 0.5 + HALF_WIDTH + WIDTH_GROWTH * ahead;
    let near: Vec<NearHull> = hulls
        .iter()
        .filter(|(_, lod, shown, _, vis)| vis.get() && shown.main && tile_roots.contains(lod.root))
        .filter_map(|(hull, _, _, tf, _)| {
            let r = (hull.radius + 2.0) * tf.scale().x.max(1e-6);
            let c = tf.translation();
            let c = Vec2::new(c.x, c.z);
            (c.distance(mid) < reach + r).then(|| {
                let inv = tf.affine().inverse();
                NearHull { hull, inv, c, r, up: inv.transform_vector3(Vec3::Y) }
            })
        })
        .collect();

    let mut profile = Vec::with_capacity(256);
    let (mut d, mut p, mut dir, mut turned) = (0.0f32, start, flat, 0.0f32);
    while d <= ahead {
        let step = SPACING + SPACING_GROWTH * d;
        let half = HALF_WIDTH + WIDTH_GROWTH * d;
        let across = dir.perp();
        let mut top = [0.0, -half, half]
            .into_iter()
            .map(|o| hull_top(&near, p + across * o, step * 0.5))
            .fold(0.0, f32::max);
        if let Some(s) = &scenery {
            // Ground metres (east, north) of the point.
            let w = pos.0 + Vec3::new(p.x - start.x, 0.0, p.y - start.y).as_dvec3();
            let r = (half + step * 0.5) as f64;
            let trees = if d < TREES_AHEAD { r.min(TREE_REACH) } else { 0.0 };
            top = top.max(s.top_near(DVec2::new(w.x, -w.z), r, trees) as f32);
        }
        if d == 0.0 {
            a.radar_ground = Some(top / WU as f32);
        }
        profile.push((d, top - here.y));
        // On along the curving track (a positive angle turns right, north to east).
        let left = to_go - turned;
        let turn = (turn_rate * step / speed).clamp(-left, left);
        turned += turn.abs();
        dir = Vec2::from_angle(turn).rotate(dir);
        p += dir * step;
        d += step;
    }
    let gamma = command(&profile, speed);
    if following {
        a.tf_gamma = Some(gamma);
    }
    // The beam sees what no nearer return hides: a return above the steepest sight line so far
    // (less the beam's half width, which still catches the ground just behind a tree).
    let mut sight = f32::NEG_INFINITY;
    radar.returns.extend(profile.iter().map(|&(d, h)| {
        let angle = h.atan2(d.max(1.0));
        let seen = angle >= sight - HALF_BEAM;
        sight = sight.max(angle);
        (d, h, seen)
    }));
    radar.on = true;
    radar.following = following;
    radar.speed = speed;
    radar.gamma = path.y.clamp(-1.0, 1.0).asin();
}

/// A tile object's collision hull near the scanned corridor: its inverse transform, centre and
/// reach (ground plane, metres) and the world's up in its model space.
struct NearHull<'a> {
    hull: &'a crate::models::CollisionHull,
    inv: bevy::math::Affine3A,
    c: Vec2,
    r: f32,
    up: Vec3,
}

/// The highest hull top (metres) within `w` of ground point `p` (Bevy x, z), 0 for none. A
/// hull smaller than the circle counts by its top over its centre.
fn hull_top(near: &[NearHull], p: Vec2, w: f32) -> f32 {
    let mut top = 0.0f32;
    for h in near {
        if p.distance(h.c) > h.r + w {
            continue;
        }
        let q = if h.r < w { h.c } else { p };
        let o = h.inv.transform_point3(Vec3::new(q.x, 0.0, q.y));
        if let Some(t) = h.hull.top(o, h.up) {
            top = top.max(t);
        }
    }
    top
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::FlightControls;

    /// A ridge 600 m high 15 km north (2 km wide each side), a 60 m mast at 23 km, flat ground
    /// otherwise (metres north -> metres).
    fn terrain(n: f32) -> f32 {
        let ridge = 600.0 * (-((n - 15_000.0) / 2000.0).powi(2)).exp();
        let mast = if (n - 23_000.0).abs() < 3.0 { 60.0 } else { 0.0 };
        ridge.max(mast)
    }

    /// The highest ground within `w` of `n`.
    fn terrain_near(n: f32, w: f32) -> f32 {
        (-4..=4).map(|i| terrain(n + w * i as f32 / 4.0)).fold(0.0, f32::max)
    }

    #[test]
    fn command_follows_the_profile() {
        let v = 250.0;
        // Level at the clearance over flat ground: hold it.
        let flat: Vec<_> = (0..200).map(|i| (i as f32 * 25.0, -CLEARANCE)).collect();
        assert!(command(&flat, v).abs() < 0.01, "{}", command(&flat, v));
        // High above it: the steepest dive.
        let high: Vec<_> = flat.iter().map(|&(d, _)| (d, -1500.0)).collect();
        assert_eq!(command(&high, v), -MAX_DIVE);
        // A 300 m wall 1.5 km ahead: climbing already, a far one not yet.
        let wall = |at: f32| {
            flat.iter().map(move |&(d, h)| (d, if d > at { 300.0 - CLEARANCE } else { h }))
        };
        assert!(command(&wall(1500.0).collect::<Vec<_>>(), v) > 0.05);
        assert!(command(&wall(4900.0).collect::<Vec<_>>(), v).abs() < 0.01);
        // Right in front: a steep climb.
        assert!(command(&wall(100.0).collect::<Vec<_>>(), v) > 0.3);
    }

    /// The Tornado's autopilot (both models) from 1500 m at 480 kt: down to the clearance,
    /// over the ridge and the mast, never near them, and low again past them.
    #[test]
    fn tornado_follows_the_terrain() {
        for classic in [false, true] {
            let name = if classic { "classic" } else { "modern" };
            let k = FlightConstants::tornado();
            let tables = super::super::SimTables::load();
            let mut a = super::super::tests::airborne(3000.0, 480.0, 100.0);
            a.autopilot_alt = a.view_z();
            let mut pos = WorldPos(crate::world::wu_to_bevy(0.0, 0.0, a.view_z() as f64));
            let mut tf = Transform::default();
            let c = FlightControls::default();
            let dt = 1.0 / 120.0;
            let (mut lowest, mut flat_max, mut t) = (f32::MAX, 0.0f32, 0.0);
            while -pos.0.z < 30_000.0 && t < 200.0 {
                let (n, y) = (-pos.0.z as f32, pos.0.y as f32);
                let speed = a.velocity / 16.0 * WU as f32;
                let mut profile = vec![];
                let mut d = 0.0;
                while d <= (speed * AHEAD_SECS).max(MIN_AHEAD) {
                    let step = SPACING + SPACING_GROWTH * d;
                    profile.push((d, terrain_near(n + d, step * 0.5) - y));
                    d += step;
                }
                a.tf_gamma = Some(command(&profile, speed));
                if classic {
                    super::super::step_one(&mut a, &mut pos, &mut tf, dt, &c, &k, &tables, None);
                } else {
                    super::super::modern::step(&mut a, &mut pos, &mut tf, dt, &c, &k, None);
                }
                a.sweep_wing(&k, dt);
                t += dt;
                lowest = lowest.min(y - terrain_near(n, 15.0));
                // Settled on the flat between the descent and the ridge, and past the mast.
                if (8500.0..9500.0).contains(&n) || n > 27_000.0 {
                    flat_max = flat_max.max(y);
                }
            }
            println!(
                "{name}: lowest {lowest:.0} m over the ground, highest on the flat {flat_max:.0} \
                 m, {:.0} kt, {:?}",
                a.knots(),
                a.ended
            );
            assert!(a.autopilot() && a.ended.is_none(), "{name}");
            assert!(-pos.0.z > 30_000.0, "{name}: too slow");
            assert!(lowest > CLEARANCE * 0.6, "{name}: {lowest} m over the ground");
            assert!(flat_max < CLEARANCE * 2.0, "{name}: {flat_max} m on the flat");
        }
    }
}
