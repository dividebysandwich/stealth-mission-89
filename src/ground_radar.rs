//! The air-to-ground radar's ground map (GM, `avionics::Format::Gm`): a beam sweeping left,
//! right and left again across `SCAN_HALF` either side of the nose paints the ground and the
//! ground targets it sees into a ground-stabilised picture, kept until the beam passes again.
//!
//! - Each ray marches out in range bins over the hills and the mountains. Ground hidden behind
//!   nearer ground (the steepest sight line so far) lies in the radar shadow and returns
//!   nothing.
//! - The ground's backscatter goes by the angle the beam meets it at (slopes facing the radar
//!   bright, those turned away dim) and what covers it: water mirrors the beam away, concrete
//!   is dark, woods and above all buildings bright.
//! - Speckle (a few looks of Rayleigh fading, the same until the next pass), a noise floor and
//!   the beam's width (a point smeared across it, wider with the range) make the picture.
//! - Ground targets (sites, ships, vehicles and the theater's buildings) the beam passes and
//!   sees return brightly and are listed (`GroundRadar::returns`) where they were painted: a
//!   moving vehicle's return jumps from one pass to the next.
//!
//! The radar maps while its format is up and not in standby (`AvionicsState::
//! gm_on`); the simulation hears it (`PlayerState::radar`).

use std::collections::VecDeque;

use bevy::math::DVec2;
use bevy::prelude::*;
use stealth_sim::oflags;

use crate::avionics::HsdView;
use crate::canvas::Picture;
use crate::flight::{Aircraft, Player};
use crate::mfd::MfdState;
use crate::world::{WU, WorldPos};

pub struct GroundRadarPlugin;

impl Plugin for GroundRadarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundRadar>()
            .add_systems(OnEnter(crate::settings::AppState::Flight), reset)
            .add_systems(
                Update,
                sweep
                    .after(crate::combat::CombatSystems)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            );
    }
}

/// The scan's half width either side of the nose (radians) and its rate (radians per second).
pub const SCAN_HALF: f32 = 60.0 * std::f32::consts::PI / 180.0;
const SCAN_RATE: f32 = 60.0 * std::f32::consts::PI / 180.0;
/// The beam's width (radians): a point return is smeared across it.
const BEAM: f32 = 2.5 * std::f32::consts::PI / 180.0;
/// Range bins along a ray.
const BINS: usize = 256;
/// Picture cells along a side, and cells per range (the picture reaches 1.25 ranges round its
/// centre, so the aircraft flies a quarter range before it is moved).
const GRID: usize = 320;
const CELLS_PER_RANGE: f64 = 128.0;
/// Rays per cell at the full range: the rays leave no cell out.
const RAY_SPACING: f32 = 0.6;
/// Most rays a frame (a long frame's sweep is cut short).
const MAX_RAYS: usize = 64;
/// The display's dynamic range: returns (`backscatter`'s units) from the dimmest
/// shown to the brightest.
const P_MIN: f32 = 0.003;
const P_MAX: f32 = 40.0;
/// The receiver noise (mean, the same units).
const NOISE: f32 = 0.002;
/// How far the gain makes up for the shallower angle the ground is seen at with the range:
/// fully at 1, not at all at 0.
const STC: f32 = 0.6;
/// The flattest the beam is taken to meet the ground (the sine of the grazing angle).
const MIN_GRAZING: f32 = 0.01;

/// A ground target the radar painted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Return {
    /// Where it was painted (metres east, north).
    pub east: f64,
    pub north: f64,
    pub target: RadarTarget,
    /// The pass that painted it.
    sweep: u32,
}

impl Return {
    #[cfg(test)]
    pub fn at(east: f64, north: f64, target: RadarTarget) -> Self {
        Self { east, north, target, sweep: 0 }
    }
}

/// What a return is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadarTarget {
    /// A world object (site, ship, base or landmark): `Sim::objects` index.
    Object(usize),
    /// A ground unit (air defence gun or vehicle): `Sim::units` index.
    Unit(usize),
    /// A theater tile object at this map point (x east, y south).
    Tile(i32, i32),
}

/// A ground target the beam may paint.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    pub east: f64,
    pub north: f64,
    /// Height of its top over the ground (metres).
    pub height: f32,
    /// Its return against open land's square on.
    pub rcs: f32,
    pub target: RadarTarget,
}

