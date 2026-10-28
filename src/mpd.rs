//! The RAH-66's multipurpose displays, the portrait screens outboard of its MFDs: the warning,
//! caution and advisory list on the left one, the engine page (digital readouts over bar graphs)
//! on the right one. New warnings and cautions flash until the master caution is pressed.

use bevy::prelude::*;

use crate::canvas::Canvas;
use crate::flight::Aircraft;
use crate::font::Fonts;
use crate::input::{Action, ActionEvent};
use crate::panel::{Countermeasures, Warnings};

/// The displays' canvas: drawing units and texels per unit (the screens are 100 by 140 mm).
pub const MPD_CANVAS: (usize, usize) = (128, 180);
pub const MPD_SCALE: usize = 3;

const BLACK: u8 = 5;
const DARK: u8 = 8;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const CYAN: u8 = 11;
const RED: u8 = 13;
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

/// The left display: the list under its title, warnings red, cautions amber, advisories
/// green; unacknowledged warnings and cautions flash (`blink`), inverted. The countermeasures
/// left at the foot.
pub fn draw_alerts(
    c: &mut Canvas,
    fonts: &Fonts,
    list: &[(Level, &'static str)],
    acked: &[&'static str],
    blink: bool,
    cm: &Countermeasures,
) {
    let (w, h) = (c.w as i32, c.h as i32);
    c.clear(BLACK);
    title(c, fonts, "WCA");
    let mut y = 20;
    let mut last = None;
    for &(level, text) in list.iter().take(16) {
        if last.is_some_and(|l| l != level) {
            c.hline(6, w - 7, y + 1, DARK);
            y += 4;
        }
        last = Some(level);
        let colour = match level {
            Level::Warning => RED,
            Level::Caution => AMBER,
            Level::Advisory => GREEN,
        };
        let flash = level != Level::Advisory && !acked.contains(&text);
        if flash && blink {
            c.fill_rect(4, y - 1, w - 8, 9, colour);
            fonts.text(c, 8, y, text.as_bytes(), BLACK);
        } else {
            fonts.text(c, 8, y, text.as_bytes(), colour);
        }
        y += 10;
    }
    if list.is_empty() {
        fonts.text(c, 8, y, b"NO WARNINGS", GREY);
    }
    // The countermeasures left.
    c.hline(4, w - 5, h - 22, GREY);
    let line = format!("CHF {:>2}  FLR {:>2}  DCY {}", cm.chaff, cm.flares, cm.decoys);
    fonts.text(c, 6, h - 16, line.as_bytes(), CYAN);
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
}
