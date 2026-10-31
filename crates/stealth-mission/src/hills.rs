//! A theater's rolling hills: ground that rises gently above the flat world (y = 0) away from
//! water, airfields, towns and the theater's objects. The height is a fixed noise scaled by a
//! mask, so the simulation, the dedicated server and every client get the same ground from the
//! same file. Made by the game (`--write-hills`) and kept in `assets/theaters/<code>.hills`:
//! `DIM` x `DIM` bytes, row 0 at the north edge, 255 where the hills may reach their full
//! height and 0 where the ground stays flat. Without the file the theater is flat.
//!
//! Only `+`, `*`, `floor` and integer hashing, so the heights are the same bit for bit on
//! every platform.

use std::sync::Arc;

/// Mask cells per side.
pub const DIM: usize = 1024;
/// The theater's side, metres.
pub const THEATER_M: f64 = 512_000.0;
/// Metres per mask cell side.
pub const CELL_M: f64 = THEATER_M / DIM as f64;
/// The highest the hills reach (the highlands', the mask at 255), metres.
pub const MAX_M: f32 = 300.0;
/// The share of `MAX_M` the rolling hills reach outside the highlands (120 m).
pub const ROLLING: f32 = 0.4;
/// The steepest ground (rise per metre) a building, an air defence site or a drone command
/// center goes on (about 8 degrees).
pub const SITE_SLOPE: f32 = 0.14;
/// Spacing of the finest grid the hills are drawn on (metres): the ground between its points
/// is flat, cut along the diagonal from each point to the one north east of it.
pub const GRID_M: f64 = 31.25;
/// Noise octaves: wavelength (metres) and weight (the weights add up to 1).
const OCTAVES: [(f64, f32, u32); 3] =
    [(4000.0, 0.55, 0x9E37), (1700.0, 0.3, 0x85EB), (700.0, 0.15, 0xC2B2)];
/// The noise below `FLOOR` is flat ground between the hills; at `TOP` and above the hills are
/// at their highest (the octaves' sum seldom leaves 0.2..0.8).
const FLOOR: f32 = 0.38;
const TOP: f32 = 0.78;
/// Metres per map unit (map x east, y south, 0x8000 across).
const MAP_M: f64 = THEATER_M / 32768.0;
/// Metres per altitude unit.
const ALT_M: f64 = THEATER_M / 1_048_576.0;
/// Line of sight samples over the hills (metres apart), and the stretch at each end left out
/// (the ground under the eye and under what it looks at).
const LOS_STEP_M: f64 = 150.0;
const LOS_END_M: f64 = 60.0;

#[derive(Clone, Default)]
pub struct Hills(Arc<Vec<u8>>);

impl std::fmt::Debug for Hills {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hills({} cells)", self.0.len())
    }
}

impl Hills {
    /// A hills file's contents (None if it is not one).
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        (bytes.len() == DIM * DIM).then(|| Self(Arc::new(bytes)))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// No hills anywhere (no file).
    pub fn is_flat(&self) -> bool {
        self.0.is_empty()
    }

    /// The mask (0..1) at a point (metres east, north), blended between the cells' centres.
    pub fn mask(&self, e: f64, n: f64) -> f32 {
        if self.0.is_empty() {
            return 0.0;
        }
        let cell = |c: i64, r: i64| {
            if !(0..DIM as i64).contains(&c) || !(0..DIM as i64).contains(&r) {
                return 0.0;
            }
            self.0[r as usize * DIM + c as usize] as f32 * (1.0 / 255.0)
        };
        let (fx, fy) = (e / CELL_M - 0.5, (THEATER_M - n) / CELL_M - 0.5);
        let (c, r) = (fx.floor() as i64, fy.floor() as i64);
        let (tx, ty) = ((fx - c as f64) as f32, (fy - r as f64) as f32);
        let top = cell(c, r) * (1.0 - tx) + cell(c + 1, r) * tx;
        let bottom = cell(c, r + 1) * (1.0 - tx) + cell(c + 1, r + 1) * tx;
        top * (1.0 - ty) + bottom * ty
    }

    /// The hills' height in metres at a point (metres east, north).
    pub fn height_m(&self, e: f64, n: f64) -> f32 {
        let m = self.mask(e, n);
        if m <= 0.0 {
            return 0.0;
        }
        MAX_M * m * shape(rolling(e, n))
    }

