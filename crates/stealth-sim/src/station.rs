//! Campaign: friendly air defence manned by a player. A missile battery (Patriot, NASAMS) is
//! fought from its radar's console (the radar's track, the launchers around it), a Gepard from
//! its turret.
//!
//! * The player locks a track; missiles fly at it **guided** (the radar steers them, hard to
//!   decoy, one target at a time: a guided missile whose station locks something else or
//!   switches to semi-active becomes semi-active) or **semi-active** (they home on the radar's
//!   illumination of the target they were fired at, several targets at once, easier to decoy).
//! * Launchers hold four missiles (a NASAMS launcher six); an empty one is refilled from the battery's stockpile, which
//!   takes [`REFILL_SECONDS`] of *real* time ([`Sim::tick_refills`], called by the host with
//!   wall clock seconds). Unmanned, the AI starts the refills itself.
//! * A Gepard fires bursts of [`BURST`] programmed airburst rounds, out to [`GEPARD_REACH`].
//!   Without a lock its turret is slewed by hand ([`StationCmd::Slew`]) and a burst is
//!   [`BURST`] tracers along the barrels.
//! * A drone command center launches and flies drones (`uav.rs`).
//! * A station whose radar (or Gepard) is destroyed is gone: [`Sim::station_of`] is None.
//! * The radar can be switched off ([`StationCmd::Radar`]): no lock and no guidance (a Gepard
//!   still fires along its barrels), but an anti-radiation missile flying at it may lose it
//!   ([`arm_lost`], the further out the likelier).

use serde::{Deserialize, Serialize};
use stealth_i18n::{Text, text};

use crate::Sim;
use crate::aaa::{GroundKind, Shell, lead};
use crate::campaign::{
    AiShell, M_PER_WU, MAX_SHELLS, PATRIOT_LINK, add, flight_metres, jitter, len, mul, sub,
};
use crate::math::approx;
use crate::types::*;
use crate::uav::{Seen, Uav, UavKind, UavMode, UavWp};

/// Missiles in a Patriot battery's stockpile, beyond those on its launchers.
pub const PATRIOT_STOCK: i32 = 8;
/// Missiles in a NASAMS battery's stockpile (three launchers' worth).
pub const NASAMS_STOCK: i32 = 18;
/// A Gepard's rounds ready, and in its stockpile.
pub const GEPARD_ROUNDS: i32 = 640;
pub const GEPARD_STOCK: i32 = 640;
/// A drone command center's stockpile: surveillance drones, attack drones.
pub const STORK_STOCK: i32 = 4;
pub const FP2_STOCK: i32 = 8;
pub const INTERCEPTOR_STOCK: i32 = 6;
/// Real seconds to refill a launcher (or a Gepard).
pub const REFILL_SECONDS: f32 = 240.0;
/// The PAC-3's reach (m): no shots at a track further out (it gets less against a target
/// that makes it turn, `guidance`).
pub const PATRIOT_MAX_RANGE: f64 = 120_000.0;
/// The ground launched AIM-120's reach (m), the AMRAAM-ER's.
pub const NASAMS_MAX_RANGE: f64 = 40_000.0;
/// Seconds between a manned launcher's missiles (the AI waits longer, `PATRIOT_INTERVAL`).
pub const LAUNCH_INTERVAL: i32 = 2;
/// A Gepard burst: rounds, and seconds it takes (two guns, 550 rounds a minute each).
pub const BURST: i32 = 25;
const BURST_SECONDS: f64 = 1.4;
/// A Gepard's programmed rounds: muzzle velocity (m/s), farthest burst point (m).
pub const GEPARD_V0: f64 = 1175.0;
pub const GEPARD_REACH: f64 = 8000.0;
/// A Gepard turret's fastest slew by hand (rad/s): traverse, elevation; its elevation limits.
pub const GEPARD_TRAVERSE: f32 = 1.0;
pub const GEPARD_ELEVATE: f32 = 0.8;
pub const GEPARD_EL: (f32, f32) = (-0.08, 1.48);
/// Hand-fired rounds self-destruct this far out (m); a round passes close to an aircraft no
/// more than this far off its line of fire (rad).
pub const MANUAL_RANGE: f64 = 8000.0;
const MANUAL_CONE: f64 = 0.05;
/// An airburst round's fragments kill within this (m) with this chance (per mille).
const AHEAD_RADIUS: f64 = 8.0;
const AHEAD_PK: i32 = 150;
/// A missile this close to its target (m) meets its countermeasures, which decoy it with this
/// chance (per mille).
const SPOOF_RANGE: f64 = 3000.0;
const SPOOF_GUIDED: i32 = 80;
const SPOOF_SEMI: i32 = 300;
/// A spoofed missile's target (none: it flies on until its fuel is gone).
const NO_TARGET: usize = usize::MAX;
/// The chance (per mille) an anti-radiation missile loses a radar switched off: this close,
/// rising to `ARM_LOST_FAR` from `ARM_FAR` metres out.
const ARM_LOST_NEAR: i32 = 150;
const ARM_LOST_FAR: i32 = 750;
const ARM_FAR: f64 = 15_000.0;
/// A missile that lost its radar flies on this far off its line (rad, at most).
const ARM_ERROR: f64 = 0.12;

