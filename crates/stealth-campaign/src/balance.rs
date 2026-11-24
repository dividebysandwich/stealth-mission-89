//! Help for the friendly side where it is outnumbered: a theater that starts it with far fewer
//! places and cities than the enemy (`underdog`, fixed for the campaign), and a war that has
//! cost it ground (`comeback`). Both raise its income and slow the enemy's offensive.

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::tuning::*;

/// The sides as they started the war.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct StartBalance {
    /// What each side's places and cities are worth (blue, red; `strength`).
    pub strength: [f64; 2],
    /// Places each side held (blue, red).
    pub held: [u16; 2],
}

impl CampaignState {
    /// A side's places and cities, weighed as its income weighs them (without the base amount
    /// both sides get).
    fn strength(&self, side: Side) -> f64 {
        let (places, cities) = self.producers(side);
        INCOME_PLACE * places as f64 + INCOME_CITY * cities as f64
    }

    /// The sides now, for `start` (when the campaign is made, or an older save is loaded).
    pub(crate) fn measure_start(&self) -> StartBalance {
        StartBalance {
            strength: [self.strength(Side::Blue), self.strength(Side::Red)],
            held: [self.held(Side::Blue) as u16, self.held(Side::Red) as u16],
        }
    }

    /// How far the theater started the friendly side outnumbered: 1 when it is about even
    /// (within `UNDERDOG_EVEN`), growing with the root of the enemy's lead beyond, at most
    /// `UNDERDOG_MAX`.
    pub fn underdog(&self) -> f64 {
        let [blue, red] = self.start.strength;
        if blue <= 0.0 || red <= 0.0 {
            return 1.0;
        }
        (red / blue / UNDERDOG_EVEN).powf(UNDERDOG_EXP).clamp(1.0, UNDERDOG_MAX)
    }

    /// The friendly side has lost ground: 1 while it holds what it started with, up by
    /// `COMEBACK` for all of it lost, at most `COMEBACK_MAX`.
    pub fn comeback(&self) -> f64 {
        let start = self.start.held[0] as f64;
        if start <= 0.0 {
            return 1.0;
        }
        let lost = (1.0 - self.held(Side::Blue) as f64 / start).max(0.0);
        (1.0 + COMEBACK * lost).min(COMEBACK_MAX)
    }

    /// Tanks and MANPADS teams each friendly place has beyond its usual garrison: some for
    /// every step of `underdog` beyond 1 (`UNDERDOG_GARRISON`).
    pub fn garrison_extra(&self) -> u8 {
        ((self.underdog() - 1.0) * UNDERDOG_GARRISON) as u8
    }

    /// Both: what the friendly side's income and air arm are scaled by.
    pub fn friendly_relief(&self) -> f64 {
        self.underdog() * self.comeback()
    }

    /// How much faster a side's places replace their guns and armour (the friendly side's
    /// when it has lost ground).
    pub(crate) fn repair_relief(&self, side: Side) -> f64 {
        if side == Side::Blue { self.comeback() } else { 1.0 }
    }

    /// `every` (an interval of the enemy's offensive: insertions, raids, convoys, drones and
    /// missiles) at the crew's pace, and longer for an outnumbered friendly side.
    pub(crate) fn offensive_paced(&self, every: f64) -> f64 {
        self.paced(every) * self.friendly_relief().sqrt() * self.hq_slow(Side::Red)
    }

    /// How much longer a side's operations take to come while its headquarters is destroyed
    /// (`HQ_SLOWDOWN`; 1 while it stands).
    pub fn hq_slow(&self, side: Side) -> f64 {
        if self.facility_down(side, crate::FacilityKind::Headquarters) { HQ_SLOWDOWN } else { 1.0 }
    }
}