/// The ground the radar maps: heights (metres) and cover at points (metres east, north).
pub trait Ground {
    fn height(&self, e: f64, n: f64) -> f32;
    /// What covers the ground at a point, over a resolution cell `r` metres across.
    fn cover(&self, e: f64, n: f64, r: f64) -> crate::scenery::Cover;
}

/// One ray: its bearing and origin, the ground's return per range bin and the targets'.
struct Ray {
    bearing: f32,
    origin: DVec2,
    ground: Vec<f32>,
    targets: Vec<f32>,
}

#[derive(Resource)]
pub struct GroundRadar {
    /// Transmitting.
    pub on: bool,
    /// The beam's azimuth from the nose (radians, right positive) and the way it moves (+1 right).
    pub beam: f32,
    dir: f32,
    /// The next ray's azimuth from the nose.
    next_ray: f32,
    /// Passes so far (a pass is one way across).
    sweeps: u32,
    /// The range (metres) the picture is made for.
    range: f64,
    /// The picture: brightness per cell (0 never painted), north-up, row 0 south, round
    /// `anchor` (metres east, north) in cells `cell` metres across.
    grid: Vec<u8>,
    anchor: DVec2,
    cell: f64,
    /// The last rays (a beam's width of them).
    rays: VecDeque<Ray>,
    pub returns: Vec<Return>,
    /// The theater's tile objects (metres east, north, and their map point), for the theater.
    tiles: Vec<(DVec2, (i32, i32))>,
    tiles_theater: Option<String>,
}

impl Default for GroundRadar {
    fn default() -> Self {
        Self {
            on: false,
            beam: -SCAN_HALF,
            dir: 1.0,
            next_ray: -SCAN_HALF,
            sweeps: 0,
            range: 0.0,
            grid: vec![0; GRID * GRID],
            anchor: DVec2::ZERO,
            cell: 1.0,
            rays: VecDeque::new(),
            returns: Vec::new(),
            tiles: Vec::new(),
            tiles_theater: None,
        }
    }
}

fn reset(mut radar: ResMut<GroundRadar>) {
    *radar = default();
}

impl GroundRadar {
    /// Drops the picture and the returns (off, or a new range).
    fn clear(&mut self) {
        self.grid.fill(0);
        self.rays.clear();
        self.returns.clear();
        self.range = 0.0;
    }

    /// The angle between rays.
    fn ray_step() -> f32 {
        RAY_SPACING / CELLS_PER_RANGE as f32
    }

    /// Rays per beam width.
    fn beam_rays() -> usize {
        ((BEAM / Self::ray_step()).round() as usize).max(1)
    }

    /// Readies the picture for `range` metres round the aircraft at `own`: cleared at a new
    /// range, moved along (keeping what it shows) once the aircraft has flown a quarter range.
    fn place(&mut self, own: DVec2, range: f64) {
        if range != self.range {
            self.clear();
            self.range = range;
            self.cell = range / CELLS_PER_RANGE;
            self.anchor = own;
            return;
        }
        if own.distance(self.anchor) < range * 0.25 {
            return;
        }
        let shift = ((own - self.anchor) / self.cell).round();
        let (dx, dy) = (shift.x as i64, shift.y as i64);
        let mut grid = vec![0; GRID * GRID];
        for y in 0..GRID as i64 {
            for x in 0..GRID as i64 {
                let (sx, sy) = (x + dx, y + dy);
                if (0..GRID as i64).contains(&sx) && (0..GRID as i64).contains(&sy) {
                    grid[(y as usize) * GRID + x as usize] =
                        self.grid[sy as usize * GRID + sx as usize];
                }
            }
        }
        self.grid = grid;
        self.anchor += shift * self.cell;
    }

    /// The picture's cell at a point (metres east, north), if it is on the picture.
    fn cell_at(&self, e: f64, n: f64) -> Option<usize> {
        let x = ((e - self.anchor.x) / self.cell).floor() + (GRID / 2) as f64;
        let y = ((n - self.anchor.y) / self.cell).floor() + (GRID / 2) as f64;
        let inside = (0.0..GRID as f64).contains(&x) && (0.0..GRID as f64).contains(&y);
        inside.then(|| y as usize * GRID + x as usize)
    }

