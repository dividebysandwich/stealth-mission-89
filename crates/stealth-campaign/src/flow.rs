//! The flow of the war as the Ops screen shows it: how long an urgent mission has left, and
//! which side the front is moving for.

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::tuning::*;

/// When an urgent mission's moment comes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Due {
    /// At this campaign time (the raid or the helicopters lift off, the downed pilot is
    /// captured, the AWACS goes home).
    At(f64),
    /// Under way now.
    Now,
}

/// Something that moved the front: a place taken, a site destroyed (by `side`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WarEvent {
    pub at: f64,
    pub side: Side,
    pub weight: u8,
}

impl CampaignState {
    /// When objective `o`'s moment comes (None: it waits for the players).
    pub fn due(&self, o: &Objective) -> Option<Due> {
        match *o {
            Objective::Intercept { insertion } | Objective::Escort { insertion } => {
                match self.insertion(insertion)?.state {
                    InsertionState::Pending { launch_at } => Some(Due::At(launch_at)),
                    InsertionState::Enroute => Some(Due::Now),
                    _ => None,
                }
            }
            Objective::Defend { raid } => match self.raid(raid)?.state {
                RaidState::Pending { launch_at } => Some(Due::At(launch_at)),
                RaidState::Enroute => Some(Due::Now),
                _ => None,
            },
            Objective::Rescue { downed } => self
                .downed_pilot(downed)
                .filter(|d| d.state == DownedState::Waiting)
                .map(|d| Due::At(d.until)),
            Objective::Awacs { tag } => {
                self.awacs.as_ref().filter(|a| a.tag == tag).map(|a| Due::At(a.until))
            }
            Objective::Cargo { tag } => {
                Some(self.cargo_run(tag)?.launch_at().map_or(Due::Now, Due::At))
            }
            _ => None,
        }
    }

    /// The front moved for `side` (a capture weighs more than a site).
    pub(crate) fn log_war(&mut self, side: Side, weight: u8) {
        if side == Side::Neutral {
            return;
        }
        let since = self.clock - MOMENTUM_WINDOW;
        self.war_log.retain(|e| e.at >= since);
        self.war_log.push(WarEvent { at: self.clock, side, weight });
    }

    /// Which way the front moved lately: positive for the friendly side, negative for the
    /// enemy (captures and sites destroyed in the last `MOMENTUM_WINDOW`).
    pub fn momentum(&self) -> i32 {
        let since = self.clock - MOMENTUM_WINDOW;
        self.war_log
            .iter()
            .filter(|e| e.at >= since)
            .map(|e| if e.side == Side::Blue { e.weight as i32 } else { -(e.weight as i32) })
            .sum()
    }
}

/// An operation against an enemy place on the front: the steps that prepare its capture
/// (each done or not). All done, the friendly insertion goes in.
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub place: PlaceId,
    pub steps: Vec<(MissionKind, bool)>,
}

impl Operation {
    pub fn done(&self) -> usize {
        self.steps.iter().filter(|s| s.1).count()
    }

    pub fn ready(&self) -> bool {
        self.steps.iter().all(|s| s.1)
    }
}

impl CampaignState {
    /// The operation against enemy place `place` (None: not on the friendly side's front).
    /// Recon (its intelligence current), SEAD (its SAM sites down) and a strike on its
    /// airfield prepare it.
    pub fn operation(&self, place: PlaceId) -> Option<Operation> {
        self.operation_on(place, &self.front(Side::Blue))
    }

    /// [`Self::operation`] with the friendly side's front known.
    pub fn operation_on(&self, place: PlaceId, front: &[PlaceId]) -> Option<Operation> {
        if !front.contains(&place) {
            return None;
        }
        let p = self.place(place)?;
        let mut steps = vec![(MissionKind::Recon, p.intel_until.is_some_and(|t| t > self.clock))];
        if !p.defences.is_empty() {
            let up = p.defences.iter().any(|&d| self.site(d).is_some_and(Site::alive));
            steps.push((MissionKind::Sead, !up));
        }
        if let Some(a) = p.airfield {
            steps.push((MissionKind::Strike, !self.site(a).is_some_and(Site::alive)));
        }
        Some(Operation { place, steps })
    }
}

/// An urgent mission the AWACS diverts a player in the air to.
pub fn divertible(o: &Objective) -> bool {
    matches!(
        o,
        Objective::Defend { .. }
            | Objective::Intercept { .. }
            | Objective::Rescue { .. }
            | Objective::Cargo { .. }
    )
}

impl CampaignState {
    /// The urgent mission the AWACS would divert `player` to: the nearest one on the board in
    /// reach of where he flies, while his own is not achieved yet nor urgent itself.
    pub fn divert_for(&self, player: PlayerId) -> Option<&MissionOffer> {
        let s = self.sortie(player)?;
        let (x, y) = s.progress.last?;
        if s.progress.achieved || divertible(&s.offer.objective) || s.progress.down.is_some() {
            return None;
        }
        self.board
            .iter()
            .filter(|o| divertible(&o.objective) && self.offer_flyable(o, s.aircraft).is_ok())
            .map(|o| (dist(o.x - x, o.y - y), o))
            .filter(|&(d, _)| d <= DIVERT_RANGE)
            .min_by_key(|&(d, o)| (d, o.id))
            .map(|(_, o)| o)
    }
}
