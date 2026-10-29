//! The RAH-66's multipurpose displays, the portrait screens outboard of its MFDs: the threat
//! picture over the warning, caution and advisory list on the left one, the engine page (digital
//! readouts over bar graphs) on the right one. New warnings and cautions flash until the master
//! caution is pressed.
//!
//! The threat picture is a search radar scope turned round, after the air defence consoles':
//! the radars painting the helicopter at their range and bearing (heading up), the one whose
//! pulse just hit flashing white with a ring going out from it, the trackers' missile envelopes
//! and their beams to the centre. In the centre the helicopter's own radar return: a bloom that
//! grows with its signature (`Emv::emv`), wide abeam and narrow fore and aft, larger and a little
//! changed in detail with the gear down or the weapon bay doors open, swelling where its belly
//! and top show when it banks or pitches; inside the dotted ring where the strongest signal
//! at it would see it, red for a moment when a pulse does. Under the scope the last 30 seconds
//! of both as a strip chart: the signature filled, the detection line over it, red where it
//! crossed.

use std::collections::VecDeque;
use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::canvas::Canvas;
use crate::flight::Aircraft;
use crate::font::Fonts;
use crate::input::{Action, ActionEvent};
use crate::panel::{Countermeasures, Emv, Warnings};
use crate::rwr::{EmitterView, Emitters};
use stealth_sim::EmitterSource;

/// The displays' canvas: drawing units and texels per unit (the screens are 100 by 140 mm).
pub const MPD_CANVAS: (usize, usize) = (128, 180);
pub const MPD_SCALE: usize = 3;

const BLACK: u8 = 5;
const NAVY: u8 = 2;
const DIM: u8 = 4;
const DARK: u8 = 8;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const CYAN: u8 = 11;
const RED: u8 = 13;
const DEEP_RED: u8 = 16;
const WHITE: u8 = 15;
const AMBER: u8 = 18;

/// Fuel (lbs) below which the low fuel caution shows (the panels' LOW FUEL light's).
const LOW_FUEL: f32 = 1500.0;
/// Fuel flow (lb/h, both engines) at 100% torque (`heli::Rotor::fuel_burn`).
const FLOW_AT_FULL: f32 = 0.3 * 3600.0;

/// How bad an entry of the list is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warning,
    Caution,
    Advisory,
}

/// The engine page's values: rotor speed, and per engine (left, right) torque, turbine gas
/// temperature, gas generator speed, oil pressure and temperature; the fuel and its flow.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EngineReadings {
    /// Rotor speed (%).
    pub nr: f32,
    /// Torque (%).
    pub tq: [f32; 2],
    /// Turbine gas temperature (deg C).
    pub tgt: [f32; 2],
    /// Gas generator speed (%).
    pub ng: [f32; 2],
    /// Oil pressure (psi) and temperature (deg C).
    pub oil_p: [f32; 2],
    pub oil_t: [f32; 2],
    /// Fuel (lbs) and the flow of both engines (lb/h).
    pub fuel: f32,
    pub flow: f32,
}

/// The engines as the helicopter model has them: one torque for both, rotor speed, fuel. An
/// engine hit (`damaged`) shows on the right engine (less torque, hotter), a fire on the left
/// one; no fuel, both run down.
pub fn engine_readings(a: &Aircraft, damaged: bool, fire: bool) -> EngineReadings {
    let running = a.fuel > 0.0;
    let tq = (a.torque() * 100.0).max(0.0);
    let mut r = EngineReadings { nr: a.rotor_speed() * 100.0, fuel: a.fuel.max(0.0), ..default() };
    for e in 0..2 {
        let share = if damaged && e == 1 { 0.6 } else { 1.0 };
        let t = if running { tq * share } else { 0.0 };
        r.tq[e] = t;
        r.ng[e] = if running { 72.0 + 0.27 * t } else { (a.rotor_speed() * 20.0).min(20.0) };
        r.tgt[e] = if running { 420.0 + 3.6 * t } else { 120.0 };
        r.oil_p[e] = if running { 45.0 + 0.3 * t } else { 0.0 };
        r.oil_t[e] = 60.0 + 0.35 * t;
    }
    if damaged {
        r.tgt[1] += 140.0;
        r.oil_p[1] *= 0.5;
    }
    if fire {
        r.tgt[0] = r.tgt[0].max(960.0);
    }
    r.flow = if running { (r.tq[0] + r.tq[1]) / 200.0 * FLOW_AT_FULL } else { 0.0 };
    r
}

