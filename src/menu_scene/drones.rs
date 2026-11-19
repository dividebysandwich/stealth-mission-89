//! The drone command center's console: an interceptor closing on a Geran (`Shot::DroneChase`),
//! an FP-2 in its terminal dive on a tank column (`Shot::DroneDive`). The camera picture is the
//! drone's forward view of the world; the map and the drones' list are drawn from where they
//! fly.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;
use stealth_sim::uav::{UavKind, UavMode, WpKind};

use super::cockpits::{Cabin, spawn_cabin};
use super::director::{Shot, shot_on};
use super::fx::{self, Airborne};
use super::pod::{self, PodCamera};
use super::stage::{self, Camera, Stage};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::cockpit3d::scene::{SceneScreens, Screen, Seat};
use crate::cockpit3d::{ConsoleScreen, FEED_CANVAS};
use crate::explosions::{Blast, Explosions};
use crate::station::drone::scene::{self as gcs, Mark};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            drones::<false>.run_if(shot_on(Shot::DroneChase)),
            drones::<true>.run_if(shot_on(Shot::DroneDive)),
        )
            .in_set(ShotSet),
    );
}

/// The drone's camera's field of view (rad high) and its picture's size (the feed screen's).
const LENS: f32 = 0.5;
const PICTURE: (u32, u32) = (FEED_CANVAS.0 as u32 * 2, FEED_CANVAS.1 as u32 * 2);
/// The chase: the Geran's speed, the interceptor's, the gap at the start (m); the dive: its
/// speed (m/s), slope (rad) and length (m).
const GERAN: f64 = 51.0;
const QUAD: f64 = 70.0;
const GAP: f64 = 65.0;
const DIVE_SPEED: f64 = 62.0;
const DIVE_SLOPE: f64 = 0.55;
const DIVE_LENGTH: f64 = 265.0;
/// The tank column (models), side by side across the drone's way 22 m apart.
const COLUMN: [&str; 4] = ["aaa_117", "aaa_117", "aaa_120", "aaa_117"];

