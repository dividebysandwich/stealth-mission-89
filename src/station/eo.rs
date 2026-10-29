//! The console's camera (the tracking camera, `trackcam`, on the radar's or the Gepard's mast):
//! it follows the locked track where it is now (interpolated every frame like the drawn
//! aircraft, not the radar picture's twice a second), slewing at its gimbal's rate. Colour or
//! thermal (the FLIR of the aircraft's pod). Without a lock it holds where it looks; a
//! Gepard's looks along the gun, slewed by hand (ahead of the simulation's steps with the
//! hand controller, drawn to where the gun is), each aircraft in reach marked with its lead.

use std::f32::consts::{PI, TAU};

use bevy::math::DVec3;
use bevy::prelude::*;

use super::scope::{GREEN, RED, WHITE, YELLOW};
use super::{ConsoleUi, Manning};
use crate::canvas::{Canvas, CanvasTexture};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, EO_CANVAS};
use crate::combat::{Combat, unit_position};
use crate::font::{Align, stroke_text_at};
use crate::world::{WU, wu_to_bevy};

/// The gimbal's fastest slew (rad/s) and how long it takes to get to it (s).
const SLEW_RATE: f32 = 1.6;
const SLEW_RAMP: f32 = 0.3;
/// The camera's height on its mast (m).
const MAST: f64 = 6.0;
/// How fast the camera slewed by hand is drawn to the gun's direction (1/s).
const TO_GUN: f32 = 4.0;

/// Where the console's camera looks (`trackcam` renders it).
#[derive(Resource, Clone, Copy)]
pub struct ConsoleEo {
    /// The camera (Bevy world) and its direction.
    pub eye: DVec3,
    pub dir: Vec3,
    /// Zoom step (`mfd::CAM_ZOOMS`).
    pub zoom: u8,
    /// The locked track (Bevy world), while it is there.
    pub target: Option<DVec3>,
    /// The slew's speed now (0..1 of the fastest), and while it is still slewing (0 once on
    /// the track: the gimbal's motor sound).
    pub(super) speed: f32,
    pub slew: f32,
    /// Slewed by hand: the gun's direction as the camera has it (azimuth, elevation).
    pub(super) manual: Option<Vec2>,
    /// Nothing shows the picture (a drone command center without a drone selected): it is not
    /// rendered.
    pub idle: bool,
    /// The drone command center's wide picture (`trackcam::WIDE`).
    pub wide: bool,
}

/// The camera points at the lock.
pub(super) fn aim(
    mut commands: Commands,
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    mut ui: ResMut<ConsoleUi>,
    eo: Option<ResMut<ConsoleEo>>,
    time: Res<Time<Real>>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c) else { return };
    // A drone command center's camera is on its drone (`drone::feed`).
    if m.drones() {
        return;
    }
    let sim = &combat.sim;
    let Some(u) = sim.aaa_units().get(m.unit) else { return };
    let eye =
        crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0) + DVec3::Y * MAST;
    let lock = sim.station_of(c.me.0).and_then(|s| s.lock).map(|l| l as usize);
    let target = lock.and_then(|i| {
        let f = sim.flights().get(i)?;
        let (e, n, z) = unit_position(combat.prev_flights.get(i), f, combat.alpha);
        Some(wu_to_bevy(e, n, z))
    });
    let Some(mut eo) = eo else {
        let dir = Vec3::new((u.aim_az).sin(), 0.05, -(u.aim_az).cos()).normalize();
        commands.insert_resource(ConsoleEo {
            eye,
            dir,
            zoom: crate::mfd::CAM_ZOOM_DEFAULT,
            target,
            speed: 0.0,
            slew: 0.0,
            manual: None,
            idle: false,
            wide: false,
        });
        return;
    };
    let dt = time.delta_secs();
    eo.eye = eye;
    eo.target = target;
    if m.gepard() && lock.is_none() {
        let wrap = |a: f32| (a + PI).rem_euclid(TAU) - PI;
        let (lo, hi) = stealth_sim::station::GEPARD_EL;
        let gun = Vec2::new(u.aim_az, u.aim_el);
        let mut a = eo.manual.unwrap_or(gun) + ui.slew * dt;
        let k = 1.0 - (-TO_GUN * dt).exp();
        a.x = wrap(a.x + wrap(gun.x - a.x) * k);
        a.y = (a.y + (gun.y - a.y) * k).clamp(lo, hi);
        eo.manual = Some(a);
        eo.dir = Vec3::new(a.x.sin() * a.y.cos(), a.y.sin(), -a.x.cos() * a.y.cos());
        let rate = stealth_sim::station::GEPARD_TRAVERSE;
        eo.speed = 0.0;
        eo.slew = (ui.slew.length() / rate).min(1.0);
        eo.zoom = ui.zoom.unwrap_or(crate::mfd::CAM_ZOOM_DEFAULT);
        return;
    }
    eo.manual = None;
    if let Some(t) = target {
        let want = (t - eye).as_vec3().normalize_or(eo.dir);
        let angle = eo.dir.angle_between(want);
        eo.speed = (eo.speed + dt / SLEW_RAMP).min(1.0);
        let step = SLEW_RATE * eo.speed * dt;
        eo.slew = if angle <= step { 0.0 } else { eo.speed };
        if angle <= step {
            eo.dir = want;
        } else {
            let axis = eo.dir.cross(want).normalize_or(Vec3::Y);
            eo.dir = Quat::from_axis_angle(axis, step) * eo.dir;
        }
        // A new lock: zoomed to see an aircraft well (the operator's zoom after that).
        if ui.zoom.is_none() {
            let range = (t - eye).length() as f32;
            ui.zoom = Some(crate::mfd::cam_zoom_for(range * 0.6 / 256.0));
        }
    } else {
        eo.speed = 0.0;
        eo.slew = 0.0;
    }
    eo.zoom = ui.zoom.unwrap_or(crate::mfd::CAM_ZOOM_DEFAULT);
}

