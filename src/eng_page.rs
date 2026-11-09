//! The engine instruments of each aircraft type: the ENG format of the modern MFDs
//! (`avionics::Format::Eng`) and the CH-47's centre display. Each type has its own kind of
//! instrument, after its engine displays (the F-22A keeps the page drawn in `avionics`):
//!
//! - UH-60A and CH-47F: thin vertical bar graphs with tick marks on both sides, a readout over
//!   each (the Black Hawk's vertical scale instruments).
//! - F-117A: filled sweep arcs with a digital readout, an engine to a column.
//! - F-16: round needle dials for its one engine.
//! - Tornado IDS: horizontal bar graphs, the left engine's over the right one's.
//! - A-10: vertical scales, the left engine's pointer on their left, the right one's on their
//!   right.
//! - AV-8B: the engine display panel's digital readouts and a nozzle angle quadrant.
//! - AH-1Z: horizontal scales with coloured ranges, engine 1's pointer over them and engine 2's
//!   under them.
//! - OH-58D: half-moon needle dials with coloured ranges for its one engine.
//! - RAH-66: thick bar graphs and readout rows (as on its own engine display).
//! - C-5: four columns of vertical tapes, an engine to a column (fan speed and turbine
//!   temperature), the core speed and oil pressure under them.
//!
//! Green in the normal range, amber over it, red past the limits; a fire shows on the left
//! engine (red), a hit on the right one (the only one of a single engined type).

#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]

use bevy::math::Vec2;

use crate::aircraft::AircraftKind;
use crate::canvas::Canvas;
use crate::flight::{Aircraft, FlightConstants};
use crate::font::{Align, stroke_text_at};

const BLACK: u8 = 5;
const DARK: u8 = 8;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const CYAN: u8 = 11;
const RED: u8 = 13;
const WHITE: u8 = 15;
const AMBER: u8 = 18;

/// Engines a page shows at most.
pub const MAX_ENGINES: usize = 4;

/// What the engine instruments show, per engine (left to right; the left only for a single).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Eng {
    pub engines: usize,
    /// Core (high pressure, gas generator) speed (%).
    pub core: [f32; MAX_ENGINES],
    /// Fan (low pressure) speed (%); a helicopter's power turbine.
    pub fan: [f32; MAX_ENGINES],
    /// Turbine temperature (deg C).
    pub temp: [f32; MAX_ENGINES],
    /// Oil pressure (psi) and temperature (deg C).
    pub oil: [f32; MAX_ENGINES],
    pub oil_t: [f32; MAX_ENGINES],
    /// Exhaust nozzle opening (%).
    pub nozzle: [f32; MAX_ENGINES],
    /// Torque (%) and rotor speed (%): helicopters.
    pub tq: [f32; MAX_ENGINES],
    pub nr: f32,
    /// Fuel flow of all engines (lb/h), fuel (lbs) and bingo.
    pub flow: f32,
    pub fuel: f32,
    pub bingo: f32,
    /// The thrust vector (deg down from aft): the AV-8B's nozzles.
    pub vector: f32,
    pub fire: bool,
    pub hit: bool,
}

/// Value limits: amber past `amber`, red past `red` or under `low` (running).
#[derive(Clone, Copy)]
struct Lim {
    amber: f32,
    red: f32,
    low: f32,
}

const fn lim(amber: f32, red: f32) -> Lim {
    Lim { amber, red, low: f32::NEG_INFINITY }
}

const NONE: Lim = lim(f32::INFINITY, f32::INFINITY);
const JET_CORE: Lim = lim(100.5, 104.0);
const JET_TEMP: Lim = lim(900.0, 950.0);
const JET_OIL: Lim = Lim { amber: 80.0, red: 95.0, low: 15.0 };
const TQ: Lim = lim(100.0, 105.0);
const TGT: Lim = lim(810.0, 870.0);
const NG: Lim = lim(103.0, 106.0);
const NR: Lim = Lim { amber: 104.0, red: 110.0, low: 90.0 };
const OIL_P: Lim = Lim { amber: 90.0, red: 100.0, low: 25.0 };
const OIL_T: Lim = lim(120.0, 140.0);

impl Lim {
    fn colour(self, v: f32) -> u8 {
        if v > self.red || (v < self.low && v > 0.5) {
            RED
        } else if v > self.amber {
            AMBER
        } else {
            GREEN
        }
    }
}

impl AircraftKind {
    /// Engines driving it.
    pub fn engines(self) -> usize {
        match self {
            AircraftKind::F16 | AircraftKind::AV8B | AircraftKind::OH58 => 1,
            AircraftKind::C5 => 4,
            _ => 2,
        }
    }
}

