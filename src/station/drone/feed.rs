//! The selected drone's camera: the console's camera (`ConsoleEo`, rendered by `trackcam`)
//! rides on the drone. As a TGP it looks at a ground point: slaved, at the next waypoint (or
//! ahead and down when that is far), tracking, at the point the stick put it on. Brackets mark
//! the enemy air defences the drone sees (a surveillance drone all within its sensor's reach,
//! an attack drone those within 2 km). Flown by hand it looks ahead, under a flight display.
//! An interceptor only looks ahead: diamonds mark the aircraft in its view within `AIR_MARKS` m
//! (their range and how far above or below; the friendly side's blue, as it would lock on to
//! them too), a circle round the flight path marker within which fire locks on, a box the one
//! it locked on to.
//! The view ahead is fixed to the airframe: it pitches and rolls with the drone. Without a
//! drone selected the camera rests.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;
use stealth_sim::aaa::M_PER_MAP;
use stealth_sim::uav::{Uav, UavKind, UavMode};
use stealth_sim::{Sim, oflags};

use super::super::eo::ConsoleEo;
use super::super::scope::{BLACK, BLUE, CYAN, GREEN, GREY, ORANGE, RED, WHITE, YELLOW};
use super::{DroneUi, center_uavs, selected};
use crate::canvas::{Canvas, CanvasTexture, TRANSPARENT};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, FEED_CANVAS};
use crate::combat::{Combat, unit_position};
use crate::font::{Align, stroke_text_at, stroke_width};
use crate::station::Manning;
use crate::world::wu_to_bevy;

/// A slaved camera looks at the next waypoint when it is this close (m), else ahead and down
/// this far for each metre of height (at least `AHEAD_MIN` m).
const SLAVE_REACH: f64 = 12_000.0;
const AHEAD: f64 = 3.0;
const AHEAD_MIN: f64 = 2500.0;
/// The camera under the drone (m).
const BELOW: f64 = 0.4;
/// An interceptor's view marks the enemy aircraft this close (m), an X-12's this many times
/// as far as it locks on.
const AIR_MARKS: f64 = 4000.0;
const X12_AIR_MARKS: f64 = 2.0;

/// The drone's position as drawn (Bevy world).
fn drone_eye(combat: &Combat, u: &Uav) -> Option<DVec3> {
    let i = u.flight as usize;
    let f = combat.sim.flights().get(i)?;
    let (e, n, z) = unit_position(combat.prev_flights.get(i), f, combat.alpha);
    Some(wu_to_bevy(e, n, z) - DVec3::Y * BELOW)
}

/// Drone `u` as drawn: its attitude carried on smoothly between the simulation's frames (from
/// its flight unit's last frame to this one), as its position is.
fn drawn(combat: &Combat, u: &Uav) -> Uav {
    let i = u.flight as usize;
    let (Some(p), Some(c)) = (combat.prev_flights.get(i), combat.sim.flights().get(i)) else {
        return u.clone();
    };
    let t = combat.alpha as f64;
    let r = std::f64::consts::TAU / 65536.0;
    // From last frame's angle by the (shortest) way to this one's.
    let lerp = |a: i32, b: i32| {
        let d = (b.wrapping_sub(a) as u16 as i16) as f64;
        (a as i16) as f64 * r + d * t * r
    };
    let mut d = u.clone();
    d.heading = lerp(p.heading, c.heading);
    d.pitch = lerp(p.pitch, c.pitch);
    d.bank = lerp(p.bank, c.bank);
    d
}

/// The view ahead (Bevy world): the camera's direction, along the nose, and its up, both turned
/// with the drone's heading, pitch and bank.
fn forward_view(u: &Uav) -> (Vec3, Vec3) {
    let (sh, ch, sp, cp) = (u.heading.sin(), u.heading.cos(), u.pitch.sin(), u.pitch.cos());
    let (sb, cb) = (u.bank.sin(), u.bank.cos());
    // East, north, up.
    let nose = DVec3::new(sh * cp, ch * cp, sp);
    let right0 = DVec3::new(ch, -sh, 0.0);
    let up0 = DVec3::new(-sh * sp, -ch * sp, cp);
    let up = up0 * cb + right0 * sb;
    let bevy = |v: DVec3| Vec3::new(v.x as f32, v.z as f32, -v.y as f32);
    (bevy(nose), bevy(up))
}

