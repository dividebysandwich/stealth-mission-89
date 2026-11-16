//! The sea's wave spectrum and its surface (Tessendorf's FFT ocean): a JONSWAP spectrum of
//! the wind and its fetch, the TMA correction for the depth, a Donelan-Banner style
//! directional spread with a swell term, on three cascades (patches of 640, 80 and 10 m) whose
//! wave number bands don't overlap. Each frame the spectrum is turned to the time and four
//! inverse FFTs per cascade give the displacement (choppy waves), the slopes and the
//! horizontal derivatives (the Jacobian: foam where the surface folds).
//!
//! The spectrum and spread functions follow Mozobo's Ocean-Simulation (MIT licence, Copyright
//! (c) 2025 Mozobo, https://github.com/Mozobo/Ocean-Simulation), which follows Horvath,
//! "Empirical directional wave spectra for computer graphics" (2015). Changes: the finite
//! depth dispersion relation is used for the frequency as well as its derivative, the
//! amplitude is the spectrum's variance per wave number cell (`Δk²` of the cascade), and the
//! horizontal displacement has Tessendorf's sign and a choppiness factor.

use super::fft::{C, Fft};

pub const G: f32 = 9.81;
/// The waves' heights over the spectrum's.
pub const WAVE_GAIN: f32 = 1.35;
/// JONSWAP peak enhancement.
const GAMMA: f32 = 3.3;

/// Cascade patch sizes (metres). They divide the shaders' `NOISE_PERIOD` (81920 m), so the
/// floating origin modulo that period keeps the waves in place.
pub const CASCADE_LENGTHS: [f32; 3] = [640.0, 80.0, 10.0];
/// Each cascade takes the wave numbers from the next one's boundary down: this many of the
/// next cascade's fundamental wave numbers.
const BOUNDARY: f32 = 6.0;

/// What the waves are made of.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeaParams {
    /// Wind speed at 10 m (m/s) the sea has developed under.
    pub wind_ms: f32,
    /// Direction the wind blows towards in the Bevy x / z plane (radians, atan2(z, x)).
    pub wind_dir: f32,
    /// Fetch (m): how far the wind has blown over open water.
    pub fetch_m: f32,
    /// Water depth (m), for the TMA correction and the dispersion.
    pub depth_m: f32,
    /// Swell: 0 wind sea only .. 1 long crested swell.
    pub swell: f32,
    /// Horizontal displacement factor (Tessendorf's λ).
    pub choppiness: f32,
    /// The waves' heights over the spectrum's (a livelier sea than the measured spectra).
    pub gain: f32,
}

impl Default for SeaParams {
    fn default() -> Self {
        Self {
            wind_ms: 8.0,
            wind_dir: 0.0,
            fetch_m: 300_000.0,
            depth_m: 200.0,
            swell: 0.3,
            choppiness: 0.8,
            gain: WAVE_GAIN,
        }
    }
}

/// Angular frequency of a wave number in water `h` deep.
pub fn omega(k: f32, h: f32) -> f32 {
    (G * k * (k * h).min(20.0).tanh()).sqrt()
}

/// dω/dk at depth `h`.
fn domega_dk(k: f32, h: f32) -> f32 {
    let kh = (k * h).min(20.0);
    let th = kh.tanh();
    let ch = kh.cosh();
    let w = omega(k, h).max(1e-6);
    G * (th + kh / (ch * ch)) / (2.0 * w)
}

/// JONSWAP peak frequency (rad/s) and Phillips constant α.
fn peak(p: &SeaParams) -> (f32, f32) {
    let u = p.wind_ms.max(0.5);
    // Past a dimensionless fetch of about 22000 the sea is fully developed (Pierson-Moskowitz):
    // a longer fetch raises no bigger waves.
    let f = p.fetch_m.clamp(1.0, 22_000.0 * u * u / G);
    let wp = 22.0 * (G * G / (u * f)).powf(1.0 / 3.0);
    let alpha = 0.076 * (u * u / (f * G)).powf(0.22);
    (wp, alpha)
}