impl Eng {
    /// The readings of aircraft `a` of type `kind` (its constants `k`).
    pub fn of(kind: AircraftKind, a: &Aircraft, k: Option<&FlightConstants>) -> Self {
        let fire = a.damage.fuel_leak > 0.0;
        let hit = a.damage.throttle_cap.is_some();
        let mut e = match k.and_then(|k| k.heli.as_ref()) {
            Some(rotor) => {
                let r = crate::mpd::engine_readings(a, rotor.fuel_burn, hit, fire);
                let twin = |v: [f32; 2]| [v[0], v[1], 0.0, 0.0];
                Eng {
                    core: twin(r.ng),
                    fan: twin([r.nr; 2]),
                    temp: twin(r.tgt),
                    oil: twin(r.oil_p),
                    oil_t: twin(r.oil_t),
                    tq: twin(r.tq),
                    nr: r.nr,
                    flow: r.flow,
                    ..Default::default()
                }
            }
            None => Self::jet(a, hit, fire, k.map_or(0.0, |k| crate::flight::fuel_flow(a, k))),
        };
        e.engines = kind.engines();
        // A single engine shows the hit itself.
        if e.engines == 1 && hit {
            for v in [&mut e.core, &mut e.fan, &mut e.temp, &mut e.oil, &mut e.oil_t, &mut e.tq] {
                v[0] = v[1];
            }
        }
        e.fuel = a.fuel.max(0.0);
        e.bingo = k.map_or(0.0, |k| k.bingo);
        e.vector = a.nozzle.to_degrees();
        e.fire = fire;
        e.hit = hit;
        e
    }

    /// A jet's engines at its thrust (% of military power, more in afterburner); `flow` in lb
    /// a minute.
    fn jet(a: &Aircraft, hit: bool, fire: bool, flow: f32) -> Self {
        let running = a.fuel > 0.0;
        let t = a.thrust.clamp(0.0, 120.0);
        let dry = t.min(100.0);
        let ab = (t - 100.0).max(0.0);
        let mut e = Eng { flow: flow * 60.0, ..Default::default() };
        for i in 0..MAX_ENGINES {
            let (core, fan, temp, oil) = if running {
                (
                    60.0 + 0.4 * dry + 0.05 * ab,
                    35.0 + 0.65 * dry,
                    350.0 + 5.0 * dry + ab,
                    25.0 + 0.4 * dry,
                )
            } else {
                (0.0, 0.0, 60.0, 0.0)
            };
            e.core[i] = core;
            e.fan[i] = fan;
            e.temp[i] = temp;
            e.oil[i] = oil;
            e.oil_t[i] = if running { 70.0 + 0.3 * dry } else { 20.0 };
            e.nozzle[i] = if t > 100.0 { ab * 5.0 } else { 100.0 - t };
        }
        if hit {
            e.core[1] = e.core[1].min(60.0 + 0.3 * dry);
            e.fan[1] *= 0.8;
            e.temp[1] += 120.0;
            e.oil[1] *= 0.5;
        }
        if fire {
            e.temp[0] = e.temp[0].max(980.0);
        }
        e
    }

    /// The colour of engine `i`'s value `v` within limits `l`: a fire's engine red.
    fn col(&self, i: usize, v: f32, l: Lim) -> u8 {
        if self.fire && i == 0 { RED } else { l.colour(v) }
    }
}

fn text(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, col: u8, align: Align) {
    stroke_text_at(c, x, y, s, size, (size * 0.15).max(1.0), col, align);
}

fn v2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// The fuel and its flow along the page's foot (at `y`, from `x0` to `x1`).
fn footer(c: &mut Canvas, e: &Eng, x0: f32, x1: f32, y: f32) {
    let col = if e.fuel < e.bingo { AMBER } else { GREEN };
    text(c, x0, y, &format!("FUEL {:.0}", e.fuel), 8.0, col, Align::Left);
    text(c, x1, y, &format!("FF {:.0} PPH", e.flow), 8.0, CYAN, Align::Right);
}

/// Draws aircraft type `kind`'s ENG format on an MFD canvas (231 x 180, the label bands at the
/// top and bottom left clear).
pub fn draw(c: &mut Canvas, kind: AircraftKind, e: &Eng) {
    c.clear(BLACK);
    let (w, h) = (c.w as f32, c.h as f32);
    let (top, bottom) = (22.0, h - 22.0);
    match kind {
        AircraftKind::UH60 => {
            thin_bars(c, (8.0, top, w - 8.0, bottom - 12.0), e, &uh60_groups(e));
        }
        AircraftKind::CH47 => {
            thin_bars(c, (8.0, top, w - 8.0, bottom - 12.0), e, &ch47_groups(e, true));
        }
        AircraftKind::F117 => sweep_arcs(c, w, top, e),
        AircraftKind::F16 => round_dials(c, w, top, e),
        AircraftKind::Tornado => horizontal_bars(c, w, top, e),
        AircraftKind::A10 => pointer_scales(c, w, top, bottom - 14.0, e),
        AircraftKind::AV8B => harrier(c, w, top, e),
        AircraftKind::AH1Z => horizontal_pointers(c, w, top, e),
        AircraftKind::OH58 => half_dials(c, w, top, e),
        AircraftKind::RAH66 | AircraftKind::F22 => thick_bars(c, w, top, bottom - 12.0, e),
        AircraftKind::C5 => four_columns(c, w, top, bottom - 12.0, e),
    }
    footer(c, e, 10.0, w - 10.0, bottom);
}