    /// The picture's brightness at a point (metres east, north): 0 dark, 1 the brightest.
    pub fn brightness(&self, e: f64, n: f64) -> f32 {
        self.cell_at(e, n).map_or(0.0, |i| self.grid[i] as f32 / 255.0)
    }

    /// One frame of the scan: the beam moves on `dt` seconds' worth, painting the rays it
    /// passes, from the aircraft at `own` (metres east, north), `alt` metres up, heading
    /// `heading` (radians clockwise from north).
    #[allow(clippy::too_many_arguments)]
    pub fn scan(
        &mut self,
        dt: f32,
        own: DVec2,
        alt: f32,
        heading: f32,
        range: f64,
        ground: &impl Ground,
        candidates: &[Candidate],
    ) {
        self.on = true;
        self.place(own, range);
        let step = Self::ray_step();
        let mut to = self.beam + self.dir * SCAN_RATE * dt;
        let mut rays = 0;
        loop {
            // Rays up to where the beam is now, or the scan's edge.
            let edge = SCAN_HALF * self.dir;
            let end = if (to - edge) * self.dir > 0.0 { edge } else { to };
            while (end - self.next_ray) * self.dir >= 0.0 && rays < MAX_RAYS {
                let az = self.next_ray;
                self.ray(own, alt, heading + az, range, ground, candidates);
                self.next_ray += step * self.dir;
                rays += 1;
            }
            if end == to || rays >= MAX_RAYS {
                break;
            }
            // Back the other way from the edge: a new pass.
            self.dir = -self.dir;
            self.sweeps = self.sweeps.wrapping_add(1);
            self.rays.clear();
            self.next_ray = edge + step * self.dir;
            to = edge + (edge - to);
        }
        self.beam = to.clamp(-SCAN_HALF, SCAN_HALF);
        // Returns not seen for two passes are gone (the aircraft turned away from them).
        let sweeps = self.sweeps;
        self.returns.retain(|r| sweeps.wrapping_sub(r.sweep) <= 2);
    }

    /// Paints one ray at `bearing` (radians clockwise from north).
    fn ray(
        &mut self,
        own: DVec2,
        alt: f32,
        bearing: f32,
        range: f64,
        ground: &impl Ground,
        candidates: &[Candidate],
    ) {
        let dir = DVec2::new(bearing.sin() as f64, bearing.cos() as f64);
        let bin = range / BINS as f64;
        let res = bin.max(self.cell);
        let mut out =
            Ray { bearing, origin: own, ground: vec![0.0; BINS], targets: vec![0.0; BINS] };
        // The steepest sight line down to the ground so far, before each bin.
        let mut sight = f32::NEG_INFINITY;
        let mut before = [f32::NEG_INFINITY; BINS];
        for (b, slot) in out.ground.iter_mut().enumerate() {
            let d = (b as f64 + 0.5) * bin;
            let p = own + dir * d;
            let h = ground.height(p.x, p.y);
            let angle = (h - alt).atan2(d as f32);
            before[b] = sight;
            let shadow = angle < sight;
            sight = sight.max(angle);
            if shadow {
                continue;
            }
            // The local surface's normal against the way to the radar.
            let s = res * 0.5;
            let de = (ground.height(p.x + s, p.y) - ground.height(p.x - s, p.y)) / (2.0 * s) as f32;
            let dn = (ground.height(p.x, p.y + s) - ground.height(p.x, p.y - s)) / (2.0 * s) as f32;
            let normal = Vec3::new(-de, -dn, 1.0).normalize();
            let to_radar = Vec3::new(-dir.x as f32 * d as f32, -dir.y as f32 * d as f32, alt - h)
                .normalize_or(Vec3::Z);
            let incidence = normal.dot(to_radar).max(0.0);
            let grazing = to_radar.z.max(MIN_GRAZING);
            *slot = backscatter(ground.cover(p.x, p.y, res), incidence) / grazing.powf(STC);
        }
        // The targets in the ray's slice that the ground leaves in sight.
        let step = Self::ray_step();
        let mut seen = Vec::new();
        for c in candidates {
            let v = DVec2::new(c.east, c.north) - own;
            let d = v.length();
            if d >= range || d < bin {
                continue;
            }
            let az = angle_diff(v.x.atan2(v.y) as f32, bearing);
            if az.abs() > step * 0.5 {
                continue;
            }
            let b = ((d / bin) as usize).min(BINS - 1);
            let h = ground.height(c.east, c.north) + c.height;
            if (h - alt).atan2(d as f32) < before[b] {
                continue;
            }
            // Glint: a target's return flickers from pass to pass.
            let glint = 0.4 + 1.2 * unit(hash(b as u32, seen.len() as u32, self.sweeps));
            out.targets[b] += c.rcs * glint;
            seen.push(c.target);
            let painted =
                Return { east: c.east, north: c.north, target: c.target, sweep: self.sweeps };
            match self.returns.iter_mut().find(|r| r.target == c.target) {
                Some(r) => *r = painted,
                None => self.returns.push(painted),
            }
        }
        // Returns in the slice not painted again are hidden or gone.
        self.returns.retain(|r| {
            let v = DVec2::new(r.east, r.north) - own;
            let az = angle_diff(v.x.atan2(v.y) as f32, bearing);
            az.abs() > step * 0.5 || v.length() >= range || seen.contains(&r.target)
        });
        self.rays.push_back(out);
        if self.rays.len() >= Self::beam_rays() {
            self.paint();
            self.rays.pop_front();
        }
    }

