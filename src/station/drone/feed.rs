//! The selected drone's camera: the console's camera (`ConsoleEo`, rendered by `trackcam`)
//! rides on the drone. As a TGP it looks at a ground point: slaved, at the next waypoint (or
//! ahead and down when that is far), tracking, at the point the stick put it on. Brackets mark
//! the enemy air defences the drone sees (a surveillance drone all within its sensor's reach,
//! an attack drone those within 2 km). Flown by hand it looks ahead, under a flight display.
//! Without a drone selected the camera rests.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;
use stealth_sim::aaa::M_PER_MAP;
use stealth_sim::uav::{Uav, UavMode};
use stealth_sim::{Sim, oflags};

use super::super::eo::ConsoleEo;
use super::super::scope::{BLACK, CYAN, GREEN, GREY, RED, WHITE, YELLOW};
use super::{DroneUi, center_uavs, selected};
use crate::canvas::{Canvas, CanvasTexture, TRANSPARENT};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, FEED_CANVAS};
use crate::combat::{Combat, unit_position};
use crate::font::{Align, stroke_text_at};
use crate::station::Manning;
use crate::world::wu_to_bevy;

/// A slaved camera looks at the next waypoint when it is this close (m), else ahead and down
/// this far for each metre of height (at least `AHEAD_MIN` m).
const SLAVE_REACH: f64 = 12_000.0;
const AHEAD: f64 = 3.0;
const AHEAD_MIN: f64 = 2500.0;
/// Flown by hand, the camera looks this far under the nose (rad).
const NOSE_DOWN: f64 = 0.12;
/// The camera under the drone (m).
const BELOW: f64 = 0.4;

/// The drone's position as drawn (Bevy world) and its heading and pitch (rad).
fn drone_pose(combat: &Combat, u: &Uav) -> Option<(DVec3, f64, f64)> {
    let i = u.flight as usize;
    let f = combat.sim.flights().get(i)?;
    let (e, n, z) = unit_position(combat.prev_flights.get(i), f, combat.alpha);
    Some((wu_to_bevy(e, n, z) - DVec3::Y * BELOW, u.heading, u.pitch))
}

/// A map point on the ground (Bevy world; the middle of its map unit, where the units stand).
fn ground(p: DVec2) -> DVec3 {
    crate::sim_render::map_point(p.x + 0.5, p.y + 0.5, 0.0)
}

/// Where a slaved camera looks (map units).
fn slaved(u: &Uav) -> DVec2 {
    let (x, y) = u.map();
    let here = DVec2::new(x as f64, y as f64);
    if let Some(w) = u.target() {
        let p = DVec2::new(w.x as f64, w.y as f64);
        if (p - here).length() * M_PER_MAP < SLAVE_REACH {
            return p;
        }
    }
    let d = (u.pos[2] * AHEAD).max(AHEAD_MIN) / M_PER_MAP;
    here + DVec2::new(u.heading.sin(), -u.heading.cos()) * d
}