/// How a missile from a manned console is guided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Guidance {
    #[default]
    Guided,
    SemiActive,
}

/// What a player at a console does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StationCmd {
    /// Man a friendly Patriot or NASAMS radar, Gepard or drone command center (AAA unit).
    Man {
        unit: u16,
    },
    Leave,
    /// Lock a track (flight unit), or drop the lock.
    Lock(Option<u16>),
    /// Switch the radar on or off.
    Radar(bool),
    Mode(Guidance),
    /// A missile at the locked track from a launcher (None: the readiest).
    Fire {
        launcher: Option<u16>,
    },
    /// Gepard: a burst at the locked track (without a lock, along the barrels).
    Burst,
    /// Gepard without a lock: slew the turret at these rates (rad/s: traverse right, elevation
    /// up; limited to [`GEPARD_TRAVERSE`], [`GEPARD_ELEVATE`]).
    Slew {
        az: f32,
        el: f32,
    },
    /// Refill an empty launcher (or the Gepard) from the stockpile.
    Refill {
        unit: u16,
    },
    /// Ask for a helicopter with a new stockpile (the campaign's business: it goes on the
    /// mission board).
    RequestResupply,
    /// Drone command center: a drone off the rail to fly a route.
    Launch {
        kind: UavKind,
        route: Vec<UavWp>,
    },
    /// A drone's new route (its id at the center).
    Route {
        uav: u16,
        route: Vec<UavWp>,
    },
    /// A drone flies to waypoint `wp` next.
    Next {
        uav: u16,
        wp: u16,
    },
    /// A drone flown by hand, or back on its route.
    Manual {
        uav: u16,
        on: bool,
    },
    /// Flying by hand: the stick (roll right, pitch up, -1..1), the rudder (nose right, -1..1)
    /// and throttle (0..1).
    Fly {
        uav: u16,
        roll: f32,
        pitch: f32,
        yaw: f32,
        throttle: f32,
    },
}

/// A manned console.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Station {
    /// The Patriot radar or Gepard (AAA unit).
    pub unit: u16,
    pub pilot: u8,
    /// The locked track (flight unit).
    pub lock: Option<u16>,
    pub mode: Guidance,
    /// Gepard: rounds left of the burst being fired, and rounds due this frame (fraction).
    pub burst: i32,
    #[serde(default)]
    pub due: f64,
    /// Gepard: the slew by hand (rad/s, traverse and elevation), and the burst being fired is
    /// along the barrels (there was no lock when it started).
    #[serde(default)]
    pub slew: [f32; 2],
    #[serde(default)]
    pub manual: bool,
    /// The radar is switched off (silent: nothing for an anti-radiation missile to home on).
    #[serde(default)]
    pub radar_off: bool,
}

/// The friendly air defence's consoles and stocks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DefenceState {
    pub stations: Vec<Station>,
    /// Per AAA unit: the stockpile at a Patriot radar (its battery's) or a Gepard; a drone
    /// command center's surveillance drones.
    pub stock: Vec<i32>,
    /// Per AAA unit: a drone command center's attack drones.
    #[serde(default)]
    pub stock2: Vec<i32>,
    /// Per AAA unit: a drone command center's interceptor drones.
    #[serde(default)]
    pub stock3: Vec<i32>,
    /// Per AAA unit: real seconds left of a launcher's or Gepard's refill (0: none).
    pub refill: Vec<f32>,
    /// The drone command centers' drones in the air, the last one's number, and what the
    /// drones have found.
    #[serde(default)]
    pub uavs: Vec<Uav>,
    #[serde(default)]
    pub uav_serial: u16,
    #[serde(default)]
    pub uav_known: Vec<Seen>,
    /// The seats taken in aircraft with more than one (`crew`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub crews: Vec<crate::crew::Crew>,
}

/// A stockpile to keep over a save: where, the unit that keeps it (a Patriot radar, a Gepard,
/// a drone command center), how much (and a drone command center's attack and interceptor
/// drones).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedStock {
    pub x: i32,
    pub y: i32,
    pub kind: GroundKind,
    pub stock: i32,
    pub stock2: i32,
    /// -1: not saved (a save from before the interceptors), the unit keeps its stockpile.
    #[serde(default = "unsaved")]
    pub stock3: i32,
}

fn unsaved() -> i32 {
    -1
}

/// The stockpile a unit starts with (and a resupply brings it back to).
pub fn initial_stock(kind: GroundKind) -> i32 {
    if let Some(b) = kind.battery_radar() {
        return b.stock();
    }
    match kind {
        GroundKind::Gepard => GEPARD_STOCK,
        GroundKind::DroneCenter => STORK_STOCK,
        _ => 0,
    }
}

