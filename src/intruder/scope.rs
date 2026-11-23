//! The A-6's round scopes: the pilot's horizontal display (PHD), the bombardier/navigator's
//! direct view radar indicator (DVRI) and his sensor scope's overlay over the pod's picture.
//! The round scopes show the search radar's ground map as a sector from the tube's foot (the
//! cursors on the computer's target), the track radar's terrain clearance plan (the ground
//! that stands into the clearance plane) and its elevation scan (the ground along the nose
//! against the clearance line).

use bevy::math::DVec2;
use bevy::prelude::*;

use super::computer::Computer;
use super::terrain::{self, TerrainScan};
use super::{IntruderPanel, PhdMode, Power, disc, ink};
use crate::canvas::Canvas;
use crate::font::{Align, Fonts, stroke_text_at};
use crate::ground_radar::{GroundRadar, RadarTarget, SCAN_HALF};

/// The round scopes' canvas (drawing units) and its texels per unit; the sensor scope's
/// overlay (4:3).
pub const CANVAS: (usize, usize) = (320, 320);
pub const SENSOR_CANVAS: (usize, usize) = (320, 240);
pub const SCALE: usize = 2;

/// The horizontal display's green tube, the direct view indicator's orange storage tube and
/// the sensor scope's pale green symbols.
pub fn phd_colours() -> std::sync::Arc<[u32; 256]> {
    with_ring(super::phosphor([3, 12, 5], [80, 255, 90]))
}

pub fn dvri_colours() -> std::sync::Arc<[u32; 256]> {
    with_ring(super::phosphor([40, 14, 2], [255, 150, 30]))
}

/// Colours past the phosphor's ramp: the heading ring's black metal, its white engraving and
/// the fixed index's orange.
const RING: u8 = 101;
const WHITE: u8 = 102;
const INDEX: u8 = 103;

fn with_ring(ramp: std::sync::Arc<[u32; 256]>) -> std::sync::Arc<[u32; 256]> {
    let mut lut = *ramp;
    lut[RING as usize] = u32::from_le_bytes([10, 10, 11, 255]);
    lut[WHITE as usize] = u32::from_le_bytes([225, 225, 215, 255]);
    lut[INDEX as usize] = u32::from_le_bytes([255, 150, 40, 255]);
    std::sync::Arc::new(lut)
}

pub fn sensor_colours() -> std::sync::Arc<[u32; 256]> {
    super::phosphor([4, 8, 5], [170, 255, 180])
}

/// The scope's tube inside its heading ring: its radius as a fraction of the canvas' half
/// width (the cockpit model's glass covers that much, `tools/models/build_cockpit_a6.py`).
pub const TUBE: f32 = 0.76;

/// The heading ring round the tube: a compass card turned to the heading (rad) under the fixed
/// index at the top, ticks every 5 degrees, longer every 10, numbered (tens of degrees) every 30.
fn heading_ring(c: &mut Canvas, heading: f32) {
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    let rad = mid.x.min(mid.y);
    let (inner, outer) = (rad * (TUBE + 0.03), rad * 0.99);
    for i in 0..72 {
        let deg = i as f32 * 5.0;
        let a = deg.to_radians() - heading;
        let d = Vec2::new(a.sin(), -a.cos());
        let len = if i % 2 == 0 { 0.4 } else { 0.22 };
        c.stroke(mid + d * inner, mid + d * (inner + (outer - inner) * len), 1.0, WHITE);
        if i % 6 == 0 {
            let p = mid + d * (inner + (outer - inner) * 0.7);
            let label = format!("{}", i * 5 / 10);
            stroke_text_at(c, p.x, p.y, &label, rad * 0.075, 1.0, WHITE, Align::Centre);
        }
    }
    // The fixed index.
    let top = mid - Vec2::Y * outer;
    c.triangle(
        [top + Vec2::Y * rad * 0.07, top + Vec2::X * rad * 0.035, top - Vec2::X * rad * 0.035],
        INDEX,
    );
}

/// What a scope shows of the flight.
pub struct Scene<'a> {
    /// The aircraft (m east, north; rad; m above the sea), where it goes above the level
    /// (rad) and its speed (m/s).
    pub own: DVec2,
    pub heading: f32,
    pub alt: f32,
    pub climb: f32,
    pub computer: &'a Computer,
    pub radar: &'a GroundRadar,
    pub scan: &'a TerrainScan,
}

