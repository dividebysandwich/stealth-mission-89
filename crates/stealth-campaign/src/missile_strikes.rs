//! The missile war of the drone theaters: the enemy's Iskander strikes (a TEL drives out from
//! an Iskander site its logistics built to a firing point, prepares and fires two ballistic
//! missiles drawn from the stock of the site's supply base) and Kh-101 strikes
//! (a Tu-95 flies to a launch point over its own territory and lets its cruise missiles go) at
//! the friendly side's airfields and cities; the friendly side's FP-7 and Flamingo strikes from
//! the launch sites its logistics missions built (`blue_strikes`, phase 4).
//!
//! The board warns of each: a TEL preparing becomes a mission to destroy it before it fires
//! (`Objective::Battery` with the TEL as its only gun, a CAS offer the AI flies too), a launch
//! a warning in the log with the place and the time to impact. The simulation flies the
//! missiles (`stealth_sim::strike_missiles`); the air defences, fighters and players can shoot
//! them down. Impacts destroy what they hit (`Report::ObjectDestroyed` as any strike).

use serde::{Deserialize, Serialize};
use stealth_i18n::{Text, text};

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, Report};

/// Campaign tags of the strikes: this plus 16 times the strike's id (the carrier's; its
/// missiles' follow it, + 1 ..).
pub const MISSILE_TAG: u32 = 0x6000_0000;

/// What a strike fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StrikeMissile {
    #[default]
    Iskander,
    Kh101,
    Fp7,
    Flamingo,
}

impl StrikeMissile {
    pub fn name(self) -> &'static str {
        match self {
            StrikeMissile::Iskander => "Iskander",
            StrikeMissile::Kh101 => "Kh-101",
            StrikeMissile::Fp7 => "FP-7",
            StrikeMissile::Flamingo => "Flamingo",
        }
    }

    /// A ballistic missile (else a cruise missile).
    pub fn ballistic(self) -> bool {
        matches!(self, StrikeMissile::Iskander | StrikeMissile::Fp7)
    }

    /// Its reach (map units).
    pub fn range(self) -> i32 {
        match self {
            StrikeMissile::Iskander => ISKANDER_RANGE,
            StrikeMissile::Fp7 => FP7_RANGE,
            StrikeMissile::Kh101 | StrikeMissile::Flamingo => CRUISE_RANGE,
        }
    }

    /// Its speed (m/s, as the simulation flies it): for the time to impact.
    pub fn speed(self) -> f64 {
        match self {
            StrikeMissile::Iskander => 1500.0,
            StrikeMissile::Fp7 => 1200.0,
            StrikeMissile::Kh101 => 220.0,
            StrikeMissile::Flamingo => 250.0,
        }
    }
}

/// Where a strike is.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum StrikeState {
    /// The TEL drives to its firing point; the bomber flies to its launch point.
    #[default]
    Moving,
    /// At its firing point, setting up until then (campaign seconds).
    Preparing {
        ready_at: f64,
    },
    /// The missiles are on their way since then.
    Inbound {
        at: f64,
    },
    Ended,
}

/// A missile strike.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissileStrike {
    pub id: u32,
    pub side: Side,
    pub missile: StrikeMissile,
    /// What it is aimed at: the site's object and place.
    pub target: ObjectId,
    pub place: Option<PlaceId>,
    /// Where its launcher starts (map units), the base it comes from, and its firing (launch)
    /// point.
    pub from: (i32, i32),
    pub base: ObjectId,
    pub firing: (i32, i32),
    /// Missiles it fires.
    pub count: u8,
    pub state: StrikeState,
    /// Its TEL's ground unit (once the simulation placed it).
    #[serde(default)]
    pub unit: Option<u32>,
    /// Missiles that hit, and that were shot down.
    #[serde(default)]
    pub hits: u8,
    #[serde(default)]
    pub down: u8,
    /// A friendly launch site's (its deployment id).
    #[serde(default)]
    pub site: Option<u32>,
    /// When it ended (campaign seconds).
    #[serde(default)]
    pub ended_at: f64,
    /// Its first missile left (a launcher's lets them go once its erector is up).
    #[serde(default)]
    pub away: bool,
    /// Aimed at this strategic facility (its building), not at `target`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facility: Option<u32>,
}

