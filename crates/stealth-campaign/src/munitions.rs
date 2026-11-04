//! The munitions of each side's airbases. The Sidewinders (the enemy's AA-8s), unguided bombs,
//! guns, flares and fuel never run out; every other weapon, the SAM missiles and the gun rounds
//! of the air defence come out of the stock of a base. Each side's rearmost airfield is its
//! production base: it makes every munition, the more complex the slower. A base that runs out
//! of one gets a cargo flight from the production base `CARGO_WAIT` later, with up to half of
//! its full stock of each (what the production base has); the other side's interceptors go up
//! `CARGO_INTERCEPT_WAIT` after it is scheduled. A cargo aircraft shot down loses its load.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use stealth_i18n::{Text, text};

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, Report};

/// A munition a base keeps a stock of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Munition {
    Aim120,
    Aim7,
    Harm,
    Penguin,
    Harpoon,
    Agm65,
    Gbu12,
    RockeyeII,
    Cbu72,
    Hellfire,
    Stinger,
    Cbu105,
    Pac3,
    GepardRounds,
    /// The enemy's radar guided air-to-air missiles (AA-6 to AA-10, and their IR versions).
    Aa10,
    Aa11,
    /// The enemy's anti-radiation missile, and its air-to-ground missile.
    As11,
    As10,
    /// The enemy's SAM missiles by their sites' reach.
    SamLong,
    SamMedium,
    SamShort,
    /// The enemy's gun rounds (ZSU-23-4, S-60).
    AaaRounds,
}

const BLUE: [Munition; 14] = [
    Munition::Aim120,
    Munition::Aim7,
    Munition::Harm,
    Munition::Agm65,
    Munition::Gbu12,
    Munition::Cbu105,
    Munition::RockeyeII,
    Munition::Cbu72,
    Munition::Hellfire,
    Munition::Stinger,
    Munition::Penguin,
    Munition::Harpoon,
    Munition::Pac3,
    Munition::GepardRounds,
];
const RED: [Munition; 8] = [
    Munition::Aa10,
    Munition::Aa11,
    Munition::As11,
    Munition::As10,
    Munition::SamLong,
    Munition::SamMedium,
    Munition::SamShort,
    Munition::AaaRounds,
];

/// The simulation's missile specs (`assets/sim/units.ron`).
pub mod spec {
    pub const AIM120: i32 = 22;
    pub const AIM9: i32 = 23;
    pub const HARM: i32 = 24;
    pub const AGM65: i32 = 27;
    pub const AIM7: i32 = 31;
    pub const AA8: i32 = 17;
    pub const AA11: i32 = 50;
}

