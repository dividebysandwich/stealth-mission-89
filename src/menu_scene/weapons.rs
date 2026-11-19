//! Weapons in action: a Tornado laying the JP233's bomblets along a runway (`Shot::Jp233`), a
//! SLAM-ER or JSOW in flight with the camera swinging round its nose (`Shot::Standoff`).

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::flyers::{Flyer, fly};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::explosions::{Blast, Calibre, Explosions};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (jp233.run_if(shot_on(Shot::Jp233)), standoff.run_if(shot_on(Shot::Standoff)))
            .in_set(ShotSet),
    );
}

/// The run: height (m), speed (m/s), over the aim point this long into the scene (s), the
/// dispensers emptying from, to (s), a bomblet out every so often (s).
const RUN_HEIGHT: f32 = 45.0;
const RUN_SPEED: f32 = 230.0;
const OVER_AIM: f32 = 1.6;
const DISPENSE: (f32, f32) = (0.6, 2.1);
const EVERY: f32 = 0.035;
/// The bomblets' drag under their chutes (1/m: `k |v| v`) along the ground and in the fall
/// (some 18 m/s down), the chute's size.
const DRAG: f64 = 0.03;
const FALL_DRAG: f64 = 0.03;
const CHUTE: f32 = 0.3;

/// A bomblet coming down: its velocity, its chute, whether it craters (else a mine).
#[derive(Component)]
struct Bomblet {
    vel: DVec3,
    age: f32,
    crater: bool,
}

/// The run: the runway's aim point and heading, the camera's place, the bomblets out.
#[derive(Component)]
struct Run {
    aim: DVec3,
    heading: f32,
    eye: DVec3,
    out: usize,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn jp233(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut run: Query<&mut Run>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut bomblets: Query<(Entity, &mut Bomblet)>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut ex: ResMut<Explosions>,
    time: Res<Time>,
) {
    if stage.begin(&[stage::AIRCRAFT]) {
        // Down a runway of the theater's (else a field's strip).
        let ends = stage.runway_ends();
        let (aim, toward) = match ends.get(stage.director.pick(ends.len().max(1))) {
            Some(&e) => e,
            None => {
                let places = stage.places();
                let f = places.airfields.first().copied().unwrap_or(DVec2::splat(256_000.0));
                (f + DVec2::new(0.0, 300.0), DVec2::new(0.0, 1.0))
            }
        };
        // Along it, from the end the aim point is at: over the runway's length.
        let heading = (-toward.x as f32).atan2(-toward.y as f32);
        let aim = enu(aim.x, 0.0, aim.y);
        let aim = DVec3::new(aim.x, stage.ground(aim), aim.z);
        let f = stage::forward(heading) * RUN_SPEED * OVER_AIM;
        let start = aim + enu(-f.x as f64, RUN_HEIGHT as f64, -f.y as f64);
        let weave = Weave {
            start,
            heading,
            speed: RUN_SPEED,
            amp: 0.0,
            period: 10.0,
            phase: 0.0,
            climb: 0.0,
        };
        let spec = AircraftKind::Tornado.spec();
        let mut state = stage::flying();
        let (r, l) = crate::aircraft::wing_slot_values(1.0, 25f32.to_radians(), 67f32.to_radians());
        let (sr, sl) = crate::aircraft::WING_SLOTS;
        (state.0[sr], state.0[sl]) = (r, l);
        if let Some(e) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) {
            stage.commands.entity(e).insert(Flyer::lead(weave, None));
        }
        // Beside the runway, a little way down it, on the right (the run passes left to
        // right across the free side of the screen).
        let along = stage::forward(heading) * 200.0;
        let side = stage::right(heading) * 85.0;
        let eye = aim + enu((along.x + side.x) as f64, 0.0, (along.y + side.y) as f64);
        let eye = DVec3::new(eye.x, stage.ground(eye) + 9.0, eye.z);
        stage.commands.spawn((ShotEntity, Run { aim, heading, eye, out: 0 }));
        return;
    }
    let t = stage.director.t;
    let Ok(mut r) = run.single_mut() else { return };
    let Some((_, p, rot, v)) = fly(&stage, &flyers, &mut q) else { return };
    // The two dispensers under the fuselage, a bomblet from one then the other.
    if (DISPENSE.0..DISPENSE.1).contains(&t) {
        let due = ((t - DISPENSE.0) / EVERY) as usize;
        while r.out <= due {
            r.out += 1;
            let k = r.out as f32;
            let side = if r.out % 2 == 0 { 0.9 } else { -0.9 };
            let at = p + (rot * Vec3::new(side, -0.9, 0.5)).as_dvec3();
            // Thrown out sideways a little, down, falling back from the aircraft.
            let spread = rot * Vec3::new(side * 4.0 + (k * 2.3).sin() * 3.0, -12.0, 0.0);
            let vel = (v * 0.9 + spread).as_dvec3();
            let canopy = stage.canopy();
            stage.commands.entity(canopy).insert((
                WorldPos(at),
                Transform::from_scale(Vec3::splat(0.01)),
                Visibility::Inherited,
                Bomblet { vel, age: 0.0, crater: r.out % 2 == 0 },
            ));
        }
    }
    // The bomblets fall under their chutes; the cratering ones burst on the runway, the mines
    // lie there.
    let dt = time.delta_secs().min(0.1);
    for (e, mut b) in &mut bomblets {
        b.age += dt;
        let open = (b.age / 0.25).min(1.0);
        // The chute brakes the throw along the ground, and the fall to its own steady rate.
        let flat = DVec3::new(b.vel.x, 0.0, b.vel.z);
        let along = -flat * flat.length() * DRAG * open as f64;
        let fall = -b.vel.y * b.vel.y.abs() * FALL_DRAG * open as f64;
        b.vel += (along + DVec3::Y * (fall - 9.81)) * dt as f64;
        let Ok((mut pos, mut tf)) = q.get_mut(e) else { continue };
        pos.0 += b.vel * dt as f64;
        tf.scale = Vec3::splat(CHUTE * open.max(0.01));
        let ground = stage.ground(pos.0);
        if pos.0.y <= ground + 0.3 {
            let at = DVec3::new(pos.0.x, ground, pos.0.z);
            if b.crater {
                ex.pending.push((at, Blast::Strike(Calibre::Tank)));
                ex.pending.push((at, Blast::Pop { size: 3.0, shade: 0.25 }));
            } else {
                ex.pending.push((at, Blast::Strike(Calibre::Small)));
            }
            stage.commands.entity(e).despawn();
        }
    }
    // Follows the Tornado past, then the carpet of bursts along the runway.
    let k = ((t - 1.6) / 1.4).clamp(0.0, 1.0) as f64;
    let along = stage::forward(r.heading) * 110.0;
    let carpet = r.aim + enu(along.x as f64, 0.0, along.y as f64);
    stage::look(&mut cam, r.eye, p + (carpet - p) * k);
    fov.0 = 38f32.to_radians();
}