impl MissileStrike {
    /// Its carrier's campaign tag (the bomber's; a TEL's missiles' base).
    pub fn tag(&self) -> u32 {
        MISSILE_TAG + self.id * 16
    }

    /// Campaign tag `tag` is one of its missiles.
    pub fn owns_missile(&self, tag: u32) -> bool {
        tag > self.tag() && tag <= self.tag() + self.count as u32
    }

    pub fn open(&self) -> bool {
        self.state != StrikeState::Ended
    }

    /// Its launcher's id with the simulation's host: a launch site's deployment, else its own.
    pub fn launcher_id(&self) -> u32 {
        self.site.unwrap_or(self.id)
    }

    /// It is fired from a TEL (else from a bomber or a launch site).
    pub fn tel(&self) -> bool {
        self.missile == StrikeMissile::Iskander
    }
}

impl CampaignState {
    /// The strike with id `id`.
    pub fn missile_strike(&self, id: u32) -> Option<&MissileStrike> {
        self.missile_strikes.iter().find(|s| s.id == id)
    }

    /// The enemy's missile strikes of the drone theaters: an Iskander TEL or a Kh-101 raid every
    /// so often at the friendly side's airfields and cities in reach.
    pub(crate) fn missile_strikes_due(&mut self, ev: &mut Vec<CampaignEvent>) {
        if !stealth_mission::drone_theater(&self.setup.theater_code) || self.rules.is_none() {
            return;
        }
        let skill = (self.setup.opponents as usize).min(3);
        let now = self.clock;
        for missile in [StrikeMissile::Iskander, StrikeMissile::Kh101] {
            let (at, every) = match missile {
                StrikeMissile::Iskander => (self.iskander_at, ISKANDER_EVERY[skill]),
                _ => (self.kh101_at, KH101_EVERY[skill]),
            };
            // The first one an interval into the war (the Iskanders' site goes up meanwhile).
            if at == 0.0 {
                self.set_strike_at(missile, now);
                if missile == StrikeMissile::Iskander {
                    self.post_red_launch_site(ev);
                }
                continue;
            }
            if now - at < self.offensive_paced(every) {
                continue;
            }
            // The TELs fire from a site the enemy's logistics built: a new one goes up when none
            // is standing or being built, and the strike is due as soon as it stands.
            if missile == StrikeMissile::Iskander && self.iskander_site().is_none() {
                self.post_red_launch_site(ev);
                continue;
            }
            self.set_strike_at(missile, now);
            if self.missile_strikes.iter().any(|s| s.open() && s.missile == missile) {
                continue;
            }
            self.order_red_strike(missile, ev);
        }
    }

    fn set_strike_at(&mut self, missile: StrikeMissile, now: f64) {
        match missile {
            StrikeMissile::Iskander => self.iskander_at = now,
            _ => self.kh101_at = now,
        }
    }

    /// The friendly airfield or city a red missile strike goes for (its object, place, x, y),
    /// the red base it comes from: the nearest in reach, cities and airfields alike.
    /// (An Iskander's TEL comes from its site at `origin`, map units: its base is the site's
    /// supply base.)
    fn red_strike_target(
        &mut self,
        missile: StrikeMissile,
        origin: Option<((i32, i32), ObjectId)>,
    ) -> Option<(ObjectId, PlaceId, ObjectId)> {
        let mut all: Vec<(ObjectId, PlaceId, ObjectId, i32)> = self
            .sites
            .iter()
            .filter(|s| s.alive() && matches!(s.kind, SiteKind::City | SiteKind::Airfield))
            .filter(|s| self.owner(s.object) == Some(Side::Blue))
            .filter_map(|s| {
                let (at, from) = match origin {
                    Some(o) => o,
                    None => {
                        let from = self.nearest_base(Side::Red, s.x, s.y)?;
                        let b = self.site(from)?;
                        ((b.x, b.y), from)
                    }
                };
                let d = dist(at.0 - s.x, at.1 - s.y);
                (d <= missile.range()).then_some((s.object, s.place?, from, d))
            })
            .collect();
        all.sort_by_key(|t| t.3);
        all.truncate(4);
        if all.is_empty() {
            return None;
        }
        let (target, place, from, _) = all[self.rng.below(all.len() as i32) as usize];
        Some((target, place, from))
    }