impl Munition {
    /// The munitions a side's bases keep.
    pub fn of_side(side: Side) -> &'static [Munition] {
        match side {
            Side::Blue => &BLUE,
            Side::Red => &RED,
            Side::Neutral => &[],
        }
    }

    /// Its name (the designation; the enemy's SAM groups translated).
    pub fn name(self) -> Text {
        use Munition::*;
        let raw = match self {
            Aim120 => "AIM-120",
            Aim7 => "AIM-7",
            Harm => "AGM-88 HARM",
            Penguin => "Penguin",
            Harpoon => "AGM-84 Harpoon",
            Agm65 => "AGM-65",
            Gbu12 => "GBU-12",
            RockeyeII => "Rockeye II",
            Cbu72 => "CBU-72",
            Hellfire => "AGM-114 Hellfire",
            Stinger => "FIM-92 Stinger",
            Cbu105 => "CBU-105",
            Pac3 => "PAC-3",
            GepardRounds => "35 mm",
            Aa10 => "AA-10",
            Aa11 => "AA-11",
            As11 => "AS-11",
            As10 => "AS-10",
            AaaRounds => "23/57 mm",
            SamLong => return text!("mu-sam-long"),
            SamMedium => return text!("mu-sam-medium"),
            SamShort => return text!("mu-sam-short"),
        };
        Text::raw(raw)
    }

    /// A base's full stock, and how many the production base makes an hour (the more complex
    /// the fewer).
    pub fn full(self) -> f64 {
        use Munition::*;
        match self {
            Aim120 => 32.0,
            Aim7 => 16.0,
            Harm => 12.0,
            Penguin => 8.0,
            Harpoon => 6.0,
            Agm65 => 24.0,
            Gbu12 => 24.0,
            RockeyeII => 16.0,
            Cbu72 => 12.0,
            Hellfire => 32.0,
            Stinger => 24.0,
            Cbu105 => 8.0,
            Pac3 => 24.0,
            GepardRounds => 4000.0,
            Aa10 => 32.0,
            Aa11 => 16.0,
            As11 => 12.0,
            As10 => 24.0,
            SamLong => 16.0,
            SamMedium => 24.0,
            SamShort => 32.0,
            AaaRounds => 20000.0,
        }
    }

    pub fn per_hour(self) -> f64 {
        use Munition::*;
        match self {
            Aim120 | Aa10 => 8.0,
            Aim7 => 10.0,
            Harm | As11 => 5.0,
            Penguin => 4.0,
            Harpoon => 3.0,
            Agm65 | As10 => 10.0,
            Gbu12 => 12.0,
            RockeyeII | Cbu72 => 10.0,
            Hellfire => 20.0,
            Stinger => 15.0,
            Cbu105 => 3.0,
            Pac3 => 4.0,
            Aa11 => 6.0,
            SamLong => 4.0,
            SamMedium => 6.0,
            SamShort => 12.0,
            GepardRounds => 3000.0,
            AaaRounds => 12000.0,
        }
    }

    /// What is taken instead when it is out (None: the side's unlimited weapon, an AIM-9 or
    /// AA-8, iron bombs).
    pub fn fallback(self) -> Option<Munition> {
        match self {
            Munition::Aim120 => Some(Munition::Aim7),
            Munition::Harm => Some(Munition::Agm65),
            Munition::As11 => Some(Munition::As10),
            _ => None,
        }
    }

    /// The munition of a player's weapon type (None: one that never runs out).
    pub fn of_weapon(weapon: i32) -> Option<Munition> {
        use Munition::*;
        Some(match weapon {
            1 => Aim120,
            2 => Harm,
            3 => Penguin,
            4 => Harpoon,
            5 => Agm65,
            6 => Gbu12,
            11 => RockeyeII,
            13 => Cbu72,
            20 => Hellfire,
            21 => Stinger,
            23 => Cbu105,
            _ => return None,
        })
    }

    /// The munition of an AI aircraft's missile (simulation spec) of a side (None: one that
    /// never runs out).
    pub fn of_spec(spec: i32, side: Side) -> Option<Munition> {
        use Munition::*;
        let blue = side == Side::Blue;
        Some(match spec {
            spec::AIM120 => Aim120,
            spec::AIM7 => Aim7,
            spec::HARM if blue => Harm,
            spec::HARM => As11,
            spec::AGM65 if blue => Agm65,
            spec::AGM65 => As10,
            18..=21 | 34..=37 => Aa10,
            spec::AA11 => Aa11,
            47 => Hellfire,
            48 => Stinger,
            _ => return None,
        })
    }

    /// The AI's missile spec of an air-to-air munition.
    pub fn spec(self) -> Option<i32> {
        match self {
            Munition::Aim120 => Some(spec::AIM120),
            Munition::Aim7 => Some(spec::AIM7),
            Munition::Aa11 => Some(spec::AA11),
            _ => None,
        }
    }

    /// The SAM missiles a site of missile spec `spec` fires (by its reach).
    pub fn sam(spec: i32) -> Munition {
        match spec {
            // SA-2, SA-5, SA-10, SA-12, SA-N-6, SA-21.
            1 | 2 | 4 | 6 | 10 | 41 => Munition::SamLong,
            // SA-8, SA-13, SA-N-4, SA-N-5, Rapier, Tiger, Seacat, SA-9, SA-22, SA-15.
            3 | 7..=9 | 13..=15 | 39 | 43 | 44 => Munition::SamShort,
            _ => Munition::SamMedium,
        }
    }

    /// Counted in rounds (shown without the "missiles" of the others).
    pub fn rounds(self) -> bool {
        matches!(self, Munition::GepardRounds | Munition::AaaRounds)
    }
}