/// The warning, caution and advisory list, worst first.
pub fn alerts(
    w: &Warnings,
    a: Option<&Aircraft>,
    cm: &Countermeasures,
) -> Vec<(Level, &'static str)> {
    use Level::*;
    let mut v = Vec::new();
    let mut add = |on: bool, level, text| {
        if on {
            v.push((level, text));
        }
    };
    add(w.fire, Warning, "ENG 1 FIRE");
    add(w.radar_missile, Warning, "RADAR MISSILE");
    add(w.ir_missile, Warning, "IR MISSILE");
    add(a.is_some_and(|a| a.rotor_speed() < 0.9 && !a.on_ground), Warning, "LOW ROTOR RPM");
    add(w.engine, Caution, "ENG 2 DAMAGE");
    add(a.is_some_and(|a| a.fuel < LOW_FUEL), Caution, "LOW FUEL");
    add(w.fuel, Caution, "FUEL SYSTEM");
    add(w.fire_control, Caution, "FLIGHT CONTROLS");
    add(w.avionics, Caution, "AVIONICS");
    add(w.jammer, Caution, "JAMMERS FAIL");
    add(w.missile_warning, Caution, "MWS FAIL");
    add(w.bay_doors, Caution, "BAY DOORS JAMMED");
    add(w.tracked, Caution, "RADAR TRACK");
    add(w.emv_warning, Caution, "DETECTED");
    add(a.is_some_and(|a| a.bay_open) && !w.bay_doors, Advisory, "BAY DOORS OPEN");
    add(a.is_some_and(|a| a.gear_down && !a.on_ground), Advisory, "GEAR DOWN");
    add(a.is_some_and(|a| !a.gear_down), Advisory, "GEAR UP");
    add(a.is_some_and(|a| a.brakes && a.on_ground), Advisory, "PARKING BRAKE");
    add(cm.ecm, Advisory, "ECM ON");
    add(cm.ir_jammer, Advisory, "IR JAMMER ON");
    v.sort_by_key(|e| e.0);
    v
}