    /// A point `frac` of the way from `a` to `b` (map units), moved back towards `a` until it is
    /// on dry land.
    fn on_land_towards(&self, a: (i32, i32), b: (i32, i32), frac: f64) -> (i32, i32) {
        let mut f = frac;
        loop {
            let p = (a.0 + ((b.0 - a.0) as f64 * f) as i32, a.1 + ((b.1 - a.1) as f64 * f) as i32);
            if f <= 0.0 || self.terrain.is_water(p.0, p.1) != Some(true) {
                return p;
            }
            f -= 0.02;
        }
    }

    /// The enemy orders a strike of `missile`.
    pub(crate) fn order_red_strike(&mut self, missile: StrikeMissile, ev: &mut Vec<CampaignEvent>) {
        // An Iskander TEL leaves its site's helipad; its missiles come out of the stock of the
        // site's supply base.
        let site = match missile {
            StrikeMissile::Iskander => {
                let Some(d) = self.iskander_site() else { return };
                let Some(base) = d.base.or_else(|| self.supply_base_for(Side::Red, d.x, d.y))
                else {
                    return;
                };
                Some(((d.x, d.y), base))
            }
            _ => None,
        };
        let Some((target, place, base)) = self.red_strike_target(missile, site) else { return };
        let count = match site {
            Some(_) if self.munitions_on() => {
                let got = self.draw(base, crate::Munition::Iskander, ISKANDER_SALVO as u32);
                if got == 0 {
                    return;
                }
                got as u8
            }
            Some(_) => ISKANDER_SALVO,
            None => {
                if !self.pay(Side::Red, MISSILE_STRIKE_COST, Spend::Operations) {
                    return;
                }
                KH101_SALVO
            }
        };
        let Some(t) = self.site(target).map(|t| (t.x, t.y)) else { return };
        let bp = match site {
            Some((at, _)) => at,
            None => {
                let Some(b) = self.site(base) else { return };
                (b.x, b.y)
            }
        };
        // Now and then a strategic facility in reach instead.
        let facility = self.strike_facility(missile, bp);
        let tp = facility.and_then(|f| self.facility(f)).map_or(t, |f| (f.x, f.y));
        let d = dist(tp.0 - bp.0, tp.1 - bp.1).max(1);
        let id = self.id();
        let (firing, count) = match missile {
            // The TEL drives a little way out from its site towards the target.
            StrikeMissile::Iskander => {
                let frac = (TEL_DRIVE as f64 / d as f64).min(0.3);
                (self.on_land_towards(bp, tp, frac), count)
            }
            // The bomber launches well short of the front, over its own side.
            _ => (self.on_land_towards(bp, tp, KH101_LAUNCH_FRAC), count),
        };
        let strike = MissileStrike {
            id,
            side: Side::Red,
            missile,
            target,
            place: Some(place),
            from: bp,
            base,
            firing,
            count,
            state: StrikeState::Moving,
            unit: None,
            hits: 0,
            down: 0,
            site: None,
            ended_at: 0.0,
            away: false,
            facility,
        };
        let aims = self.strike_aims(&strike);
        match missile {
            StrikeMissile::Iskander => ev.push(CampaignEvent::LauncherDeploy {
                strike: id,
                missile,
                from: bp,
                to: firing,
                blue: false,
                fixed: false,
            }),
            _ => ev.push(CampaignEvent::CarrierLaunched {
                strike: id,
                missile,
                base,
                launch: firing,
                aims,
                tag: strike.tag(),
            }),
        }
        self.missile_strikes.push(strike);
    }

