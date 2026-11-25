//! The EA-18G's JAM page (`avionics::Format::Jam`): sets its ALQ-99 pods on the enemy's radars
//! (`stealth_sim::jamming`), from the emitters its ALQ-218 receivers hear.
//!
//! - The left bezel buttons pick one of the five strongest radars heard (their RWR symbol and
//!   range beside them, a J by those jammed).
//! - The top row: POD1 to POD3 set the picked radar on that pod (pressed again with the same
//!   radar picked, the pod goes off it); the fourth is XMIT, the transmitters on or off; the
//!   fifth makes the page the SOI.
//! - The right bezel buttons FWD1 to FWD3 set that pod on the sector ahead of the aircraft
//!   (60 degrees wide, turning with it).
//!
//! Only as many pods as the aircraft carries work.

use stealth_sim::EmitterSource;
use stealth_sim::jamming::{JamInput, JamTarget, PODS};

use crate::avionics::Format;
use crate::canvas::Canvas;
use crate::font::stroke_text;
use crate::mfd::{MfdCmd, MfdParams, MfdState, OsbFn};

/// A bezel button of the JAM page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JamOsb {
    /// The k-th radar on the list (by its site).
    Pick(u16),
    Pod(u8),
    Ahead(u8),
    Xmit,
}

/// What a pod is set on, as the page keeps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodSet {
    Site(u16),
    /// The sector ahead (its bearing the aircraft's heading each moment).
    Ahead,
}

/// The page's state.
#[derive(Debug, Clone, Default)]
pub struct JamPage {
    pub on: bool,
    pub pods: [Option<PodSet>; PODS],
    /// The radar picked (its site).
    pub pick: Option<u16>,
    /// The pods aboard.
    pub loaded: u8,
    /// The radars on the left buttons (their sites), as heard this frame.
    pub list: Vec<u16>,
}

impl JamPage {
    /// The report to the simulation, the aircraft heading `heading` (BAM).
    pub fn input(&self, heading: u16) -> JamInput {
        let pods = self.pods.map(|p| {
            p.map(|p| match p {
                PodSet::Site(s) => JamTarget::Site(s),
                PodSet::Ahead => JamTarget::Sector(heading),
            })
        });
        JamInput { on: self.on, pods }
    }

    /// A bezel button pressed.
    pub fn apply(&mut self, o: JamOsb) {
        match o {
            JamOsb::Pick(s) => self.pick = Some(s),
            JamOsb::Xmit => self.on = !self.on,
            JamOsb::Pod(k) => {
                let k = (k as usize).min(PODS - 1);
                let want = self.pick.map(PodSet::Site);
                self.pods[k] = if self.pods[k] == want { None } else { want.or(self.pods[k]) };
            }
            JamOsb::Ahead(k) => {
                let k = (k as usize).min(PODS - 1);
                self.pods[k] =
                    if self.pods[k] == Some(PodSet::Ahead) { None } else { Some(PodSet::Ahead) };
            }
        }
    }
}

/// The jammer pods aboard: the player's own, or in another's aircraft (its electronic warfare
/// officer) its pilot's, as the crew is told.
pub fn pods_aboard(stores: &crate::hud::Stores, combat: Option<&crate::combat::Combat>) -> u32 {
    let ridden = combat.and_then(|c| {
        let me = c.sim.active_pilot();
        c.sim.crew_view(me).filter(|v| v.owner != me)
    });
    // (Its stores by station as everyone sees them: the crew's view has only the first six.)
    match ridden.zip(combat) {
        Some((v, c)) => c.sim.remote_pilots().iter().find(|p| p.id == v.owner).map_or(0, |p| {
            p.stores
                .iter()
                .filter(|&&(w, _)| w as i32 == stealth_sim::jamming::ALQ99_WEAPON)
                .map(|&(_, n)| n as u32)
                .sum()
        }),
        None => stores
            .bays
            .iter()
            .filter(|b| b.weapon == stealth_sim::jamming::ALQ99_WEAPON)
            .map(|b| b.count)
            .sum(),
    }
}

/// Mirrors the pods aboard onto the page, and the radars heard for its buttons.
pub fn update(
    stores: bevy::prelude::Res<crate::hud::Stores>,
    combat: Option<bevy::prelude::Res<crate::combat::Combat>>,
    emitters: bevy::prelude::Res<crate::rwr::Emitters>,
    mut state: bevy::prelude::ResMut<MfdState>,
) {
    let pods = pods_aboard(&stores, combat.as_deref());
    let list: Vec<u16> = radars(&emitters).into_iter().map(site).collect();
    let page = &state.av.jam;
    if page.loaded as u32 != pods || page.list != list {
        let page = &mut state.av.jam;
        page.loaded = pods.min(255) as u8;
        page.list = list;
    }
}