    /// The hills' height in metres at a map point (x east, y south).
    pub fn at_map(&self, x: f64, y: f64) -> f32 {
        self.height_m(x * MAP_M, THEATER_M - y * MAP_M)
    }

    /// The ground as drawn on a grid of `step` metres (a multiple of `GRID_M`): the hills'
    /// heights at the grid points, flat triangles between them.
    pub fn surface_m(&self, e: f64, n: f64, step: f64) -> f32 {
        if self.0.is_empty() {
            return 0.0;
        }
        let (fx, fy) = (e / step, n / step);
        let (i, j) = (fx.floor(), fy.floor());
        let (tx, ty) = ((fx - i) as f32, (fy - j) as f32);
        let (x0, y0) = (i * step, j * step);
        let h00 = self.height_m(x0, y0);
        let h11 = self.height_m(x0 + step, y0 + step);
        if tx >= ty {
            let h10 = self.height_m(x0 + step, y0);
            h00 + (h10 - h00) * tx + (h11 - h10) * ty
        } else {
            let h01 = self.height_m(x0, y0 + step);
            h00 + (h11 - h01) * tx + (h01 - h00) * ty
        }
    }

    /// The ground as drawn up close (`surface_m` on the finest grid), metres east, north.
    pub fn ground_m(&self, e: f64, n: f64) -> f32 {
        self.surface_m(e, n, GRID_M)
    }

    /// The drawn ground at a map point (x east, y south) in altitude units.
    pub fn ground_alt(&self, x: i32, y: i32) -> i32 {
        if self.0.is_empty() {
            return 0;
        }
        (self.ground_m(x as f64 * MAP_M, THEATER_M - y as f64 * MAP_M) as f64 / ALT_M) as i32
    }

    /// Whether the hills leave the line of sight open between two points (map x, y south,
    /// altitude units): sampled where the line runs lower than the hills reach.
    pub fn sight_open(&self, a: (i32, i32, i32), b: (i32, i32, i32)) -> bool {
        if self.0.is_empty() {
            return true;
        }
        let (za, zb) = (a.2 as f64 * ALT_M, b.2 as f64 * ALT_M);
        let top = MAX_M as f64;
        if za >= top && zb >= top {
            return true;
        }
        let (ea, na) = (a.0 as f64 * MAP_M, THEATER_M - a.1 as f64 * MAP_M);
        let (eb, nb) = (b.0 as f64 * MAP_M, THEATER_M - b.1 as f64 * MAP_M);
        let len = (eb - ea).hypot(nb - na);
        if len < 2.0 * LOS_END_M {
            return true;
        }
        // The stretch of the line below the hills' top.
        let (mut t0, mut t1) = (0.0f64, 1.0f64);
        if (zb - za).abs() > 1e-9 {
            let t = (top - za) / (zb - za);
            if za < zb { t1 = t1.min(t) } else { t0 = t0.max(t) }
        }
        let end = LOS_END_M / len;
        let (t0, t1) = (t0.max(end), t1.min(1.0 - end));
        if t0 >= t1 {
            return true;
        }
        let steps = (((t1 - t0) * len) / LOS_STEP_M).ceil().max(1.0) as usize;
        (0..=steps).all(|i| {
            let t = t0 + (t1 - t0) * i as f64 / steps as f64;
            let z = za + (zb - za) * t;
            self.height_m(ea + (eb - ea) * t, na + (nb - na) * t) as f64 <= z
        })
    }

    /// The steepest rise per metre of the ground within `r` metres of a point (metres east,
    /// north): from the point to 8 around it at `r` and at half of it.
    pub fn slope_within(&self, e: f64, n: f64, r: f64) -> f32 {
        if self.0.is_empty() || !self.any_in(e, n, r) {
            return 0.0;
        }
        let r = r.max(GRID_M);
        let h0 = self.height_m(e, n);
        let mut steep = 0.0f32;
        for k in 0..8 {
            let a = k as f64 * std::f64::consts::FRAC_PI_4;
            for d in [r * 0.5, r] {
                let h = self.height_m(e + a.cos() * d, n + a.sin() * d);
                steep = steep.max((h - h0).abs() / d as f32);
            }
        }
        steep
    }