    /// Where a strike's missiles go (map units) and the object each is aimed at: the target,
    /// the rest of a salvo a little apart round it.
    pub(crate) fn strike_aims(&self, s: &MissileStrike) -> Vec<((i32, i32), Option<ObjectId>)> {
        // A facility's building (no world object), or the target object.
        let (at, object) = match s.facility.and_then(|f| self.facility(f)) {
            Some(f) => ((f.x, f.y), None),
            None => {
                let Some(t) = self.site(s.target) else { return Vec::new() };
                ((t.x, t.y), Some(s.target))
            }
        };
        (0..s.count as i32)
            .map(|k| {
                let off = [(0, 0), (3, 2), (-3, 2), (2, -3), (-2, -3), (4, 0)][(k as usize) % 6];
                ((at.0 + off.0, at.1 + off.1), object.filter(|_| k == 0))
            })
            .collect()
    }

    /// A friendly strategic facility an enemy strike of `missile` from `from` goes for, one
    /// time in `FACILITY_STRIKE_ONE_IN`: the most valuable in reach (not one just rebuilt).
    fn strike_facility(&mut self, missile: StrikeMissile, from: (i32, i32)) -> Option<u32> {
        // (No draw of the dice in a campaign without facilities.)
        if self.facilities.is_empty() || self.rng.below(FACILITY_STRIKE_ONE_IN) != 0 {
            return None;
        }
        let now = self.clock;
        self.facilities
            .iter()
            .filter(|f| f.alive() && self.facility_side(f) == Side::Blue)
            .filter(|f| f.restored_at.is_none_or(|t| now - t >= FACILITY_GRACE))
            .filter(|f| dist(f.x - from.0, f.y - from.1) <= missile.range())
            .max_by_key(|f| (f.kind.value(), std::cmp::Reverse(f.id)))
            .map(|f| f.id)
    }

