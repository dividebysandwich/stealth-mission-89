//! Campaign: friendly air defence manned by a player. A Patriot battery is fought from its
//! radar's console (the radar's track, the launchers around it), a Gepard from its turret.
//!
//! * The player locks a track; missiles fly at it **guided** (the radar steers them, hard to
//!   decoy, one target at a time: a guided missile whose station locks something else or
//!   switches to semi-active becomes semi-active) or **semi-active** (they home on the radar's
//!   illumination of the target they were fired at, several targets at once, easier to decoy).
//! * Launchers hold four missiles; an empty one is refilled from the battery's stockpile, which
//!   takes [`REFILL_SECONDS`] of *real* time ([`Sim::tick_refills`], called by the host with
//!   wall clock seconds). Unmanned, the AI starts the refills itself.
//! * A Gepard fires bursts of [`BURST`] programmed airburst rounds, out to [`GEPARD_REACH`].
//! * A station whose radar (or Gepard) is destroyed is gone: [`Sim::station_of`] is None.

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{AaaKind, PATRIOT, Shell, lead};
use crate::campaign::{
    AiShell, MAX_SHELLS, PATRIOT_ELEVATION, PATRIOT_LINK, PATRIOT_ROUNDS, add, flight_metres,
    jitter, len, mul, sub,
};
use crate::math::approx;
use crate::types::*;

/// Missiles in a Patriot battery's stockpile, beyond those on its launchers.
pub const PATRIOT_STOCK: i32 = 8;
/// A Gepard's rounds ready, and in its stockpile.
pub const GEPARD_ROUNDS: i32 = 640;
pub const GEPARD_STOCK: i32 = 640;
/// Real seconds to refill a launcher (or a Gepard).
pub const REFILL_SECONDS: f32 = 240.0;
/// The PAC-3's reach (m): no shots at a track further out (it gets less against a target
/// that makes it turn, `guidance`).
pub const PATRIOT_MAX_RANGE: f64 = 80_000.0;
/// Seconds between a manned launcher's missiles (the AI waits longer, `PATRIOT_INTERVAL`).
pub const LAUNCH_INTERVAL: i32 = 2;
/// A Gepard burst: rounds, and seconds it takes (two guns, 550 rounds a minute each).
pub const BURST: i32 = 25;
const BURST_SECONDS: f64 = 1.4;
/// A Gepard's programmed rounds: muzzle velocity (m/s), farthest burst point (m).
const GEPARD_V0: f64 = 1175.0;
pub const GEPARD_REACH: f64 = 2000.0;
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

/// How a missile from a manned console is guided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Guidance {
    #[default]
    Guided,
    SemiActive,
}

/// What a player at a console does.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StationCmd {
    /// Man a friendly Patriot radar or Gepard (AAA unit).
    Man {
        unit: u16,
    },
    Leave,
    /// Lock a track (flight unit), or drop the lock.
    Lock(Option<u16>),
    Mode(Guidance),
    /// A missile at the locked track from a launcher (None: the readiest).
    Fire {
        launcher: Option<u16>,
    },
    /// Gepard: a burst at the locked track.
    Burst,
    /// Refill an empty launcher (or the Gepard) from the stockpile.
    Refill {
        unit: u16,
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
}

/// The friendly air defence's consoles and stocks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DefenceState {
    pub stations: Vec<Station>,
    /// Per AAA unit: the stockpile at a Patriot radar (its battery's) or a Gepard.
    pub stock: Vec<i32>,
    /// Per AAA unit: real seconds left of a launcher's or Gepard's refill (0: none).
    pub refill: Vec<f32>,
}

/// A stockpile to keep over a save: where, a Patriot battery (else a Gepard), how much.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedStock {
    pub x: i32,
    pub y: i32,
    pub patriot: bool,
    pub stock: i32,
}

/// The stockpile a unit starts with.
fn initial_stock(kind: AaaKind) -> i32 {
    match kind {
        AaaKind::PatriotRadar => PATRIOT_STOCK,
        AaaKind::Gepard => GEPARD_STOCK,
        _ => 0,
    }
}

/// A unit's load when full.
fn full_load(kind: AaaKind) -> i32 {
    match kind {
        AaaKind::PatriotLauncher => PATRIOT_ROUNDS,
        AaaKind::Gepard => GEPARD_ROUNDS,
        _ => 0,
    }
}

/// A unit a player can man.
pub fn mannable(kind: AaaKind) -> bool {
    matches!(kind, AaaKind::PatriotRadar | AaaKind::Gepard)
}

impl Sim {
    /// The consoles and stocks (a campaign's).
    pub fn defence(&self) -> Option<&crate::station::DefenceState> {
        self.campaign.as_ref().map(|c| &c.defence)
    }

