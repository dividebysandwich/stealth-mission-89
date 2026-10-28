//! A Patriot console's radar signal screen (where a Gepard's shows its target data, `info`):
//! what the phased array receives, live, refreshed like a real display a dozen times a second.
//!
//! * The A-scope: return amplitude (dB over the noise) against range out to the air picture's
//!   range: the receiver's noise, ground clutter near the site, an echo for every aircraft and
//!   missile in the beam (strength from its radar cross section and range, the fourth power
//!   law), the lock's range gate over its echo.
//! * The lock's Doppler spectrum: main beam clutter at zero (strong for a low target), the
//!   target's line at its radial velocity's Doppler shift with the engine's compressor blade
//!   modulation (JEM) beside it, a helicopter's rotor smeared wide.
//! * The monopulse error: the lock's angle error in the beam (it wanders more the weaker the
//!   echo), and the signal's numbers: signal to noise, range, radial velocity, Doppler, the
//!   guidance mode and the battery's missiles in the air.

use bevy::prelude::*;
use stealth_sim::station::Guidance;
use stealth_sim::{AircraftKind, Sim};

use super::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};
use super::{ConsoleUi, Manning};
use crate::canvas::{Canvas, CanvasTexture};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, INFO_CANVAS};
use crate::font::{Align, stroke_text_at};

/// Redraws per second (the noise flickers at this rate).
const RATE: f32 = 12.0;
/// The radar's wavelength (m: C band, about 5.5 GHz).
const WAVELENGTH: f64 = 0.055;
/// A 1 m² target's echo at 1 km (dB over the noise); the fourth power law from there.
const ONE_SQM_AT_KM: f64 = 95.0;
/// The A-scope's and the spectrum's amplitude scale (dB over the noise, full height).
const SCOPE_DB: f32 = 60.0;
/// The spectrum's span (kHz each side of zero).
const SPAN_KHZ: f64 = 16.0;
/// Range resolution of the echoes on the A-scope (m).
const PULSE_M: f64 = 450.0;

/// Areas: the A-scope, the spectrum, the monopulse box (left, top, right, bottom), the
/// numbers' column (left, right), the bottom row.
const ASCOPE: (f32, f32, f32, f32) = (24.0, 30.0, 314.0, 118.0);
const SPECTRUM: (f32, f32, f32, f32) = (24.0, 140.0, 194.0, 206.0);
const ERROR_BOX: (f32, f32, f32, f32) = (204.0, 140.0, 254.0, 190.0);
const NUMBERS_X: f32 = 260.0;
const NUMBERS_RIGHT: f32 = 314.0;
const BOTTOM_Y: f32 = 228.0;
/// Text height and pen.
const TEXT: f32 = 6.0;
const PEN: f32 = 1.0;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<crate::combat::Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<ConsoleUi>,
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    mut seed: Local<u32>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c) else { return };
    if m.gepard {
        return;
    }
    *since += time.delta_secs();
    if *since < 1.0 / RATE {
        return;
    }
    *since = 0.0;
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Info) else {
        return;
    };
    tex.dirty = true;
    *seed = seed.wrapping_add(0x9E37_79B9);
    let mut noise = Noise(*seed | 1);
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let sim = &combat.sim;
    let Some(radar) = sim.aaa_units().get(m.unit).copied() else { return };
    let site = radar.metres();
    let st = sim.station_of(c.me.0);
    let lock = st.and_then(|s| s.lock).map(|l| l as usize);
    let (w, h) = (INFO_CANVAS.0 as f32, INFO_CANVAS.1 as f32);
    cv.frame(Vec2::new(1.0, 1.0), Vec2::new(w - 2.0, h - 2.0), 1.0, GREEN);
    stroke_text_at(cv, 8.0, 12.0, "RADAR SIGNAL", 7.0, 1.1, WHITE, Align::Left);
    let mode = if radar.destroyed {
        ("OFF", RED)
    } else if lock.is_some() {
        ("TRACK", GREEN)
    } else {
        ("SEARCH", YELLOW)
    };
    stroke_text_at(cv, w - 8.0, 12.0, mode.0, 7.0, 1.1, mode.1, Align::Right);
    if radar.destroyed {
        stroke_text_at(cv, w / 2.0, h / 2.0, "NO SIGNAL", 9.0, 1.4, RED, Align::Centre);
        return;
    }

    let echoes = echoes(sim, site);
    let range_m = ui.range_km as f64 * 1000.0;
    let target = lock.and_then(|l| echoes.iter().find(|e| e.unit == Some(l)));
    ascope(cv, &echoes, target, range_m, &mut noise);
    match target {
        Some(t) => {
            spectrum(cv, t, &mut noise);
            monopulse(cv, t, &mut noise);
            numbers(cv, t, sim, m.unit, st.map(|s| s.mode));
        }
        None => {
            let (l, t, r, b) = SPECTRUM;
            cv.frame(Vec2::new(l, t), Vec2::new(r, b), 0.8, DARK);
            let text = if lock.is_some() { "TRACK LOST" } else { "NO TRACK" };
            let col = if lock.is_some() { RED } else { GREY };
            stroke_text_at(cv, (l + r) / 2.0, (t + b) / 2.0, text, 7.0, 1.0, col, Align::Centre);
        }
    }
    let missiles = sim.ai_missiles().iter().filter(|a| a.station == Some(m.unit as u16)).count();
    let illum = match st.map(|s| s.mode) {
        Some(Guidance::SemiActive) => "ILLUM CW",
        _ => "TVM UPLINK",
    };
    stroke_text_at(cv, 8.0, BOTTOM_Y, illum, TEXT, PEN, CYAN, Align::Left);
    stroke_text_at(
        cv,
        w / 2.0,
        BOTTOM_Y,
        &format!("{} ECHOES", echoes.iter().filter(|e| e.range <= range_m).count()),
        TEXT,
        PEN,
        GREY,
        Align::Centre,
    );
    stroke_text_at(
        cv,
        w - 8.0,
        BOTTOM_Y,
        &format!("MSL AIR {missiles}"),
        TEXT,
        PEN,
        if missiles > 0 { YELLOW } else { GREY },
        Align::Right,
    );
}