/// The second stockpile a unit starts with: a drone command center's attack drones.
pub fn initial_stock2(kind: GroundKind) -> i32 {
    if kind == GroundKind::DroneCenter { FP2_STOCK } else { 0 }
}

/// The third stockpile a unit starts with: a drone command center's interceptor drones.
pub fn initial_stock3(kind: GroundKind) -> i32 {
    if kind == GroundKind::DroneCenter { INTERCEPTOR_STOCK } else { 0 }
}

/// A unit's load when full.
fn full_load(kind: GroundKind) -> i32 {
    if let Some(b) = kind.battery_launcher() {
        return b.rounds();
    }
    match kind {
        GroundKind::Gepard => GEPARD_ROUNDS,
        _ => 0,
    }
}

/// A unit a player can man.
pub fn mannable(kind: GroundKind) -> bool {
    kind.battery_radar().is_some() || matches!(kind, GroundKind::Gepard | GroundKind::DroneCenter)
}

impl Sim {
    /// The consoles and stocks (a campaign's).
    pub fn defence(&self) -> Option<&crate::station::DefenceState> {
        self.campaign.as_ref().map(|c| &c.defence)
    }

    pub(crate) fn defence_mut(&mut self) -> &mut DefenceState {
        &mut self.campaign.as_mut().expect("campaign").defence
    }

    /// The console pilot `pilot` mans.
    pub fn station_of(&self, pilot: u8) -> Option<&Station> {
        self.defence()?.stations.iter().find(|s| s.pilot == pilot)
    }

    /// Who mans AAA unit `unit`.
    pub fn manned_by(&self, unit: usize) -> Option<u8> {
        let d = self.defence()?;
        d.stations.iter().find(|s| s.unit as usize == unit).map(|s| s.pilot)
    }

    /// The live launchers a missile battery's radar commands.
    pub fn battery_launchers(&self, radar: usize) -> Vec<usize> {
        let Some(r) = self.units.get(radar) else { return Vec::new() };
        let Some(b) = r.kind.battery_radar() else { return Vec::new() };
        let blue = self.aaa_blue(radar);
        (0..self.units.len())
            .filter(|&k| {
                let u = &self.units[k];
                u.kind == b.launcher()
                    && self.aaa_blue(k) == blue
                    && approx(u.x - r.x, u.y - r.y) < PATRIOT_LINK
            })
            .collect()
    }

    /// The launcher a battery fires from next: the one already begun (fewest missiles left), so
    /// it empties one launcher before it starts on the next. None when none is live, loaded and
    /// done refilling.
    pub fn next_launcher(&self, radar: usize) -> Option<usize> {
        self.battery_launchers(radar)
            .into_iter()
            .filter(|&k| {
                let u = &self.units[k];
                !u.destroyed && u.rounds > 0 && self.refill_left(k) == 0.0
            })
            .min_by_key(|&k| (self.units[k].rounds, k))
    }

    /// The battery radar that commands a launcher.
    pub fn launcher_radar(&self, launcher: usize) -> Option<usize> {
        let l = self.units.get(launcher)?;
        let b = l.kind.battery_launcher()?;
        let blue = self.aaa_blue(launcher);
        (0..self.units.len()).find(|&r| {
            let u = &self.units[r];
            u.kind == b.radar()
                && !u.destroyed
                && self.aaa_blue(r) == blue
                && approx(u.x - l.x, u.y - l.y) < PATRIOT_LINK
        })
    }

    /// Where a unit's stock is kept: a launcher's at its radar, anything else its own.
    fn stock_unit(&self, k: usize) -> Option<usize> {
        if self.units.get(k)?.kind.battery_launcher().is_some() {
            self.launcher_radar(k)
        } else {
            Some(k)
        }
    }

    /// Unit `k`'s stockpile (a launcher's: its battery's).
    pub fn stockpile(&self, k: usize) -> i32 {
        let s = self.stock_unit(k);
        s.and_then(|s| self.defence()?.stock.get(s).copied()).unwrap_or(0)
    }

    /// Unit `k`'s stockpile less what the refills under way will take from it.
    pub fn stockpile_free(&self, k: usize) -> i32 {
        (self.stockpile(k) - self.stock_promised(k)).max(0)
    }

    /// What the refills under way from unit `k`'s stockpile will take from it.
    fn stock_promised(&self, k: usize) -> i32 {
        let Some(s) = self.stock_unit(k) else { return 0 };
        (0..self.units.len())
            .filter(|&j| self.refill_left(j) > 0.0 && self.stock_unit(j) == Some(s))
            .map(|j| (full_load(self.units[j].kind) - self.units[j].rounds).max(0))
            .sum()
    }

    /// Unit `k`'s stockpile has the missiles (rounds) to refill it: a full load beyond what the
    /// refills under way take, or what is left when no other refill is under way.
    fn stock_for_refill(&self, k: usize) -> bool {
        let need = full_load(self.units[k].kind) - self.units[k].rounds;
        let free = self.stockpile_free(k);
        free >= need || (free > 0 && self.stock_promised(k) == 0)
    }

