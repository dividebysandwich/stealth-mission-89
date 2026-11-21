//! A blue aircraft firing one of its weapons, seen from beside and behind it (`Shot::Strike`):
//! an air-to-air missile at a fighter ahead, an air-to-ground missile or a retarded bomb at a
//! vehicle on the ground, or a gun burst at one.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::flyers::{Flyer, fly};
use super::fx::{self, Aim, Airborne, Missile};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotSet};
use crate::aircraft::AircraftKind as K;
use crate::explosions::{Blast, Calibre, Explosions};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(Update, strike.in_set(ShotSet).run_if(shot_on(Shot::Strike)));
}

/// What is fired.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Weapon {
    /// An air-to-air missile (store model) at a fighter ahead.
    AirMissile(&'static str),
    /// An air-to-ground missile (store model) at a vehicle.
    GroundMissile(&'static str),
    /// Retarded bombs (store model) at vehicles under the track.
    Bombs(&'static str),
    /// A gun burst at a vehicle.
    Gun,
}

/// The aircraft and what they fire.
const LOADS: [(K, Weapon); 14] = [
    (K::F16, Weapon::AirMissile("aircraft_050")),
    (K::F16, Weapon::AirMissile("aircraft_051")),
    (K::FA18E, Weapon::AirMissile("aircraft_051")),
    (K::F22, Weapon::AirMissile("aircraft_051")),
    (K::A10, Weapon::GroundMissile("aircraft_055")),
    (K::FA18E, Weapon::GroundMissile("aircraft_055")),
    (K::F16, Weapon::GroundMissile("aircraft_052")),
    (K::AH64, Weapon::GroundMissile("aircraft_069")),
    (K::AH1Z, Weapon::GroundMissile("aircraft_069")),
    (K::Tornado, Weapon::Bombs("aircraft_060")),
    (K::AV8B, Weapon::Bombs("aircraft_060")),
    (K::F117, Weapon::Bombs("aircraft_060")),
    (K::A10, Weapon::Gun),
    (K::AH64, Weapon::Gun),
];

/// When the weapon goes (seconds into the scene; a stick of bombs sooner, it takes long to
/// fall), and a gun's burst's end.
const FIRE_AT: f32 = 1.0;
const BOMBS_AT: f32 = 0.3;
const BURST_END: f32 = 3.2;
/// Rounds drawn a second (a tracer for several real ones).
const ROUNDS_PER_S: f32 = 18.0;
/// A retarded bomb's drag (1/m): it falls behind the aircraft.
const SNAKEYE_DRAG: f64 = 0.0012;

/// How far (m) a retarded bomb released level at `speed` (m/s), `height` metres up, travels
/// before it hits the ground.
fn reach(speed: f64, height: f64) -> f32 {
    let (mut v, mut p) = (bevy::math::DVec2::new(speed, 0.0), bevy::math::DVec2::new(0.0, height));
    let dt = 0.01;
    while p.y > 0.0 {
        let drag = -v * v.length() * SNAKEYE_DRAG;
        v += (drag - bevy::math::DVec2::Y * 9.81) * dt;
        p += v * dt;
    }
    p.x as f32
}

/// The scene's state: the load, the shooter, the target, what has been fired and when the
/// next round goes.
#[derive(Component)]
struct Strike {
    kind: K,
    weapon: Weapon,
    target: Option<Entity>,
    aim: DVec3,
    fired: usize,
    weapon_entity: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
fn strike(
    mut stage: Stage,
    flyers: Query<(Entity, &Flyer)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut state: Query<&mut Strike>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    (mut ex, mut materials, palette): (
        ResMut<Explosions>,
        ResMut<Assets<crate::palette_material::PaletteMaterial>>,
        Res<crate::palette::Palette>,
    ),
    mut tracer: Local<fx::TracerCache>,
) {
    if stage.begin(&[stage::AIRCRAFT, stage::AAA]) {
        spawn(&mut stage);
        return;
    }
    let t = stage.director.t;
    let Some((_, p, rot, v)) = fly(&stage, &flyers, &mut q) else { return };
    let Ok(mut s) = state.single_mut() else { return };
    let heli = s.kind.spec().helicopter;
    let len = stage::length(s.kind);
    let heading = v.x.atan2(-v.z);
    // The station under the right wing (or stub wing).
    let station = |side: f32| {
        let o =
            if heli { Vec3::new(1.6 * side, -0.9, -0.5) } else { Vec3::new(2.4 * side, -0.7, 0.0) };
        p + (rot * o).as_dvec3()
    };
    match s.weapon {
        Weapon::AirMissile(name) | Weapon::GroundMissile(name) if t >= FIRE_AT && s.fired == 0 => {
            s.fired = 1;
            let air = matches!(s.weapon, Weapon::AirMissile(_));
            let (accel, top, turn) = match name {
                "aircraft_069" => (260.0, 450.0, 0.6),
                _ if air => (320.0, 950.0, 0.5),
                _ => (180.0, 600.0, 0.4),
            };
            let aim = s.target.map_or(Aim::Point(s.aim), Aim::Thing);
            let m = Missile {
                vel: v,
                accel,
                top,
                turn,
                straight: 0.35,
                aim,
                blast: Blast::Kill,
                age: 0.0,
            };
            s.weapon_entity = fx::launch(&mut stage, name, station(1.0), v, m);
        }
        Weapon::Bombs(name) if t >= BOMBS_AT && s.fired < 4 => {
            // A stick of four, a fifth of a second apart, from alternate wings.
            let next = BOMBS_AT + s.fired as f32 * 0.2;
            if t >= next {
                let side = if s.fired % 2 == 0 { 1.0 } else { -1.0 };
                let e = fx::drop(&mut stage, name, station(side), v.as_dvec3(), SNAKEYE_DRAG);
                s.weapon_entity = s.weapon_entity.or(e);
                s.fired += 1;
            }
        }
        Weapon::Gun if (FIRE_AT..BURST_END).contains(&t) => {
            let due = ((t - FIRE_AT) * ROUNDS_PER_S) as usize;
            let material = fx::tracer_material(&mut tracer, &mut materials, &palette);
            while s.fired <= due {
                s.fired += 1;
                let muzzle = p + (rot * Vec3::new(0.0, -1.0, -len * 0.45)).as_dvec3();
                // At the target, a little spread.
                let k = s.fired as f64;
                let spread = DVec3::new((k * 1.7).sin() * 4.0, 0.0, (k * 2.3).cos() * 4.0);
                let dir = (s.aim + spread - muzzle).normalize();
                let vel = dir * 1000.0 + v.as_dvec3();
                let round = fx::round(vel, Calibre::Cannon, 0.03);
                fx::fire(&mut stage.commands, material.clone(), &mut ex, muzzle, v, round, true);
            }
        }
        _ => {}
    }
    // The burst's last rounds knock the target out.
    if s.weapon == Weapon::Gun
        && t >= BURST_END - 0.4
        && let Some(target) = s.target.take()
    {
        stage.commands.entity(target).insert(fx::Hit);
        ex.pending.push((s.aim, Blast::Kill));
    }
    if let Weapon::Bombs(_) = s.weapon {
        // By the column: the jet comes over low, the stick falls on the vehicles.
        let side = stage::right(heading) * 120.0;
        let back = stage::forward(heading) * 60.0;
        let eye = s.aim + enu((side.x - back.x) as f64, 12.0, (side.y - back.y) as f64);
        let k = ((t - 1.0) / 1.5).clamp(0.0, 1.0) as f64;
        stage::look(&mut cam, eye, p + (s.aim - p) * k);
    } else {
        // Behind the shooter, looking past it down the line to the target.
        // On its left: it shows right of the menu.
        let eye = stage::behind(p, heading, len * 2.0, -len * 0.8, len * 0.3);
        let to = (s.aim - p).normalize_or(DVec3::NEG_Z);
        let ahead = if heli { 1.5 } else { 4.0 };
        stage::look(&mut cam, eye, p + to * (len * ahead) as f64);
    }
    fov.0 = 45f32.to_radians();
}

/// A ground target's run: the target on open ground (no town, no woods round it), the heading
/// the shooter comes in on chosen so that it clears the hills on its way and sees the target
/// from where it fires (no hill or building in between); the heading, the shooter's start and
/// the target's place.
fn ground_run(
    stage: &Stage,
    first: f32,
    agl: f32,
    speed: f32,
    ahead: f32,
    climb: f32,
) -> (f32, DVec3, DVec3) {
    let target_at = stage.land();
    let aim = target_at + DVec3::Y * 1.5;
    let run = |heading: f32| {
        let f = stage::forward(heading);
        let back = enu((-f.x * ahead) as f64, 0.0, (-f.y * ahead) as f64);
        let start = target_at + back;
        // As high over the ground as it should be at the start, and over the target too.
        let start = DVec3::new(start.x, stage.ground(start).max(target_at.y) + agl as f64, start.z);
        (start, f)
    };
    let good = |heading: f32| {
        let (start, f) = run(heading);
        // The track to the target (and a little past it) above the hills.
        let flown = ahead / speed + 1.0;
        let path_ok = (0..=40).all(|i| {
            let t = flown * i as f32 / 40.0;
            let along = f * speed * t;
            let p = start + enu(along.x as f64, (climb * t) as f64, along.y as f64);
            p.y - stage.ground(p) > (agl as f64 * 0.4).min(25.0)
        });
        // In sight from where it fires and from the camera behind it.
        let fire = start + enu((f.x * speed * FIRE_AT) as f64, 0.0, (f.y * speed * FIRE_AT) as f64);
        let eye = stage::behind(fire, heading, 40.0, 0.0, 6.0);
        path_ok && stage.sight_clear(fire, aim, 3.0) && stage.sight_clear(eye, aim, 3.0)
    };
    // Round the compass from the scene's own heading until a run is clear.
    let heading = (0..24)
        .map(|i| first + i as f32 * std::f32::consts::TAU / 24.0)
        .find(|&h| good(h))
        .unwrap_or(first);
    (heading, run(heading).0, target_at)
}

/// The shooter on its way to its target, the target there.
fn spawn(stage: &mut Stage) {
    let (kind, weapon) = LOADS[stage.director.pick(LOADS.len())];
    let heli = kind.spec().helicopter;
    // Where the shooter is 0 s in, how high and fast, and where the target stands (metres
    // along the track from the start).
    let (agl, speed, ahead) = match weapon {
        Weapon::AirMissile(_) => (2500.0, 220.0, 1100.0),
        Weapon::GroundMissile(_) if heli => (40.0, 15.0, 1000.0),
        Weapon::GroundMissile(_) => (600.0, 190.0, 1400.0),
        // The stick's middle falls where it reaches from its release (`SNAKEYE_DRAG`).
        Weapon::Bombs(_) => (50.0, 220.0, 220.0 * (BOMBS_AT + 0.3) + reach(220.0, 50.0)),
        Weapon::Gun if heli => (50.0, 20.0, 900.0),
        Weapon::Gun => (500.0, 150.0, 1300.0),
    };
    // Diving at a ground target with the gun.
    let climb = match weapon {
        Weapon::Gun if !heli => -(agl - 60.0) / (ahead / speed),
        _ => 0.0,
    };
    let first = ((stage.director.roll >> 8) % 360) as f32 * 0.0175;
    let (heading, start, target_at) = if matches!(weapon, Weapon::AirMissile(_)) {
        let ground = stage.land();
        let f = stage::forward(first);
        let target_at = ground + enu((f.x * ahead) as f64, 0.0, (f.y * ahead) as f64);
        (first, ground + DVec3::Y * agl as f64, target_at)
    } else {
        ground_run(stage, first, agl, speed, ahead, climb)
    };
    let f = stage::forward(heading);
    let weave = Weave {
        start,
        heading,
        speed,
        amp: Weave::amp_for(10.0, if heli { 5.0 } else { 15.0 }),
        period: 10.0,
        phase: 0.0,
        climb,
    };
    let Some(shooter) = stage.aircraft(kind) else { return };
    stage.commands.entity(shooter).insert(Flyer::lead(weave, None));
    let mut target = None;
    let aim = match weapon {
        Weapon::AirMissile(_) => {
            // A MiG-29 crossing ahead, a little higher.
            let side = stage::right(heading) * 150.0;
            let at =
                start + enu((f.x * ahead + side.x) as f64, 80.0, (f.y * ahead + side.y) as f64);
            let mig = Weave {
                start: at,
                heading: heading + 0.6,
                speed: 230.0,
                amp: Weave::amp_for(8.0, 30.0),
                period: 8.0,
                phase: 0.0,
                climb: 0.0,
            };
            if let Some(e) = stage.model(stage::AIRCRAFT, "aircraft_016", 0.0, stage::flying()) {
                let (_, _, v) = mig.at(0.0);
                stage.commands.entity(e).insert((Flyer::other(mig, None), Airborne(v)));
                target = Some(e);
            }
            at
        }
        _ => {
            // A few vehicles of a column on the road, the first one the target.
            let names = ["aaa_117", "aaa_110", "aaa_120"];
            for (i, name) in names.into_iter().enumerate() {
                let back = f * (i as f32 * 25.0);
                let at = target_at - enu(back.x as f64, 0.0, back.y as f64);
                let at = DVec3::new(at.x, stage.ground(at), at.z);
                if let Some(e) =
                    stage.model(stage::AAA, name, 0.0, crate::models::ModelState::default())
                {
                    stage.commands.entity(e).insert((
                        WorldPos(at),
                        Transform::from_rotation(stage::attitude(heading + 0.4, 0.0, 0.0)),
                    ));
                    if i == 0 {
                        target = Some(e);
                    }
                }
            }
            target_at + DVec3::Y * 1.5
        }
    };
    stage.commands.spawn((
        super::ShotEntity,
        Strike { kind, weapon, target, aim, fired: 0, weapon_entity: None },
    ));
}