    /// Paints the ray in the middle of the last beam width of them: the ground's return
    /// averaged across the beam, a target's as strong anywhere in it (the beam's width smears
    /// it), then the speckle, the noise and the display's log scale.
    fn paint(&mut self) {
        let n = self.rays.len();
        let mid = &self.rays[n / 2];
        let (bearing, origin) = (mid.bearing, mid.origin);
        let dir = DVec2::new(bearing.sin() as f64, bearing.cos() as f64);
        let bin = self.range / BINS as f64;
        let (lo, hi) = (P_MIN.ln(), P_MAX.ln());
        for b in 0..BINS {
            let mut ground = 0.0;
            let mut target = 0.0f32;
            for (k, r) in self.rays.iter().enumerate() {
                // The beam's shape: full in the middle, half at its edges.
                let off = (k as f32 - (n - 1) as f32 * 0.5).abs() / (n as f32 * 0.5).max(1.0);
                let w = 1.0 - 0.5 * off * off;
                ground += r.ground[b];
                target = target.max(r.targets[b] * w);
            }
            ground /= n as f32;
            let p = origin + dir * ((b as f64 + 0.5) * bin);
            let Some(i) = self.cell_at(p.x, p.y) else { continue };
            // Speckle and noise by the ground cell, the same until the next pass.
            let (cx, cy) = ((p.x / self.cell).floor() as i64, (p.y / self.cell).floor() as i64);
            let key =
                |k: u32| hash(cx as u32, cy as u32 ^ k.wrapping_mul(0x9E37_79B9), self.sweeps);
            let speckle = -(1..=4).map(|k| unit(key(k)).ln()).sum::<f32>() / 4.0;
            let noise = -unit(key(5)).ln() * NOISE;
            let power = (ground + target) * speckle + noise;
            let v = ((power.max(1e-9).ln() - lo) / (hi - lo)).clamp(0.0, 1.0);
            self.grid[i] = 1 + (v * 254.0) as u8;
        }
    }

    /// Draws the picture through `view` (display pixels, `scale` texels each) into `pic`, inside
    /// the scope's wedge (`SCAN_HALF` either side of up from `view.origin`, out to
    /// `view.radius`); black outside it.
    pub fn draw(&self, view: &HsdView, scale: f32, pic: &mut Picture) {
        self.draw_in(view, scale, pic, (0, 0, pic.w, pic.h));
    }