    /// Real seconds left of unit `k`'s refill (0: none).
    pub fn refill_left(&self, k: usize) -> f32 {
        self.defence().and_then(|d| d.refill.get(k).copied()).unwrap_or(0.0)
    }

    /// The defence's lists follow the units (new ones get their stockpiles).
    pub(crate) fn defence_sizes(&mut self) {
        let Some(c) = self.campaign.as_mut() else { return };
        let d = &mut c.defence;
        let n = self.units.len();
        d.stock.truncate(n);
        for u in &self.units[d.stock.len()..] {
            d.stock.push(initial_stock(u.kind));
        }
        d.stock2.truncate(n);
        for u in &self.units[d.stock2.len()..] {
            d.stock2.push(initial_stock2(u.kind));
        }
        d.stock3.truncate(n);
        for u in &self.units[d.stock3.len()..] {
            d.stock3.push(initial_stock3(u.kind));
        }
        d.refill.resize(n, 0.0);
    }

    /// A console command of pilot `pilot`.
    pub fn station_command(&mut self, pilot: u8, cmd: StationCmd) -> Result<(), Text> {
        if self.campaign.is_none() {
            return Err(text!("st-no-campaign"));
        }
        self.defence_sizes();
        if let StationCmd::Man { unit } = cmd {
            return self.man_station(pilot, unit as usize);
        }
        let Some(at) =
            self.defence().and_then(|d| d.stations.iter().position(|s| s.pilot == pilot))
        else {
            return Err(text!("st-not-at-a-console"));
        };
        let st = self.defence().expect("campaign").stations[at];
        match cmd {
            StationCmd::Man { .. } => unreachable!(),
            StationCmd::Leave => {
                self.defence_mut().stations.remove(at);
                self.uavs_unmanned(st.unit);
            }
            StationCmd::Lock(t) => {
                if t.is_some() && st.radar_off {
                    return Err(text!("st-radar-off"));
                }
                if let Some(t) = t
                    && self.track_alive(t as usize).is_none()
                {
                    return Err(text!("st-no-such-track"));
                }
                self.defence_mut().stations[at].lock = t;
            }
            StationCmd::Radar(on) => {
                self.defence_mut().stations[at].radar_off = !on;
                if !on {
                    self.radar_silent(at);
                }
            }
            StationCmd::Mode(m) => {
                self.defence_mut().stations[at].mode = m;
                if m == Guidance::SemiActive {
                    self.release_guided(st.unit, None);
                }
            }
            StationCmd::Fire { launcher } => self.station_fire(st, launcher.map(|l| l as usize))?,
            StationCmd::Burst => self.station_burst(at)?,
            StationCmd::Slew { az, el } => {
                self.gepard_here(st)?;
                self.defence_mut().stations[at].slew = [
                    az.clamp(-GEPARD_TRAVERSE, GEPARD_TRAVERSE),
                    el.clamp(-GEPARD_ELEVATE, GEPARD_ELEVATE),
                ];
            }
            StationCmd::Refill { unit } => self.start_refill(st.unit as usize, unit as usize)?,
            StationCmd::RequestResupply => {
                return Err(text!("st-resupply-goes-through-the-campaign"));
            }
            cmd @ (StationCmd::Launch { .. }
            | StationCmd::Route { .. }
            | StationCmd::Next { .. }
            | StationCmd::Manual { .. }
            | StationCmd::Fly { .. }) => self.uav_command(&st, cmd).map_err(Text::raw)?,
        }
        Ok(())
    }

    fn man_station(&mut self, pilot: u8, unit: usize) -> Result<(), Text> {
        let Some(u) = self.units.get(unit).copied() else { return Err(text!("st-no-such-unit")) };
        if !mannable(u.kind) || !self.aaa_blue(unit) {
            return Err(text!("st-only-friendly-air-defence-can-be-manned"));
        }
        if u.destroyed {
            return Err(text!("st-unit-destroyed", unit = u.kind.name()));
        }
        if self.manned_by(unit).is_some_and(|p| p != pilot) {
            return Err(text!("st-another-player-mans-it"));
        }
        let d = self.defence_mut();
        d.stations.retain(|s| s.pilot != pilot);
        d.stations.push(Station {
            unit: unit as u16,
            pilot,
            lock: None,
            mode: Guidance::Guided,
            burst: 0,
            due: 0.0,
            slew: [0.0; 2],
            manual: false,
            radar_off: false,
        });
        Ok(())
    }

    /// Pilot `pilot` leaves his console (a sortie, a disconnect).
    pub fn leave_station(&mut self, pilot: u8) {
        let Some(unit) = self.station_of(pilot).map(|s| s.unit) else { return };
        if let Some(c) = self.campaign.as_mut() {
            c.defence.stations.retain(|s| s.pilot != pilot);
        }
        self.uavs_unmanned(unit);
    }

    /// Nobody mans drone center `unit` any more: the drones flown by hand go on by themselves.
    fn uavs_unmanned(&mut self, unit: u16) {
        for u in self.defence_mut().uavs.iter_mut().filter(|u| u.center == unit) {
            if u.mode == UavMode::Manual {
                u.mode = UavMode::Auto;
            }
        }
    }

