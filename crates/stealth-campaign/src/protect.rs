//! Cover for the friendly side's supply and cargo helicopters: gunships go ahead of one whose
//! way passes enemy guns, fighters fly with one where enemy fighters are about, and one that
//! would fly into a SAM site's reach (or past guns with no gunships to send) waits a while.

use serde::{Deserialize, Serialize};

use crate::ai_missions::Short;
use crate::munitions::segment_dist;
use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, FighterTask, WorldView};
use stealth_i18n::text;

/// Gunships sent ahead of a helicopter run (a resupply request, or a site's load).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeloCover {
    /// The request's or the site's id, and the load (0 for a request).
    pub key: (u32, u8),
    /// Their AI flight.
    pub mission: u32,
    pub at: f64,
}

/// Fighters flying with a helicopter (its tag, theirs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeloEscort {
    pub helo: u32,
    pub fighters: Vec<u32>,
}

impl CampaignState {
    /// The enemy's guns and MANPADS (unit indices) within `HELO_GUN_CLEAR` of the way from `a`
    /// to `b`.
    pub(crate) fn guns_on_way(&self, a: (i32, i32), b: (i32, i32), view: &WorldView) -> Vec<u32> {
        (0..view.units.len() as u32)
            .filter(|&k| {
                let u = &view.units[k as usize];
                u.alive
                    && u.side == Side::Red
                    && matches!(u.vehicle, None | Some(crate::ConvoyUnit::Escort))
                    && segment_dist(a, b, (u.x, u.y)) <= HELO_GUN_CLEAR
            })
            .collect()
    }

    /// A live enemy SAM site within `HELO_SAM_CLEAR` of the way from `a` to `b`.
    fn sam_on_way(&self, a: (i32, i32), b: (i32, i32)) -> bool {
        self.sites.iter().any(|s| {
            s.kind == SiteKind::Sam
                && s.alive()
                && self.owner(s.object) == Some(Side::Red)
                && segment_dist(a, b, (s.x, s.y)) <= HELO_SAM_CLEAR
        })
    }

    /// Whether a friendly helicopter run (`key`) from base `from` to `to`, due for
    /// `waited` seconds, goes now. Gunships go first where enemy guns stand along the way (the
    /// run follows `HELO_COVER_LEAD` after them, or once they are back); it waits by a SAM
    /// site, or for gunships when there are none to send, up to `HELO_HOLD_MAX`.
    pub(crate) fn helo_may_go(
        &mut self,
        key: (u32, u8),
        from: ObjectId,
        to: (i32, i32),
        waited: f64,
        view: &WorldView,
    ) -> bool {
        if !self.ai_missions_on() {
            return true;
        }
        let now = self.clock;
        if let Some(k) = self.helo_cover.iter().position(|c| c.key == key) {
            let c = self.helo_cover[k];
            let out = self.ai_missions.iter().any(|m| m.id == c.mission && m.open());
            if out && now - c.at < HELO_COVER_LEAD {
                return false;
            }
            self.helo_cover.remove(k);
            return true;
        }
        let Some(a) = self.site(from).map(|s| (s.x, s.y)) else { return true };
        if waited >= HELO_HOLD_MAX {
            return true;
        }
        if self.sam_on_way(a, to) {
            return false;
        }
        let guns = self.guns_on_way(a, to, view);
        if guns.is_empty() {
            return true;
        }
        if let Some(mission) = self.send_gunships(from, &guns, view) {
            self.helo_cover.push(HeloCover { key, mission, at: now });
        }
        false
    }

    /// A pair of the friendly side's attack aircraft (its helicopters close by) from base
    /// `from` against the enemy guns `guns`: their AI flight's id.
    fn send_gunships(&mut self, from: ObjectId, guns: &[u32], view: &WorldView) -> Option<u32> {
        let n = guns.len() as i32;
        let at = |k: u32| (view.units[k as usize].x, view.units[k as usize].y);
        let x = guns.iter().map(|&k| at(k).0).sum::<i32>() / n;
        let y = guns.iter().map(|&k| at(k).1).sum::<i32>() / n;
        let (place, _) = self
            .places
            .iter()
            .enumerate()
            .filter(|(_, p)| p.owner == Side::Red)
            .map(|(k, p)| (k as PlaceId, crate::state::dist(p.x - x, p.y - y)))
            .min_by_key(|p| p.1)?;
        let name = self.places[place as usize].name.clone();
        let o = MissionOffer {
            id: self.id(),
            kind: MissionKind::Cas,
            title: text!("cb-cas", place = name.as_str()),
            objective: Objective::Cas { place, aaa: guns.to_vec() },
            x,
            y,
            base: from,
            threat: n,
            posted: self.clock,
        };
        match self.launch_ai_from(Side::Blue, &o, view, from) {
            Ok(()) => self.ai_missions.last().map(|m| m.id),
            Err(Short::Aircraft | Short::Materiel | Short::Offer) => None,
        }
    }

    /// Friendly helicopter `tag` just took off from `from` for `to`: with enemy fighters about
    /// its way, a pair of fighters from that base flies with it (if it has them).
    pub(crate) fn escort_helo(
        &mut self,
        tag: u32,
        from: ObjectId,
        to: (i32, i32),
        view: &WorldView,
        ev: &mut Vec<CampaignEvent>,
    ) {
        if !self.ai_missions_on() || !self.bases(Side::Blue).contains(&from) {
            return;
        }
        let Some(a) = self.site(from).map(|s| (s.x, s.y)) else { return };
        let hostile = view
            .flights
            .iter()
            .any(|f| f.side == Side::Red && segment_dist(a, to, (f.x, f.y)) <= AIR_THREAT_RANGE);
        let n = HELO_ESCORTS as u32;
        if !hostile
            || self.forces(Side::Blue).fighters < n
            || self.forces(Side::Blue).materiel < n as f64 * SORTIE_COST
        {
            return;
        }
        self.pay(Side::Blue, n as f64 * SORTIE_COST, Spend::Operations);
        self.take_fighters(Side::Blue, n);
        let now = self.clock;
        let mut fighters = Vec::new();
        for _ in 0..n {
            let f = self.id();
            self.patrols.push((f, Side::Blue, now));
            fighters.push(f);
            ev.push(CampaignEvent::LaunchFighters {
                tag: f,
                side: Side::Blue,
                from,
                x: to.0,
                y: to.1,
                task: FighterTask::Patrol,
                escort: Some(tag),
            });
        }
        self.helo_escorts.push(HeloEscort { helo: tag, fighters });
    }

    /// Helicopter `tag`'s run is over: the fighters with it come home.
    pub(crate) fn helo_done(&mut self, tag: u32, ev: &mut Vec<CampaignEvent>) {
        let Some(k) = self.helo_escorts.iter().position(|e| e.helo == tag) else { return };
        let e = self.helo_escorts.remove(k);
        for f in e.fighters.into_iter().filter(|f| self.patrols.iter().any(|p| p.0 == *f)) {
            ev.push(CampaignEvent::Recall { tag: f });
        }
    }
}
