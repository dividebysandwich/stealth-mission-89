//! Radix-2 complex FFT (iterative Cooley-Tukey) for the ocean's square power-of-two grids.
//! Only the inverse transform is needed, without the 1/N scale (the spectrum carries the
//! amplitudes).

/// A complex number.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct C {
    pub re: f32,
    pub im: f32,
}

impl C {
    pub const ZERO: C = C { re: 0.0, im: 0.0 };

    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    pub fn conj(self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    /// i * self.
    pub fn i(self) -> Self {
        Self { re: -self.im, im: self.re }
    }

    pub fn scale(self, s: f32) -> Self {
        Self { re: self.re * s, im: self.im * s }
    }

    /// e^{i a}.
    pub fn cis(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Self { re: c, im: s }
    }
}

impl std::ops::Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C { re: self.re + o.re, im: self.im + o.im }
    }
}

impl std::ops::Sub for C {
    type Output = C;
    fn sub(self, o: C) -> C {
        C { re: self.re - o.re, im: self.im - o.im }
    }
}

impl std::ops::Mul for C {
    type Output = C;
    fn mul(self, o: C) -> C {
        C { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
}

/// The plan of an inverse FFT of size `n` (a power of two): bit reversal and twiddles.
#[derive(Debug, Clone)]
pub struct Fft {
    n: usize,
    rev: Vec<u32>,
    /// e^{+2πi k/n}, k < n/2.
    twiddle: Vec<C>,
}

impl Fft {
    pub fn new(n: usize) -> Self {
        assert!(n.is_power_of_two() && n >= 2);
        let bits = n.trailing_zeros();
        let rev = (0..n as u32).map(|i| i.reverse_bits() >> (32 - bits)).collect();
        let twiddle =
            (0..n / 2).map(|k| C::cis(std::f32::consts::TAU * k as f32 / n as f32)).collect();
        Self { n, rev, twiddle }
    }

    /// In-place inverse transform of one row: x[j] = Σ_k X[k] e^{+2πi jk/n}.
    pub fn inverse(&self, x: &mut [C]) {
        let n = self.n;
        debug_assert_eq!(x.len(), n);
        for (i, &j) in self.rev.iter().enumerate() {
            let j = j as usize;
            if i < j {
                x.swap(i, j);
            }
        }
        let mut size = 2;
        while size <= n {
            let half = size / 2;
            let step = n / size;
            for start in (0..n).step_by(size) {
                for k in 0..half {
                    let w = self.twiddle[k * step];
                    let a = x[start + k];
                    let b = x[start + k + half] * w;
                    x[start + k] = a + b;
                    x[start + k + half] = a - b;
                }
            }
            size *= 2;
        }
    }

    /// In-place inverse transform of an n x n grid (row major), using `scratch` (n values) for
    /// the columns.
    pub fn inverse_2d(&self, grid: &mut [C], scratch: &mut Vec<C>) {
        let n = self.n;
        debug_assert_eq!(grid.len(), n * n);
        for row in grid.chunks_exact_mut(n) {
            self.inverse(row);
        }
        scratch.resize(n, C::ZERO);
        for x in 0..n {
            for y in 0..n {
                scratch[y] = grid[y * n + x];
            }
            self.inverse(scratch);
            for y in 0..n {
                grid[y * n + x] = scratch[y];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Against the direct sum.
    #[test]
    fn matches_the_direct_transform() {
        let n = 16;
        let fft = Fft::new(n);
        let input: Vec<C> =
            (0..n).map(|i| C::new((i as f32 * 0.7).sin(), (i as f32 * 1.3).cos())).collect();
        let mut x = input.clone();
        fft.inverse(&mut x);
        for (j, got) in x.iter().enumerate() {
            let want = input.iter().enumerate().fold(C::ZERO, |s, (k, v)| {
                s + *v * C::cis(std::f32::consts::TAU * (j * k) as f32 / n as f32)
            });
            assert!((got.re - want.re).abs() < 1e-4 && (got.im - want.im).abs() < 1e-4);
        }
    }

    /// A single wave number in 2D gives the plane wave.
    #[test]
    fn plane_wave_in_2d() {
        let n = 8;
        let fft = Fft::new(n);
        let mut g = vec![C::ZERO; n * n];
        // kx = 1, ky = 2 (row y, column x).
        g[2 * n + 1] = C::new(1.0, 0.0);
        fft.inverse_2d(&mut g, &mut Vec::new());
        for y in 0..n {
            for x in 0..n {
                let want = C::cis(std::f32::consts::TAU * (x + 2 * y) as f32 / n as f32);
                let got = g[y * n + x];
                assert!((got.re - want.re).abs() < 1e-5 && (got.im - want.im).abs() < 1e-5);
            }
        }
    }
}