    /// The ground's rise per metre east and north at a point (central differences over 2 m).
    pub fn gradient(&self, e: f64, n: f64) -> (f32, f32) {
        let de = self.height_m(e + 1.0, n) - self.height_m(e - 1.0, n);
        let dn = self.height_m(e, n + 1.0) - self.height_m(e, n - 1.0);
        (de * 0.5, dn * 0.5)
    }

    /// Whether any hill may rise inside a square (metres east, north, half side `r`): the mask
    /// is above 0 at one of the cells it touches.
    pub fn any_in(&self, e: f64, n: f64, r: f64) -> bool {
        if self.0.is_empty() {
            return false;
        }
        let col = |e: f64| ((e / CELL_M - 0.5).floor() as i64).clamp(0, DIM as i64 - 1);
        let row =
            |n: f64| (((THEATER_M - n) / CELL_M - 0.5).floor() as i64).clamp(0, DIM as i64 - 1);
        let (c0, c1) = (col(e - r), (col(e + r) + 1).min(DIM as i64 - 1));
        let (r0, r1) = (row(n + r), (row(n - r) + 1).min(DIM as i64 - 1));
        (r0..=r1).any(|r| (c0..=c1).any(|c| self.0[r as usize * DIM + c as usize] != 0))
    }
}

/// The drawn ground (`Hills::ground_m`) over a rectangle, its grid points' heights worked out
/// once: for laying many points on it.
pub struct Patch {
    /// Grid indices of the first point east and north.
    i0: i64,
    j0: i64,
    cols: usize,
    heights: Vec<f32>,
}

impl Patch {
    /// The ground over a rectangle (metres east, north; `None` if the hills stay flat there).
    pub fn new(hills: &Hills, min: (f64, f64), max: (f64, f64)) -> Option<Self> {
        let r = (max.0 - min.0).max(max.1 - min.1) * 0.5;
        if !hills.any_in((min.0 + max.0) * 0.5, (min.1 + max.1) * 0.5, r + GRID_M) {
            return None;
        }
        let (i0, j0) = ((min.0 / GRID_M).floor() as i64 - 1, (min.1 / GRID_M).floor() as i64 - 1);
        let (i1, j1) = ((max.0 / GRID_M).floor() as i64 + 2, (max.1 / GRID_M).floor() as i64 + 2);
        let (cols, rows) = ((i1 - i0 + 1) as usize, (j1 - j0 + 1) as usize);
        let heights = (0..cols * rows)
            .map(|k| {
                let (i, j) = (i0 + (k % cols) as i64, j0 + (k / cols) as i64);
                hills.height_m(i as f64 * GRID_M, j as f64 * GRID_M)
            })
            .collect();
        Some(Self { i0, j0, cols, heights })
    }

    /// `Hills::ground_m` at a point inside the rectangle (the edge's height outside).
    pub fn ground_m(&self, e: f64, n: f64) -> f32 {
        let (fx, fy) = (e / GRID_M, n / GRID_M);
        let (i, j) = (fx.floor(), fy.floor());
        let (tx, ty) = ((fx - i) as f32, (fy - j) as f32);
        let rows = self.heights.len() / self.cols;
        let at = |di: i64, dj: i64| {
            let c = (i as i64 + di - self.i0).clamp(0, self.cols as i64 - 1) as usize;
            let r = (j as i64 + dj - self.j0).clamp(0, rows as i64 - 1) as usize;
            self.heights[r * self.cols + c]
        };
        let (h00, h11) = (at(0, 0), at(1, 1));
        if tx >= ty {
            let h10 = at(1, 0);
            h00 + (h10 - h00) * tx + (h11 - h10) * ty
        } else {
            let h01 = at(0, 1);
            h00 + (h11 - h01) * tx + (h01 - h00) * ty
        }
    }
}

