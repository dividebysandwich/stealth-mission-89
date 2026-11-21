//! The AC-130 in its left-hand pylon turn over a target, its side guns firing down at it: the
//! 40 mm every half second, the 105 mm now and then, their tracers glowing on the way down
//! and the bursts round the vehicles (`Shot::Gunship`); and its gunner's console, the pod's
//! FLIR picture on the vehicles as the rounds strike them (`Shot::GunnerConsole`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::cockpits::{Cabin, spawn_cabin};
use super::director::{Shot, shot_on};
use super::fx::{self, Shell};
use super::pod::{self, PodCamera};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::{AircraftKind, C130_MUZZLES};
use crate::cockpit3d::ConsoleScreen;
use crate::cockpit3d::scene::{SceneScreens, Screen, Seat};
use crate::explosions::{Blast, Calibre, Explosions};
use crate::font::{Align, stroke_text_at};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (gunship.run_if(shot_on(Shot::Gunship)), console.run_if(shot_on(Shot::GunnerConsole)))
            .in_set(ShotSet),
    );
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

/// A few vehicles round the target at `centre`: their roots and places.
fn vehicles(stage: &mut Stage, centre: DVec3) -> Vec<(Entity, DVec3)> {
    let mut out = Vec::new();
    for (i, name) in ["aaa_117", "aaa_120", "aaa_117", "aaa_110"].into_iter().enumerate() {
        let a = i as f32 * 1.7;
        let at = centre + enu((a.cos() * 30.0 * i as f32) as f64, 0.0, (a.sin() * 25.0) as f64);
        let at = DVec3::new(at.x, stage.ground(at), at.z);
        if let Some(e) = stage.model(stage::AAA, name, 0.0, crate::models::ModelState::default()) {
            stage
                .commands
                .entity(e)
                .insert((WorldPos(at), Transform::from_rotation(Quat::from_rotation_y(a))));
            out.push((e, at));
        }
    }
    out
}