/// JONSWAP spectrum S(ω) (m² s).
fn jonswap(w: f32, wp: f32, alpha: f32) -> f32 {
    let sigma = if w <= wp { 0.07 } else { 0.09 };
    let r = (-(w - wp) * (w - wp) / (2.0 * sigma * sigma * wp * wp)).exp();
    alpha * G * G / w.powi(5) * (-1.25 * (wp / w).powi(4)).exp() * GAMMA.powf(r)
}

/// TMA (Kitaigorodskii) depth attenuation.
fn tma(w: f32, h: f32) -> f32 {
    let wh = w * (h / G).sqrt();
    if wh <= 1.0 {
        0.5 * wh * wh
    } else if wh < 2.0 {
        1.0 - 0.5 * (2.0 - wh) * (2.0 - wh)
    } else {
        1.0
    }
}

/// Spread exponent s(ω) (Donelan-Banner fit, as Mozobo's).
fn spread_power(w: f32, wp: f32, u: f32) -> f32 {
    if w < 1.05 * wp {
        6.97 * (w / wp).abs().powf(4.06)
    } else {
        let peak_speed = G / wp;
        let mu = -2.33 - 1.45 * (u / peak_speed - 1.17);
        9.77 * (w / wp).abs().powf(mu)
    }
}

/// Normalisation of cos(θ/2)^{2s} over the circle.
fn spread_norm(s: f32) -> f32 {
    use std::f32::consts::{LN_2, PI};
    if s <= 0.4 {
        0.09 * s * s * s + (LN_2 * LN_2 / PI - PI / 12.0) * s * s + LN_2 / PI * s + 1.0 / (2.0 * PI)
    } else {
        s.sqrt() / (2.0 * PI.sqrt()) + 1.0 / (16.0 * (PI * s).sqrt())
    }
}

/// Directional spread D(ω, θ) about the wind direction, the swell narrowing it.
fn spread(w: f32, wp: f32, theta: f32, p: &SeaParams) -> f32 {
    let s =
        (spread_power(w, wp, p.wind_ms) + 16.0 * (w / wp).tanh() * p.swell * p.swell).min(200.0);
    spread_norm(s) * (0.5 * (theta - p.wind_dir)).cos().abs().powf(2.0 * s)
}

/// The k vector of grid cell (x, y) of a cascade (FFT order: indices past n/2 negative).
fn k_vector(x: usize, y: usize, n: usize, length: f32) -> (f32, f32) {
    let dk = std::f32::consts::TAU / length;
    let signed = |i: usize| if i < n / 2 { i as f32 } else { i as f32 - n as f32 };
    (signed(x) * dk, signed(y) * dk)
}

/// A cascade's band of wave numbers (rad/m).
pub fn band(cascade: usize) -> (f32, f32) {
    let fundamental = |i: usize| std::f32::consts::TAU / CASCADE_LENGTHS[i];
    let lo = if cascade == 0 { 0.0 } else { BOUNDARY * fundamental(cascade) };
    let hi = CASCADE_LENGTHS
        .get(cascade + 1)
        .map_or(f32::INFINITY, |_| BOUNDARY * fundamental(cascade + 1));
    (lo, hi)
}

/// Standard normal pairs (Box-Muller from a xorshift stream).
pub fn gaussian_noise(n: usize, seed: u32) -> Vec<(f32, f32)> {
    let mut s = seed | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        ((s >> 8) as f32 + 0.5) / (1u32 << 24) as f32
    };
    (0..n)
        .map(|_| {
            let (u1, u2) = (next(), next());
            let r = (-2.0 * u1.ln()).sqrt();
            let a = std::f32::consts::TAU * u2;
            (r * a.cos(), r * a.sin())
        })
        .collect()
}

/// One cascade's initial spectrum h0(k) and its ω(k).
#[derive(Debug, Clone)]
pub struct Cascade {
    pub length: f32,
    h0: Vec<C>,
    omega: Vec<f32>,
}

