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
//!
//! The scope's parts (`draw_scope`, `bloom`, `strip_chart`) also draw the modern MFDs' RCS page
//! (`avionics::draw_rcs`).

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
    /// Bingo fuel (lbs), below which the fuel readout is amber.
    pub bingo: f32,
}

/// The engines as the helicopter model has them: one torque for both, rotor speed, fuel and its
/// flow (`burn`: lb/s at 100% torque, `heli::Rotor::fuel_burn`). An
/// engine hit (`damaged`) shows on the right engine (less torque, hotter), a fire on the left
/// one; no fuel, both run down.
pub fn engine_readings(a: &Aircraft, burn: f32, damaged: bool, fire: bool) -> EngineReadings {
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
    r.flow = if running { (r.tq[0] + r.tq[1]) / 200.0 * burn * 3600.0 } else { 0.0 };
    r
}

/// The warning, caution and advisory list, worst first (LOW FUEL below `bingo` lbs).
pub fn alerts(
    w: &Warnings,
    a: Option<&Aircraft>,
    cm: &Countermeasures,
    bingo: f32,
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
    add(a.is_some_and(|a| a.fuel < bingo), Caution, "LOW FUEL");
    add(w.fuel, Caution, "FUEL SYSTEM");
    add(w.fire_control, Caution, "FLIGHT CONTROLS");
    add(w.avionics, Caution, "AVIONICS");
    add(w.jammer, Caution, "JAMMERS FAIL");
    add(w.missile_warning, Caution, "MWS FAIL");
    add(w.bay_doors, Caution, "BAY DOORS JAMMED");
    add(w.radar_track, Caution, RADAR_TRACK);
    add(w.emv_warning, Caution, DETECTED);
    add(a.is_some_and(|a| a.bay_open) && !w.bay_doors, Advisory, "BAY DOORS OPEN");
    add(a.is_some_and(|a| a.gear_down && !a.on_ground), Advisory, "GEAR DOWN");
    add(a.is_some_and(|a| !a.gear_down), Advisory, "GEAR UP");
    add(a.is_some_and(|a| a.brakes && a.on_ground), Advisory, "PARKING BRAKE");
    add(cm.ecm, Advisory, "ECM ON");
    add(cm.ir_jammer, Advisory, "IR JAMMER ON");
    v.sort_by_key(|e| e.0);
    v
}

/// The cautions of being seen by radar: on the list, but they leave the master caution dark.
const RADAR_TRACK: &str = "RADAR TRACK";
const DETECTED: &str = "DETECTED";

