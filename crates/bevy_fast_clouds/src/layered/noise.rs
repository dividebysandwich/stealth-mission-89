//! The layered clouds' 3D noise textures, generated once on the CPU (deterministic, tileable):
//!
//! - shape (`SHAPE`³ RGBA8): R Perlin-Worley (Perlin fBm carved by Worley fBm), G, B, A Worley
//!   fBm at 2x, 4x and 8x the base frequency, the base cloud shapes;
//! - detail (`DETAIL`³ RGBA8): R, G, B Worley fBm at three frequencies, eroding the edges.
//!
//! On native builds the textures are made on a thread started by [`start`]; the browser has none
//! and makes smaller ones at once.

use std::sync::OnceLock;

/// Shape texture size (texels per side).
#[cfg(not(target_arch = "wasm32"))]
pub const SHAPE: usize = 128;
#[cfg(target_arch = "wasm32")]
pub const SHAPE: usize = 64;
/// Detail texture size.
pub const DETAIL: usize = 32;

/// RGBA8 texels, x fastest, then y, then z.
pub struct NoiseSet {
    pub shape: Vec<u8>,
    pub detail: Vec<u8>,
}

static NOISE: OnceLock<NoiseSet> = OnceLock::new();
static STARTED: OnceLock<()> = OnceLock::new();

/// The textures, once made.
pub fn get() -> Option<&'static NoiseSet> {
    NOISE.get()
}

/// Starts making the textures (once); `ready` tells when they are there.
pub fn start() {
    if STARTED.set(()).is_err() {
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(|| {
        let _ = NOISE.set(generate());
    });
    #[cfg(target_arch = "wasm32")]
    {
        let _ = NOISE.set(generate());
    }
}

/// The textures are made.
pub fn ready() -> bool {
    NOISE.get().is_some()
}

fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x
}

fn hash3(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    hash((x as u32).wrapping_mul(0x8DA6_B343)
        ^ (y as u32).wrapping_mul(0xD816_3841)
        ^ (z as u32).wrapping_mul(0xCB1A_B31F)
        ^ seed.wrapping_mul(0x9E37_79B9))
}

fn unit(h: u32) -> f32 {
    (h >> 8) as f32 / 16_777_216.0
}

/// Gradient noise at `p` (0..1 per tile) with `period` cells across the tile, about -1..1.
fn perlin(p: [f32; 3], period: i32, seed: u32) -> f32 {
    let q = p.map(|v| v * period as f32);
    let i = q.map(|v| v.floor() as i32);
    let f = [q[0] - i[0] as f32, q[1] - i[1] as f32, q[2] - i[2] as f32];
    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let u = f.map(fade);
    let mut c = [0.0f32; 8];
    for (k, ck) in c.iter_mut().enumerate() {
        let o = [(k & 1) as i32, ((k >> 1) & 1) as i32, ((k >> 2) & 1) as i32];
        let w = [0, 1, 2].map(|a| (i[a] + o[a]).rem_euclid(period));
        let h = hash3(w[0], w[1], w[2], seed);
        // One of the 12 cube-edge directions.
        let g: [f32; 3] = match h % 12 {
            0 => [1.0, 1.0, 0.0],
            1 => [-1.0, 1.0, 0.0],
            2 => [1.0, -1.0, 0.0],
            3 => [-1.0, -1.0, 0.0],
            4 => [1.0, 0.0, 1.0],
            5 => [-1.0, 0.0, 1.0],
            6 => [1.0, 0.0, -1.0],
            7 => [-1.0, 0.0, -1.0],
            8 => [0.0, 1.0, 1.0],
            9 => [0.0, -1.0, 1.0],
            10 => [0.0, 1.0, -1.0],
            _ => [0.0, -1.0, -1.0],
        };
        let d = [f[0] - o[0] as f32, f[1] - o[1] as f32, f[2] - o[2] as f32];
        *ck = g[0] * d[0] + g[1] * d[1] + g[2] * d[2];
    }
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x0 = lerp(lerp(c[0], c[1], u[0]), lerp(c[2], c[3], u[0]), u[1]);
    let x1 = lerp(lerp(c[4], c[5], u[0]), lerp(c[6], c[7], u[0]), u[1]);
    lerp(x0, x1, u[2])
}