/// The CH-47's centre display (an MPD, 128 x 180): its thin bar graphs, the fuel at the foot.
pub fn draw_ch47_centre(c: &mut Canvas, e: &Eng) {
    c.clear(BLACK);
    let (w, h) = (c.w as f32, c.h as f32);
    text(c, w / 2.0, 7.0, "ENGINE", 8.0, CYAN, Align::Centre);
    c.stroke(v2(4.0, 13.0), v2(w - 4.0, 13.0), 1.0, GREY);
    thin_bars(c, (3.0, 18.0, w - 3.0, h - 30.0), e, &ch47_groups(e, false));
    c.stroke(v2(4.0, h - 26.0), v2(w - 4.0, h - 26.0), 1.0, GREY);
    let col = if e.fuel < e.bingo { AMBER } else { GREEN };
    text(c, 6.0, h - 18.0, &format!("FUEL {:.0} LB", e.fuel), 7.0, col, Align::Left);
    text(c, 6.0, h - 8.0, &format!("FLOW {:.0} LB/H", e.flow), 7.0, CYAN, Align::Left);
}

// --- thin vertical bar graphs (UH-60, CH-47)

/// A group of bar graphs under one name: (value, colour) a bar, the scale (lo, hi), the major
/// tick step, and the mark of the normal limit (100% torque), if any.
struct Group {
    name: &'static str,
    bars: Vec<(&'static str, f32, u8)>,
    scale: (f32, f32),
    step: f32,
    mark: Option<f32>,
}

fn pair(e: &Eng, v: [f32; MAX_ENGINES], l: Lim) -> Vec<(&'static str, f32, u8)> {
    vec![("1", v[0], e.col(0, v[0], l)), ("2", v[1], e.col(1, v[1], l))]
}

fn uh60_groups(e: &Eng) -> Vec<Group> {
    vec![
        Group {
            name: "%RPM",
            bars: vec![
                ("1", e.fan[0], e.col(0, e.fan[0], NR)),
                ("R", e.nr, NR.colour(e.nr)),
                ("2", e.fan[1], e.col(1, e.fan[1], NR)),
            ],
            scale: (0.0, 120.0),
            step: 10.0,
            mark: Some(100.0),
        },
        Group {
            name: "TRQ",
            bars: pair(e, e.tq, TQ),
            scale: (0.0, 140.0),
            step: 10.0,
            mark: Some(100.0),
        },
        Group {
            name: "TGT",
            bars: pair(e, e.temp, TGT),
            scale: (0.0, 1000.0),
            step: 100.0,
            mark: Some(810.0),
        },
        Group {
            name: "NG",
            bars: pair(e, e.core, NG),
            scale: (0.0, 110.0),
            step: 10.0,
            mark: None,
        },
        Group {
            name: "OIL P",
            bars: pair(e, e.oil, OIL_P),
            scale: (0.0, 120.0),
            step: 20.0,
            mark: None,
        },
        Group {
            name: "OIL T",
            bars: pair(e, e.oil_t, OIL_T),
            scale: (0.0, 160.0),
            step: 20.0,
            mark: None,
        },
    ]
}

/// The CH-47F's: rotor speed, torque (Q), the power turbine inlet temperature (PTIT) and the gas
/// producer speed (N1); with the oil when `wide`.
fn ch47_groups(e: &Eng, wide: bool) -> Vec<Group> {
    let mut g = vec![
        Group {
            name: "NR",
            bars: vec![("R", e.nr, NR.colour(e.nr))],
            scale: (0.0, 120.0),
            step: 10.0,
            mark: Some(100.0),
        },
        Group {
            name: "Q",
            bars: pair(e, e.tq, TQ),
            scale: (0.0, 140.0),
            step: 10.0,
            mark: Some(100.0),
        },
        Group {
            name: "PTIT",
            bars: pair(e, e.temp, TGT),
            scale: (0.0, 1000.0),
            step: 100.0,
            mark: Some(810.0),
        },
        Group {
            name: "N1",
            bars: pair(e, e.core, NG),
            scale: (0.0, 110.0),
            step: 10.0,
            mark: None,
        },
    ];
    if wide {
        g.push(Group {
            name: "OIL P",
            bars: pair(e, e.oil, OIL_P),
            scale: (0.0, 120.0),
            step: 20.0,
            mark: None,
        });
        g.push(Group {
            name: "OIL T",
            bars: pair(e, e.oil_t, OIL_T),
            scale: (0.0, 160.0),
            step: 20.0,
            mark: None,
        });
    }
    g
}

/// Thin vertical bar graphs in `(x0, y0, x1, y1)`: a readout over each bar, tick marks on both
/// its sides (long ones every other step), the bar's number under it and the group's name under
/// that.
fn thin_bars(c: &mut Canvas, (x0, y0, x1, y1): (f32, f32, f32, f32), _e: &Eng, groups: &[Group]) {
    let bars: usize = groups.iter().map(|g| g.bars.len()).sum();
    let gaps = groups.len().saturating_sub(1) as f32;
    // A bar's pitch, and a group's extra gap.
    let pitch = (x1 - x0) / (bars as f32 + gaps * 0.6);
    let gap = pitch * 0.6;
    let size = (pitch * 0.42).clamp(5.0, 7.0);
    // Readouts too wide for a bar's pitch take turns at two heights.
    let stagger = pitch < size * 2.4;
    let (top, foot) = (y0 + if stagger { 19.0 } else { 12.0 }, y1 - 18.0);
    let mut n = 0;
    let mut x = x0 + pitch / 2.0;
    for g in groups {
        let first = x;
        for &(num, v, col) in &g.bars {
            let frac = |v: f32| ((v - g.scale.0) / (g.scale.1 - g.scale.0)).clamp(0.0, 1.0);
            let y = |v: f32| foot - frac(v) * (foot - top);
            // The ticks either side.
            let mut t = g.scale.0;
            let mut k = 0;
            while t <= g.scale.1 + 0.01 {
                let long = if k % 2 == 0 { 3.0 } else { 1.8 };
                let yy = y(t);
                c.stroke(v2(x - 2.2 - long, yy), v2(x - 2.2, yy), 0.7, GREY);
                c.stroke(v2(x + 2.2, yy), v2(x + 2.2 + long, yy), 0.7, GREY);
                t += g.step;
                k += 1;
            }
            if let Some(m) = g.mark {
                let yy = y(m);
                c.stroke(v2(x - 5.5, yy), v2(x + 5.5, yy), 1.0, WHITE);
            }
            // The bar, thin, from the foot up.
            c.stroke(v2(x, foot), v2(x, y(v)), 2.4, col);
            text(
                c,
                x,
                top - 6.0 - if stagger && n % 2 == 1 { 7.0 } else { 0.0 },
                &format!("{v:.0}"),
                size,
                if col == GREEN { WHITE } else { col },
                Align::Centre,
            );
            n += 1;
            text(c, x, foot + 6.0, num, size, GREY, Align::Centre);
            x += pitch;
        }
        let last = x - pitch;
        text(c, (first + last) / 2.0, foot + 14.0, g.name, size, GREEN, Align::Centre);
        x += gap;
    }
}

// --- filled sweep arcs (F-117A)

/// A 240-degree sweep from lower left over the top, filled to `frac`, a red radial at `red`.
#[allow(clippy::too_many_arguments)]
fn sweep(c: &mut Canvas, o: Vec2, r: f32, frac: f32, red: f32, col: u8, value: &str) {
    let a0 = 150f32.to_radians();
    let span = 240f32.to_radians();
    c.arc(o, r, a0, a0 + span, 1.0, DARK);
    let f = frac.clamp(0.0, 1.0);
    if f > 0.0 {
        c.arc(o, r - 1.5, a0, a0 + span * f, 3.2, col);
    }
    let t = a0 + span * red.clamp(0.0, 1.0);
    let d = Vec2::from_angle(t);
    c.stroke(o + d * (r - 5.0), o + d * (r + 2.0), 1.4, RED);
    c.frame(o + v2(-13.0, 3.0), o + v2(13.0, 13.0), 1.0, GREY);
    text(c, o.x + 11.0, o.y + 8.0, value, 7.0, WHITE, Align::Right);
}

fn sweep_arcs(c: &mut Canvas, w: f32, top: f32, e: &Eng) {
    let cols = [w / 2.0 - 52.0, w / 2.0 + 52.0];
    for (i, &x) in cols.iter().enumerate() {
        text(c, x, top + 2.0, if i == 0 { "L" } else { "R" }, 8.0, CYAN, Align::Centre);
        let o = v2(x, top + 30.0);
        let rpm = e.core[i];
        sweep(
            c,
            o,
            20.0,
            rpm / 110.0,
            104.0 / 110.0,
            e.col(i, rpm, JET_CORE),
            &format!("{rpm:.0}"),
        );
        let o = v2(x, top + 80.0);
        let t = e.temp[i];
        sweep(
            c,
            o,
            20.0,
            (t - 200.0) / 900.0,
            750.0 / 900.0,
            e.col(i, t, JET_TEMP),
            &format!("{t:.0}"),
        );
    }
    text(c, w / 2.0, top + 30.0, "RPM", 8.0, GREEN, Align::Centre);
    text(c, w / 2.0, top + 80.0, "EGT", 8.0, GREEN, Align::Centre);
    let y = top + 112.0;
    for (k, (name, v)) in [("OIL", e.oil), ("NOZ", e.nozzle)].into_iter().enumerate() {
        let yy = y + 11.0 * k as f32;
        text(c, w / 2.0, yy, name, 7.5, GREEN, Align::Centre);
        for (i, &x) in cols.iter().enumerate() {
            let col = if name == "OIL" { e.col(i, v[i], JET_OIL) } else { WHITE };
            text(c, x, yy, &format!("{:.0}", v[i]), 7.5, col, Align::Centre);
        }
    }
}

// --- round needle dials (F-16)

/// A round dial: its scale from `lo` to `hi` over 300 degrees from the bottom, a tick every
/// `step` (numbered every `label` ticks, at the value / `div`), the needle at `v`.
#[allow(clippy::too_many_arguments)]
fn dial(
    c: &mut Canvas,
    o: Vec2,
    r: f32,
    name: &str,
    (lo, hi): (f32, f32),
    step: f32,
    label: usize,
    div: f32,
    v: f32,
    col: u8,
) {
    c.ring(o, r, 1.2, GREY);
    let a0 = 120f32.to_radians();
    let span = 300f32.to_radians();
    let at = |v: f32| a0 + span * ((v - lo) / (hi - lo)).clamp(0.0, 1.04);
    let mut t = lo;
    let mut k = 0;
    while t <= hi + 0.01 {
        let d = Vec2::from_angle(at(t));
        let long = k % label == 0;
        c.stroke(o + d * (r - if long { 5.0 } else { 3.0 }), o + d * (r - 0.6), 1.0, WHITE);
        if long {
            text(
                c,
                o.x + d.x * (r - 10.0),
                o.y + d.y * (r - 10.0),
                &format!("{:.0}", t / div),
                5.5,
                WHITE,
                Align::Centre,
            );
        }
        t += step;
        k += 1;
    }
    text(c, o.x, o.y + r + 6.0, name, 6.5, GREEN, Align::Centre);
    let d = Vec2::from_angle(at(v));
    c.stroke(o - d * 3.0, o + d * (r - 4.0), 1.8, col);
    c.circle(o.x, o.y, 2.0, GREY);
}

fn round_dials(c: &mut Canvas, w: f32, top: f32, e: &Eng) {
    let (l, r) = (w / 2.0 - 42.0, w / 2.0 + 42.0);
    let (y0, y1) = (top + 28.0, top + 92.0);
    let rad = 26.0;
    dial(
        c,
        v2(l, y0),
        rad,
        "RPM",
        (0.0, 110.0),
        10.0,
        2,
        1.0,
        e.core[0],
        e.col(0, e.core[0], JET_CORE),
    );
    dial(
        c,
        v2(r, y0),
        rad,
        "FTIT",
        (200.0, 1200.0),
        100.0,
        2,
        100.0,
        e.temp[0],
        e.col(0, e.temp[0], JET_TEMP),
    );
    dial(c, v2(l, y1), rad, "NOZ POS", (0.0, 100.0), 10.0, 2, 1.0, e.nozzle[0], WHITE);
    dial(
        c,
        v2(r, y1),
        rad,
        "OIL",
        (0.0, 100.0),
        10.0,
        2,
        1.0,
        e.oil[0],
        e.col(0, e.oil[0], JET_OIL),
    );
}

// --- horizontal bar graphs (Tornado)

fn horizontal_bars(c: &mut Canvas, w: f32, top: f32, e: &Eng) {
    let rows = [
        ("NL", e.fan, (0.0, 110.0), 10.0, JET_CORE),
        ("NH", e.core, (0.0, 110.0), 10.0, JET_CORE),
        ("TOT", e.temp, (200.0, 1100.0), 100.0, JET_TEMP),
        ("NOZ", e.nozzle, (0.0, 100.0), 10.0, NONE),
        ("OIL", e.oil, (0.0, 100.0), 10.0, JET_OIL),
    ];
    let (x0, x1) = (40.0, w - 44.0);
    for (k, (name, v, (lo, hi), step, l)) in rows.into_iter().enumerate() {
        let y = top + 6.0 + 23.0 * k as f32;
        text(c, 8.0, y + 6.0, name, 8.0, GREEN, Align::Left);
        let at = |v: f32| x0 + (x1 - x0) * ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
        // The scale's ticks over the bars.
        let mut t = lo;
        while t <= hi + 0.01 {
            c.stroke(v2(at(t), y - 1.5), v2(at(t), y + 0.5), 0.8, GREY);
            t += step;
        }
        for i in 0..2 {
            let yy = y + 4.0 + 6.0 * i as f32;
            let col = if name == "NOZ" { CYAN } else { e.col(i, v[i], l) };
            c.frame(v2(x0, yy - 2.2), v2(x1, yy + 2.2), 0.6, DARK);
            c.stroke(v2(x0, yy), v2(at(v[i]).max(x0 + 0.5), yy), 3.4, col);
            text(c, w - 8.0, yy, &format!("{:.0}", v[i]), 5.5, WHITE, Align::Right);
            text(c, x0 - 3.0, yy, ["L", "R"][i], 5.0, GREY, Align::Right);
        }
    }
}

// --- vertical scales with two pointers (A-10)

fn pointer_scales(c: &mut Canvas, w: f32, top: f32, bottom: f32, e: &Eng) {
    let cols = [
        ("ITT", e.temp, (200.0, 1000.0), 100.0, JET_TEMP),
        ("CORE", e.core, (0.0, 110.0), 10.0, JET_CORE),
        ("FAN", e.fan, (0.0, 110.0), 10.0, JET_CORE),
        ("OIL", e.oil, (0.0, 100.0), 10.0, JET_OIL),
        ("NOZ", e.nozzle, (0.0, 100.0), 10.0, NONE),
    ];
    let (y0, y1) = (top + 18.0, bottom - 10.0);
    let pitch = (w - 20.0) / cols.len() as f32;
    for (k, (name, v, (lo, hi), step, l)) in cols.into_iter().enumerate() {
        let x = 10.0 + pitch * (k as f32 + 0.5);
        let at = |v: f32| y1 - (y1 - y0) * ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
        c.stroke(v2(x, y0), v2(x, y1), 1.0, WHITE);
        let mut t = lo;
        let mut n = 0;
        while t <= hi + 0.01 {
            let half = if n % 2 == 0 { 3.0 } else { 1.5 };
            c.stroke(v2(x - half, at(t)), v2(x + half, at(t)), 0.8, WHITE);
            t += step;
            n += 1;
        }
        for i in 0..2 {
            let s = if i == 0 { -1.0 } else { 1.0 };
            let y = at(v[i]);
            let col = if name == "NOZ" { CYAN } else { e.col(i, v[i], l) };
            let tip = v2(x + s * 4.0, y);
            c.triangle([tip, tip + v2(s * 7.0, -4.0), tip + v2(s * 7.0, 4.0)], col);
            text(c, x + s * 11.0, y0 - 9.0, &format!("{:.0}", v[i]), 5.0, col, Align::Centre);
        }
        text(c, x, y1 + 8.0, name, 7.0, GREEN, Align::Centre);
    }
    text(c, 10.0, top + 2.0, "L", 7.0, CYAN, Align::Left);
    text(c, w - 10.0, top + 2.0, "R", 7.0, CYAN, Align::Right);
}

// --- digital engine display panel and nozzle quadrant (AV-8B)

fn harrier(c: &mut Canvas, w: f32, top: f32, e: &Eng) {
    let rows = [
        ("RPM", format!("{:.1}", e.core[0]), e.col(0, e.core[0], JET_CORE)),
        ("JPT", format!("{:.0}", e.temp[0]), e.col(0, e.temp[0], JET_TEMP)),
        ("FF", format!("{:.0}", e.flow / 60.0), WHITE),
        ("OIL", format!("{:.0}", e.oil[0]), e.col(0, e.oil[0], JET_OIL)),
        ("H2O", "OFF".to_string(), GREY),
    ];
    for (k, (name, value, col)) in rows.iter().enumerate() {
        let y = top + 8.0 + 22.0 * k as f32;
        text(c, 10.0, y, name, 7.5, GREEN, Align::Left);
        c.frame(v2(38.0, y - 7.0), v2(94.0, y + 7.0), 1.0, GREY);
        text(c, 90.0, y, value, 9.0, *col, Align::Right);
    }
    // The nozzle quadrant: aft (0, right) to down (90) and the braking stop (98.5).
    let o = v2(w - 92.0, top + 30.0);
    let r = 62.0;
    c.arc(o, r, 0.0, 98.5f32.to_radians(), 1.2, GREY);
    let mut d = 0.0f32;
    while d <= 90.0 {
        let u = Vec2::from_angle(d.to_radians());
        let long = (d as i32) % 30 == 0;
        c.stroke(o + u * (r - if long { 7.0 } else { 4.0 }), o + u * r, 1.0, WHITE);
        if long {
            text(
                c,
                o.x + u.x * (r + 8.0),
                o.y + u.y * (r + 7.0),
                &format!("{d:.0}"),
                6.0,
                WHITE,
                Align::Centre,
            );
        }
        d += 10.0;
    }
    let stop = Vec2::from_angle(98.5f32.to_radians());
    c.stroke(o + stop * (r - 7.0), o + stop * (r + 2.0), 1.4, AMBER);
    let u = Vec2::from_angle(e.vector.clamp(0.0, 98.5).to_radians());
    c.stroke(o, o + u * (r - 3.0), 2.0, CYAN);
    c.circle(o.x, o.y, 2.5, GREY);
    text(c, o.x - 22.0, o.y + 34.0, "NOZZ", 7.5, GREEN, Align::Centre);
    text(c, o.x - 22.0, o.y + 47.0, &format!("{:.0}", e.vector), 9.0, CYAN, Align::Centre);
}

// --- horizontal scales with pointers above and below (AH-1Z)

fn horizontal_pointers(c: &mut Canvas, w: f32, top: f32, e: &Eng) {
    // (name, values, scale, green to, amber to, red line)
    let rows = [
        ("TQ", e.tq, (0.0, 120.0), TQ),
        ("TGT", e.temp, (0.0, 1000.0), TGT),
        ("NG", e.core, (50.0, 110.0), NG),
        ("NP", e.fan, (50.0, 120.0), NR),
        ("OIL", e.oil, (0.0, 120.0), OIL_P),
    ];
    let (x0, x1) = (36.0, w - 40.0);
    for (k, (name, v, (lo, hi), l)) in rows.into_iter().enumerate() {
        let y = top + 10.0 + 25.0 * k as f32;
        let at = |v: f32| x0 + (x1 - x0) * ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
        text(c, 8.0, y, name, 7.5, GREEN, Align::Left);
        // The ranges along the scale: green to amber, amber to red, red beyond.
        let low = if l.low.is_finite() { l.low } else { lo };
        c.stroke(v2(at(low), y), v2(at(l.amber.min(hi)), y), 2.0, GREEN);
        if l.amber < hi {
            c.stroke(v2(at(l.amber), y), v2(at(l.red.min(hi)), y), 2.0, AMBER);
        }
        if l.red < hi {
            c.stroke(v2(at(l.red), y - 4.0), v2(at(l.red), y + 4.0), 1.4, RED);
        }
        if l.low.is_finite() {
            c.stroke(v2(at(lo), y), v2(at(l.low), y), 2.0, RED);
        }
        for t in 0..=4 {
            let x = at(lo + (hi - lo) * t as f32 / 4.0);
            c.stroke(v2(x, y - 2.0), v2(x, y + 2.0), 0.8, WHITE);
        }
        for i in 0..2 {
            let s = if i == 0 { -1.0 } else { 1.0 };
            let x = at(v[i]);
            let col = e.col(i, v[i], l);
            let tip = v2(x, y + s * 2.5);
            c.triangle([tip, tip + v2(-3.5, s * 6.0), tip + v2(3.5, s * 6.0)], col);
            text(
                c,
                w - 8.0,
                y + s * 5.0,
                &format!("{}:{:.0}", i + 1, v[i]),
                6.0,
                col,
                Align::Right,
            );
        }
    }
}

// --- half-moon needle dials with coloured ranges (OH-58D)

/// A half-moon dial (over its centre), its range bands on the arc, the needle and the readout.
fn half_dial(
    c: &mut Canvas,
    o: Vec2,
    r: f32,
    name: &str,
    (lo, hi): (f32, f32),
    l: Lim,
    v: f32,
    col: u8,
) {
    let pi = std::f32::consts::PI;
    let ang = |v: f32| pi + pi * ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
    let low = if l.low.is_finite() { l.low } else { lo };
    if l.low.is_finite() {
        c.arc(o, r, ang(lo), ang(l.low), 3.0, RED);
    }
    c.arc(o, r, ang(low), ang(l.amber.min(hi)), 3.0, GREEN);
    if l.amber < hi {
        c.arc(o, r, ang(l.amber), ang(l.red.min(hi)), 3.0, AMBER);
    }
    if l.red < hi {
        let d = Vec2::from_angle(ang(l.red));
        c.stroke(o + d * (r - 5.0), o + d * (r + 3.0), 1.6, RED);
    }
    for t in 0..=5 {
        let d = Vec2::from_angle(ang(lo + (hi - lo) * t as f32 / 5.0));
        c.stroke(o + d * (r - 6.0), o + d * (r - 2.5), 0.9, WHITE);
    }
    let d = Vec2::from_angle(ang(v));
    c.stroke(o, o + d * (r - 4.0), 1.8, WHITE);
    c.circle(o.x, o.y, 2.0, GREY);
    text(c, o.x, o.y + 7.0, &format!("{v:.0}"), 8.0, col, Align::Centre);
    text(c, o.x, o.y - r * 0.4, name, 6.5, GREEN, Align::Centre);
}

fn half_dials(c: &mut Canvas, w: f32, top: f32, e: &Eng) {
    let (l, r) = (w / 2.0 - 52.0, w / 2.0 + 52.0);
    let (y0, y1) = (top + 46.0, top + 112.0);
    let rad = 36.0;
    half_dial(c, v2(l, y0), rad, "TQ", (0.0, 120.0), TQ, e.tq[0], e.col(0, e.tq[0], TQ));
    half_dial(c, v2(r, y0), rad, "TOT", (0.0, 1000.0), TGT, e.temp[0], e.col(0, e.temp[0], TGT));
    half_dial(c, v2(l, y1), rad, "NG", (0.0, 110.0), NG, e.core[0], e.col(0, e.core[0], NG));
    half_dial(c, v2(r, y1), rad, "NR", (0.0, 120.0), NR, e.nr, NR.colour(e.nr));
}

// --- thick bars and readout rows (RAH-66)

fn thick_bars(c: &mut Canvas, w: f32, top: f32, bottom: f32, e: &Eng) {
    let bars = [
        ("NR", e.nr, 120.0, NR.colour(e.nr)),
        ("TQ1", e.tq[0], 120.0, e.col(0, e.tq[0], TQ)),
        ("TQ2", e.tq[1], 120.0, e.col(1, e.tq[1], TQ)),
    ];
    let (y0, y1) = (top + 16.0, bottom - 10.0);
    for (k, &(name, v, max, col)) in bars.iter().enumerate() {
        let x = 22.0 + 30.0 * k as f32;
        c.frame(v2(x - 8.0, top - 2.0), v2(x + 8.0, top + 9.0), 1.0, col);
        text(c, x, top + 3.5, &format!("{v:.0}"), 7.0, WHITE, Align::Centre);
        c.frame(v2(x - 6.0, y0), v2(x + 6.0, y1), 1.0, GREY);
        let f = (v / max).clamp(0.0, 1.0);
        let yv = y1 - f * (y1 - y0);
        c.stroke(v2(x, y1 - 1.0), v2(x, yv.min(y1 - 1.0)), 10.0, col);
        let y100 = y1 - 100.0 / max * (y1 - y0);
        c.stroke(v2(x - 9.0, y100), v2(x + 9.0, y100), 1.2, WHITE);
        text(c, x, y1 + 7.0, name, 6.5, GREY, Align::Centre);
    }
    let rows = [
        ("TGT", e.temp, 1000.0, TGT),
        ("NG", e.core, 110.0, NG),
        ("OIL P", e.oil, 120.0, OIL_P),
        ("OIL T", e.oil_t, 160.0, OIL_T),
    ];
    let x0 = 108.0;
    for (k, (name, v, max, l)) in rows.into_iter().enumerate() {
        let y = top + 12.0 + 28.0 * k as f32;
        text(c, x0, y, name, 7.0, GREY, Align::Left);
        for i in 0..2 {
            let yy = y + 10.0 + 8.0 * i as f32;
            let col = e.col(i, v[i], l);
            text(c, x0 + 32.0, yy, &format!("{:.0}", v[i]), 6.5, col, Align::Right);
            c.frame(v2(x0 + 38.0, yy - 2.0), v2(w - 10.0, yy + 2.0), 0.6, DARK);
            let f = (v[i] / max).clamp(0.0, 1.0);
            c.stroke(v2(x0 + 38.0, yy), v2(x0 + 38.0 + f * (w - 48.0 - x0), yy), 3.0, col);
        }
    }
}

// --- four columns of vertical tapes (C-5)

/// The C-5's: the engines' numbers over four columns; in each the fan speed (N1) and turbine
/// temperature (EGT) as filled tapes with an amber limit mark, the reading under each tape; the
/// core speed (N2) and oil pressure as numbers under them, clear of the fuel line.
fn four_columns(c: &mut Canvas, w: f32, top: f32, _bottom: f32, e: &Eng) {
    let x0 = 34.0;
    let step = (w - x0 - 6.0) / 4.0;
    let x = |i: usize| x0 + step * (i as f32 + 0.5);
    for i in 0..4 {
        text(c, x(i), top + 4.0, &format!("{}", i + 1), 7.0, CYAN, Align::Centre);
    }
    let half = 7.0;
    let tapes = [
        ("N1", e.fan, 110.0, JET_CORE, top + 12.0, top + 46.0),
        ("EGT", e.temp, 1000.0, JET_TEMP, top + 60.0, top + 94.0),
    ];
    for (name, v, max, l, y0, y1) in tapes {
        text(c, 4.0, (y0 + y1) / 2.0, name, 7.0, GREY, Align::Left);
        for i in 0..4 {
            let col = e.col(i, v[i], l);
            let (a, b) = (v2(x(i) - half, y0), v2(x(i) + half, y1));
            let f = (v[i] / max).clamp(0.0, 1.0);
            let yv = y1 - f * (y1 - y0);
            if yv < y1 - 0.5 {
                let (p, q) = (v2(a.x + 1.5, yv), v2(b.x - 1.5, y1 - 1.5));
                c.triangle([p, v2(q.x, p.y), q], col);
                c.triangle([p, q, v2(p.x, q.y)], col);
            }
            c.frame(a, b, 1.0, GREY);
            let ya = y1 - l.amber.min(max) / max * (y1 - y0);
            c.stroke(v2(a.x - 3.0, ya), v2(b.x + 3.0, ya), 1.0, AMBER);
            text(c, x(i), y1 + 6.0, &format!("{:.0}", v[i]), 6.5, col, Align::Centre);
        }
    }
    for (k, (name, v, l)) in [("N2", e.core, JET_CORE), ("OIL", e.oil, JET_OIL)].into_iter().enumerate() {
        let y = top + 111.0 + 10.0 * k as f32;
        text(c, 4.0, y, name, 7.0, GREY, Align::Left);
        for i in 0..4 {
            text(c, x(i), y, &format!("{:.0}", v[i]), 6.5, e.col(i, v[i], l), Align::Centre);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every type's page draws within the canvas, each its own; with the right engine hit and
    /// the left one on fire, the twins' pages show red and amber.
    #[test]
    fn every_type_has_its_own_page() {
        let mut a = Aircraft::new_on_runway(2000.0);
        a.thrust = 85.0;
        let mut pages = Vec::new();
        for kind in AircraftKind::ALL {
            let k = kind.spec().flight();
            let e = Eng::of(kind, &a, Some(&k));
            assert_eq!(e.engines, kind.engines());
            let mut c = Canvas::scaled(crate::mfd::HIRES.0, crate::mfd::HIRES.1, 2);
            draw(&mut c, kind, &e);
            assert!(c.px.contains(&GREEN), "{kind:?}");
            if let Ok(dir) = std::env::var("SM89_ENG_PNG") {
                dump(&c, &format!("{dir}/eng_{kind:?}.png"));
            }
            pages.push((kind, c.px));
        }
        for (i, (k1, p1)) in pages.iter().enumerate() {
            for (k2, p2) in &pages[i + 1..] {
                // The F-22's own page is avionics' (this one is the RAH-66's fallback).
                if *k1 != AircraftKind::F22 && *k2 != AircraftKind::F22 {
                    assert_ne!(p1, p2, "{k1:?} and {k2:?} look the same");
                }
            }
        }
        let mut hurt = a.clone();
        hurt.damage.fuel_leak = 1.0;
        hurt.damage.throttle_cap = Some(0.5);
        for kind in [AircraftKind::UH60, AircraftKind::Tornado] {
            let k = kind.spec().flight();
            let e = Eng::of(kind, &hurt, Some(&k));
            let mut c = Canvas::scaled(crate::mfd::HIRES.0, crate::mfd::HIRES.1, 2);
            draw(&mut c, kind, &e);
            assert!(c.px.contains(&RED), "{kind:?}");
        }
        let mut c = Canvas::scaled(crate::mpd::MPD_CANVAS.0, crate::mpd::MPD_CANVAS.1, 2);
        let k = AircraftKind::CH47.spec().flight();
        draw_ch47_centre(&mut c, &Eng::of(AircraftKind::CH47, &a, Some(&k)));
        assert!(c.px.contains(&GREEN));
        if let Ok(dir) = std::env::var("SM89_ENG_PNG") {
            dump(&c, &format!("{dir}/eng_CH47_centre.png"));
        }
    }

    /// The page as a picture, for a look at it (`SM89_ENG_PNG=dir`).
    fn dump(c: &Canvas, path: &str) {
        let rgb = |i: u8| match i {
            GREEN => [40, 220, 60],
            CYAN => [60, 220, 230],
            RED => [230, 40, 40],
            WHITE => [240, 240, 240],
            AMBER => [240, 170, 30],
            GREY => [150, 150, 150],
            DARK => [70, 70, 70],
            _ => [0, 0, 0],
        };
        let px: Vec<u8> = c.px.iter().flat_map(|&i| rgb(i)).collect();
        image::RgbImage::from_raw(c.pw() as u32, c.ph() as u32, px).unwrap().save(path).unwrap();
    }
}