    /// Flight unit `j` in the air: its position (m) and velocity (m/s).
    pub fn track_alive(&self, j: usize) -> Option<([f64; 3], [f64; 3])> {
        let t = self.flights.get(j)?;
        if t.flags & fflags::DESTROYED != 0 || t.flags & fflags::AIRBORNE == 0 {
            return None;
        }
        let v = mul(self.flight_velocity(j), M_PER_WU);
        Some((flight_metres(t), [v[0], -v[1], v[2]]))
    }

    /// Flight unit `j` in the air within reach of station unit `unit`'s weapon: a battery
    /// radar's missiles, a Gepard's rounds (at the lead).
    pub fn in_reach(&self, unit: usize, j: usize) -> bool {
        let hills = self.hills();
        let (Some(u), Some((pos, vel))) = (self.units.get(unit), self.track_alive(j)) else {
            return false;
        };
        if u.kind == GroundKind::Gepard {
            let (_, _, time) = lead(u.metres_on(&hills), pos, vel, GEPARD_V0);
            time * GEPARD_V0 <= GEPARD_REACH
        } else {
            let reach = u.kind.battery().map_or(0.0, |b| b.max_range());
            len(sub(pos, u.metres_on(&hills))) <= reach
        }
    }

    /// A locked track the station may shoot at: alive and not friendly.
    fn hostile_lock(&self, st: &Station) -> Result<(usize, [f64; 3], [f64; 3]), Text> {
        let Some(j) = st.lock.map(|j| j as usize) else { return Err(text!("st-no-track-locked")) };
        let Some((pos, vel)) = self.track_alive(j) else { return Err(text!("st-track-lost")) };
        if self.friendly_unit(j, self.flights[j].spec) {
            return Err(text!("st-friendly-track-weapons-hold"));
        }
        Ok((j, pos, vel))
    }

    fn station_fire(&mut self, st: Station, launcher: Option<usize>) -> Result<(), Text> {
        let hills = self.hills();
        let radar = st.unit as usize;
        let Some(b) = self.units[radar].kind.battery_radar() else {
            return Err(text!("st-no-missiles-here"));
        };
        let (j, pos, _) = self.hostile_lock(&st)?;
        if !self.in_reach(radar, j) {
            return Err(text!("st-out-of-range"));
        }
        let ready = |sim: &Sim, k: usize| {
            let u = &sim.units[k];
            !u.destroyed && u.rounds > 0 && u.cooldown == 0 && sim.refill_left(k) == 0.0
        };
        let launchers = self.battery_launchers(radar);
        let k = match launcher {
            Some(k) if launchers.contains(&k) => {
                if !ready(self, k) {
                    return Err(text!("st-launcher-not-ready"));
                }
                k
            }
            Some(_) => return Err(text!("st-not-this-battery-s-launcher")),
            None => {
                let k = self.next_launcher(radar).ok_or_else(|| text!("st-no-launcher-ready"))?;
                if !ready(self, k) {
                    return Err(text!("st-launcher-not-ready"));
                }
                k
            }
        };
        let u = self.units[k];
        let from = add(u.metres_on(&hills), [0.0, 0.0, 3.0]);
        let d = sub(pos, from);
        let az = d[0].atan2(d[1]);
        let el = b.elevation() as f64;
        let dir = [az.sin() * el.cos(), az.cos() * el.cos(), el.sin()];
        let mut m = self.ai_missile(b.missile(), from, dir, j, true);
        m.guidance = Some(st.mode);
        m.station = Some(st.unit);
        m.radar = Some(st.unit);
        let fs = self.fs.max(1);
        let a = &mut self.units[k];
        a.rounds -= 1;
        a.cooldown = LAUNCH_INTERVAL * fs;
        a.aim_az = az as f32;
        self.campaign.as_mut().expect("campaign").missiles.push(m);
        Ok(())
    }

    fn gepard_here(&self, st: Station) -> Result<(), Text> {
        if self.units[st.unit as usize].kind != GroundKind::Gepard {
            return Err(text!("st-no-gun-here"));
        }
        Ok(())
    }

    fn station_burst(&mut self, at: usize) -> Result<(), Text> {
        let st = self.defence().expect("campaign").stations[at];
        let n = self.burst_rounds(&st)?;
        let d = self.defence_mut();
        d.stations[at].burst = n;
        d.stations[at].due = 0.0;
        d.stations[at].manual = st.lock.is_none();
        Ok(())
    }

    /// The rounds of a burst station `st` can fire now (at its lock, or without one along the
    /// barrels).
    fn burst_rounds(&self, st: &Station) -> Result<i32, Text> {
        self.gepard_here(*st)?;
        let u = self.units[st.unit as usize];
        if st.burst > 0 {
            return Err(text!("st-firing"));
        }
        if u.rounds <= 0 || self.refill_left(st.unit as usize) > 0.0 {
            return Err(text!("st-out-of-ammunition"));
        }
        if st.lock.is_none() {
            return Ok(BURST.min(u.rounds));
        }
        let (j, _, _) = self.hostile_lock(st)?;
        if !self.in_reach(st.unit as usize, j) {
            return Err(text!("st-out-of-range"));
        }
        Ok(BURST.min(u.rounds))
    }

