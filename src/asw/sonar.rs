//! The SONAR page: a waterfall for each sensor in the water, one under the other (the live
//! buoys, at most four, and the dipping sonar while it is down). A waterfall's rows are the
//! sensor's reports, newest at the top, four a second (`asw::SENSE_HZ`): across it the true
//! bearing (000 to 360), the signal heard from each direction: the sea's noise as speckle, each
//! contact a trace at its bearing (wider the less sure its bearing, cavitating screws wider
//! still), moving across as its bearing from the sensor changes; another sonar's ping a bright
//! dash at its bearing ringing on in the water for seconds after it (the reverberation), and its
//! echoes off the submarines (sharp dashes at their bearings, a moment later: the sound's way
//! round by them) however quiet they run; the sensor's own ping a bright row. An omnidirectional sensor (a DICASS
//! listening) raises its whole row: it cannot tell the direction. Each row's header has the
//! sensor's kind, number, depth, mode, the strongest contact's signal, bearing, range and
//! Doppler; touch a header to select the sensor (PING and AUDIO then act on it), its buttons
//! switch a DICASS between listening and pinging and scuttle a buoy. A BT buoy's header shows
//! the temperature profile it measured instead.

use std::collections::VecDeque;
use std::sync::Arc;

use bevy::prelude::*;
use stealth_sim::asw::{AswCmd, AswCrew, BuoyKind, Sample};

use super::{AswUi, marks, owner_at};
use crate::canvas::{Canvas, Picture};
use crate::font::{Align, stroke_text_at};
use crate::station::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};

/// Bearing bins across a waterfall (2 degrees each, 0 to 360 true); rows kept.
pub const BINS: usize = 180;
pub const DEG_PER_BIN: f32 = 2.0;
const ROWS: usize = 440;
/// The page's layout (canvas units): the title bar, the rows' area, the header's width.
const TOP: f32 = 26.0;
const BOTTOM: f32 = 452.0;
const HEADER: f32 = 168.0;
const LEFT: f32 = 4.0;
const RIGHT: f32 = 636.0;

/// A sensor's history.
#[derive(Debug, Clone, Default)]
pub struct Waterfall {
    pub seq: u16,
    pub rows: VecDeque<[u8; BINS]>,
    pub last: Sample,
    /// Its own noise stream (the speckle).
    pub rng: u32,
    /// Another sonar's ping heard: its ringing in the water dying away (1 at the ping), the
    /// bearing it came from (None: an omnidirectional sensor).
    pub reverb: f32,
    pub reverb_at: Option<f32>,
    /// The echoes heard lately (the console's clock, bearing, signal): a mark can be put on one
    /// after it has gone by.
    pub echoes: VecDeque<(f32, u16, i8)>,
}

/// The reverberation's time constant (s) after a ping heard: it dies away over a few seconds.
const REVERB_TAU: f32 = 1.6;
/// Echoes kept for marking (s).
pub const ECHO_KEEP: f32 = 30.0;

/// The angle between two bearings (degrees, 0..180).
pub fn bearing_off(a: f32, b: f32) -> f32 {
    ((a - b).rem_euclid(360.0) + 180.0).rem_euclid(360.0) - 180.0
}