/// The radars on the list: the sites heard, strongest (most urgent) first.
fn radars(p: &crate::rwr::Emitters) -> Vec<&crate::rwr::EmitterView> {
    let mut v: Vec<_> =
        p.list.iter().filter(|e| matches!(e.source, EmitterSource::Site(_))).collect();
    v.sort_by_key(|e| -e.priority);
    v.truncate(5);
    v
}

fn site(e: &crate::rwr::EmitterView) -> u16 {
    match e.source {
        EmitterSource::Site(i) => i as u16,
        _ => u16::MAX,
    }
}

const POD_LABELS: [&str; PODS] = ["POD1", "POD2", "POD3"];
const AHEAD_LABELS: [&str; PODS] = ["FWD1", "FWD2", "FWD3"];

/// The bezel button `slot`'s label, function and lit state on the JAM page.
pub fn osb(slot: u8, s: &MfdState) -> Option<(&'static str, OsbFn, bool)> {
    let page = &s.av.jam;
    let cmd = |o| OsbFn::Cmd(MfdCmd::Jam(o));
    let pods = (page.loaded as usize).min(PODS);
    Some(match slot {
        0..=2 if (slot as usize) < pods => {
            let k = slot as usize;
            let lit = page.pods[k].is_some();
            (POD_LABELS[k], cmd(JamOsb::Pod(slot)), lit)
        }
        3 => ("XMIT", cmd(JamOsb::Xmit), page.on),
        4 => ("SOI", OsbFn::Cmd(MfdCmd::Soi(s.av.shown(Format::Jam)?)), {
            s.av.soi_format() == Some(Format::Jam)
        }),
        10..=14 => {
            let id = *page.list.get(slot as usize - 10)?;
            ("", cmd(JamOsb::Pick(id)), page.pick == Some(id))
        }
        15..=17 if ((slot - 15) as usize) < pods => {
            let k = (slot - 15) as usize;
            (AHEAD_LABELS[k], cmd(JamOsb::Ahead(slot - 15)), page.pods[k] == Some(PodSet::Ahead))
        }
        _ => return None,
    })
}

/// Colours (palette).
const BLACK: u8 = 5;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const AMBER: u8 = 18;
/// Height of the bezel label bands (`avionics::BAND`).
const BAND: f32 = 16.0;

/// Draws the page.
pub fn draw(p: &MfdParams, c: &mut Canvas) {
    let page = &p.state.av.jam;
    let (w, h) = (c.w as f32, c.h as f32);
    let text = |c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, col: u8| {
        stroke_text(c, x, y, s, size, 1.0, col)
    };
    c.clear(BLACK);
    let jammed = |id: u16| page.pods.iter().any(|p| *p == Some(PodSet::Site(id)));
    // The radars beside the left buttons: symbol, range, J when jammed.
    for (k, e) in radars(&p.emitters).into_iter().enumerate() {
        let row = h * (2 * k as i32 + 1) as f32 / 10.0;
        let y = row.clamp(BAND + 6.0, h - BAND - 6.0) + 3.0;
        let id = site(e);
        let col = if page.pick == Some(id) { AMBER } else { GREEN };
        let mark = if jammed(id) { " J" } else { "" };
        text(c, 6.0, y, &format!("{} {:.0}KM{mark}", e.label, e.range_km), 7.0, col);
    }
    // The transmitters and each pod's setting, in the middle.
    let x0 = w * 0.42;
    let (status, col) = if page.on { ("XMIT", AMBER) } else { ("STBY", GREEN) };
    text(c, x0, BAND + 22.0, &format!("JAM {status}"), 9.0, col);
    let pods = (page.loaded as usize).min(PODS);
    for k in 0..PODS {
        let s = if k >= pods {
            format!("POD{} ---", k + 1)
        } else {
            match page.pods[k] {
                None => format!("POD{} OFF", k + 1),
                Some(PodSet::Ahead) => format!("POD{} FWD", k + 1),
                Some(PodSet::Site(id)) => {
                    let label = p
                        .emitters
                        .list
                        .iter()
                        .find(|e| site(e) == id)
                        .map_or("LOST".to_string(), |e| e.label.clone());
                    format!("POD{} {label}", k + 1)
                }
            }
        };
        let col = if k >= pods {
            GREY
        } else if page.pods[k].is_some() && page.on {
            AMBER
        } else {
            GREEN
        };
        text(c, x0, BAND + 44.0 + k as f32 * 13.0, &s, 7.0, col);
    }
    if pods == 0 {
        text(c, x0, h - BAND - 20.0, "NO PODS", 7.0, GREY);
    }
}
