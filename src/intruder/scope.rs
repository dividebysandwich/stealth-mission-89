//! The A-6's round scopes: the pilot's horizontal display (PHD), the bombardier/navigator's
//! direct view radar indicator (DVRI) and his sensor scope's overlay over the pod's picture.
//! The round scopes show the search radar's ground map as a sector from the tube's foot (the
//! objects and ground units it painted bright, the keyed target coordinates in a diamond, the
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
    /// The target's coordinates keyed into the computer (m east, north).
    pub keyed: Option<DVec2>,
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

/// A drawing unit of the tube under a sector picture: its share of the reach from the tube's
/// foot (past 1 off the picture) and its bearing right of the nose, with the bearing's sine and
/// cosine.
#[derive(Clone, Copy)]
struct Polar {
    r: f32,
    az: f32,
    sin: f32,
    cos: f32,
}

/// The tube's units in rows over its bounding square (None outside the tube), for a canvas of
/// this size: the same every frame, so worked out once (`sector`).
struct PolarGrid {
    size: (usize, usize),
    x0: i32,
    y0: i32,
    n: usize,
    cells: Vec<Option<Polar>>,
}

impl PolarGrid {
    fn new(c: &Canvas) -> Self {
        let (mid, rad) = glass(c);
        let foot = mid + Vec2::Y * rad * FOOT;
        let reach = rad * REACH;
        let (x0, y0) = ((mid.x - rad) as i32, (mid.y - rad) as i32);
        let n = ((mid.x + rad) as i32 - x0 + 1) as usize;
        let rows = ((mid.y + rad) as i32 - y0 + 1) as usize;
        let cells = (0..rows * n)
            .map(|i| {
                let (x, y) = (x0 + (i % n) as i32, y0 + (i / n) as i32);
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                (p.distance_squared(mid) <= rad * rad).then(|| {
                    let d = p - foot;
                    let az = d.x.atan2(-d.y);
                    let (sin, cos) = az.sin_cos();
                    Polar { r: d.length() / reach, az, sin, cos }
                })
            })
            .collect();
        Self { size: (c.w, c.h), x0, y0, n, cells }
    }

    /// The grid for this canvas' size, made the first time.
    fn of(c: &Canvas) -> std::sync::Arc<PolarGrid> {
        static GRIDS: std::sync::Mutex<Vec<std::sync::Arc<PolarGrid>>> =
            std::sync::Mutex::new(Vec::new());
        let mut grids = GRIDS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(g) = grids.iter().find(|g| g.size == (c.w, c.h)) {
            return g.clone();
        }
        let g = std::sync::Arc::new(PolarGrid::new(c));
        grids.push(g.clone());
        g
    }
}

