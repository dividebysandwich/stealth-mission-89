//! Where the open water is round the viewpoint: a square of `SIZE` x `SIZE` texels `SPAN`
//! metres across, made off the main thread from the scenery's ground (`Scenery::is_water`).
//! Red: water. Green: how far the waves are let rise, ramped up over `SHORE_M` from the
//! nearest land (the waves die down at the shore and in rivers and small lakes stay flat).

use bevy::math::DVec2;

/// Texels per side.
pub const SIZE: usize = 256;
/// Metres across.
pub const SPAN: f64 = 8192.0;
/// The mask is made again when the viewpoint is this far from its centre (metres).
pub const RECENTRE: f64 = 1536.0;
/// Waves rise fully this far from land (metres).
const SHORE_M: f32 = 240.0;

/// A finished mask.
#[derive(Clone)]
pub struct Mask {
    /// Centre (metres east, north).
    pub centre: DVec2,
    /// RGBA8: rows of increasing Bevy z (southwards), columns of increasing x (eastwards), the
    /// first texel at the (min x, min z) corner.
    pub rgba: Vec<u8>,
}

impl Mask {
    /// Builds the mask round `centre` (metres east, north), where `water` says.
    pub fn build(centre: DVec2, water: impl Fn(DVec2) -> bool) -> Self {
        let texel = SPAN / SIZE as f64;
        // Bevy x = east, z = -north: texel (i, j) at x = min_x + (i + 0.5) texel, z likewise.
        let min_x = centre.x - SPAN / 2.0;
        let min_z = -centre.y - SPAN / 2.0;
        let mut wet = vec![false; SIZE * SIZE];
        for j in 0..SIZE {
            for i in 0..SIZE {
                let x = min_x + (i as f64 + 0.5) * texel;
                let z = min_z + (j as f64 + 0.5) * texel;
                wet[j * SIZE + i] = water(DVec2::new(x, -z));
            }
        }
        // Distance to the nearest land (two pass chamfer, texels).
        let mut d = vec![0.0f32; SIZE * SIZE];
        let far = (SIZE * 2) as f32;
        for (k, w) in wet.iter().enumerate() {
            d[k] = if *w { far } else { 0.0 };
        }
        let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
        for j in 0..SIZE {
            for i in 0..SIZE {
                let k = j * SIZE + i;
                let mut v = d[k];
                if i > 0 {
                    v = v.min(d[k - 1] + a);
                }
                if j > 0 {
                    v = v.min(d[k - SIZE] + a);
                    if i > 0 {
                        v = v.min(d[k - SIZE - 1] + b);
                    }
                    if i + 1 < SIZE {
                        v = v.min(d[k - SIZE + 1] + b);
                    }
                }
                d[k] = v;
            }
        }
        for j in (0..SIZE).rev() {
            for i in (0..SIZE).rev() {
                let k = j * SIZE + i;
                let mut v = d[k];
                if i + 1 < SIZE {
                    v = v.min(d[k + 1] + a);
                }
                if j + 1 < SIZE {
                    v = v.min(d[k + SIZE] + a);
                    if i + 1 < SIZE {
                        v = v.min(d[k + SIZE + 1] + b);
                    }
                    if i > 0 {
                        v = v.min(d[k + SIZE - 1] + b);
                    }
                }
                d[k] = v;
            }
        }
        let mut rgba = Vec::with_capacity(SIZE * SIZE * 4);
        for k in 0..SIZE * SIZE {
            let rise = (d[k] * texel as f32 / SHORE_M).clamp(0.0, 1.0);
            let rise = rise * rise * (3.0 - 2.0 * rise);
            rgba.extend_from_slice(&[if wet[k] { 255 } else { 0 }, (rise * 255.0) as u8, 0, 255]);
        }
        Self { centre, rgba }
    }

    /// Water (0..1) and the waves' rise (0..1) at a world point (Bevy x, z), bilinear; none
    /// outside the mask.
    pub fn sample(&self, x: f64, z: f64) -> (f32, f32) {
        let texel = SPAN / SIZE as f64;
        let u = (x - (self.centre.x - SPAN / 2.0)) / texel - 0.5;
        let v = (z - (-self.centre.y - SPAN / 2.0)) / texel - 0.5;
        if u < 0.0 || v < 0.0 || u >= (SIZE - 1) as f64 || v >= (SIZE - 1) as f64 {
            return (0.0, 0.0);
        }
        let (i, j) = (u as usize, v as usize);
        let (fx, fy) = (u.fract() as f32, v.fract() as f32);
        let at = |i: usize, j: usize, c: usize| self.rgba[(j * SIZE + i) * 4 + c] as f32 / 255.0;
        let bil = |c| {
            let top = at(i, j, c) + (at(i + 1, j, c) - at(i, j, c)) * fx;
            let bot = at(i, j + 1, c) + (at(i + 1, j + 1, c) - at(i, j + 1, c)) * fx;
            top + (bot - top) * fy
        };
        (bil(0), bil(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sea east of x = 0: water and a ramp of the waves away from the shore.
    #[test]
    fn shore_ramp() {
        let m = Mask::build(DVec2::ZERO, |p| p.x > 0.0);
        let (w, rise) = m.sample(-500.0, 0.0);
        assert_eq!((w, rise), (0.0, 0.0));
        let (w, near) = m.sample(60.0, 0.0);
        assert!(w > 0.99 && near < 0.5, "{w} {near}");
        let (_, far) = m.sample(1000.0, 0.0);
        assert!(far > 0.99);
        // North is -z: a point north of the centre is sampled at negative z.
        let m = Mask::build(DVec2::ZERO, |p| p.y > 0.0);
        assert!(m.sample(0.0, -1000.0).0 > 0.99 && m.sample(0.0, 1000.0).0 < 0.01);
    }
}
