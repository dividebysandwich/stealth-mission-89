//! The F/A-18E's SLAM page (`avionics::Format::Slam`): programs the SLAM-ERs' four data link
//! channels and flies the missiles once their seekers are on (`stealth_sim::slam`).
//!
//! - The side bezel buttons pick the channel: CH1 and CH2 down the left, CH3 and CH4 down the
//!   right, each with its status beside it (RDY, the seconds to its seeker, SKR once the
//!   seeker is on, nothing without a missile to fire).
//! - The top row sets the selected channel's program: FLT (cruise altitude LO, MED, HI), TGT
//!   (the mission's primary or secondary target), SKR- / SKR+ (the range to the target at which
//!   the seeker comes on); the fifth makes the page the SOI.
//! - In flight, before its seeker is on, the page shows the selected missile's time to seeker
//!   (TTS). Once it is on the page shows its seeker's FLIR picture (the tracking camera rides
//!   the missile) and the slew controls steer it while the page is the SOI; let go, it locks
//!   onto the target nearest the crosshair (LOCK), else holds the ground there (GND).

use bevy::prelude::*;
use stealth_sim::slam::{CHANNELS, FLT_LABELS, SEEKER_NM, SlamInput, SlamProgram};

use crate::avionics::Format;
use crate::canvas::Canvas;
use crate::font::stroke_text;
use crate::mfd::{MfdCmd, MfdParams, MfdState, OsbFn};

/// A bezel button of the SLAM page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlamOsb {
    Channel(u8),
    Flt,
    Target,
    Seeker(i8),
}

/// A channel's missile in flight, mirrored from the simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlamRound {
    /// Its projectile slot.
    pub slot: usize,
    /// Seconds to its seeker coming on.
    pub tts: f32,
    pub active: bool,
    /// It has snapped onto a target.
    pub snapped: bool,
    /// Metres to what it flies into, once its seeker is on.
    pub range_m: Option<f32>,
}

/// The page's state.
#[derive(Debug, Clone)]
pub struct SlamPage {
    /// The channel selected (0..4).
    pub sel: u8,
    pub prog: [SlamProgram; CHANNELS],
    pub rounds: [Option<SlamRound>; CHANNELS],
    /// SLAM-ERs still on the pylons.
    pub loaded: u8,
    /// The slew steering the selected missile (x right, y up; -1..1).
    pub steer: Vec2,
}

impl Default for SlamPage {
    /// Channels 1 and 3 on the primary target, 2 and 4 on the secondary.
    fn default() -> Self {
        let mut prog = [SlamProgram::default(); CHANNELS];
        for (k, p) in prog.iter_mut().enumerate() {
            p.target = (k % 2) as u8;
        }
        SlamPage { sel: 0, prog, rounds: [None; CHANNELS], loaded: 0, steer: Vec2::ZERO }
    }
}

impl SlamPage {
    /// The selected missile with its seeker on: its picture is up.
    pub fn picture(&self) -> Option<SlamRound> {
        self.rounds[self.sel as usize % CHANNELS].filter(|r| r.active)
    }

    /// The report to the simulation.
    pub fn input(&self) -> SlamInput {
        let q = |v: f32| (v.clamp(-1.0, 1.0) * 127.0).round() as i8;
        SlamInput { sel: self.sel, prog: self.prog, steer: (q(self.steer.x), q(self.steer.y)) }
    }

    /// A bezel button pressed.
    pub fn apply(&mut self, o: SlamOsb) {
        let p = &mut self.prog[self.sel as usize % CHANNELS];
        match o {
            SlamOsb::Channel(k) => self.sel = k.min(CHANNELS as u8 - 1),
            SlamOsb::Flt => p.flt = (p.flt + 1) % FLT_LABELS.len() as u8,
            SlamOsb::Target => p.target = 1 - p.target.min(1),
            SlamOsb::Seeker(d) => {
                let at = SEEKER_NM.iter().position(|&n| n >= p.seeker_nm).unwrap_or(0) as i32;
                let k = (at + d as i32).clamp(0, SEEKER_NM.len() as i32 - 1);
                p.seeker_nm = SEEKER_NM[k as usize];
            }
        }
    }
}

const CHANNEL_LABELS: [&str; CHANNELS] = ["CH1", "CH2", "CH3", "CH4"];