    /// Starts refilling an empty launcher (of station unit `unit`'s battery) or Gepard.
    fn start_refill(&mut self, station: usize, k: usize) -> Result<(), Text> {
        let Some(u) = self.units.get(k).copied() else { return Err(text!("st-no-such-unit")) };
        let ours = k == station || self.battery_launchers(station).contains(&k);
        if !ours || full_load(u.kind) == 0 {
            return Err(text!("st-nothing-to-refill-there"));
        }
        if u.destroyed {
            return Err(text!("st-destroyed"));
        }
        if self.refill_left(k) > 0.0 {
            return Err(text!("st-already-refilling"));
        }
        if u.rounds > 0 {
            return Err(text!("st-not-empty-yet"));
        }
        if self.stockpile(k) <= 0 {
            return Err(text!("st-stockpile-empty"));
        }
        if !self.stock_for_refill(k) {
            return Err(text!("st-stockpile-short"));
        }
        self.defence_mut().refill[k] = REFILL_SECONDS;
        Ok(())
    }

    /// Station `at`'s radar goes silent: its lock is dropped (a burst at it stops) and the
    /// missiles it guided or illuminated for lose their target.
    fn radar_silent(&mut self, at: usize) {
        let st = &mut self.defence_mut().stations[at];
        st.lock = None;
        let unit = st.unit;
        let c = self.campaign.as_mut().expect("campaign");
        for m in &mut c.missiles {
            if m.station == Some(unit) {
                m.target = NO_TARGET;
                m.guidance = None;
            }
        }
        if let Some(u) = self.units.get_mut(unit as usize) {
            u.tracking = false;
        }
    }

    /// Guided missiles of station `unit` become semi-active (all, or those not at `keep`).
    fn release_guided(&mut self, unit: u16, keep: Option<usize>) {
        let Some(c) = self.campaign.as_mut() else { return };
        for m in &mut c.missiles {
            if m.station == Some(unit)
                && m.guidance == Some(Guidance::Guided)
                && keep != Some(m.target)
            {
                m.guidance = Some(Guidance::SemiActive);
            }
        }
    }

    /// The refills run on `real` seconds of wall clock (the host's, whatever the time scale);
    /// unmanned empty launchers and Gepards start their own.
    pub fn tick_refills(&mut self, real: f32) {
        if self.campaign.is_none() {
            return;
        }
        self.defence_sizes();
        let mut done = Vec::new();
        for k in 0..self.units.len() {
            let u = self.units[k];
            let left = self.refill_left(k);
            if left > 0.0 {
                if u.destroyed {
                    self.defence_mut().refill[k] = 0.0;
                    continue;
                }
                let left = (left - real).max(0.0);
                self.defence_mut().refill[k] = left;
                if left == 0.0 {
                    done.push(k);
                }
                continue;
            }
            let full = full_load(u.kind);
            let manned = self.stock_unit(k).and_then(|s| self.manned_by(s)).is_some();
            if full > 0
                && !u.destroyed
                && u.rounds == 0
                && !manned
                && self.aaa_blue(k)
                && self.stockpile(k) > 0
                && self.stock_for_refill(k)
            {
                self.defence_mut().refill[k] = REFILL_SECONDS;
            }
        }
        for k in done {
            let Some(s) = self.stock_unit(k) else { continue };
            let full = full_load(self.units[k].kind);
            let rounds = self.units[k].rounds;
            let d = self.defence_mut();
            let n = (full - rounds).min(d.stock[s]).max(0);
            d.stock[s] -= n;
            self.units[k].rounds += n;
        }
    }

    /// One frame of the manned consoles: lost ones go, guided missiles follow their lock,
    /// the guns fire their bursts, the turrets and launchers point at the lock (a Gepard's
    /// without one where it is slewed).
    pub(crate) fn update_stations(&mut self) {
        let hills = self.hills();
        self.defence_sizes();
        let Some(c) = self.campaign.as_ref() else { return };
        if c.defence.stations.is_empty() && c.missiles.iter().all(|m| m.guidance.is_none()) {
            return;
        }
        let stations = c.defence.stations.clone();
        let mut keep = Vec::new();
        for st in stations {
            let k = st.unit as usize;
            if self.units.get(k).is_none_or(|u| u.destroyed) {
                // The missiles it guided are lost.
                let c = self.campaign.as_mut().expect("campaign");
                for m in &mut c.missiles {
                    if m.station == Some(st.unit) {
                        m.target = NO_TARGET;
                        m.guidance = None;
                    }
                }
                continue;
            }
            let lock = st.lock.map(|j| j as usize).filter(|&j| self.track_alive(j).is_some());
            self.release_guided(st.unit, lock);
            let mut st = Station { lock: lock.map(|j| j as u16), ..st };
            if let Some((pos, vel)) = lock.and_then(|j| self.track_alive(j)) {
                let u = self.units[k];
                let (az, el, _) = lead(u.metres_on(&hills), pos, vel, GEPARD_V0);
                let a = &mut self.units[k];
                a.tracking = true;
                a.aim_az = az as f32;
                if a.kind == GroundKind::Gepard {
                    a.aim_el = el as f32;
                }
                if a.kind.battery_radar().is_some() {
                    for l in self.battery_launchers(k) {
                        let d = sub(pos, self.units[l].metres_on(&hills));
                        self.units[l].aim_az = d[0].atan2(d[1]) as f32;
                    }
                }
            } else {
                self.units[k].tracking = false;
            }
            let gepard = self.units[k].kind == GroundKind::Gepard;
            if gepard && lock.is_none() {
                self.slew_frame(&st);
            }
            if st.burst > 0 {
                self.burst_frame(&mut st);
            }
            keep.push(st);
        }
        self.defence_mut().stations = keep;
    }