/// A sector picture: `shade(the unit's place)` per unit of the tube inside `half` radians
/// either side of the nose and inside the reach, 0 dark.
fn sector(c: &mut Canvas, half: f32, shade: impl Fn(&Polar) -> f32) {
    let grid = PolarGrid::of(c);
    let (s, pw) = (c.scale, c.pw());
    for (i, cell) in grid.cells.iter().enumerate() {
        let Some(p) = cell else { continue };
        let v = if p.r <= 1.0 && p.az.abs() <= half { shade(p) } else { 0.0 };
        let (x, y) = (grid.x0 + (i % grid.n) as i32, grid.y0 + (i / grid.n) as i32);
        if x < 0 || y < 0 || x as usize >= c.w || y as usize >= c.h {
            continue;
        }
        // The unit's texels, row by row (as `Canvas::set`, without its clip: the tube is
        // inside the canvas).
        let colour = if v > 0.0 { ink(v) } else { 0 };
        let at = y as usize * s * pw + x as usize * s;
        for row in 0..s {
            c.px[at + row * pw..at + row * pw + s].fill(colour);
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
    // The shade of each of the picture's levels, and the bearing's sine and cosine turned to
    // the heading.
    let shades: [f32; 256] = std::array::from_fn(|l| {
        let b = l as f32 / 255.0;
        if amti { 0.04 + b * 0.22 } else { 0.04 + b.powf(0.8) * 0.9 }
    });
    let (sh, ch) = s.heading.sin_cos();
    sector(c, SCAN_HALF, |p| {
        let (sin, cos) = (sh * p.cos + ch * p.sin, ch * p.cos - sh * p.sin);
        let d = p.r * range;
        shades[radar.level(s.own.x + (sin * d) as f64, s.own.y + (cos * d) as f64) as usize]
    });
    range_marks(c, SCAN_HALF, 0.3);
    // A point's (m east, north) bearing and share of the range, if it is on the sector.
    let polar = |e: f64, n: f64| {
        let d = DVec2::new(e, n) - s.own;
        let az = super::computer::wrap((d.x as f32).atan2(d.y as f32) - s.heading);
        let r = d.length() as f32 / range;
        (r < 1.0 && az.abs() < SCAN_HALF).then_some((az, r))
    };
    // What can be attacked stands out of the ground's clutter: the world's objects (the
    // mission's targets among them) and the ground units the beam painted, bright; with AMTI
    // only what moves.
    for ret in &radar.returns {
        let p = match ret.target {
            RadarTarget::Unit(_) => polar(ret.east, ret.north),
            RadarTarget::Object(_) if !amti => polar(ret.east, ret.north),
            _ => None,
        };
        if let Some((az, r)) = p {
            let p = place(c, az, r);
            seg(c, p - Vec2::X * 2.0, p + Vec2::X * 2.0, 2.4, ink(1.0));
        }
    }
    // The keyed coordinates: a diamond round the point.
    if let Some((az, r)) = s.keyed.and_then(|k| polar(k.x, k.y)) {
        let p = place(c, az, r);
        let q = [p - Vec2::Y * 6.0, p + Vec2::X * 6.0, p + Vec2::Y * 6.0, p - Vec2::X * 6.0];
        for i in 0..4 {
            seg(c, q[i], q[(i + 1) % 4], 1.2, ink(0.9));
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
    sector(c, terrain::HALF, |p| {
        let b = ((p.az / terrain::HALF + 1.0) / 2.0 * (terrain::BEARINGS - 1) as f32).round();
        let i = ((p.r * range / terrain::REACH).sqrt() * terrain::RANGES as f32 - 1.0).round();
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

    /// The objects the radar painted and the keyed coordinates show on the ground map.
    #[test]
    fn ground_map_marks_targets_and_keyed_point() {
        use crate::ground_radar::Return;
        let computer = Computer::default();
        let scan = TerrainScan::default();
        let range = super::super::SearchRange::Mi15;
        let metres = range.miles() as f64 * 1852.0;
        let draw = |radar: &GroundRadar, keyed: Option<DVec2>| {
            let panel = IntruderPanel { search_power: Power::On, search_range: range, ..default() };
            let s = Scene {
                own: DVec2::ZERO,
                heading: 0.0,
                alt: 1000.0,
                climb: 0.0,
                computer: &computer,
                radar,
                scan: &scan,
                keyed,
            };
            let mut c = Canvas::scaled(CANVAS.0, CANVAS.1, SCALE);
            draw_dvri(&mut c, &Fonts::builtin(), &panel, &s);
            c
        };
        let bright = |c: &Canvas, at: Vec2| {
            (-4..=4)
                .flat_map(|dy| (-7..=7).map(move |dx| (dx, dy)))
                .filter(|(dx, dy)| {
                    let (x, y) = (at.x as i32 + dx, at.y as i32 + dy);
                    let i = (y as usize * SCALE) * c.pw() + x as usize * SCALE;
                    c.px[i] >= ink(0.85)
                })
                .count()
        };
        let mut radar = GroundRadar::default();
        radar.on = true;
        let empty = draw(&radar, None);
        let probe = Canvas::scaled(CANVAS.0, CANVAS.1, SCALE);
        let object = place(&probe, 0.0, 0.5);
        let keyed = place(&probe, 0.3, 0.7);
        assert_eq!(bright(&empty, object), 0);
        assert_eq!(bright(&empty, keyed), 0);
        radar.returns.push(Return::at(0.0, metres * 0.5, RadarTarget::Object(1)));
        let k = DVec2::new(0.3f64.sin(), 0.3f64.cos()) * metres * 0.7;
        let marked = draw(&radar, Some(k));
        assert!(bright(&marked, object) > 3, "object");
        assert!(bright(&marked, keyed) > 3, "keyed");
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
                keyed: None,
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