/// The sector pictures' foot under the tube's middle and their reach, in tube radii.
const FOOT: f32 = 0.66;
const REACH: f32 = 1.6;

/// The tube: its middle and radius (drawing units).
fn glass(c: &Canvas) -> (Vec2, f32) {
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    (mid, mid.x.min(mid.y) * TUBE - 1.0)
}

/// A line drawn where it is on the tube.
fn seg(c: &mut Canvas, a: Vec2, b: Vec2, w: f32, colour: u8) {
    let (mid, rad) = glass(c);
    const N: usize = 24;
    let on = |p: Vec2| p.distance_squared(mid) < rad * rad;
    for i in 0..N {
        let (p, q) = (a.lerp(b, i as f32 / N as f32), a.lerp(b, (i + 1) as f32 / N as f32));
        if on(p) && on(q) {
            c.stroke(p, q, w, colour);
        }
    }
}

/// A sector picture: `shade(bearing right of the nose, share of the range)` per point of the
/// tube inside `half` radians either side, 0 dark.
fn sector(c: &mut Canvas, half: f32, shade: impl Fn(f32, f32) -> f32) {
    let (mid, rad) = glass(c);
    let foot = mid + Vec2::Y * rad * FOOT;
    let reach = rad * REACH;
    for y in (mid.y - rad) as i32..=(mid.y + rad) as i32 {
        for x in (mid.x - rad) as i32..=(mid.x + rad) as i32 {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            if p.distance_squared(mid) > rad * rad {
                continue;
            }
            let d = p - foot;
            let (r, az) = (d.length() / reach, d.x.atan2(-d.y));
            let v = if r <= 1.0 && az.abs() <= half { shade(az, r) } else { 0.0 };
            c.set(x, y, if v > 0.0 { ink(v) } else { 0 });
        }
    }
}

/// Where a point `bearing` right of the nose at `r` of the range shows on a sector picture.
fn place(c: &Canvas, bearing: f32, r: f32) -> Vec2 {
    let (mid, rad) = glass(c);
    mid + Vec2::Y * rad * FOOT + Vec2::new(bearing.sin(), -bearing.cos()) * r * rad * REACH
}

/// Range marks across a sector at every quarter of the range.
fn range_marks(c: &mut Canvas, half: f32, shade: f32) {
    for i in 1..=4 {
        let r = i as f32 / 4.0;
        for k in 0..16 {
            let (a, b) = (-half + half * k as f32 / 8.0, -half + half * (k + 1) as f32 / 8.0);
            let (p, q) = (place(c, a, r), place(c, b, r));
            seg(c, p, q, 0.8, ink(shade));
        }
    }
}

/// The search radar's ground map out to `range` metres, the beam's line and the cursors on
/// the computer's target; with `amti` the ground dim and what moves bright.
fn ground_map(c: &mut Canvas, s: &Scene, range: f32, amti: bool) {
    let radar = s.radar;
    if !radar.on {
        return;
    }
    let at = |az: f32, r: f32| {
        let (sin, cos) = (s.heading + az).sin_cos();
        (s.own.x + (sin * r * range) as f64, s.own.y + (cos * r * range) as f64)
    };
    sector(c, SCAN_HALF, |az, r| {
        let (e, n) = at(az, r);
        let b = radar.brightness(e, n);
        if amti { 0.04 + b * 0.22 } else { 0.04 + b.powf(0.8) * 0.9 }
    });
    range_marks(c, SCAN_HALF, 0.3);
    if amti {
        for ret in &radar.returns {
            if !matches!(ret.target, RadarTarget::Unit(_)) {
                continue;
            }
            let d = DVec2::new(ret.east, ret.north) - s.own;
            let az = super::computer::wrap((d.x as f32).atan2(d.y as f32) - s.heading);
            let r = d.length() as f32 / range;
            if r < 1.0 && az.abs() < SCAN_HALF {
                let p = place(c, az, r);
                seg(c, p - Vec2::X * 2.0, p + Vec2::X * 2.0, 2.4, ink(1.0));
            }
        }
    }
    let foot = place(c, 0.0, 0.0);
    let beam = place(c, radar.beam, 1.0);
    seg(c, foot, beam, 1.0, ink(0.55));
    // The cursors: the azimuth line and the range mark cross on the target.
    if s.computer.target.is_some() {
        let (az, r) = (s.computer.bearing, s.computer.range / range);
        if az.abs() < SCAN_HALF {
            let edge = place(c, az, 1.0);
            seg(c, foot, edge, 0.8, ink(0.8));
            if r < 1.0 {
                for k in -4..4 {
                    let (a, b) = (az + k as f32 * 0.03, az + (k + 1) as f32 * 0.03);
                    let (p, q) = (place(c, a, r), place(c, b, r));
                    seg(c, p, q, 1.6, ink(1.0));
                }
            }
        }
    }
}

