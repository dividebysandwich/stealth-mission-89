//! The AC-130 in its left-hand pylon turn over a target, its side guns firing down at it: the
//! 40 mm every half second, the 105 mm now and then, their tracers glowing on the way down
//! and the bursts round the vehicles (`Shot::Gunship`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::fx::{self, Shell};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::{AircraftKind, C130_MUZZLES};
use crate::explosions::{Calibre, Explosions};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(Update, gunship.in_set(ShotSet).run_if(shot_on(Shot::Gunship)));
}

/// The orbit: radius and height over the target (m), speed (m/s).
const RADIUS: f32 = 1300.0;
const HEIGHT: f32 = 1000.0;
const SPEED: f32 = 100.0;
/// The 40 mm's rounds (seconds apart) and the 105 mm's shots (seconds into the scene).
const BOFORS_EVERY: f32 = 0.5;
const HOWITZER_AT: [f32; 2] = [0.8, 3.3];
/// The guns' tracer streaks and glows (`side_guns.rs`).
const STREAK: [f64; 2] = [0.05, 0.035];
const GLOW: [(f32, f32); 2] = [(0.9, 0.06), (1.4, 0.05)];

/// The scene: the target's centre, where on the circle the aircraft starts (radians from
/// east), the rounds fired of each gun.
#[derive(Component)]
struct Orbit {
    centre: DVec3,
    start: f32,
    fired: [usize; 2],
}

/// The aircraft on its circle `t` seconds in: place, attitude, velocity (Bevy).
fn on_circle(o: &Orbit, t: f32) -> (DVec3, Quat, Vec3) {
    let a = o.start + t * SPEED / RADIUS;
    let p = o.centre + enu((RADIUS * a.cos()) as f64, HEIGHT as f64, (RADIUS * a.sin()) as f64);
    // Anticlockwise from above: a left turn, the guns towards the centre.
    let (ve, vn) = (-a.sin() * SPEED, a.cos() * SPEED);
    let heading = ve.atan2(vn);
    let bank = (SPEED * SPEED / (9.81 * RADIUS)).atan();
    (p, stage::attitude(heading, 0.0, bank), Vec3::new(ve, 0.0, -vn))
}

#[allow(clippy::too_many_arguments)]
fn gunship(
    mut stage: Stage,
    mut orbit: Query<&mut Orbit>,
    aircraft: Query<Entity, With<GunshipCraft>>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    (mut ex, mut materials, palette): (
        ResMut<Explosions>,
        ResMut<Assets<crate::palette_material::PaletteMaterial>>,
        Res<crate::palette::Palette>,
    ),
    mut tracer: Local<
        Option<(
            Handle<crate::palette_material::PaletteMaterial>,
            AssetId<bevy::render::storage::ShaderBuffer>,
        )>,
    >,
) {
    if stage.begin(&[stage::AIRCRAFT, stage::AAA]) {
        let centre = stage.land();
        let start = (stage.director.roll % 628) as f32 * 0.01;
        if let Some(e) = stage.aircraft(AircraftKind::AC130) {
            stage.commands.entity(e).insert(GunshipCraft);
        }
        // A few vehicles round the target.
        for (i, name) in ["aaa_117", "aaa_120", "aaa_117", "aaa_110"].into_iter().enumerate() {
            let a = i as f32 * 1.7;
            let at = centre + enu((a.cos() * 30.0 * i as f32) as f64, 0.0, (a.sin() * 25.0) as f64);
            let at = DVec3::new(at.x, stage.ground(at), at.z);
            if let Some(e) =
                stage.model(stage::AAA, name, 0.0, crate::models::ModelState::default())
            {
                stage
                    .commands
                    .entity(e)
                    .insert((WorldPos(at), Transform::from_rotation(Quat::from_rotation_y(a))));
            }
        }
        stage.commands.spawn((ShotEntity, Orbit { centre, start, fired: [0, 0] }));
        return;
    }
    let t = stage.director.t;
    let Ok(mut o) = orbit.single_mut() else { return };
    let (p, rot, v) = on_circle(&o, t);
    if let Ok(e) = aircraft.single() {
        stage::put(&mut q, e, p, rot);
    }
    // The guns: rounds due by now, at the target with a little spread, aimed up by their drop.
    let material = fx::tracer_material(&mut tracer, &mut materials, &palette);
    let due =
        [((t / BOFORS_EVERY) as usize).min(10), HOWITZER_AT.iter().filter(|&&s| t >= s).count()];
    for gun in 0..2 {
        while o.fired[gun] < due[gun] {
            o.fired[gun] += 1;
            let k = (o.fired[gun] * 7 + gun * 3) as f64;
            let muzzle = p + (rot * C130_MUZZLES[gun]).as_dvec3();
            let spread = DVec3::new((k * 1.3).sin() * 25.0, 0.0, (k * 2.1).cos() * 25.0);
            let aim = o.centre + spread;
            let speed = stealth_sim::gunship::SHELL_SPEED[gun];
            let range = (aim - muzzle).length();
            let tof = range / speed;
            let lift = DVec3::Y * (0.5 * 9.81 * tof * tof);
            let dir = (aim + lift - muzzle).normalize();
            let calibre = if gun == 0 { Calibre::Mm40 } else { Calibre::Mm105 };
            let shell = Shell {
                vel: dir * speed + v.as_dvec3(),
                calibre,
                streak: STREAK[gun],
                glow: Some(GLOW[gun]),
                age: 0.0,
                fuse: None,
            };
            fx::fire(&mut stage.commands, material.clone(), &mut ex, muzzle, v, shell, true);
        }
    }
    // Outside the circle a little behind and above the left wing's tip: the gun side, the
    // target below beyond it.
    let a = o.start + t * SPEED / RADIUS;
    let outward = Vec2::new(a.cos(), a.sin());
    let along = Vec2::new(-a.sin(), a.cos());
    let off = outward * 35.0 - along * 110.0;
    let eye = p + enu(off.x as f64, 22.0, off.y as f64);
    let at = p + (o.centre - p).normalize() * 12.0;
    stage::look(&mut cam, eye, at);
    fov.0 = 50f32.to_radians();
}

/// The AC-130's model root.
#[derive(Component)]
struct GunshipCraft;