/// What a base has of each munition (whole ones count; production adds fractions).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    /// The side whose munitions these are (a captured base starts empty for its new owner).
    pub side: Option<Side>,
    pub counts: BTreeMap<Munition, f64>,
}

impl Stock {
    /// A side's full stock.
    pub fn full(side: Side) -> Self {
        Self {
            side: Some(side),
            counts: Munition::of_side(side).iter().map(|&m| (m, m.full())).collect(),
        }
    }

    /// Whole ones of `m`.
    pub fn count(&self, m: Munition) -> u32 {
        self.counts.get(&m).copied().unwrap_or(0.0).max(0.0).floor() as u32
    }

    /// The fraction of `m` the production base has made towards the next one.
    pub fn progress(&self, m: Munition) -> f64 {
        self.counts.get(&m).copied().unwrap_or(0.0).fract()
    }

    /// Takes up to `n` of `m`: what was taken.
    pub fn take(&mut self, m: Munition, n: u32) -> u32 {
        let have = self.count(m);
        let n = n.min(have);
        if n > 0 {
            *self.counts.entry(m).or_insert(0.0) -= n as f64;
        }
        n
    }

    pub fn put(&mut self, m: Munition, n: u32) {
        *self.counts.entry(m).or_insert(0.0) += n as f64;
    }

    /// The munitions it is out of.
    pub fn empty(&self, side: Side) -> Vec<Munition> {
        Munition::of_side(side).iter().copied().filter(|&m| self.count(m) == 0).collect()
    }
}

/// Where a cargo flight is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CargoState {
    /// The load is set aside at the production base; the aircraft takes off then.
    Scheduled { launch_at: f64 },
    /// In the air.
    Airborne,
}

/// A cargo flight of a side from its production base to a base that ran out of a munition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoRun {
    pub id: u32,
    pub side: Side,
    pub from: ObjectId,
    pub to: ObjectId,
    /// The aircraft's campaign tag (the other side's interceptors hunt it).
    pub tag: u32,
    pub load: Vec<(Munition, u32)>,
    pub state: CargoState,
    /// When it was scheduled.
    pub posted: f64,
}

impl CargoRun {
    /// When it takes off (None: in the air).
    pub fn launch_at(&self) -> Option<f64> {
        match self.state {
            CargoState::Scheduled { launch_at } => Some(launch_at),
            CargoState::Airborne => None,
        }
    }
}

/// Index of a side in the per-side arrays (blue, red).
fn ix(side: Side) -> usize {
    (side == Side::Red) as usize
}

impl CampaignState {
    /// The bases keep stocks of munitions (a host's campaign; not an older test state).
    pub fn munitions_on(&self) -> bool {
        self.rules.is_some()
    }

    /// A side's production base.
    pub fn production_base(&self, side: Side) -> Option<ObjectId> {
        self.production.get(ix(side)).copied().flatten()
    }

    /// The stock of a base (None: it never runs out of anything, a carrier's).
    pub fn stock(&self, base: ObjectId) -> Option<&Stock> {
        if !self.munitions_on() {
            return None;
        }
        self.stocks.get(&base)
    }

    /// How many of `m` base `base` has (u32::MAX: unlimited).
    pub fn available(&self, base: ObjectId, m: Munition) -> u32 {
        self.stock(base).map_or(u32::MAX, |s| s.count(m))
    }

    /// Takes up to `n` of `m` from a base's stock: what it got.
    pub fn draw(&mut self, base: ObjectId, m: Munition, n: u32) -> u32 {
        if self.stock(base).is_none() {
            return n;
        }
        self.stocks.get_mut(&base).map_or(n, |s| s.take(m, n))
    }

