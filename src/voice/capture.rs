//! The microphone: its samples (mono, at its own rate) piling up for the transmit system to
//! take. On the desktop through cpal on a thread of its own (a cpal stream may not move between
//! threads); in the browser through getUserMedia and an AudioWorklet (`web/voice-worklet.js`).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// Seconds of samples kept at most (older ones go: nobody is taking them).
const KEEP_S: usize = 2;

/// What the capture shares with the game.
#[derive(Default)]
struct Shared {
    samples: Mutex<VecDeque<f32>>,
    /// Sample rate (0 until the device is open).
    rate: AtomicU32,
    stop: AtomicBool,
    error: Mutex<Option<String>>,
}

impl Shared {
    fn push(&self, mono: impl Iterator<Item = f32>) {
        let rate = self.rate.load(Ordering::Relaxed).max(8000) as usize;
        let Ok(mut s) = self.samples.lock() else { return };
        s.extend(mono);
        let extra = s.len().saturating_sub(rate * KEEP_S);
        s.drain(..extra);
    }

    fn fail(&self, e: String) {
        bevy::log::warn!("microphone: {e}");
        if let Ok(mut err) = self.error.lock() {
            *err = Some(e);
        }
    }
}

/// An open microphone (closed when dropped).
pub struct Mic {
    shared: Arc<Shared>,
}

impl Mic {
    /// Opens the input device named `device` (empty: the default one).
    pub fn open(device: &str) -> Self {
        let shared = Arc::new(Shared::default());
        start(shared.clone(), device.to_string());
        Self { shared }
    }

    /// The samples since the last call, and their rate (None until the device is open).
    pub fn take(&self) -> (Vec<f32>, Option<u32>) {
        let rate = self.shared.rate.load(Ordering::Relaxed);
        let samples =
            self.shared.samples.lock().map(|mut s| s.drain(..).collect()).unwrap_or_default();
        (samples, (rate > 0).then_some(rate))
    }

    /// Why it could not be opened.
    pub fn error(&self) -> Option<String> {
        self.shared.error.lock().ok().and_then(|e| e.clone())
    }
}

impl Drop for Mic {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
    }
}

/// The names of the input devices (the desktop's).
#[cfg(not(target_arch = "wasm32"))]
pub fn devices() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    host.input_devices()
        .map(|d| d.filter_map(|d| d.description().ok().map(|d| d.name().to_string())).collect())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn devices() -> Vec<String> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn start(shared: Arc<Shared>, device: String) {
    let spawned = std::thread::Builder::new().name("microphone".into()).spawn(move || {
        if let Err(e) = run(&shared, &device) {
            shared.fail(e);
        }
    });
    if let Err(e) = spawned {
        bevy::log::warn!("microphone thread: {e}");
    }
}

/// Opens the device and keeps its stream going until the microphone is dropped.
#[cfg(not(target_arch = "wasm32"))]
fn run(shared: &Arc<Shared>, device: &str) -> Result<(), String> {
    use cpal::SampleFormat as F;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let host = cpal::default_host();
    let dev = if device.is_empty() {
        host.default_input_device()
    } else {
        host.input_devices()
            .map_err(|e| e.to_string())?
            .find(|d| d.description().is_ok_and(|n| n.name() == device))
            .or_else(|| host.default_input_device())
    }
    .ok_or("no input device")?;
    let config = dev.default_input_config().map_err(|e| e.to_string())?;
    let channels = config.channels().max(1) as usize;
    let stream = match config.sample_format() {
        F::F32 => build::<f32>(&dev, &config.config(), channels, shared),
        F::I16 => build::<i16>(&dev, &config.config(), channels, shared),
        F::U16 => build::<u16>(&dev, &config.config(), channels, shared),
        F::I32 => build::<i32>(&dev, &config.config(), channels, shared),
        f => return Err(format!("sample format {f} not supported")),
    }?;
    stream.play().map_err(|e| e.to_string())?;
    shared.rate.store(config.sample_rate(), Ordering::Relaxed);
    bevy::log::info!(
        "microphone: {} at {} Hz, {channels} channel(s)",
        dev.description().map(|d| d.name().to_string()).unwrap_or_default(),
        config.sample_rate()
    );
    while !shared.stop.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn build<T>(
    dev: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    shared: &Arc<Shared>,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    use cpal::traits::DeviceTrait;
    let data = shared.clone();
    let errors = shared.clone();
    dev.build_input_stream(
        config,
        move |input: &[T], _: &cpal::InputCallbackInfo| {
            data.push(input.chunks(channels).map(|frame| {
                frame.iter().map(|s| <f32 as cpal::FromSample<T>>::from_sample_(*s)).sum::<f32>()
                    / channels as f32
            }));
        },
        move |e| errors.fail(e.to_string()),
        None,
    )
    .map_err(|e| e.to_string())
}

/// The browser: the page's `smVoiceStart` (`web/voice.js`) asks for the microphone and hands
/// its samples to the callback below, its rate first.
#[cfg(target_arch = "wasm32")]
fn start(shared: Arc<Shared>, _device: String) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    let Some(window) = web_sys::window() else { return };
    let Ok(f) = js_sys::Reflect::get(&window, &JsValue::from_str("smVoiceStart")) else { return };
    let Some(f) = f.dyn_ref::<js_sys::Function>() else {
        shared.fail("this page has no microphone support".into());
        return;
    };
    let data = shared.clone();
    let on_samples =
        Closure::<dyn FnMut(JsValue, f64) -> bool>::new(move |block: JsValue, rate: f64| {
            if data.stop.load(Ordering::Relaxed) {
                return false;
            }
            data.rate.store(rate as u32, Ordering::Relaxed);
            if let Some(block) = block.dyn_ref::<js_sys::Float32Array>() {
                data.push(block.to_vec().into_iter());
            }
            true
        });
    let errors = shared.clone();
    let on_error = Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
        errors.fail(e.as_string().unwrap_or_else(|| "microphone refused".into()));
    });
    if let Err(e) = f.call2(&JsValue::NULL, on_samples.as_ref(), on_error.as_ref()) {
        shared.fail(format!("{e:?}"));
    }
    // The page holds them for as long as the microphone runs.
    on_samples.forget();
    on_error.forget();
}
