//! Scenes of aircraft in flight seen from outside: a random one with the camera orbiting close
//! (`Shot::Orbit`), the Tornado low over the ground roaring past (`Shot::Tornado`) and an
//! interceptor drone closing on a Geran (`Shot::Interceptor`), a helicopter low over the land
//! (`Shot::Nap`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotSet};
use crate::aircraft::AircraftKind;
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            orbit.run_if(shot_on(Shot::Orbit)),
            tornado.run_if(shot_on(Shot::Tornado)),
            interceptor.run_if(shot_on(Shot::Interceptor)),
            nap.run_if(shot_on(Shot::Nap)),
        )
            .in_set(ShotSet),
    );
}

/// Something flying a weave, its height over the ground held (`agl` metres) if set; the lead
/// is the one the camera follows.
#[derive(Component)]
pub(super) struct Flyer {
    pub weave: Weave,
    pub agl: Option<f32>,
    pub lead: bool,
}

impl Flyer {
    pub fn lead(weave: Weave, agl: Option<f32>) -> Self {
        Flyer { weave, agl, lead: true }
    }

    pub fn other(weave: Weave, agl: Option<f32>) -> Self {
        Flyer { weave, agl, lead: false }
    }
}

/// Flies the scene's flyers to their place `t` seconds in; the lead's entity, place,
/// attitude and velocity.
pub(super) fn fly<F: bevy::ecs::query::QueryFilter>(
    stage: &Stage,
    flyers: &Query<(Entity, &Flyer)>,
    q: &mut Query<(&mut WorldPos, &mut Transform), F>,
) -> Option<(Entity, DVec3, Quat, Vec3)> {
    let t = stage.director.t;
    let mut lead = None;
    for (e, f) in flyers {
        let (mut p, r, v) = f.weave.at(t);
        if let Some(agl) = f.agl {
            p.y = stage.ground(p) + agl as f64;
        }
        stage::put(q, e, p, r);
        if f.lead {
            lead = Some((e, p, r, v));
        }
    }
    lead
}

/// The aircraft types that orbit: any flyable one.
fn orbit_kind(pick: usize) -> AircraftKind {
    AircraftKind::ALL[pick % AircraftKind::ALL.len()]
}

/// A random aircraft weaving gently over the land, the camera circling it close.
fn orbit(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    let kind = orbit_kind(stage.director.pick(AircraftKind::ALL.len()));
    if stage.begin(&[stage::AIRCRAFT]) {
        let heli = kind.spec().helicopter;
        let start = stage.land();
        let heading = (stage.director.roll % 360) as f32 * 0.0175;
        let agl = if heli { 120.0 } else { 700.0 };
        let weave = Weave {
            start,
            heading,
            speed: stage::cruise(kind),
            amp: Weave::amp_for(12.0, if heli { 12.0 } else { 35.0 }),
            period: 12.0,
            phase: 0.0,
            climb: 0.0,
        };
        if let Some(e) = stage.aircraft(kind) {
            stage.commands.entity(e).insert(Flyer::lead(weave, Some(agl)));
        }
        return;
    }
    let Some((_, p, _, v)) = fly(&stage, &flyers, &mut q) else { return };
    // Round it a quarter turn over the scene, a little above, from its side to its nose.
    let t = stage.director.t;
    let len = stage::length(kind);
    let heading = v.x.atan2(-v.z);
    let a = heading + 2.2 - 0.32 * t;
    let r = len * 1.5 + 6.0;
    let eye = p + enu((a.sin() * r) as f64, (len * 0.25) as f64, (a.cos() * r) as f64);
    stage::look(&mut cam, eye, p);
    fov.0 = 40f32.to_radians();
}