    /// Takes `n` of `m`, the rest from its fallbacks in turn: what was taken of each (what is
    /// still missing comes from the side's unlimited weapons).
    pub fn draw_with_fallback(
        &mut self,
        base: ObjectId,
        m: Munition,
        n: u32,
    ) -> Vec<(Munition, u32)> {
        let mut out = Vec::new();
        let mut left = n;
        let mut next = Some(m);
        while let Some(m) = next.filter(|_| left > 0) {
            let got = self.draw(base, m, left);
            if got > 0 {
                out.push((m, got));
            }
            left -= got;
            next = m.fallback();
        }
        out
    }

    /// Munitions back into a base's stock (unused ones of a landed aircraft).
    pub fn put_back(&mut self, base: ObjectId, m: Munition, n: u32) {
        if self.stock(base).is_none() || n == 0 {
            return;
        }
        let side = self.owner(base);
        if let Some(s) = self.stocks.get_mut(&base)
            && s.side == side
            && Munition::of_side(side.unwrap_or(Side::Neutral)).contains(&m)
        {
            s.put(m, n);
        }
    }

    /// The airfield of a side nearest a point within `radius` (map units).
    pub fn airfield_near(&self, side: Side, x: i32, y: i32, radius: i32) -> Option<ObjectId> {
        self.sites
            .iter()
            .filter(|s| !s.removed && s.kind == SiteKind::Airfield)
            .filter(|s| self.owner(s.object) == Some(side))
            .map(|s| (s.object, dist(s.x - x, s.y - y)))
            .filter(|&(_, d)| d <= radius)
            .min_by_key(|&(_, d)| d)
            .map(|(b, _)| b)
    }

    /// The airfields that keep stocks (every one held by a side, bombed or not).
    fn stocked_airfields(&self) -> Vec<(ObjectId, Side)> {
        self.sites
            .iter()
            .filter(|s| !s.removed && s.kind == SiteKind::Airfield)
            .filter_map(|s| {
                let side = self.owner(s.object)?;
                (side != Side::Neutral).then_some((s.object, side))
            })
            .collect()
    }

    /// A side's rearmost intact airfield: the furthest from the other side's places.
    fn rearmost(&self, side: Side) -> Option<ObjectId> {
        let enemy: Vec<(i32, i32)> =
            self.places.iter().filter(|p| p.owner == side.enemy()).map(|p| (p.x, p.y)).collect();
        self.bases(side)
            .into_iter()
            .filter_map(|b| self.site(b).filter(|s| s.kind == SiteKind::Airfield))
            .map(|s| {
                let d = enemy.iter().map(|&(x, y)| dist(s.x - x, s.y - y)).min();
                (s.object, d.unwrap_or(i32::MAX))
            })
            .max_by_key(|&(o, d)| (d, std::cmp::Reverse(o)))
            .map(|(o, _)| o)
    }

    /// Every tick: stocks for the airfields (full at the start, empty for a new owner), the
    /// production bases picked and making munitions, cargo flights scheduled and launched.
    pub(crate) fn run_munitions(&mut self, dt: f64, ev: &mut Vec<CampaignEvent>) {
        if !self.munitions_on() {
            return;
        }
        let start = self.stocks.is_empty();
        for (base, side) in self.stocked_airfields() {
            match self.stocks.get_mut(&base) {
                None => {
                    // The war's first tick: every airfield fully stocked.
                    let s = if start { Stock::full(side) } else { Stock::default() };
                    self.stocks.insert(base, Stock { side: Some(side), ..s });
                }
                Some(s) if s.side != Some(side) => {
                    *s = Stock { side: Some(side), ..Stock::default() }
                }
                Some(_) => {}
            }
        }
        for side in [Side::Blue, Side::Red] {
            let p = self.production_base(side);
            let held = p.is_some_and(|b| self.owner(b) == Some(side));
            if !held {
                let now = self.rearmost(side);
                self.production[ix(side)] = now;
                if side == Side::Blue
                    && let Some(b) = now.filter(|_| p.is_some())
                {
                    let name = self.site(b).map_or(String::new(), |s| s.name.clone());
                    let t = text!("cl-production-moved", base = name);
                    self.note(t.clone());
                    ev.push(CampaignEvent::Message(t));
                }
            }
        }
        self.produce(dt);
        self.schedule_cargo(ev);
        self.launch_cargo(ev);
    }

