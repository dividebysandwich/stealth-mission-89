//! The F-14's HUD and VDI symbology (after the VDIG-R's): one set of symbols for both, on the
//! modern HUD's canvas (`modern`: angular pixels round the boresight) and its field.
//!
//! - Along the top the heading scale (a tick every 5 degrees, the tens under every other one)
//!   under its caret, the bearing to steer an open triangle under the scale.
//! - The horizon across the field, the pitch ladder every 5 degrees (solid over the horizon,
//!   dashed under it, each bar's inner end turned towards the horizon, the degrees at the left
//!   bar's outer end), the flight path marker (an arc on its wings with the caret beside it).
//! - Left the airspeed in a ring of dots (its needle goes round once in 100 knots) under the
//!   ground speed (G), right the altitude in its ring (once round in 1,000 feet) under the
//!   radar altitude (R); with the HUD decluttered both are boxed numbers.
//! - Lower left the angle of attack, Mach number and load factor, under them the radar's mode
//!   and the bearing and range (nmi) to the steering point (BA); in air-to-ground the weapon
//!   (CP: the computer pilot release).
//! - Lower right the target's bearing, range (nmi), altitude (thousands of feet) and closing
//!   speed, the steering point's number and range, the time (Z), the time to go to it (TTG) and
//!   when it is reached (ETA); our radar missiles in flight with the soonest's time to impact.
//! - Air-to-air: the steering circle (dashed; its solid arc from the top is the target's
//!   range of the scale, the triangles on it the longest and shortest launch ranges), the
//!   steering symbol in it towards the target, the target's box (corner dots) with SHOOT over
//!   it in range, the next track-while-scan target a dotted box, the weapon and its rounds
//!   under the circle.
//! - Air-to-ground, the gun and rockets: `modern`'s cues.

use bevy::prelude::*;

use super::modern::{
    CX, CY, INK, PEN, TEXT, THIN, acm_scan, air_to_ground, bomb_readouts, boxed, cross_out, euler,
    field, gun_sight, horizon_axes, ils_bars, inside, min_sec, pod_cue, project, rocket_sight,
    small, steerpoint_cue, text,
};
use super::{HudFrame, HudMode};
use crate::canvas::Canvas;
use crate::font::Align;

/// Kilometres a nautical mile.
const KM_NM: f32 = 1.852;
/// The heading scale: its row, pixels a degree and how far either side it shows (degrees).
const TAPE_Y: f32 = 100.0;
const TAPE_PX: f32 = 8.0;
const TAPE_HALF: f32 = 21.0;
/// The speed's and the altitude's rings: their centres and radius.
const SPEED_AT: Vec2 = Vec2::new(392.0, 262.0);
const ALT_AT: Vec2 = Vec2::new(888.0, 262.0);
const RING: f32 = 44.0;
/// The steering circle: its centre under the boresight and its radius.
const CIRCLE_AT: Vec2 = Vec2::new(CX, CY + 60.0);
const CIRCLE: f32 = 105.0;
/// The symbols' half width round the boresight and their left and right edges: the combiner
/// is as wide as the oval front pane (the aircraft's HUD scale fits this into it).
const HALF: f32 = 300.0;
const LEFT: f32 = CX - HALF;
const RIGHT: f32 = CX + HALF;
/// Radar altitudes shown (feet).
const RADAR_ALT_TOP: f32 = 5000.0;

fn dot(c: &mut Canvas, p: Vec2) {
    c.stroke(p, p, PEN * 1.5, INK);
}

/// An open triangle with its point at `p` towards `dir`.
fn triangle(c: &mut Canvas, p: Vec2, dir: Vec2, size: f32) {
    let side = Vec2::new(-dir.y, dir.x);
    let (a, b) = (p - dir * size + side * size * 0.6, p - dir * size - side * size * 0.6);
    c.polyline(&[p, a, b, p], THIN, INK);
}

