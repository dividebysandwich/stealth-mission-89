//! The F-14's displays drawn on their canvases (green phosphor: `ink`, index 0 the dark tube):
//!
//! - **DDD** (the RIO's detail data display): a B-scope of the AWG-9's returns, range against
//!   azimuth (pulse Doppler search: closure against azimuth), the scan's limits, the antenna's
//!   carets, a single target track's gates and its readouts, jammers' strobes.
//! - **TID** (the RIO's tactical information display): the tactical picture around the aircraft
//!   (ground stabilised north up, aircraft stabilised nose up, or attack: nose up with the
//!   aircraft at the foot) out to its range: the TWS tracks in the NTDS symbols (hostile a
//!   peaked hat, friendly a round one, unknown a square one) with their velocity leaders, track
//!   numbers and shot priorities, the Phoenixes' time to go, the hooked track's data, the scan's
//!   limits and the waypoints.
//! - **HSD** (the pilot's horizontal situation display): the compass rose turned to the heading
//!   with the course to the steering point, or the TID's picture repeated, or the threats.
//! - **VDI** (the pilot's vertical display): the HUD's symbols repeated (`hud::vdig`).
//! - **ECMD** (the RIO's electronic countermeasures display, the ALR-67's): the radars painting
//!   the aircraft round its centre, the most dangerous inside, launches flashing.
//! - **LANTIRN**: the targeting pod's symbology over its picture (on the TID's TV mode and the
//!   VDI's).

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use crate::awg9::{Awg9, Awg9Controls, GIMBAL, TrackFile};
use crate::canvas::Canvas;
use crate::font::{Align, Fonts, stroke_text_at};
use crate::mfd::ContactRef;

/// Metres per nautical mile; knots per m/s.
const NM: f64 = 1852.0;
const KT: f32 = 1.943_844;

/// The phosphor at a brightness 0..1 (`intruder::ink`).
fn ink(t: f32) -> u8 {
    crate::intruder::ink(t)
}

const BRIGHT: f32 = 1.0;
const MID: f32 = 0.65;
const DIM: f32 = 0.35;

fn text(c: &mut Canvas, p: Vec2, s: &str, h: f32, t: f32, align: Align) {
    stroke_text_at(c, p.x, p.y, s, h, 1.0, ink(t), align);
}

/// The aircraft: position (metres east, north, up), heading, pitch and roll (radians), speed.
#[derive(Debug, Clone, Copy, Default)]
pub struct Own {
    pub pos: DVec3,
    pub heading: f32,
    pub pitch: f32,
    #[allow(dead_code)]
    pub roll: f32,
    pub knots: f32,
}

// ---------------------------------------------------------------------------------------------
// DDD

/// The DDD's closure scale (knots): -200 at the foot, this at the top (twice it with the Vc
/// switch at 2X).
pub const VC_TOP: f32 = 1200.0;
/// The elevations the DDD's antenna and infrared pictures show (radians either side of the
/// horizon).
const EL_SPAN: f32 = 60.0 * std::f32::consts::PI / 180.0;

/// The DDD's picture (its display mode buttons).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DddPicture {
    /// PULSE VIDEO: the raw returns by range (as strong as they are near), with the ground's
    /// clutter while the antenna looks down. A mode without ranges (pulse Doppler search) shows
    /// none.
    Pulse,
    /// PD: the radar's processed returns, by range (pulse Doppler search: by closing speed).
    #[default]
    Pd,
    /// ANT TRK: where the antenna looks: the scan's pattern and its bars by azimuth and
    /// elevation, the returns at theirs.
    Antenna,
    /// RDROT: the returns by closing speed in every mode (the rate display).
    Rate,
    /// JAT: the jammers' strobes only (their bearings; the returns are off).
    Jam,
    /// IROT: what the infrared sensor sees, by azimuth and elevation (no radar return needed).
    Ir,
}

impl DddPicture {
    /// Up the scope is elevation (not range or closure).
    fn angles(self) -> bool {
        matches!(self, DddPicture::Antenna | DddPicture::Ir)
    }

    /// Up the scope is the closing speed in the radar's mode `mode`.
    fn rate(self, mode: crate::awg9::Mode) -> bool {
        self == DddPicture::Rate || (self == DddPicture::Pd && mode.velocity_only())
    }
}

/// What the DDD shows: its picture, its closure scale's top (knots) and the hand controller's
/// acquisition cursor (azimuth in radians, 0..1 up the scope), if it is on this display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DddView {
    pub picture: DddPicture,
    pub vc_top: f32,
    pub cursor: Option<Vec2>,
}

impl Default for DddView {
    fn default() -> Self {
        Self { picture: DddPicture::Pd, vc_top: VC_TOP, cursor: None }
    }
}

/// A return's place on the DDD: its azimuth (radians) and how far up the scope it is (0 the
/// foot, 1 the top: its range of the scope's, its closing speed or its elevation by the
/// picture). `el`: radians over the horizon, `range`: metres, `closure`: m/s.
pub fn ddd_spot(
    r: &Awg9,
    ctl: &Awg9Controls,
    v: &DddView,
    (az, el): (f32, f32),
    range: f32,
    closure: f32,
) -> Vec2 {
    let up = if v.picture.angles() {
        el / EL_SPAN * 0.5 + 0.5
    } else if v.picture.rate(r.mode) {
        (closure * KT + 200.0) / (v.vc_top + 200.0)
    } else {
        range / ctl.range
    };
    Vec2::new(az, up)
}

/// The DDD's antenna line's colour index (white in `tomcat::displays::ddd_colours`; the ramp
/// is the dark ink on the lit tube).
pub const DDD_SWEEP: u8 = 200;