/// The camera's overlay: the reticle, the lock, the picture's mode.
pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    eo: Option<Res<ConsoleEo>>,
    mfd: Res<crate::mfd::MfdState>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat), Some(c), Some(eo)) = (manning, combat, c, eo) else { return };
    if m.drones() {
        return;
    }
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Eo) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(crate::canvas::TRANSPARENT);
    let (w, h) = (EO_CANVAS.0 as f32, EO_CANVAS.1 as f32);
    let col = if mfd.flir { WHITE } else { GREEN };
    let o = Vec2::new(w / 2.0, h / 2.0);
    for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        cv.stroke(o + d * 8.0, o + d * 28.0, 1.2, col);
    }
    stroke_text_at(
        cv,
        8.0,
        10.0,
        if mfd.flir { "FLIR WHOT" } else { "TV" },
        7.0,
        1.0,
        col,
        Align::Left,
    );
    stroke_text_at(
        cv,
        w - 8.0,
        10.0,
        crate::mfd::cam_zoom_label(Some(eo.zoom)),
        7.0,
        1.0,
        col,
        Align::Right,
    );
    let sim = &combat.sim;
    let lock = sim.station_of(c.me.0).and_then(|s| s.lock);
    let bottom = h - 10.0;
    let Some(t) = eo.target else {
        if m.gepard() && lock.is_none() {
            // Slewed by hand: where the gun points, and while it fires.
            let d = eo.dir;
            let brg = d.x.atan2(-d.z).to_degrees().rem_euclid(360.0);
            let elv = d.y.atan2(d.x.hypot(d.z)).to_degrees();
            leads(cv, &combat, m.unit, &eo, col);
            let text = format!("MAN B{brg:03.0} E{elv:+.0}");
            stroke_text_at(cv, w / 2.0, bottom, &text, 7.0, 1.0, col, Align::Centre);
            if sim.aaa_units().get(m.unit).is_some_and(|u| u.burst > 0) {
                stroke_text_at(cv, w / 2.0, 24.0, "FIRING", 7.0, 1.0, YELLOW, Align::Centre);
            }
            return;
        }
        let text = if lock.is_some() { "TRACK LOST" } else { "NO LOCK" };
        stroke_text_at(
            cv,
            w / 2.0,
            bottom,
            text,
            7.0,
            1.0,
            if lock.is_some() { RED } else { col },
            Align::Centre,
        );
        return;
    };
    // On the track: its gate; slewing: where it is.
    let on = eo.dir.angle_between((t - eo.eye).as_vec3()) < 0.01;
    if on {
        let g = 18.0;
        for (a, b) in [
            (Vec2::new(-g, -g), Vec2::new(-g + 6.0, -g)),
            (Vec2::new(-g, -g), Vec2::new(-g, -g + 6.0)),
        ] {
            for s in [
                Vec2::new(1.0, 1.0),
                Vec2::new(-1.0, 1.0),
                Vec2::new(1.0, -1.0),
                Vec2::new(-1.0, -1.0),
            ] {
                cv.stroke(o + a * s, o + b * s, 1.2, col);
            }
        }
    }
    let range = (t - eo.eye).length() as f32;
    let d = (t - eo.eye).as_vec3();
    let brg = d.x.atan2(-d.z).to_degrees().rem_euclid(360.0);
    let elv = d.y.atan2(d.x.hypot(d.z)).to_degrees();
    let tn = lock.map_or(String::new(), |l| format!("{:04}", 1000 + l as usize));
    let state = if on { "TRK" } else { "SLEW" };
    stroke_text_at(cv, 8.0, bottom, &format!("{state} {tn}"), 7.0, 1.0, col, Align::Left);
    stroke_text_at(
        cv,
        w - 8.0,
        bottom,
        &format!("{:.1} KM", range / 1000.0),
        7.0,
        1.0,
        col,
        Align::Right,
    );
    stroke_text_at(
        cv,
        w / 2.0,
        bottom,
        &format!("B{brg:03.0} E{elv:+.0}"),
        7.0,
        1.0,
        col,
        Align::Centre,
    );
    if m.gepard() {
        let reach = stealth_sim::station::GEPARD_REACH as f32;
        let (text, c2) = if range <= reach { ("IN RANGE", YELLOW) } else { ("OUT OF RANGE", RED) };
        stroke_text_at(cv, w / 2.0, 24.0, text, 7.0, 1.0, c2, Align::Centre);
    }
}