/// The master caution light: lit while a warning or caution of `list` is not acknowledged (but
/// for a radar track or a detection).
pub fn master_caution(list: &[(Level, &'static str)], acked: &[&'static str]) -> bool {
    list.iter().any(|e| {
        e.0 != Level::Advisory && ![RADAR_TRACK, DETECTED].contains(&e.1) && !acked.contains(&e.1)
    })
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
    settings: Res<crate::settings::Settings>,
) {
    let bingo = settings.aircraft.spec().flight().bingo;
    let now = alerts(&warnings, aircraft.single().ok(), &cm, bingo);
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
pub const SAMPLE_S: f32 = 30.0 / HISTORY as f32;
pub const HISTORY: usize = 360;
/// Seconds a radar flashes after its pulse hits, and the bloom stays red after one saw us.
const PULSE_S: f32 = 0.8;
const DETECT_S: f32 = 1.5;
/// The scope's ranges (km): the shortest that holds the farthest radar.
pub const RANGES: [f32; 5] = [5.0, 10.0, 20.0, 40.0, 80.0];

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
    let st = ScopeStyle::MPD;
    let range = scope_range(th.emitters);
    draw_scope(c, th, &st, range, blink, |c, p, label, col| {
        fonts.text(c, p.x as i32 + 3, p.y as i32 - 6, label.as_bytes(), col);
    });
    // The corners: the range, the radars heard, our signature and the strongest signal.
    let (emv, signal) = (th.emv.emv, signal(th.emv));
    let detected = detected(th.history);
    let (wi, top, bottom) = (c.w as i32, 3, 92);
    fonts.text(c, 4, top, format!("{range:.0}KM").as_bytes(), GREY);
    let n = format!("{} RDR", th.emitters.len());
    fonts.text(c, wi - 4 - fonts.width(n.as_bytes()), top, n.as_bytes(), GREY);
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

/// A signature or signal (0..1) as the displays' percentage (0..99).
pub fn pct(v: f32) -> i32 {
    (v.clamp(0.0, 1.0) * 99.0).round() as i32
}

/// The strongest radar signal at the aircraft (ground or air).
pub fn signal(emv: &Emv) -> f32 {
    emv.egr.max(emv.ear)
}

/// A radar's pulse saw the aircraft lately (the bloom shows red).
pub fn detected(h: &ThreatHistory) -> bool {
    h.pulse.is_some_and(|(_, age, detects)| detects && age < DETECT_S)
}

/// The scope's range (km): the shortest of `RANGES` that holds the farthest radar.
pub fn scope_range(emitters: &[EmitterView]) -> f32 {
    let far = emitters.iter().map(|e| e.range_km).fold(0.0, f32::max);
    RANGES.into_iter().find(|&k| far <= k * 0.95).unwrap_or(RANGES[RANGES.len() - 1])
}

/// The bloom's base radius on a scope of radius `r` for a signature `v` (0..1); the detection
/// ring of a signal `s` is the size of the signature `1 - s`.
pub fn bloom_size(v: f32, r: f32) -> f32 {
    (2.5 + v.clamp(0.0, 1.0) * 18.0) * r / 40.0
}

/// The bloom's outline about `centre`, `n` steps round from the nose: `base` (`bloom_size`)
/// times `lobes`, no farther out than `max`.
pub fn bloom(centre: Vec2, base: f32, max: f32, th: &Threats, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let a = i as f32 * TAU / n as f32;
            centre + Vec2::new(a.sin(), -a.cos()) * (base * lobes(a, th)).min(max)
        })
        .collect()
}

/// Fills a bloom outline (`bloom`) from its centre.
pub fn fill_bloom(c: &mut Canvas, centre: Vec2, pts: &[Vec2], col: u8) {
    for p in pts.windows(2) {
        c.triangle([centre, p[0], p[1]], col);
    }
}

/// A dotted ring (18 dots).
pub fn dotted_ring(c: &mut Canvas, centre: Vec2, r: f32, w: f32, col: u8) {
    let n = 36;
    for i in (0..n).step_by(2) {
        let (a0, a1) = (i as f32 * TAU / n as f32, (i as f32 + 0.6) * TAU / n as f32);
        c.arc(centre, r, a0, a1, w, col);
    }
}

/// How far a radar's flash after its pulse has gone (0..1; None: not flashing).
pub fn pulse_of(h: &ThreatHistory, source: EmitterSource) -> Option<f32> {
    h.pulse.filter(|&(s, age, _)| s == source && age < PULSE_S).map(|p| p.1 / PULSE_S)
}

/// Where and how big `draw_scope` draws (drawing units): the scope's centre and radius, its
/// thin pen (rings, ticks, beams), the outline pen (bloom, symbols), the bold one (strobes, the
/// aircraft) and the radars' symbol size; `received_only`: only the radars whose signal reaches
/// us now, a searching one as its strobe and beam (the RCS page), not every radar heard lately.
#[derive(Debug, Clone, Copy)]
pub struct ScopeStyle {
    pub centre: Vec2,
    pub r: f32,
    pub thin: f32,
    pub pen: f32,
    pub bold: f32,
    pub sym: f32,
    pub received_only: bool,
}

impl ScopeStyle {
    /// The multipurpose display's (RAH-66, F-22A).
    pub const MPD: ScopeStyle = ScopeStyle {
        centre: Vec2::new(MPD_CANVAS.0 as f32 / 2.0, 55.0),
        r: 48.0,
        thin: 0.4,
        pen: 0.5,
        bold: 0.6,
        sym: 2.6,
        received_only: false,
    };
}