    /// [`Self::draw`] into the texels `x0..x1`, `y0..y1` of `pic` (`rect`) only: the zoomed
    /// window's picture over the ground map's.
    pub fn draw_in(
        &self,
        view: &HsdView,
        scale: f32,
        pic: &mut Picture,
        rect: (usize, usize, usize, usize),
    ) {
        let (w, h) = (pic.w, pic.h);
        let (rx0, ry0) = (rect.0.min(w), rect.1.min(h));
        let (rx1, ry1) = (rect.2.clamp(rx0, w), rect.3.clamp(ry0, h));
        for y in ry0..ry1 {
            for px in pic.rgba[(y * w + rx0) * 4..(y * w + rx1) * 4].as_chunks_mut::<4>().0 {
                *px = [0, 0, 0, 255];
            }
        }
        if !self.on {
            return;
        }
        let ramp = &*RAMP;
        let (r2, tan) = ((view.radius * scale).powi(2), SCAN_HALF.tan());
        let o = view.origin * scale;
        // The picture is looked up once a display pixel (a cell is about one at every range),
        // the wedge's edges cut by the texel.
        let k = (scale.round() as usize).max(1);
        let cols = w.div_ceil(k);
        let mut row = vec![[0u8, 0, 0, 255]; cols];
        // A texel row's span inside the wedge (and the rectangle).
        let span = |y: usize| {
            let up = o.y - (y as f32 + 0.5);
            let half = if up > 0.0 { (up * tan).min((r2 - up * up).max(0.0).sqrt()) } else { 0.0 };
            let x0 = ((o.x - half).floor().max(0.0)) as usize;
            let x1 = ((o.x + half).ceil().min(w as f32)) as usize;
            (x0.max(rx0), x1.min(rx1))
        };
        for y in ry0..ry1 {
            let up = o.y - (y as f32 + 0.5);
            let (x0, x1) = span(y);
            if y % k == 0 || y == ry0 {
                // The display pixels' row, as wide as its widest texel row.
                let (b0, b1) = (y..((y / k + 1) * k).min(ry1))
                    .map(span)
                    .filter(|(a, b)| a < b)
                    .fold((usize::MAX, 0), |(a, b), (c, d)| (a.min(c), b.max(d)));
                if b0 < b1 {
                    // The world (metres) along the row, in steps of one pixel.
                    let py = (y / k) as f32 + 0.5;
                    let at = |c: usize| view.to_world(Vec2::new(c as f32 + 0.5, py)) * WU;
                    let (c0, c1) = (b0 / k, (b1 - 1) / k + 1);
                    let (start, step) = (at(c0), at(c0 + 1) - at(c0));
                    for (c, slot) in row.iter_mut().enumerate().take(c1).skip(c0) {
                        let g = start + step * (c - c0) as f64;
                        *slot = ramp[(self.brightness(g.x, g.y) * 255.0) as usize];
                    }
                }
            }
            for x in x0..x1 {
                let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - o;
                if d.length_squared() > r2 || d.x.abs() > up * tan {
                    continue;
                }
                let i = (y * w + x) * 4;
                pic.rgba[i..i + 4].copy_from_slice(&row[x / k]);
            }
        }
    }
}

/// Backscatter of the ground by its cover and the cosine of the angle the beam meets it at: open
/// land as a Lambert surface (with the square of the cosine, 4 met square on), so slopes facing
/// the radar stand out bright and those turned away dark.
fn backscatter(cover: crate::scenery::Cover, incidence: f32) -> f32 {
    use crate::scenery::Cover::*;
    match cover {
        // Mirrors the beam away but straight down; a little sea clutter.
        Water => 0.0005 + 0.3 * incidence.powi(8),
        Concrete => 0.05 * incidence,
        Open => incidence * incidence * 4.0,
        Forest => 7.0 * incidence * incidence,
        // Walls and roofs make corner reflectors whichever way the beam meets them.
        Built => 4.0 * (0.4 + 0.6 * incidence),
    }
}

static RAMP: std::sync::LazyLock<[[u8; 4]; 256]> = std::sync::LazyLock::new(ramp);

/// The display's colours: a green phosphor from black to near white.
fn ramp() -> [[u8; 4]; 256] {
    let mut out = [[0, 0, 0, 255]; 256];
    for (i, c) in out.iter_mut().enumerate() {
        // Black below a threshold (the noise floor's dimmest speckles), a gamma lifting the
        // mid tones.
        let v = ((i as f32 / 255.0 - 0.06) / 0.94).max(0.0);
        let g = v.powf(1.1);
        *c = [
            (255.0 * 0.55 * v.powf(2.2)) as u8,
            (255.0 * g) as u8,
            (255.0 * 0.45 * v.powf(1.8)) as u8,
            255,
        ];
    }
    out
}