    fn defence_mut(&mut self) -> &mut DefenceState {
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

    /// The live launchers a Patriot radar commands.
    pub fn battery_launchers(&self, radar: usize) -> Vec<usize> {
        let Some(r) = self.aaa.get(radar) else { return Vec::new() };
        let blue = self.aaa_blue(radar);
        (0..self.aaa.len())
            .filter(|&k| {
                let u = &self.aaa[k];
                u.kind == AaaKind::PatriotLauncher
                    && self.aaa_blue(k) == blue
                    && approx(u.x - r.x, u.y - r.y) < PATRIOT_LINK
            })
            .collect()
    }

    /// The Patriot radar that commands a launcher.
    pub fn launcher_radar(&self, launcher: usize) -> Option<usize> {
        let l = self.aaa.get(launcher)?;
        let blue = self.aaa_blue(launcher);
        (0..self.aaa.len()).find(|&r| {
            let u = &self.aaa[r];
            u.kind == AaaKind::PatriotRadar
                && !u.destroyed
                && self.aaa_blue(r) == blue
                && approx(u.x - l.x, u.y - l.y) < PATRIOT_LINK
        })
    }

    /// Where a unit's stock is kept: a launcher's at its radar, anything else its own.
    fn stock_unit(&self, k: usize) -> Option<usize> {
        match self.aaa.get(k)?.kind {
            AaaKind::PatriotLauncher => self.launcher_radar(k),
            _ => Some(k),
        }
    }

    /// Unit `k`'s stockpile (a launcher's: its battery's).
    pub fn stockpile(&self, k: usize) -> i32 {
        let s = self.stock_unit(k);
        s.and_then(|s| self.defence()?.stock.get(s).copied()).unwrap_or(0)
    }

    /// Real seconds left of unit `k`'s refill (0: none).
    pub fn refill_left(&self, k: usize) -> f32 {
        self.defence().and_then(|d| d.refill.get(k).copied()).unwrap_or(0.0)
    }

    /// The defence's lists follow the units (new ones get their stockpiles).
    pub(crate) fn defence_sizes(&mut self) {
        let Some(c) = self.campaign.as_mut() else { return };
        let d = &mut c.defence;
        let n = self.aaa.len();
        d.stock.truncate(n);
        for k in d.stock.len()..n {
            d.stock.push(initial_stock(self.aaa[k].kind));
        }
        d.refill.resize(n, 0.0);
    }

    /// A console command of pilot `pilot`.
    pub fn station_command(&mut self, pilot: u8, cmd: StationCmd) -> Result<(), String> {
        if self.campaign.is_none() {
            return Err("No campaign".into());
        }
        self.defence_sizes();
        if let StationCmd::Man { unit } = cmd {
            return self.man_station(pilot, unit as usize);
        }
        let Some(at) =
            self.defence().and_then(|d| d.stations.iter().position(|s| s.pilot == pilot))
        else {
            return Err("Not at a console".into());
        };
        let st = self.defence().expect("campaign").stations[at];
        match cmd {
            StationCmd::Man { .. } => unreachable!(),
            StationCmd::Leave => {
                self.defence_mut().stations.remove(at);
            }
            StationCmd::Lock(t) => {
                if let Some(t) = t
                    && self.track_alive(t as usize).is_none()
                {
                    return Err("No such track".into());
                }
                self.defence_mut().stations[at].lock = t;
            }
            StationCmd::Mode(m) => {
                self.defence_mut().stations[at].mode = m;
                if m == Guidance::SemiActive {
                    self.release_guided(st.unit, None);
                }
            }
            StationCmd::Fire { launcher } => self.station_fire(st, launcher.map(|l| l as usize))?,
            StationCmd::Burst => self.station_burst(at)?,
            StationCmd::Refill { unit } => self.start_refill(st.unit as usize, unit as usize)?,
        }
        Ok(())
    }

    fn man_station(&mut self, pilot: u8, unit: usize) -> Result<(), String> {
        let Some(u) = self.aaa.get(unit).copied() else { return Err("No such unit".into()) };
        if !mannable(u.kind) || !self.aaa_blue(unit) {
            return Err("Only friendly Patriot batteries and Gepards can be manned".into());
        }
        if u.destroyed {
            return Err(format!("The {} is destroyed", u.kind.name()));
        }
        if self.manned_by(unit).is_some_and(|p| p != pilot) {
            return Err("Another player mans it".into());
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
        });
        Ok(())
    }

    /// Pilot `pilot` leaves his console (a sortie, a disconnect).
    pub fn leave_station(&mut self, pilot: u8) {
        if let Some(c) = self.campaign.as_mut() {
            c.defence.stations.retain(|s| s.pilot != pilot);
        }
    }