/// A ring of ten dots round `at` with the needle at `turn` (0..1 clockwise from the top), the
/// number in it and `over` above it.
fn dial(c: &mut Canvas, at: Vec2, turn: f32, number: &str, over: Option<String>) {
    let dir = |t: f32| {
        let a = t * std::f32::consts::TAU;
        Vec2::new(a.sin(), -a.cos())
    };
    for k in 0..10 {
        dot(c, at + dir(k as f32 / 10.0) * RING);
    }
    c.stroke(at + dir(turn) * (RING - 20.0), at + dir(turn) * (RING - 3.0), PEN, INK);
    text(c, at.x, at.y, number, Align::Centre);
    if let Some(s) = over {
        text(c, at.x, at.y - RING - 18.0, &s, Align::Centre);
    }
}

/// The selected weapon as the HUD names it with its rounds: SW, SP, PH, G for the gun.
fn weapon_label(f: &HudFrame) -> Option<String> {
    if f.gun_selected {
        return Some("G".to_string());
    }
    let (count, name) = f.weapon.as_ref()?;
    let upper = name.to_uppercase();
    let short = if upper.contains("SIDEWINDER") || upper.contains("AIM-9") {
        "SW"
    } else if upper.contains("SPARROW") || upper.contains("AIM-7") {
        "SP"
    } else if upper.contains("PHOENIX") || upper.contains("AIM-54") {
        "PH"
    } else {
        return Some(format!("{upper} {count}"));
    };
    Some(format!("{short}{count}"))
}

/// The radar's mode in the HUD's letters (TWSA, PDSTT).
fn mode_label(mode: &str) -> String {
    match mode {
        "TWS AUTO" => "TWSA".to_string(),
        "TWS MAN" => "TWSM".to_string(),
        m => m.replace(' ', ""),
    }
}