impl Cascade {
    pub fn new(cascade: usize, n: usize, p: &SeaParams, noise: &[(f32, f32)]) -> Self {
        let length = CASCADE_LENGTHS[cascade];
        let (lo, hi) = band(cascade);
        let (wp, alpha) = peak(p);
        let dk = std::f32::consts::TAU / length;
        let mut h0 = vec![C::ZERO; n * n];
        let mut om = vec![0.0; n * n];
        for y in 0..n {
            for x in 0..n {
                let i = y * n + x;
                let (kx, kz) = k_vector(x, y, n, length);
                let k = (kx * kx + kz * kz).sqrt();
                if k <= 0.0 {
                    continue;
                }
                let w = omega(k, p.depth_m);
                om[i] = w;
                if k < lo || k >= hi {
                    continue;
                }
                // The 2D wave number spectrum S(k) = S(ω) D(θ) dω/dk / k, its variance in the
                // cell: S(k) Δk². h(k, t) = h0(k) e^{iωt} + conj(h0(-k)) e^{-iωt} carries twice
                // E|h0|², so E|h0|² = S Δk² / 2, with h0 = (ξ1 + iξ2) a: a = sqrt(S Δk²) / 2.
                let s = jonswap(w, wp, alpha)
                    * tma(w, p.depth_m)
                    * spread(w, wp, kz.atan2(kx), p)
                    * domega_dk(k, p.depth_m)
                    / k;
                let a = (s.max(0.0) * dk * dk).sqrt() * 0.5 * p.gain;
                let (g1, g2) = noise[(y * n + x) % noise.len()];
                h0[i] = C::new(g1 * a, g2 * a);
            }
        }
        Self { length, h0, omega: om }
    }

    /// The spectrum at time `t`: h(k, t) = h0(k) e^{iωt} + conj(h0(-k)) e^{-iωt}.
    fn at(&self, n: usize, t: f32, out: &mut [C]) {
        for y in 0..n {
            for x in 0..n {
                let i = y * n + x;
                let j = ((n - y) % n) * n + (n - x) % n;
                let e = C::cis(self.omega[i] * t);
                out[i] = self.h0[i] * e + self.h0[j].conj() * e.conj();
            }
        }
    }
}

/// One cascade's surface at a time: per grid point (x along Bevy x, y along Bevy z)
/// displacement (Dx, Dy, Dz), foam 0..1, and the derivatives (∂xDy, ∂zDy, ∂xDx, ∂zDz).
#[derive(Debug, Clone, Default)]
pub struct Surface {
    pub disp: Vec<[f32; 4]>,
    pub deriv: Vec<[f32; 4]>,
}

/// The three cascades' surfaces at a time.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub n: usize,
    /// The time it is the surface at (s).
    #[allow(dead_code)]
    pub t: f64,
    pub cascades: Vec<Surface>,
}

/// The sea's wave simulation (the worker's): its cascades and their foam.
pub struct Sim {
    pub n: usize,
    params: SeaParams,
    cascades: Vec<Cascade>,
    noise: Vec<Vec<(f32, f32)>>,
    fft: Fft,
    work: [Vec<C>; 4],
    scratch: Vec<C>,
    foam: Vec<Vec<f32>>,
    last_t: Option<f64>,
}

/// Foam fades with this time constant (s).
const FOAM_FADE_S: f32 = 1.8;
/// Where the Jacobian drops below `foam_threshold`, the crest breaks into foam, fully
/// `FOAM_RANGE` below it. The threshold rises with the wind (whitecaps from about 4 m/s, a few
/// percent of the sea at 10 m/s, much of it in a gale).
const FOAM_RANGE: f32 = 0.08;

fn foam_threshold(wind_ms: f32) -> f32 {
    if wind_ms < 4.0 { 0.0 } else { (0.64 + 0.011 * wind_ms).min(0.9) }
}