    /// The strikes' reports from the simulation.
    pub(crate) fn missile_report(&mut self, r: &Report, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        match *r {
            Report::LauncherPlaced { strike, unit } => {
                for s in self.missile_strikes.iter_mut().filter(|s| s.launcher_id() == strike) {
                    s.unit = Some(unit);
                }
                // (An enemy Iskander site's parked TEL.)
                for d in self.deployments.iter_mut().filter(|d| d.id == strike) {
                    d.unit = Some(unit);
                }
            }
            Report::LauncherArrived { strike } => {
                let Some(k) = self.missile_strikes.iter().position(|s| s.id == strike) else {
                    return;
                };
                if self.missile_strikes[k].state != StrikeState::Moving {
                    return;
                }
                let prep = if self.missile_strikes[k].side == Side::Red {
                    TEL_PREPARE
                } else {
                    SITE_PREPARE
                };
                self.missile_strikes[k].state = StrikeState::Preparing { ready_at: now + prep };
                let s = self.missile_strikes[k].clone();
                if s.side == Side::Red {
                    let text = text!(
                        "cl-tel-preparing",
                        place = self.near_name(s.firing.0, s.firing.1),
                        target = self.target_name(s.target),
                        mins = (prep / 60.0).round() as i64
                    );
                    self.note(text.clone());
                    ev.push(CampaignEvent::Message(text));
                    self.board_at = f64::MIN;
                }
            }
            Report::LauncherLost { strike } => {
                let Some(k) =
                    self.missile_strikes.iter().position(|s| s.launcher_id() == strike && s.open())
                else {
                    return;
                };
                // (Or as it was told to fire, before its missiles left.)
                let st = self.missile_strikes[k].state;
                let away = self.missile_strikes[k].away;
                if matches!(st, StrikeState::Moving | StrikeState::Preparing { .. })
                    || matches!(st, StrikeState::Inbound { .. }) && !away
                {
                    self.missile_strikes[k].state = StrikeState::Ended;
                    self.missile_strikes[k].ended_at = self.clock;
                    let s = self.missile_strikes[k].clone();
                    let id =
                        if s.side == Side::Red { "cl-tel-destroyed" } else { "cl-launcher-lost" };
                    let text = text!(id, place = self.near_name(s.firing.0, s.firing.1));
                    self.note(text.clone());
                    ev.push(CampaignEvent::Message(text));
                    self.board_at = f64::MIN;
                }
            }
            Report::MissilesAway { tag } => {
                let Some(k) = self.missile_strikes.iter().position(|s| s.tag() == tag) else {
                    return;
                };
                self.missile_strikes[k].away = true;
                // A launcher's fire was announced when it was ordered.
                if !matches!(self.missile_strikes[k].state, StrikeState::Moving) {
                    return;
                }
                self.missile_strikes[k].state = StrikeState::Inbound { at: now };
                let s = self.missile_strikes[k].clone();
                self.launch_warning(&s, ev);
            }
            Report::MissileImpact { tag, .. } => {
                if let Some(k) = self.missile_strikes.iter().position(|s| s.owns_missile(tag)) {
                    self.missile_strikes[k].hits += 1;
                    self.strike_done(k, ev);
                }
            }
            Report::FlightDestroyed { tag, .. } => {
                if let Some(k) = self.missile_strikes.iter().position(|s| s.owns_missile(tag)) {
                    self.missile_strikes[k].down += 1;
                    self.strike_done(k, ev);
                } else if let Some(k) = self
                    .missile_strikes
                    .iter()
                    .position(|s| s.tag() == tag && s.state == StrikeState::Moving)
                {
                    // Its bomber shot down before it fired.
                    self.missile_strikes[k].state = StrikeState::Ended;
                    self.missile_strikes[k].ended_at = self.clock;
                }
            }
            _ => {}
        }
    }

    /// The launch is on the board: the place it is aimed at, the time to impact.
    fn launch_warning(&mut self, s: &MissileStrike, ev: &mut Vec<CampaignEvent>) {
        let Some(t) = self.site(s.target) else { return };
        let d = dist(t.x - s.firing.0, t.y - s.firing.1) as f64 * MAP_UNIT_M;
        // A launcher's missiles leave once its erector is up.
        let erect = if s.missile == StrikeMissile::Kh101 { 0.0 } else { LAUNCHER_ERECT_SECS };
        let secs = (d / s.missile.speed() + erect).round() as i64;
        let target = self.target_name(s.target);
        let id = match (s.side, s.missile.ballistic()) {
            (Side::Red, true) => "cl-tbm-launch",
            (Side::Red, false) => "cl-cm-launch",
            (_, _) => "cl-blue-missile-launch",
        };
        let text = text!(
            id,
            count = s.count as i64,
            missile = s.missile.name(),
            target = target,
            mins = (secs / 60).max(1),
            secs = secs
        );
        self.note(text.clone());
        ev.push(CampaignEvent::Message(text));
    }

    /// A missile of strike `k` came down or was shot down: when all are, the strike's result.
    fn strike_done(&mut self, k: usize, ev: &mut Vec<CampaignEvent>) {
        let s = &self.missile_strikes[k];
        if (s.hits + s.down) < s.count || s.state == StrikeState::Ended {
            return;
        }
        let s = s.clone();
        self.missile_strikes[k].state = StrikeState::Ended;
        self.missile_strikes[k].ended_at = self.clock;
        let id = if s.side == Side::Red { "cl-missile-strike-over" } else { "cl-blue-strike-over" };
        let text = text!(
            id,
            missile = s.missile.name(),
            target = self.target_name(s.target),
            hits = s.hits as i64,
            down = s.down as i64
        );
        self.note(text.clone());
        ev.push(CampaignEvent::Message(text));
    }