/// A round of gun `gun` (0 the 40 mm, 1 the 105 mm) from the aircraft at `p` turned `rot`
/// flying `v`, at `aim`, fired `ago` seconds before now (on its way since): its time of flight
/// back. The gun is laid as the fire control lays it: for the shell's drop and for the
/// aircraft's own speed carried into it (some 100 m/s across the line of fire), so that it
/// strikes where aimed.
#[allow(clippy::too_many_arguments)]
fn fire_at(
    stage: &mut Stage,
    material: Handle<crate::palette_material::PaletteMaterial>,
    ex: &mut Explosions,
    gun: usize,
    p: DVec3,
    rot: Quat,
    v: Vec3,
    aim: DVec3,
    ago: f64,
) -> f64 {
    let muzzle = p + (rot * C130_MUZZLES[gun]).as_dvec3();
    let speed = stealth_sim::gunship::SHELL_SPEED[gun];
    // The flight's time: the muzzle speed relative to the air, the aircraft's speed with it.
    let carry = v.as_dvec3();
    let mut tof = (aim - muzzle).length() / speed;
    for _ in 0..3 {
        tof = (aim - muzzle - carry * tof).length() / speed;
    }
    let lift = DVec3::Y * (0.5 * fx::G * tof * tof);
    let vel = (aim + lift - muzzle) / tof;
    let calibre = if gun == 0 { Calibre::Mm40 } else { Calibre::Mm105 };
    let ago = ago.clamp(0.0, tof * 0.95);
    let shell = Shell {
        vel: vel - DVec3::Y * (fx::G * ago),
        calibre,
        streak: STREAK[gun],
        glow: Some(GLOW[gun]),
        age: ago,
        fuse: None,
    };
    let at = muzzle + vel * ago - DVec3::Y * (0.5 * fx::G * ago * ago);
    fx::fire(&mut stage.commands, material, ex, at, v, shell, ago < 0.05);
    tof
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
    mut tracer: Local<fx::TracerCache>,
) {
    if stage.begin(&[stage::AIRCRAFT, stage::AAA]) {
        let centre = stage.land();
        let start = (stage.director.roll % 628) as f32 * 0.01;
        if let Some(e) = stage.aircraft(AircraftKind::AC130) {
            stage.commands.entity(e).insert(GunshipCraft);
        }
        vehicles(&mut stage, centre);
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
            let spread = DVec3::new((k * 1.3).sin() * 25.0, 0.0, (k * 2.1).cos() * 25.0);
            fire_at(&mut stage, material.clone(), &mut ex, gun, p, rot, v, o.centre + spread, 0.0);
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

/// The console's pod picture (pixels: the console's camera screen's canvas, twice).
const POD: (u32, u32) = (616, 480);
/// The pod's lens (degrees high) and its zoom's name.
const POD_LENS: f32 = 2.2;
const POD_ZOOM: &str = "10X";
/// The pod under the fuselage's left side (the aircraft's frame, m).
const POD_AT: Vec3 = Vec3::new(-2.2, -2.0, -8.0);
/// The gunner's console in the cabin (the aircraft's frame, m) and its facing (yaw left, rad).
const CONSOLE_AT: Vec3 = Vec3::new(0.6, 0.6, 2.0);
const CONSOLE_YAW: f32 = 0.0;
/// The guns' magazines at the scene's start.
const MAGAZINE: [usize; 2] = [212, 44];

/// At the console the guns have been firing a while when the scene opens: the 40 mm's first
/// round went this long (s) before it, so its rounds strike from the first second on; the
/// 105 mm's shot (it strikes some 3.3 s later, on the pod's picture).
const CONSOLE_LEAD: f32 = 2.0;
const CONSOLE_HOWITZER: f32 = -0.3;

/// The console scene: the vehicles (roots, places, wrecked; the pod and the guns on the
/// first), the rounds on their way (when they strike, the 105 mm's wrecks it).
#[derive(Component)]
struct Fire {
    vehicles: Vec<(Entity, DVec3, bool)>,
    strikes: Vec<(f32, bool)>,
}

/// The AC-130 gunner's console as the gunship orbits: close on the pod's FLIR picture of the
/// vehicles, the crosshair on one, the 40 mm walking onto it and the 105 mm wrecking it; the
/// console's small screen with the orbit and the guns.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn console(
    mut stage: Stage,
    mut screens: SceneScreens,
    mut orbit: Query<(&mut Orbit, &mut Fire)>,
    mut cabins: Query<
        (&Cabin, &mut WorldPos, &mut Transform, &GlobalTransform),
        (Without<SceneCamera>, Without<PodCamera>),
    >,
    mut pods: pod::Pods,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut flir: ResMut<super::SceneFlir>,
    (mut ex, mut materials, palette): (
        ResMut<Explosions>,
        ResMut<Assets<crate::palette_material::PaletteMaterial>>,
        Res<crate::palette::Palette>,
    ),
    mut tracer: Local<fx::TracerCache>,
) {
    if stage.begin(&[stage::AAA]) {
        let centre = stage.land();
        let start = (stage.director.roll % 628) as f32 * 0.01;
        let vehicles = vehicles(&mut stage, centre).into_iter().map(|(e, p)| (e, p, false));
        let seat = Seat::Console(crate::cockpit3d::GUNNER_CONSOLE_MODEL);
        let cabin = spawn_cabin(&mut stage, Cabin::new(seat, None));
        stage.commands.entity(cabin).insert(pod::WantsPod(POD, POD_LENS.to_radians()));
        let fire = Fire { vehicles: vehicles.collect(), strikes: Vec::new() };
        stage.commands.spawn((ShotEntity, Orbit { centre, start, fired: [0, 0] }, fire));
        return;
    }
    let t = stage.director.t;
    let Ok((mut o, mut f)) = orbit.single_mut() else { return };
    let (p, rot, v) = on_circle(&o, t);
    if f.vehicles.is_empty() {
        return;
    }
    // The 40 mm walking in on the vehicle, the 105 mm on it: the rounds fired before the
    // scene opened already on their way (from where the aircraft was then).
    let material = fx::tracer_material(&mut tracer, &mut materials, &palette);
    let due = [((t + CONSOLE_LEAD) / BOFORS_EVERY) as usize, usize::from(t >= CONSOLE_HOWITZER)];
    let mark = f.vehicles[0].1;
    for gun in 0..2 {
        while o.fired[gun] < due[gun] {
            o.fired[gun] += 1;
            let shot = if gun == 1 {
                CONSOLE_HOWITZER
            } else {
                o.fired[0] as f32 * BOFORS_EVERY - CONSOLE_LEAD
            };
            let (gp, grot, gv) = on_circle(&o, shot);
            let k = (o.fired[gun] * 7 + gun * 3) as f64;
            let miss = if gun == 1 { 0.0 } else { (14.0 - 3.0 * o.fired[0] as f64).max(2.5) };
            let spread = DVec3::new((k * 1.3).sin() * miss, 0.0, (k * 2.1).cos() * miss);
            let ago = (t - shot).max(0.0) as f64;
            let tof = fire_at(
                &mut stage,
                material.clone(),
                &mut ex,
                gun,
                gp,
                grot,
                gv,
                mark + spread,
                ago,
            );
            f.strikes.push((shot + tof as f32, gun == 1));
        }
    }
    // The 105 mm's strike wrecks it.
    let struck = f.strikes.iter().any(|s| s.1 && s.0 <= t);
    f.strikes.retain(|s| s.0 > t);
    if struck && !f.vehicles[0].2 {
        f.vehicles[0].2 = true;
        stage.commands.entity(f.vehicles[0].0).insert(fx::Hit);
        ex.pending.push((mark + DVec3::Y, Blast::Kill));
    }
    // The pod on the mark, from under the left side.
    let eye = p + (rot * POD_AT).as_dvec3();
    let target = mark + DVec3::Y * 1.5;
    pod::aim_pod(&mut pods, eye, target);
    let Ok((cabin, mut wp, mut tf, gt)) = cabins.single_mut() else { return };
    let crot = rot * Quat::from_rotation_y(CONSOLE_YAW);
    (wp.0, tf.rotation) = (p + (rot * CONSOLE_AT).as_dvec3(), crot);
    if cabin.fitted.is_none() {
        return;
    }
    flir.0 = Some(POD);
    screens.all_pictures();
    let los = (target - eye).as_vec3().normalize_or(Vec3::NEG_Y);
    let range = (target - eye).length() as f32;
    let rounds = [MAGAZINE[0] - o.fired[0], MAGAZINE[1] - o.fired[1]];
    // The gun laid: the 105 mm about its shot, else the 40 mm.
    let gun = usize::from((CONSOLE_HOWITZER - 0.6..CONSOLE_HOWITZER + 0.6).contains(&t));
    let tof = range as f64 / stealth_sim::gunship::SHELL_SPEED[gun];
    if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Eo)) {
        let look = crate::gunner::gimbal_look(rot, los);
        draw_eo(&mut tex.canvas, t, gun, rounds, tof, range, &look);
        tex.transparent = true;
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Info)) {
        let alt = (p.y - stage.ground(p)) as f32;
        draw_info(&mut tex.canvas, alt, v.length(), gun, rounds);
        tex.dirty = true;
    }
    super::comanche::close_on(
        &mut cam,
        &mut fov,
        &screens,
        cabin,
        gt,
        wp.0,
        crot,
        Screen::Console(ConsoleScreen::Eo),
        52.0,
    );
}

