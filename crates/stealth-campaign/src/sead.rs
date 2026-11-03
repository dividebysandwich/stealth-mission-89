//! The enemy's SEAD: a friendly SAM battery that shoots down one of its aircraft draws a raid
//! of strike aircraft that fire anti-radiation missiles at its radar.

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, WorldView};
use stealth_i18n::text;

impl CampaignState {
    /// An aircraft of `side` was shot down by the SAM battery of radar `aaa`: the enemy sends
    /// SEAD against a friendly one (paced by its skill, one raid per battery).
    pub(crate) fn sam_kill(
        &mut self,
        aaa: u32,
        side: Side,
        view: &WorldView,
        ev: &mut Vec<CampaignEvent>,
    ) {
        let skill = (self.setup.opponents as usize).min(3);
        let now = self.clock;
        if side != Side::Red
            || self.outcome.is_some()
            || now - self.sead_at < self.paced(SEAD_EVERY[skill])
        {
            return;
        }
        let Some(u) = view.units.get(aaa as usize).filter(|u| u.alive && u.side == Side::Blue)
        else {
            return;
        };
        let (x, y) = (u.x, u.y);
        if self.raids.iter().any(|r| r.open() && r.sam.is_some_and(|s| s.aaa == aaa)) {
            return;
        }
        let Some(from) = self
            .nearest_base(Side::Red, x, y)
            .filter(|&b| self.site(b).is_some_and(|b| dist(b.x - x, b.y - y) <= RAID_RANGE))
        else {
            return;
        };
        let Some(place) = self
            .places
            .iter()
            .enumerate()
            .filter(|(_, p)| p.owner == Side::Blue)
            .min_by_key(|(_, p)| dist(p.x - x, p.y - y))
            .map(|(k, _)| k as PlaceId)
        else {
            return;
        };
        let red = self.forces(Side::Red);
        if (red.fighters as usize) < RAID_AIRCRAFT + RAID_ESCORTS
            || !self.pay(Side::Red, RAID_COST, Spend::Operations)
        {
            return;
        }
        self.sead_at = now;
        let id = self.id();
        self.raids.push(Raid {
            id,
            due: Vec::new(),
            side: Side::Red,
            place,
            target: 0,
            from,
            aircraft: Vec::new(),
            alive: Vec::new(),
            state: RaidState::Pending { launch_at: now + RAID_LEAD },
            ended: None,
            escorted: false,
            drones: false,
            enroute_at: None,
            sam: Some(SamTarget { aaa, x, y, site: None }),
            feint: false,
        });
        let name = self.places[place as usize].name.clone();
        let mins = (RAID_LEAD / 60.0).round() as i64;
        let text = text!("cl-sead-intel", place = name, mins = mins);
        self.note(text.clone());
        self.board_at = f64::MIN;
        ev.push(CampaignEvent::Message(text));
    }

    /// Air defence unit `aaa` was destroyed: the SEAD raid on it (if one is under way) hit,
    /// unless a player destroyed it (`by_ai` false): then it has nothing left to go for.
    pub(crate) fn sead_hit(&mut self, aaa: u32, by_ai: bool, ev: &mut Vec<CampaignEvent>) {
        let Some(k) = self
            .raids
            .iter()
            .position(|r| r.state == RaidState::Enroute && r.sam.is_some_and(|s| s.aaa == aaa))
        else {
            return;
        };
        if !by_ai {
            let r = &self.raids[k];
            let tags: Vec<u32> =
                r.aircraft.iter().zip(&r.alive).filter(|(_, a)| **a).map(|(&t, _)| t).collect();
            ev.extend(tags.into_iter().map(|tag| CampaignEvent::Recall { tag }));
            self.end_raid(k, RaidState::CalledOff);
            return;
        }
        let name = self.places[self.raids[k].place as usize].name.clone();
        let site = self.raids[k].sam.and_then(|s| s.site).and_then(|d| self.deployment(d));
        let text = match site {
            Some(d) => text!("cl-infra-hit", site = d.label()),
            None => text!("cl-sead-hit", place = name),
        };
        self.end_raid(k, RaidState::Hit);
        self.note(text.clone());
        ev.push(CampaignEvent::Message(text!("cl-exclaim", text = text)));
    }

    /// Where a raid goes: its SAM battery, else its target object.
    pub(crate) fn raid_point(&self, r: &Raid) -> Option<(i32, i32)> {
        match r.sam {
            Some(s) => Some((s.x, s.y)),
            None => self.site(r.target).map(|s| (s.x, s.y)),
        }
    }
}