impl Waterfall {
    fn noise(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    /// A row from a report (at `now` on the console's clock): signal strength over bearing.
    fn push(&mut self, s: &Sample, now: f32) {
        let mut row = [0f32; BINS];
        let base = 25.0 + s.noise.saturating_sub(40) as f32 * 3.0;
        for v in row.iter_mut() {
            let n = self.noise();
            *v = base + (n % 45) as f32 + if n % 23 == 0 { 40.0 } else { 0.0 };
        }
        // Another sonar's ping heard: it rings on in the water for seconds, loudest from its
        // direction, everywhere at first.
        if s.heard {
            self.reverb = 1.0;
            self.reverb_at = s.lines.iter().find(|l| l.ping).and_then(|l| l.bearing.map(|b| b as f32));
        } else {
            self.reverb *= (-1.0 / (stealth_sim::asw::SENSE_HZ as f32 * REVERB_TAU)).exp();
        }
        if self.reverb > 0.02 {
            let r = self.reverb;
            for (k, v) in row.iter_mut().enumerate() {
                let n = (self.noise() % 100) as f32 / 100.0;
                let toward = self.reverb_at.map_or(0.5, |b| {
                    let d = bearing_off(k as f32 * DEG_PER_BIN + DEG_PER_BIN / 2.0, b);
                    (-(d * d) / (2.0 * 25.0 * 25.0)).exp()
                });
                *v += r * (60.0 + 140.0 * toward) * (0.6 + 0.4 * n);
            }
        }
        for l in &s.lines {
            let snr = l.snr.max(0) as f32;
            let strength = if l.ping {
                255.0
            } else if l.echo {
                (150.0 + snr * 3.0).min(255.0)
            } else {
                (40.0 + snr * 3.0).clamp(20.0, 200.0)
            };
            if l.echo
                && let Some(b) = l.bearing
            {
                self.echoes.push_back((now, b, l.snr));
            }
            match l.bearing {
                // A trace at its bearing, as wide as the bearing is uncertain (cavitating
                // screws spread wider), smooth at its edges; a ping and an echo sharp.
                Some(b) => {
                    let mut sigma = if l.ping || l.echo { 2.0 } else { 3.0 + 30.0 / (1.0 + snr) };
                    if l.ship || l.torpedo {
                        sigma *= 1.6;
                    }
                    for (k, v) in row.iter_mut().enumerate() {
                        let d = bearing_off(k as f32 * DEG_PER_BIN + DEG_PER_BIN / 2.0, b as f32);
                        *v += strength * (-(d * d) / (2.0 * sigma * sigma)).exp();
                    }
                }
                // An omnidirectional sensor: the energy over every bearing (a ping heard all
                // of it).
                None => {
                    let k = if l.ping { 1.0 } else { 0.25 };
                    for v in row.iter_mut() {
                        *v += strength * k;
                    }
                }
            }
        }
        while self.echoes.front().is_some_and(|e| now - e.0 > ECHO_KEEP) {
            self.echoes.pop_front();
        }
        if s.ping {
            for v in row.iter_mut() {
                *v += 120.0;
            }
        }
        self.rows.push_front(row.map(|v| v.min(255.0) as u8));
        self.rows.truncate(ROWS);
        self.last = s.clone();
        self.seq = s.seq;
    }
}

/// The sensors of the crew shown, in order: its buoys (id, kind, its report), then the
/// dipping sonar (0) while it is in the water.
pub fn sensors(asw: &stealth_sim::asw::Asw, crew: &AswCrew, owner: u8) -> Vec<(u16, Option<BuoyKind>)> {
    let mut v: Vec<(u16, Option<BuoyKind>)> = asw.buoys_of(owner).map(|b| (b.id, Some(b.kind))).collect();
    v.sort_by_key(|s| s.0);
    if crew.dip.wet() {
        v.push((0, None));
    }
    v
}

/// New reports into the histories (also off the SONAR page, at another seat's too).
pub fn record(
    manning: Option<Res<crate::station::Manning>>,
    combat: Option<Res<crate::combat::Combat>>,
    settings: Res<crate::settings::Settings>,
    mut ui: ResMut<AswUi>,
) {
    let Some(combat) = combat else { return };
    let Some((owner, _)) = super::crew_owner(manning.as_deref(), &combat, &settings) else { return };
    let asw = combat.sim.asw();
    let crew = asw.crew(owner).cloned().unwrap_or_default();
    let ui = &mut *ui;
    let now = ui.t;
    let mut live = Vec::new();
    for b in asw.buoys_of(owner).filter(|b| b.fall <= 0.0) {
        live.push(b.id);
        let w = ui.falls.entry(b.id).or_insert_with(|| Waterfall { rng: 0x9E37_79B9 ^ b.id as u32, ..Default::default() });
        if w.seq != b.sample.seq {
            w.push(&b.sample, now);
            marks::follow(&mut ui.marks, b.id, &b.sample, now);
        }
    }
    if crew.dip.wet() {
        live.push(0);
        let w = ui.falls.entry(0).or_insert_with(|| Waterfall { rng: 0x1234_5679, ..Default::default() });
        if w.seq != crew.dip.sample.seq {
            w.push(&crew.dip.sample, now);
            marks::follow(&mut ui.marks, 0, &crew.dip.sample, now);
        }
    }
    ui.falls.retain(|id, _| live.contains(id));
    // A sensor gone: its marks with it.
    let n = ui.marks.len();
    ui.marks.retain(|m| live.contains(&m.sensor));
    if ui.marks.len() != n {
        ui.sel_mark = None;
    }
    if ui.sensor.is_some_and(|s| !live.contains(&s)) {
        ui.sensor = live.first().copied();
    }
}

/// A row's area (canvas units): top, height.
fn row_band(k: usize, n: usize) -> (f32, f32) {
    let h = (BOTTOM - TOP) / n.max(1) as f32;
    (TOP + k as f32 * h, h)
}

/// The header's buttons of row `k`: (what, rect x, y, w, h).
#[derive(Debug, Clone, Copy, PartialEq)]
enum RowButton {
    Mode,
    Scuttle,
}

fn row_buttons(top: f32, h: f32, kind: Option<BuoyKind>) -> Vec<(RowButton, [f32; 4])> {
    let y = top + h - 20.0;
    let mut v = Vec::new();
    if kind == Some(BuoyKind::Dicass) || kind.is_none() {
        v.push((RowButton::Mode, [LEFT + 6.0, y, 70.0, 16.0]));
    }
    if kind.is_some() {
        v.push((RowButton::Scuttle, [LEFT + 84.0, y, 74.0, 16.0]));
    }
    v
}

/// The title bar's hydrophone depth button, the mark letter's.
const DEPTH_BUTTON: [f32; 4] = [500.0, 3.0, 134.0, 19.0];
const MARK_BUTTON: [f32; 4] = [400.0, 3.0, 92.0, 19.0];

fn inside(r: [f32; 4], p: Vec2) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

/// A touch on the page.
pub fn click(ui: &mut AswUi, p: Vec2, phase: crate::cockpit3d::DragPhase, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    use crate::cockpit3d::DragPhase;
    // On a waterfall (pressed or dragged): its listening cursor there, the sensor in the
    // headset.
    let list = sensors(sim.asw(), crew, owner);
    for (k, &(id, kind)) in list.iter().enumerate() {
        let (top, h) = row_band(k, list.len());
        let (x0, x1) = (LEFT + HEADER, RIGHT - 2.0);
        if kind != Some(BuoyKind::Bt) && p.y >= top && p.y < top + h - 12.0 && p.x >= x0 && p.x <= x1 {
            let brg = (p.x - x0) / (x1 - x0) * 360.0;
            // One cursor: the headset hears this sensor alone, in that direction.
            ui.cursor.clear();
            ui.cursor.insert(id, brg.round().rem_euclid(360.0));
            ui.sensor = Some(id);
            ui.audio = Some(id);
            return;
        }
    }
    if phase != DragPhase::Start {
        return;
    }
    if inside(DEPTH_BUTTON, p) {
        ui.deep = !ui.deep;
        return;
    }
    if inside(MARK_BUTTON, p) {
        ui.mark_id = (ui.mark_id + 1) % marks::IDS as u8;
        return;
    }
    for (k, &(id, kind)) in list.iter().enumerate() {
        let (top, h) = row_band(k, list.len());
        if p.y < top || p.y >= top + h {
            continue;
        }
        for (b, r) in row_buttons(top, h, kind) {
            if inside(r, p) {
                match (b, kind) {
                    (RowButton::Mode, None) => ui.outbox.push(AswCmd::DipMode { active: !crew.dip.active }),
                    (RowButton::Mode, Some(_)) => {
                        let active = sim.asw().buoys.iter().any(|b| b.id == id && b.active);
                        ui.outbox.push(AswCmd::BuoyMode { id, active: !active });
                    }
                    (RowButton::Scuttle, _) => ui.outbox.push(AswCmd::Scuttle { id }),
                }
                return;
            }
        }
        ui.sensor = Some(id);
        return;
    }
}

/// Draws the page; returns the waterfalls' picture for under it.
pub fn draw(
    cv: &mut Canvas,
    ui: &mut AswUi,
    crew: &AswCrew,
    sim: &stealth_sim::Sim,
    owner: u8,
    prev: Option<Arc<Picture>>,
) -> Option<Arc<Picture>> {
    cv.reset_clip();
    cv.clear(BLACK);
    let asw = sim.asw();
    let here = owner_at(sim, owner).map(|p| p.0);
    // The title bar: the layer, the hydrophone setting for the next buoys.
    stroke_text_at(cv, 8.0, 13.0, "SONAR", 10.0, 1.5, YELLOW, Align::Left);
    let layer = crew.layer.map_or("LAYER ?".to_string(), |l| format!("LAYER {l} M"));
    stroke_text_at(cv, 110.0, 13.0, &layer, 8.0, 1.2, CYAN, Align::Left);
    stroke_text_at(
        cv,
        250.0,
        13.0,
        &format!("BUOYS {}/4", asw.buoys_of(owner).count()),
        8.0,
        1.2,
        GREEN,
        Align::Left,
    );
    let r = DEPTH_BUTTON;
    cv.rect(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32, GREY);
    let hyd = if ui.deep { "HYD DEEP 120M" } else { "HYD SHAL 27M" };
    stroke_text_at(cv, r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 + 1.0, hyd, 7.0, 1.1, WHITE, Align::Centre);
    let r = MARK_BUTTON;
    cv.rect(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32, CYAN);
    let t = format!("MARK ID {}", marks::letter(ui.mark_id));
    stroke_text_at(cv, r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 + 1.0, &t, 7.0, 1.1, CYAN, Align::Centre);
    let list = sensors(asw, crew, owner);
    if list.is_empty() {
        stroke_text_at(cv, 320.0, 220.0, "NO SENSORS IN THE WATER", 10.0, 1.4, GREY, Align::Centre);
        stroke_text_at(cv, 320.0, 246.0, "DROP A BUOY OR DIP THE SONAR", 8.0, 1.2, DARK, Align::Centre);
        return None;
    }
    // The picture under the canvas: the waterfalls (the rest black).
    let (pw, ph, s) = (cv.pw(), cv.ph(), cv.scale as f32);
    let mut pic = prev.filter(|p| p.w == pw && p.h == ph).unwrap_or_else(|| {
        Arc::new(Picture { w: pw, h: ph, rgba: vec![0; pw * ph * 4] })
    });
    let px = MakeMut::make_mut_or_new(&mut pic, pw, ph);
    px.rgba.iter_mut().enumerate().for_each(|(i, v)| *v = if i % 4 == 3 { 255 } else { 0 });
    for (k, &(id, kind)) in list.iter().enumerate() {
        let (top, h) = row_band(k, list.len());
        let selected = ui.sensor == Some(id);
        cv.rect(LEFT as i32, top as i32 + 1, (RIGHT - LEFT) as i32, h as i32 - 2, if selected { YELLOW } else { DARK });
        let falls = ui.falls.get(&id);
        // The waterfall: transparent on the canvas, drawn into the picture.
        let (x0, x1) = (LEFT + HEADER, RIGHT - 2.0);
        let (y0, y1) = (top + 3.0, top + h - 14.0);
        cv.fill_rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, 0);
        if kind == Some(BuoyKind::Bt) {
            cv.fill_rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, BLACK);
            profile(cv, crew.layer, x0, y0, x1, y1);
        } else if let Some(w) = falls {
            // A row a texel: four reports a second.
            let rows = (((y1 - y0) * s) as usize).min(w.rows.len());
            for py in 0..rows {
                let row = &w.rows[py];
                let y = (y0 * s) as usize + py;
                for pxx in 0..((x1 - x0) * s) as usize {
                    let b = pxx * BINS / ((x1 - x0) * s) as usize;
                    let v = row[b.min(BINS - 1)];
                    let x = (x0 * s) as usize + pxx;
                    let i = (y * pw + x) * 4;
                    if i + 3 < px.rgba.len() {
                        px.rgba[i] = v / 7;
                        px.rgba[i + 1] = v;
                        px.rgba[i + 2] = v / 3;
                    }
                }
            }
        }
        // The listening cursor, and what is near it.
        if let Some(&brg) = ui.cursor.get(&id).filter(|_| kind != Some(BuoyKind::Bt)) {
            let x = x0 + (x1 - x0) * brg / 360.0;
            cv.stroke(Vec2::new(x, y0), Vec2::new(x, y1), 1.2, YELLOW);
            let near = falls.and_then(|w| {
                w.last.lines.iter().find(|l| l.bearing.is_some_and(|b| bearing_off(b as f32, brg).abs() <= 8.0))
            });
            let t = match near {
                Some(l) => format!("{brg:03.0} {:+}", l.snr),
                None => format!("{brg:03.0}"),
            };
            let tx = if x > x1 - 60.0 { x - 4.0 } else { x + 4.0 };
            let al = if x > x1 - 60.0 { Align::Right } else { Align::Left };
            stroke_text_at(cv, tx, y0 + 7.0, &t, 6.0, 1.0, YELLOW, al);
        }
        // Its marks: a tick at the top of the waterfall at each, its letter (grey: lost).
        if kind != Some(BuoyKind::Bt) {
            for m in ui.marks.iter().filter(|m| m.sensor == id) {
                let x = x0 + (x1 - x0) * m.bearing / 360.0;
                let col = if m.lost(ui.t) { GREY } else { CYAN };
                cv.triangle([Vec2::new(x - 4.0, y0), Vec2::new(x + 4.0, y0), Vec2::new(x, y0 + 6.0)], col);
                cv.dashed(Vec2::new(x, y0 + 6.0), Vec2::new(x, y1), 3.0, 5.0, 0.8, col);
                stroke_text_at(cv, x + 5.0, y0 + 14.0, &m.letter().to_string(), 7.0, 1.2, col, Align::Left);
            }
        }
        // The bearing scale under it (true).
        if kind != Some(BuoyKind::Bt) {
            for (deg, label) in [(0, "000"), (90, "090"), (180, "180"), (270, "270"), (360, "360")] {
                let x = x0 + (x1 - x0) * deg as f32 / 360.0;
                cv.line(x as i32, y1 as i32, x as i32, y1 as i32 + 3, GREY);
                let align = match deg {
                    0 => Align::Left,
                    360 => Align::Right,
                    _ => Align::Centre,
                };
                stroke_text_at(cv, x, y1 + 8.0, label, 5.0, 1.0, GREY, align);
            }
        }
        // The header.
        let b = asw.buoys.iter().find(|b| b.id == id);
        let name = match kind {
            Some(k) => format!("B{} {}", id, k.name()),
            None => "DIP AQS-13F".into(),
        };
        let depth = b.map_or(crew.dip.depth, |b| b.depth);
        stroke_text_at(cv, LEFT + 6.0, top + 12.0, &name, 8.0, 1.3, if selected { YELLOW } else { GREEN }, Align::Left);
        let active = b.map_or(crew.dip.active, |b| b.active);
        let mode = match kind {
            Some(BuoyKind::Difar) => "PAS DIR".to_string(),
            Some(BuoyKind::Bt) => "BATHY".to_string(),
            Some(BuoyKind::Dicass) => format!("{} {}P", if active { "ACT" } else { "PAS" }, b.map_or(0, |b| b.pings)),
            None => (if active { "ACT" } else { "PAS" }).to_string(),
        };
        stroke_text_at(cv, LEFT + 6.0, top + 26.0, &format!("{depth:.0}M {mode}"), 7.0, 1.1, GREEN, Align::Left);
        if let Some(w) = falls
            && let Some(l) = w.last.lines.first()
        {
            let mut t = format!("SNR {:+}", l.snr);
            if let Some(br) = l.bearing {
                t.push_str(&format!(" BRG {br:03}"));
            }
            stroke_text_at(cv, LEFT + 6.0, top + 40.0, &t, 7.0, 1.1, YELLOW, Align::Left);
            if let Some(r) = l.range {
                let t = format!("RNG {:.1}KM DOP {:+}", r as f32 / 1000.0, l.doppler);
                stroke_text_at(cv, LEFT + 6.0, top + 54.0, &t, 7.0, 1.1, RED, Align::Left);
            }
        } else if kind != Some(BuoyKind::Bt) {
            stroke_text_at(cv, LEFT + 6.0, top + 40.0, "NO CONTACT", 7.0, 1.1, DARK, Align::Left);
        }
        // Its range from the aircraft.
        if let (Some(b), Some(here)) = (b, here) {
            let d = (b.at[0] - here[0]).hypot(b.at[1] - here[1]);
            stroke_text_at(cv, LEFT + HEADER - 8.0, top + 12.0, &format!("{:.1}KM", d / 1000.0), 6.0, 1.0, GREY, Align::Right);
        }
        if ui.audio == Some(id) {
            stroke_text_at(cv, LEFT + HEADER - 8.0, top + 26.0, "AUD", 6.0, 1.0, CYAN, Align::Right);
        }
        if h >= 60.0 {
            for (btn, r) in row_buttons(top, h, kind) {
                cv.rect(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32, GREY);
                let label = match btn {
                    RowButton::Mode if active => "LISTEN",
                    RowButton::Mode => "PING",
                    RowButton::Scuttle => "SCUTTLE",
                };
                stroke_text_at(cv, r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 + 1.0, label, 6.0, 1.0, WHITE, Align::Centre);
            }
        }
    }
    Some(pic)
}