/// A cheap noise source (xorshift): uniform 0..1.
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1 << 24) as f32
    }

    /// A Rayleigh-like amplitude fluctuation (dB, about zero mean).
    fn speckle(&mut self) -> f32 {
        10.0 * (-(self.next().max(1e-4)).ln()).log10()
    }
}

/// An echo: the flight unit (None: a missile), slant range (m), strength (dB over the noise),
/// radial velocity (m/s, positive opening), its height (m) and kind.
struct Echo {
    unit: Option<usize>,
    range: f64,
    db: f64,
    radial: f64,
    alt: f64,
    rotary: bool,
}

/// The echo of a target of `rcs` m² at `range` m.
fn strength(rcs: f64, range: f64) -> f64 {
    ONE_SQM_AT_KM + 10.0 * rcs.log10() - 40.0 * (range / 1000.0).max(0.2).log10()
}

fn echoes(sim: &Sim, site: [f64; 3]) -> Vec<Echo> {
    let echo = |unit, pos: [f64; 3], vel: [f64; 3], rcs: f64, rotary| {
        let d = [pos[0] - site[0], pos[1] - site[1], pos[2] - site[2]];
        let range = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1.0);
        let radial = (d[0] * vel[0] + d[1] * vel[1] + d[2] * vel[2]) / range;
        Echo { unit, range, db: strength(rcs, range), radial, alt: pos[2], rotary }
    };
    let mut out: Vec<Echo> = (0..sim.flights().len())
        .filter_map(|i| {
            let (pos, vel) = sim.track_alive(i)?;
            let rotary = sim.aircraft_kind(sim.flights()[i].spec) == AircraftKind::Helicopter;
            Some(echo(Some(i), pos, vel, if rotary { 8.0 } else { 4.0 }, rotary))
        })
        .collect();
    out.extend(sim.ai_missiles().iter().map(|a| echo(None, a.pos, a.vel, 0.08, false)));
    out
}

/// Amplitude (dB) to a height in an area from its bottom.
fn y_of(db: f32, (_, t, _, b): (f32, f32, f32, f32)) -> f32 {
    b - (db / SCOPE_DB).clamp(0.0, 1.0) * (b - t)
}