/// The scene: the console's place, the drone's line (start, unit direction), the quarry's (a
/// Geran's start and direction, or the column's lead vehicle), the end.
#[derive(Component)]
struct Hunt {
    centre: DVec3,
    start: DVec3,
    dir: DVec3,
    quarry: Entity,
    quarry_at: DVec3,
    /// The column's middle (the dive).
    column_at: DVec3,
    hit: bool,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn drones<const DIVE: bool>(
    mut stage: Stage,
    mut screens: SceneScreens,
    mut hunt: Query<&mut Hunt>,
    mut cabins: Query<
        (&Cabin, &mut WorldPos, &mut Transform),
        (Without<SceneCamera>, Without<PodCamera>),
    >,
    mut pods: pod::Pods,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut flir: ResMut<super::SceneFlir>,
    mut ex: ResMut<Explosions>,
) {
    if stage.begin(&[stage::DRONES, stage::AAA]) {
        let centre = stage.land();
        let heading = (stage.director.roll % 628) as f64 * 0.01;
        let flat = DVec3::new(heading.sin(), 0.0, -heading.cos());
        // A few kilometres out from the center.
        let out = centre + flat * 2500.0;
        let (start, dir, quarry, quarry_at) = if DIVE {
            // The column across the drone's way; it dives on one of its vehicles.
            let pick = stage.director.pick(COLUMN.len());
            let side = DVec3::new(-flat.z, 0.0, flat.x);
            let mut target = (Entity::PLACEHOLDER, out);
            for (i, name) in COLUMN.into_iter().enumerate() {
                let at = out + side * (i as f64 * 22.0 - 33.0) + flat * (i as f64 * 4.0);
                let at = DVec3::new(at.x, stage.ground(at), at.z);
                if let Some(e) =
                    stage.model(stage::AAA, name, 0.0, crate::models::ModelState::default())
                {
                    stage.commands.entity(e).insert((
                        WorldPos(at),
                        Transform::from_rotation(Quat::from_rotation_y(-(heading as f32) + 1.5)),
                    ));
                    if i == pick {
                        target = (e, at);
                    }
                }
            }
            let dir = (flat * DIVE_SLOPE.cos() - DVec3::Y * DIVE_SLOPE.sin()).normalize();
            let start = target.1 - dir * DIVE_LENGTH;
            (start, dir, target.0, target.1)
        } else {
            let alt = stage.ground(out) + 400.0;
            let start = DVec3::new(out.x, alt, out.z);
            let geran_at = start + flat * GAP + DVec3::Y * 6.0;
            let g = stage.model(stage::DRONES, "drones_100", 0.0, stage::flying());
            if let Some(e) = g {
                let rot = stage::attitude(heading as f32, 0.0, 0.0);
                stage.commands.entity(e).insert((
                    WorldPos(geran_at),
                    Transform::from_rotation(rot),
                    Airborne((flat * GERAN).as_vec3()),
                ));
            }
            (start, flat, g.unwrap_or(Entity::PLACEHOLDER), geran_at)
        };
        let cabin = spawn_cabin(
            &mut stage,
            Cabin::new(Seat::Console(crate::cockpit3d::DRONE_CONSOLE_MODEL), None),
        );
        stage.commands.entity(cabin).insert((
            pod::WantsPod(PICTURE, LENS),
            WorldPos(centre + DVec3::Y * 2.0),
            Transform::from_rotation(stage::attitude(heading as f32, 0.0, 0.0)),
        ));
        stage.commands.spawn((
            ShotEntity,
            Hunt { centre, start, dir, quarry, quarry_at, column_at: out, hit: false },
        ));
        return;
    }
    let t = stage.director.t as f64;
    let (Ok(mut h), Ok((cabin, wp, tf))) = (hunt.single_mut(), cabins.single_mut()) else { return };
    let speed = if DIVE { DIVE_SPEED } else { QUAD };
    // Where the quarry is now (the Geran flies on; the column stands).
    let quarry = if DIVE { h.quarry_at + DVec3::Y * 1.5 } else { h.quarry_at + h.dir * GERAN * t };
    if !DIVE && !h.hit {
        stage.commands.entity(h.quarry).try_insert(WorldPos(quarry));
    }
    // The drone: on its line; the chase closes on the Geran and ends at it.
    let travel = if DIVE { (speed * t).min(DIVE_LENGTH) } else { speed * t };
    let mut at = h.start + h.dir * travel;
    if !DIVE {
        // It steers onto the Geran's tail.
        let k = (t / 3.0).min(1.0);
        at = at.lerp(quarry - h.dir * (GAP - (QUAD - GERAN) * t).max(4.0), k * 0.6);
    }
    let reached = if DIVE { travel >= DIVE_LENGTH - 1.0 } else { (quarry - at).length() < 8.0 };
    if reached && !h.hit {
        h.hit = true;
        ex.pending.push((quarry, if DIVE { Blast::Bomb } else { Blast::Kill }));
        stage.commands.entity(h.quarry).try_insert(fx::Hit);
    }
    // Its camera: forward, the quarry ahead (it stops with the link).
    if !h.hit {
        let ahead = if DIVE { at + h.dir * 50.0 } else { quarry };
        pod::aim_pod(&mut pods, at, ahead);
    }
    let fwd = h.dir.as_vec3();
    let heading = fwd.x.atan2(-fwd.z);
    let pitch = if DIVE { -DIVE_SLOPE as f32 } else { 0.0 };
    // Where the quarry shows on the feed.
    let rel = (quarry - at).as_vec3();
    let look = if DIVE { fwd } else { rel.normalize_or(fwd) };
    let right = look.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(look);
    let z = rel.dot(look);
    let (fw, fh) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let f = fh / 2.0 / (LENS / 2.0).tan();
    let box_at = (z > 1.0)
        .then(|| Vec2::new(fw / 2.0 + rel.dot(right) / z * f, fh / 2.0 - rel.dot(up) / z * f));
    if cabin.fitted.is_some() {
        // The drone's camera in FLIR.
        flir.0 = Some(PICTURE);
        let flat_at = |p: DVec3| Vec2::new((p.x - h.centre.x) as f32, (h.centre.z - p.z) as f32);
        let kind = if DIVE { UavKind::Fp2 } else { UavKind::Interceptor };
        let mode = if DIVE { UavMode::Dive } else { UavMode::Auto };
        let m = |p: DVec3| DVec2::new(p.x, -p.z);
        let route = [
            (m(h.start), WpKind::Nav),
            (m(h.quarry_at), if DIVE { WpKind::Terminate } else { WpKind::Nav }),
        ];
        let u = gcs::uav(
            kind,
            if DIVE { 3 } else { 1 },
            mode,
            [at.x, -at.z, at.y],
            [heading as f64, pitch as f64, 0.0],
            speed,
            if DIVE { 0.9 } else { 1.0 },
            &route,
        );
        screens.all_pictures();
        if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Feed)) {
            if h.hit {
                // The last picture (the blast) under it.
                tex.canvas.clear(crate::canvas::TRANSPARENT);
                let text = if DIVE { "LINK LOST - IMPACT" } else { "LINK LOST - DETONATED" };
                crate::font::stroke_text_at(
                    &mut tex.canvas,
                    fw / 2.0,
                    fh / 2.0,
                    text,
                    9.0,
                    1.2,
                    crate::station::scope::RED,
                    crate::font::Align::Centre,
                );
                tex.transparent = true;
            } else {
                gcs::feed(&mut tex.canvas, &u, (!DIVE).then_some("GERAN-2"), box_at);
                tex.transparent = true;
            }
            tex.dirty = true;
        }
        if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Map)) {
            let blue = crate::station::scope::CYAN;
            let red = crate::station::scope::RED;
            let label = u.label();
            let mut marks =
                vec![Mark { at: Vec2::ZERO, heading: 0.0, drone: None, col: blue, label: "DCC" }];
            if !h.hit {
                marks.push(Mark {
                    at: flat_at(at),
                    heading,
                    drone: Some(kind),
                    col: blue,
                    label: &label,
                });
            }
            if DIVE {
                for i in 0..COLUMN.len() {
                    let p =
                        flat_at(h.column_at) + Vec2::new(i as f32 * 22.0 - 33.0, i as f32 * 6.0);
                    marks.push(Mark {
                        at: p,
                        heading: 0.0,
                        drone: None,
                        col: red,
                        label: if i == 0 { "ARMOR" } else { "" },
                    });
                }
            } else if !h.hit {
                marks.push(Mark {
                    at: flat_at(quarry),
                    heading,
                    drone: Some(UavKind::Fp2),
                    col: red,
                    label: "GERAN",
                });
            }
            let route: Vec<Vec2> = [h.start, h.quarry_at].iter().map(|&p| flat_at(p)).collect();
            let list = if DIVE {
                [("FP2-3", "DIVE"), ("STK-1", "AUTO"), ("FP2-4", "AUTO")]
            } else {
                [("INT-1", "AUTO LOCK"), ("STK-2", "AUTO"), ("X12-1", "AUTO")]
            };
            gcs::map(&mut tex.canvas, if DIVE { 7.0 } else { 9.0 }, &marks, &route, &list);
            tex.dirty = true;
        }
    }
    super::cockpits::eye_camera(&mut cam, cabin, wp.0, tf.rotation, Vec2::new(0.0, -0.05));
    fov.0 = 62f32.to_radians();
}