    /// The production bases make munitions (not while bombed), up to `PRODUCTION_CAP` full
    /// stocks.
    fn produce(&mut self, dt: f64) {
        for side in [Side::Blue, Side::Red] {
            let Some(b) = self.production_base(side) else { continue };
            if !self.site(b).is_some_and(Site::alive) {
                continue;
            }
            let Some(s) = self.stocks.get_mut(&b) else { continue };
            for &m in Munition::of_side(side) {
                let cap = m.full() * PRODUCTION_CAP;
                let c = s.counts.entry(m).or_insert(0.0);
                if *c < cap {
                    *c = (*c + m.per_hour() * dt / 3600.0).min(cap);
                }
            }
        }
    }

    /// A base out of a munition with no cargo flight coming gets one: its load (up to
    /// `CARGO_SHARE` of a full stock of each munition, what the production base has) is set
    /// aside now, the aircraft takes off `CARGO_WAIT` later.
    fn schedule_cargo(&mut self, ev: &mut Vec<CampaignEvent>) {
        for (base, side) in self.stocked_airfields() {
            let Some(from) = self.production_base(side) else { continue };
            if base == from || self.cargo_runs.iter().any(|r| r.to == base) {
                continue;
            }
            if !self.site(base).is_some_and(Site::alive) {
                continue;
            }
            let Some(s) = self.stocks.get(&base) else { continue };
            if s.empty(side).is_empty() {
                continue;
            }
            let have: Vec<(Munition, u32)> =
                Munition::of_side(side).iter().map(|&m| (m, s.count(m))).collect();
            let Some(pool) = self.stocks.get_mut(&from) else { continue };
            // Worth the flight: a quarter of a full stock at least of something it is out of
            // (else it waits for the production base to make more).
            let worth = have.iter().any(|&(m, n)| {
                n == 0 && pool.count(m) as f64 >= (m.full() * CARGO_SHARE / 2.0).max(1.0)
            });
            if !worth {
                continue;
            }
            let mut load = Vec::new();
            for (m, n) in have {
                let want =
                    ((m.full() * CARGO_SHARE) as u32).min((m.full() as u32).saturating_sub(n));
                let got = pool.take(m, want);
                if got > 0 {
                    load.push((m, got));
                }
            }
            // Nothing made yet of what it lacks: it waits.
            if load.is_empty() {
                continue;
            }
            let (id, tag) = (self.id(), self.id());
            let now = self.clock;
            self.cargo_runs.push(CargoRun {
                id,
                side,
                from,
                to: base,
                tag,
                load,
                state: CargoState::Scheduled { launch_at: now + CARGO_WAIT },
                posted: now,
            });
            // The interceptors' offer goes up with it.
            self.board_at = f64::MIN;
            if side == Side::Blue {
                let t = text!(
                    "cl-cargo-scheduled",
                    base = self.site(base).map_or(String::new(), |s| s.name.clone()),
                    from = self.site(from).map_or(String::new(), |s| s.name.clone())
                );
                self.note(t.clone());
                ev.push(CampaignEvent::Message(t));
            }
        }
    }

    /// The cargo flights due take off (one for a base no longer the side's is called off, its
    /// load back at the production base; one whose production base is bombed waits).
    fn launch_cargo(&mut self, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        for k in (0..self.cargo_runs.len()).rev() {
            let r = self.cargo_runs[k].clone();
            if !r.launch_at().is_some_and(|t| now >= t) {
                continue;
            }
            if self.owner(r.to) != Some(r.side) {
                self.cargo_runs.remove(k);
                for (m, n) in r.load {
                    self.put_back(r.from, m, n);
                }
                self.board_at = f64::MIN;
                continue;
            }
            if self.owner(r.from) != Some(r.side) || !self.site(r.from).is_some_and(Site::alive) {
                // Its production base lost or bombed: from the new one, or later.
                match self.production_base(r.side).filter(|&b| b != r.from) {
                    Some(b) if self.site(b).is_some_and(Site::alive) => {
                        self.cargo_runs[k].from = b;
                    }
                    _ => {
                        self.cargo_runs[k].state =
                            CargoState::Scheduled { launch_at: now + CARGO_RELAUNCH };
                        continue;
                    }
                }
            }
            let r = &mut self.cargo_runs[k];
            r.state = CargoState::Airborne;
            ev.push(CampaignEvent::CargoPlaneLaunched {
                id: r.id,
                side: r.side,
                from: r.from,
                to: r.to,
                tag: r.tag,
            });
        }
    }