    /// Preparing launchers ready: they fire.
    pub(crate) fn run_missile_strikes(&mut self, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        for k in 0..self.missile_strikes.len() {
            let StrikeState::Preparing { ready_at } = self.missile_strikes[k].state else {
                continue;
            };
            if now < ready_at {
                continue;
            }
            let s = self.missile_strikes[k].clone();
            let aims = self.strike_aims(&s);
            if aims.is_empty() {
                self.missile_strikes[k].state = StrikeState::Ended;
                self.missile_strikes[k].ended_at = self.clock;
                continue;
            }
            // Fired, and announced.
            self.missile_strikes[k].state = StrikeState::Inbound { at: now };
            self.launch_warning(&s, ev);
            ev.push(CampaignEvent::LauncherFire {
                strike: s.launcher_id(),
                missile: s.missile,
                aims,
                tag: s.tag(),
            });
        }
        // Long over: forgotten.
        let keep_from = self.missile_strikes.len().saturating_sub(MISSILE_STRIKES_KEPT);
        let mut k = 0;
        self.missile_strikes.retain(|s| {
            k += 1;
            s.open() || k > keep_from
        });
    }

    /// A strike's launcher left over from before the campaign was saved is gone with the
    /// simulation (loading): the strike ends.
    pub(crate) fn ground_missile_strikes(&mut self) {
        for s in &mut self.missile_strikes {
            if s.open() && s.site.is_none() {
                s.state = StrikeState::Ended;
            }
        }
    }

    /// The board's TEL hunts: each enemy TEL out and not fired yet (its gun list its unit), and
    /// each standing enemy Iskander site (its parked TEL).
    pub(crate) fn tel_hunts(&self) -> Vec<(Text, Objective, i32, i32)> {
        let sites = self
            .deployments
            .iter()
            .filter(|d| d.side() == Side::Red && d.built && d.seen)
            .filter_map(|d| {
                let (x, y) = d.unit_at();
                Some((
                    text!("cb-iskander-site", place = d.name.as_str()),
                    Objective::Battery { battery: d.id, aaa: vec![d.unit?] },
                    x,
                    y,
                ))
            });
        self.missile_strikes
            .iter()
            .filter(|s| s.side == Side::Red && s.tel())
            .filter(|s| matches!(s.state, StrikeState::Moving | StrikeState::Preparing { .. }))
            .filter_map(|s| {
                let unit = s.unit?;
                Some((
                    text!("cb-tel-hunt", place = self.near_name(s.firing.0, s.firing.1)),
                    Objective::Battery { battery: s.id, aaa: vec![unit] },
                    s.firing.0,
                    s.firing.1,
                ))
            })
            .chain(sites)
            .collect()
    }

    /// The enemy's Iskander site standing (built, its TEL seen in the simulation).
    pub(crate) fn iskander_site(&self) -> Option<&Deployment> {
        self.deployments.iter().find(|d| d.kind == DefenceKind::IskanderSite && d.built && d.seen)
    }

    /// The enemy posts a new Iskander site, unless one stands or is being built: its loads go
    /// by Mi-24 from its nearest airfield, one after the other.
    fn post_red_launch_site(&mut self, ev: &mut Vec<CampaignEvent>) {
        let kind = DefenceKind::IskanderSite;
        if self.deployments.iter().any(|d| d.kind == kind) {
            return;
        }
        let Some((x, y)) = self.launch_spot(kind) else { return };
        let d = self.new_site(kind, x, y);
        let text = text!("cl-red-launch-site-posted", place = d.name.as_str());
        self.deployments.push(d);
        self.note(text.clone());
        ev.push(CampaignEvent::Message(text));
        self.board_at = f64::MIN;
    }