/// The A-scope: noise, clutter and echoes against range, the range gate on the lock.
fn ascope(
    cv: &mut Canvas,
    echoes: &[Echo],
    target: Option<&Echo>,
    range_m: f64,
    noise: &mut Noise,
) {
    let area @ (l, t, r, b) = ASCOPE;
    // The grid: every 10 dB, every fifth of the range.
    for k in 1..6 {
        let y = y_of(k as f32 * 10.0, area);
        cv.dashed(Vec2::new(l, y), Vec2::new(r, y), 2.0, 4.0, 0.6, DARK);
    }
    for k in 0..=5 {
        let x = l + (r - l) * k as f32 / 5.0;
        cv.stroke(Vec2::new(x, b), Vec2::new(x, b + 3.0), 0.8, GREY);
        let km = range_m / 1000.0 * k as f64 / 5.0;
        stroke_text_at(cv, x, b + 8.0, &format!("{km:.0}"), 5.0, 0.8, GREY, Align::Centre);
    }
    for db in [20, 40, 60] {
        let y = y_of(db as f32, area);
        stroke_text_at(cv, l - 3.0, y, &db.to_string(), 5.0, 0.8, GREY, Align::Right);
    }
    stroke_text_at(cv, r, t - 6.0, "KM", 5.0, 0.8, GREY, Align::Right);
    stroke_text_at(cv, l, t - 6.0, "DB", 5.0, 0.8, GREY, Align::Left);
    cv.frame(Vec2::new(l, t), Vec2::new(r, b), 0.8, GREY);
    // The trace: the column's power, summed, in dB.
    let n = (r - l) as usize;
    let mut pts = Vec::with_capacity(n);
    for k in 0..n {
        let at = range_m * (k as f64 + 0.5) / n as f64;
        // The receiver's noise (0 dB mean), ground clutter falling off over the first 25 km.
        let mut power = 10f64.powf(noise.speckle() as f64 / 10.0);
        let clutter = 42.0 - at / 600.0 + noise.speckle() as f64 * 2.0;
        power += 10f64.powf(clutter.max(-20.0) / 10.0);
        for e in echoes {
            let off = (at - e.range) / PULSE_M;
            if off.abs() < 3.0 {
                power +=
                    10f64.powf((e.db + noise.speckle() as f64 * 0.5) / 10.0) * (-off * off).exp();
            }
        }
        let db = (10.0 * power.log10()) as f32;
        pts.push(Vec2::new(l + k as f32, y_of(db, area)));
    }
    cv.polyline(&pts, 1.0, GREEN);
    // The range gate over the lock's echo, and its strength.
    if let Some(e) = target.filter(|e| e.range <= range_m) {
        let x = l + (e.range / range_m) as f32 * (r - l);
        let y = y_of(e.db as f32, area);
        for s in [-1.0, 1.0] {
            let gx = x + s * 5.0;
            cv.stroke(Vec2::new(gx, t + 2.0), Vec2::new(gx, b - 2.0), 0.8, YELLOW);
            cv.stroke(Vec2::new(gx, t + 2.0), Vec2::new(gx - s * 3.0, t + 2.0), 0.8, YELLOW);
            cv.stroke(Vec2::new(gx, b - 2.0), Vec2::new(gx - s * 3.0, b - 2.0), 0.8, YELLOW);
        }
        stroke_text_at(cv, x, (y - 6.0).max(t + 6.0), "TGT", 5.0, 0.8, YELLOW, Align::Centre);
    }
}

/// The lock's Doppler spectrum: clutter at zero, the target line with its modulation.
fn spectrum(cv: &mut Canvas, e: &Echo, noise: &mut Noise) {
    let area @ (l, t, r, b) = SPECTRUM;
    cv.frame(Vec2::new(l, t), Vec2::new(r, b), 0.8, GREY);
    let mid = (l + r) / 2.0;
    cv.dashed(Vec2::new(mid, t), Vec2::new(mid, b), 2.0, 3.0, 0.6, DARK);
    for k in [-SPAN_KHZ, 0.0, SPAN_KHZ] {
        let x = mid + (k / SPAN_KHZ) as f32 * (r - l) / 2.0;
        stroke_text_at(cv, x, b + 7.0, &format!("{k:+.0}"), 5.0, 0.8, GREY, Align::Centre);
    }
    stroke_text_at(cv, l, t - 6.0, "DOPPLER KHZ", 5.0, 0.8, GREY, Align::Left);
    // Closing targets shift up (a positive Doppler).
    let fd = -2.0 * e.radial / WAVELENGTH / 1000.0;
    // Main beam clutter: the lower the target, the more ground in the beam.
    let clutter = (40.0 - e.alt / 60.0).clamp(6.0, 40.0);
    let snr = e.db.clamp(0.0, 70.0);
    let n = (r - l) as usize;
    let mut pts = Vec::with_capacity(n);
    for k in 0..n {
        let f = ((k as f64 + 0.5) / n as f64 * 2.0 - 1.0) * SPAN_KHZ;
        let mut power = 10f64.powf(noise.speckle() as f64 / 10.0);
        power += 10f64.powf(clutter / 10.0) * (-(f / 0.35).powi(2)).exp();
        // The target line; its blade or rotor modulation either side.
        let line = |off: f64, width: f64, db: f64| {
            10f64.powf(db / 10.0) * (-((f - fd - off) / width).powi(2)).exp()
        };
        power += line(0.0, 0.18, snr);
        if e.unit.is_some() {
            if e.rotary {
                power += line(0.0, 3.0, snr - 18.0);
            } else {
                for (j, off) in [1.6, 3.2].into_iter().enumerate() {
                    let db = snr - 14.0 - 6.0 * j as f64;
                    power += line(off, 0.12, db) + line(-off, 0.12, db);
                }
            }
        }
        pts.push(Vec2::new(l + k as f32, y_of((10.0 * power.log10()) as f32, area)));
    }
    cv.polyline(&pts, 1.0, GREEN);
    if fd.abs() <= SPAN_KHZ {
        let x = mid + (fd / SPAN_KHZ) as f32 * (r - l) / 2.0;
        cv.stroke(Vec2::new(x, t + 1.0), Vec2::new(x, t + 6.0), 1.0, YELLOW);
    }
}