/// The DDD: returns, gates and carets of the radar `r` set by `ctl` in the picture `v` (the
/// cursor: two bars over each other at its place). The returns the IFF found friendly get bars
/// over and under them. The tube is lit and its marks dark (`tomcat::displays::ddd_colours`),
/// the antenna's azimuth a white line up it.
pub fn draw_ddd(c: &mut Canvas, r: &Awg9, ctl: &Awg9Controls, v: &DddView) {
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    let m = 0.08 * w;
    let (x0, x1, y0, y1) = (m, w - m, m, h - m);
    let az_x = |az: f32| x0 + (az / GIMBAL * 0.5 + 0.5) * (x1 - x0);
    let picture = v.picture;
    let (angles, rate) = (picture.angles(), picture.rate(r.mode));
    // A mark's row by the picture.
    let row = |el: f32, range: f32, closure: f32| {
        let up = ddd_spot(r, ctl, v, (0.0, el), range, closure).y;
        y1 - up.clamp(0.0, 1.05) * (y1 - y0)
    };
    // The frame's grid: azimuth every 10 degrees along the foot with 30 and 0 printed, the two
    // range lines at 30 degrees either side with their tenths, the cross in the middle.
    for k in -6i32..=6 {
        let x = az_x(k as f32 * 10f32.to_radians());
        let len = if k % 3 == 0 { 0.03 * h } else { 0.015 * h };
        c.stroke(Vec2::new(x, y1), Vec2::new(x, y1 + len), 1.0, ink(MID));
        if k % 3 == 0 && k.abs() <= 3 {
            let s = format!("{}", (k * 10).abs());
            text(c, Vec2::new(x, y1 + 0.055 * h), &s, 0.03 * h, MID, Align::Centre);
        }
    }
    for s in [-1.0, 1.0] {
        let x = az_x(s * 30f32.to_radians());
        c.stroke(Vec2::new(x, y0), Vec2::new(x, y1), 1.0, ink(MID));
        for q in 0..=10 {
            let y = y1 - (y1 - y0) * q as f32 / 10.0;
            let len = if q % 5 == 0 { 0.03 * w } else { 0.015 * w };
            c.stroke(Vec2::new(x - len, y), Vec2::new(x + len, y), 1.0, ink(MID));
        }
    }
    let mid = Vec2::new(az_x(0.0), (y0 + y1) / 2.0);
    for d in [Vec2::X, Vec2::Y] {
        c.stroke(mid - d * 0.03 * h, mid + d * 0.03 * h, 1.0, ink(MID));
    }
    let title = |c: &mut Canvas, s: &str| {
        text(c, Vec2::new(w / 2.0, 0.04 * h), s, 0.035 * h, MID, Align::Centre);
    };
    let cursor = |c: &mut Canvas| {
        if let Some(k) = v.cursor {
            let p = Vec2::new(az_x(k.x), y1 - k.y.clamp(0.0, 1.0) * (y1 - y0));
            for sy in [-1.0, 1.0] {
                let y = p.y + sy * 0.012 * h;
                c.stroke(
                    Vec2::new(p.x - 0.04 * w, y),
                    Vec2::new(p.x + 0.04 * w, y),
                    1.5,
                    ink(BRIGHT),
                );
            }
        }
    };
    let hooked = r.hook;
    let hook_box = |c: &mut Canvas, x: f32, y: f32| {
        let half = 0.036 * w;
        c.frame(
            Vec2::new(x - half, y - 0.03 * h),
            Vec2::new(x + half, y + 0.03 * h),
            1.0,
            ink(1.0),
        );
    };
    // The infrared sensor's picture: the hot spots by azimuth and elevation, with the radar or
    // without it.
    if picture == DddPicture::Ir {
        c.dashed(Vec2::new(x0, mid.y), Vec2::new(x1, mid.y), 4.0, 6.0, 1.0, ink(DIM));
        for spot in &r.ir {
            let (x, y) = (az_x(spot.az), row(spot.el, 0.0, 0.0));
            let s = 0.02 * w;
            c.polyline(
                &[
                    Vec2::new(x, y - s),
                    Vec2::new(x + s, y),
                    Vec2::new(x, y + s),
                    Vec2::new(x - s, y),
                    Vec2::new(x, y - s),
                ],
                1.5,
                ink(BRIGHT),
            );
            if Some(spot.target) == hooked {
                hook_box(c, x, y);
            }
        }
        cursor(c);
        return title(c, "IR  EL 60");
    }
    if !r.mode.on() {
        text(c, Vec2::new(w / 2.0, h * 0.3), "STBY", 0.07 * h, MID, Align::Centre);
        return;
    }
    // The scan's limits; in the antenna's picture its whole pattern: the bars between them.
    let scan = ctl.scan();
    if !r.mode.stt() {
        let half = scan.az_half;
        for s in [-1.0, 1.0] {
            let x = az_x(s * half);
            c.dashed(Vec2::new(x, y0), Vec2::new(x, y1), 4.0, 6.0, 1.0, ink(DIM));
        }
        if angles && !r.mode.acm() {
            for bar in 0..scan.bars.max(1) {
                let y = row(scan.bar_elevation(bar), 0.0, 0.0);
                let t = if bar == r.bar { MID } else { DIM };
                c.dashed(
                    Vec2::new(az_x(-half), y),
                    Vec2::new(az_x(half), y),
                    6.0,
                    4.0,
                    1.0,
                    ink(t),
                );
            }
        }
    }
    // The antenna: its azimuth a white line up the scope, its elevation a caret at the left.
    let bx = az_x(r.beam);
    c.stroke(Vec2::new(bx, y0), Vec2::new(bx, y1), 2.5, DDD_SWEEP);
    let ey = (y0 + y1) / 2.0 - r.elevation / EL_SPAN * (y1 - y0) / 2.0;
    c.triangle(
        [
            Vec2::new(x0 - 0.01 * w, ey),
            Vec2::new(x0 - 0.05 * w, ey - 0.02 * h),
            Vec2::new(x0 - 0.05 * w, ey + 0.02 * h),
        ],
        ink(MID),
    );
    if angles {
        // Where the beam is now.
        c.ring(Vec2::new(bx, ey), 0.025 * w, 1.5, ink(BRIGHT));
    }
    // Jammers: strobes up their azimuth (the jamming's picture: nothing else, and darker).
    let jam = picture == DddPicture::Jam;
    for s in &r.strobes {
        let x = az_x(s.az);
        let t = (1.0 - s.age / (2.0 * r.frame)).clamp(0.2, 1.0);
        let (width, dark) = if jam { (3.0, BRIGHT) } else { (1.5, MID) };
        c.stroke(Vec2::new(x, y0), Vec2::new(x, y1), width, ink(dark * t));
        if jam && Some(s.target) == hooked {
            hook_box(c, x, mid.y);
        }
    }
    cursor(c);
    if jam {
        return title(c, &format!("JAT  {}", r.strobes.len()));
    }
    let pulse = picture == DddPicture::Pulse;
    if pulse && r.elevation < 0.0 {
        // The ground's clutter under the beam: speckles from where the beam meets the ground
        // outwards, new ones as the antenna moves.
        let mut seed = (r.beam * 4000.0) as i32 as u32 ^ 0x9E37_79B9;
        let mut next = || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / 16_777_216.0
        };
        let near = (0.15 / (-r.elevation).max(0.02)).clamp(0.05, 1.0);
        for _ in 0..90 {
            let x = bx + (next() - 0.5) * 0.12 * w;
            let y = y1 - (near + next() * (1.0 - near)).min(1.0) * (y1 - y0);
            c.stroke(Vec2::new(x, y), Vec2::new(x + 0.01 * w, y), 0.008 * h, ink(DIM * next()));
        }
    }
    // The raw video of a mode without ranges has no returns to show.
    let blind = pulse && r.mode.velocity_only();
    for hit in r.hits.iter().filter(|_| !blind) {
        let x = az_x(hit.az);
        let y = row(hit.el, hit.range, hit.closure);
        let fade = (1.0 - hit.age / (2.0 * r.frame)).clamp(0.15, 1.0);
        // The raw video: the nearer the stronger (wider and thicker).
        let strength =
            if pulse { (1.6 - hit.range / ctl.range.max(1.0)).clamp(0.6, 1.6) } else { 1.0 };
        let half = 0.018 * w * strength;
        c.stroke(
            Vec2::new(x - half, y),
            Vec2::new(x + half, y),
            0.012 * h * strength,
            ink(BRIGHT * fade),
        );
        if hit.friendly == Some(true) {
            // The IFF's reply: grey bars over and under the return.
            for sy in [-1.0, 1.0] {
                let by = y + sy * 0.022 * h;
                let b = 1.4 * half;
                c.stroke(Vec2::new(x - b, by), Vec2::new(x + b, by), 0.008 * h, ink(0.4 * fade));
            }
        }
        if Some(hit.target) == hooked {
            hook_box(c, x, y);
        }
    }
    // A single target track: its gates round it, its data along the top.
    if let Some(t) = r.stt {
        let x = az_x(t.az);
        let y = row(t.el, t.range, t.closure);
        let (gx, gy) = (0.035 * w, 0.03 * h);
        for s in [-1.0, 1.0] {
            c.stroke(
                Vec2::new(x - gx, y + s * gy),
                Vec2::new(x + gx, y + s * gy),
                1.5,
                ink(BRIGHT),
            );
            c.stroke(
                Vec2::new(x + s * gx * 1.6, y - gy),
                Vec2::new(x + s * gx * 1.6, y + gy),
                1.5,
                ink(BRIGHT),
            );
        }
        c.stroke(Vec2::new(x - gx * 0.6, y), Vec2::new(x + gx * 0.6, y), 0.012 * h, ink(BRIGHT));
        let data = format!(
            "{:.1}NM  VC {:+.0}  {:.0}KT",
            t.range as f64 / NM,
            t.closure * KT,
            t.speed * KT
        );
        text(c, Vec2::new(w / 2.0, y0 + 0.035 * h), &data, 0.03 * h, MID, Align::Centre);
    }
    // The mode and scale over the scope (the tube's corners are round).
    let scale = if angles {
        "EL 60".to_string()
    } else if rate {
        format!("VC {:.0}", v.vc_top)
    } else if blind {
        "NO RANGE".to_string()
    } else {
        format!("{:.0}", ctl.range as f64 / NM)
    };
    title(c, &format!("{}  {scale}", r.mode.label()));
}

// ---------------------------------------------------------------------------------------------
// TID

/// How the TID is turned and where the aircraft is on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TidStab {
    /// North up, the aircraft in the middle.
    #[default]
    Ground,
    /// Nose up, the aircraft in the middle.
    Aircraft,
    /// Nose up, the aircraft low on the tube (the attack picture ahead).
    Attack,
}

