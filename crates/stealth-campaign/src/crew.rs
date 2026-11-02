//! Coop scaling: the more players fly the war, the faster the enemy's tempo, the more
//! fighters it puts up and the more it earns to pay for them. Never their skill: each
//! player's own fights stay as hard as the opponents' level says.

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::tuning::*;

/// The players in the war, as the enemy's tempo counts them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Crew {
    /// Players connected now (the server's count; 1 in single player).
    pub online: u32,
    /// The count the tempo follows: it rises to `online` over `CREW_RISE` per player, falls
    /// over `CREW_FALL` (a player dropping out for a moment changes little).
    pub level: f64,
}

impl Default for Crew {
    fn default() -> Self {
        Self { online: 1, level: 1.0 }
    }
}

/// The enemy's tempo for `players` players: sub-linear (players work together), 1 for one.
pub fn crew_scale(players: f64) -> f64 {
    players.max(1.0).powf(CREW_EXPONENT).min(CREW_SCALE_MAX)
}

impl CampaignState {
    /// The server's count of players connected now.
    pub fn set_crew(&mut self, online: usize) {
        self.crew.online = online.max(1) as u32;
    }

    /// The tempo follows the players online.
    pub(crate) fn run_crew(&mut self, dt: f64) {
        let c = &mut self.crew;
        let to = c.online.max(1) as f64;
        c.level = if c.level < to {
            (c.level + dt / CREW_RISE).min(to)
        } else {
            (c.level - dt / CREW_FALL).max(to)
        };
    }

    /// How much faster the enemy works for the players flying the war (1 alone, or with
    /// coop scaling off).
    pub fn pace(&self) -> f64 {
        if self.setup.crew_scaling { crew_scale(self.crew.level) } else { 1.0 }
    }

    /// `every` (an enemy's interval) at the crew's pace.
    pub(crate) fn paced(&self, every: f64) -> f64 {
        every / self.pace()
    }

    /// More of an enemy's aircraft for the crew: `per` for each step of pace beyond one.
    pub(crate) fn crew_extra(&self, per: f64) -> usize {
        ((self.pace() - 1.0) * per).round().max(0.0) as usize
    }

    /// Offers on the board at most: more for more players, each flight its own work.
    pub(crate) fn board_size(&self) -> usize {
        let more = if self.setup.crew_scaling { self.crew.level.round() as usize - 1 } else { 0 };
        BOARD_SIZE + (CREW_OFFERS * more).min(CREW_OFFERS_MAX)
    }
}