/// Flat below `FLOOR`, rising smoothly to 1 at `TOP`.
fn shape(v: f32) -> f32 {
    let t = ((v - FLOOR) * (1.0 / (TOP - FLOOR))).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sum of the octaves of value noise at a point, 0..1.
fn rolling(e: f64, n: f64) -> f32 {
    OCTAVES.iter().map(|&(w, k, seed)| k * value(e / w, n / w, seed)).sum()
}

/// Value noise, 0..1, smooth (quintic) between the lattice points.
fn value(x: f64, y: f64, seed: u32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = ((x - ix) as f32, (y - iy) as f32);
    let (ix, iy) = (ix as i64 as i32, iy as i64 as i32);
    let q = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (sx, sy) = (q(fx), q(fy));
    let a = lattice(ix, iy, seed);
    let b = lattice(ix + 1, iy, seed);
    let c = lattice(ix, iy + 1, seed);
    let d = lattice(ix + 1, iy + 1, seed);
    let top = a + (b - a) * sx;
    let bottom = c + (d - c) * sx;
    top + (bottom - top) * sy
}

/// A lattice point's value, 0..1.
fn lattice(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x27D4_EB2D) ^ (y as u32).wrapping_mul(0x1656_67B1) ^ seed;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h >> 8) as f32 * (1.0 / 16_777_216.0)
}