/// The pod picture's overlay (as `gunner::draw_screen`'s): the crosshair, FLIR, the point
/// track and the zoom, the guns and the solution, the gimbal's scope.
fn draw_eo(
    cv: &mut crate::canvas::Canvas,
    t: f32,
    gun: usize,
    rounds: [usize; 2],
    tof: f64,
    range: f32,
    look: &crate::gunner::GimbalLook,
) {
    use crate::station::scope::{GREEN, WHITE, YELLOW};
    cv.reset_clip();
    cv.clear(crate::canvas::TRANSPARENT);
    let (w, h) = (cv.w as f32, cv.h as f32);
    let col = WHITE;
    let text = |cv: &mut crate::canvas::Canvas, x: f32, y: f32, s: &str, col: u8, align| {
        stroke_text_at(cv, x, y, s, h * 0.032, 1.1, col, align);
    };
    let o = Vec2::new(w / 2.0, h / 2.0);
    let arm = h * 0.12;
    for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        cv.stroke(o + d * arm * 0.25, o + d * arm, 1.4, col);
    }
    text(cv, 10.0, 14.0, "FLIR WHOT", col, Align::Left);
    text(cv, 10.0, 28.0, "PNT VEH", col, Align::Left);
    text(cv, w - 10.0, 14.0, POD_ZOOM, col, Align::Right);
    let y = h - 16.0;
    let name = stealth_sim::gunship::gun_name;
    text(cv, 10.0, y - 14.0, &format!("{}  {}", name(gun), rounds[gun]), YELLOW, Align::Left);
    let other = 1 - gun;
    text(cv, 10.0, y, &format!("{} {}", name(other), rounds[other]), col, Align::Left);
    // The 105 mm loading between its shots.
    let loading = gun == 1 && t >= CONSOLE_HOWITZER;
    let (state, scol) = if loading {
        (format!("LOADING  TOF {tof:.1}"), YELLOW)
    } else {
        (format!("READY  TOF {tof:.1}"), GREEN)
    };
    text(cv, w - 10.0, y - 14.0, &state, scol, Align::Right);
    text(cv, w - 10.0, y, &format!("RNG {:.1} KM", range / 1000.0), col, Align::Right);
    let (at, size) = (Vec2::new(w - 92.0, 24.0), Vec2::new(82.0, 70.0));
    crate::gunner::draw_gimbal_scope(cv, at, size, look, GREEN, col, true);
}

/// The console's small screen (as `gunner::draw_info`'s): the orbit and the guns.
fn draw_info(cv: &mut crate::canvas::Canvas, alt: f32, speed: f32, gun: usize, rounds: [usize; 2]) {
    use crate::station::scope::{BLACK, GREEN, YELLOW};
    cv.reset_clip();
    cv.clear(BLACK);
    let (w, h) = (cv.w as i32, cv.h as i32);
    cv.rect(1, 1, w - 2, h - 2, YELLOW);
    let line = |cv: &mut crate::canvas::Canvas, row: i32, s: &str, col: u8| {
        stroke_text_at(cv, 12.0, 22.0 + row as f32 * 26.0, s, 9.0, 1.3, col, Align::Left);
    };
    line(cv, 0, "AC-130 GUNNER", YELLOW);
    line(cv, 1, "PILOT  AI (ORBIT)", GREEN);
    line(cv, 2, &format!("ALT    {:.0} FT", alt * 3.281), GREEN);
    line(cv, 3, &format!("SPEED  {:.0} KT", speed * 1.944), GREEN);
    line(cv, 4, "ORBIT  LEFT", GREEN);
    line(cv, 5, &format!("40MM   {}", rounds[0]), if gun == 0 { YELLOW } else { GREEN });
    line(cv, 6, &format!("105MM  {}", rounds[1]), if gun == 1 { YELLOW } else { GREEN });
}