/// Each aircraft in the picture within reach of the hand-fired rounds: a line from it to its
/// lead, where the barrels must point for a burst to meet it (the simulation's lead, the
/// shells' drop included).
fn leads(cv: &mut Canvas, combat: &Combat, unit: usize, eo: &ConsoleEo, col: u8) {
    use stealth_sim::station::{GEPARD_V0, MANUAL_RANGE};
    let sim = &combat.sim;
    let Some(gun) = sim.aaa_units().get(unit).map(|u| u.metres()) else { return };
    let (w, h) = (EO_CANVAS.0 as f32, EO_CANVAS.1 as f32);
    // The camera's projection (`trackcam`): f = 256 times the zoom over its 77 pixels' width,
    // vertically 0.75 f.
    let f = 256.0 * crate::mfd::cam_magnification(eo.zoom) * w / 77.0;
    let up = if eo.dir.y.abs() > 0.99 { Vec3::NEG_Z } else { Vec3::Y };
    let right = eo.dir.cross(up).normalize();
    let up = right.cross(eo.dir);
    let screen = |v: Vec3| {
        let z = v.dot(eo.dir);
        let p = Vec2::new(w / 2.0 + v.dot(right) / z * f, h / 2.0 - v.dot(up) / z * f * 0.75);
        (z > 0.0 && p.x.abs() < 2.0 * w && p.y.abs() < 2.0 * h).then_some(p)
    };
    let shown = |p: Vec2| p.x >= 0.0 && p.x < w && p.y >= 0.0 && p.y < h;
    for (i, u) in sim.flights().iter().enumerate() {
        // Drawn where it is shown, led from where the simulation has it (the rounds fly there).
        let Some((pos, vel)) = sim.track_alive(i) else { continue };
        let (e, n, z) = unit_position(combat.prev_flights.get(i), u, combat.alpha);
        // Its velocity over the last simulation step (the AI flies slower than its nominal
        // speed in a bank), else the nominal one.
        let vel = combat
            .prev_flights
            .get(i)
            .filter(|p| {
                p.flags == u.flags && (p.wx - u.wx).abs() < 0x400 && (p.wy - u.wy).abs() < 0x400
            })
            .map_or(vel, |p| {
                let k = WU * crate::combat::FS as f64;
                [(u.wx - p.wx) as f64 * k, (p.wy - u.wy) as f64 * k, (u.alt - p.alt) as f64 * k]
            });
        let (az, el, time) = stealth_sim::aaa::lead(gun, pos, vel, GEPARD_V0);
        if time * GEPARD_V0 > MANUAL_RANGE {
            continue;
        }
        let (az, el) = (az as f32, el as f32);
        let aim = Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos());
        let (Some(p), Some(q)) = (screen((wu_to_bevy(e, n, z) - eo.eye).as_vec3()), screen(aim))
        else {
            continue;
        };
        if shown(p) || shown(q) {
            cv.dashed(p, q, 3.0, 3.0, 1.0, col);
            cv.ring(q, 5.0, 1.2, col);
        }
    }
}