/// The camera follows the selected drone.
pub(super) fn aim(
    mut commands: Commands,
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut ui: ResMut<DroneUi>,
    eo: Option<ResMut<ConsoleEo>>,
    controls: Res<crate::input::FlightControls>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    if !m.drones() {
        return;
    }
    let uavs = center_uavs(&combat, &m);
    let u = selected(&ui, &uavs).cloned();
    let pose = u.as_ref().and_then(|u| drone_pose(&combat, u));
    let (eye, dir, zoom, target, idle) = match (u, pose) {
        (Some(u), Some((eye, heading, pitch))) => {
            let cam = ui.cam(u.id);
            if cam.forward {
                let p = pitch - NOSE_DOWN;
                let dir = Vec3::new(
                    (heading.sin() * p.cos()) as f32,
                    p.sin() as f32,
                    (-heading.cos() * p.cos()) as f32,
                );
                (eye, dir, 0, None, false)
            } else {
                let look = cam.track.unwrap_or_else(|| slaved(&u));
                ui.cam_mut(u.id).look = Some(look);
                let at = ground(look);
                let dir = (at - eye).as_vec3().normalize_or(Vec3::NEG_Y);
                (eye, dir, cam.zoom, Some(at), false)
            }
        }
        _ => {
            let at = combat.sim.aaa_units().get(m.unit).map_or(DVec3::ZERO, |u| {
                crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0)
            });
            (at + DVec3::Y * 5.0, Vec3::NEG_Z, crate::mfd::CAM_ZOOM_DEFAULT, None, true)
        }
    };
    // The camera's picture is shown in the corner of the map or on the main screen.
    let idle = idle || ui.selected.is_none();
    let slew = (controls.roll.abs().max(controls.pitch.abs())).min(1.0);
    match eo {
        Some(mut eo) => {
            eo.eye = eye;
            eo.dir = dir;
            eo.zoom = zoom;
            eo.target = target;
            eo.slew = if idle { 0.0 } else { slew };
            eo.idle = idle;
        }
        None => commands.insert_resource(ConsoleEo {
            eye,
            dir,
            zoom,
            target,
            speed: 0.0,
            slew: 0.0,
            manual: None,
            idle,
            wide: true,
        }),
    }
}

/// An enemy air defence unit or SAM site the drone sees: where (Bevy world) and what.
fn seen(sim: &Sim, u: &Uav) -> Vec<(DVec3, String)> {
    let r = u.kind.sensor();
    let near = |e: f64, n: f64| (e - u.pos[0]).hypot(n - u.pos[1]) < r;
    let mut out = Vec::new();
    for (k, a) in sim.aaa_units().iter().enumerate() {
        let m = a.metres();
        if !a.destroyed && !sim.aaa_blue(k) && near(m[0], m[1]) {
            let at = crate::sim_render::map_point(a.x as f64 + 0.5, a.y as f64 + 0.5, 0.0);
            out.push((at, a.kind.name().to_string()));
        }
    }
    for o in sim.objects().iter() {
        let hostile = o.flags & (oflags::DESTROYED | oflags::FRIENDLY | oflags::NEUTRAL) == 0;
        let (e, n) = (o.x as f64 * M_PER_MAP, (0x8000 - o.y) as f64 * M_PER_MAP);
        if o.unit_type != 0 && hostile && near(e, n) {
            let at = crate::sim_render::map_point(o.x as f64 + 0.5, o.y as f64 + 0.5, 0.0);
            out.push((at, sim.threat_name(o.unit_type).to_string()));
        }
    }
    out
}