/// A Phoenix (or Sparrow) of ours in flight on the TID: its target and seconds to go
/// (negative: to its seeker coming on, positive: to impact once active).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InFlight {
    pub target: ContactRef,
    pub to_go: f32,
    /// Seconds to its impact, whether its seeker is on yet or not.
    pub tti: f32,
}

impl InFlight {
    /// Its own seeker is on (it went "pitbull").
    pub fn active(&self) -> bool {
        self.to_go >= 0.0
    }
}

/// The TID's vectors' full length in tube radii (1.5 inches on the tube): a velocity leader's at
/// `VECTOR_KNOTS`, a launch zone vector's at `ZONE_SECS`.
const VECTOR_FULL: f32 = 0.43;
const VECTOR_KNOTS: f32 = 1800.0;
const ZONE_SECS: f32 = 180.0;
/// The launch zone vectors replace the velocity leaders this many seconds before a track comes
/// into the Phoenix's range.
const ZONE_SHOWN: f32 = 60.0;
/// The time to impact flashes this often a second once the missile's own seeker is on.
const TTI_FLASH: f32 = 3.0;

/// The seconds until a track closing at `closure` m/s from `range` m is at the Phoenix's
/// shortest, best and longest launch ranges (the launch zone vector's TUMR, TUOR and TUIR;
/// negative: already inside). None for a track that does not close. The ranges are the
/// computer's estimates by the closing speed: some 20 nmi against a slow or beam target up to
/// 60 against a fast one head on, the best shot at half of that.
pub fn launch_times(range: f32, closure: f32) -> Option<[f32; 3]> {
    if closure < 20.0 {
        return None;
    }
    let nm = NM as f32;
    let longest = (20.0 + closure * 0.07).min(60.0) * nm;
    Some([2.0 * nm, 0.5 * longest, longest].map(|r| (range - r) / closure))
}

/// What the TID shows.
pub struct TidView<'a> {
    pub radar: &'a Awg9,
    pub controls: &'a Awg9Controls,
    pub own: Own,
    /// Its range (nautical miles from the aircraft to the tube's edge ahead).
    pub range_nm: f32,
    pub stab: TidStab,
    /// The route's steering points (metres east, north) and the current one.
    pub waypoints: &'a [DVec2],
    pub steer: Option<usize>,
    pub in_flight: &'a [InFlight],
    /// The velocity leaders and the tracks' altitudes (in thousands of feet) are shown.
    pub vel_vectors: bool,
    pub alt_numbers: bool,
    /// The hand controller's cursor (`tid_spot`'s units), if it is on this display.
    pub cursor: Option<Vec2>,
    /// Seconds (the time to impact's flashing).
    pub t: f32,
}

/// A point's place on the TID (metres east, north) in tube radii from the tube's middle (x
/// right, y up; beyond 1: off the tube).
pub fn tid_spot(stab: TidStab, own: &Own, range_nm: f32, p: DVec2) -> Vec2 {
    let c = Canvas::new(64, 64);
    let tube = Tube::new(&c, stab, own, range_nm);
    let d = (tube.at(p) - tube.mid) / tube.rad;
    Vec2::new(d.x, -d.y)
}

/// The tube: its centre and radius on the canvas, the aircraft's place on it.
struct Tube {
    mid: Vec2,
    rad: f32,
    own: Vec2,
    up: f32,
    scale: f32,
    origin: DVec3,
}

impl Tube {
    fn new(c: &Canvas, stab: TidStab, own: &Own, range_nm: f32) -> Self {
        let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
        let rad = mid.x.min(mid.y) * 0.96;
        let (own_at, reach) = match stab {
            TidStab::Attack => (mid + Vec2::Y * rad * 0.7, rad * 1.7),
            _ => (mid, rad),
        };
        let up = if stab == TidStab::Ground { 0.0 } else { own.heading };
        Tube {
            mid,
            rad,
            own: own_at,
            up,
            scale: reach / (range_nm.max(1.0) * NM as f32),
            origin: own.pos,
        }
    }

    /// A world point (metres east, north) on the canvas.
    fn at(&self, p: DVec2) -> Vec2 {
        let d = (p - self.origin.truncate()).as_vec2();
        let (s, c) = self.up.sin_cos();
        // Turned so `up` points up the tube (canvas y down).
        let right = d.x * c - d.y * s;
        let fwd = d.x * s + d.y * c;
        self.own + Vec2::new(right, -fwd) * self.scale
    }

    /// A direction (radians clockwise from north) on the tube.
    fn dir(&self, bearing: f32) -> Vec2 {
        let a = bearing - self.up;
        Vec2::new(a.sin(), -a.cos())
    }

    fn inside(&self, p: Vec2) -> bool {
        p.distance(self.mid) <= self.rad
    }

    /// Where the line from `a` (inside) towards `b` leaves the tube, or `b` if it does not.
    fn clip(&self, a: Vec2, b: Vec2) -> Vec2 {
        if self.inside(b) {
            return b;
        }
        let d = b - a;
        let f = a - self.mid;
        let (qa, qb, qc) = (d.dot(d), 2.0 * f.dot(d), f.dot(f) - self.rad * self.rad);
        let k = (-qb + (qb * qb - 4.0 * qa * qc).max(0.0).sqrt()) / (2.0 * qa.max(1e-6));
        a + d * k.clamp(0.0, 1.0)
    }

    /// A ring round `centre`, only where it is on the tube.
    fn ring(&self, c: &mut Canvas, centre: Vec2, r: f32, t: f32) {
        let n = 96;
        let at = |k: usize| {
            let a = k as f32 / n as f32 * std::f32::consts::TAU;
            centre + Vec2::new(a.cos(), a.sin()) * r
        };
        for k in 0..n {
            let (a, b) = (at(k), at(k + 1));
            if self.inside(a) && self.inside(b) {
                c.stroke(a, b, 1.0, ink(t));
            }
        }
    }
}

/// An NTDS track symbol at `p`: hostile a peaked hat, friendly a round one, unknown a square
/// one (open below).
fn ntds(c: &mut Canvas, p: Vec2, s: f32, friendly: Option<bool>, t: f32) {
    match friendly {
        Some(false) => c.polyline(
            &[p + Vec2::new(-s, s * 0.3), p - Vec2::Y * s, p + Vec2::new(s, s * 0.3)],
            1.5,
            ink(t),
        ),
        Some(true) => c.arc(
            p + Vec2::Y * s * 0.3,
            s,
            std::f32::consts::PI,
            std::f32::consts::TAU,
            1.5,
            ink(t),
        ),
        None => c.polyline(
            &[
                p + Vec2::new(-s, s * 0.3),
                p + Vec2::new(-s, -s * 0.7),
                p + Vec2::new(s, -s * 0.7),
                p + Vec2::new(s, s * 0.3),
            ],
            1.5,
            ink(t),
        ),
    }
}