/// The monopulse angle error: the lock in the beam's centre, wandering with the noise.
fn monopulse(cv: &mut Canvas, e: &Echo, noise: &mut Noise) {
    let (l, t, r, b) = ERROR_BOX;
    let c = Vec2::new((l + r) / 2.0, (t + b) / 2.0);
    cv.frame(Vec2::new(l, t), Vec2::new(r, b), 0.8, GREY);
    cv.stroke(Vec2::new(l + 2.0, c.y), Vec2::new(r - 2.0, c.y), 0.6, DARK);
    cv.stroke(Vec2::new(c.x, t + 2.0), Vec2::new(c.x, b - 2.0), 0.6, DARK);
    cv.ring(c, (r - l) / 4.0, 0.6, DARK);
    stroke_text_at(cv, l, t - 5.0, "AZ / EL ERR", 5.0, 0.8, GREY, Align::Left);
    // Thermal noise: the error's spread goes as one over the voltage signal to noise.
    let spread = (r - l) / 2.0 * (10f64.powf(-e.db.max(0.0) / 20.0) * 12.0).min(1.0) as f32;
    let jitter = Vec2::new(noise.next() - 0.5, noise.next() - 0.5) * 2.0 * spread;
    let p = (c + jitter).clamp(Vec2::new(l + 3.0, t + 3.0), Vec2::new(r - 3.0, b - 3.0));
    cv.stroke(p - Vec2::X * 3.0, p + Vec2::X * 3.0, 1.2, YELLOW);
    cv.stroke(p - Vec2::Y * 3.0, p + Vec2::Y * 3.0, 1.2, YELLOW);
}

/// The lock's signal in numbers.
fn numbers(cv: &mut Canvas, e: &Echo, sim: &Sim, radar: usize, mode: Option<Guidance>) {
    let fd = -2.0 * e.radial / WAVELENGTH / 1000.0;
    let in_reach = e.unit.is_some_and(|j| sim.in_reach(radar, j));
    let rows = [
        ("SNR", format!("{:.0} DB", e.db), if e.db >= 13.0 { GREEN } else { YELLOW }),
        ("RNG", format!("{:.1} KM", e.range / 1000.0), if in_reach { GREEN } else { RED }),
        (
            if e.radial <= 0.0 { "CLOS" } else { "OPEN" },
            format!("{:.0} M/S", e.radial.abs()),
            WHITE,
        ),
        ("FD", format!("{fd:+.1} KHZ"), WHITE),
        (
            "MODE",
            if mode == Some(Guidance::SemiActive) { "SEMI" } else { "GUID" }.to_string(),
            CYAN,
        ),
    ];
    for (i, (label, value, col)) in rows.into_iter().enumerate() {
        let y = ERROR_BOX.1 + 3.0 + i as f32 * 16.0;
        stroke_text_at(cv, NUMBERS_X, y, label, 5.0, 0.8, GREY, Align::Left);
        stroke_text_at(cv, NUMBERS_RIGHT, y + 8.0, &value, 5.5, 0.9, col, Align::Right);
    }
}
