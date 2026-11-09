//! Campaign munitions in the simulation: the cargo aircraft flying munitions from a side's
//! production base to another base, the fighters hunting the other side's, the AI aircraft's
//! loads as the host drew them from a base's stock (and what they bring back unused), and the
//! enemy's SAM sites' and guns' magazines (refilled by the host from the nearest base).

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::GroundKind;
use crate::ai_loadout::{AiLoadout, FighterTask};
use crate::campaign::{CampaignFlight, Role};
use crate::math::approx;
use crate::types::*;

/// The cargo aircraft: the friendly side's C-5, the enemy's An-72.
pub const C5: i32 = 27;
pub const AN72: i32 = 9;
/// A cargo aircraft this close to its base on the way in (map units, ~4 km) has landed (too
/// heavy for the AI's tight circuit, as the AWACS).
const TRANSPORT_HOME: i32 = 0x100;
/// How far to either side of a player they escort the escorts keep station (map units, 2 km).
const ESCORT_SIDE: i32 = 0x80;
/// Missiles ready at an enemy SAM site, and rounds at a gun.
pub const SITE_MAGAZINE: i32 = 4;
pub const GUN_MAGAZINE: i32 = 1200;
/// Rounds a ZSU-23-4's tracer stands for (four barrels), an S-60's shot.
pub const ZSU_ROUNDS: i32 = 8;
pub const S60_ROUNDS: i32 = 4;

/// A long range SAM's missile (SA-2, SA-5, SA-10, SA-12, SA-N-6, SA-21): with magazines, a
/// site holds its fire at an aircraft another long range missile is already on its way to.
pub fn long_range_sam(spec: i32) -> bool {
    matches!(spec, 1 | 2 | 4 | 6 | 10 | 41)
}

/// What a campaign aircraft brought back when it landed: its missiles left, and its ground
/// weapons not fired.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LandedLoad {
    pub tag: u32,
    pub blue: bool,
    pub x: i32,
    pub y: i32,
    pub loadout: AiLoadout,
    /// Ground weapons not fired, and whether they were anti-radiation missiles (or iron
    /// bombs, which never run out).
    pub unfired: u8,
    pub harm: bool,
    pub iron: bool,
}

/// An enemy SAM site's or gun's magazine that is not full: the object (a site) or AAA unit
/// (a gun), where it is, the site's missile spec, how many are missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagazineNeed {
    pub site: bool,
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub missile: i32,
    pub missing: i32,
}

impl Sim {
    /// Campaign: a side's cargo aircraft takes off from base `from` and flies to base `to`,
    /// where it lands. Returns its flight unit.
    pub fn spawn_transport(&mut self, from: usize, to: usize, tag: u32, blue: bool) -> usize {
        let i = self.campaign_slot();
        let spec = if blue { C5 } else { AN72 };
        self.flights[i] = FlightUnit { spec, ..Default::default() };
        self.spawn(i, from);
        let dest = self.objects.get(to).map_or((0, 0), |o| (o.x, o.y));
        let u = &mut self.flights[i];
        u.flags |= fflags::RTB;
        u.wp = to as i32;
        let c = self.campaign.as_mut().expect("campaign");
        c.flights[i] = Some(CampaignFlight {
            tag,
            role: Role::Transport,
            blue,
            goal: dest,
            target: None,
            cooldown: 0,
            loadout: AiLoadout::default(),
            task: FighterTask::Patrol,
            home: to as i32,
            anchor: dest,
            quiet: 0,
            burst: 0,
            killed_by_ai: false,
            killed_by: None,
            killed_by_radar: None,
            reported: false,
            strike: None,
            kamikaze: false,
            strike_aaa: false,
            supply: None,
            strike_unit: false,
            defending: false,
            fought: 0,
            passes: 0,
            pass_wait: 0,
            hunt: None,
            iron: false,
        });
        i
    }

    /// The flight unit of campaign aircraft `tag` (on the runway too).
    pub fn campaign_unit(&self, tag: u32) -> Option<usize> {
        let c = self.campaign.as_ref()?;
        c.flights.iter().position(|f| f.is_some_and(|f| f.tag == tag))
    }

