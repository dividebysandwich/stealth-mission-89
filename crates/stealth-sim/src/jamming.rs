//! The EA-18G Growler's jamming, simplified for the game.
//!
//! Each ALQ-99 pod the Growler carries jams one thing, as set on the JAM page: an enemy radar
//! (a site), or a sector 60 degrees wide centred on a bearing from the aircraft. With the
//! transmitters on, a radar jammed:
//! - sees every friendly aircraft much less (its detection score cut) until the aircraft comes
//!   within its burn-through range;
//! - guides its missiles worse (a radar guided missile it fires may lose its lock, as against
//!   the aircraft's own ECM).
//!
//! The noise itself is seen: a jamming Growler is the easier to detect (its EMV goes up).

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::math::{approx, s16};

/// The AN/ALQ-99 jammer pod's weapon type (never released).
pub const ALQ99_WEAPON: i32 = 29;
/// Pods a Growler works at most (the JAM page's).
pub const PODS: usize = 3;
/// How far (map units, about 150 km) a pod's noise reaches a radar, and half the width of a
/// sector (BAM, 30 degrees).
pub const JAM_REACH: i32 = 9600;
pub const SECTOR_HALF: i32 = 0x1555;
/// What a jammed radar sees (percent) of an aircraft beyond its burn-through range (map units,
/// about 10 km).
pub const JAM_SCORE_PCT: i32 = 35;
pub const BURN_THROUGH: i32 = 640;
/// What jamming adds to the Growler's own EMV.
pub const JAM_EMV: i32 = 20;

/// What one pod jams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JamTarget {
    /// A radar site (world object index).
    Site(u16),
    /// A sector round this bearing from the aircraft (BAM, clockwise from north).
    Sector(u16),
}

/// The JAM page's report to the simulation each frame: the transmitters on, each pod's target.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JamInput {
    pub on: bool,
    pub pods: [Option<JamTarget>; PODS],
}

impl Sim {
    /// ALQ-99 pods the active pilot carries.
    pub fn jammer_pods(&self) -> usize {
        self.pl.bays.iter().filter(|b| b.0 == ALQ99_WEAPON && b.1 > 0).count()
    }

    /// The active pilot jams: a Growler with pods, its transmitters on, a pod set.
    pub(crate) fn jamming(&self) -> bool {
        self.pl.growler
            && self.pl.eject == 0
            && self.jammer_pods() > 0
            && self.pl.last_input.jam.is_some_and(|j| j.on && j.pods.iter().any(Option::is_some))
    }

    /// The jamming pilots in the world now: where each is (map units) and its pods' targets
    /// (as many as it carries pods).
    fn jammers(&self) -> Vec<((i32, i32), Vec<JamTarget>)> {
        let mut out = Vec::new();
        let mut add = |pl: &crate::pilot::Pilot| {
            if !pl.growler || pl.eject != 0 || pl.absent {
                return;
            }
            let pods = pl.bays.iter().filter(|b| b.0 == ALQ99_WEAPON && b.1 > 0).count();
            let Some(j) = pl.last_input.jam.filter(|j| j.on) else { return };
            let set: Vec<JamTarget> = j.pods.iter().flatten().copied().take(pods).collect();
            if !set.is_empty() {
                out.push(((pl.p.mx, pl.p.my), set));
            }
        };
        add(&self.pl);
        for (k, p) in self.parked.iter().enumerate() {
            if k != self.pl.id as usize {
                add(p);
            }
        }
        out
    }

    /// Site `i` (a world object) is jammed now by some Growler.
    pub fn site_jammed(&self, i: usize) -> bool {
        let Some(o) = self.objects.get(i) else { return false };
        self.jammers().iter().any(|&((x, y), ref set)| {
            let (dx, dy) = (s16(o.x - x), s16(o.y - y));
            if approx(dx, dy) > JAM_REACH {
                return false;
            }
            let brg = crate::math::bearing(dx, s16(-dy));
            set.iter().any(|t| match *t {
                JamTarget::Site(s) => s as usize == i,
                JamTarget::Sector(b) => s16(brg - b as i32).abs() <= SECTOR_HALF,
            })
        })
    }

    /// A site's detection score of the active pilot, as its jamming leaves it: while it is
    /// jammed and he is beyond its burn-through range, what it sees of him (the score with his
    /// EMV) is cut.
    pub(crate) fn jammed_score(&self, i: usize, score: i32) -> i32 {
        if score <= 0 || !self.site_jammed(i) {
            return score;
        }
        let o = self.objects[i];
        let d = approx(s16(self.pl.p.mx - o.x), s16(self.pl.p.my - o.y));
        if d <= BURN_THROUGH {
            return score;
        }
        let emv = self.pl.emv;
        (score + emv) * JAM_SCORE_PCT / 100 - emv
    }
}
