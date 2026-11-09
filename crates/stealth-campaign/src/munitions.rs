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
            Munition::Harpoon => Some(Munition::Penguin),
            Munition::Penguin | Munition::Cbu105 => Some(Munition::Agm65),
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

    /// What the enemy's SAM sites (by class) and guns fire: their index in a base's
    /// [`CampaignState::site_demand`] (None for the aircraft's munitions).
    pub fn site_fed(self) -> Option<usize> {
        match self {
            Munition::SamLong => Some(0),
            Munition::SamMedium => Some(1),
            Munition::SamShort => Some(2),
            Munition::AaaRounds => Some(3),
            _ => None,
        }
    }

    /// A base's stock for each site (gun) it supplies, and what the production base makes an
    /// hour for each site (gun) of the theater.
    fn per_site(self) -> f64 {
        if self == Munition::AaaRounds { AAA_STOCK_PER_GUN } else { SAM_STOCK_PER_SITE }
    }

    fn per_site_hour(self) -> f64 {
        match self {
            Munition::SamLong => SAM_LONG_PER_SITE_HOUR,
            Munition::SamMedium => SAM_MEDIUM_PER_SITE_HOUR,
            Munition::SamShort => SAM_SHORT_PER_SITE_HOUR,
            _ => AAA_PER_GUN_HOUR,
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
    /// Flown by a player in the cargo jet (an airlift sortie): on its way while he is.
    Player { player: PlayerId },
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
    /// Its escort's fighters (campaign tags) in the air.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub escorts: Vec<u32>,
}

impl CargoRun {
    /// When it takes off (None: in the air, or a player's).
    pub fn launch_at(&self) -> Option<f64> {
        match self.state {
            CargoState::Scheduled { launch_at } => Some(launch_at),
            CargoState::Airborne | CargoState::Player { .. } => None,
        }
    }
}

/// How close the line from `a` to `b` passes to `p` (map units).
fn segment_dist(a: (i32, i32), b: (i32, i32), p: (i32, i32)) -> i32 {
    let (ax, ay, bx, by) = (a.0 as f64, a.1 as f64, b.0 as f64, b.1 as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = (dx * dx + dy * dy).max(1.0);
    let t = (((p.0 as f64 - ax) * dx + (p.1 as f64 - ay) * dy) / len2).clamp(0.0, 1.0);
    let (cx, cy) = (ax + t * dx, ay + t * dy);
    (p.0 as f64 - cx).hypot(p.1 as f64 - cy) as i32
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

    /// A base's full stock of `m`: for the munitions of the enemy's air defence, enough for the
    /// sites and guns it supplies (none without any), once the host has counted them.
    pub fn full_stock(&self, base: ObjectId, m: Munition) -> f64 {
        match m.site_fed() {
            Some(k) if !self.site_demand.is_empty() => {
                self.site_demand.get(&base).map_or(0, |d| d[k]) as f64 * m.per_site()
            }
            _ => m.full(),
        }
    }

    /// The sites (guns) of a class in the whole theater (None: not counted yet).
    fn theater_sites(&self, k: usize) -> Option<u32> {
        (!self.site_demand.is_empty()).then(|| self.site_demand.values().map(|d| d[k]).sum())
    }

    /// What the production base makes of `m` an hour: for the air defence's munitions as much
    /// as the theater's sites need.
    pub fn production_rate(&self, m: Munition) -> f64 {
        match m.site_fed().and_then(|k| self.theater_sites(k)) {
            Some(n) => n as f64 * m.per_site_hour(),
            None => m.per_hour(),
        }
    }

    /// The most the production base keeps of `m`: `PRODUCTION_CAP` full stocks, or the whole
    /// theater's stock of the air defence's munitions.
    pub fn production_cap(&self, m: Munition) -> f64 {
        match m.site_fed().and_then(|k| self.theater_sites(k)) {
            Some(n) => (n as f64 * m.per_site()).max(m.per_site()),
            None => m.full() * PRODUCTION_CAP,
        }
    }

    /// The munitions a base of `side` is out of (of those it keeps).
    pub fn out_of(&self, base: ObjectId, side: Side) -> Vec<Munition> {
        let Some(s) = self.stocks.get(&base) else { return Vec::new() };
        Munition::of_side(side)
            .iter()
            .copied()
            .filter(|&m| s.count(m) == 0 && self.full_stock(base, m) >= 1.0)
            .collect()
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
        let got = self.stocks.get_mut(&base).map_or(n, |s| s.take(m, n));
        let log = self.munition_log.entry(m).or_default();
        log[0] += got as u64;
        log[2] += (n - got) as u64;
        got
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
        if self.stock(base).is_some() {
            self.munition_log.entry(m).or_default()[1] += n as u64;
        }
        self.put_in(base, m, n);
    }

    /// Munitions into a base's stock (of its side's).
    fn put_in(&mut self, base: ObjectId, m: Munition, n: u32) {
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

    /// The other side's airfield nearest a side's carriers (where it would fly from with no
    /// airfield of its own).
    fn nearest_enemy_airfield(&self, side: Side) -> Option<ObjectId> {
        let carriers: Vec<(i32, i32)> = self
            .sites
            .iter()
            .filter(|s| s.kind == SiteKind::Carrier && s.alive())
            .filter(|s| self.owner(s.object) == Some(side))
            .map(|s| (s.x, s.y))
            .collect();
        self.sites
            .iter()
            .filter(|s| s.kind == SiteKind::Airfield && !s.removed)
            .filter(|s| self.owner(s.object) == Some(side.enemy()))
            .filter_map(|s| {
                let d = carriers.iter().map(|&(x, y)| dist(s.x - x, s.y - y)).min()?;
                Some((d, s.object))
            })
            .min()
            .map(|(_, o)| o)
    }

    /// The airfield a side with only carriers left is to take for its production base.
    pub fn production_target(&self, side: Side) -> Option<ObjectId> {
        self.production_target.get(ix(side)).copied().flatten()
    }

    /// A side's production base: of its intact airfields the one whose cargo flights to the
    /// others cross the fewest of the other side's places (none over the enemy's land if it
    /// can be helped), the furthest from them of those.
    fn rearmost(&self, side: Side) -> Option<ObjectId> {
        let enemy: Vec<(i32, i32)> =
            self.places.iter().filter(|p| p.owner == side.enemy()).map(|p| (p.x, p.y)).collect();
        let fields: Vec<(ObjectId, i32, i32)> = self
            .bases(side)
            .into_iter()
            .filter_map(|b| self.site(b).filter(|s| s.kind == SiteKind::Airfield))
            .map(|s| (s.object, s.x, s.y))
            .collect();
        // Flights from `a` to the others passing over an enemy place.
        let crossings = |a: (i32, i32)| {
            fields
                .iter()
                .filter(|&&(_, x, y)| {
                    (x, y) != a
                        && enemy.iter().any(|&p| segment_dist(a, (x, y), p) <= CARGO_ROUTE_CLEAR)
                })
                .count()
        };
        fields
            .iter()
            .map(|&(o, x, y)| {
                let d = enemy.iter().map(|&(ex, ey)| dist(x - ex, y - ey)).min();
                (o, crossings((x, y)), d.unwrap_or(i32::MAX))
            })
            .max_by_key(|&(o, n, d)| (std::cmp::Reverse(n), d, std::cmp::Reverse(o)))
            .map(|(o, _, _)| o)
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
                    let mut s = Stock { side: Some(side), ..Stock::default() };
                    if start {
                        for &m in Munition::of_side(side) {
                            s.counts.insert(m, self.full_stock(base, m));
                        }
                    }
                    self.stocks.insert(base, s);
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
                // A side with only carriers left: the enemy airfield nearest them is to be its
                // production base once taken (it comes with a full stock).
                let target = self.production_target[ix(side)];
                let now = match target.filter(|&t| self.owner(t) == Some(side)) {
                    Some(t) => {
                        if let Some(s) = self.stocks.get(&t).cloned()
                            && Munition::of_side(side).iter().all(|&m| s.count(m) == 0)
                        {
                            let full: Vec<(Munition, f64)> = Munition::of_side(side)
                                .iter()
                                .map(|&m| (m, self.full_stock(t, m)))
                                .collect();
                            let s = self.stocks.get_mut(&t).expect("stock");
                            s.counts.extend(full);
                        }
                        Some(t)
                    }
                    None => self.rearmost(side),
                };
                self.production_target[ix(side)] =
                    if now.is_none() { self.nearest_enemy_airfield(side) } else { None };
                self.production[ix(side)] = now;
                let name =
                    |c: &Self, b: ObjectId| c.site(b).map_or(String::new(), |s| s.name.clone());
                let news = match (now, self.production_target(side)) {
                    (Some(b), _) if p.is_some() || target.is_some() => {
                        Some(text!("cl-production-moved", base = name(self, b)))
                    }
                    (None, Some(t)) if p.is_some() => {
                        Some(text!("cl-production-lost", base = name(self, t)))
                    }
                    _ => None,
                };
                if side == Side::Blue
                    && let Some(t) = news
                {
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
            let rates: Vec<(Munition, f64, f64)> = Munition::of_side(side)
                .iter()
                .map(|&m| (m, self.production_rate(m), self.production_cap(m)))
                .collect();
            let Some(s) = self.stocks.get_mut(&b) else { continue };
            for (m, rate, cap) in rates {
                let c = s.counts.entry(m).or_insert(0.0);
                if *c < cap {
                    *c = (*c + rate * dt / 3600.0).min(cap);
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
            // One at a time, and none for a while after one was shot down.
            let cooling =
                self.cargo_cooldown.iter().any(|&(b, until)| b == base && until > self.clock);
            if base == from || cooling || self.cargo_runs.iter().any(|r| r.to == base) {
                continue;
            }
            if !self.site(base).is_some_and(Site::alive) {
                continue;
            }
            let Some(s) = self.stocks.get(&base) else { continue };
            if self.out_of(base, side).is_empty() {
                continue;
            }
            let have: Vec<(Munition, u32, f64)> = Munition::of_side(side)
                .iter()
                .map(|&m| (m, s.count(m), self.full_stock(base, m)))
                .collect();
            let Some(pool) = self.stocks.get_mut(&from) else { continue };
            // Worth the flight: a quarter of a full stock at least of something it is out of
            // (else it waits for the production base to make more).
            let worth = have.iter().any(|&(m, n, full)| {
                n == 0 && full >= 1.0 && pool.count(m) as f64 >= (full * CARGO_SHARE / 2.0).max(1.0)
            });
            if !worth {
                continue;
            }
            let mut load = Vec::new();
            for (m, n, full) in have {
                let want = ((full * CARGO_SHARE) as u32).min((full as u32).saturating_sub(n));
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
                escorts: Vec::new(),
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
                    self.put_in(r.from, m, n);
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
            self.escort_cargo(k, ev);
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
        let at = landed.and_then(|(x, y)| self.airfield_near(run.side, x, y, CARGO_LAND_RADIUS));
        match at {
            Some(base) => self.cargo_delivered(&run, base, ev),
            None => self.cargo_ended(&run, landed.is_none(), ev),
        }
    }

    /// Cargo flight `run` (taken off the list) landed its load at `base`.
    fn cargo_delivered(&mut self, run: &CargoRun, base: ObjectId, ev: &mut Vec<CampaignEvent>) {
        self.board_at = f64::MIN;
        self.recall_escorts(run, ev);
        for &(m, n) in &run.load {
            self.put_in(base, m, n);
        }
        if run.side == Side::Blue {
            let name = self.site(base).map_or(String::new(), |s| s.name.clone());
            let t = text!("cl-cargo-delivered", base = name);
            self.note(t.clone());
            ev.push(CampaignEvent::Message(t));
        }
    }

    /// Cargo flight `run` (taken off the list) did not get its load there: shot down (`lost`:
    /// the load is gone, no other one for its base for a while) or down somewhere else.
    fn cargo_ended(&mut self, run: &CargoRun, lost: bool, ev: &mut Vec<CampaignEvent>) {
        self.board_at = f64::MIN;
        self.recall_escorts(run, ev);
        if lost {
            self.cargo_lost.push(run.tag);
            let until = self.clock + CARGO_LOST_COOLDOWN;
            self.cargo_cooldown.retain(|&(b, t)| b != run.to && t > self.clock);
            self.cargo_cooldown.push((run.to, until));
        }
        let id = if run.side == Side::Blue { "cl-cargo-lost" } else { "cl-enemy-cargo-lost" };
        let name = self.site(run.to).map_or(String::new(), |s| s.name.clone());
        let t = text!(id, base = name);
        self.note(t.clone());
        ev.push(CampaignEvent::Message(t));
    }

    /// A cargo flight's escort is done.
    fn recall_escorts(&self, run: &CargoRun, ev: &mut Vec<CampaignEvent>) {
        for &tag in run.escorts.iter().filter(|t| self.patrols.iter().any(|p| p.0 == **t)) {
            ev.push(CampaignEvent::Recall { tag });
        }
    }

    /// The airlift offers of `side`'s board: its own cargo flights waiting at the production
    /// base, for a player to fly in the cargo jet. Each with the threat of the other side's SAM
    /// sites where its interceptors would wait.
    pub(crate) fn airlift_offers(&self, side: Side) -> Vec<(Text, Objective, i32, i32, i32)> {
        let name = |b: ObjectId| self.site(b).map_or(String::new(), |s| s.name.clone());
        self.cargo_runs
            .iter()
            .filter(|r| r.side == side && matches!(r.state, CargoState::Scheduled { .. }))
            .filter_map(|r| {
                let to = self.site(r.to)?;
                let (x, y) = self.cargo_point(r);
                let title = text!("cb-airlift", base = name(r.to), from = name(r.from));
                let threat = self.sam_threat(side.enemy(), x, y);
                Some((title, Objective::Airlift { run: r.id }, to.x, to.y, threat))
            })
            .collect()
    }

    /// A player takes cargo run `run` to fly it himself: it waits for him at its production base
    /// (its escort goes up with him). None: it is not waiting there any more.
    pub(crate) fn fly_airlift(
        &mut self,
        player: PlayerId,
        run: u32,
        ev: &mut Vec<CampaignEvent>,
    ) -> Option<ObjectId> {
        let k = self.cargo_runs.iter().position(|r| {
            r.id == run && matches!(r.state, CargoState::Scheduled { .. })
        })?;
        self.cargo_runs[k].state = CargoState::Player { player };
        self.board_at = f64::MIN;
        self.escort_cargo(k, ev);
        Some(self.cargo_runs[k].from)
    }

    /// The cargo run a player flies (his airlift).
    pub fn flown_cargo(&self, player: PlayerId) -> Option<&CargoRun> {
        self.cargo_runs.iter().find(|r| r.state == CargoState::Player { player })
    }

    /// A player flying a cargo run came to a stop on a friendly airbase: at the run's own one
    /// he delivers the load (his objective is met).
    pub(crate) fn airlift_landed(
        &mut self,
        player: PlayerId,
        base: ObjectId,
        ev: &mut Vec<CampaignEvent>,
    ) -> bool {
        let Some(k) = self
            .cargo_runs
            .iter()
            .position(|r| r.state == CargoState::Player { player } && r.to == base)
        else {
            return false;
        };
        let run = self.cargo_runs.remove(k);
        self.cargo_delivered(&run, base, ev);
        true
    }

    /// A player's airlift sortie ended before the load was there: lost with his aircraft
    /// (`lost`), else the run waits at its production base for the next aircraft.
    pub(crate) fn airlift_ended(
        &mut self,
        player: PlayerId,
        lost: bool,
        ev: &mut Vec<CampaignEvent>,
    ) {
        let Some(k) =
            self.cargo_runs.iter().position(|r| r.state == CargoState::Player { player })
        else {
            return;
        };
        if lost {
            let run = self.cargo_runs.remove(k);
            self.cargo_ended(&run, true, ev);
        } else {
            let run = self.cargo_runs[k].clone();
            self.recall_escorts(&run, ev);
            let r = &mut self.cargo_runs[k];
            r.state = CargoState::Scheduled { launch_at: self.clock + CARGO_RELAUNCH };
            r.escorts.clear();
            self.board_at = f64::MIN;
        }
    }

    /// After loading a save: the cargo flights that were in the air take off again soon, with
    /// their loads.
    pub(crate) fn ground_cargo(&mut self) {
        let at = self.clock + CARGO_RELAUNCH;
        for r in self
            .cargo_runs
            .iter_mut()
            .filter(|r| matches!(r.state, CargoState::Airborne | CargoState::Player { .. }))
        {
            r.state = CargoState::Scheduled { launch_at: at };
            r.escorts.clear();
        }
    }

    /// Cargo flight `k` (just up) to a base near the front gets an escort: a pair of fighters
    /// from that base meets it and flies with it until it lands (if the side has them).
    fn escort_cargo(&mut self, k: usize, ev: &mut Vec<CampaignEvent>) {
        let r = self.cargo_runs[k].clone();
        let Some(to) = self.site(r.to).map(|s| (s.x, s.y)) else { return };
        let front = self
            .places
            .iter()
            .filter(|p| p.owner == r.side.enemy())
            .any(|p| dist(p.x - to.0, p.y - to.1) <= CARGO_ESCORT_RANGE);
        // Its base's fighters take off (a bombed one has none to send).
        let up = self.bases(r.side).contains(&r.to);
        let n = CARGO_ESCORTS as u32;
        if !front
            || !up
            || self.forces(r.side).fighters < n
            || self.forces(r.side).materiel < n as f64 * SORTIE_COST
        {
            return;
        }
        self.pay(r.side, n as f64 * SORTIE_COST, Spend::Operations);
        self.take_fighters(r.side, n);
        let (x, y) = self.cargo_point(&r);
        let now = self.clock;
        for _ in 0..n {
            let tag = self.id();
            self.patrols.push((tag, r.side, now));
            self.cargo_runs[k].escorts.push(tag);
            ev.push(CampaignEvent::LaunchFighters {
                tag,
                side: r.side,
                from: r.to,
                x,
                y,
                task: crate::FighterTask::Patrol,
                escort: Some(r.tag),
            });
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
    /// Each with its threat: the cargo flight's side's live SAM sites around where the
    /// interceptors wait (the AI does not go where it is heavy).
    pub(crate) fn cargo_offers(&self, side: Side) -> Vec<(Text, Objective, i32, i32, i32)> {
        self.cargo_runs
            .iter()
            .filter(|r| r.side == side.enemy())
            .map(|r| {
                let (x, y) = self.cargo_point(r);
                let base = self.site(r.to).map_or(String::new(), |s| s.name.clone());
                let title = text!("cb-intercept-cargo", base = base);
                (title, Objective::Cargo { tag: r.tag }, x, y, self.sam_threat(r.side, x, y))
            })
            .collect()
    }

    /// The threat of a side's live SAM sites within `CARGO_HUNT_SAM_RADIUS` of a point.
    pub(crate) fn sam_threat(&self, side: Side, x: i32, y: i32) -> i32 {
        let n = self
            .sites
            .iter()
            .filter(|s| s.kind == SiteKind::Sam && s.alive() && self.owner(s.object) == Some(side))
            .filter(|s| dist(s.x - x, s.y - y) <= CARGO_HUNT_SAM_RADIUS)
            .count() as i32;
        // The friendly side's Patriot and NASAMS batteries (their stockpiles are noted where
        // they stand).
        let patriots = if side == Side::Blue {
            self.defence_stock
                .iter()
                .filter(|d| matches!(d.kind, DefenceKind::Patriot | DefenceKind::Nasams))
                .filter(|d| dist(d.x - x, d.y - y) <= CARGO_HUNT_SAM_RADIUS)
                .count() as i32
        } else {
            0
        };
        (n + patriots) * SAM_THREAT
    }

    /// The bases of a side whose stock is out of something.
    pub fn short_bases(&self, side: Side) -> Vec<ObjectId> {
        self.stocks
            .iter()
            .filter(|(b, s)| s.side == Some(side) && !self.out_of(**b, side).is_empty())
            .map(|(&b, _)| b)
            .collect()
    }
}