/// The Tornado, wings swept back, low over the ground at speed; the camera stands beside its
/// track a little above the ground and turns to follow it past.
fn tornado(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut eye: Local<DVec3>,
) {
    const SPEED: f32 = 240.0;
    const AGL: f32 = 35.0;
    if stage.begin(&[stage::AIRCRAFT]) {
        let mid = stage.land();
        let heading = (stage.director.roll % 360) as f32 * 0.0175;
        // Abeam the camera 3 s in.
        let f = stage::forward(heading) * SPEED * 3.0;
        let start = mid - enu(f.x as f64, 0.0, f.y as f64);
        let weave = Weave {
            start,
            heading,
            speed: SPEED,
            amp: Weave::amp_for(6.0, 25.0),
            period: 6.0,
            phase: 0.0,
            climb: 0.0,
        };
        let spec = AircraftKind::Tornado.spec();
        let mut state = stage::flying();
        let w = spec.flight().aero.swing_wing.expect("a swing wing");
        let (r, l) = crate::aircraft::wing_slot_values(1.0, w.forward, w.back);
        let (sr, sl) = crate::aircraft::WING_SLOTS;
        (state.0[sr], state.0[sl]) = (r, l);
        if let Some(e) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) {
            stage.commands.entity(e).insert(Flyer::lead(weave, Some(AGL)));
        }
        // Beside the track, on the scene's side, low.
        let side = if stage.director.roll & 1 == 0 { 1.0 } else { -1.0 };
        let r = stage::right(heading) * 80.0 * side;
        let at = mid + enu(r.x as f64, 0.0, r.y as f64);
        // A little under its height: the ground and the horizon behind it.
        *eye = at + DVec3::Y * (stage.ground(mid).max(stage.ground(at)) - at.y + AGL as f64 - 12.0);
        return;
    }
    let Some((_, p, _, _)) = fly(&stage, &flyers, &mut q) else { return };
    stage::look(&mut cam, *eye, p);
    fov.0 = 30f32.to_radians();
}

/// An interceptor closing on a Geran ahead of it, the camera riding with it at a random
/// place behind it (near or farther, beside either way, above or below): the quadcopter
/// (0.6 m long) on a Geran-2, or the X-12 jet (2.4 m) on a jet Geran-3. Both fly level along
/// their way, and the interceptor is still behind the Geran at the scene's end.
fn interceptor(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    // Interceptor model, its speed and length, the Geran's model and speed (m/s, m).
    const QUAD: (&str, f32, f32, &str, f32) = ("drones_105", 60.0, 0.6, "drones_100", 51.0);
    const X12: (&str, f32, f32, &str, f32) = ("drones_106", 132.0, 2.4, "drones_101", 122.0);
    let (model, speed, len, quarry, quarry_speed) =
        if (stage.director.roll >> 9) & 1 == 0 { QUAD } else { X12 };
    // The Geran's lead at the start: it closes over the scene to some lengths of the
    // interceptor at the end, never past it.
    let gap = ((speed - quarry_speed) * Shot::Interceptor.seconds() + len * 8.0).max(70.0);
    if stage.begin(&[stage::DRONES]) {
        let start = stage.land() + DVec3::Y * 350.0;
        let heading = (stage.director.roll % 360) as f32 * 0.0175;
        let own = Weave {
            start,
            heading,
            speed,
            amp: Weave::amp_for(5.0, 20.0),
            period: 5.0,
            phase: 0.0,
            climb: 0.0,
        };
        let f = stage::forward(heading) * gap;
        let geran = Weave {
            start: start + enu(f.x as f64, 6.0, f.y as f64),
            speed: quarry_speed,
            amp: Weave::amp_for(7.0, 10.0),
            period: 7.0,
            ..own
        };
        if let Some(e) = stage.model(stage::DRONES, model, 0.0, stage::flying()) {
            stage.commands.entity(e).insert(Flyer::lead(own, None));
        }
        if let Some(e) = stage.model(stage::DRONES, quarry, 0.0, stage::flying()) {
            stage.commands.entity(e).insert(Flyer::other(geran, None));
        }
        return;
    }
    let t = stage.director.t;
    let Some((_, p, _, v)) = fly(&stage, &flyers, &mut q) else { return };
    let heading = v.x.atan2(-v.z);
    // The camera's place behind it, the scene's pick (lengths of a quadcopter, as far as the
    // interceptor's size wants): near or farther back, beside it either way, above or under
    // it, drifting in or out a little; the Geran beyond it.
    let r = |n: u32| {
        let h = (stage.director.roll ^ n.wrapping_mul(0x9E37_79B9)).wrapping_mul(0x45D9_F3B);
        ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
    };
    let sign = if r(1) < 0.5 { 1.0 } else { -1.0 };
    let back = 1.6 + 2.2 * r(2) + (r(3) - 0.6) * 0.3 * t;
    let side = sign * ((0.5 + 1.3 * r(4)) + (r(5) - 0.3) * 0.3 * t);
    let up = -0.7 + 1.9 * r(6);
    let k = len / 0.6;
    let s = k.sqrt();
    let eye = stage::behind(p, heading, back * s * 1.4f32.min(k), side * s * 1.6f32.min(k), up * s);
    let ahead = stage::forward(heading) * len * 0.8;
    stage::look(&mut cam, eye, p + enu(ahead.x as f64, 0.3, ahead.y as f64));
    fov.0 = 50f32.to_radians();
}