/// The terrain clearance plan: the ground within `terrain::HALF` of the nose out to `range`
/// metres that stands into the plane `clearance` metres under the aircraft, brighter the
/// higher it stands.
fn clearance_plan(c: &mut Canvas, s: &Scene, range: f32, clearance: f32) {
    if !s.scan.on {
        return;
    }
    let (scan, alt) = (s.scan, s.alt);
    sector(c, terrain::HALF, |az, r| {
        let b = ((az / terrain::HALF + 1.0) / 2.0 * (terrain::BEARINGS - 1) as f32).round();
        let i = ((r * range / terrain::REACH).sqrt() * terrain::RANGES as f32 - 1.0).round();
        let over = scan.height(b as usize, i.max(0.0) as usize) - (alt - clearance);
        if over > 0.0 { (0.35 + over / 300.0).min(1.0) } else { 0.03 }
    });
    range_marks(c, terrain::HALF, 0.4);
    let (foot, top) = (place(c, 0.0, 0.0), place(c, 0.0, 1.0));
    seg(c, foot, top, 0.8, ink(0.5));
}

/// The elevation scan: the ground along the nose out to `range` metres from the tube's left,
/// 10 degrees up to 15 down, the flight vector's line and the clearance line `clearance`
/// metres under it.
fn elevation_scan(c: &mut Canvas, s: &Scene, range: f32, clearance: f32) {
    let (mid, rad) = glass(c);
    disc(c, mid, rad, ink(0.03));
    if !s.scan.on {
        return;
    }
    let (x0, span) = (mid.x - rad * 0.9, rad * 1.8);
    let per_rad = rad * 1.5 / 25f32.to_radians();
    let at = |d: f32, el: f32| {
        Vec2::new(x0 + d / range * span, mid.y - (el + 2.5f32.to_radians()) * per_rad)
    };
    // Range marks at every quarter and the level line.
    for i in 1..=4 {
        let d = range * i as f32 / 4.0;
        seg(c, at(d, 0.17), at(d, -0.26), 0.7, ink(0.25));
    }
    seg(c, at(0.0, 0.0), at(range, 0.0), 0.7, ink(0.3));
    // The ground the radar sees.
    let mut last: Option<Vec2> = None;
    for (d, h, seen) in s.scan.profile(s.alt) {
        let p = (seen && d <= range).then(|| at(d, (h / d).atan().clamp(-0.26, 0.17)));
        if let (Some(a), Some(b)) = (last, p) {
            seg(c, a, b, 2.0, ink(0.95));
        }
        last = p;
    }
    // The flight vector and the clearance line under it.
    seg(c, at(0.0, s.climb), at(range, s.climb), 1.0, ink(0.6));
    let mut last = None;
    for i in 1..=32 {
        let d = range * i as f32 / 32.0;
        let p = at(d, (s.climb - (clearance / d).atan()).max(-0.26));
        if let Some(a) = last {
            seg(c, a, p, 1.2, ink(0.8));
        }
        last = Some(p);
    }
}

/// The tube dark (off) or faintly lit (standby), the ring round it.
fn tube(c: &mut Canvas, glow: f32) {
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    c.clear(RING);
    disc(c, mid, mid.x.min(mid.y) * TUBE, if glow > 0.0 { ink(glow) } else { 0 });
}