/// The TID's picture.
pub fn draw_tid(c: &mut Canvas, v: &TidView) {
    c.clear(0);
    let tube = Tube::new(c, v.stab, &v.own, v.range_nm);
    let (rad, h) = (tube.rad, c.h as f32);
    // The tube's rim, the range rings (halves), the north mark.
    c.ring(tube.mid, rad, 1.0, ink(DIM));
    let ring_m = v.range_nm as f64 * NM / 2.0;
    for k in 1..=2 {
        let r = (ring_m * k as f64) as f32 * tube.scale;
        tube.ring(c, tube.own, r, DIM * 0.7);
    }
    let north = tube.mid + tube.dir(0.0) * rad * 0.92;
    text(c, north, "N", 0.04 * h, DIM, Align::Centre);
    // The scan's limits from the aircraft.
    let r = v.radar;
    if r.mode.on() && !r.mode.stt() {
        let half = v.controls.scan().az_half;
        for s in [-1.0, 1.0] {
            let d = tube.dir(v.own.heading + s * half);
            let end = tube.clip(tube.own, tube.own + d * rad * 2.0);
            c.dashed(tube.own, end, 5.0, 7.0, 1.0, ink(DIM));
        }
    }
    // The route.
    let pts: Vec<Vec2> = v.waypoints.iter().map(|&p| tube.at(p)).collect();
    for (k, w) in pts.windows(2).enumerate() {
        let t = if v.steer.is_some_and(|s| s == k + 1) { MID } else { DIM };
        if tube.inside(w[0]) {
            c.dashed(w[0], tube.clip(w[0], w[1]), 3.0, 5.0, 1.0, ink(t));
        } else if tube.inside(w[1]) {
            c.dashed(w[1], tube.clip(w[1], w[0]), 3.0, 5.0, 1.0, ink(t));
        }
    }
    for (k, &p) in pts.iter().enumerate() {
        if tube.inside(p) {
            let s = 0.015 * h;
            let t = if v.steer == Some(k) { BRIGHT } else { MID };
            c.polyline(
                &[
                    p - Vec2::Y * s,
                    p + Vec2::X * s,
                    p + Vec2::Y * s,
                    p - Vec2::X * s,
                    p - Vec2::Y * s,
                ],
                1.0,
                ink(t),
            );
        }
    }
    // The aircraft.
    let nose = tube.dir(v.own.heading);
    let side = Vec2::new(-nose.y, nose.x);
    let s = 0.03 * h;
    c.stroke(tube.own - nose * s, tube.own + nose * s, 1.5, ink(BRIGHT));
    c.stroke(tube.own - side * s * 0.8, tube.own + side * s * 0.8, 1.5, ink(BRIGHT));
    // The tracks.
    let hooked = r.hooked;
    let (hs, hc) = v.own.heading.sin_cos();
    let own_vel = DVec2::new(hs as f64, hc as f64) * (v.own.knots * v.own.pitch.cos() / KT) as f64;
    let full = VECTOR_FULL * rad;
    for t in &r.tracks {
        let p = tube.at(t.now().truncate());
        if !tube.inside(p) {
            continue;
        }
        let coasting = t.unseen > 2.0 * r.frame;
        let bright = if coasting { DIM } else { BRIGHT };
        ntds(c, p, 0.02 * h, t.friendly, bright);
        // The track's centre dot.
        c.stroke(p, p, 2.5, ink(bright));
        let vel = t.vel.truncate();
        let speed = vel.length();
        let along = if speed > 5.0 { tube.dir(vel.x.atan2(vel.y) as f32) } else { tube.dir(0.0) };
        // The launch zone vector of a hostile track about to come into range: its length the
        // time until the shortest range, the bar on it the best range and the dot the longest
        // as far from its end as the time until them (the end comes back to them).
        let d = t.now().truncate() - v.own.pos.truncate();
        let range = d.length();
        let closure = -(vel - own_vel).dot(d) / range.max(1.0);
        let zone = launch_times(range as f32, closure as f32)
            .filter(|z| t.friendly != Some(true) && z[2] < ZONE_SHOWN && z[0] > 0.0);
        if let Some([least, best, longest]) = zone {
            let len = least.min(ZONE_SECS) / ZONE_SECS * full;
            c.stroke(p, p + along * len, 1.0, ink(bright));
            let at = |secs: f32| len - secs / ZONE_SECS * full;
            let bar = at(best);
            if bar > 0.0 && best > 0.0 {
                let half = (0.02 * full).max(1.5);
                let (a, b) = ((bar - half).max(0.0), (bar + half).min(len));
                c.stroke(p + along * a, p + along * b, 4.0, ink(BRIGHT));
            }
            let dot = at(longest);
            if dot > 0.0 && longest > 0.0 {
                c.stroke(p + along * dot, p + along * dot, 3.5, ink(BRIGHT));
            }
        } else if v.vel_vectors && speed > 5.0 {
            // The velocity leader: the track's heading, its length the speed.
            let len = (speed as f32 * KT / VECTOR_KNOTS).min(1.0) * full;
            c.stroke(p, p + along * len.max(0.01 * h), 1.0, ink(bright));
        }
        if v.alt_numbers {
            // The altitude to the nearest ten thousand feet, left of the symbol.
            let alt = (t.pos.z * 3.2808 / 10_000.0).round().max(0.0);
            let at = p + Vec2::new(-0.03 * h, 0.0);
            text(c, at, &format!("{alt:.0}"), 0.032 * h, MID, Align::Right);
        }
        // Right of it: the missile's time to impact once one flies at it (flashing with the
        // missile's seeker on), else its place in the firing order.
        let right = p + Vec2::new(0.03 * h, 0.0);
        if let Some(m) = v.in_flight.iter().find(|m| m.target == t.target) {
            if !m.active() || (v.t * TTI_FLASH).fract() < 0.5 {
                let tti = format!("{:.0}", m.tti.clamp(0.0, 999.0));
                text(c, right, &tti, 0.032 * h, BRIGHT, Align::Left);
            }
        } else if let Some(k) = t.priority {
            text(c, right, &format!("{k}"), 0.032 * h, BRIGHT, Align::Left);
        }
        if hooked == Some(t.number) {
            let b = 0.035 * h;
            c.frame(p - Vec2::splat(b), p + Vec2::splat(b), 1.0, ink(BRIGHT));
        }
    }
    // A single target track not on the TWS tracks: its symbol from the STT.
    if let Some(t) = r.stt {
        let bearing = v.own.heading + t.az;
        let d = (t.range as f64 * (t.el as f64).cos()) as f32;
        let p = tube.own + tube.dir(bearing) * d * tube.scale;
        if tube.inside(p) {
            let b = 0.03 * h;
            ntds(c, p, 0.02 * h, r.identity(t.target), BRIGHT);
            c.ring(p, b, 1.0, ink(BRIGHT));
        }
    }
    // The hooked track's data along the foot.
    if let Some(t) = hooked.and_then(|n| r.track(n)) {
        let readout = track_readout(t, &v.own);
        text(c, Vec2::new(tube.mid.x, h - 0.06 * h), &readout, 0.035 * h, BRIGHT, Align::Centre);
    }
    let scale = format!("{:.0}", v.range_nm);
    text(c, tube.mid + Vec2::new(rad * 0.7, -rad * 0.75), &scale, 0.035 * h, MID, Align::Centre);
    // The hand controller's cursor: an open cross.
    if let Some(k) = v.cursor {
        let p = tube.mid + Vec2::new(k.x, -k.y) * rad;
        let (gap, arm) = (0.012 * h, 0.035 * h);
        for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            c.stroke(p + d * gap, p + d * arm, 1.5, ink(BRIGHT));
        }
    }
}

/// A track's data: number, bearing and range from us, altitude (thousands of feet), heading
/// and speed.
fn track_readout(t: &TrackFile, own: &Own) -> String {
    let d = t.now() - own.pos;
    let brg = (d.x.atan2(d.y).to_degrees() + 360.0) % 360.0;
    let hdg = (t.vel.x.atan2(t.vel.y).to_degrees() + 360.0) % 360.0;
    let kt = t.vel.truncate().length() as f32 * KT;
    format!(
        "{}  {:03.0}/{:.0}  {:.0}K  {:03.0}  {:.0}KT",
        t.number,
        brg,
        d.truncate().length() / NM,
        t.pos.z * 3.2808 / 1000.0,
        hdg,
        kt
    )
}

// ---------------------------------------------------------------------------------------------
// HSD