    /// A Gepard's turret slewed by hand for a frame.
    fn slew_frame(&mut self, st: &Station) {
        let dt = 1.0 / self.fs.max(1) as f32;
        let a = &mut self.units[st.unit as usize];
        let az = a.aim_az + st.slew[0] * dt;
        a.aim_az =
            (az + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        a.aim_el = (a.aim_el + st.slew[1] * dt).clamp(GEPARD_EL.0, GEPARD_EL.1);
    }

    /// The rounds of a Gepard burst without a lock due this frame: tracers along the barrels,
    /// at the hostile aircraft whose lead is nearest the line of fire (one far off it can't be
    /// hit).
    fn manual_frame(&mut self, st: &mut Station) {
        let hills = self.hills();
        let k = st.unit as usize;
        let u = self.units[k];
        let fs = self.fs.max(1) as f64;
        let (az, el) = (u.aim_az as f64, u.aim_el as f64);
        let dir = [az.sin() * el.cos(), az.cos() * el.cos(), el.sin()];
        let target = (0..self.flights.len())
            .filter_map(|j| {
                let (pos, vel) = self.track_alive(j)?;
                if self.friendly_unit(j, self.flights[j].spec) {
                    return None;
                }
                let r = len(sub(pos, u.metres_on(&hills)));
                let (az, el, _) = lead(u.metres_on(&hills), pos, vel, GEPARD_V0);
                let to = [az.sin() * el.cos(), az.cos() * el.cos(), el.sin()];
                let off = crate::campaign::dot(to, dir).clamp(-1.0, 1.0).acos();
                (r < MANUAL_RANGE && off < MANUAL_CONE).then_some((j, off))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(NO_TARGET, |t| t.0);
        st.due += BURST as f64 / (BURST_SECONDS * fs);
        let Some(mut c) = self.campaign.take() else { return };
        let mut rng = c.rng;
        while st.due >= 1.0 && st.burst > 0 && self.units[k].rounds > 0 {
            st.due -= 1.0;
            st.burst -= 1;
            self.units[k].rounds -= 1;
            let sigma = 0.0025;
            let (a, e) = (az + jitter(&mut rng) * sigma, el + jitter(&mut rng) * sigma);
            let d = [a.sin() * e.cos(), a.cos() * e.cos(), e.sin()];
            // Either barrel, a little to each side of the turret.
            let side = if st.burst & 1 == 0 { 1.2 } else { -1.2 };
            let from = add(u.metres_on(&hills), [az.cos() * side, -az.sin() * side, 1.5]);
            let shell = Shell {
                pos: from,
                vel: mul(d, GEPARD_V0),
                ttl: (MANUAL_RANGE / GEPARD_V0 * fs).ceil() as i32,
                flak: false,
                pilot: 0,
            };
            if c.shells.len() < MAX_SHELLS {
                if self.record_shells {
                    self.new_ai_shells.push(shell);
                }
                c.shells.push(AiShell { shell, target, ahead: false, gun: Some(k as u16) });
            }
        }
        if self.units[k].rounds == 0 {
            st.burst = 0;
        }
        self.units[k].burst = st.burst;
        c.rng = rng;
        self.campaign = Some(c);
    }

    /// The rounds of a Gepard burst due this frame, each laid on the lock anew (a burst
    /// without one: along the barrels).
    fn burst_frame(&mut self, st: &mut Station) {
        let hills = self.hills();
        if st.manual {
            return self.manual_frame(st);
        }
        let k = st.unit as usize;
        let fs = self.fs.max(1) as f64;
        let target = st.lock.map(|j| j as usize).and_then(|j| self.track_alive(j).map(|t| (j, t)));
        let Some((j, (pos, vel))) = target else {
            st.burst = 0;
            self.units[k].burst = 0;
            return;
        };
        st.due += BURST as f64 / (BURST_SECONDS * fs);
        let Some(mut c) = self.campaign.take() else { return };
        let mut rng = c.rng;
        while st.due >= 1.0 && st.burst > 0 && self.units[k].rounds > 0 {
            st.due -= 1.0;
            st.burst -= 1;
            self.units[k].rounds -= 1;
            let u = self.units[k];
            let (az, el, time) = lead(u.metres_on(&hills), pos, vel, GEPARD_V0);
            let sigma = 0.0025;
            let (a, e) = (az + jitter(&mut rng) * sigma, el + jitter(&mut rng) * sigma);
            let dir = [a.sin() * e.cos(), a.cos() * e.cos(), e.sin()];
            // Either barrel, a little to each side of the turret.
            let side = if st.burst & 1 == 0 { 1.2 } else { -1.2 };
            let from = add(u.metres_on(&hills), [az.cos() * side, -az.sin() * side, 1.5]);
            let shell = Shell {
                pos: from,
                vel: mul(dir, GEPARD_V0),
                ttl: ((time * (1.0 + jitter(&mut rng) * 0.01)) * fs).ceil().max(1.0) as i32,
                flak: true,
                pilot: 0,
            };
            if c.shells.len() < MAX_SHELLS {
                if self.record_shells {
                    self.new_ai_shells.push(shell);
                }
                c.shells.push(AiShell { shell, target: j, ahead: true, gun: Some(k as u16) });
            }
        }
        if self.units[k].rounds == 0 {
            st.burst = 0;
        }
        self.units[k].burst = st.burst;
        c.rng = rng;
        self.campaign = Some(c);
    }

    /// An airburst round going off `miss` metres from its target: a kill?
    pub(crate) fn ahead_kills(rng: &mut crate::rng::Rng, miss: f64) -> bool {
        miss < AHEAD_RADIUS && rng.random(1000) < AHEAD_PK
    }

    /// A console's missile close to its target meets the target's countermeasures (once): a
    /// spoofed one loses it.
    pub(crate) fn spoofed(
        rng: &mut crate::rng::Rng,
        m: &mut crate::campaign::AiMissile,
        tp: [f64; 3],
    ) -> bool {
        let Some(g) = m.guidance else { return false };
        if m.spoof_checked || len(sub(m.pos, tp)) > SPOOF_RANGE {
            return false;
        }
        m.spoof_checked = true;
        let chance = if g == Guidance::Guided { SPOOF_GUIDED } else { SPOOF_SEMI };
        if rng.random(1000) < chance {
            m.target = NO_TARGET;
            m.guidance = None;
            return true;
        }
        false
    }

    /// The stockpiles to keep over a save.
    pub fn saved_stocks(&self) -> Vec<SavedStock> {
        let Some(d) = self.defence() else { return Vec::new() };
        (0..self.units.len())
            .filter(|&k| {
                self.aaa_blue(k) && !self.units[k].destroyed && mannable(self.units[k].kind)
            })
            .map(|k| {
                let u = &self.units[k];
                SavedStock {
                    x: u.x,
                    y: u.y,
                    kind: u.kind,
                    stock: d.stock.get(k).copied().unwrap_or(0),
                    stock2: d.stock2.get(k).copied().unwrap_or(0),
                    stock3: d.stock3.get(k).copied().unwrap_or(0),
                }
            })
            .collect()
    }

    /// Saved stockpiles back on the units (each on the nearest of its kind, within 6 km).
    pub fn restore_stocks(&mut self, saved: &[SavedStock]) {
        self.defence_sizes();
        let mut taken = vec![false; self.units.len()];
        for s in saved {
            let best = (0..self.units.len())
                .filter(|&k| !taken[k] && self.units[k].kind == s.kind && self.aaa_blue(k))
                .map(|k| (k, approx(self.units[k].x - s.x, self.units[k].y - s.y)))
                .filter(|&(_, d)| d < 0x180)
                .min_by_key(|&(_, d)| d);
            if let Some((k, _)) = best {
                taken[k] = true;
                let d = self.defence_mut();
                d.stock[k] = s.stock;
                d.stock2[k] = s.stock2;
                if s.stock3 >= 0 {
                    d.stock3[k] = s.stock3;
                }
            }
        }
    }
}

/// An anti-radiation missile `range` metres from the radar it flies at, which has just gone
/// silent (checked once): has it lost it? If so it flies on a little off its line.
pub(crate) fn arm_lost(rng: &mut crate::rng::Rng, vel: &mut [f64; 3], range: f64) -> bool {
    let far = (range / ARM_FAR).clamp(0.0, 1.0);
    let chance = ARM_LOST_NEAR + ((ARM_LOST_FAR - ARM_LOST_NEAR) as f64 * far) as i32;
    if rng.random(1000) >= chance {
        return false;
    }
    let side = if rng.random(2) == 0 { -1.0 } else { 1.0 };
    let a = side * ARM_ERROR * (0.4 + 0.6 * rng.random(1000) as f64 / 1000.0);
    let (s, c) = a.sin_cos();
    *vel = [vel[0] * c - vel[1] * s, vel[0] * s + vel[1] * c, vel[2]];
    true
}