/// The threat scope, heading up, `range` km to the rim: range rings, bearing ticks and the nose
/// mark; the bloom with the dotted ring where the strongest signal would see it (red for a while
/// after a pulse did); the trackers' missile envelopes and beams, the aircraft, and the radars
/// (strobes on the rim, symbols at their range, `label` writing a symbol's text next to it).
pub fn draw_scope(
    c: &mut Canvas,
    th: &Threats,
    st: &ScopeStyle,
    range: f32,
    blink: bool,
    mut label: impl FnMut(&mut Canvas, Vec2, &str, u8),
) {
    let (centre, r) = (st.centre, st.r);
    let k = st.sym / 2.6;
    // Where a bearing (radians clockwise from the nose) and a fraction of the radius land.
    let at = |b: f32, f: f32| centre + Vec2::new(b.sin(), -b.cos()) * r * f;
    // Range rings, the bearing ticks every 30 degrees, the nose mark.
    for k in 1..=3 {
        c.ring(centre, r * k as f32 / 3.0, st.thin, if k == 3 { GREY } else { DARK });
    }
    for k in 0..12 {
        let b = k as f32 * TAU / 12.0;
        let inner = if k % 3 == 0 { 0.9 } else { 0.95 };
        c.stroke(at(b, inner), at(b, 1.0), st.thin, GREY);
    }
    c.triangle([at(0.0, 1.02), at(-0.05, 1.1), at(0.05, 1.1)], WHITE);
    // The bloom: our return, filled dim under the contacts; the dotted ring is the size at
    // which the strongest signal at us would see it.
    let (emv, signal) = (th.emv.emv, signal(th.emv));
    let detected = detected(th.history);
    let pts = bloom(centre, bloom_size(emv, r), r, th, 48);
    let (fill, line) = if detected { (DEEP_RED, RED) } else { (NAVY, CYAN) };
    fill_bloom(c, centre, &pts, fill);
    c.polyline(&pts, st.pen, line);
    if signal > 0.0 {
        dotted_ring(c, centre, bloom_size(1.0 - signal, r), st.thin, line);
    }
    // The trackers' missile envelopes (inside the scope) and their beams at the aircraft.
    let place = |e: &EmitterView| (e.range_km / range).min(1.0);
    for e in th.emitters.iter().filter(|e| e.tracking || e.launch) {
        let p = at(e.rel_bearing, place(e));
        let col = if e.launch { RED } else { AMBER };
        ring_inside(c, p, e.lock_range_km / range * r, centre, r, st.thin, col);
        c.dashed(p, centre, 2.0 * k, 2.0 * k, st.thin, col);
    }
    // The aircraft.
    let (a, b) = (Vec2::new(0.0, -2.5) * k, Vec2::new(0.0, 2.5) * k);
    c.stroke(centre + a, centre + b, st.bold, WHITE);
    let (a, b) = (Vec2::new(-2.0, -0.5) * k, Vec2::new(2.0, -0.5) * k);
    c.stroke(centre + a, centre + b, st.bold, WHITE);
    // The radars: a strobe on the rim at each one's bearing (longer the more dangerous), its
    // symbol at its range (on the rim when out of range), green when heard lately; the one whose
    // pulse just hit white, a ring going out from it and its beam to the centre.
    for e in th.emitters.iter().rev() {
        let len = 0.04 + 0.12 * (e.priority as f32 / 520.0).clamp(0.0, 1.0);
        let pulse = pulse_of(th.history, e.source);
        let fights = e.tracking || e.launch;
        if st.received_only && !(fights || e.receiving || pulse.is_some()) {
            continue;
        }
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
        c.stroke(at(e.rel_bearing, 1.0), at(e.rel_bearing, 1.0 - len), st.bold, col);
        let p = at(e.rel_bearing, place(e));
        if let Some(f) = pulse {
            c.ring(p, (3.5 + 9.0 * f) * k, st.pen, if f < 0.5 { WHITE } else { GREY });
            c.stroke(p, centre, st.pen, if f < 0.5 { WHITE } else { GREY });
        }
        // Received only: a searching radar is its strobe and a faint beam at us.
        if st.received_only && !fights && pulse.is_none() {
            c.dashed(p, centre, 1.0 * k, 3.0 * k, st.thin, DARK);
            continue;
        }
        let s = if pulse.is_some() { 3.2 * k } else { st.sym };
        if e.air {
            c.polyline(
                &[p + Vec2::new(-s, s * 0.6), p + Vec2::new(0.0, -s), p + Vec2::new(s, s * 0.6)],
                st.pen,
                col,
            );
        } else {
            let d = [Vec2::new(0.0, -s), Vec2::new(s, 0.0), Vec2::new(0.0, s), Vec2::new(-s, 0.0)];
            c.polyline(&[p + d[0], p + d[1], p + d[2], p + d[3], p + d[0]], st.pen, col);
        }
        if e.tracking || e.launch || e.heard || pulse.is_some() {
            label(c, p, &e.label, col);
        }
    }
}