/// The pilot's HSD in navigation (after the real one's): the compass rose turned to the
/// heading under the lubber line, the steering point's diamond on the rose with the course
/// line through it, the range to it top left, the destination's block in the middle (its
/// name, bearing and range, true air speed, ground speed).
pub fn draw_hsd_nav(c: &mut Canvas, own: &Own, steer: Option<DVec2>, label: &str) {
    c.clear(0);
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0 + 0.02 * c.h as f32);
    let rad = mid.x.min(mid.y) * 0.62;
    let h = c.h as f32;
    let small = 0.032 * h;
    let at = |a: f32| Vec2::new(a.sin(), -a.cos());
    for i in 0..72 {
        let deg = i as f32 * 5.0;
        let d = at(deg.to_radians() - own.heading);
        let len = if i % 2 == 0 { 0.10 } else { 0.05 };
        c.stroke(mid + d * rad, mid + d * rad * (1.0 + len), 1.0, ink(MID));
        if i % 6 == 0 {
            let s = match i / 6 {
                0 => "N".to_string(),
                3 => "E".to_string(),
                6 => "S".to_string(),
                9 => "W".to_string(),
                k => format!("{}", k * 3),
            };
            text(c, mid + d * rad * 0.86, &s, small, MID, Align::Centre);
        }
    }
    // The lubber line.
    c.stroke(mid - Vec2::Y * rad * 1.28, mid - Vec2::Y * rad * 1.12, 1.5, ink(BRIGHT));
    let line = 1.25 * small;
    let tas = own.knots;
    let gs = own.knots * own.pitch.cos();
    let mut block = vec!["DEST".to_string()];
    if let Some(p) = steer {
        let d = p - own.pos.truncate();
        let brg = d.x.atan2(d.y) as f32;
        let dir = at(brg - own.heading);
        let side = Vec2::new(-dir.y, dir.x);
        // The course line through the steering point, its diamond on the rose and its
        // reciprocal's.
        let gap = 2.6 * line;
        c.stroke(mid + dir * gap, mid + dir * rad * 1.05, 1.0, ink(BRIGHT));
        c.stroke(mid - dir * gap, mid - dir * rad * 1.05, 1.0, ink(MID));
        let diamond = |c: &mut Canvas, p: Vec2, k: f32, t: f32| {
            let s = 0.035 * h * k;
            let q = [p + dir * s, p + side * s, p - dir * s, p - side * s, p + dir * s];
            c.polyline(&q, 1.0, ink(t));
        };
        diamond(c, mid + dir * rad * 1.05, 1.0, BRIGHT);
        diamond(c, mid - dir * rad * 1.05, 0.7, MID);
        let nm = d.length() / NM;
        text(c, Vec2::new(c.w as f32 * 0.08, 0.07 * h), "RNG", small, MID, Align::Left);
        text(
            c,
            Vec2::new(c.w as f32 * 0.08, 0.07 * h + line),
            &format!("{:03.0}.", nm.min(999.0)),
            small,
            BRIGHT,
            Align::Left,
        );
        let brg = (brg.to_degrees() + 360.0) % 360.0;
        block.push(format!("{label} {brg:03.0}/{:03.0}", nm.min(999.0)));
    }
    block.push(format!("TAS {:04.0}", tas.max(0.0)));
    block.push(format!("GS {:04.0}", gs.max(0.0)));
    let top = mid.y - (block.len() as f32 - 1.0) * line / 2.0;
    for (k, s) in block.iter().enumerate() {
        text(c, Vec2::new(mid.x, top + k as f32 * line), s, small, BRIGHT, Align::Centre);
    }
}

// ---------------------------------------------------------------------------------------------
// ECMD

/// A radar painting the aircraft on the ECMD.
#[derive(Debug, Clone, PartialEq)]
pub struct Threat {
    /// Bearing off the nose (radians, clockwise), its symbol, rank (0 the most dangerous).
    pub rel_bearing: f32,
    pub label: String,
    pub air: bool,
    pub rank: usize,
    pub tracking: bool,
    pub launch: bool,
}

/// The ECMD: the threats round the aircraft symbol, the most dangerous on the inner ring in a
/// diamond, those tracking inside the others; a launch flashes. `t` (s) blinks.
pub fn draw_ecmd(c: &mut Canvas, threats: &[Threat], ecm_on: bool, t: f32) {
    c.clear(0);
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    let rad = mid.x.min(mid.y) * 0.9;
    let h = c.h as f32;
    for k in [0.33, 0.66, 1.0] {
        c.ring(mid, rad * k, 1.0, ink(DIM));
    }
    for k in 0..12 {
        let a = k as f32 * std::f32::consts::TAU / 12.0;
        let d = Vec2::new(a.sin(), -a.cos());
        c.stroke(mid + d * rad * 0.97, mid + d * rad, 1.0, ink(MID));
    }
    let s = 0.05 * h;
    c.polyline(&[mid - Vec2::Y * s, mid + Vec2::Y * s], 1.5, ink(MID));
    c.polyline(&[mid - Vec2::X * s * 0.7, mid + Vec2::X * s * 0.7], 1.5, ink(MID));
    let blink = (t * 3.0).fract() < 0.5;
    // Symbols too close to read are moved apart along their ring, the more dangerous first
    // keeping their place.
    let apart = 0.13 * h;
    let mut placed: Vec<Vec2> = Vec::with_capacity(threats.len());
    for th in threats {
        let ring = if th.rank == 0 {
            0.33
        } else if th.tracking {
            0.58
        } else {
            0.8
        };
        let at = |a: f32| mid + Vec2::new(a.sin(), -a.cos()) * rad * ring;
        let step = (apart / (rad * ring)).min(0.6);
        let p = (0..24)
            .map(|k| {
                let n = (k + 1) / 2;
                let sign = if k % 2 == 1 { 1.0 } else { -1.0 };
                at(th.rel_bearing + sign * n as f32 * step)
            })
            .find(|p| placed.iter().all(|q| q.distance(*p) >= apart))
            .unwrap_or_else(|| at(th.rel_bearing));
        placed.push(p);
        if th.launch && !blink {
            continue;
        }
        let bright = if th.rank == 0 || th.launch { BRIGHT } else { MID };
        text(c, p, &th.label, 0.07 * h, bright, Align::Centre);
        if th.air {
            c.polyline(
                &[
                    p + Vec2::new(-0.04 * h, -0.045 * h),
                    p + Vec2::new(0.0, -0.07 * h),
                    p + Vec2::new(0.04 * h, -0.045 * h),
                ],
                1.0,
                ink(bright),
            );
        }
        if th.rank == 0 {
            let b = 0.07 * h;
            c.polyline(
                &[
                    p - Vec2::Y * b,
                    p + Vec2::X * b,
                    p + Vec2::Y * b,
                    p - Vec2::X * b,
                    p - Vec2::Y * b,
                ],
                1.0,
                ink(BRIGHT),
            );
        }
    }
    if ecm_on {
        text(c, Vec2::new(mid.x, h - 0.06 * h), "ECM", 0.05 * h, MID, Align::Centre);
    }
}

// ---------------------------------------------------------------------------------------------
// TCS

/// What the television camera set's picture shows over it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Tcs {
    /// The range of the track it follows (nmi); None: it looks along the nose.
    pub range_nm: Option<f32>,
    pub narrow: bool,
    /// The aircraft's heading (degrees).
    pub heading: f32,
}

/// The TCS's symbology over its picture (which shows through index 0): the open cross on what
/// it looks at, its field of view, the track's range or BRST (boresight, nothing tracked).
pub fn draw_tcs(c: &mut Canvas, d: &Tcs) {
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    let mid = Vec2::new(w / 2.0, h / 2.0);
    let (gap, arm) = (0.035 * h, 0.14 * h);
    for dir in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        c.stroke(mid + dir * gap, mid + dir * arm, 1.5, ink(BRIGHT));
    }
    let size = 0.05 * h;
    text(c, Vec2::new(0.10 * w, 0.10 * h), "TCS", size, BRIGHT, Align::Left);
    let fov = if d.narrow { "NAR" } else { "WIDE" };
    text(c, Vec2::new(0.90 * w, 0.10 * h), fov, size, BRIGHT, Align::Right);
    let hdg = format!("{:03.0}", d.heading.rem_euclid(360.0));
    text(c, Vec2::new(0.10 * w, 0.90 * h), &hdg, size, BRIGHT, Align::Left);
    let range = match d.range_nm {
        Some(r) => format!("{r:.1} NM"),
        None => "BRST".to_string(),
    };
    text(c, Vec2::new(0.90 * w, 0.90 * h), &range, size, BRIGHT, Align::Right);
}

// ---------------------------------------------------------------------------------------------
// LANTIRN