impl Sim {
    pub fn new(n: usize, p: SeaParams, seed: u32) -> Self {
        let noise: Vec<_> = (0..CASCADE_LENGTHS.len())
            .map(|c| {
                gaussian_noise(n * n, seed.wrapping_mul(0x9E37_79B9).wrapping_add(c as u32 * 7919))
            })
            .collect();
        let cascades =
            (0..CASCADE_LENGTHS.len()).map(|c| Cascade::new(c, n, &p, &noise[c])).collect();
        Self {
            n,
            params: p,
            cascades,
            noise,
            fft: Fft::new(n),
            work: std::array::from_fn(|_| vec![C::ZERO; n * n]),
            scratch: Vec::new(),
            foam: vec![vec![0.0; n * n]; CASCADE_LENGTHS.len()],
            last_t: None,
        }
    }

    /// New wind or sea: the spectrum again, with the same random phases (the waves grow or
    /// calm down where they are).
    pub fn set_params(&mut self, p: SeaParams) {
        if p == self.params {
            return;
        }
        self.params = p;
        self.cascades = (0..CASCADE_LENGTHS.len())
            .map(|c| Cascade::new(c, self.n, &p, &self.noise[c]))
            .collect();
    }

    /// The surface at time `t` (s).
    pub fn frame(&mut self, t: f64) -> Frame {
        let n = self.n;
        let lambda = self.params.choppiness;
        let dt = self.last_t.map_or(0.0, |l| (t - l).clamp(0.0, 1.0) as f32);
        self.last_t = Some(t);
        let fade = (-dt / FOAM_FADE_S).exp();
        let threshold = foam_threshold(self.params.wind_ms);
        let tf = (t % 100_000.0) as f32;
        let mut out = Frame { n, t, cascades: Vec::with_capacity(self.cascades.len()) };
        for (ci, c) in self.cascades.iter().enumerate() {
            let [f1, f2, f3, f4] = &mut self.work;
            c.at(n, tf, f1);
            // Pack two real fields into each complex one: A + iB (both spectra Hermitian).
            for y in 0..n {
                for x in 0..n {
                    let i = y * n + x;
                    let h = f1[i];
                    let (kx, kz) = k_vector(x, y, n, c.length);
                    let k = (kx * kx + kz * kz).sqrt();
                    let inv = if k > 0.0 { 1.0 / k } else { 0.0 };
                    let ih = h.i();
                    // Dx = -iλ kx/k h, Dz = -iλ kz/k h; ∂xDy = i kx h, ∂zDy = i kz h;
                    // ∂xDx = λ kx²/k h, ∂zDz = λ kz²/k h, ∂xDz = λ kx kz/k h.
                    let dx = ih.scale(-lambda * kx * inv);
                    let dz = ih.scale(-lambda * kz * inv);
                    let dyx = ih.scale(kx);
                    let dyz = ih.scale(kz);
                    let dxx = h.scale(lambda * kx * kx * inv);
                    let dzz = h.scale(lambda * kz * kz * inv);
                    let dxz = h.scale(lambda * kx * kz * inv);
                    f1[i] = dx + dz.i();
                    f2[i] = h + dxz.i();
                    f3[i] = dyx + dyz.i();
                    f4[i] = dxx + dzz.i();
                }
            }
            for f in self.work.iter_mut() {
                self.fft.inverse_2d(f, &mut self.scratch);
            }
            let [f1, f2, f3, f4] = &self.work;
            let foam = &mut self.foam[ci];
            let mut s = Surface { disp: vec![[0.0; 4]; n * n], deriv: vec![[0.0; 4]; n * n] };
            for i in 0..n * n {
                let (dxx, dzz, dxz) = (f4[i].re, f4[i].im, f2[i].im);
                let j = (1.0 + dxx) * (1.0 + dzz) - dxz * dxz;
                let breaking = ((threshold - j) / FOAM_RANGE).clamp(0.0, 1.0);
                foam[i] = (foam[i] * fade).max(breaking);
                s.disp[i] = [f1[i].re, f2[i].re, f1[i].im, foam[i]];
                s.deriv[i] = [f3[i].re, f3[i].im, dxx, dzz];
            }
            out.cascades.push(s);
        }
        out
    }
}

