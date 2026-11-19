//! Scenes of aircraft in flight seen from outside: a random one with the camera orbiting close
//! (`Shot::Orbit`), the Tornado low over the ground roaring past (`Shot::Tornado`) and an
//! interceptor drone closing on a Geran (`Shot::Interceptor`).

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
pub(super) fn fly(
    stage: &Stage,
    flyers: &Query<(Entity, &Flyer)>,
    q: &mut Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
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
        let (r, l) = crate::aircraft::wing_slot_values(1.0, 25f32.to_radians(), 67f32.to_radians());
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

/// An interceptor quadcopter (0.6 m long), nose down and racing, closing on a Geran ahead of
/// it; the camera rides beside and behind it.
fn interceptor(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    const SPEED: f32 = 60.0;
    const GERAN_SPEED: f32 = 51.0;
    if stage.begin(&[stage::DRONES]) {
        let start = stage.land() + DVec3::Y * 350.0;
        let heading = (stage.director.roll % 360) as f32 * 0.0175;
        let quad = Weave {
            start,
            heading,
            speed: SPEED,
            amp: Weave::amp_for(5.0, 20.0),
            period: 5.0,
            phase: 0.0,
            climb: 0.0,
        };
        let f = stage::forward(heading) * 70.0;
        let geran = Weave {
            start: start + enu(f.x as f64, 6.0, f.y as f64),
            speed: GERAN_SPEED,
            amp: Weave::amp_for(7.0, 10.0),
            period: 7.0,
            ..quad
        };
        if let Some(e) = stage.model(stage::DRONES, "drones_105", 0.0, stage::flying()) {
            stage.commands.entity(e).insert(Flyer::lead(quad, None));
        }
        if let Some(e) = stage.model(stage::DRONES, "drones_100", 0.0, stage::flying()) {
            stage.commands.entity(e).insert(Flyer::other(geran, None));
        }
        return;
    }
    let t = stage.director.t;
    let Some((e, p, r, v)) = fly(&stage, &flyers, &mut q) else { return };
    // The quad flies nose down (its thrust tilted forward).
    let heading = v.x.atan2(-v.z);
    stage::put(&mut q, e, p, r * Quat::from_rotation_x(-0.45));
    // Behind and beside it, swinging out a little; the Geran beyond it.
    let side = 1.2 + 0.2 * t;
    let eye = stage::behind(p, heading, 2.4 - 0.2 * t, side, 0.6);
    let ahead = stage::forward(heading) * 1.0;
    stage::look(&mut cam, eye, p + enu(ahead.x as f64, 0.3, ahead.y as f64));
    fov.0 = 50f32.to_radians();
}