    /// Flight unit `j` in the air: its position (m) and velocity (m/s).
    fn track_alive(&self, j: usize) -> Option<([f64; 3], [f64; 3])> {
        let t = self.flights.get(j)?;
        if t.flags & fflags::DESTROYED != 0 || t.flags & fflags::AIRBORNE == 0 {
            return None;
        }
        let vel = mul(crate::campaign::direction(t.heading, t.pitch), t.speed as f64 * 0.514);
        Some((flight_metres(t), vel))
    }

    /// A locked track the station may shoot at: alive and not friendly.
    fn hostile_lock(&self, st: &Station) -> Result<(usize, [f64; 3], [f64; 3]), String> {
        let Some(j) = st.lock.map(|j| j as usize) else { return Err("No track locked".into()) };
        let Some((pos, vel)) = self.track_alive(j) else { return Err("Track lost".into()) };
        if self.friendly_unit(j, self.flights[j].spec) {
            return Err("Friendly track: weapons hold".into());
        }
        Ok((j, pos, vel))
    }

    fn station_fire(&mut self, st: Station, launcher: Option<usize>) -> Result<(), String> {
        let radar = st.unit as usize;
        if self.aaa[radar].kind != AaaKind::PatriotRadar {
            return Err("No missiles here".into());
        }
        let (j, pos, _) = self.hostile_lock(&st)?;
        if len(sub(pos, self.aaa[radar].metres())) > PATRIOT_MAX_RANGE {
            return Err("Out of range".into());
        }
        let ready = |sim: &Sim, k: usize| {
            let u = &sim.aaa[k];
            !u.destroyed && u.rounds > 0 && u.cooldown == 0 && sim.refill_left(k) == 0.0
        };
        let launchers = self.battery_launchers(radar);
        let k = match launcher {
            Some(k) if launchers.contains(&k) => {
                if !ready(self, k) {
                    return Err("Launcher not ready".into());
                }
                k
            }
            Some(_) => return Err("Not this battery's launcher".into()),
            None => launchers
                .iter()
                .copied()
                .filter(|&k| ready(self, k))
                .max_by_key(|&k| self.aaa[k].rounds)
                .ok_or("No launcher ready")?,
        };
        let u = self.aaa[k];
        let from = add(u.metres(), [0.0, 0.0, 3.0]);
        let d = sub(pos, from);
        let az = d[0].atan2(d[1]);
        let el = PATRIOT_ELEVATION as f64;
        let dir = [az.sin() * el.cos(), az.cos() * el.cos(), el.sin()];
        let mut m = self.ai_missile(PATRIOT, from, dir, j, true);
        m.guidance = Some(st.mode);
        m.station = Some(st.unit);
        let fs = self.fs.max(1);
        let a = &mut self.aaa[k];
        a.rounds -= 1;
        a.cooldown = LAUNCH_INTERVAL * fs;
        a.aim_az = az as f32;
        self.campaign.as_mut().expect("campaign").missiles.push(m);
        Ok(())
    }

    fn station_burst(&mut self, at: usize) -> Result<(), String> {
        let st = self.defence().expect("campaign").stations[at];
        let u = self.aaa[st.unit as usize];
        if u.kind != AaaKind::Gepard {
            return Err("No gun here".into());
        }
        if st.burst > 0 {
            return Err("Firing".into());
        }
        if u.rounds <= 0 || self.refill_left(st.unit as usize) > 0.0 {
            return Err("Out of ammunition".into());
        }
        let (_, pos, vel) = self.hostile_lock(&st)?;
        let (_, _, time) = lead(u.metres(), pos, vel, GEPARD_V0);
        if time * GEPARD_V0 > GEPARD_REACH {
            return Err("Out of range".into());
        }
        let d = self.defence_mut();
        d.stations[at].burst = BURST.min(u.rounds);
        d.stations[at].due = 0.0;
        Ok(())
    }

    /// Starts refilling an empty launcher (of station unit `unit`'s battery) or Gepard.
    fn start_refill(&mut self, station: usize, k: usize) -> Result<(), String> {
        let Some(u) = self.aaa.get(k).copied() else { return Err("No such unit".into()) };
        let ours = k == station || self.battery_launchers(station).contains(&k);
        if !ours || full_load(u.kind) == 0 {
            return Err("Nothing to refill there".into());
        }
        if u.destroyed {
            return Err("Destroyed".into());
        }
        if self.refill_left(k) > 0.0 {
            return Err("Already refilling".into());
        }
        if u.rounds > 0 {
            return Err("Not empty yet".into());
        }
        if self.stockpile(k) <= 0 {
            return Err("Stockpile empty".into());
        }
        self.defence_mut().refill[k] = REFILL_SECONDS;
        Ok(())
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
        for k in 0..self.aaa.len() {
            let u = self.aaa[k];
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
            {
                self.defence_mut().refill[k] = REFILL_SECONDS;
            }
        }
        for k in done {
            let Some(s) = self.stock_unit(k) else { continue };
            let full = full_load(self.aaa[k].kind);
            let rounds = self.aaa[k].rounds;
            let d = self.defence_mut();
            let n = (full - rounds).min(d.stock[s]).max(0);
            d.stock[s] -= n;
            self.aaa[k].rounds += n;
        }
    }

