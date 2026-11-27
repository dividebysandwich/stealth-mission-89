//! A single mission's SEAD package (`MissionSettings::sead`): friendly F-16s take off from the
//! player's base ahead of him, one after another, each at one of the enemy's SAM sites along
//! his route (the first on the way first). Each fires an anti-radiation missile at its site
//! from the missile's reach and flies home.
//!
//! They are the campaign's strike aircraft (a SEAD flight's HARM shot, `Sim::campaign_bombs`),
//! hosted by its bookkeeping without making the mission a campaign, as a fighter escort is
//! ([`Sim::launch_escorts`]).

use serde::{Deserialize, Serialize};
use stealth_mission::Mission;

use crate::Sim;
use crate::types::*;

/// Campaign tag of the package's aircraft: this plus 1, 2...
pub const SEAD_TAG: u32 = 0xE5C2_0000;
/// Most aircraft in the package (one per SAM site).
pub const SEAD_MAX: usize = 4;
/// Seconds between the package's take-offs.
const GAP_S: i32 = 12;
/// A site this far (map units, about 10 km) outside its missile's reach of the route is still
/// on the way.
const MARGIN: f64 = 640.0;
/// The F-16 (AI aircraft type), the SEAD aircraft.
const F16: i32 = 22;

/// The package still to take off.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SeadPackage {
    /// SAM sites (objects) still to send an aircraft at, the next first.
    pub queue: Vec<usize>,
    /// Base object they take off from.
    pub base: usize,
    /// Frames until the next takes off.
    pub wait: i32,
    /// Aircraft launched so far.
    pub launched: u32,
}

/// How far point `p` is from the route `r` (map units), and where along it the nearest point
/// is (leg, part of it).
fn along(r: &[(f64, f64)], p: (f64, f64)) -> (f64, (usize, f64)) {
    let mut best = (f64::MAX, (0, 0.0));
    for (k, w) in r.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 {
            (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let d = (p.0 - a.0 - t * dx).hypot(p.1 - a.1 - t * dy);
        if d < best.0 {
            best = (d, (k, t));
        }
    }
    best
}

impl Sim {
    /// The SAM sites (objects) along the player's route from his base over his targets to his
    /// landing base, in the order he comes to them: those whose missiles reach within `MARGIN`
    /// of it, none of them his own targets.
    pub fn sead_targets(&self, m: &Mission) -> Vec<usize> {
        let p = |x: i32, y: i32| (x as f64, y as f64);
        let route = [
            p(m.takeoff.position.x, m.takeoff.position.y),
            p(m.waypoints[1].x, m.waypoints[1].y),
            p(m.waypoints[2].x, m.waypoints[2].y),
            p(m.waypoints[3].x, m.waypoints[3].y),
        ];
        let own: Vec<i32> = self.pl.targets.iter().map(|t| t.object).collect();
        let mut sites: Vec<(usize, (usize, f64))> = (0..self.ground_units.min(self.objects.len()))
            .filter_map(|i| {
                let o = self.objects[i];
                let sam = self.threat_type(o.unit_type).is_some_and(|t| t.missile.is_some());
                if !sam
                    || o.flags & (oflags::FRIENDLY | oflags::NEUTRAL | oflags::DESTROYED) != 0
                    || own.contains(&(i as i32))
                {
                    return None;
                }
                let reach = (self.site_lock_range(o.unit_type) << 6) as f64;
                let (d, at) = along(&route, p(o.x, o.y));
                (d <= reach + MARGIN).then_some((i, at))
            })
            .collect();
        sites.sort_by(|a, b| a.1.0.cmp(&b.1.0).then(a.1.1.total_cmp(&b.1.1)));
        sites.into_iter().map(|s| s.0).take(SEAD_MAX).collect()
    }

    /// The package for mission `m`: it takes off from the player's base, the first at once.
    /// None when no SAM site is on his way.
    pub(crate) fn plan_sead(&mut self, m: &Mission, stores: [i32; 4], gun: i32) {
        if self.opts.campaign {
            return;
        }
        let queue = self.sead_targets(m);
        let base = self.pl.targets[0].base;
        let Ok(base) = usize::try_from(base) else { return };
        if queue.is_empty() || base >= self.objects.len() {
            return;
        }
        if self.campaign.is_none() {
            self.campaign = Some(Box::new(self.campaign_state(stores, gun)));
        }
        self.sead = Some(SeadPackage { queue, base, wait: 0, launched: 0 });
    }

    /// Every frame: the next aircraft of the package takes off when its time comes.
    pub(crate) fn update_sead(&mut self) {
        let fs = self.fs.max(1);
        let Some(p) = self.sead.as_mut() else { return };
        if p.wait > 0 {
            p.wait -= 1;
            return;
        }
        if p.queue.is_empty() {
            self.sead = None;
            return;
        }
        let target = p.queue.remove(0);
        p.wait = GAP_S * fs;
        p.launched += 1;
        let (base, tag) = (p.base, SEAD_TAG + p.launched);
        // Taken out already (by the player): the next one.
        if self.objects.get(target).is_none_or(|o| o.flags & oflags::DESTROYED != 0) {
            if let Some(p) = self.sead.as_mut() {
                p.wait = 0;
            }
            return;
        }
        let i = self.spawn_strike(base, target, tag, true, F16, false);
        // Fuel for the whole mission (tankers on the way): the longest the timer runs.
        self.flights[i].timer = 0x7FFF;
    }

    /// The SEAD package's aircraft still to take off.
    pub fn sead_waiting(&self) -> usize {
        self.sead.as_ref().map_or(0, |p| p.queue.len())
    }
}
