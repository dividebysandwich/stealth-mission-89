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
/// The highest the hills reach, metres.
pub const MAX_M: f32 = 120.0;
/// Noise octaves: wavelength (metres) and weight (the weights add up to 1).
const OCTAVES: [(f64, f32, u32); 3] =
    [(4000.0, 0.55, 0x9E37), (1700.0, 0.3, 0x85EB), (700.0, 0.15, 0xC2B2)];
/// The noise below `FLOOR` is flat ground between the hills; at `TOP` and above the hills are
/// at their highest (the octaves' sum seldom leaves 0.2..0.8).
const FLOOR: f32 = 0.38;
const TOP: f32 = 0.78;
/// Metres per map unit (map x east, y south, 0x8000 across).
const MAP_M: f64 = THEATER_M / 32768.0;

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
        let h = full();
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
        assert!(top <= MAX_M && top > 60.0, "{top}");
        // tan 15 degrees.
        assert!(steep < 0.27, "{steep}");
        assert!(flat * 10 > n, "{flat} of {n} flat");
        eprintln!("top {top} steep {steep} flat {flat}/{n}");
    }

    /// Where the mask is 0 (water, airfields) the ground is flat; it fades in over the cells.
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