/// The angle from `b` to `a` (radians), in -pi..pi.
pub(crate) fn angle_diff(a: f32, b: f32) -> f32 {
    (a - b + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

pub(crate) fn hash(a: u32, b: u32, c: u32) -> u32 {
    let mut h =
        a.wrapping_mul(0x85EB_CA6B) ^ b.wrapping_mul(0xC2B2_AE35) ^ c.wrapping_mul(0x27D4_EB2F);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

/// A hash as a number in (0, 1].
pub(crate) fn unit(h: u32) -> f32 {
    ((h >> 8) as f32 + 1.0) / (1 << 24) as f32
}

/// The theater's ground for the radar: the hills, the mountains and the scenery's cover.
pub(crate) struct TheaterGround<'a> {
    pub hills: &'a stealth_sim::hills::Hills,
    pub relief: &'a stealth_sim::relief::Relief,
    pub scenery: Option<&'a crate::scenery::Scenery>,
}

/// Metres per map unit, and the theater's side (metres).
const MAP_M: f64 = 15.625;
const THEATER_M: f64 = 0x8000 as f64 * MAP_M;

impl Ground for TheaterGround<'_> {
    fn height(&self, e: f64, n: f64) -> f32 {
        let hill = self.hills.ground_m(e, n);
        let mountain = self.relief.smooth_m(e / MAP_M, (THEATER_M - n) / MAP_M);
        hill.max(mountain)
    }

    fn cover(&self, e: f64, n: f64, r: f64) -> crate::scenery::Cover {
        self.scenery.map_or(crate::scenery::Cover::Open, |s| s.radar_cover(DVec2::new(e, n), r))
    }
}

/// What a ground unit returns, by its kind.
fn unit_rcs(u: &stealth_sim::aaa::GroundUnit) -> (f32, f32) {
    let rcs = if u.kind.tank() {
        14.0
    } else if u.kind.armour() {
        11.0
    } else if u.kind.battery().is_some() {
        18.0
    } else {
        8.0
    };
    // A wreck returns less.
    if u.destroyed { (rcs * 0.4, 2.0) } else { (rcs, 3.0) }
}

/// The ground targets within `range` metres of `own` (metres east, north).
pub fn candidates(
    sim: &stealth_sim::Sim,
    tiles: &[(DVec2, (i32, i32))],
    own: DVec2,
    range: f64,
) -> Vec<Candidate> {
    let near = |e: f64, n: f64| DVec2::new(e, n).distance_squared(own) < range * range;
    let mut out = Vec::new();
    for (i, o) in sim.objects().iter().enumerate().skip(3) {
        if o.x == 0 && o.y == 0 {
            continue;
        }
        let (e, n) = (o.x as f64 * MAP_M, (0x8000 - o.y) as f64 * MAP_M);
        if !near(e, n) {
            continue;
        }
        let destroyed = o.flags & oflags::DESTROYED != 0;
        let (rcs, height) = if o.flags & oflags::SHIP != 0 {
            (120.0, 15.0)
        } else if o.flags & oflags::AIRFIELD != 0 {
            (30.0, 10.0)
        } else {
            (24.0, 8.0)
        };
        let rcs = if destroyed { rcs * 0.3 } else { rcs };
        out.push(Candidate { east: e, north: n, height, rcs, target: RadarTarget::Object(i) });
    }
    for (k, u) in sim.units().iter().enumerate() {
        if u.gone {
            continue;
        }
        let [e, n, _] = u.metres();
        if !near(e, n) {
            continue;
        }
        let (rcs, height) = unit_rcs(u);
        out.push(Candidate { east: e, north: n, height, rcs, target: RadarTarget::Unit(k) });
    }
    for &(p, (x, y)) in tiles {
        if near(p.x, p.y) {
            out.push(Candidate {
                east: p.x,
                north: p.y,
                height: 8.0,
                rcs: 16.0,
                target: RadarTarget::Tile(x, y),
            });
        }
    }
    out
}

/// The theater's tile objects (all levels): metres east, north and the map point.
fn tile_objects(theater: &crate::world::Theater) -> Vec<(DVec2, (i32, i32))> {
    use stealth_formats::terrain::{GRID_DIM, LEVEL_UNIT};
    let mut out = Vec::new();
    for lod in 1..=3usize {
        let unit = LEVEL_UNIT[lod] as i64;
        for row in 0..GRID_DIM[lod] {
            for col in 0..GRID_DIM[lod] {
                let t = theater.grid.tile(lod, col, row) as usize;
                let Some(objs) = theater.tiles.tiles.get(lod).and_then(|l| l.get(t)) else {
                    continue;
                };
                for o in objs {
                    let east = (col as i64 * 4096 + 2048 + o.x as i64) * unit;
                    let north = (row as i64 * 4096 + 2048 + o.y as i64) * unit;
                    let map = ((east >> 5) as i32, 0x8000 - (north >> 5) as i32);
                    out.push((DVec2::new(east as f64 * WU, north as f64 * WU), map));
                }
            }
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn sweep(
    time: Res<Time>,
    state: Res<MfdState>,
    mut radar: ResMut<GroundRadar>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    combat: Option<Res<crate::combat::Combat>>,
    theater: Option<Res<crate::world::Theater>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
) {
    let flying = aircraft.single().ok().filter(|(a, _)| !a.on_ground && a.ended.is_none());
    let (Some((a, pos)), Some(combat), true) = (flying, combat, state.modern && state.av.gm_on())
    else {
        if radar.on {
            radar.on = false;
            radar.clear();
        }
        return;
    };
    if let Some(t) = &theater
        && radar.tiles_theater.as_deref() != Some(t.id.as_str())
    {
        radar.tiles = tile_objects(t);
        radar.tiles_theater = Some(t.id.clone());
    }
    let own = DVec2::new(pos.0.x, -pos.0.z);
    let range = state.av.gm_range_km() as f64 * 1000.0;
    let hills = theater.as_ref().map(|t| t.hills.clone()).unwrap_or_default();
    let relief = combat.sim.relief();
    let ground = TheaterGround { hills: &hills, relief: &relief, scenery: scenery.as_deref() };
    let found = candidates(&combat.sim, &radar.tiles, own, range);
    let heading = a.euler().0;
    radar.scan(time.delta_secs(), own, pos.0.y as f32, heading, range, &ground, &found);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenery::Cover;

    /// Flat open land, with an optional ridge running east-west at `ridge` metres north
    /// (`ridge_h` high, 500 m wide), and water north of `water`.
    struct TestGround {
        ridge: Option<(f64, f32)>,
        water: Option<f64>,
    }

    impl Ground for TestGround {
        fn height(&self, _e: f64, n: f64) -> f32 {
            match self.ridge {
                Some((at, h)) if (n - at).abs() < 250.0 => h,
                _ => 0.0,
            }
        }
        fn cover(&self, _e: f64, n: f64, _r: f64) -> Cover {
            match self.water {
                Some(w) if n > w => Cover::Water,
                _ => Cover::Open,
            }
        }
    }

    const RANGE: f64 = 20_000.0;

    /// A full pass (left to right) north from the origin, 1000 m up.
    fn pass(radar: &mut GroundRadar, ground: &TestGround, targets: &[Candidate]) {
        for _ in 0..240 {
            radar.scan(1.0 / 60.0, DVec2::ZERO, 1000.0, 0.0, RANGE, ground, targets);
        }
    }

    fn mean(radar: &GroundRadar, n0: f64, n1: f64) -> f32 {
        let samples: Vec<f32> = (0..40)
            .flat_map(|i| (0..10).map(move |j| (i, j)))
            .map(|(i, j)| {
                let e = -2000.0 + i as f64 * 100.0;
                let n = n0 + (n1 - n0) * j as f64 / 10.0;
                radar.brightness(e, n)
            })
            .collect();
        samples.iter().sum::<f32>() / samples.len() as f32
    }

    #[test]
    fn sweep_turns_at_the_edges() {
        let mut radar = GroundRadar::default();
        let ground = TestGround { ridge: None, water: None };
        let mut dirs = vec![];
        for _ in 0..400 {
            radar.scan(1.0 / 60.0, DVec2::ZERO, 1000.0, 0.0, RANGE, &ground, &[]);
            assert!(radar.beam.abs() <= SCAN_HALF + 1e-4);
            if dirs.last() != Some(&radar.dir) {
                dirs.push(radar.dir);
            }
        }
        // Right, left, right, left over 6.7 s at 60 degrees a second across 120.
        assert!(dirs.len() >= 3, "{dirs:?}");
    }

    #[test]
    fn ridge_shadows_the_ground_behind() {
        let mut radar = GroundRadar::default();
        let ground = TestGround { ridge: Some((6000.0, 400.0)), water: None };
        pass(&mut radar, &ground, &[]);
        pass(&mut radar, &ground, &[]);
        let front = mean(&radar, 3000.0, 5500.0);
        let behind = mean(&radar, 7000.0, 9000.0);
        assert!(front > 0.3, "front {front}");
        assert!(behind < front * 0.5, "front {front} behind {behind}");
    }

    #[test]
    fn water_is_darker_than_land() {
        let mut radar = GroundRadar::default();
        let ground = TestGround { ridge: None, water: Some(8000.0) };
        pass(&mut radar, &ground, &[]);
        pass(&mut radar, &ground, &[]);
        let land = mean(&radar, 4000.0, 7000.0);
        let water = mean(&radar, 9000.0, 12000.0);
        assert!(water < land * 0.5, "land {land} water {water}");
    }

    fn vehicle(e: f64, n: f64) -> Candidate {
        Candidate { east: e, north: n, height: 3.0, rcs: 14.0, target: RadarTarget::Unit(7) }
    }

    #[test]
    fn target_behind_a_hill_is_not_painted() {
        let ground = TestGround { ridge: Some((6000.0, 400.0)), water: None };
        let mut radar = GroundRadar::default();
        pass(&mut radar, &ground, &[vehicle(0.0, 8000.0)]);
        assert!(radar.returns.is_empty());
        let mut radar = GroundRadar::default();
        pass(&mut radar, &ground, &[vehicle(0.0, 4000.0)]);
        assert_eq!(radar.returns.len(), 1);
        assert!(radar.brightness(0.0, 4000.0) > 0.8);
    }

    #[test]
    fn moving_target_jumps_when_the_beam_passes() {
        let ground = TestGround { ridge: None, water: None };
        let mut radar = GroundRadar::default();
        // Due north: the beam (from the left edge) reaches it a second into the pass.
        let at = |n: f64| [vehicle(0.0, n)];
        for _ in 0..30 {
            radar.scan(1.0 / 60.0, DVec2::ZERO, 1000.0, 0.0, RANGE, &ground, &at(5000.0));
        }
        assert!(radar.returns.is_empty(), "the beam is still left of it");
        for _ in 0..40 {
            radar.scan(1.0 / 60.0, DVec2::ZERO, 1000.0, 0.0, RANGE, &ground, &at(5000.0));
        }
        assert_eq!(radar.returns[0].north, 5000.0);
        // It drives on; the return stays where it was painted until the beam comes back.
        for _ in 0..20 {
            radar.scan(1.0 / 60.0, DVec2::ZERO, 1000.0, 0.0, RANGE, &ground, &at(5200.0));
        }
        assert_eq!(radar.returns[0].north, 5000.0);
        for _ in 0..120 {
            radar.scan(1.0 / 60.0, DVec2::ZERO, 1000.0, 0.0, RANGE, &ground, &at(5200.0));
        }
        assert_eq!(radar.returns[0].north, 5200.0);
    }

    /// The zoomed window's picture changes its rectangle only.
    #[test]
    fn draw_in_paints_its_rectangle_only() {
        let mut radar = GroundRadar::default();
        let ground = TestGround { ridge: Some((6000.0, 400.0)), water: None };
        pass(&mut radar, &ground, &[]);
        radar.on = true;
        let size = Vec2::new(160.0, 120.0);
        let view = HsdView::wedge(DVec2::ZERO, 0.0, (RANGE / 1000.0) as f32, size);
        let mut pic = Picture { w: 320, h: 240, rgba: vec![0; 320 * 240 * 4] };
        radar.draw(&view, 2.0, &mut pic);
        let before = pic.rgba.clone();
        let rect = (100, 60, 220, 140);
        let zoomed = view.zoomed(Vec2::new(80.0, 70.0), Vec2::new(80.0, 50.0), 4.0);
        radar.draw_in(&zoomed, 2.0, &mut pic, rect);
        let inside = |i: usize| {
            let (x, y) = ((i / 4) % 320, (i / 4) / 320);
            (rect.0..rect.2).contains(&x) && (rect.1..rect.3).contains(&y)
        };
        let changed: Vec<usize> =
            (0..pic.rgba.len()).filter(|&i| pic.rgba[i] != before[i]).collect();
        assert!(!changed.is_empty());
        assert!(changed.iter().all(|&i| inside(i)));
    }
}