/// "hh:mm:ss" of seconds of the day.
fn clock(secs: f32) -> String {
    let s = secs.max(0.0) as i64 % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

fn heading_scale(c: &mut Canvas, f: &HudFrame) {
    let hdg = euler(f).0.to_degrees();
    let first = ((hdg - TAPE_HALF) / 5.0).ceil() as i32;
    for k in first..=((hdg + TAPE_HALF) / 5.0).floor() as i32 {
        let x = CX + (k as f32 * 5.0 - hdg) * TAPE_PX;
        let long = k.rem_euclid(2) == 0;
        let len = if long { 12.0 } else { 6.0 };
        c.stroke(Vec2::new(x, TAPE_Y - len), Vec2::new(x, TAPE_Y), THIN, INK);
        if long {
            let tens = (k * 5).rem_euclid(360) / 10;
            small(c, x, TAPE_Y + 12.0, &format!("{tens:02}"), Align::Centre);
        }
    }
    triangle(c, Vec2::new(CX, TAPE_Y - 14.0), Vec2::Y, 13.0);
    if let Some(b) = f.waypoint_bearing {
        let brg = b as f32 * 360.0 / 65536.0;
        let off = (brg - hdg + 540.0).rem_euclid(360.0) - 180.0;
        let x = CX + off.clamp(-TAPE_HALF, TAPE_HALF) * TAPE_PX;
        triangle(c, Vec2::new(x, TAPE_Y + 34.0), Vec2::Y, 11.0);
    }
}

/// The horizon and the pitch ladder centred on the flight path marker.
fn ladder(c: &mut Canvas, f: &HudFrame, fpm: Vec2, h: Vec2, down: Vec2) {
    let inv = f.orientation.inverse();
    let track = Vec3::new(f.path_dir.x, 0.0, f.path_dir.z)
        .try_normalize()
        .or_else(|| {
            let n = f.orientation * Vec3::NEG_Z;
            Vec3::new(n.x, 0.0, n.z).try_normalize()
        })
        .unwrap_or(Vec3::NEG_Z);
    let gamma = f.path_dir.y.clamp(-1.0, 1.0).asin().to_degrees();
    let fld = field(f);
    c.set_clip(LEFT as i32, TAPE_Y as i32 + 44, RIGHT as i32, fld.1.y as i32 + 10);
    for k in -18i32..=18 {
        let deg = k as f32 * 5.0;
        if (deg - gamma).abs() > 22.0 || deg.abs() > 89.0 {
            continue;
        }
        let rad = deg.to_radians();
        let dir = track * rad.cos() + Vec3::Y * rad.sin();
        let Some(p) = project(inv * dir, f.scale) else { continue };
        let p = p + h * (fpm - p).dot(h);
        if k == 0 {
            c.stroke(p - h * HALF, p - h * 34.0, PEN, INK);
            c.stroke(p + h * 34.0, p + h * HALF, PEN, INK);
            continue;
        }
        let (gap, len, tip) = (36.0, 66.0, 11.0);
        let towards = if k > 0 { down } else { -down };
        for side in [-1.0f32, 1.0] {
            let (a, b) = (p + h * side * gap, p + h * side * (gap + len));
            if k > 0 {
                c.stroke(a, b, PEN, INK);
            } else {
                c.dashed(a, b, 13.0, 9.0, PEN, INK);
            }
            c.stroke(a, a + towards * tip, PEN, INK);
        }
        let lp = p - h * (gap + len + 6.0) - towards * 9.0;
        small(c, lp.x, lp.y, &format!("{}", k * 5), Align::Right);
    }
    c.reset_clip();
}

/// The flight path marker: an arc on its wings, the caret beside it; crossed out held at the
/// field's edge.
fn path_marker(c: &mut Canvas, p: Vec2, caged: bool) {
    let r = 9.0;
    c.arc(p, r, std::f32::consts::PI, std::f32::consts::TAU, PEN, INK);
    c.stroke(p - Vec2::X * r, p - Vec2::X * (r + 20.0), PEN, INK);
    c.stroke(p + Vec2::X * r, p + Vec2::X * (r + 12.0), PEN, INK);
    let k = p + Vec2::X * (r + 20.0);
    c.polyline(&[k + Vec2::new(9.0, -7.0), k, k + Vec2::new(9.0, 7.0)], PEN, INK);
    if caged {
        cross_out(c, p, r);
    }
}

/// The steering circle with the range on it, the steering symbol, the target's box.
fn air_to_air(c: &mut Canvas, f: &HudFrame) {
    let fld = field(f);
    let fld = (Vec2::new(LEFT + 16.0, fld.0.y), Vec2::new(RIGHT - 16.0, fld.1.y));
    let cues = &f.cues;
    let boresight = Vec2::new(CX, CY);
    let locked = cues.target_dir.and_then(|d| project(d, f.scale));
    // The next track-while-scan target with nothing locked: a dotted box with its number.
    let next =
        f.tomcat.as_ref().and_then(|t| t.next).filter(|_| locked.is_none()).and_then(
            |(d, km, n)| project(f.orientation.inverse() * d, f.scale).map(|p| (p, km, n)),
        );
    let target = locked.or(next.map(|t| t.0));
    let range_km = cues
        .launch_zone
        .map(|z| z.range_km)
        .or(f.target_range_km)
        .filter(|_| locked.is_some())
        .or(next.map(|t| t.1));
    let s = 14.0;
    if let Some(p) = locked {
        let q = p.clamp(fld.0, fld.1);
        c.frame(q - Vec2::splat(s), q + Vec2::splat(s), PEN, INK);
        for d in
            [Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0), Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0)]
        {
            dot(c, q + d * s);
        }
        if q != p {
            cross_out(c, q, s);
        }
        let shoot = match cues.launch_zone {
            Some(z) => z.shoot,
            None => cues.lock && cues.in_range,
        };
        if shoot && f.blink {
            text(c, q.x, q.y - s - 16.0, "SHOOT", Align::Centre);
        }
        if cues.launch_zone.is_some_and(|z| z.too_close) {
            text(c, q.x, q.y - s - 16.0, "BREAK", Align::Centre);
        }
    } else if let Some((p, _, n)) = next {
        let q = p.clamp(fld.0, fld.1);
        let corners = [Vec2::new(-s, -s), Vec2::new(s, -s), Vec2::new(s, s), Vec2::new(-s, s)];
        for k in 0..4 {
            c.dashed(q + corners[k], q + corners[(k + 1) % 4], 3.0, 4.0, THIN, INK);
        }
        if q != p {
            cross_out(c, q, s);
        }
        small(c, q.x, q.y - s - 12.0, &format!("{n}"), Align::Centre);
    }
    if let Some(t) = &f.tomcat {
        acm_scan(c, f, t);
    }
    if !f.weapon_air_to_air && !f.gun_selected {
        return;
    }
    if let Some(label) = weapon_label(f) {
        triangle(c, CIRCLE_AT + Vec2::Y * (CIRCLE + 4.0), Vec2::NEG_Y, 11.0);
        text(c, CIRCLE_AT.x, CIRCLE_AT.y + CIRCLE + 30.0, &label, Align::Centre);
    }
    if f.gun_selected {
        return;
    }
    // The circle: dashed, solid from the top round to the target's range of the scale.
    let at = |turn: f32| {
        let a = turn * std::f32::consts::TAU;
        CIRCLE_AT + Vec2::new(a.sin(), -a.cos()) * CIRCLE
    };
    let zone = cues.launch_zone.and_then(|z| z.zone_km);
    let scale = range_km.map(|r| {
        let most = r.max(zone.map_or(0.0, |z| z.1)) / KM_NM;
        [5.0, 10.0, 20.0, 50.0, 100.0, 200.0].into_iter().find(|s| *s >= most).unwrap_or(400.0)
            * KM_NM
    });
    let solid = range_km.zip(scale).map_or(0.0, |(r, s)| (r / s).clamp(0.0, 1.0));
    let n = 72;
    for k in 0..n {
        let (a, b) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
        if b <= solid {
            c.stroke(at(a), at(b), PEN * 1.4, INK);
        } else if k % 2 == 0 {
            c.stroke(at(a), at(b), THIN, INK);
        }
    }
    if let (Some((lo, hi)), Some(scale)) = (zone, scale) {
        for r in [lo, hi] {
            let p = at((r / scale).clamp(0.0, 1.0));
            let out = (p - CIRCLE_AT).normalize_or_zero();
            triangle(c, p + out * 3.0, -out, 11.0);
        }
    }
    // The steering symbol: the target against the nose, held inside the circle.
    if let Some(p) = target {
        let q = CIRCLE_AT + (p - boresight).clamp_length_max(CIRCLE - 12.0);
        c.ring(q, 7.0, PEN, INK);
        c.stroke(q - Vec2::X * 7.0, q - Vec2::X * 18.0, PEN, INK);
        c.stroke(q + Vec2::X * 7.0, q + Vec2::X * 18.0, PEN, INK);
    }
}