    /// A cargo aircraft landed (its load goes into the stock of the base it came down at) or
    /// was shot down (the load is lost).
    pub(crate) fn cargo_plane_report(&mut self, r: &Report, ev: &mut Vec<CampaignEvent>) {
        let (tag, landed) = match *r {
            Report::FlightDestroyed { tag, .. } => (tag, None),
            Report::FlightLanded { tag, x, y } => (tag, Some((x, y))),
            _ => return,
        };
        let Some(k) =
            self.cargo_runs.iter().position(|c| c.tag == tag && c.state == CargoState::Airborne)
        else {
            return;
        };
        let run = self.cargo_runs.remove(k);
        self.board_at = f64::MIN;
        let name =
            |c: &CampaignState, b: ObjectId| c.site(b).map_or(String::new(), |s| s.name.clone());
        let at = landed.and_then(|(x, y)| self.airfield_near(run.side, x, y, CARGO_LAND_RADIUS));
        if let Some(base) = at {
            for &(m, n) in &run.load {
                self.put_back(base, m, n);
            }
            if run.side == Side::Blue {
                let t = text!("cl-cargo-delivered", base = name(self, base));
                self.note(t.clone());
                ev.push(CampaignEvent::Message(t));
            }
            return;
        }
        if landed.is_none() {
            self.cargo_lost.push(run.tag);
        }
        let id = if run.side == Side::Blue { "cl-cargo-lost" } else { "cl-enemy-cargo-lost" };
        let t = text!(id, base = name(self, run.to));
        self.note(t.clone());
        ev.push(CampaignEvent::Message(t));
    }

    /// After loading a save: the cargo flights that were in the air take off again soon, with
    /// their loads.
    pub(crate) fn ground_cargo(&mut self) {
        let at = self.clock + CARGO_RELAUNCH;
        for r in self.cargo_runs.iter_mut().filter(|r| r.state == CargoState::Airborne) {
            r.state = CargoState::Scheduled { launch_at: at };
        }
    }

    /// The cargo flight with campaign tag `tag`.
    pub fn cargo_run(&self, tag: u32) -> Option<&CargoRun> {
        self.cargo_runs.iter().find(|r| r.tag == tag)
    }

    /// Where interceptors wait for a cargo flight: three quarters of its way, towards the
    /// base it flies to (nearer the front than its production base in the rear).
    pub(crate) fn cargo_point(&self, r: &CargoRun) -> (i32, i32) {
        let at = |b: ObjectId| self.site(b).map_or((0, 0), |s| (s.x, s.y));
        let (a, b) = (at(r.from), at(r.to));
        ((a.0 + 3 * b.0) / 4, (a.1 + 3 * b.1) / 4)
    }

    /// The intercept offers of `side`'s board: the other side's cargo flights (scheduled or
    /// in the air).
    pub(crate) fn cargo_offers(&self, side: Side) -> Vec<(Text, Objective, i32, i32)> {
        self.cargo_runs
            .iter()
            .filter(|r| r.side == side.enemy())
            .map(|r| {
                let (x, y) = self.cargo_point(r);
                let base = self.site(r.to).map_or(String::new(), |s| s.name.clone());
                (text!("cb-intercept-cargo", base = base), Objective::Cargo { tag: r.tag }, x, y)
            })
            .collect()
    }

    /// The bases of a side whose stock is out of something.
    pub fn short_bases(&self, side: Side) -> Vec<ObjectId> {
        self.stocks
            .iter()
            .filter(|(_, s)| s.side == Some(side) && !s.empty(side).is_empty())
            .map(|(&b, _)| b)
            .collect()
    }
}