/// The helicopters that fly low: their height over the ground (m, over the trees) and speed
/// (m/s).
const NAP: [(AircraftKind, f32, f32); 3] = [
    (AircraftKind::OH58, 22.0, 45.0),
    (AircraftKind::AH64, 25.0, 55.0),
    (AircraftKind::CH47, 30.0, 60.0),
];

/// A Kiowa, an Apache or a Chinook low over the land, nose down, flying straight along the
/// ground's rise and fall; the camera, the scene's pick, rides behind it beside its track, flies along
/// abeam, or stands on the ground ahead and turns as it passes.
fn nap(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut post: Local<DVec3>,
) {
    let (kind, agl, speed) = NAP[stage.director.pick(NAP.len())];
    let view = (stage.director.roll >> 4) % 3;
    let side = if (stage.director.roll >> 6) & 1 == 0 { 1.0 } else { -1.0 };
    if stage.begin(&[stage::AIRCRAFT]) {
        let mid = stage.land();
        let heading = (stage.director.roll % 360) as f32 * 0.0175;
        // Past the scene's middle half way through.
        let f = stage::forward(heading) * speed * super::director::SHOT_SECONDS * 0.5;
        let start = mid - enu(f.x as f64, 0.0, f.y as f64);
        let weave = Weave { start, heading, speed, amp: 0.0, period: 7.0, phase: 0.0, climb: 0.0 };
        let spec = kind.spec();
        if let Some(e) = stage.aircraft(kind) {
            stage.commands.entity(e).insert(Flyer::lead(weave, Some(agl)));
            // Nose down in fast forward flight.
            let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
            let tilt = Transform::from_xyz(0.0, spec.model_lift, 0.0)
                .with_rotation(Quat::from_rotation_x(-0.09))
                .with_scale(Vec3::splat(scale));
            stage.commands.queue(move |w: &mut World| {
                if let Some(m) = w.get::<stage::Model>(e).map(|m| m.0) {
                    w.entity_mut(m).insert(tilt);
                }
            });
        }
        // The post: beside the track, ahead of where it passes, a little under its height (the
        // land and the horizon behind it).
        let r = stage::right(heading) * 60.0 * side;
        let at = mid + enu(r.x as f64, 0.0, r.y as f64);
        *post = DVec3::new(at.x, stage.ground(at).max(stage.ground(mid)) + agl as f64 * 0.6, at.z);
        return;
    }
    let t = stage.director.t;
    let Some((_, p, _, v)) = fly(&stage, &flyers, &mut q) else { return };
    let heading = v.x.atan2(-v.z);
    let len = stage::length(kind);
    match view {
        0 => {
            // Chasing, behind and to the side, a little above.
            let eye = stage::behind(p, heading, len * 2.2, len * 0.9 * side, len * 0.35);
            let eye = eye.with_y(eye.y.max(stage.ground(eye) + 4.0));
            let ahead = stage::forward(heading) * len;
            stage::look(&mut cam, eye, p + enu(ahead.x as f64, 0.0, ahead.y as f64));
            fov.0 = 45f32.to_radians();
        }
        1 => {
            // Abeam, drifting from its quarter to its nose.
            let back = len * (1.0 - 0.35 * t);
            let eye = stage::behind(p, heading, back, len * 2.4 * side, len * 0.1);
            let eye = eye.with_y(eye.y.max(stage.ground(eye) + 4.0));
            stage::look(&mut cam, eye, p);
            fov.0 = 40f32.to_radians();
        }
        _ => {
            // From the ground as it goes by.
            stage::look(&mut cam, *post, p);
            fov.0 = 35f32.to_radians();
        }
    }
}
