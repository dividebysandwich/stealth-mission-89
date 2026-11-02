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