/// A map point on the ground (Bevy world; the middle of its map unit, where the units stand,
/// on the rolling hills).
fn ground(sim: &Sim, p: DVec2) -> DVec3 {
    crate::sim_render::on_ground(
        &sim.hills(),
        crate::sim_render::map_point(p.x + 0.5, p.y + 0.5, 0.0),
    )
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
    let pose = u.as_ref().and_then(|u| drone_eye(&combat, u));
    let (eye, dir, up, zoom, target, idle) = match (u, pose) {
        (Some(u), Some(eye)) => {
            let cam = ui.cam(u.id);
            if super::forward(&ui, &u) {
                let (dir, up) = forward_view(&drawn(&combat, &u));
                (eye, dir, Some(up), 0, None, false)
            } else {
                let look = cam.track.unwrap_or_else(|| slaved(&u));
                ui.cam_mut(u.id).look = Some(look);
                let at = ground(&combat.sim, look);
                let dir = (at - eye).as_vec3().normalize_or(Vec3::NEG_Y);
                (eye, dir, None, cam.zoom, Some(at), false)
            }
        }
        _ => {
            let at =
                combat.sim.units().get(m.unit).map_or(DVec3::ZERO, |u| {
                    ground(&combat.sim, DVec2::new(u.x as f64, u.y as f64))
                });
            (at + DVec3::Y * 5.0, Vec3::NEG_Z, None, crate::mfd::CAM_ZOOM_DEFAULT, None, true)
        }
    };
    // The camera's picture is shown in the corner of the map or on the main screen.
    let idle = idle || ui.selected.is_none();
    let slew = (controls.roll.abs().max(controls.pitch.abs())).min(1.0);
    match eo {
        Some(mut eo) => {
            eo.eye = eye;
            eo.dir = dir;
            eo.up = up;
            eo.zoom = zoom;
            eo.target = target;
            eo.slew = if idle { 0.0 } else { slew };
            eo.idle = idle;
        }
        None => commands.insert_resource(ConsoleEo {
            eye,
            dir,
            up,
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
    for (k, a) in sim.units().iter().enumerate() {
        let m = a.metres();
        if !a.destroyed && !sim.aaa_blue(k) && near(m[0], m[1]) {
            let at = ground(sim, DVec2::new(a.x as f64, a.y as f64));
            out.push((at, a.kind.name().to_string()));
        }
    }
    for o in sim.objects().iter() {
        let hostile = o.flags & (oflags::DESTROYED | oflags::FRIENDLY | oflags::NEUTRAL) == 0;
        let (e, n) = (o.x as f64 * M_PER_MAP, (0x8000 - o.y) as f64 * M_PER_MAP);
        if o.unit_type != 0 && hostile && near(e, n) {
            let at = ground(sim, DVec2::new(o.x as f64, o.y as f64));
            out.push((at, sim.threat_name(o.unit_type).to_string()));
        }
    }
    out
}

/// What the overlay shows besides the drone: how long it has flown (s), its command center
/// (m east, north), the wind (the bearing it blows to (rad), m/s).
pub struct Osd {
    pub flown: f64,
    pub home: Option<DVec2>,
    pub wind: Option<(f32, f32)>,
}

/// The main screen: the camera's picture under an FPV-style overlay (the TGP's crosshair or the
/// forward camera's horizon; the speed and height tapes either side, the compass on top, the
/// flight's figures in the corners) while a drone is selected; without its picture here, NO
/// FEED; the selected drone gone (it hit, or was shot down), NO SIGNAL.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    ui: Res<DroneUi>,
    eo: Option<Res<ConsoleEo>>,
    mfd: Res<crate::mfd::MfdState>,
    weather: Option<Res<crate::weather::WeatherState>>,
    time: Res<Time>,
    mut launched: Local<Vec<(u16, f64)>>,
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
    // When each drone in the air was first seen (its flight time).
    let now = time.elapsed_secs_f64();
    launched.retain(|(id, _)| uavs.iter().any(|u| u.id == *id));
    for u in &uavs {
        if !launched.iter().any(|(id, _)| *id == u.id) {
            launched.push((u.id, now));
        }
    }
    let smooth = selected(&ui, &uavs).map(|u| drawn(&combat, u));
    let Some(u) = smooth.as_ref() else {
        cv.clear(BLACK);
        if ui.lost {
            stroke_text_at(
                cv,
                w / 2.0,
                h / 2.0 - 8.0,
                "NO SIGNAL",
                12.0,
                1.4,
                WHITE,
                Align::Centre,
            );
        } else {
            stroke_text_at(cv, w / 2.0, h / 2.0 - 8.0, "NO FEED", 9.0, 1.2, GREEN, Align::Centre);
        }
        let hint = "SELECT A DRONE ON THE MAP";
        stroke_text_at(cv, w / 2.0, h / 2.0 + 12.0, hint, 6.0, 0.9, GREY, Align::Centre);
        return;
    };
    let cam = ui.cam(u.id);
    let forward = super::forward(&ui, u);
    let col = GREEN;
    let picture = eo.is_some();
    if picture {
        cv.clear(TRANSPARENT);
    } else {
        cv.clear(BLACK);
        stroke_text_at(cv, w / 2.0, h / 2.0 - 10.0, "NO FEED", 9.0, 1.2, GREEN, Align::Centre);
    }
    // The top line: the drone and the camera, the mode.
    let sensor = if mfd.flir { "FLIR" } else { "TV" };
    let left = format!("{} {} {sensor}", u.label(), if forward { "FWD" } else { "TGP" });
    stroke_text_at(cv, 52.0, 9.0, &left, 6.0, 0.9, col, Align::Left);
    let mode = match (u.mode, forward) {
        (UavMode::Manual, _) => "MANUAL".to_string(),
        (m, true) => m.name().to_string(),
        (m, false) => {
            format!("{} {}", if cam.track.is_some() { "TRACK" } else { "SLAVE" }, m.name())
        }
    };
    let mcol = if u.mode == UavMode::Manual { YELLOW } else { col };
    stroke_text_at(cv, w - 52.0, 9.0, &mode, 6.0, 0.9, mcol, Align::Right);
    if u.mode == UavMode::Dive {
        stroke_text_at(cv, w / 2.0, 72.0, "TERMINAL DIVE", 7.0, 1.0, RED, Align::Centre);
    } else if u.stalled() {
        stroke_text_at(cv, w / 2.0, 72.0, "STALL", 7.0, 1.0, RED, Align::Centre);
    }
    if let Some(j) = u.lock {
        let name = combat.sim.flights().get(j as usize).map(|f| combat.sim.aircraft_name(f.spec));
        let text = format!("LOCK {}", name.unwrap_or_default().to_uppercase());
        stroke_text_at(cv, w / 2.0, 84.0, &text, 7.0, 1.0, RED, Align::Centre);
        if u.mode == UavMode::Manual {
            stroke_text_at(cv, w / 2.0, 96.0, "THROTTLE UP", 5.0, 0.8, RED, Align::Centre);
        }
    } else if u.kind.hunter() && u.mode != UavMode::Launch {
        stroke_text_at(cv, w / 2.0, h - 26.0, "FIRE TO LOCK", 5.0, 0.8, GREY, Align::Centre);
    }
    if let Some(eo) = eo.filter(|_| picture) {
        if forward {
            horizon(cv, u, &eo, col);
            if !u.kind.tgp() {
                air(cv, &combat, u, &eo);
            }
        } else {
            tgp(cv, &combat.sim, u, &eo, col);
        }
    }
    let osd = Osd {
        flown: launched.iter().find(|(id, _)| *id == u.id).map_or(0.0, |(_, t)| now - t),
        home: combat.sim.units().get(m.unit).map(|a| {
            let p = a.metres();
            DVec2::new(p[0], p[1])
        }),
        wind: weather.map(|s| ((s.wind.from_deg + 180.0).to_radians(), s.wind.kt * 0.514)),
    };
    // Flown by hand, the lever as the console holds it (the drone follows a moment later).
    let thr = if u.mode == UavMode::Manual { ui.throttle } else { u.throttle() as f32 };
    overlay(cv, u, &osd, thr, col);
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
        w - 52.0,
        84.0,
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
    gimbal_azimuth(cv, u, eo, col);
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

/// How far the TGP looks left or right of the nose: a scale at the bottom (from 180
/// left to 180 right, the nose in the middle), a caret at the camera's azimuth and its
/// degrees beside the scale.
fn gimbal_azimuth(cv: &mut Canvas, u: &Uav, eo: &ConsoleEo, col: u8) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let (cx, y, half) = (w / 2.0, h - 12.0, 72.0);
    // The camera's bearing (Bevy world: x east, -z north) less the drone's heading.
    let bearing = eo.dir.x.atan2(-eo.dir.z) as f64;
    let rel = (bearing - u.heading + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;
    let deg = rel.to_degrees() as f32;
    cv.stroke(Vec2::new(cx - half, y), Vec2::new(cx + half, y), 1.0, col);
    for k in -4..=4 {
        let x = cx + half * k as f32 / 4.0;
        let len = match k {
            0 => 6.0,
            _ if k % 2 == 0 => 4.0,
            _ => 2.5,
        };
        cv.stroke(Vec2::new(x, y), Vec2::new(x, y - len), 1.0, col);
    }
    let x = cx + half * (deg / 180.0).clamp(-1.0, 1.0);
    let tip = Vec2::new(x, y + 1.5);
    cv.triangle([tip, tip + Vec2::new(-3.5, 5.5), tip + Vec2::new(3.5, 5.5)], col);
    let side = if deg.round() == 0.0 {
        String::from("000")
    } else {
        format!("{}{:03.0}", if deg < 0.0 { "L" } else { "R" }, deg.abs())
    };
    stroke_text_at(cv, cx + half + 6.0, y - 1.0, &format!("AZ {side}"), 5.0, 0.8, col, Align::Left);
}

/// The interceptor's view: a diamond on each aircraft ahead within `AIR_MARKS` (its range in
/// km, how many metres above or below; yellow the enemy's, blue the friendly side's), yellow
/// brackets round those fire would lock on to, a box on the one it is locked on to, and the circle round the flight path marker within which fire
/// locks on.
fn air(cv: &mut Canvas, combat: &Combat, u: &Uav, eo: &ConsoleEo) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let project = projection(eo);
    let sim = &combat.sim;
    // The lock cone round the way it goes.
    let o = path_marker(u, eo);
    let f = 256.0 * crate::mfd::cam_magnification(eo.zoom) * w / crate::trackcam::WIDE.0;
    let r = (stealth_sim::uav::LOCK_CONE as f32).tan() * f;
    if u.lock.is_none() {
        cv.ring(o, r, 0.8, GREY);
    }
    for j in sim.interceptor_targets() {
        let t = sim.flights()[j];
        let (e, n, z) = unit_position(combat.prev_flights.get(j), &t, combat.alpha);
        let at = wu_to_bevy(e, n, z);
        let d = at - eo.eye;
        let range = d.length();
        let marks = match u.kind {
            stealth_sim::uav::UavKind::X12 => u.kind.lock_range().0 * X12_AIR_MARKS,
            _ => AIR_MARKS,
        };
        if range > marks {
            continue;
        }
        let Some(p) = project(d.as_vec3()) else { continue };
        if p.x < 46.0 || p.y < 4.0 || p.x > w - 46.0 || p.y > h - 4.0 {
            continue;
        }
        let locked = u.lock == Some(j as u16);
        // Fire would lock on to it: within reach, inside the circle round the marker.
        let lockable = u.lock.is_none()
            && range < u.kind.lock_range().0
            && d.as_vec3().normalize_or_zero().dot(way(u).normalize_or_zero())
                > (stealth_sim::uav::LOCK_CONE as f32).cos();
        if lockable {
            bracket(cv, p, 9.0, YELLOW);
        }
        let col = if locked {
            RED
        } else if sim.friendly_unit(j, t.spec) {
            CYAN
        } else {
            YELLOW
        };
        if locked {
            cv.frame(p - Vec2::splat(9.0), p + Vec2::splat(9.0), 1.4, RED);
        }
        let s = 5.0;
        let pts = [
            p + Vec2::new(0.0, -s),
            p + Vec2::new(s, 0.0),
            p + Vec2::new(0.0, s),
            p + Vec2::new(-s, 0.0),
            p + Vec2::new(0.0, -s),
        ];
        cv.polyline(&pts, 1.0, col);
        let above = (t.alt.max(0) as f64 * crate::world::WU - u.pos[2]).round();
        let text = format!("{:.1} {above:+.0}M", range / 1000.0);
        stroke_text_at(cv, p.x, p.y + 13.0, &text, 5.0, 0.8, col, Align::Centre);
    }
}

/// Height of the tapes' scale: pixels a unit (m/s, m), units between the numbers.
const TAPE_PX: f32 = 8.0;
const TAPE_STEP: i32 = 5;

/// A battery's cells and capacity (mAh); none for a jet's fuel.
fn battery_pack(kind: UavKind) -> Option<(f32, f32)> {
    match kind {
        _ if kind.jet() => None,
        UavKind::Stork => Some((12.0, 30_000.0)),
        UavKind::Interceptor => Some((6.0, 6_000.0)),
        _ => Some((8.0, 22_000.0)),
    }
}

/// The FPV overlay: the airspeed tape on the left and the height's on the right, the compass on
/// top (the home and the next waypoint marked, the wind under the heading); the time flown,
/// ground speed and throttle top left, the distance home and the link top right; the battery
/// (volts, mAh used), the next waypoint and the metres a mAh bottom left; the battery's (or
/// tank's) charge, pitch and roll bottom right.
pub(super) fn overlay(cv: &mut Canvas, u: &Uav, osd: &Osd, thr: f32, col: u8) {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    tape(cv, 44.0, false, u.speed as f32, Some(u.kind.perf().max as f32), ["AIRSPD", "(M/S)"], col);
    tape(cv, w - 44.0, true, u.pos[2] as f32, None, ["ALT", "(M)"], col);
    compass(cv, u, osd, col);
    // Top left.
    let x = 52.0;
    let t = osd.flown.max(0.0) as i64;
    let time = format!("T : {:02}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60);
    stroke_text_at(cv, x, 31.0, &time, 6.0, 0.9, col, Align::Left);
    let gspd = format!("GSPD : {:.0} M/S", u.speed);
    stroke_text_at(cv, x, 42.0, &gspd, 6.0, 0.9, col, Align::Left);
    let tcol = if u.mode == UavMode::Manual { YELLOW } else { col };
    let throttle = format!("THR : {:.0}%", thr.clamp(0.0, 1.0) * 100.0);
    stroke_text_at(cv, x, 53.0, &throttle, 6.0, 0.9, tcol, Align::Left);
    // Top right: home and the link (weaker the further it is).
    let rx = w - 52.0;
    if let Some(home) = osd.home {
        let d = (DVec2::new(u.pos[0], u.pos[1]) - home).length();
        stroke_text_at(cv, rx, 31.0, &format!("DST : {d:.0} M"), 6.0, 0.9, col, Align::Right);
        let dbm = (-40.0 - 20.0 * (d.max(100.0) / 100.0).log10()) as f32;
        link(cv, rx, 36.0, false, dbm);
        link(cv, rx, 56.0, true, dbm - 3.0);
    }
    // Bottom left.
    if let Some((cells, cap)) = battery_pack(u.kind) {
        let b = u.battery.clamp(0.0, 1.0) as f32;
        let volts = cells * (3.3 + 0.9 * b);
        stroke_text_at(cv, x, h - 58.0, &format!("V : {volts:.1}"), 6.0, 0.9, RED, Align::Left);
        let used = format!("MAH : {:.0}", cap * (1.0 - b));
        stroke_text_at(cv, x, h - 47.0, &used, 6.0, 0.9, ORANGE, Align::Left);
        // Metres flown a mAh, on a scale to 10.
        let rate = cap * u.drain() as f32;
        let per = if rate > 0.0 { u.speed as f32 / rate } else { 0.0 };
        let (x0, x1, y) = (x, x + 176.0, h - 20.0);
        cv.stroke(Vec2::new(x0, y), Vec2::new(x1, y), 1.0, col);
        stroke_text_at(cv, x0, h - 9.0, "M/MAH", 5.0, 0.8, col, Align::Left);
        for (k, l) in ["2.5", "5", "7.5", "10"].iter().enumerate() {
            let tx = x0 + (x1 - x0) * (k + 1) as f32 / 4.0;
            cv.stroke(Vec2::new(tx, y - 2.0), Vec2::new(tx, y + 2.0), 1.0, col);
            stroke_text_at(cv, tx, h - 9.0, l, 5.0, 0.8, col, Align::Centre);
        }
        let dot = x0 + (x1 - x0) * (per / 10.0).clamp(0.0, 1.0);
        cv.ring(Vec2::new(dot, y), 1.75, 3.5, BLUE);
    }
    if let (Some(wp), Some((_, km))) = (u.target(), to_waypoint(u)) {
        let y = h - 36.0;
        arrow(cv, Vec2::new(x, y), Vec2::new(x + 9.0, y), col);
        let text = format!("WP{} {} {km:.1} KM", u.next + 1, wp.kind.name());
        stroke_text_at(cv, x + 13.0, y, &text, 6.0, 0.9, col, Align::Left);
    }
    // Bottom right.
    let bat = (u.battery * 100.0).round() as i32;
    let tank = if u.kind.jet() { "FUEL" } else { "BAT" };
    let charge = format!("{tank} : {bat}% {}", super::map::duration(u.endurance() as i32));
    let bcol = if bat < 20 { RED } else { col };
    stroke_text_at(cv, rx, h - 35.0, &charge, 6.0, 0.9, bcol, Align::Right);
    let pitch = format!("P : {:.1}°", u.pitch.to_degrees());
    stroke_text_at(cv, rx, h - 24.0, &pitch, 6.0, 0.9, col, Align::Right);
    let roll = format!("R : {:.1}°", u.bank.to_degrees());
    stroke_text_at(cv, rx, h - 13.0, &roll, 6.0, 0.9, col, Align::Right);
}

/// A tape down the side at `x`: its title on top, a number every `TAPE_STEP` outside the line
/// (orange past `warn`), the value inside it at the middle behind a pointer, a line on from it
/// towards the middle of the picture.
fn tape(
    cv: &mut Canvas,
    x: f32,
    right: bool,
    value: f32,
    warn: Option<f32>,
    title: [&str; 2],
    col: u8,
) {
    let h = FEED_CANVAS.1 as f32;
    let (top, bottom) = (26.0, h - 6.0);
    let mid = (top + bottom) / 2.0;
    // Outwards and inwards.
    let (out, inw) = if right { (1.0, -1.0) } else { (-1.0, 1.0) };
    let (al_out, al_in) =
        if right { (Align::Left, Align::Right) } else { (Align::Right, Align::Left) };
    let (tx, tal) = if right { (x + 40.0, Align::Right) } else { (x - 40.0, Align::Left) };
    stroke_text_at(cv, tx, 8.0, title[0], 6.0, 0.9, col, tal);
    stroke_text_at(cv, tx, 17.0, title[1], 6.0, 0.9, col, tal);
    cv.stroke(Vec2::new(x, top), Vec2::new(x, bottom), 1.0, col);
    let span = (mid - top) / TAPE_PX;
    let half_step = TAPE_STEP as f32 / 2.0;
    let lo = ((value - span) / half_step).ceil() as i32;
    let hi = ((value + span) / half_step).floor() as i32;
    for k in lo.max(0)..=hi {
        let v = k as f32 * half_step;
        let y = mid - (v - value) * TAPE_PX;
        if k % 2 == 1 {
            cv.stroke(Vec2::new(x, y), Vec2::new(x + out * 3.0, y), 1.0, col);
            continue;
        }
        let c = if warn.is_some_and(|m| v > m) { ORANGE } else { col };
        cv.stroke(Vec2::new(x, y), Vec2::new(x + out * 6.0, y), 1.0, c);
        stroke_text_at(cv, x + out * 9.0, y, &format!("{v:.0}"), 7.0, 1.0, c, al_out);
    }
    let tip = Vec2::new(x + inw * 2.0, mid);
    cv.polyline(
        &[tip, tip + Vec2::new(inw * 6.0, -4.0), tip + Vec2::new(inw * 6.0, 4.0), tip],
        1.0,
        col,
    );
    let text = format!("{value:.0}");
    let tx = x + inw * 11.0;
    stroke_text_at(cv, tx, mid, &text, 9.0, 1.2, col, al_in);
    let a = tx + inw * (stroke_width(&text, 9.0) + 5.0);
    cv.stroke(Vec2::new(a, mid), Vec2::new(x + inw * 84.0, mid), 1.0, col);
}

/// The compass tape across the top: a tick every 15 degrees, the points of the compass by
/// letter, the next waypoint (cyan) and home (a yellow H) at their bearings (at the tape's ends
/// when off it); under it the heading and the wind (blue, an arrow the way it blows).
pub(super) fn compass(cv: &mut Canvas, u: &Uav, osd: &Osd, col: u8) {
    let w = FEED_CANVAS.0 as f32;
    let (c, half, per_deg, y) = (w / 2.0, 96.0, 96.0 / 70.0, 18.0);
    let hdg = u.heading.to_degrees().rem_euclid(360.0) as f32;
    let off = |brg: f32| ((brg - hdg + 540.0).rem_euclid(360.0) - 180.0) * per_deg;
    let lo = ((hdg - half / per_deg) / 15.0).ceil() as i32;
    let hi = ((hdg + half / per_deg) / 15.0).floor() as i32;
    for k in lo..=hi {
        let deg = k * 15;
        let x = c + (deg as f32 - hdg) * per_deg;
        let d = deg.rem_euclid(360);
        if d % 45 == 0 {
            cv.stroke(Vec2::new(x, y - 5.0), Vec2::new(x, y), 1.0, col);
            let l = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"][d as usize / 45];
            stroke_text_at(cv, x, 7.0, l, 6.0, 0.9, col, Align::Centre);
        } else {
            cv.stroke(Vec2::new(x, y - 2.5), Vec2::new(x, y), 1.0, col);
        }
    }
    if let Some((brg, _)) = to_waypoint(u) {
        let x = c + off(brg.to_degrees()).clamp(-half, half);
        cv.triangle(
            [Vec2::new(x, y + 1.0), Vec2::new(x - 3.5, y - 5.0), Vec2::new(x + 3.5, y - 5.0)],
            CYAN,
        );
    }
    if let Some(home) = osd.home {
        let (de, dn) = (home.x - u.pos[0], home.y - u.pos[1]);
        let x = c + off(de.atan2(dn).to_degrees() as f32).clamp(-half, half);
        cv.ring(Vec2::new(x, y - 3.0), 2.75, 5.5, YELLOW);
        stroke_text_at(cv, x, y - 3.0, "H", 4.5, 0.9, BLACK, Align::Centre);
    }
    let tip = Vec2::new(c, y + 3.0);
    cv.polyline(&[tip, tip + Vec2::new(-4.0, 6.0), tip + Vec2::new(4.0, 6.0), tip], 1.0, col);
    stroke_text_at(cv, c, y + 16.0, &format!("{hdg:03.0}°"), 8.0, 1.1, col, Align::Centre);
    if let Some((to, ms)) = osd.wind {
        stroke_text_at(cv, c, y + 28.0, &format!("{ms:.1} M/S"), 5.0, 0.8, BLUE, Align::Centre);
        if ms > 0.05 {
            let a = to - u.heading as f32;
            let d = Vec2::new(a.sin(), -a.cos());
            let p = Vec2::new(c, y + 39.0);
            arrow(cv, p - d * 6.0, p + d * 6.0, BLUE);
        }
    }
}

/// An arrow from `a` to `b`.
fn arrow(cv: &mut Canvas, a: Vec2, b: Vec2, col: u8) {
    let d = (b - a).normalize_or(Vec2::X);
    let side = Vec2::new(-d.y, d.x);
    cv.stroke(a, b, 1.2, col);
    cv.stroke(b, b - d * 4.0 + side * 3.0, 1.2, col);
    cv.stroke(b, b - d * 4.0 - side * 3.0, 1.2, col);
}

/// A link's strength ending at `right`: L, an arrow (up the uplink, down the downlink), five
/// bars and the dBm under them; orange fair, red weak.
fn link(cv: &mut Canvas, right: f32, y: f32, up: bool, dbm: f32) {
    let c = if dbm < -85.0 {
        RED
    } else if dbm < -72.0 {
        ORANGE
    } else {
        GREEN
    };
    let lit = ((dbm + 100.0) / 10.0).ceil().clamp(0.0, 5.0) as i32;
    for i in 0..5 {
        let x = right - 29.0 + i as f32 * 6.0;
        let bh = 2.0 + i as f32 * 2.0;
        let (a, b) = (Vec2::new(x, y + 9.0 - bh), Vec2::new(x + 4.0, y + 9.0));
        if i < lit {
            cv.fill_rect(a.x as i32, a.y as i32, 5, bh as i32 + 1, c);
        } else {
            cv.frame(a, b, 0.8, GREY);
        }
    }
    let ax = right - 36.0;
    let (a, b) = if up { (y + 9.0, y + 1.0) } else { (y + 1.0, y + 9.0) };
    arrow(cv, Vec2::new(ax, a), Vec2::new(ax, b), c);
    stroke_text_at(cv, ax - 5.0, y + 5.0, "L", 7.0, 1.0, c, Align::Right);
    stroke_text_at(
        cv,
        right - 13.0,
        y + 15.5,
        &format!("{dbm:.0} DBM"),
        4.0,
        0.7,
        c,
        Align::Centre,
    );
}

/// The bearing (rad) and distance (km) to the next waypoint.
fn to_waypoint(u: &Uav) -> Option<(f32, f64)> {
    let m = u.target()?.metres();
    let (de, dn) = (m[0] - u.pos[0], m[1] - u.pos[1]);
    Some((de.atan2(dn) as f32, de.hypot(dn) / 1000.0))
}

/// The camera's projection (`trackcam::WIDE`: f = 256 times the zoom over its width in
/// pixels, square pixels): a direction from the camera to the screen.
fn projection(eo: &ConsoleEo) -> impl Fn(Vec3) -> Option<Vec2> {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    let f = 256.0 * crate::mfd::cam_magnification(eo.zoom) * w / crate::trackcam::WIDE.0;
    let dir = eo.dir;
    let up = eo.up.unwrap_or(if dir.y.abs() > 0.99 { Vec3::NEG_Z } else { Vec3::Y });
    let right = dir.cross(up).normalize();
    let up = right.cross(dir);
    move |v: Vec3| {
        let z = v.dot(dir);
        (z > 0.0).then(|| Vec2::new(w / 2.0 + v.dot(right) / z * f, h / 2.0 - v.dot(up) / z * f))
    }
}

/// Where the flight path marker is in the picture (held inside it): along the drone's way (a
/// winged drone's along its nose).
fn path_marker(u: &Uav, eo: &ConsoleEo) -> Vec2 {
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    projection(eo)(way(u)).map_or(Vec2::new(w / 2.0, h / 2.0), |p| {
        p.clamp(Vec2::new(30.0, 10.0), Vec2::new(w - 30.0, h - 10.0))
    })
}

/// The way the drone goes (Bevy world; at rest, along its nose), as the simulation's lock
/// takes it (`Sim::uav_lock_target`).
fn way(u: &Uav) -> Vec3 {
    let w = if u.vel.iter().any(|&v| v != 0.0) {
        u.vel
    } else {
        [u.heading.sin() * u.pitch.cos(), u.heading.cos() * u.pitch.cos(), u.pitch.sin()]
    };
    Vec3::new(w[0] as f32, w[2] as f32, -w[1] as f32)
}

/// Corner brackets round a point.
pub(super) fn bracket(cv: &mut Canvas, p: Vec2, s: f32, col: u8) {
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            let c = p + Vec2::new(sx * s, sy * s);
            cv.stroke(c, c - Vec2::new(sx * 4.0, 0.0), 1.2, col);
            cv.stroke(c, c - Vec2::new(0.0, sy * 4.0), 1.2, col);
        }
    }
}

/// The forward camera's overlay: the horizon as a long thin line where it is in the picture (the
/// camera rolls and pitches with the drone), a small cross where it is going (on the nose, the
/// middle, but for an interceptor sinking or sliding).
fn horizon(cv: &mut Canvas, u: &Uav, eo: &ConsoleEo, col: u8) {
    let project = projection(eo);
    // A direction (Bevy world) at a heading on the horizon.
    let at = |hdg: f64| Vec3::new(hdg.sin() as f32, 0.0, -hdg.cos() as f32);
    let (hdg, span) = (u.heading, 0.3);
    if let (Some(a), Some(c), Some(b)) =
        (project(at(hdg - span)), project(at(hdg)), project(at(hdg + span)))
    {
        let along = (b - a).normalize_or(Vec2::X);
        cv.stroke(c - along * 120.0, c + along * 120.0, 1.0, col);
    }
    let p = path_marker(u, eo);
    cv.stroke(p - Vec2::splat(3.0), p + Vec2::splat(3.0), 1.0, col);
    cv.stroke(p + Vec2::new(-3.0, 3.0), p + Vec2::new(3.0, -3.0), 1.0, col);
}