/// The main screen: the camera's picture under its overlay (the TGP's, or the forward FLIR's
/// horizon), the speed and height ladders either side and the HSI at the bottom while a drone
/// is selected; without its picture here, NO FEED.
pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    ui: Res<DroneUi>,
    eo: Option<Res<ConsoleEo>>,
    mfd: Res<crate::mfd::MfdState>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    if !m.drones() {
        return;
    }
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Feed) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let uavs = center_uavs(&combat, &m);
    let Some(u) = selected(&ui, &uavs) else {
        cv.clear(BLACK);
        stroke_text_at(cv, w / 2.0, h / 2.0 - 8.0, "NO FEED", 9.0, 1.2, GREEN, Align::Centre);
        let hint = "SELECT A DRONE ON THE MAP";
        stroke_text_at(cv, w / 2.0, h / 2.0 + 10.0, hint, 6.0, 0.9, GREY, Align::Centre);
        return;
    };
    let cam = ui.cam(u.id);
    let col = if mfd.flir { WHITE } else { GREEN };
    let picture = ui.on_main && eo.is_some();
    if picture {
        cv.clear(TRANSPARENT);
    } else {
        cv.clear(BLACK);
        stroke_text_at(cv, w / 2.0, h / 2.0 - 10.0, "NO FEED", 9.0, 1.2, GREEN, Align::Centre);
    }
    // The top line: the drone, the camera, the mode; the battery under it.
    let sensor = if mfd.flir { "FLIR" } else { "TV" };
    let left = if cam.forward {
        format!("{} FWD {sensor}", u.label())
    } else {
        format!("{} TGP {sensor}", u.label())
    };
    stroke_text_at(cv, 46.0, 9.0, &left, 6.0, 0.9, col, Align::Left);
    let mode = match (u.mode, cam.forward) {
        (UavMode::Manual, _) => format!("MANUAL THR {:.0}%", u.ctrl[2] * 100.0),
        (m, true) => m.name().to_string(),
        (m, false) => {
            format!("{} {}", if cam.track.is_some() { "TRACK" } else { "SLAVE" }, m.name())
        }
    };
    let mcol = if u.mode == UavMode::Manual { YELLOW } else { col };
    stroke_text_at(cv, w - 46.0, 9.0, &mode, 6.0, 0.9, mcol, Align::Right);
    let bat = (u.battery * 100.0).round() as i32;
    let end = u.endurance() as i32;
    let bcol = if bat < 20 { RED } else { col };
    let battery = format!("BAT {bat}% {}:{:02}", end / 3600, end / 60 % 60);
    stroke_text_at(cv, 46.0, 20.0, &battery, 5.0, 0.8, bcol, Align::Left);
    if u.mode == UavMode::Dive {
        stroke_text_at(cv, w / 2.0, 32.0, "TERMINAL DIVE", 7.0, 1.0, RED, Align::Centre);
    }
    if let Some(eo) = eo.filter(|_| picture) {
        if cam.forward {
            horizon(cv, u, &eo, col);
        } else {
            tgp(cv, &combat.sim, u, &eo, col);
        }
    }
    ladders(cv, u, col);
    // Flying by the forward view: a heading tape on top and the waypoint at the bottom; else
    // the HSI.
    if cam.forward {
        heading_tape(cv, u, col);
        waypoint_line(cv, u);
    } else {
        hsi(cv, u, col);
    }
}

/// The TGP's overlay: the crosshair, the zoom, the range to the ground point, brackets on the
/// enemy air defences the drone sees.
fn tgp(cv: &mut Canvas, sim: &Sim, u: &Uav, eo: &ConsoleEo, col: u8) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let o = Vec2::new(w / 2.0, h / 2.0);
    for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        cv.stroke(o + d * 6.0, o + d * 22.0, 1.2, col);
    }
    stroke_text_at(
        cv,
        w - 46.0,
        20.0,
        crate::mfd::cam_zoom_label(Some(eo.zoom)),
        5.0,
        0.8,
        col,
        Align::Right,
    );
    if let Some(t) = eo.target {
        let km = (t - eo.eye).length() / 1000.0;
        stroke_text_at(cv, o.x + 26.0, o.y + 12.0, &format!("{km:.1}"), 6.0, 0.9, col, Align::Left);
    }
    let project = projection(eo);
    for (at, name) in seen(sim, u) {
        let Some(p) = project((at - eo.eye).as_vec3()) else { continue };
        if p.x < 46.0 || p.y < 4.0 || p.x > w - 46.0 || p.y > h - 4.0 {
            continue;
        }
        bracket(cv, p, 7.0, RED);
        stroke_text_at(cv, p.x, p.y - 12.0, &name, 5.0, 0.8, RED, Align::Centre);
    }
}