    /// A launch site of `kind` posted at (x, y) for its side's logistics to build.
    fn new_site(&mut self, kind: DefenceKind, x: i32, y: i32) -> Deployment {
        let now = self.clock;
        let loads = crate::deploy::loads(kind);
        Deployment {
            id: self.id(),
            x,
            y,
            kind,
            name: self.near_name(x, y),
            by: None,
            posted: now,
            expires: now + DEPLOY_TIMEOUT,
            loads,
            delivered: Vec::new(),
            ai: Vec::new(),
            // (The enemy's first helicopter goes at once.)
            open_since: if kind.side() == Side::Red { f64::MIN } else { now },
            built: false,
            seen: false,
            base: self.supply_base_for(kind.side(), x, y),
            crates: vec![CrateState::AtBase; loads as usize],
            unit: None,
        }
    }

    /// The friendly side's missile war of the drone theaters: a launch site of each kind
    /// (FP-7, Flamingo) is posted for the logistics missions to build at a spot the war picks,
    /// clear of the airfields; the built sites fire at the enemy's military targets in reach.
    pub(crate) fn blue_missile_war(&mut self, ev: &mut Vec<CampaignEvent>) {
        if !stealth_mission::drone_theater(&self.setup.theater_code) || self.rules.is_none() {
            return;
        }
        self.post_launch_sites(ev);
        self.launch_site_salvos(ev);
    }

    /// A new launch site of a kind none of which stands or is being built, now and then.
    fn post_launch_sites(&mut self, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        if self.launch_site_at != 0.0 && now - self.launch_site_at < LAUNCH_SITE_EVERY {
            return;
        }
        self.launch_site_at = now;
        for kind in [DefenceKind::Fp7Site, DefenceKind::FlamingoSite] {
            if self.deployments.iter().any(|d| d.kind == kind) {
                continue;
            }
            let Some((x, y)) = self.launch_spot(kind) else { continue };
            let d = self.new_site(kind, x, y);
            let text = text!("cl-launch-site-posted", site = d.label());
            self.deployments.push(d);
            self.note(text.clone());
            ev.push(CampaignEvent::Message(text));
            self.board_at = f64::MIN;
        }
    }

    /// The enemy's military targets a friendly missile goes for (the site's object, where, a SAM
    /// site): its SAM sites and its airfields.
    fn red_military(&self) -> Vec<(ObjectId, i32, i32, bool)> {
        self.sites
            .iter()
            .filter(|s| s.alive() && matches!(s.kind, SiteKind::Sam | SiteKind::Airfield))
            .filter(|s| self.owner(s.object) == Some(Side::Red))
            .map(|s| (s.object, s.x, s.y, s.kind == SiteKind::Sam))
            .collect()
    }

    /// The friendly cities and airfields (the site's object, where; never a SAM site).
    fn blue_targets(&self) -> Vec<(ObjectId, i32, i32, bool)> {
        self.sites
            .iter()
            .filter(|s| s.alive() && matches!(s.kind, SiteKind::City | SiteKind::Airfield))
            .filter(|s| self.owner(s.object) == Some(Side::Blue))
            .map(|s| (s.object, s.x, s.y, false))
            .collect()
    }