impl Frame {
    /// Bilinear sample of a cascade's displacement at a world point (Bevy x, z, absolute).
    fn sample(&self, c: usize, x: f64, z: f64) -> [f32; 4] {
        let n = self.n;
        let l = CASCADE_LENGTHS[c] as f64;
        let u = (x / l).rem_euclid(1.0) * n as f64;
        let v = (z / l).rem_euclid(1.0) * n as f64;
        let (x0, y0) = (u.floor() as usize % n, v.floor() as usize % n);
        let (x1, y1) = ((x0 + 1) % n, (y0 + 1) % n);
        let (fx, fy) = (u.fract() as f32, v.fract() as f32);
        let d = &self.cascades[c].disp;
        let at = |x: usize, y: usize| d[y * n + x];
        let lerp =
            |a: [f32; 4], b: [f32; 4], t: f32| std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t);
        lerp(lerp(at(x0, y0), at(x1, y0), fx), lerp(at(x0, y1), at(x1, y1), fx), fy)
    }

    /// The summed displacement of the cascades from the undisplaced point (x, z), each
    /// weighted (the finest ones matter little to a hull).
    pub fn displacement(&self, x: f64, z: f64, weights: [f32; 3]) -> [f32; 3] {
        let mut d = [0.0; 3];
        for (c, w) in weights.iter().enumerate().take(self.cascades.len()) {
            if *w == 0.0 {
                continue;
            }
            let s = self.sample(c, x, z);
            for i in 0..3 {
                d[i] += s[i] * w;
            }
        }
        d
    }

    /// The water's height (m) at the world point (x, z): the undisplaced point found by a few
    /// fixed point steps against the horizontal displacement.
    pub fn height(&self, x: f64, z: f64, weights: [f32; 3]) -> f32 {
        let (mut px, mut pz) = (x, z);
        let mut d = self.displacement(px, pz, weights);
        for _ in 0..3 {
            px = x - d[0] as f64;
            pz = z - d[2] as f64;
            d = self.displacement(px, pz, weights);
        }
        d[1]
    }
}