/// The master caution light: lit while a warning or caution of `list` is not acknowledged.
pub fn master_caution(list: &[(Level, &'static str)], acked: &[&'static str]) -> bool {
    list.iter().any(|e| e.0 != Level::Advisory && !acked.contains(&e.1))
}

/// The warnings and cautions the master caution has acknowledged (they stop flashing); one
/// that clears and comes back flashes again.
#[derive(Resource, Debug, Default)]
pub struct WcaAck(pub Vec<&'static str>);

pub(crate) fn acknowledge(
    mut actions: MessageReader<ActionEvent>,
    mut ack: ResMut<WcaAck>,
    warnings: Res<Warnings>,
    cm: Res<Countermeasures>,
    aircraft: Query<&Aircraft, With<crate::flight::Player>>,
) {
    let now = alerts(&warnings, aircraft.single().ok(), &cm);
    ack.0.retain(|t| now.iter().any(|e| e.1 == *t));
    if actions.read().any(|e| e.0 == Action::MasterCaution) {
        ack.0 = now.into_iter().map(|e| e.1).collect();
    }
}

/// The threat page's strip chart: the signature and the strongest radar signal at the helicopter
/// (fractions of the gauge's full scale), one sample every `SAMPLE_S` seconds, oldest first;
/// and the radar whose pulse hit last, with the seconds since and whether it saw the helicopter.
#[derive(Resource, Debug, Default)]
pub struct ThreatHistory {
    pub samples: VecDeque<(f32, f32)>,
    clock: f32,
    pub pulse: Option<(EmitterSource, f32, bool)>,
    /// The pulse display's frames left last time (a rise is a new pulse).
    pulse_frames: i32,
}

/// Seconds between the chart's samples and how many it keeps (30 seconds, a sample per texel
/// column of the chart).
const SAMPLE_S: f32 = 30.0 / HISTORY as f32;
const HISTORY: usize = 360;
/// Seconds a radar flashes after its pulse hits, and the bloom stays red after one saw us.
const PULSE_S: f32 = 0.8;
const DETECT_S: f32 = 1.5;
/// The scope's ranges (km): the shortest that holds the farthest radar.
const RANGES: [f32; 5] = [5.0, 10.0, 20.0, 40.0, 80.0];

pub(crate) fn sample_threats(
    time: Res<Time>,
    emv: Res<Emv>,
    emitters: Res<Emitters>,
    mut h: ResMut<ThreatHistory>,
) {
    let dt = time.delta_secs();
    if let Some((_, age, _)) = &mut h.pulse {
        *age += dt;
    }
    match emitters.pulse {
        Some((source, frames, detects)) => {
            let new = frames > h.pulse_frames || h.pulse.is_none_or(|p| p.0 != source);
            if new {
                h.pulse = Some((source, 0.0, detects));
            }
            h.pulse_frames = frames;
        }
        None => h.pulse_frames = 0,
    }
    h.clock += dt;
    while h.clock >= SAMPLE_S {
        h.clock -= SAMPLE_S;
        if h.samples.len() == HISTORY {
            h.samples.pop_front();
        }
        h.samples.push_back((emv.emv, emv.egr.max(emv.ear)));
    }
}

/// What the threat picture shows.
pub struct Threats<'a> {
    /// The radars painting the helicopter, most dangerous first.
    pub emitters: &'a [EmitterView],
    pub emv: &'a Emv,
    pub history: &'a ThreatHistory,
    /// The weapon bay doors are open (the signature's jump; broad lobes abeam, where they open).
    pub bay_open: bool,
    pub gear_down: bool,
    /// Pitch and roll (radians, nose up and right wing down positive).
    pub pitch: f32,
    pub roll: f32,
}

/// The left display: the threat picture (`draw_threats`) on top, the warning list
/// under it (warnings red, cautions amber, advisories green; unacknowledged warnings and
/// cautions flash (`blink`), inverted), the countermeasures left at the foot.
#[allow(clippy::too_many_arguments)]
pub fn draw_left(
    c: &mut Canvas,
    fonts: &Fonts,
    threats: &Threats,
    list: &[(Level, &'static str)],
    acked: &[&'static str],
    blink: bool,
    cm: &Countermeasures,
) {
    let (w, h) = (c.w as i32, c.h as i32);
    c.clear(BLACK);
    draw_threats(c, fonts, threats, blink);
    // The list: as many as fit, the last line counting the rest.
    c.hline(4, w - 5, 128, GREY);
    const ROWS: usize = 4;
    let mut y = 131;
    for (k, &(level, text)) in list.iter().enumerate().take(ROWS) {
        if k == ROWS - 1 && list.len() > ROWS {
            let more = format!("+{} MORE", list.len() - k);
            fonts.text(c, 8, y, more.as_bytes(), GREY);
            break;
        }
        let colour = match level {
            Level::Warning => RED,
            Level::Caution => AMBER,
            Level::Advisory => GREEN,
        };
        let flash = level != Level::Advisory && !acked.contains(&text);
        if flash && blink {
            c.fill_rect(4, y - 1, w - 8, 7, colour);
            fonts.text(c, 8, y, text.as_bytes(), BLACK);
        } else {
            fonts.text(c, 8, y, text.as_bytes(), colour);
        }
        y += 7;
    }
    if list.is_empty() {
        fonts.text(c, 8, y, b"NO WARNINGS", GREY);
    }
    // The countermeasures left.
    c.hline(4, w - 5, h - 22, GREY);
    let line = format!("CHF {:>2}  FLR {:>2}  DCY {}", cm.chaff, cm.flares, cm.decoys);
    fonts.text(c, 6, h - 16, line.as_bytes(), CYAN);
}

/// The scope (centred at the top) and the strip chart under it.
fn draw_threats(c: &mut Canvas, fonts: &Fonts, th: &Threats, blink: bool) {
    let w = c.w as f32;
    let centre = Vec2::new(w / 2.0, 55.0);
    let r = 48.0;
    // Where a bearing (radians clockwise from the nose) and a fraction of the radius land.
    let at = |b: f32, f: f32| centre + Vec2::new(b.sin(), -b.cos()) * r * f;
    let far = th.emitters.iter().map(|e| e.range_km).fold(0.0, f32::max);
    let range = RANGES.into_iter().find(|&k| far <= k * 0.95).unwrap_or(RANGES[4]);
    // Range rings, the bearing ticks every 30 degrees, the nose mark.
    for k in 1..=3 {
        c.ring(centre, r * k as f32 / 3.0, 0.4, if k == 3 { GREY } else { DARK });
    }
    for k in 0..12 {
        let b = k as f32 * TAU / 12.0;
        let inner = if k % 3 == 0 { 0.9 } else { 0.95 };
        c.stroke(at(b, inner), at(b, 1.0), 0.4, GREY);
    }
    c.triangle([at(0.0, 1.02), at(-0.05, 1.1), at(0.05, 1.1)], WHITE);
    // The bloom: our return, filled dim under the contacts; the dotted ring is the size at
    // which the strongest signal at us would see it.
    let (emv, signal) = (th.emv.emv, th.emv.egr.max(th.emv.ear));
    let size = |v: f32| (2.5 + v.clamp(0.0, 1.0) * 18.0) * r / 40.0;
    let base = size(emv);
    let detected = th.history.pulse.is_some_and(|(_, age, detects)| detects && age < DETECT_S);
    let pts: Vec<Vec2> = (0..=48)
        .map(|i| {
            let a = i as f32 * TAU / 48.0;
            centre + Vec2::new(a.sin(), -a.cos()) * (base * lobes(a, th)).min(r)
        })
        .collect();
    let (fill, line) = if detected { (DEEP_RED, RED) } else { (NAVY, CYAN) };
    for p in pts.windows(2) {
        c.triangle([centre, p[0], p[1]], fill);
    }
    c.polyline(&pts, 0.5, line);
    if signal > 0.0 {
        let limit = size(1.0 - signal);
        let n = 36;
        for i in (0..n).step_by(2) {
            let (a0, a1) = (i as f32 * TAU / n as f32, (i as f32 + 0.6) * TAU / n as f32);
            c.arc(centre, limit, a0, a1, 0.4, line);
        }
    }
    // The trackers' missile envelopes (inside the scope) and their beams at the helicopter.
    let place = |e: &EmitterView| (e.range_km / range).min(1.0);
    for e in th.emitters.iter().filter(|e| e.tracking || e.launch) {
        let p = at(e.rel_bearing, place(e));
        let col = if e.launch { RED } else { AMBER };
        ring_inside(c, p, e.lock_range_km / range * r, centre, r, col);
        c.dashed(p, centre, 2.0, 2.0, 0.4, col);
    }
    // The helicopter.
    c.stroke(centre + Vec2::new(0.0, -2.5), centre + Vec2::new(0.0, 2.5), 0.6, WHITE);
    c.stroke(centre + Vec2::new(-2.0, -0.5), centre + Vec2::new(2.0, -0.5), 0.6, WHITE);
    // The radars: a strobe on the rim at each one's bearing (longer the more dangerous), its
    // symbol at its range (on the rim when out of range), green when heard lately; the one whose
    // pulse just hit white, a ring going out from it and its beam to the centre.
    for e in th.emitters.iter().rev() {
        let len = 0.04 + 0.12 * (e.priority as f32 / 520.0).clamp(0.0, 1.0);
        let pulse = th
            .history
            .pulse
            .filter(|&(source, age, _)| source == e.source && age < PULSE_S)
            .map(|p| p.1 / PULSE_S);
        let col = if pulse.is_some() {
            WHITE
        } else if e.launch {
            if blink { RED } else { WHITE }
        } else if e.tracking {
            AMBER
        } else if e.heard {
            GREEN
        } else {
            DIM
        };
        c.stroke(at(e.rel_bearing, 1.0), at(e.rel_bearing, 1.0 - len), 0.6, col);
        let p = at(e.rel_bearing, place(e));
        if let Some(f) = pulse {
            c.ring(p, 3.5 + 9.0 * f, 0.5, if f < 0.5 { WHITE } else { GREY });
            c.stroke(p, centre, 0.5, if f < 0.5 { WHITE } else { GREY });
        }
        let s = if pulse.is_some() { 3.2 } else { 2.6 };
        if e.air {
            c.polyline(
                &[p + Vec2::new(-s, s * 0.6), p + Vec2::new(0.0, -s), p + Vec2::new(s, s * 0.6)],
                0.5,
                col,
            );
        } else {
            let d = [Vec2::new(0.0, -s), Vec2::new(s, 0.0), Vec2::new(0.0, s), Vec2::new(-s, 0.0)];
            c.polyline(&[p + d[0], p + d[1], p + d[2], p + d[3], p + d[0]], 0.5, col);
        }
        if e.tracking || e.launch || e.heard || pulse.is_some() {
            fonts.text(c, p.x as i32 + 3, p.y as i32 - 6, e.label.as_bytes(), col);
        }
    }
    // The corners: the range, the radars heard, our signature and the strongest signal.
    let (wi, top, bottom) = (c.w as i32, 3, 92);
    fonts.text(c, 4, top, format!("{range:.0}KM").as_bytes(), GREY);
    let n = format!("{} RDR", th.emitters.len());
    fonts.text(c, wi - 4 - fonts.width(n.as_bytes()), top, n.as_bytes(), GREY);
    let pct = |v: f32| (v.clamp(0.0, 1.0) * 99.0).round() as i32;
    fonts.text(c, 4, bottom, b"RCS", CYAN);
    let v = format!("{:>2}", pct(emv));
    let x = fonts.text(c, 4, bottom + 7, v.as_bytes(), if detected { RED } else { CYAN });
    if th.bay_open {
        fonts.text(c, x + 3, bottom + 7, b"BAY", AMBER);
    }
    let sig = b"SIG";
    fonts.text(c, wi - 4 - fonts.width(sig), bottom, sig, AMBER);
    let v = format!("{:>2}", pct(signal));
    fonts.text(c, wi - 4 - fonts.width(v.as_bytes()), bottom + 7, v.as_bytes(), AMBER);
    strip_chart(c, th.history, 4, 107, wi - 8, 18);
}

/// The bloom's radius at bearing `a` (radians clockwise from the nose) in its base size. The
/// airframe's: broad abeam, narrow fore and aft, with small spikes where its edges are aligned.
/// The gear down and the open bays make it larger through the signature, and both add detail: small spikes beside the nose and the tail for the gear, either side of
/// the beam for the bays. Banked, the belly and the top swell it abeam (a little more towards
/// the low side); pitched, fore and aft (a little more where the belly faces), never past the
/// beam. Nearly symmetrical, and steady while those stay.
fn lobes(a: f32, th: &Threats) -> f32 {
    let lobe = |dir: f32, width: f32| {
        let d = ((a - dir + TAU / 2.0).rem_euclid(TAU) - TAU / 2.0) / width;
        (-d * d).exp()
    };
    let q = TAU / 4.0;
    let mut k = 0.65 + 0.55 * a.sin().powi(2);
    for dir in [0.7, -0.7, 2.45, -2.45] {
        k += 0.15 * lobe(dir, 0.08);
    }
    for dir in [0.0, q, -q, 2.0 * q] {
        k += 0.08 * lobe(dir, 0.12);
    }
    if th.bay_open {
        for dir in [q - 0.3, q + 0.3, -q - 0.3, -q + 0.3] {
            k += 0.12 * lobe(dir, 0.1);
        }
    }
    if th.gear_down {
        for dir in [0.35, -0.35, 2.0 * q - 0.35, 0.35 - 2.0 * q] {
            k += 0.1 * lobe(dir, 0.12);
        }
    }
    // The top shows to the other side about as much as the belly does.
    let bank = (th.roll.sin().abs() * 1.6).min(0.9) * 0.4;
    let (low, high) = if th.roll > 0.0 { (-q, q) } else { (q, -q) };
    k += bank * (lobe(low, 0.6) + 0.85 * lobe(high, 0.6));
    let pitch = (th.pitch.sin().abs() * 1.6).min(0.9) * 0.3;
    let (belly, top) = if th.pitch > 0.0 { (0.0, 2.0 * q) } else { (2.0 * q, 0.0) };
    k += pitch * (lobe(belly, 0.6) + 0.85 * lobe(top, 0.6));
    k
}

/// A ring of radius `r` about `p`, only where it lies inside the scope (`centre`, `scope`).
fn ring_inside(c: &mut Canvas, p: Vec2, r: f32, centre: Vec2, scope: f32, col: u8) {
    let n = ((r * 0.8) as usize).clamp(16, 96);
    let pt = |i: usize| p + Vec2::from_angle(i as f32 * TAU / n as f32) * r;
    for i in 0..n {
        let (a, b) = (pt(i), pt(i + 1));
        if i % 2 == 0 && a.distance(centre) < scope && b.distance(centre) < scope {
            c.stroke(a, b, 0.4, col);
        }
    }
}

/// The last 30 seconds right aligned in the box (x, y, w, h), drawn in texels (one sample per
/// texel column): the signature filled from the bottom, the detection line (where the signal
/// would see it) over it; columns past the line red.
fn strip_chart(c: &mut Canvas, h: &ThreatHistory, x: i32, y: i32, w: i32, ht: i32) {
    c.rect(x, y, w, ht, DARK);
    let k = c.scale as i32;
    let pw = c.pw();
    let (x0, cols) = ((x + 1) * k, (w - 2) * k);
    let (bottom, rows) = ((y + ht - 1) * k - 1, (ht - 2) * k);
    let mut put = |tx: i32, ty: i32, col: u8| c.px[ty as usize * pw + tx as usize] = col;
    let level = |v: f32| (v.clamp(0.0, 1.0) * rows as f32).round() as i32;
    let n = h.samples.len() as i32;
    for (i, &(emv, signal)) in h.samples.iter().enumerate() {
        // The newest at the right edge.
        let tx = x0 + cols - (n - i as i32);
        if tx < x0 {
            continue;
        }
        let e = level(emv);
        let crossed = signal > 0.0 && emv + signal > 1.0;
        for d in 0..e {
            put(tx, bottom - d, if crossed { RED } else { DIM });
        }
        if e > 0 {
            for d in e - 2..e {
                put(tx, bottom - d.max(0), if crossed { RED } else { CYAN });
            }
        }
        if signal > 0.0 {
            let l = level(1.0 - signal).min(rows - 1);
            for d in l - 1..=l {
                put(tx, bottom - d.max(0), AMBER);
            }
        }
    }
}

/// The right display: rotor speed and the engines' torque as bar graphs with their readouts
/// over them, the temperatures, gas generator speeds and oil as readouts with short bars, the
/// fuel and its flow at the foot. Green in the normal range, amber over it, red past the
/// limits.
pub fn draw_engines(c: &mut Canvas, fonts: &Fonts, r: &EngineReadings) {
    let (w, h) = (c.w as i32, c.h as i32);
    c.clear(BLACK);
    title(c, fonts, "ENGINE");
    // Tall bars: NR (90-110% normal), TQ 1 and 2 (100% continuous, 105% the limit).
    let bars = [
        ("NR", r.nr, 120.0, colour(r.nr, 90.0, 104.0, 110.0, true)),
        ("TQ1", r.tq[0], 120.0, colour(r.tq[0], 0.0, 100.0, 105.0, false)),
        ("TQ2", r.tq[1], 120.0, colour(r.tq[1], 0.0, 100.0, 105.0, false)),
    ];
    let (top, bottom) = (36, 98);
    for (k, &(name, value, max, col)) in bars.iter().enumerate() {
        let cx = 22 + k as i32 * 42;
        let readout = format!("{value:.0}");
        let rw = fonts.width(readout.as_bytes());
        c.rect(cx - 13, 19, 26, 11, col);
        fonts.text(c, cx - rw / 2, 21, readout.as_bytes(), WHITE);
        bar_v(c, cx - 6, top, 12, bottom - top, value / max, col);
        // The 100% mark.
        let y100 = bottom - ((100.0 / max) * (bottom - top) as f32) as i32;
        c.hline(cx - 9, cx + 8, y100, WHITE);
        let nw = fonts.width(name.as_bytes());
        fonts.text(c, cx - nw / 2, bottom + 3, name.as_bytes(), GREY);
    }
    // Readout rows: both engines side by side with a short bar each.
    // (name, values, full scale, amber past, red past)
    let rows = [
        ("TGT", r.tgt, 1000.0, 810.0, 870.0),
        ("NG", r.ng, 110.0, 103.0, 106.0),
        ("OIL P", r.oil_p, 120.0, 90.0, 100.0),
        ("OIL T", r.oil_t, 160.0, 120.0, 140.0),
    ];
    let mut y = 114;
    for (name, v, max, amber, red) in rows {
        fonts.text(c, 4, y, name.as_bytes(), GREY);
        for e in 0..2 {
            let x = 40 + e as i32 * 44;
            let low = name == "OIL P" && v[e] < 25.0;
            let col = if low { RED } else { colour(v[e], 0.0, amber, red, false) };
            let text = format!("{:>4.0}", v[e]);
            fonts.text(c, x, y, text.as_bytes(), col);
            bar_h(c, x + 18, y + 1, 20, 5, v[e] / max, col);
        }
        y += 11;
    }
    // Fuel.
    c.hline(4, w - 5, h - 26, GREY);
    let fuel = format!("FUEL {:>5.0} LB", r.fuel);
    let col = if r.fuel < LOW_FUEL { AMBER } else { GREEN };
    fonts.text(c, 6, h - 22, fuel.as_bytes(), col);
    let flow = format!("FLOW {:>5.0} LB/H", r.flow);
    fonts.text(c, 6, h - 12, flow.as_bytes(), CYAN);
}

/// The page title, centred over a rule.
fn title(c: &mut Canvas, fonts: &Fonts, text: &str) {
    let w = c.w as i32;
    let tw = fonts.width(text.as_bytes());
    fonts.text(c, (w - tw) / 2, 4, text.as_bytes(), CYAN);
    c.hline(4, w - 5, 14, GREY);
}

/// Green in the normal range, amber past `amber`, red past `red` (or under `low` when `low_red`).
fn colour(v: f32, low: f32, amber: f32, red: f32, low_red: bool) -> u8 {
    if v > red || (low_red && v < low) {
        RED
    } else if v > amber {
        AMBER
    } else {
        GREEN
    }
}

/// A vertical bar filled `frac` from the bottom, in its frame.
fn bar_v(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, frac: f32, col: u8) {
    c.rect(x - 1, y - 1, w + 2, h + 2, GREY);
    let f = (frac.clamp(0.0, 1.0) * h as f32) as i32;
    c.fill_rect(x, y + h - f, w, f, col);
}

/// A horizontal bar filled `frac` from the left, in its frame.
fn bar_h(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, frac: f32, col: u8) {
    c.rect(x - 1, y - 1, w + 2, h + 2, DARK);
    let f = (frac.clamp(0.0, 1.0) * w as f32) as i32;
    c.fill_rect(x, y, f, h, col);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hovering() -> Aircraft {
        let mut a = Aircraft::new_on_runway(2000.0);
        a.on_ground = false;
        a.set_rotor(1.0, 0.6);
        a
    }

    #[test]
    fn engines_follow_the_torque() {
        let a = hovering();
        let r = engine_readings(&a, false, false);
        assert_eq!(r.nr, 100.0);
        assert!(r.tq.iter().all(|t| (t - 60.0).abs() < 0.01), "{:?}", r.tq);
        assert!(r.tgt[0] > 500.0 && r.tgt[0] < 810.0);
        assert!((r.flow - 0.6 * FLOW_AT_FULL).abs() < 1.0);
        // An engine hit: the right engine gives less and runs hot; no fuel, both run down.
        let hit = engine_readings(&a, true, false);
        assert!(hit.tq[1] < hit.tq[0] && hit.tgt[1] > hit.tgt[0]);
        let mut dry = a.clone();
        dry.fuel = 0.0;
        let out = engine_readings(&dry, false, false);
        assert_eq!((out.tq, out.flow), ([0.0, 0.0], 0.0));
    }

    #[test]
    fn alerts_worst_first() {
        let w = Warnings { fire: true, tracked: true, ..default() };
        let mut a = hovering();
        a.fuel = 1000.0;
        a.gear_down = false;
        let list = alerts(&w, Some(&a), &Countermeasures::default());
        let texts: Vec<_> = list.iter().map(|e| e.1).collect();
        assert_eq!(texts, ["ENG 1 FIRE", "LOW FUEL", "RADAR TRACK", "GEAR UP"]);
        assert!(list.windows(2).all(|p| p[0].0 <= p[1].0));
    }

    #[test]
    fn master_caution_until_acknowledged() {
        let mut a = hovering();
        a.gear_down = false;
        let cm = Countermeasures::default();
        // Advisories alone leave it dark; a launch lights it until acknowledged.
        assert!(!master_caution(&alerts(&Warnings::default(), Some(&a), &cm), &[]));
        let w = Warnings { radar_missile: true, ..default() };
        let list = alerts(&w, Some(&a), &cm);
        assert!(master_caution(&list, &[]));
        let acked: Vec<_> = list.iter().map(|e| e.1).collect();
        assert!(!master_caution(&list, &acked));
    }
}