/// What the LANTIRN's symbology shows over its picture.
#[derive(Debug, Clone, Default)]
pub struct Lantirn {
    /// Latitude and longitude of the pod's point (degrees), its range (nmi, `SRA`), the pod's
    /// azimuth and elevation (degrees), its track mode's label.
    pub lat: f64,
    pub lon: f64,
    pub range_nm: f32,
    pub az: f32,
    pub el: f32,
    pub track: String,
    /// The aircraft's altitude (ft), ground speed (kt), dive angle (degrees, down positive).
    pub alt_ft: f32,
    pub ground_kt: f32,
    pub dive: f32,
    /// The heading (degrees), mission time (s), the laser's code and whether it fires, the FLIR's
    /// polarity (white hot), the field of view (wide), air-to-air mode.
    pub heading: f32,
    pub clock: f32,
    pub laser_code: u16,
    pub lasing: bool,
    pub white_hot: bool,
    /// The FLIR's picture (else the TV's).
    pub flir: bool,
    pub wide: bool,
    pub air: bool,
}

/// The LANTIRN's field of regard map: the limits of what the pod sees and where it looks, drawn
/// fixed on its picture (as the real pod's: the lines stay put, the gimbal dot moves). The map
/// is polar: straight down in the middle of the picture, the nose up, the angle off straight
/// down outward (`GIMBAL_MAP` picture heights per degree), so a dot ahead on the horizon is high
/// up the picture.
///
/// The lines are traced from the real pod's video (a 480 x 360 picture, `LIMIT_FRAME`): the
/// masking limit (where the airframe is in the way: down the left, along the foot and up to the
/// right) and the gimbal limit (top right). Some stretches are dotted, as on the real one.
const LIMIT_FRAME: (f32, f32) = (480.0, 360.0);
const MASK_LINE: &[(bool, &[(f32, f32)])] = &[
    // The masking limit: its hook at the top, down the left, along the foot.
    (
        true,
        &[(190.0, 7.0), (183.0, 9.0), (181.0, 15.0), (182.0, 23.0), (180.0, 31.0), (178.0, 38.0)],
    ),
    (
        false,
        &[
            (178.0, 38.0),
            (173.0, 43.0),
            (165.0, 45.0),
            (152.0, 48.0),
            (144.0, 52.0),
            (141.0, 57.0),
            (132.0, 70.0),
            (128.0, 80.0),
            (125.0, 90.0),
            (124.0, 100.0),
            (126.0, 110.0),
            (125.0, 120.0),
            (123.0, 135.0),
            (122.0, 150.0),
            (121.0, 165.0),
            (120.0, 180.0),
            (120.0, 195.0),
            (122.0, 207.0),
            (126.0, 220.0),
            (131.0, 235.0),
            (138.0, 250.0),
            (146.0, 262.0),
            (155.0, 272.0),
            (165.0, 280.0),
            (178.0, 284.0),
            (192.0, 285.0),
            (205.0, 283.0),
            (220.0, 281.0),
            (240.0, 281.0),
            (260.0, 282.0),
            (275.0, 285.0),
            (290.0, 289.0),
            (302.0, 293.0),
            (313.0, 297.0),
        ],
    ),
    // Down into its notch, then up to the right.
    (true, &[(313.0, 297.0), (330.0, 307.0), (345.0, 318.0), (352.0, 324.0)]),
    (
        false,
        &[
            (352.0, 324.0),
            (370.0, 310.0),
            (390.0, 290.0),
            (410.0, 268.0),
            (420.0, 256.0),
            (427.0, 248.0),
        ],
    ),
    (true, &[(427.0, 248.0), (440.0, 249.0), (455.0, 249.0), (470.0, 250.0)]),
    // The gimbal limit.
    (
        false,
        &[
            (350.0, 2.0),
            (365.0, 9.0),
            (380.0, 16.0),
            (393.0, 22.0),
            (405.0, 30.0),
            (418.0, 40.0),
            (430.0, 50.0),
            (442.0, 60.0),
        ],
    ),
    (true, &[(442.0, 60.0), (452.0, 70.0), (460.0, 78.0), (466.0, 82.0)]),
];
/// The region the pod sees within the lines (closed far beyond the picture): `occluded` is
/// outside it.
const SEEN: &[(f32, f32)] = &[
    (190.0, 7.0),
    (182.0, 23.0),
    (178.0, 38.0),
    (165.0, 45.0),
    (144.0, 52.0),
    (132.0, 70.0),
    (125.0, 90.0),
    (124.0, 100.0),
    (123.0, 135.0),
    (120.0, 180.0),
    (122.0, 207.0),
    (131.0, 235.0),
    (146.0, 262.0),
    (165.0, 280.0),
    (192.0, 285.0),
    (220.0, 281.0),
    (260.0, 282.0),
    (290.0, 289.0),
    (313.0, 297.0),
    (352.0, 324.0),
    (390.0, 290.0),
    (427.0, 248.0),
    (470.0, 250.0),
    (900.0, 250.0),
    (900.0, 300.0),
    (466.0, 82.0),
    (442.0, 60.0),
    (418.0, 40.0),
    (393.0, 22.0),
    (365.0, 9.0),
    (350.0, 2.0),
    (350.0, -900.0),
    (190.0, -900.0),
];
/// The map's scale: picture heights per degree off straight down (the real pod's dot: ahead
/// 33 degrees down a quarter height up the picture).
const GIMBAL_MAP: f32 = 0.0047;

/// Where the pod looking at azimuth `az` (degrees off the nose, right positive) and elevation
/// `el` (degrees, up positive) is on the field of regard map, in `LIMIT_FRAME`'s pixels.
fn gimbal_point(az: f32, el: f32) -> Vec2 {
    let r = GIMBAL_MAP * (90.0 + el).max(0.0) * LIMIT_FRAME.1;
    let a = az.to_radians();
    Vec2::new(LIMIT_FRAME.0 / 2.0, LIMIT_FRAME.1 / 2.0) + Vec2::new(a.sin(), -a.cos()) * r
}

/// The pod looking at azimuth `az` and elevation `el` (degrees) is masked by the airframe or
/// past its gimbals: off the region the limit lines bound.
pub fn occluded(az: f32, el: f32) -> bool {
    let p = gimbal_point(az, el);
    let mut inside = false;
    for k in 0..SEEN.len() {
        let (a, b) = (SEEN[k], SEEN[(k + 1) % SEEN.len()]);
        if (a.1 > p.y) != (b.1 > p.y) && p.x < a.0 + (p.y - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            inside = !inside;
        }
    }
    !inside
}

/// The field of regard map on a canvas: `LIMIT_FRAME`'s pixels scaled by the canvas height
/// round its middle (a square canvas shows its middle).
fn limit_point(c: &Canvas, (x, y): (f32, f32)) -> Vec2 {
    let h = c.h as f32;
    Vec2::new(c.w as f32 / 2.0, h / 2.0)
        + Vec2::new(x - LIMIT_FRAME.0 / 2.0, y - LIMIT_FRAME.1 / 2.0) * (h / LIMIT_FRAME.1)
}

/// The limit lines and the gimbal dot where the pod looks.
fn field_of_regard(c: &mut Canvas, az: f32, el: f32) {
    let w = 1.2 * c.h as f32 / 360.0;
    for &(dotted, pts) in MASK_LINE {
        for (k, s) in pts.windows(2).enumerate() {
            let (a, b) = (limit_point(c, s[0]), limit_point(c, s[1]));
            if dotted {
                c.dashed(a, b, 2.0 * w, 2.0 * w, w, ink(BRIGHT));
            } else if k < pts.len() {
                c.stroke(a, b, w, ink(BRIGHT));
            }
        }
    }
    let p = gimbal_point(az, el);
    let p = limit_point(c, (p.x, p.y));
    let s = 2.5 * w;
    c.fill_rect(
        (p.x - s) as i32,
        (p.y - s * 0.7) as i32,
        (2.0 * s) as i32,
        (1.4 * s) as i32,
        ink(BRIGHT),
    );
}

/// Degrees as the pod writes them: degrees and minutes with hundredths.
fn dm(v: f64, pos: char, neg: char) -> String {
    let h = if v >= 0.0 { pos } else { neg };
    let a = v.abs();
    let d = a.floor();
    let m = (a - d) * 60.0;
    format!("{h} {d:.0}:{m:05.2}")
}