/// The pilot's horizontal display, by its mode and power switches, in its heading ring.
pub fn draw_phd(c: &mut Canvas, _fonts: &Fonts, panel: &IntruderPanel, s: &Scene) {
    c.reset_clip();
    match panel.phd_power {
        Power::Off => tube(c, 0.0),
        Power::Standby => tube(c, 0.04),
        Power::On => {
            tube(c, 0.03);
            let range = panel.scale.miles() as f32 * 1852.0;
            let clearance = panel.clearance.feet() as f32 * 0.3048;
            match panel.phd_mode {
                PhdMode::Tcppi => clearance_plan(c, s, range, clearance),
                PhdMode::EScan => elevation_scan(c, s, range, clearance),
                PhdMode::Search if panel.search_power == Power::On => {
                    ground_map(c, s, panel.search_range.miles() as f32 * 1852.0, panel.amti)
                }
                PhdMode::Search => {}
            }
        }
    }
    heading_ring(c, s.heading);
}

/// The direct view radar indicator: the search radar's ground map, by its power switch, in
/// its heading ring.
pub fn draw_dvri(c: &mut Canvas, _fonts: &Fonts, panel: &IntruderPanel, s: &Scene) {
    c.reset_clip();
    match panel.search_power {
        Power::Off => tube(c, 0.0),
        Power::Standby => tube(c, 0.04),
        Power::On => {
            tube(c, 0.03);
            ground_map(c, s, panel.search_range.miles() as f32 * 1852.0, panel.amti);
        }
    }
    heading_ring(c, s.heading);
}

/// The sensor scope's overlay over the pod's picture (index 0 see-through): a reticle, open
/// in the middle, the track's state and the range to the point looked at (nautical miles).
pub fn draw_sensor(c: &mut Canvas, _fonts: &Fonts, computer: &Computer) {
    c.reset_clip();
    c.clear(crate::canvas::TRANSPARENT);
    let (w, h) = (c.w as f32, c.h as f32);
    let mid = Vec2::new(w / 2.0, h / 2.0);
    for d in [Vec2::X, Vec2::Y] {
        c.stroke(mid + d * 10.0, mid + d * 40.0, 1.2, ink(0.9));
        c.stroke(mid - d * 10.0, mid - d * 40.0, 1.2, ink(0.9));
    }
    let state = if computer.discrete {
        "TRACK"
    } else if computer.designated {
        "GROUND"
    } else {
        "SLEW"
    };
    stroke_text_at(c, 12.0, h - 14.0, state, 8.0, 1.2, ink(0.8), Align::Left);
    if computer.designated {
        let miles = computer.range / 1852.0;
        stroke_text_at(
            c,
            w - 12.0,
            h - 14.0,
            &format!("{miles:.1}"),
            8.0,
            1.2,
            ink(0.8),
            Align::Right,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(c: &Canvas, over: f32) -> usize {
        c.px.iter().filter(|p| (ink(over)..=100).contains(*p)).count()
    }

    /// Ground that stands into the clearance plane shows on the plan and over the clearance
    /// line of the elevation scan; flat ground far below shows on neither.
    #[test]
    fn terrain_pictures_show_what_stands_in_the_way() {
        let computer = Computer::default();
        let radar = GroundRadar::default();
        let mut scan = TerrainScan::default();
        scan.on = true;
        let draw = |scan: &TerrainScan, mode: PhdMode| {
            let panel = IntruderPanel { phd_mode: mode, ..default() };
            let s = Scene {
                own: DVec2::ZERO,
                heading: 0.0,
                alt: 300.0,
                climb: 0.0,
                computer: &computer,
                radar: &radar,
                scan,
            };
            let mut c = Canvas::scaled(CANVAS.0, CANVAS.1, SCALE);
            draw_phd(&mut c, &Fonts::builtin(), &panel, &s);
            c
        };
        let flat = lit(&draw(&scan, PhdMode::Tcppi), 0.34);
        // A ridge 400 m high across the fan from 3 to 4 km.
        for b in 0..terrain::BEARINGS {
            for i in 0..terrain::RANGES {
                if (3000.0..4000.0).contains(&terrain::range_at(i)) {
                    scan.ground[b * terrain::RANGES + i] = 400.0;
                }
            }
        }
        let ridge = lit(&draw(&scan, PhdMode::Tcppi), 0.34);
        assert!(ridge > flat + 500, "{ridge} {flat}");
        assert!(lit(&draw(&scan, PhdMode::EScan), 0.9) > 50);
    }
}