/// The readouts at the lower left and right.
fn readouts(c: &mut Canvas, f: &HudFrame) {
    let st = &f.state;
    let line = 22.0;
    // Angle of attack, Mach number, load factor.
    let (x, y) = (LEFT + 22.0, 340.0);
    for (k, (name, value)) in [
        ("\u{3b1}", format!("{:.1}", f.aoa)),
        ("M", format!("{:.2}", f.mach)),
        ("G", format!("{:.1}", f.gees as f32 / 16.0)),
    ]
    .into_iter()
    .enumerate()
    {
        let yy = y + k as f32 * line;
        text(c, x, yy, name, Align::Left);
        text(c, x + 76.0, yy, &value, Align::Right);
    }
    // The radar's mode, the bearing and range to the steering point, the weapon to drop.
    let y = y + 3.0 * line + 6.0;
    if let Some(t) = &f.tomcat {
        text(c, LEFT, y, &mode_label(t.mode), Align::Left);
    }
    let nm = f.steerpoint.map(|s| s.1 / KM_NM);
    if let Some((b, nm)) = f.waypoint_bearing.zip(nm) {
        let brg = b as f32 * 360.0 / 65536.0;
        text(c, LEFT, y + line, &format!("BA {brg:03.0}/{nm:.0}"), Align::Left);
    }
    if st.mode == HudMode::Gnd {
        let weapon = if f.gun_selected {
            Some("GUN".to_string())
        } else {
            f.weapon.as_ref().map(|w| w.1.to_uppercase())
        };
        if let Some(w) = weapon {
            text(c, LEFT, y + 2.0 * line, &format!("CP {w}"), Align::Left);
        }
    }
    if let Some(t) = f.throttle {
        text(c, LEFT, y + 3.0 * line, &super::throttle_text(t), Align::Left);
    }
    // Right: the bomb's release data, else the target's data over the steering point's.
    if let Some(b) = f.cues.bomb.filter(|_| st.mode == HudMode::Gnd) {
        return bomb_readouts(c, &b);
    }
    let mut rows: Vec<(&str, String)> = Vec::new();
    let locked = f.cues.target_dir.filter(|_| st.mode == HudMode::Air);
    let next = f.tomcat.as_ref().and_then(|t| t.next).filter(|_| st.mode == HudMode::Air);
    let target = locked
        .map(|d| (f.orientation * d, f.cues.launch_zone.map(|z| z.range_km).or(f.target_range_km)))
        .or(next.map(|(d, km, _)| (d, Some(km))));
    if let Some((d, km)) = target {
        let brg = d.x.atan2(-d.z).to_degrees().rem_euclid(360.0);
        rows.push(("BRG", format!("{brg:03.0}")));
        if let Some(km) = km {
            rows.push(("RNG", format!("{:.1}", km / KM_NM)));
        }
        if let Some(alt) = f.tomcat.as_ref().and_then(|t| t.target_alt) {
            rows.push(("ALT", format!("{alt:.1}")));
        }
        if let Some(vc) = f.tomcat.as_ref().and_then(|t| t.closure) {
            rows.push(("VC", format!("{vc:.0}")));
        }
    }
    if let Some((n, km, secs)) = f.steerpoint {
        rows.push(("WP", format!("{n} {:.1}", km / KM_NM)));
        if let Some(t) = &f.tomcat {
            rows.push(("Z", clock(t.clock)));
            rows.push(("TTG", clock(secs.clamp(0.0, 86_399.0))));
            rows.push(("ETA", clock(t.clock + secs.max(0.0))));
        }
    }
    if let Some(t) = &f.tomcat
        && let Some(s) = t.in_flight.iter().copied().min_by(|a, b| a.abs().total_cmp(&b.abs()))
    {
        let word = if s < 0.0 { "ACT" } else { "TTI" };
        rows.push((word, format!("{} {}", t.in_flight.len(), min_sec(s.abs()))));
    }
    let top = 330.0;
    for (k, (name, value)) in rows.iter().enumerate() {
        let yy = top + k as f32 * line;
        text(c, RIGHT - 142.0, yy, name, Align::Left);
        text(c, RIGHT, yy, value, Align::Right);
    }
}