/// The bezel button `slot`'s label, function and lit state on the SLAM page.
pub fn osb(slot: u8, s: &MfdState) -> Option<(&'static str, OsbFn, bool)> {
    let page = &s.av.slam;
    let p = page.prog[page.sel as usize % CHANNELS];
    let cmd = |o| OsbFn::Cmd(MfdCmd::Slam(o));
    // A program is set before launch: the channel's missile in flight keeps its own.
    let flying = page.rounds[page.sel as usize % CHANNELS].is_some();
    Some(match slot {
        0 if !flying => (FLT_LABELS[p.flt.min(2) as usize], cmd(SlamOsb::Flt), false),
        1 if !flying => (if p.target == 0 { "PRI" } else { "SEC" }, cmd(SlamOsb::Target), false),
        2 if !flying => ("SKR-", cmd(SlamOsb::Seeker(-1)), false),
        3 if !flying => ("SKR+", cmd(SlamOsb::Seeker(1)), false),
        4 => ("SOI", OsbFn::Cmd(MfdCmd::Soi(s.av.shown(Format::Slam)?)), {
            s.av.soi_format() == Some(Format::Slam)
        }),
        10 | 11 | 15 | 16 => {
            let k = channel_of(slot)?;
            (CHANNEL_LABELS[k as usize], cmd(SlamOsb::Channel(k)), page.sel == k)
        }
        _ => return None,
    })
}

/// The channel on side button `slot`: CH1/CH2 on the left, CH3/CH4 on the right.
fn channel_of(slot: u8) -> Option<u8> {
    match slot {
        10 => Some(0),
        11 => Some(1),
        15 => Some(2),
        16 => Some(3),
        _ => None,
    }
}

/// Colours (palette).
const BLACK: u8 = 5;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const AMBER: u8 = 18;
/// Height of the bezel label bands (`avionics::BAND`).
const BAND: f32 = 16.0;

/// A channel's status beside its button.
fn status(page: &SlamPage, k: usize) -> String {
    match page.rounds[k] {
        Some(r) if r.active => "SKR".into(),
        Some(r) => format!("{:.0}S", r.tts.ceil()),
        None if page.loaded > 0 => "RDY".into(),
        None => "---".into(),
    }
}

/// Draws the page (`live`: the selected missile's seeker picture shows through).
pub fn draw(p: &MfdParams, c: &mut Canvas, live: bool) {
    let page = &p.state.av.slam;
    let (w, h) = (c.w as f32, c.h as f32);
    let text = |c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, col: u8| {
        stroke_text(c, x, y, s, size, 1.0, col)
    };
    // Channel statuses beside their buttons.
    let status_at = |c: &mut Canvas, k: usize, slot: u8| {
        let s = status(page, k);
        let row = h * (2 * (slot as i32 % 5) + 1) as f32 / 10.0;
        let y = row.clamp(BAND + 6.0, h - BAND - 6.0) + 9.0;
        let width = crate::font::stroke_width(&s, 6.0);
        let x = if slot < 15 { 6.0 } else { w - 6.0 - width };
        let col = if page.rounds[k].is_some_and(|r| r.active) { AMBER } else { GREEN };
        text(c, x, y, &s, 6.0, col);
    };
    let sel = page.sel as usize % CHANNELS;
    if live && let Some(r) = page.picture() {
        c.clear(0);
        let ink = p.camera_text_colour();
        // Crosshair with a gap, a box once locked onto a target.
        let o = Vec2::new(w / 2.0, h / 2.0);
        for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            c.stroke(o + d * 5.0, o + d * 24.0, 1.3, ink);
        }
        if r.snapped {
            c.frame(o - Vec2::splat(9.0), o + Vec2::splat(9.0), 1.3, ink);
        }
        let y0 = BAND + 4.0;
        text(c, 40.0, y0, &format!("SLAM {}", CHANNEL_LABELS[sel]), 7.0, ink);
        let mode = if r.snapped { "LOCK" } else { "GND" };
        text(c, w - 40.0 - crate::font::stroke_width(mode, 7.0), y0, mode, 7.0, ink);
        if let Some(m) = r.range_m {
            let s = format!("RNG {:.1}", m / 1852.0);
            text(c, 40.0, h - BAND - 14.0, &s, 7.0, ink);
            let tti = format!("TTI {:.0}", (m as f64 * WU_M / stealth_sim::slam::SPEED).ceil());
            let tw = crate::font::stroke_width(&tti, 7.0);
            text(c, w - 40.0 - tw, h - BAND - 14.0, &tti, 7.0, ink);
        }
        text(c, 40.0, h - BAND - 24.0, "FLIR", 6.0, ink);
        for (k, slot) in [(0, 10), (1, 11), (2, 15), (3, 16)] {
            status_at(c, k, slot);
        }
        return;
    }
    c.clear(BLACK);
    let x0 = w * 0.25;
    text(c, x0, BAND + 22.0, "SLAM-ER", 9.0, GREEN);
    let prog = page.prog[sel];
    let lines = [
        format!("{}", CHANNEL_LABELS[sel]),
        format!("FLT {}", FLT_LABELS[prog.flt.min(2) as usize]),
        format!("TGT {}", if prog.target == 0 { "PRI" } else { "SEC" }),
        format!("SKR {} NM", prog.seeker_nm),
    ];
    for (i, l) in lines.iter().enumerate() {
        text(c, x0, BAND + 40.0 + i as f32 * 12.0, l, 7.0, GREEN);
    }
    // The selected missile in flight: seconds to its seeker, big.
    let y = BAND + 94.0;
    match page.rounds[sel] {
        Some(r) => {
            text(c, x0, y, &format!("TTS {:.0}", r.tts.ceil()), 12.0, AMBER);
        }
        None => {
            let s = if page.loaded > 0 {
                format!("RDY  {} LOADED", page.loaded)
            } else {
                "NO STORE".into()
            };
            text(c, x0, y, &s, 7.0, if page.loaded > 0 { GREEN } else { GREY });
        }
    }
    for (k, slot) in [(0, 10), (1, 11), (2, 15), (3, 16)] {
        status_at(c, k, slot);
    }
}

