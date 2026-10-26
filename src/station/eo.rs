//! The console's camera (the tracking camera, `trackcam`, on the radar's or the Gepard's mast):
//! it follows the locked track where it is now (interpolated every frame like the drawn
//! aircraft, not the radar picture's twice a second), slewing at its gimbal's rate. Colour or
//! thermal (the FLIR of the aircraft's pod). Without a lock it holds where it looks; a
//! Gepard's looks along the gun, slewed by hand (ahead of the simulation's steps with the
//! hand controller, drawn to where the gun is).

use std::f32::consts::{PI, TAU};

use bevy::math::DVec3;
use bevy::prelude::*;

use super::scope::{GREEN, RED, WHITE, YELLOW};
use super::{ConsoleUi, Manning};
use crate::canvas::CanvasTexture;
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, EO_CANVAS};
use crate::combat::{Combat, unit_position};
use crate::font::{Align, stroke_text_at};
use crate::world::wu_to_bevy;

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
    /// Zoom shift (focal length 256 << zoom).
    pub zoom: u8,
    /// The locked track (Bevy world), while it is there.
    pub target: Option<DVec3>,
    /// The slew's speed now (0..1 of the fastest), and while it is still slewing (0 once on
    /// the track: the gimbal's motor sound).
    speed: f32,
    pub slew: f32,
    /// Slewed by hand: the gun's direction as the camera has it (azimuth, elevation).
    manual: Option<Vec2>,
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
            zoom: 2,
            target,
            speed: 0.0,
            slew: 0.0,
            manual: None,
        });
        return;
    };
    let dt = time.delta_secs();
    eo.eye = eye;
    eo.target = target;
    if m.gepard && lock.is_none() {
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
        eo.zoom = ui.zoom.unwrap_or(2);
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
            ui.zoom = Some((range * 0.6 / 256.0).max(1.0).log2().round().clamp(0.0, 6.0) as u8);
        }
    } else {
        eo.speed = 0.0;
        eo.slew = 0.0;
    }
    eo.zoom = ui.zoom.unwrap_or(2);
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
    stroke_text_at(cv, w - 8.0, 10.0, &format!("Z{}", eo.zoom + 1), 7.0, 1.0, col, Align::Right);
    let sim = &combat.sim;
    let lock = sim.station_of(c.me.0).and_then(|s| s.lock);
    let bottom = h - 10.0;
    let Some(t) = eo.target else {
        if m.gepard && lock.is_none() {
            // Slewed by hand: where the gun points, and while it fires.
            let d = eo.dir;
            let brg = d.x.atan2(-d.z).to_degrees().rem_euclid(360.0);
            let elv = d.y.atan2(d.x.hypot(d.z)).to_degrees();
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
    if m.gepard {
        let reach = stealth_sim::station::GEPARD_REACH as f32;
        let (text, c2) = if range <= reach { ("IN RANGE", YELLOW) } else { ("OUT OF RANGE", RED) };
        stroke_text_at(cv, w / 2.0, 24.0, text, 7.0, 1.0, c2, Align::Centre);
    }
}