/// Draws the F-14's HUD (and the VDI's repeat of it).
pub fn draw(c: &mut Canvas, f: &HudFrame) {
    c.reset_clip();
    let st = &f.state;
    let inv = f.orientation.inverse();
    let (h, down) = horizon_axes(f);
    let fld = field(f);
    let fpm_raw = project(inv * f.path_dir, f.scale);
    let fpm = fpm_raw.map_or(Vec2::new(CX, fld.1.y), |p| p.clamp(fld.0, fld.1));
    let caged = fpm_raw.is_none_or(|p| !inside(fld, p));
    if st.declutter < 2 || st.mode == HudMode::Nav {
        ladder(c, f, fpm, h, down);
    }
    path_marker(c, fpm, caged);
    // The armament datum line's cross at the boresight.
    for d in [Vec2::X, Vec2::Y] {
        c.stroke(Vec2::new(CX, CY) - d * 12.0, Vec2::new(CX, CY) + d * 12.0, PEN, INK);
    }
    heading_scale(c, f);
    let (kt, alt) = (f.knots.max(0), f.alt.max(0));
    let radar_alt = (f.radar_alt < RADAR_ALT_TOP).then(|| format!("{:.0}R", f.radar_alt.max(0.0)));
    if st.declutter == 0 {
        let ground = (kt as f32 * f.path_dir.y.clamp(-1.0, 1.0).asin().cos()).round();
        let over = Some(format!("{ground:.0}G"));
        dial(c, SPEED_AT, kt as f32 % 100.0 / 100.0, &format!("{kt}"), over);
        let feet = alt / 10 * 10;
        dial(c, ALT_AT, alt as f32 % 1000.0 / 1000.0, &format!("{feet}"), radar_alt);
    } else {
        boxed(c, SPEED_AT.x, SPEED_AT.y, &format!("{kt}"), 3, Align::Centre);
        boxed(c, ALT_AT.x, ALT_AT.y, &format!("{}", alt / 10 * 10), 5, Align::Centre);
        if let Some(r) = radar_alt {
            text(c, ALT_AT.x, ALT_AT.y - 30.0, &r, Align::Centre);
        }
    }
    readouts(c, f);
    if let Some(d) = f.steerpoint_dir {
        steerpoint_cue(c, inv * d, f);
    }
    if let Some(d) = f.tgp_dir {
        pod_cue(c, inv * d, f);
    }
    match st.mode {
        HudMode::Nav => {}
        HudMode::Air => air_to_air(c, f),
        HudMode::Gnd => air_to_ground(c, f, fpm, down),
    }
    gun_sight(c, f);
    rocket_sight(c, f);
    if let Some((loc, gs)) = f.ils {
        ils_bars(c, fpm, h, down, loc, gs);
        text(c, RIGHT - 142.0, 308.0, "AWL", Align::Left);
    }
    if let Some(m) = &f.message {
        text(c, CX, 160.0, m, Align::Centre);
    }
    let accel = super::accel_label(f.accel);
    let status: Vec<&str> = [(accel.as_deref(), true), (Some("TRAINING"), f.training)]
        .iter()
        .filter(|s| s.1)
        .filter_map(|s| s.0)
        .collect();
    if !status.is_empty() {
        text(c, CX, 182.0, &status.join("  "), Align::Centre);
    }
    if let Some(r) = &f.cues.recon {
        text(c, CX, 204.0, &r.text("TGP"), Align::Centre);
    }
    let _ = TEXT;
}