/// Significant wave height (m) the spectrum gives: 4 √m0 over the cascades' bands.
#[cfg(test)]
pub fn significant_height(p: &SeaParams) -> f32 {
    let (wp, alpha) = peak(p);
    let (k_lo, k_hi) =
        (band(0).0.max(std::f32::consts::TAU / CASCADE_LENGTHS[0]), band(2).1.min(200.0));
    let steps = 4000;
    let mut m0 = 0.0;
    let (w0, w1) = (omega(k_lo, p.depth_m), omega(k_hi, p.depth_m));
    let dw = (w1 - w0) / steps as f32;
    for i in 0..steps {
        let w = w0 + (i as f32 + 0.5) * dw;
        m0 += jonswap(w, wp, alpha) * tma(w, p.depth_m) * dw;
    }
    4.0 * m0.sqrt() * p.gain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variance(f: &Frame) -> f32 {
        let n = f.n;
        let mut v = 0.0f64;
        // The sum of the cascades at the grid of the finest-but-one is aliased; the cascades
        // are independent, so their variances add.
        for s in &f.cascades {
            let mean: f64 = s.disp.iter().map(|d| d[1] as f64).sum::<f64>() / (n * n) as f64;
            v += s.disp.iter().map(|d| (d[1] as f64 - mean).powi(2)).sum::<f64>() / (n * n) as f64;
        }
        v as f32
    }

    /// The surface's height variance matches the spectrum's (Hs = 4 σ) within the sampling's
    /// randomness, and a stronger wind raises far bigger waves.
    #[test]
    fn wave_height_matches_the_spectrum() {
        for wind in [5.0, 12.0, 20.0] {
            let p = SeaParams { wind_ms: wind, ..Default::default() };
            let mut sim = Sim::new(128, p, 7);
            let f = sim.frame(10.0);
            let hs = 4.0 * variance(&f).sqrt();
            let want = significant_height(&p);
            println!("wind {wind} m/s: Hs {hs:.2} m, spectrum {want:.2} m");
            assert!((hs / want - 1.0).abs() < 0.35, "wind {wind}: {hs} vs {want}");
        }
        let calm = significant_height(&SeaParams { wind_ms: 4.0, ..Default::default() });
        let gale = significant_height(&SeaParams { wind_ms: 20.0, ..Default::default() });
        assert!(gale > calm * 10.0, "{calm} {gale}");
    }

    /// Shallow water (TMA) holds the long waves down.
    #[test]
    fn shallow_water_is_calmer() {
        let deep = significant_height(&SeaParams { wind_ms: 18.0, ..Default::default() });
        let shallow =
            significant_height(&SeaParams { wind_ms: 18.0, depth_m: 8.0, ..Default::default() });
        assert!(shallow < deep * 0.8, "{shallow} {deep}");
    }

    /// The surface is real and periodic over each patch, and its height sampled at a point
    /// moves smoothly with time.
    #[test]
    fn surface_is_real_and_smooth_in_time() {
        let p = SeaParams { wind_ms: 12.0, ..Default::default() };
        let mut sim = Sim::new(64, p, 3);
        let mut prev: Option<f32> = None;
        for i in 0..50 {
            let f = sim.frame(i as f64 * 0.02);
            let h = f.height(123.4, -56.7, [1.0, 1.0, 0.0]);
            if let Some(p) = prev {
                assert!((h - p).abs() < 0.2, "{p} -> {h}");
            }
            prev = Some(h);
        }
        // Periodic: the same height a patch away.
        let f = sim.frame(3.0);
        let a = f.height(10.0, 20.0, [1.0, 0.0, 0.0]);
        let b = f.height(10.0 + 640.0, 20.0 - 640.0, [1.0, 0.0, 0.0]);
        assert!((a - b).abs() < 1e-3);
    }

    /// Waves run downwind: the wind's direction carries most of the energy.
    #[test]
    fn waves_follow_the_wind() {
        let n = 64;
        let p = SeaParams { wind_ms: 12.0, wind_dir: 0.0, ..Default::default() };
        let c = Cascade::new(0, n, &p, &gaussian_noise(n * n, 5));
        let (mut along, mut across) = (0.0, 0.0);
        for y in 0..n {
            for x in 0..n {
                let (kx, kz) = k_vector(x, y, n, c.length);
                let e = c.h0[y * n + x].re.powi(2) + c.h0[y * n + x].im.powi(2);
                if kx.abs() > kz.abs() {
                    along += e;
                } else {
                    across += e;
                }
            }
        }
        assert!(along > across * 3.0, "{along} {across}");
    }
}

#[cfg(test)]
mod bench {
    /// `cargo test --release --lib ocean_frame_time -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn ocean_frame_time() {
        for n in [64, 128, 256] {
            let mut sim = super::Sim::new(n, super::SeaParams::default(), 1);
            let t0 = std::time::Instant::now();
            for i in 0..20 {
                std::hint::black_box(sim.frame(i as f64 / 60.0));
            }
            println!("N={n}: {:.2} ms per frame", t0.elapsed().as_secs_f64() * 1000.0 / 20.0);
        }
    }
}

#[cfg(test)]
mod foam_probe {
    #[test]
    #[ignore]
    fn foam_coverage() {
        for wind in [6.0, 10.0, 14.0, 18.0, 22.0] {
            let mut sim =
                super::Sim::new(128, super::SeaParams { wind_ms: wind, ..Default::default() }, 3);
            let f = sim.frame(5.0);
            let mut out = String::new();
            for c in &f.cascades {
                let jmin =
                    c.deriv.iter().map(|d| (1.0 + d[2]) * (1.0 + d[3])).fold(9.0f32, f32::min);
                let cover =
                    c.disp.iter().filter(|d| d[3] > 0.1).count() as f32 / c.disp.len() as f32;
                out += &format!(" [jmin~{jmin:.2} foam {:.1}%]", cover * 100.0);
            }
            println!("wind {wind}:{out}");
        }
    }
}