/// The ladders: the speed (knots) on the left, the height (m) on the right, the value boxed at
/// the middle.
fn ladders(cv: &mut Canvas, u: &Uav, col: u8) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let (top, bottom) = (30.0, h - 30.0);
    let mid = (top + bottom) / 2.0;
    let half = (bottom - top) / 2.0;
    let tape = |cv: &mut Canvas,
                x0: f32,
                x1: f32,
                right: bool,
                value: f32,
                per_px: f32,
                tick: f32,
                label: f32| {
        cv.stroke(
            Vec2::new(if right { x0 } else { x1 }, top),
            Vec2::new(if right { x0 } else { x1 }, bottom),
            1.0,
            col,
        );
        let lo = ((value - half * per_px) / tick).ceil() as i32;
        let hi = ((value + half * per_px) / tick).floor() as i32;
        for k in lo..=hi {
            let v = k as f32 * tick;
            let y = mid - (v - value) / per_px;
            let major = (v / label).fract().abs() < 1e-3;
            let len = if major { 6.0 } else { 3.0 };
            let (a, b) = if right { (x0, x0 + len) } else { (x1 - len, x1) };
            cv.stroke(Vec2::new(a, y), Vec2::new(b, y), 1.0, col);
            if major && (y - mid).abs() > 8.0 && v >= 0.0 {
                let (tx, al) =
                    if right { (x0 + 8.0, Align::Left) } else { (x1 - 8.0, Align::Right) };
                stroke_text_at(cv, tx, y, &format!("{v:.0}"), 5.0, 0.8, col, al);
            }
        }
        // The value boxed at the middle.
        let (bx0, bx1) = (x0 - 2.0, x1 + 2.0);
        cv.fill_rect(bx0 as i32, mid as i32 - 6, (bx1 - bx0) as i32, 12, BLACK);
        cv.frame(Vec2::new(bx0, mid - 6.0), Vec2::new(bx1, mid + 6.0), 1.0, col);
        stroke_text_at(
            cv,
            (bx0 + bx1) / 2.0,
            mid,
            &format!("{value:.0}"),
            6.0,
            0.9,
            col,
            Align::Centre,
        );
    };
    let kt = (u.speed / 0.514) as f32;
    tape(cv, 4.0, 40.0, false, kt, 0.5, 5.0, 20.0);
    stroke_text_at(cv, 22.0, bottom + 9.0, "KT", 5.0, 0.8, col, Align::Centre);
    tape(cv, w - 40.0, w - 4.0, true, u.pos[2] as f32, 2.5, 25.0, 100.0);
    stroke_text_at(cv, w - 22.0, bottom + 9.0, "M", 5.0, 0.8, col, Align::Centre);
}

/// The bearing (rad) and distance (km) to the next waypoint.
fn to_waypoint(u: &Uav) -> Option<(f32, f64)> {
    let m = u.target()?.metres();
    let (de, dn) = (m[0] - u.pos[0], m[1] - u.pos[1]);
    Some((de.atan2(dn) as f32, de.hypot(dn) / 1000.0))
}

/// The heading tape across the top: ticks every 5 degrees, numbers every 10 (the cardinal
/// points by letter), the heading boxed at the middle, the next waypoint's bearing marked.
fn heading_tape(cv: &mut Canvas, u: &Uav, col: u8) {
    let w = FEED_CANVAS.0 as f32;
    let (y, half, per_deg) = (40.0, 80.0, 2.0);
    let c = w / 2.0;
    let hdg = u.heading.to_degrees().rem_euclid(360.0) as f32;
    cv.stroke(Vec2::new(c - half, y), Vec2::new(c + half, y), 1.0, col);
    let lo = ((hdg - half / per_deg) / 5.0).ceil() as i32;
    let hi = ((hdg + half / per_deg) / 5.0).floor() as i32;
    for k in lo..=hi {
        let deg = k * 5;
        let x = c + (deg as f32 - hdg) * per_deg;
        let major = deg % 10 == 0;
        cv.stroke(Vec2::new(x, y), Vec2::new(x, y + if major { 5.0 } else { 3.0 }), 1.0, col);
        if major && (x - c).abs() > 14.0 {
            let d = deg.rem_euclid(360);
            let label = match d {
                0 => "N".to_string(),
                90 => "E".to_string(),
                180 => "S".to_string(),
                270 => "W".to_string(),
                _ => format!("{:02}", d / 10),
            };
            stroke_text_at(cv, x, y + 11.0, &label, 5.0, 0.8, col, Align::Centre);
        }
    }
    // The next waypoint's bearing: a mark on the tape (at its edge when off it).
    if let Some((brg, _)) = to_waypoint(u) {
        let off = ((brg.to_degrees() - hdg + 540.0).rem_euclid(360.0) - 180.0) * per_deg;
        let x = c + off.clamp(-half, half);
        cv.triangle(
            [Vec2::new(x, y - 1.0), Vec2::new(x - 4.0, y - 7.0), Vec2::new(x + 4.0, y - 7.0)],
            CYAN,
        );
    }
    cv.fill_rect(c as i32 - 13, y as i32 + 6, 26, 11, BLACK);
    cv.frame(Vec2::new(c - 13.0, y + 6.0), Vec2::new(c + 13.0, y + 17.0), 1.0, col);
    stroke_text_at(cv, c, y + 11.5, &format!("{hdg:03.0}"), 6.0, 0.9, col, Align::Centre);
}