/// A BT buoy's temperature profile: warm and steady down to the layer, colder under it.
fn profile(cv: &mut Canvas, layer: Option<u16>, x0: f32, y0: f32, x1: f32, y1: f32) {
    stroke_text_at(cv, x0 + 6.0, y0 + 8.0, "TEMP", 6.0, 1.0, GREY, Align::Left);
    let Some(layer) = layer else {
        stroke_text_at(cv, (x0 + x1) / 2.0, (y0 + y1) / 2.0, "PROBE SINKING", 8.0, 1.2, GREY, Align::Centre);
        return;
    };
    let max_depth = 400.0;
    let y = |d: f32| y0 + (y1 - y0) * d / max_depth;
    let x = |t: f32| x0 + 40.0 + (x1 - x0 - 60.0) * (t - 4.0) / 24.0;
    let l = layer as f32;
    let pts: Vec<Vec2> = [0.0, l * 0.9, l * 1.1, l + 60.0, max_depth]
        .iter()
        .zip([24.0, 23.5, 18.0, 12.0, 8.0])
        .map(|(&d, t)| Vec2::new(x(t), y(d)))
        .collect();
    cv.polyline(&pts, 1.5, CYAN);
    cv.dashed(Vec2::new(x0 + 30.0, y(l)), Vec2::new(x1 - 10.0, y(l)), 6.0, 4.0, 1.0, YELLOW);
    stroke_text_at(cv, x1 - 12.0, y(l) - 6.0, &format!("LAYER {layer}M"), 6.0, 1.0, YELLOW, Align::Right);
}

/// `Arc::get_mut`, or a fresh picture when it is shared.
trait MakeMut {
    fn make_mut_or_new(this: &mut Self, w: usize, h: usize) -> &mut Picture;
}

impl MakeMut for Arc<Picture> {
    fn make_mut_or_new(this: &mut Self, w: usize, h: usize) -> &mut Picture {
        if Arc::get_mut(this).is_none() {
            *this = Arc::new(Picture { w, h, rgba: vec![0; w * h * 4] });
        }
        Arc::get_mut(this).expect("unique")
    }
}
