//! The missile war of the drone theaters: the enemy's Iskander strikes (a TEL drives out from
//! a red base to a firing point, prepares and fires two ballistic missiles) and Kh-101 strikes
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
            // The first one an interval into the war.
            if at == 0.0 {
                self.set_strike_at(missile, now);
                continue;
            }
            if now - at < self.paced(every) {
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
    fn red_strike_target(
        &mut self,
        missile: StrikeMissile,
    ) -> Option<(ObjectId, PlaceId, ObjectId)> {
        let mut all: Vec<(ObjectId, PlaceId, ObjectId, i32)> = self
            .sites
            .iter()
            .filter(|s| s.alive() && matches!(s.kind, SiteKind::City | SiteKind::Airfield))
            .filter(|s| self.owner(s.object) == Some(Side::Blue))
            .filter_map(|s| {
                let from = self.nearest_base(Side::Red, s.x, s.y)?;
                let b = self.site(from)?;
                let d = dist(b.x - s.x, b.y - s.y);
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
    fn order_red_strike(&mut self, missile: StrikeMissile, ev: &mut Vec<CampaignEvent>) {
        let Some((target, place, base)) = self.red_strike_target(missile) else { return };
        if !self.pay(Side::Red, MISSILE_STRIKE_COST, Spend::Operations) {
            return;
        }
        let (Some(t), Some(b)) = (self.site(target), self.site(base)) else { return };
        let (tp, bp) = ((t.x, t.y), (b.x, b.y));
        let d = dist(tp.0 - bp.0, tp.1 - bp.1).max(1);
        let id = self.id();
        let (firing, count) = match missile {
            // The TEL drives a little way out from its base towards the target.
            StrikeMissile::Iskander => {
                let frac = (TEL_DRIVE as f64 / d as f64).min(0.3);
                (self.on_land_towards(bp, tp, frac), ISKANDER_SALVO)
            }
            // The bomber launches well short of the front, over its own side.
            _ => (self.on_land_towards(bp, tp, KH101_LAUNCH_FRAC), KH101_SALVO),
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
        let Some(t) = self.site(s.target) else { return Vec::new() };
        (0..s.count as i32)
            .map(|k| {
                let off = [(0, 0), (3, 2), (-3, 2), (2, -3), (-2, -3), (4, 0)][(k as usize) % 6];
                ((t.x + off.0, t.y + off.1), (k == 0).then_some(s.target))
            })
            .collect()
    }

    /// The strikes' reports from the simulation.
    pub(crate) fn missile_report(&mut self, r: &Report, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        match *r {
            Report::LauncherPlaced { strike, unit } => {
                if let Some(s) = self.missile_strikes.iter_mut().find(|s| s.id == strike) {
                    s.unit = Some(unit);
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
                let Some(k) = self.missile_strikes.iter().position(|s| s.id == strike) else {
                    return;
                };
                // (Or as it was told to fire: nothing left it.)
                let st = self.missile_strikes[k].state;
                if matches!(st, StrikeState::Moving | StrikeState::Preparing { .. })
                    || matches!(st, StrikeState::Inbound { at } if now - at < 1.0)
                {
                    self.missile_strikes[k].state = StrikeState::Ended;
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
                }
            }
            _ => {}
        }
    }

    /// The launch is on the board: the place it is aimed at, the time to impact.
    fn launch_warning(&mut self, s: &MissileStrike, ev: &mut Vec<CampaignEvent>) {
        let Some(t) = self.site(s.target) else { return };
        let d = dist(t.x - s.firing.0, t.y - s.firing.1) as f64 * MAP_UNIT_M;
        let secs = (d / s.missile.speed()).round() as i64;
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
        // A friendly launch site reloads for its next salvo.
        if let Some(site) = s.site {
            self.site_reloads(site, ev);
        }
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
                continue;
            }
            // Fired, and announced.
            self.missile_strikes[k].state = StrikeState::Inbound { at: now };
            self.launch_warning(&s, ev);
            ev.push(CampaignEvent::LauncherFire {
                strike: s.id,
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

    /// The board's TEL hunts: each enemy TEL out and not fired yet (its gun list its unit).
    pub(crate) fn tel_hunts(&self) -> Vec<(Text, Objective, i32, i32)> {
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
            .collect()
    }

    /// A friendly launch site's next salvo (`blue_strikes`: phase 4 fills it in).
    pub(crate) fn site_reloads(&mut self, _site: u32, _ev: &mut Vec<CampaignEvent>) {}

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