/// The next waypoint at the bottom: its number and kind, bearing and distance.
fn waypoint_line(cv: &mut Canvas, u: &Uav) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let (Some(wp), Some((brg, km))) = (u.target(), to_waypoint(u)) else { return };
    let deg = brg.to_degrees().rem_euclid(360.0);
    let text = format!("WP{} {} {deg:03.0} {km:.1}KM", u.next + 1, wp.kind.name());
    stroke_text_at(cv, w / 2.0, h - 10.0, &text, 6.0, 0.9, CYAN, Align::Centre);
}

/// The HSI at the bottom: the compass rose turning with the heading, the lubber line, the
/// course to the next waypoint and its distance.
fn hsi(cv: &mut Canvas, u: &Uav, col: u8) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let r = 26.0;
    let c = Vec2::new(w / 2.0, h - r - 6.0);
    let (bx0, bx1) = (c.x - 92.0, c.x + 92.0);
    cv.fill_rect(
        bx0 as i32,
        (c.y - r - 4.0) as i32,
        (bx1 - bx0) as i32,
        (2.0 * r + 8.0) as i32,
        BLACK,
    );
    cv.frame(Vec2::new(bx0, c.y - r - 4.0), Vec2::new(bx1, c.y + r + 4.0), 1.0, col);
    let hdg = u.heading as f32;
    // Screen direction of a bearing (heading up).
    let dir = |b: f32| {
        let a = b - hdg;
        Vec2::new(a.sin(), -a.cos())
    };
    cv.ring(c, r, 1.0, col);
    for k in 0..12 {
        let b = (k as f32 * 30.0).to_radians();
        let d = dir(b);
        let len = if k % 3 == 0 { 5.0 } else { 3.0 };
        cv.stroke(c + d * r, c + d * (r - len), 1.0, col);
        if k % 3 == 0 {
            let l = ["N", "E", "S", "W"][k / 3];
            let p = c + d * (r - 10.0);
            stroke_text_at(cv, p.x, p.y, l, 5.0, 0.8, col, Align::Centre);
        }
    }
    // The lubber line, the drone in the middle.
    cv.stroke(c - Vec2::new(0.0, r + 3.0), c - Vec2::new(0.0, r - 4.0), 1.4, YELLOW);
    cv.stroke(c + Vec2::new(-5.0, 0.0), c + Vec2::new(5.0, 0.0), 1.2, YELLOW);
    cv.stroke(c + Vec2::new(0.0, -4.0), c + Vec2::new(0.0, 5.0), 1.2, YELLOW);
    let deg = u.heading.to_degrees().rem_euclid(360.0);
    stroke_text_at(
        cv,
        bx0 + 6.0,
        c.y - r + 4.0,
        &format!("HDG {deg:03.0}"),
        5.0,
        0.8,
        col,
        Align::Left,
    );
    // The course to the next waypoint: an arrow across the rose.
    if let Some(wp) = u.target() {
        let m = wp.metres();
        let (de, dn) = (m[0] - u.pos[0], m[1] - u.pos[1]);
        let brg = de.atan2(dn) as f32;
        let d = dir(brg);
        let (a, b) = (c - d * (r - 6.0), c + d * (r - 6.0));
        cv.stroke(a, b, 1.4, CYAN);
        let side = Vec2::new(-d.y, d.x);
        cv.triangle([b + d * 3.0, b - d * 4.0 + side * 4.0, b - d * 4.0 - side * 4.0], CYAN);
        let km = de.hypot(dn) / 1000.0;
        let deg = brg.to_degrees().rem_euclid(360.0);
        let lines =
            [format!("WP{} {}", u.next + 1, wp.kind.name()), format!("{deg:03.0} {km:.1}KM")];
        for (i, l) in lines.iter().enumerate() {
            stroke_text_at(
                cv,
                bx1 - 6.0,
                c.y - r + 4.0 + i as f32 * 10.0,
                l,
                5.0,
                0.8,
                CYAN,
                Align::Right,
            );
        }
    }
}