/// The VDI's repeat of the HUD's symbols: the picture's field (`VDI_FROM`, a square) drawn at
/// a wider angle into `c`'s texels in the tube's ink `ink`, its strokes thickened for the tube.
pub fn draw_vdi(c: &mut Canvas, f: &HudFrame, ink: u8) {
    let mut wide = f.clone();
    wide.scale = f.scale * VDI_ZOOM;
    wide.field_bottom = None;
    let mut hud = Canvas::new(super::modern::WIDTH, super::modern::HEIGHT);
    draw(&mut hud, &wide);
    c.clear(0);
    let (pw, ph, hw) = (c.pw(), c.ph(), hud.pw());
    let side = VDI_SIDE.min(pw).min(ph);
    let (ox, oy) = ((pw - side) / 2, (ph - side) / 2);
    let set = |x: usize, y: usize| {
        let (sx, sy) = (x + VDI_FROM.0, (y + VDI_FROM.1) as isize - VDI_DROP as isize);
        sy >= 0 && (sy as usize) < hud.ph() && sx < hw && hud.px[sy as usize * hw + sx] != 0
    };
    for y in 0..side {
        for x in 0..side {
            // A texel is lit with any of its neighbours to the right or below: thicker strokes.
            if set(x, y) || set(x + 1, y) || set(x, y + 1) {
                c.px[(oy + y) * pw + ox + x] = ink;
            }
        }
    }
}

/// The VDI's picture: the HUD canvas's square from this corner (the field's width), dropped
/// this many rows to stand in the tube's middle, drawn at this share of the HUD's scale.
const VDI_FROM: (usize, usize) = (240, 0);
const VDI_SIDE: usize = 800;
const VDI_DROP: usize = 120;
const VDI_ZOOM: f32 = 0.6;

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mode draws; the VDI's repeat is the tube's ink only, inside its canvas.
    #[test]
    fn the_hud_and_its_repeat_draw() {
        for mode in [HudMode::Nav, HudMode::Air, HudMode::Gnd] {
            let mut f = HudFrame { scale: 900.0, knots: 420, alt: 22_350, ..Default::default() };
            f.state.mode = mode;
            f.path_dir = Vec3::NEG_Z;
            f.weapon = Some((4, "Sidewinder".to_string()));
            f.weapon_air_to_air = true;
            f.cues.target_dir = Some(Vec3::new(0.05, 0.02, -1.0));
            let mut c = Canvas::new(super::super::modern::WIDTH, super::super::modern::HEIGHT);
            draw(&mut c, &f);
            assert!(c.px.iter().any(|p| *p != 0), "{mode:?}");
            let mut v = Canvas::scaled(400, 400, 2);
            draw_vdi(&mut v, &f, 100);
            assert!(v.px.iter().any(|p| *p == 100));
            assert!(v.px.iter().all(|p| *p == 0 || *p == 100));
        }
        assert_eq!(
            weapon_label(&HudFrame { gun_selected: true, ..Default::default() }).unwrap(),
            "G"
        );
        assert_eq!(mode_label("TWS AUTO"), "TWSA");
        assert_eq!(clock(3_725.0), "01:02:05");
    }
}