    /// A spot for a new launch site of `kind` (map units): in its side's territory, on land,
    /// clear of the towns and the airfields, behind the front within its missile's reach of an
    /// enemy target (the enemy's: of a friendly city or airfield).
    pub fn launch_spot(&mut self, kind: DefenceKind) -> Option<(i32, i32)> {
        let missile = kind.launch_missile()?;
        let side = kind.side();
        let targets = if side == Side::Blue { self.red_military() } else { self.blue_targets() };
        if targets.is_empty() {
            return None;
        }
        let ours: Vec<(i32, i32)> =
            self.places.iter().filter(|p| p.owner == side).map(|p| (p.x, p.y)).collect();
        let airfields: Vec<(i32, i32)> = self
            .sites
            .iter()
            .filter(|s| s.kind == SiteKind::Airfield)
            .map(|s| (s.x, s.y))
            .collect();
        for _ in 0..80 {
            let t = targets[self.rng.below(targets.len() as i32) as usize];
            let p = *ours.iter().min_by_key(|p| dist(p.0 - t.1, p.1 - t.2))?;
            let d = dist(p.0 - t.1, p.1 - t.2).max(1) as f64;
            let (ux, uy) = ((p.0 - t.1) as f64 / d, (p.1 - t.2) as f64 / d);
            let frac = 0.45 + self.rng.below(30) as f64 / 100.0;
            // Well behind the front, but on the map (a cruise missile reaches much further).
            let reach = missile.range().min(FP7_RANGE) as f64 * frac;
            let side = (self.rng.below(41) - 20) as f64 / 100.0 * reach;
            let (x, y) =
                (t.1 + (ux * reach - uy * side) as i32, t.2 + (uy * reach + ux * side) as i32);
            if self.deploy_refusal(x, y, kind, &[]).is_some() {
                continue;
            }
            if airfields.iter().any(|a| dist(a.0 - x, a.1 - y) < LAUNCH_SITE_AIRFIELD_CLEAR) {
                continue;
            }
            return Some((x, y));
        }
        None
    }

    /// Built launch sites, rested since their last salvo and with missiles at their supply base,
    /// set up a new one at the enemy's nearest military target in reach (a SAM site first).
    fn launch_site_salvos(&mut self, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        let sites: Vec<(u32, DefenceKind, (i32, i32), Option<ObjectId>)> = self
            .deployments
            .iter()
            .filter(|d| d.built && d.kind.launch_site() && d.side() == Side::Blue)
            .map(|d| (d.id, d.kind, d.unit_at(), d.base))
            .collect();
        let targets = self.red_military();
        for (site, kind, at, base) in sites {
            let Some(missile) = kind.launch_missile() else { continue };
            let busy = self.missile_strikes.iter().any(|s| s.site == Some(site) && s.open());
            let rested = self
                .missile_strikes
                .iter()
                .filter(|s| s.site == Some(site))
                .all(|s| now - s.ended_at >= LAUNCH_SITE_REST * self.hq_slow(Side::Blue));
            if busy || !rested {
                continue;
            }
            let Some(&(target, _, _, _)) = targets
                .iter()
                .filter(|t| dist(t.1 - at.0, t.2 - at.1) <= missile.range())
                .min_by_key(|t| (!t.3, dist(t.1 - at.0, t.2 - at.1)))
            else {
                continue;
            };
            let count = if missile.ballistic() { FP7_SALVO } else { FLAMINGO_SALVO };
            let munition = match missile {
                StrikeMissile::Fp7 => crate::Munition::Fp7,
                _ => crate::Munition::Flamingo,
            };
            if self.munitions_on()
                && let Some(b) = base
                && self.draw(b, munition, count as u32) < count as u32
            {
                continue;
            }
            let id = self.id();
            let place = self.site(target).and_then(|s| s.place);
            self.missile_strikes.push(MissileStrike {
                id,
                side: Side::Blue,
                missile,
                target,
                place,
                from: at,
                base: base.unwrap_or(0),
                firing: at,
                count,
                state: StrikeState::Preparing { ready_at: now + SITE_PREPARE },
                unit: None,
                hits: 0,
                down: 0,
                site: Some(site),
                ended_at: 0.0,
                away: false,
                facility: None,
            });
            // A loaded launcher at the site (the last one left empty).
            ev.push(CampaignEvent::LauncherDeploy {
                strike: site,
                missile,
                from: at,
                to: at,
                blue: true,
                fixed: true,
            });
        }
    }

    /// The name of the place nearest a point (map units).
    pub(crate) fn near_name(&self, x: i32, y: i32) -> String {
        self.places
            .iter()
            .min_by_key(|p| dist(p.x - x, p.y - y))
            .map_or_else(String::new, |p| p.name.clone())
    }
}

/// Metres per map unit.
const MAP_UNIT_M: f64 = 15.625;