/// World units per metre.
const WU_M: f64 = 1.0 / stealth_sim::WU_METRES;

/// Mirrors the SLAM-ERs in flight onto the page, brings the page up (and makes it the SOI) when
/// a missile's seeker comes on, and takes the slew for the selected missile.
pub fn update(
    combat: Option<Res<crate::combat::Combat>>,
    controls: Res<crate::input::FlightControls>,
    waypoints: Res<crate::hud::Waypoints>,
    mut state: ResMut<MfdState>,
) {
    let Some(combat) = combat else { return };
    if !state.modern || !state.av.offered(Format::Slam) {
        return;
    }
    let sim = &combat.sim;
    let fine = stealth_sim::ROUND_FINE as f64;
    let mut rounds = [None; CHANNELS];
    for (slot, m) in sim.slam_rounds() {
        let (Some(g), Some(f)) = (m.slam, m.flight) else { continue };
        let range_m = g.aim.map(|(x, y, z)| {
            let d = [
                x as f64 - f.x as f64 / fine,
                y as f64 - f.y as f64 / fine,
                z as f64 - f.alt as f64 / fine,
            ];
            ((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / WU_M) as f32
        });
        let tts = stealth_sim::slam::time_to_seeker(&m).unwrap_or(0.0) as f32;
        rounds[(g.ch as usize).min(CHANNELS - 1)] =
            Some(SlamRound { slot, tts, active: g.active, snapped: g.snapped, range_m });
    }
    let (bays, _) = sim.bays();
    let loaded = bays
        .iter()
        .filter(|b| b.0 == stealth_sim::SLAMER_WEAPON)
        .map(|b| b.1.max(0))
        .sum::<i32>()
        .min(255) as u8;
    let page = &state.av.slam;
    // A seeker just came on: its picture up, the selected one's unless that is on already.
    let came_on = (0..CHANNELS).find(|&k| {
        rounds[k].is_some_and(|r| r.active) && !page.rounds[k].is_some_and(|r| r.active)
    });
    let changed = page.rounds != rounds || page.loaded != loaded;
    if changed {
        let av = &mut state.av;
        av.slam.rounds = rounds;
        av.slam.loaded = loaded;
        if let Some(k) = came_on {
            if av.slam.picture().is_none() {
                av.slam.sel = k as u8;
            }
            av.show(Format::Slam);
            av.soi = Some(Format::Slam);
        }
    }
    let steer = if state.av.soi_format() == Some(Format::Slam)
        && state.av.slam.picture().is_some()
        && !crate::mfd::editing_waypoints(&state, &waypoints)
    {
        controls.slew
    } else {
        Vec2::ZERO
    };
    if state.av.slam.steer != steer {
        state.av.slam.steer = steer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_start_on_both_targets_and_step_their_seeker_range() {
        let mut p = SlamPage::default();
        assert_eq!(p.prog.map(|p| p.target), [0, 1, 0, 1]);
        p.apply(SlamOsb::Channel(2));
        p.apply(SlamOsb::Seeker(1));
        p.apply(SlamOsb::Flt);
        p.apply(SlamOsb::Target);
        assert_eq!(p.prog[2], SlamProgram { flt: 1, seeker_nm: 8, target: 1 });
        assert_eq!(p.prog[0], SlamProgram::default());
        for _ in 0..9 {
            p.apply(SlamOsb::Seeker(-1));
        }
        assert_eq!(p.prog[2].seeker_nm, SEEKER_NM[0]);
        p.steer = Vec2::new(1.0, -0.5);
        assert_eq!(p.input().steer, (127, -64));
    }
}