    /// Campaign: the fighter `tag` hunts the other side's aircraft `quarry` (a cargo flight):
    /// it waits over its goal until that one is up, then follows it.
    pub fn set_hunt(&mut self, tag: u32, quarry: u32) {
        let Some(i) = self.campaign_unit(tag) else { return };
        if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
            f.hunt = Some(quarry);
        }
    }

    /// Campaign: pilot `id` flies the cargo run with campaign tag `tag` (None: none): the
    /// fighters hunting or escorting that tag follow him once he is up.
    pub fn set_cargo_tag(&mut self, id: u8, tag: Option<u32>) {
        if id == self.pl.id {
            self.pl.cargo_tag = tag;
        } else if let Some(p) = self.parked.get_mut(id as usize) {
            p.cargo_tag = tag;
        }
    }

    /// Campaign: the load campaign aircraft `i` takes off with (what its base had), and
    /// whether it bombs with iron bombs (out of guided weapons).
    pub fn arm_flight(&mut self, i: usize, loadout: AiLoadout, iron: bool) {
        let Some(Some(f)) = self.campaign.as_mut().and_then(|c| c.flights.get_mut(i)) else {
            return;
        };
        f.loadout = loadout;
        f.iron = iron;
        if f.role == Role::Fighter {
            self.flights[i].missile = loadout.stores[0].missile;
        }
    }

    /// What the campaign aircraft that landed since the last call brought back.
    pub fn take_landed(&mut self) -> Vec<LandedLoad> {
        self.campaign.as_mut().map(|c| std::mem::take(&mut c.landed)).unwrap_or_default()
    }

    /// Hunters follow their quarry once it is up (escorts beside it, `ESCORT_SIDE` map units); landing cargo aircraft are down at their
    /// base.
    pub(crate) fn campaign_hunts(&mut self) {
        let Some(c) = self.campaign.as_ref() else { return };
        let mut goals = Vec::new();
        // A player flying a cargo run: hunted and escorted as the AI's cargo aircraft is.
        let flown: Vec<(u32, (i32, i32))> = self
            .present()
            .into_iter()
            .map(|k| self.pilot_ref(k))
            .filter(|p| !p.on_ground)
            .filter_map(|p| p.cargo_tag.map(|t| (t, (p.p.mx, p.p.my))))
            .collect();
        for (i, f) in c.flights.iter().enumerate() {
            let Some(f) = f else { continue };
            let Some(q) = f.hunt else { continue };
            if let Some(j) = c.flights.iter().position(|g| g.is_some_and(|g| g.tag == q)) {
                let t = self.flights[j];
                if t.flags & fflags::AIRBORNE != 0 && t.flags & fflags::DESTROYED == 0 {
                    goals.push((i, (t.x, t.y)));
                }
            } else if let Some(&(_, at)) = flown.iter().find(|(t, _)| *t == q) {
                // Escorts each keep their own side of him (not all on one spot).
                let side = if f.blue { ESCORT_SIDE * if i % 2 == 0 { 1 } else { -1 } } else { 0 };
                goals.push((i, (at.0 + side, at.1)));
            }
        }
        let c = self.campaign.as_mut().expect("campaign");
        for (i, g) in goals {
            if let Some(f) = c.flights[i].as_mut() {
                f.goal = g;
            }
        }
        for (i, slot) in c.flights.iter().enumerate() {
            let Some(f) = slot.filter(|f| f.role == Role::Transport) else { continue };
            let u = &mut self.flights[i];
            if u.flags & fflags::DESTROYED == 0
                && u.flags & fflags::TAKEOFF == 0
                && approx(u.x - f.anchor.0, u.y - f.anchor.1) < TRANSPORT_HOME
            {
                u.flags &= !fflags::AIRBORNE;
            }
        }
    }

    /// Notes what a campaign aircraft landing now brings back.
    pub(crate) fn note_landed(&mut self, f: &CampaignFlight, x: i32, y: i32) {
        let unfired = match f.role {
            Role::Strike if f.strike.is_some() && !f.kamikaze => {
                if f.strike_unit {
                    f.passes.max(1)
                } else {
                    1
                }
            }
            _ => 0,
        };
        let empty = f.loadout.missiles() == 0;
        if empty && unfired == 0 {
            return;
        }
        if let Some(c) = self.campaign.as_mut() {
            c.landed.push(LandedLoad {
                tag: f.tag,
                blue: f.blue,
                x,
                y,
                loadout: f.loadout,
                unfired,
                harm: f.strike_aaa,
                iron: f.iron,
            });
        }
    }

    /// Campaign: the enemy's SAM sites and guns fire what their magazines hold from now on
    /// (the host refills them from its bases' stocks).
    pub fn enable_magazines(&mut self) {
        if let Some(c) = self.campaign.as_mut() {
            c.magazines = true;
        }
        self.magazine_sizes();
    }

    /// Every object and unit has a magazine (new ones start full; a ship's never runs out, no
    /// base refills it).
    pub(crate) fn magazine_sizes(&mut self) {
        let (no, nu) = (self.objects.len(), self.units.len());
        let ships: Vec<bool> = self.objects.iter().map(|o| o.flags & oflags::SHIP != 0).collect();
        let Some(c) = self.campaign.as_mut().filter(|c| c.magazines) else { return };
        while c.site_rounds.len() < no {
            let ship = ships[c.site_rounds.len()];
            c.site_rounds.push(if ship { i32::MAX / 2 } else { SITE_MAGAZINE });
        }
        if c.gun_rounds.len() < nu {
            c.gun_rounds.resize(nu, GUN_MAGAZINE);
        }
    }

    /// Enemy SAM site `i` has a missile ready (always without magazines).
    pub(crate) fn site_ready(&self, i: usize) -> bool {
        self.campaign
            .as_ref()
            .filter(|c| c.magazines)
            .is_none_or(|c| c.site_rounds.get(i).is_none_or(|&n| n > 0))
    }

    /// Salvo discipline (with magazines): a long range site's missile `spec` holds its fire at
    /// the active pilot while another site's long range missile is on its way to him.
    pub(crate) fn salvo_held(&self, spec: i32, site: usize) -> bool {
        self.campaign.as_ref().is_some_and(|c| c.magazines)
            && long_range_sam(spec)
            && (0..crate::ENEMY_SLOTS).any(|k| {
                let p = &self.proj[k];
                p.ttl > 0
                    && self.proj_pilot[k] == self.pl.id
                    && p.launcher >= 0
                    && p.launcher as usize != site
                    && long_range_sam(p.spec)
            })
    }

    /// Enemy SAM site `i` fired a missile.
    pub(crate) fn site_fired(&mut self, i: usize) {
        if let Some(c) = self.campaign.as_mut().filter(|c| c.magazines)
            && let Some(n) = c.site_rounds.get_mut(i)
        {
            *n = (*n - 1).max(0);
        }
    }

    /// Enemy gun `k` has rounds (always without magazines).
    pub(crate) fn gun_ready(&self, k: usize) -> bool {
        self.campaign
            .as_ref()
            .filter(|c| c.magazines)
            .is_none_or(|c| c.gun_rounds.get(k).is_none_or(|&n| n > 0))
    }

    /// Enemy gun `k` fired `n` rounds.
    pub(crate) fn gun_fired(&mut self, k: usize, n: i32) {
        if let Some(c) = self.campaign.as_mut().filter(|c| c.magazines)
            && let Some(r) = c.gun_rounds.get_mut(k)
        {
            *r = (*r - n).max(0);
        }
    }

    /// The enemy SAM sites and guns whose magazines are not full.
    pub fn magazine_needs(&self) -> Vec<MagazineNeed> {
        self.magazines().into_iter().filter(|m| m.missing > 0).collect()
    }

    /// Every enemy SAM site and gun with a magazine (`missing` 0 when it is full).
    pub fn magazines(&self) -> Vec<MagazineNeed> {
        let Some(c) = self.campaign.as_ref().filter(|c| c.magazines) else { return Vec::new() };
        let mut out = Vec::new();
        for (i, &n) in c.site_rounds.iter().enumerate().take(self.ground_units) {
            let o = self.objects[i];
            if o.unit_type == 0
                || o.flags & (oflags::DESTROYED | oflags::FRIENDLY | oflags::NEUTRAL | oflags::SHIP)
                    != 0
            {
                continue;
            }
            out.push(MagazineNeed {
                site: true,
                index: i,
                x: o.x,
                y: o.y,
                missile: self.site_missile(o.unit_type),
                missing: (SITE_MAGAZINE - n).max(0),
            });
        }
        for (k, &n) in c.gun_rounds.iter().enumerate() {
            let Some(u) = self.units.get(k) else { continue };
            if u.destroyed
                || c.aaa_blue.get(k).copied().unwrap_or(false)
                || !matches!(u.kind, GroundKind::Zsu | GroundKind::S60)
            {
                continue;
            }
            out.push(MagazineNeed {
                site: false,
                index: k,
                x: u.x,
                y: u.y,
                missile: 0,
                missing: (GUN_MAGAZINE - n).max(0),
            });
        }
        out
    }

    /// `n` missiles (a site) or rounds (a gun) into a magazine.
    pub fn reload_magazine(&mut self, site: bool, index: usize, n: i32) {
        let Some(c) = self.campaign.as_mut().filter(|c| c.magazines) else { return };
        let (v, full) = if site {
            (&mut c.site_rounds, SITE_MAGAZINE)
        } else {
            (&mut c.gun_rounds, GUN_MAGAZINE)
        };
        if let Some(r) = v.get_mut(index) {
            *r = (*r + n).min(full);
        }
    }

    /// The enemy SAM sites' magazines (object, missiles) to keep over a save.
    pub fn site_magazines(&self) -> Vec<(u16, i32)> {
        let Some(c) = self.campaign.as_ref().filter(|c| c.magazines) else { return Vec::new() };
        c.site_rounds
            .iter()
            .enumerate()
            .take(self.ground_units)
            .filter(|&(_, &n)| n < SITE_MAGAZINE)
            .map(|(i, &n)| (i as u16, n))
            .collect()
    }

    /// Saved SAM site magazines back.
    pub fn restore_site_magazines(&mut self, saved: &[(u16, i32)]) {
        self.magazine_sizes();
        let Some(c) = self.campaign.as_mut().filter(|c| c.magazines) else { return };
        for &(i, n) in saved {
            if let Some(r) = c.site_rounds.get_mut(i as usize) {
                *r = n.clamp(0, SITE_MAGAZINE);
            }
        }
    }
}