/// Inverted cellular noise (1 at the feature points, 0 a cell away) with `cells` per tile.
fn worley(p: [f32; 3], cells: i32, seed: u32) -> f32 {
    let q = p.map(|v| v * cells as f32);
    let i = q.map(|v| v.floor() as i32);
    let mut best = f32::MAX;
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let c = [i[0] + dx, i[1] + dy, i[2] + dz];
                let w = c.map(|v| v.rem_euclid(cells));
                let h = hash3(w[0], w[1], w[2], seed);
                let fp = [
                    c[0] as f32 + unit(h),
                    c[1] as f32 + unit(hash(h ^ 0x68E3_1DA4)),
                    c[2] as f32 + unit(hash(h ^ 0xB529_7A4D)),
                ];
                let d = [fp[0] - q[0], fp[1] - q[1], fp[2] - q[2]];
                best = best.min(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
            }
        }
    }
    1.0 - best.sqrt().min(1.0)
}

fn worley_fbm(p: [f32; 3], cells: i32, seed: u32) -> f32 {
    worley(p, cells, seed) * 0.625
        + worley(p, cells * 2, seed + 1) * 0.25
        + worley(p, cells * 4, seed + 2) * 0.125
}

fn remap(v: f32, lo: f32, hi: f32, a: f32, b: f32) -> f32 {
    a + (v - lo) / (hi - lo) * (b - a)
}

/// The shape texel at `p` (0..1 per tile): Perlin-Worley and three Worley fBms.
fn shape(p: [f32; 3]) -> [f32; 4] {
    let mut perlin_fbm = 0.0;
    let (mut amp, mut period) = (1.0, 4);
    for o in 0..4 {
        perlin_fbm += amp * perlin(p, period, 11 + o);
        amp *= 0.5;
        period *= 2;
    }
    let perlin01 = (perlin_fbm * 0.5 + 0.5).clamp(0.0, 1.0);
    let w = worley_fbm(p, 4, 101);
    let pw = remap(perlin01, 0.0, 1.0, w, 1.0).clamp(0.0, 1.0);
    [pw, worley_fbm(p, 4, 201), worley_fbm(p, 8, 301), worley_fbm(p, 16, 401)]
}

fn detail(p: [f32; 3]) -> [f32; 4] {
    [worley_fbm(p, 2, 501), worley_fbm(p, 4, 601), worley_fbm(p, 8, 701), 1.0]
}

fn bake(n: usize, texel: fn([f32; 3]) -> [f32; 4]) -> Vec<u8> {
    let mut out = vec![0u8; n * n * n * 4];
    let slice = n * n * 4;
    let fill = |z: usize, chunk: &mut [u8]| {
        for y in 0..n {
            for x in 0..n {
                let p = [x, y, z].map(|v| (v as f32 + 0.5) / n as f32);
                let t = texel(p);
                let o = (y * n + x) * 4;
                for (c, v) in t.iter().enumerate() {
                    chunk[o + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).max(1);
        let per = n.div_ceil(threads);
        std::thread::scope(|s| {
            for (t, block) in out.chunks_mut(slice * per).enumerate() {
                let fill = &fill;
                s.spawn(move || {
                    for (k, chunk) in block.chunks_mut(slice).enumerate() {
                        fill(t * per + k, chunk);
                    }
                });
            }
        });
    }
    #[cfg(target_arch = "wasm32")]
    for (z, chunk) in out.chunks_mut(slice).enumerate() {
        fill(z, chunk);
    }
    out
}

/// Makes both textures.
pub fn generate() -> NoiseSet {
    NoiseSet { shape: bake(SHAPE, shape), detail: bake(DETAIL, detail) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_tiles() {
        for p in [[0.013, 0.42, 0.77], [0.5, 0.001, 0.25], [0.9, 0.9, 0.1]] {
            for axis in 0..3 {
                let mut q = p;
                q[axis] += 1.0;
                let (a, b) = (shape(p), shape(q));
                let (c, d) = (detail(p), detail(q));
                for k in 0..4 {
                    assert!((a[k] - b[k]).abs() < 1e-4, "shape {k} along {axis}");
                    assert!((c[k] - d[k]).abs() < 1e-4, "detail {k} along {axis}");
                }
            }
        }
    }

    #[test]
    fn noise_has_range() {
        let n = 16;
        let mut lo = [1.0f32; 4];
        let mut hi = [0.0f32; 4];
        for i in 0..n * n * n {
            let p = [i % n, (i / n) % n, i / (n * n)].map(|v| (v as f32 + 0.5) / n as f32);
            let s = shape(p);
            for k in 0..4 {
                lo[k] = lo[k].min(s[k]);
                hi[k] = hi[k].max(s[k]);
            }
        }
        for k in 0..4 {
            assert!(hi[k] - lo[k] > 0.4, "channel {k}: {} .. {}", lo[k], hi[k]);
        }
    }
}