/// The bloom's radius at bearing `a` (radians clockwise from the nose) in its base size: the
/// simulation's aspect lobes (`stealth_sim::aspect_lobes`), the same the radars see.
pub fn lobes(a: f32, th: &Threats) -> f32 {
    let pose = stealth_sim::Pose {
        gear_down: th.gear_down,
        bay_open: th.bay_open,
        pitch: th.pitch,
        roll: th.roll,
    };
    stealth_sim::aspect_lobes(a, &pose)
}

/// A ring of radius `r` about `p`, only where it lies inside the scope (`centre`, `scope`).
fn ring_inside(c: &mut Canvas, p: Vec2, r: f32, centre: Vec2, scope: f32, w: f32, col: u8) {
    let n = ((r * 0.8) as usize).clamp(16, 96);
    let pt = |i: usize| p + Vec2::from_angle(i as f32 * TAU / n as f32) * r;
    for i in 0..n {
        let (a, b) = (pt(i), pt(i + 1));
        if i % 2 == 0 && a.distance(centre) < scope && b.distance(centre) < scope {
            c.stroke(a, b, w, col);
        }
    }
}

/// The last 30 seconds right aligned in the box (x, y, w, h), drawn in texels (one sample per
/// texel column): the signature filled from the bottom, the detection line (where the signal
/// would see it) over it; columns past the line red.
pub fn strip_chart(c: &mut Canvas, h: &ThreatHistory, x: i32, y: i32, w: i32, ht: i32) {
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
        for (e, &ve) in v.iter().enumerate() {
            let x = 40 + e as i32 * 44;
            let low = name == "OIL P" && ve < 25.0;
            let col = if low { RED } else { colour(ve, 0.0, amber, red, false) };
            let text = format!("{:>4.0}", ve);
            fonts.text(c, x, y, text.as_bytes(), col);
            bar_h(c, x + 18, y + 1, 20, 5, ve / max, col);
        }
        y += 11;
    }
    // Fuel.
    c.hline(4, w - 5, h - 26, GREY);
    let fuel = format!("FUEL {:>5.0} LB", r.fuel);
    let col = if r.fuel < r.bingo { AMBER } else { GREEN };
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

