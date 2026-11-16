//! The ocean's frames, computed off the main thread (in the browser, where there are no
//! threads, on it at a lower resolution): the surface for the requested time, and its textures'
//! bytes (half floats, every mip level of every cascade) ready to upload.

use std::sync::Arc;

use super::spectrum::{Frame, SeaParams, Sim};

/// Texels per side of each cascade's textures.
#[cfg(not(target_arch = "wasm32"))]
pub const N: usize = 128;
#[cfg(target_arch = "wasm32")]
pub const N: usize = 64;

/// Mip levels of the cascades' textures (down to 1 x 1).
pub const MIPS: usize = N.trailing_zeros() as usize + 1;

/// A finished frame.
pub struct Done {
    pub frame: Arc<Frame>,
    /// Displacement and foam (Dx, Dy, Dz, foam), then the derivatives (∂xDy, ∂zDy, ∂xDx,
    /// ∂zDz): RGBA16F, mip level by mip level, each level's three layers in turn.
    pub disp: Vec<u8>,
    pub deriv: Vec<u8>,
}

struct Job {
    t: f64,
    params: SeaParams,
}

fn f16_bytes(v: f32, out: &mut Vec<u8>) {
    out.extend_from_slice(&half::f16::from_f32(v).to_le_bytes());
}

/// A texture's bytes with its mip levels (box filtered) from the cascades' grids.
fn pack(levels: &[&[[f32; 4]]], n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(levels.len() * n * n * 8 * 4 / 3 + 64);
    let mut current: Vec<Vec<[f32; 4]>> = levels.iter().map(|l| l.to_vec()).collect();
    let mut size = n;
    loop {
        for layer in &current {
            for t in layer {
                for v in t {
                    f16_bytes(*v, &mut out);
                }
            }
        }
        if size == 1 {
            break;
        }
        let half = size / 2;
        for layer in current.iter_mut() {
            let mut next = vec![[0.0; 4]; half * half];
            for y in 0..half {
                for x in 0..half {
                    let at = |dx: usize, dy: usize| layer[(2 * y + dy) * size + 2 * x + dx];
                    let (a, b, c, d) = (at(0, 0), at(1, 0), at(0, 1), at(1, 1));
                    next[y * half + x] =
                        std::array::from_fn(|i| (a[i] + b[i] + c[i] + d[i]) * 0.25);
                }
            }
            *layer = next;
        }
        size = half;
    }
    out
}

fn compute(sim: &mut Sim, job: Job) -> Done {
    sim.set_params(job.params);
    let frame = sim.frame(job.t);
    let disp: Vec<&[[f32; 4]]> = frame.cascades.iter().map(|c| &c.disp[..]).collect();
    let deriv: Vec<&[[f32; 4]]> = frame.cascades.iter().map(|c| &c.deriv[..]).collect();
    let (disp, deriv) = (pack(&disp, frame.n), pack(&deriv, frame.n));
    Done { frame: Arc::new(frame), disp, deriv }
}

/// The ocean's worker: one frame at a time.
pub struct Worker {
    #[cfg(not(target_arch = "wasm32"))]
    tx: std::sync::mpsc::Sender<Job>,
    #[cfg(not(target_arch = "wasm32"))]
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<Done>>,
    #[cfg(target_arch = "wasm32")]
    sim: Sim,
    #[cfg(target_arch = "wasm32")]
    ready: Option<Done>,
    busy: bool,
}

impl Worker {
    pub fn new(params: SeaParams, seed: u32) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, jobs) = std::sync::mpsc::channel::<Job>();
            let (done, rx) = std::sync::mpsc::channel();
            std::thread::Builder::new()
                .name("ocean".into())
                .spawn(move || {
                    let mut sim = Sim::new(N, params, seed);
                    while let Ok(mut job) = jobs.recv() {
                        // Only the latest request matters.
                        while let Ok(j) = jobs.try_recv() {
                            job = j;
                        }
                        if done.send(compute(&mut sim, job)).is_err() {
                            break;
                        }
                    }
                })
                .expect("ocean thread");
            Self { tx, rx: std::sync::Mutex::new(rx), busy: false }
        }
        #[cfg(target_arch = "wasm32")]
        Self { sim: Sim::new(N, params, seed), ready: None, busy: false }
    }

    /// Asks for the surface at `t` (unless a frame is still being computed).
    pub fn request(&mut self, t: f64, params: SeaParams) {
        if self.busy {
            return;
        }
        self.busy = true;
        #[cfg(not(target_arch = "wasm32"))]
        let _ = self.tx.send(Job { t, params });
        #[cfg(target_arch = "wasm32")]
        {
            self.ready = Some(compute(&mut self.sim, Job { t, params }));
        }
    }

    /// The frame finished since, if any.
    pub fn poll(&mut self) -> Option<Done> {
        #[cfg(not(target_arch = "wasm32"))]
        let done = self.rx.get_mut().ok().and_then(|rx| rx.try_recv().ok());
        #[cfg(target_arch = "wasm32")]
        let done = self.ready.take();
        if done.is_some() {
            self.busy = false;
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_every_mip_of_every_layer() {
        let n = 8;
        let a = vec![[1.0f32, 2.0, 3.0, 4.0]; n * n];
        let bytes = pack(&[&a, &a, &a], n);
        let texels: usize = (0..4).map(|m| (n >> m) * (n >> m)).sum();
        assert_eq!(bytes.len(), texels * 3 * 8);
        // The last level averages to the same values.
        let last = &bytes[bytes.len() - 8..];
        assert_eq!(half::f16::from_le_bytes([last[6], last[7]]).to_f32(), 4.0);
    }

    #[test]
    fn worker_returns_the_requested_time() {
        let mut w = Worker::new(SeaParams::default(), 1);
        w.request(2.5, SeaParams::default());
        let done = loop {
            if let Some(d) = w.poll() {
                break d;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        assert_eq!(done.frame.t, 2.5);
        assert_eq!(done.disp.len(), done.deriv.len());
    }
}