/// The camera's projection (`trackcam::WIDE`: f = 256 times the zoom over its width in
/// pixels, square pixels): a direction from the camera to the screen.
fn projection(eo: &ConsoleEo) -> impl Fn(Vec3) -> Option<Vec2> {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let f = 256.0 * crate::mfd::cam_magnification(eo.zoom) * w / crate::trackcam::WIDE.0;
    let dir = eo.dir;
    let up = if dir.y.abs() > 0.99 { Vec3::NEG_Z } else { Vec3::Y };
    let right = dir.cross(up).normalize();
    let up = right.cross(dir);
    move |v: Vec3| {
        let z = v.dot(dir);
        (z > 0.0).then(|| Vec2::new(w / 2.0 + v.dot(right) / z * f, h / 2.0 - v.dot(up) / z * f))
    }
}

/// Corner brackets round a point.
fn bracket(cv: &mut Canvas, p: Vec2, s: f32, col: u8) {
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            let c = p + Vec2::new(sx * s, sy * s);
            cv.stroke(c, c - Vec2::new(sx * 4.0, 0.0), 1.2, col);
            cv.stroke(c, c - Vec2::new(0.0, sy * 4.0), 1.2, col);
        }
    }
}

/// The forward FLIR's overlay: the horizon (banked, pitched) with its pitch steps and the
/// drone's symbol on its nose line, the heading.
fn horizon(cv: &mut Canvas, u: &Uav, eo: &ConsoleEo, col: u8) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let o = Vec2::new(w / 2.0, h / 2.0);
    let f = 256.0 * crate::mfd::cam_magnification(eo.zoom) * w / crate::trackcam::WIDE.0;
    // The camera looks NOSE_DOWN under the nose: level flight puts the horizon above centre.
    let cam_pitch = (u.pitch - NOSE_DOWN) as f32;
    let horizon = o + Vec2::new(0.0, cam_pitch.tan() * f);
    let bank = u.bank as f32;
    let along = Vec2::new(bank.cos(), -bank.sin());
    cv.stroke(horizon - along * 90.0, horizon - along * 20.0, 1.2, col);
    cv.stroke(horizon + along * 20.0, horizon + along * 90.0, 1.2, col);
    // Pitch steps of 10 degrees.
    for deg in [-20.0f32, -10.0, 10.0, 20.0] {
        let p = horizon + Vec2::new(-along.y, along.x) * (-(deg.to_radians().tan()) * f);
        if (p - o).length() < h / 2.0 - 30.0 {
            cv.dashed(p - along * 24.0, p + along * 24.0, 3.0, 3.0, 0.8, col);
            stroke_text_at(
                cv,
                p.x + along.x * 30.0,
                p.y,
                &format!("{deg:.0}"),
                5.0,
                0.8,
                col,
                Align::Left,
            );
        }
    }
    // The drone, fixed at the camera's centre line through its nose.
    let nose = o - Vec2::new(0.0, (NOSE_DOWN as f32).tan() * f);
    cv.stroke(nose - Vec2::new(16.0, 0.0), nose - Vec2::new(5.0, 0.0), 1.4, YELLOW);
    cv.stroke(nose + Vec2::new(5.0, 0.0), nose + Vec2::new(16.0, 0.0), 1.4, YELLOW);
    cv.ring(nose, 4.0, 1.2, YELLOW);
}