/// The LANTIRN's symbology (the pod's picture shows through index 0 under it): the coordinates
/// and own data top left, the heading top middle, the polarity and the field of view left, the
/// crosshair and its corner marks, the slant range, the pod's angles, the clock and the laser
/// along the foot.
pub fn draw_lantirn(c: &mut Canvas, fonts: &Fonts, d: &Lantirn, t: f32) {
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    // The pod's own low-resolution characters: the flight font's pixels this many units big.
    let px = ((h * 0.036 / 5.0).round() as i32).max(1);
    let th = (5 * px) as f32;
    let line = (8 * px) as f32;
    let label = |c: &mut Canvas, p: Vec2, s: &str, scale: i32, align: Align| {
        let s = crate::font::upper(s);
        let width = fonts.width(&s) * scale;
        let x = match align {
            Align::Left => p.x as i32,
            Align::Centre => p.x as i32 - width / 2,
            Align::Right => p.x as i32 - width,
        };
        fonts.text_scaled(c, x, p.y as i32 - fonts.height() * scale / 2, &s, ink(BRIGHT), scale);
    };
    let x = 0.08 * w;
    let lines = [
        dm(d.lat, 'N', 'S'),
        dm(d.lon, 'E', 'W'),
        format!("ALT {:5.0}", d.alt_ft.max(0.0)),
        format!("KGS {:04.0}", d.ground_kt.max(0.0)),
        format!("DIVE {:+.0}", d.dive),
    ];
    for (k, s) in lines.iter().enumerate() {
        label(c, Vec2::new(x, 0.08 * h + k as f32 * line), s, px, Align::Left);
    }
    label(
        c,
        Vec2::new(0.62 * w, 0.08 * h),
        &format!("{:03.0}", d.heading.rem_euclid(360.0)),
        px,
        Align::Centre,
    );
    let sensor = match (d.flir, d.white_hot) {
        (false, _) => "TV",
        (true, true) => "WHOT",
        (true, false) => "BHOT",
    };
    label(c, Vec2::new(x, 0.40 * h), sensor, px, Align::Left);
    label(
        c,
        Vec2::new(x, 0.40 * h + line),
        if d.wide { "PIC W" } else { "PIC N" },
        px,
        Align::Left,
    );
    label(c, Vec2::new(x, 0.52 * h), &d.track, px, Align::Left);
    // The crosshair and the corner marks of the track gate.
    let mid = Vec2::new(w / 2.0, h / 2.0);
    let a = 0.11 * h;
    let gap = 0.015 * h;
    for dir in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
        c.stroke(mid + dir * gap, mid + dir * a, 2.0, ink(BRIGHT));
    }
    let g = 0.2 * h;
    for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let corner = mid + Vec2::new(sx * g * 1.2, sy * g * 0.8);
        c.stroke(corner, corner - Vec2::new(sx * 0.03 * h, 0.0), 2.0, ink(BRIGHT));
        c.stroke(corner, corner - Vec2::new(0.0, sy * 0.03 * h), 2.0, ink(BRIGHT));
    }
    let foot = h - 4.5 * line - th;
    let range =
        if d.range_nm > 0.0 { format!("SRA {:4.1}", d.range_nm) } else { "SRA XX.X".to_string() };
    label(c, Vec2::new(x, foot), &range, px, Align::Left);
    let az = format!("AZ {:03.0} {}", d.az.abs(), if d.az >= 0.0 { 'R' } else { 'L' });
    label(c, Vec2::new(x, foot + line), &az, px, Align::Left);
    label(c, Vec2::new(x, foot + 2.0 * line), &format!("EL {:03.0}", d.el), px, Align::Left);
    let secs = d.clock.max(0.0) as u32;
    let clock = format!("{:02}:{:02}:{:02}", secs / 3600 % 24, secs / 60 % 60, secs % 60);
    label(c, Vec2::new(x, foot + 3.0 * line), &clock, px, Align::Left);
    label(
        c,
        Vec2::new(w * 0.5, foot + 3.0 * line),
        if d.air { "A/A" } else { "A/G" },
        px,
        Align::Centre,
    );
    let laser =
        format!("{}{}", if d.lasing && (t * 2.0).fract() < 0.5 { "L" } else { "M" }, d.laser_code);
    label(c, Vec2::new(w * 0.92, foot + 3.0 * line), &laser, px, Align::Right);
    field_of_regard(c, d.az, d.el);
    // The pod looks into the airframe, or past its gimbals.
    if occluded(d.az, d.el) && (t * 2.0).fract() < 0.6 {
        label(c, Vec2::new(w / 2.0, 0.32 * h), "MASK", px + 1, Align::Centre);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awg9::Awg9;

    fn lit(c: &Canvas) -> usize {
        (0..c.h as i32)
            .flat_map(|y| (0..c.w as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| c.get(x, y) != 0)
            .count()
    }

    /// A closing track's launch times run down together a second a second; one that draws
    /// away has none. A missile with its seeker on is active.
    #[test]
    fn launch_times_run_down() {
        let far = launch_times(150_000.0, 500.0).unwrap();
        let near = launch_times(150_000.0 - 5_000.0, 500.0).unwrap();
        for k in 0..3 {
            assert!((far[k] - near[k] - 10.0).abs() < 1e-3);
        }
        assert!(far[0] > far[1] && far[1] > far[2]);
        assert!(launch_times(50_000.0, -100.0).is_none());
        let m = InFlight { target: ContactRef::Unit(1), to_go: 12.0, tti: 12.0 };
        assert!(m.active());
        assert!(!InFlight { to_go: -3.0, ..m }.active());
    }

    /// Every display draws on its canvas without panicking, standby or idle too.
    #[test]
    fn the_displays_draw() {
        let r = Awg9::default();
        let ctl = Awg9Controls::default();
        let own = Own {
            pos: DVec3::new(0.0, 0.0, 9000.0),
            heading: 0.3,
            pitch: 0.1,
            roll: -0.4,
            knots: 400.0,
        };
        let mut c = Canvas::scaled(320, 320, 1);
        draw_ddd(&mut c, &r, &ctl, &DddView { cursor: Some(Vec2::new(0.1, 0.4)), ..default() });
        assert!(lit(&c) > 0);
        let wps = [DVec2::new(10_000.0, 40_000.0), DVec2::new(30_000.0, 80_000.0)];
        for stab in [TidStab::Ground, TidStab::Aircraft, TidStab::Attack] {
            let v = TidView {
                radar: &r,
                controls: &ctl,
                own,
                range_nm: 50.0,
                stab,
                waypoints: &wps,
                steer: Some(0),
                in_flight: &[],
                vel_vectors: true,
                alt_numbers: true,
                cursor: None,
                t: 0.0,
            };
            draw_tid(&mut c, &v);
            assert!(lit(&c) > 0);
        }
        draw_hsd_nav(&mut c, &own, Some(wps[0]), "WP 1");
        let th = [Threat {
            rel_bearing: 0.5,
            label: "29".into(),
            air: true,
            rank: 0,
            tracking: true,
            launch: false,
        }];
        draw_ecmd(&mut c, &th, true, 0.2);
        draw_lantirn(
            &mut c,
            &Fonts::builtin(),
            &Lantirn { lat: 42.3, lon: 39.5, laser_code: 1688, ..default() },
            0.0,
        );
        assert!(lit(&c) > 0);
        assert_eq!(dm(42.3, 'N', 'S'), "N 42:18.00");
    }

    /// Each DDD picture draws its own; the raw video of a mode without ranges shows no returns;
    /// the infrared picture needs no radar.
    #[test]
    fn the_ddd_pictures_differ() {
        let ctl = Awg9Controls { mode: crate::awg9::Mode::Rws, ..default() };
        let mut r = Awg9::default();
        r.mode = crate::awg9::Mode::Rws;
        r.elevation = -0.1;
        r.hits = vec![crate::awg9::Hit {
            az: 0.3,
            el: 0.2,
            range: ctl.range * 0.25,
            closure: 400.0,
            target: ContactRef::Unit(1),
            friendly: None,
            age: 0.0,
        }];
        r.strobes =
            vec![crate::aa_radar::Strobe { az: -0.4, target: ContactRef::Unit(2), age: 0.0 }];
        r.ir = vec![crate::awg9::IrSpot { az: -0.2, el: 0.1, target: ContactRef::Unit(3) }];
        let draw = |r: &Awg9, picture| {
            let mut c = Canvas::scaled(240, 240, 1);
            draw_ddd(&mut c, r, &ctl, &DddView { picture, ..default() });
            c.px
        };
        use DddPicture::*;
        let all = [Pulse, Pd, Antenna, Rate, Jam, Ir].map(|p| draw(&r, p));
        for (i, a) in all.iter().enumerate() {
            assert!(all[..i].iter().all(|b| a != b), "picture {i}");
        }
        // The return's place by the picture: range, elevation, closing speed.
        let v = |picture| DddView { picture, ..default() };
        let h = r.hits[0];
        let spot = |p| ddd_spot(&r, &ctl, &v(p), (h.az, h.el), h.range, h.closure).y;
        assert!((spot(Pd) - 0.25).abs() < 1e-3);
        assert!((spot(Antenna) - (0.5 + 0.2 / EL_SPAN * 0.5)).abs() < 1e-3);
        assert!((spot(Rate) - (400.0 * KT + 200.0) / (VC_TOP + 200.0)).abs() < 1e-3);
        // Standby: the infrared picture still shows its spot, the others say STBY.
        let mut off = Awg9::default();
        off.ir = r.ir.clone();
        assert_ne!(draw(&off, Ir), draw(&Awg9::default(), Ir));
        assert_eq!(draw(&off, Pd), draw(&Awg9::default(), Pd));
    }

    /// The field of regard map: ahead and down is seen, the left beyond the fuselage, back under
    /// the tail and high up to the right past the gimbals are not; the real pod's dot ahead 33
    /// degrees down sits a quarter of the picture up.
    #[test]
    fn the_pod_sees_within_its_limits() {
        assert!(!occluded(0.0, -33.0) && !occluded(0.0, -70.0) && !occluded(126.0, -52.0));
        assert!(occluded(-90.0, -10.0) && occluded(180.0, -10.0) && occluded(45.0, 70.0));
        let p = gimbal_point(0.0, -33.0);
        assert!((p - Vec2::new(240.0, 88.0)).length() < 5.0, "{p}");
        // The page says MASK, flashing.
        let mut c = Canvas::scaled(160, 160, 1);
        let fonts = Fonts::builtin();
        draw_lantirn(&mut c, &fonts, &Lantirn { az: -90.0, el: -10.0, ..default() }, 0.0);
        let masked = lit(&c);
        draw_lantirn(&mut c, &fonts, &Lantirn { az: 0.0, el: -60.0, ..default() }, 0.0);
        assert!(masked > lit(&c));
    }

    /// The TID puts a world point where it should: ground stabilised, north up from the middle.
    #[test]
    fn the_tid_maps_points() {
        let c = Canvas::scaled(200, 200, 1);
        let own = Own { pos: DVec3::ZERO, heading: 1.0, ..default() };
        let t = Tube::new(&c, TidStab::Ground, &own, 10.0);
        let p = t.at(DVec2::new(0.0, 10.0 * NM / 2.0));
        assert!((p - Vec2::new(100.0, 100.0 - t.rad / 2.0)).length() < 0.5, "{p}");
        // Aircraft stabilised: the nose (east here, heading 90) up.
        let own = Own { heading: std::f32::consts::FRAC_PI_2, ..own };
        let t = Tube::new(&c, TidStab::Aircraft, &own, 10.0);
        let p = t.at(DVec2::new(10.0 * NM / 2.0, 0.0));
        assert!((p - Vec2::new(100.0, 100.0 - t.rad / 2.0)).length() < 0.5, "{p}");
    }
}

#[cfg(test)]
mod preview {
    use super::*;
    use crate::aa_radar::Target;

    /// Preview aid: `SM89_A6_PNG=<dir> cargo test --lib f14_scopes::preview -- --ignored`.
    #[test]
    #[ignore]
    fn write_previews() {
        let mut r = Awg9::default();
        let ctl = Awg9Controls::default();
        let pi = std::f32::consts::PI;
        let mut t: Vec<Target> = [
            (1, -15_000.0, 90_000.0),
            (2, 0.0, 70_000.0),
            (3, 12_000.0, 80_000.0),
            (4, 30_000.0, 50_000.0),
        ]
        .iter()
        .map(|&(i, e, n)| Target {
            target: ContactRef::Unit(i),
            pos: DVec3::new(e, n, 8000.0),
            heading: pi + 0.2,
            friendly: i == 4,
            detect: 150_000.0,
            jam: false,
        })
        .collect();
        let mut own = DVec3::new(0.0, 0.0, 9000.0);
        for _ in 0..(4.0 * ctl.scan().frame() * 30.0) as usize {
            own.y += 250.0 / 30.0;
            for x in t.iter_mut() {
                let (s, c) = x.heading.sin_cos();
                x.pos += DVec3::new(s as f64, c as f64, 0.0) * 230.0 / 30.0;
            }
            r.step(1.0 / 30.0, own, 0.0, &ctl, &t, None, |_, _| true);
        }
        // The IFF pushed: the friend among them shows.
        r.interrogate();
        r.step(1.0 / 30.0, own, 0.0, &ctl, &t, None, |_, _| true);
        r.hooked = r.tracks.first().map(|t| t.number);
        let pal = crate::intruder::phosphor([2, 10, 4], [80, 255, 110]);
        let own = Own { pos: own, heading: 0.0, pitch: 0.05, roll: -0.35, knots: 450.0 };
        let mut c = Canvas::scaled(320, 320, 2);
        draw_ddd(&mut c, &r, &ctl, &DddView { cursor: Some(Vec2::new(0.1, 0.4)), ..default() });
        crate::intruder::dump_png(&c, &crate::tomcat::displays::ddd_colours(), "f14_ddd");
        let wps = [DVec2::new(10_000.0, 40_000.0), DVec2::new(-20_000.0, 100_000.0)];
        let inf = [InFlight { target: ContactRef::Unit(2), to_go: -23.0, tti: 41.0 }];
        draw_tid(
            &mut c,
            &TidView {
                radar: &r,
                controls: &ctl,
                own,
                range_nm: 100.0,
                stab: TidStab::Attack,
                waypoints: &wps,
                steer: Some(0),
                in_flight: &inf,
                vel_vectors: true,
                alt_numbers: true,
                cursor: Some(Vec2::new(0.2, 0.3)),
                t: 0.0,
            },
        );
        crate::intruder::dump_png(&c, &pal, "f14_tid");
        draw_hsd_nav(&mut c, &own, Some(wps[0]), "WP 1");
        crate::intruder::dump_png(&c, &pal, "f14_hsd");
        let th = [
            Threat {
                rel_bearing: 0.5,
                label: "29".into(),
                air: true,
                rank: 0,
                tracking: true,
                launch: false,
            },
            Threat {
                rel_bearing: -2.0,
                label: "10".into(),
                air: false,
                rank: 1,
                tracking: false,
                launch: false,
            },
        ];
        draw_ecmd(&mut c, &th, true, 0.2);
        crate::intruder::dump_png(&c, &pal, "f14_ecmd");
        draw_lantirn(
            &mut c,
            &Fonts::builtin(),
            &Lantirn {
                lat: 42.3034,
                lon: 39.4892,
                alt_ft: 48.0,
                ground_kt: 178.0,
                dive: 5.0,
                heading: 342.0,
                clock: 86075.0,
                laser_code: 1688,
                white_hot: false,
                flir: true,
                wide: true,
                track: "MGC".into(),
                range_nm: 0.0,
                az: 0.0,
                el: -33.0,
                ..default()
            },
            0.0,
        );
        crate::intruder::dump_png(&c, &pal, "f14_lantirn");
        for (name, az, el) in [
            ("f14_lantirn_ahead", 1.0f32, 0.0f32),
            ("f14_lantirn_aft", 126.0, -52.0),
            ("f14_lantirn_left", -100.0, -40.0),
        ] {
            draw_lantirn(
                &mut c,
                &Fonts::builtin(),
                &Lantirn { az, el, laser_code: 1688, track: "AREA".into(), ..default() },
                0.0,
            );
            crate::intruder::dump_png(&c, &pal, name);
        }
    }
}