/// The missile: its speed (m/s), height over the ground (m), and how far round its nose the
/// camera swings in the scene (rad).
const SLAM_SPEED: f32 = 240.0;
const STANDOFF_AGL: f32 = 120.0;
const SWING: f32 = 1.6;

/// A SLAM-ER cruising low or a JSOW gliding, wings out, the camera close round its nose.
fn standoff(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    let jsow = stage.director.roll & 1 == 1;
    if stage.begin(&[stage::AIRCRAFT]) {
        let start = stage.land();
        let heading = ((stage.director.roll >> 3) % 628) as f32 * 0.01;
        let weave = Weave {
            start,
            heading,
            speed: if jsow { 190.0 } else { SLAM_SPEED },
            amp: Weave::amp_for(9.0, if jsow { 6.0 } else { 12.0 }),
            period: 9.0,
            phase: 0.0,
            climb: if jsow { -12.0 } else { 0.0 },
        };
        // The wings out (`stores` WINGS_SLOT).
        let mut state = crate::models::ModelState::default();
        state.0[15] = 1;
        let name = if jsow { "aircraft_082" } else { "aircraft_080" };
        if let Some(e) = stage.model(stage::AIRCRAFT, name, 0.0, state) {
            stage.commands.entity(e).insert(Flyer::lead(weave, Some(STANDOFF_AGL)));
        }
        return;
    }
    let t = stage.director.t;
    let Some((_, p, rot, _)) = fly(&stage, &flyers, &mut q) else { return };
    // Round its nose from one side to the other, a little above, a few metres out.
    let a = -SWING / 2.0 + SWING * t / super::director::SHOT_SECONDS + 0.4;
    let off = Vec3::new(a.sin() * 5.5, 0.9 + 0.4 * (t * 0.7).sin(), -a.cos() * 5.5);
    let eye = p + (rot * off).as_dvec3();
    let nose = p + (rot * Vec3::new(0.0, 0.0, -0.6)).as_dvec3();
    stage::look(&mut cam, eye, nose);
    fov.0 = 42f32.to_radians();
}