/// Radars round the aircraft: a tracker whose pulse just saw it, one heard, one air radar
/// launching, one quiet.
#[cfg(test)]
pub(crate) fn threat_fixture() -> (Vec<EmitterView>, Emv, ThreatHistory) {
    let e = |source, b: f32, km, air, tracking, launch, heard| EmitterView {
        source,
        east: 0.0,
        north: 0.0,
        rel_bearing: b,
        range_km: km,
        lock_range_km: 6.0,
        label: "6".into(),
        air,
        tracking,
        launch,
        priority: 300,
        heard,
        receiving: heard,
    };
    let emitters = vec![
        e(EmitterSource::Site(1), 0.6, 9.0, false, true, false, true),
        e(EmitterSource::Site(2), -2.0, 15.0, false, false, false, true),
        e(EmitterSource::Aircraft(3), 3.5, 12.0, true, false, true, true),
        e(EmitterSource::Gun(4), 1.6, 4.0, false, false, false, false),
    ];
    let emv = Emv { emv: 0.35, egr: 0.5, ear: 0.2 };
    let mut h = ThreatHistory::default();
    for i in 0..HISTORY {
        let t = i as f32 / HISTORY as f32;
        h.samples.push_back((0.2 + 0.3 * t, if i % 90 < 60 { 0.5 + 0.3 * t } else { 0.0 }));
    }
    h.pulse = Some((EmitterSource::Site(1), 0.3, true));
    (emitters, emv, h)
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
    fn left_display_draws() {
        let (emitters, emv, history) = threat_fixture();
        let th = Threats {
            emitters: &emitters,
            emv: &emv,
            history: &history,
            bay_open: true,
            gear_down: true,
            pitch: 0.2,
            roll: -0.5,
        };
        let mut c = Canvas::scaled(MPD_CANVAS.0, MPD_CANVAS.1, MPD_SCALE);
        let list = [(Level::Caution, DETECTED)];
        draw_left(&mut c, &Fonts::builtin(), &th, &list, &[], true, &Countermeasures::default());
        // The bloom shows red (the pulse saw us) and the strip chart has its crossings.
        assert!(c.px.contains(&DEEP_RED) && c.px.contains(&RED) && c.px.contains(&AMBER));
    }

    #[test]
    fn engines_follow_the_torque() {
        let a = hovering();
        let r = engine_readings(&a, 0.3, false, false);
        assert_eq!(r.nr, 100.0);
        assert!(r.tq.iter().all(|t| (t - 60.0).abs() < 0.01), "{:?}", r.tq);
        assert!(r.tgt[0] > 500.0 && r.tgt[0] < 810.0);
        assert!((r.flow - 0.6 * 0.3 * 3600.0).abs() < 1.0);
        // An engine hit: the right engine gives less and runs hot; no fuel, both run down.
        let hit = engine_readings(&a, 0.3, true, false);
        assert!(hit.tq[1] < hit.tq[0] && hit.tgt[1] > hit.tgt[0]);
        let mut dry = a.clone();
        dry.fuel = 0.0;
        let out = engine_readings(&dry, 0.3, false, false);
        assert_eq!((out.tq, out.flow), ([0.0, 0.0], 0.0));
    }

    #[test]
    fn alerts_worst_first() {
        let w = Warnings { fire: true, radar_track: true, ..default() };
        let mut a = hovering();
        a.fuel = 1000.0;
        a.gear_down = false;
        let list = alerts(&w, Some(&a), &Countermeasures::default(), 1500.0);
        let texts: Vec<_> = list.iter().map(|e| e.1).collect();
        assert_eq!(texts, ["ENG 1 FIRE", "LOW FUEL", "RADAR TRACK", "GEAR UP"]);
        assert!(list.windows(2).all(|p| p[0].0 <= p[1].0));
    }

    #[test]
    fn master_caution_until_acknowledged() {
        let mut a = hovering();
        a.gear_down = false;
        let cm = Countermeasures::default();
        // Advisories, a radar track and a detection leave it dark; a launch lights it until
        // acknowledged.
        assert!(!master_caution(&alerts(&Warnings::default(), Some(&a), &cm, 1500.0), &[]));
        let seen = Warnings { radar_track: true, emv_warning: true, ..default() };
        assert!(!master_caution(&alerts(&seen, Some(&a), &cm, 1500.0), &[]));
        let w = Warnings { radar_missile: true, ..default() };
        let list = alerts(&w, Some(&a), &cm, 1500.0);
        assert!(master_caution(&list, &[]));
        let acked: Vec<_> = list.iter().map(|e| e.1).collect();
        assert!(!master_caution(&list, &acked));
    }
}