/// The highlands for the mask writer: 0 outside, up to 1 in their heart (higher and steeper
/// hills), about a tenth of the land (wavelength about 40 km).
pub fn highland(e: f64, n: f64) -> f32 {
    let v = 0.75 * value(e / 40_000.0, n / 40_000.0, 0x3C6E)
        + 0.25 * value(e / 12_000.0, n / 12_000.0, 0x1B87);
    let t = ((v - 0.7) * (1.0 / 0.12)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Regional variety for the mask writer: 0 on the plains, up to 1 in the hill country, about
/// 40% of the land plains (wavelength about 25 km).
pub fn region(e: f64, n: f64) -> f32 {
    let v = 0.7 * value(e / 25_000.0, n / 25_000.0, 0x51ED)
        + 0.3 * value(e / 9_000.0, n / 9_000.0, 0x7F4A);
    let t = ((v - 0.38) * (1.0 / 0.22)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full() -> Hills {
        Hills::from_bytes(vec![255; DIM * DIM]).unwrap()
    }

    /// The rolling hills outside the highlands.
    fn rolling_hills() -> Hills {
        Hills::from_bytes(vec![(ROLLING * 255.0).round() as u8; DIM * DIM]).unwrap()
    }

    #[test]
    fn flat_without_a_mask() {
        let h = Hills::default();
        assert_eq!(h.height_m(100_000.0, 200_000.0), 0.0);
        assert!(!h.any_in(100_000.0, 200_000.0, 5000.0));
        assert!(Hills::from_bytes(vec![0; 5]).is_none());
    }

    /// The same heights everywhere (the dedicated server and the clients agree).
    #[test]
    fn heights_are_fixed() {
        let h = full();
        let a = h.height_m(123_456.7, 234_567.8);
        assert_eq!(a.to_bits(), full().height_m(123_456.7, 234_567.8).to_bits());
        assert_eq!(h.at_map(16384.0, 16384.0), h.height_m(256_000.0, 256_000.0));
    }

    /// Within the height limit, mostly gentle, with flat ground between the hills.
    #[test]
    fn rolling_not_steep() {
        let h = rolling_hills();
        let (mut top, mut steep, mut flat, mut n) = (0.0f32, 0.0f32, 0, 0);
        for i in 0..400 {
            for j in 0..400 {
                let (e, nn) = (100_000.0 + i as f64 * 37.0, 100_000.0 + j as f64 * 41.0);
                let z = h.height_m(e, nn);
                let (ge, gn) = h.gradient(e, nn);
                top = top.max(z);
                steep = steep.max(ge.hypot(gn));
                flat += (z == 0.0) as usize;
                n += 1;
            }
        }
        assert!(top <= MAX_M * ROLLING + 0.5 && top > 60.0, "{top}");
        // tan 15 degrees.
        assert!(steep < 0.27, "{steep}");
        assert!(flat * 10 > n, "{flat} of {n} flat");
        eprintln!("top {top} steep {steep} flat {flat}/{n}");
    }

    /// Where the mask is 0 (water, airfields) the ground is flat; it fades in over the cells.
    /// The highlands: up to `MAX_M`, steep in places, still flat in the valleys.
    #[test]
    fn highlands_higher_and_steeper() {
        let h = full();
        let (mut top, mut steep) = (0.0f32, 0.0f32);
        for i in 0..300 {
            for j in 0..300 {
                let (e, n) = (100_000.0 + i as f64 * 37.0, 100_000.0 + j as f64 * 41.0);
                top = top.max(h.height_m(e, n));
                let (ge, gn) = h.gradient(e, n);
                steep = steep.max(ge.hypot(gn));
            }
        }
        assert!(top <= MAX_M && top > 200.0, "{top}");
        // Steeper than a vehicle drives (20 degrees), not cliffs (35 degrees).
        assert!(steep > 0.36 && steep < 0.7, "{steep}");
        let c = (100_000.0, 100_000.0);
        assert!(h.slope_within(c.0, c.1, 60.0) >= 0.0);
        assert_eq!(Hills::default().slope_within(c.0, c.1, 60.0), 0.0);
    }

    #[test]
    fn highlands_are_rare() {
        let mut n = 0;
        for i in 0..100 {
            for j in 0..100 {
                n += (highland(i as f64 * 5000.0, j as f64 * 5000.0) > 0.0) as usize;
            }
        }
        assert!((300..2500).contains(&n), "{n}");
    }

    #[test]
    fn masked_flat() {
        let mut m = vec![255; DIM * DIM];
        for r in 0..DIM {
            for c in 0..512 {
                m[r * DIM + c] = 0;
            }
        }
        let h = Hills::from_bytes(m).unwrap();
        for k in 0..100 {
            assert_eq!(h.height_m(250_000.0, 10_000.0 + k as f64 * 997.0), 0.0);
        }
        assert!(!h.any_in(200_000.0, 300_000.0, 1000.0));
        assert!(h.any_in(255_800.0, 300_000.0, 1000.0));
    }

    /// The drawn ground meets the hills at the grid points and stays close between them.
    #[test]
    fn surface_on_the_grid() {
        let h = full();
        let (e, n) = (200_000.0, 300_000.0);
        assert_eq!(h.surface_m(e, n, GRID_M), h.height_m(e, n));
        let mut worst = 0.0f32;
        for k in 0..2000 {
            let (x, y) = (e + k as f64 * 7.3, n + (k * 13 % 997) as f64 * 3.1);
            worst = worst.max((h.ground_m(x, y) - h.height_m(x, y)).abs());
        }
        assert!(worst < 0.5, "{worst}");
        // Corners of the cell: continuous across the diagonal.
        let g = GRID_M;
        let c = h.surface_m(e + g * 0.5, n + g * 0.5, g);
        let (a, b) = (h.height_m(e, n), h.height_m(e + g, n + g));
        assert!((c - (a + b) * 0.5).abs() < 1e-3);
    }

    /// A patch lays points exactly where the drawn ground is.
    #[test]
    fn patch_matches() {
        let h = full();
        let p = Patch::new(&h, (100_000.0, 200_000.0), (101_000.0, 201_000.0)).unwrap();
        for k in 0..500 {
            let (e, n) = (100_000.0 + k as f64 * 1.97, 200_000.0 + (k * 7 % 500) as f64 * 1.93);
            assert_eq!(p.ground_m(e, n).to_bits(), h.ground_m(e, n).to_bits());
        }
        assert!(Patch::new(&Hills::default(), (0.0, 0.0), (10.0, 10.0)).is_none());
    }

    /// A low helicopter behind a hill is hidden from a radar; higher up it is seen.
    #[test]
    fn hill_hides_a_low_target() {
        let h = full();
        // The highest point along a line west to east, and points 3 km either side of it.
        let y = 16384;
        let top = (14000..15000).max_by_key(|&x| h.ground_alt(x, y)).unwrap();
        let (west, east) = (top - 200, top + 200);
        let alt = |x: i32, above_m: f64| h.ground_alt(x, y) + (above_m / ALT_M) as i32;
        let peak = h.ground_alt(top, y);
        assert!(peak > alt(west, 0.0) + 60 && peak > alt(east, 0.0) + 60, "no hill there");
        assert!(!h.sight_open((west, y, alt(west, 15.0)), (east, y, alt(east, 10.0))));
        assert!(h.sight_open((west, y, alt(west, 15.0)), (east, y, peak + 400)));
        assert!(Hills::default().sight_open((west, y, 0), (east, y, 0)));
    }

    #[test]
    fn regions_vary() {
        let mut plains = 0;
        for i in 0..100 {
            for j in 0..100 {
                plains += (region(i as f64 * 5000.0, j as f64 * 5000.0) == 0.0) as usize;
            }
        }
        assert!((2000..6000).contains(&plains), "{plains}");
    }
}