    /// One frame of the manned consoles: lost ones go, guided missiles follow their lock,
    /// the guns fire their bursts, the turrets and launchers point at the lock.
    pub(crate) fn update_stations(&mut self) {
        self.defence_sizes();
        let Some(c) = self.campaign.as_ref() else { return };
        if c.defence.stations.is_empty() && c.missiles.iter().all(|m| m.guidance.is_none()) {
            return;
        }
        let stations = c.defence.stations.clone();
        let mut keep = Vec::new();
        for st in stations {
            let k = st.unit as usize;
            if self.aaa.get(k).is_none_or(|u| u.destroyed) {
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
                let u = self.aaa[k];
                let (az, el, _) = lead(u.metres(), pos, vel, GEPARD_V0);
                let a = &mut self.aaa[k];
                a.tracking = true;
                a.aim_az = az as f32;
                if a.kind == AaaKind::Gepard {
                    a.aim_el = el as f32;
                }
                if a.kind == AaaKind::PatriotRadar {
                    for l in self.battery_launchers(k) {
                        let d = sub(pos, self.aaa[l].metres());
                        self.aaa[l].aim_az = d[0].atan2(d[1]) as f32;
                    }
                }
            } else {
                self.aaa[k].tracking = false;
            }
            if st.burst > 0 {
                self.burst_frame(&mut st);
            }
            keep.push(st);
        }
        self.defence_mut().stations = keep;
    }

    /// The rounds of a Gepard burst due this frame, each laid on the lock anew.
    fn burst_frame(&mut self, st: &mut Station) {
        let k = st.unit as usize;
        let fs = self.fs.max(1) as f64;
        let target = st.lock.map(|j| j as usize).and_then(|j| self.track_alive(j).map(|t| (j, t)));
        let Some((j, (pos, vel))) = target else {
            st.burst = 0;
            return;
        };
        st.due += BURST as f64 / (BURST_SECONDS * fs);
        let Some(mut c) = self.campaign.take() else { return };
        let mut rng = c.rng;
        while st.due >= 1.0 && st.burst > 0 && self.aaa[k].rounds > 0 {
            st.due -= 1.0;
            st.burst -= 1;
            self.aaa[k].rounds -= 1;
            let u = self.aaa[k];
            let (az, el, time) = lead(u.metres(), pos, vel, GEPARD_V0);
            let sigma = 0.0025;
            let (a, e) = (az + jitter(&mut rng) * sigma, el + jitter(&mut rng) * sigma);
            let dir = [a.sin() * e.cos(), a.cos() * e.cos(), e.sin()];
            // Either barrel, a little to each side of the turret.
            let side = if st.burst & 1 == 0 { 1.2 } else { -1.2 };
            let from = add(u.metres(), [az.cos() * side, -az.sin() * side, 1.5]);
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
                c.shells.push(AiShell { shell, target: j, ahead: true });
            }
        }
        if self.aaa[k].rounds == 0 {
            st.burst = 0;
        }
        self.aaa[k].burst = st.burst;
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
        (0..self.aaa.len())
            .filter(|&k| self.aaa_blue(k) && !self.aaa[k].destroyed && mannable(self.aaa[k].kind))
            .map(|k| {
                let u = &self.aaa[k];
                SavedStock {
                    x: u.x,
                    y: u.y,
                    patriot: u.kind == AaaKind::PatriotRadar,
                    stock: d.stock.get(k).copied().unwrap_or(0),
                }
            })
            .collect()
    }

    /// Saved stockpiles back on the units (each on the nearest of its kind, within 6 km).
    pub fn restore_stocks(&mut self, saved: &[SavedStock]) {
        self.defence_sizes();
        let mut taken = vec![false; self.aaa.len()];
        for s in saved {
            let kind = if s.patriot { AaaKind::PatriotRadar } else { AaaKind::Gepard };
            let best = (0..self.aaa.len())
                .filter(|&k| !taken[k] && self.aaa[k].kind == kind && self.aaa_blue(k))
                .map(|k| (k, approx(self.aaa[k].x - s.x, self.aaa[k].y - s.y)))
                .filter(|&(_, d)| d < 0x180)
                .min_by_key(|&(_, d)| d);
            if let Some((k, _)) = best {
                taken[k] = true;
                self.defence_mut().stock[k] = s.stock;
            }
        }
    }
}
