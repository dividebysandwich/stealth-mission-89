//! The bases' munitions between the campaign and the simulation: what the AI's aircraft take
//! off with (drawn from their base, the fallbacks when it is out) and bring back, the players'
//! stores, the friendly air defence's resupplies and the enemy's SAM and gun magazines.

use std::collections::BTreeMap;

use stealth_campaign::tuning::{CARGO_LAND_RADIUS, MAGAZINE_REFILL_EVERY, MAGAZINE_SUPPLY_RANGE};
use stealth_campaign::{Munition, ObjectId, PlayerId, Side, munition_spec as spec};
use stealth_i18n::{Text, text};
use stealth_sim::Sim;
use stealth_sim::aaa::GroundKind;
use stealth_sim::ai_loadout::{Seeker, Store};
use stealth_sim::campaign::Role;
use stealth_sim::oflags;

use crate::host::Host;

/// The limited munitions a player's stations (weapon index, rounds) take.
pub fn store_needs(stations: &[(i32, i32)]) -> BTreeMap<Munition, u32> {
    let mut out = BTreeMap::new();
    for &(w, n) in stations {
        if let Some(m) = Munition::of_weapon(w).filter(|_| n > 0) {
            *out.entry(m).or_insert(0) += n as u32;
        }
    }
    out
}

impl Host {
    /// Why a base cannot load these stores (None: it can).
    pub fn stores_refusal(&self, base: ObjectId, stations: &[(i32, i32)]) -> Option<Text> {
        store_needs(stations)
            .into_iter()
            .find(|&(m, n)| self.state.available(base, m) < n)
            .map(|(m, _)| text!("ce-out-of-stock", weapon = m.name()))
    }

    /// The stores out of a base's stock: as many of each as it has (the rest of a station
    /// stays empty).
    pub(crate) fn draw_stores(&mut self, base: ObjectId, stations: &mut [(i32, i32)]) {
        for st in stations.iter_mut() {
            let Some(m) = Munition::of_weapon(st.0).filter(|_| st.1 > 0) else { continue };
            st.1 = self.state.draw(base, m, st.1 as u32) as i32;
        }
    }

    /// Stores back into a base's stock.
    pub(crate) fn return_stores(&mut self, base: ObjectId, stations: &[(i32, i32)]) {
        for (m, n) in store_needs(stations) {
            self.state.put_back(base, m, n);
        }
    }

    /// A player landed at a base rearms with `stations`: the difference to what he has left
    /// comes out of (or goes back into) its stock. Err: the base does not have it.
    pub(crate) fn rearm_stock(
        &mut self,
        sim: &Sim,
        player: PlayerId,
        stations: &[(i32, i32)],
    ) -> Result<(), Text> {
        let Some(base) = self.landed_at(player) else { return Ok(()) };
        let have = store_needs(&sim.pilot_bays(player.0));
        let want = store_needs(stations);
        for (&m, &n) in &want {
            let extra = n.saturating_sub(have.get(&m).copied().unwrap_or(0));
            if self.state.available(base, m) < extra {
                return Err(text!("ce-out-of-stock", weapon = m.name()));
            }
        }
        for (&m, &n) in &want {
            let h = have.get(&m).copied().unwrap_or(0);
            self.state.draw(base, m, n.saturating_sub(h));
        }
        for (&m, &h) in &have {
            let n = want.get(&m).copied().unwrap_or(0);
            self.state.put_back(base, m, h.saturating_sub(n));
        }
        Ok(())
    }

    /// AI aircraft `i` of `side` taking off from `base`: its missiles and ground weapons come
    /// out of the base's stock; what is not there is replaced by the fallbacks, in the end by
    /// the side's Sidewinders (AA-8s) and iron bombs.
    pub(crate) fn arm_from_stock(&mut self, sim: &mut Sim, i: usize, side: Side, base: ObjectId) {
        if !self.state.munitions_on() {
            return;
        }
        let Some(f) = sim.campaign_flight(i) else { return };
        let mut loadout = f.loadout;
        for s in loadout.stores.iter_mut().filter(|s| s.count > 0) {
            let Some(m) = Munition::of_spec(s.missile, side) else { continue };
            let want = s.count as u32;
            let mut next = Some(m);
            let mut got = None;
            while let Some(k) = next {
                let n = self.state.draw(base, k, want);
                if n > 0 {
                    got = Some((k, n));
                    break;
                }
                next = k.fallback().filter(|f| f.spec().is_some());
            }
            *s = match got {
                Some((k, n)) => {
                    Store { missile: k.spec().unwrap_or(s.missile), count: n as i32, ..*s }
                }
                // Out of them all: the side's short range missiles, which never run out.
                None => Store {
                    missile: if side == Side::Blue { spec::AIM9 } else { spec::AA8 },
                    count: s.count,
                    seeker: Seeker::Ir,
                },
            };
        }
        let mut iron = false;
        if f.role == Role::Strike && !f.kamikaze {
            let n = if f.strike_unit { f.passes.max(1) as u32 } else { 1 };
            let m = ground_munition(sim, i, &f, side);
            let got = self.state.draw_with_fallback(base, m, n);
            iron = got.is_empty();
            self.ground_loads.retain(|g| g.0 != f.tag);
            if let Some(&(m, _)) = got.first() {
                self.ground_loads.push((f.tag, m));
            }
        }
        sim.arm_flight(i, loadout, iron);
    }

    /// What the AI's aircraft that landed brought back goes into the stock of the base they
    /// landed at.
    pub(crate) fn return_landed(&mut self, sim: &mut Sim) {
        for l in sim.take_landed() {
            let side = if l.blue { Side::Blue } else { Side::Red };
            let Some(base) = self.state.airfield_near(side, l.x, l.y, CARGO_LAND_RADIUS) else {
                continue;
            };
            for s in l.loadout.stores.iter().filter(|s| s.count > 0) {
                if let Some(m) = Munition::of_spec(s.missile, side) {
                    self.state.put_back(base, m, s.count as u32);
                }
            }
            // Its ground weapons not fired, as drawn.
            let drawn = self.ground_loads.iter().position(|g| g.0 == l.tag);
            let drawn = drawn.map(|k| self.ground_loads.swap_remove(k).1);
            if l.unfired > 0
                && !l.iron
                && let Some(m) = drawn
            {
                self.state.put_back(base, m, l.unfired as u32);
            }
        }
    }

    /// The munition a friendly air defence unit's stockpile is of (None: drones, which do not
    /// come from the bases).
    pub(crate) fn defence_munition(sim: &Sim, aaa: usize) -> Option<Munition> {
        match sim.units().get(aaa)?.kind {
            GroundKind::PatriotRadar => Some(Munition::Pac3),
            GroundKind::Gepard => Some(Munition::GepardRounds),
            _ => None,
        }
    }

    /// A resupply delivered to AAA unit `aaa`: what its base has of what it lacks.
    pub(crate) fn restock_from_base(&mut self, sim: &mut Sim, aaa: usize) {
        let Some(m) = Self::defence_munition(sim, aaa).filter(|_| self.state.munitions_on()) else {
            sim.restock(aaa);
            return;
        };
        let u = sim.units()[aaa];
        let missing = sim.stock_missing(aaa) as u32;
        let n = match self.state.supply_base(u.x, u.y) {
            Some(b) => self.state.draw(b, m, missing),
            None => missing,
        };
        sim.restock_with(aaa, n as i32);
    }

    /// A friendly air defence unit may be resupplied: its base has some of what it needs.
    pub(crate) fn can_resupply(&self, sim: &Sim, aaa: usize) -> bool {
        let Some(m) = Self::defence_munition(sim, aaa) else { return true };
        let u = sim.units()[aaa];
        self.state.supply_base(u.x, u.y).is_none_or(|b| self.state.available(b, m) > 0)
    }

    /// Now and then the enemy's SAM sites and guns refill their magazines from the nearest of
    /// its bases.
    pub(crate) fn refill_magazines(&mut self, sim: &mut Sim, dt: f64) {
        if !self.state.munitions_on() {
            return;
        }
        self.magazine_timer += dt;
        if self.magazine_timer < MAGAZINE_REFILL_EVERY {
            return;
        }
        self.magazine_timer = 0.0;
        self.count_site_demand(sim);
        // Strike aircraft shot down took their weapons with them.
        self.ground_loads.retain(|g| sim.campaign_unit(g.0).is_some());
        let bases = self.state.bases(Side::Red);
        for need in sim.magazine_needs() {
            let m = if need.site { Munition::sam(need.missile) } else { Munition::AaaRounds };
            // The bases in reach with some, nearest first (the nearest one at least).
            let mut near: Vec<(i32, ObjectId)> = bases
                .iter()
                .filter_map(|&b| {
                    let s = self.state.site(b)?;
                    Some((stealth_campaign::dist(s.x - need.x, s.y - need.y), b))
                })
                .collect();
            near.sort_unstable();
            let mut missing = need.missing as u32;
            for (k, &(d, b)) in near.iter().enumerate() {
                if missing == 0 || (k > 0 && d > MAGAZINE_SUPPLY_RANGE) {
                    break;
                }
                let n = self.state.draw(b, m, missing);
                if n > 0 {
                    sim.reload_magazine(need.site, need.index, n as i32);
                    missing -= n;
                }
            }
        }
    }

    /// Counts the enemy's SAM sites (by class) and guns each of its bases supplies (the
    /// nearest): what their stocks are sized for.
    pub(crate) fn count_site_demand(&mut self, sim: &Sim) {
        let mut demand: BTreeMap<ObjectId, [u32; 4]> = BTreeMap::new();
        for b in self.state.bases(Side::Red) {
            demand.insert(b, [0; 4]);
        }
        for u in sim.magazines() {
            let m = if u.site { Munition::sam(u.missile) } else { Munition::AaaRounds };
            let (Some(k), Some(b)) = (m.site_fed(), self.state.nearest_base(Side::Red, u.x, u.y))
            else {
                continue;
            };
            demand.entry(b).or_default()[k] += 1;
        }
        if !demand.is_empty() && demand != self.state.site_demand {
            self.state.site_demand = demand;
        }
    }
}

/// What AI strike aircraft `i` (`f`) of a side goes in with: an anti-radiation missile at a
/// radar; a helicopter its Hellfires; at a ship Harpoons; the A-10's CAS its CBU-105s, the
/// others' Mavericks; at anything else GBU-12s (the enemy: its AS-11s, else AS-10s).
fn ground_munition(
    sim: &Sim,
    i: usize,
    f: &stealth_sim::campaign::CampaignFlight,
    side: Side,
) -> Munition {
    use stealth_sim::ai_missions::{A10, AH1Z, OH58, RAH66};
    if harm(sim, f) {
        return if side == Side::Blue { Munition::Harm } else { Munition::As11 };
    }
    if side != Side::Blue {
        return Munition::As10;
    }
    let spec = sim.flights().get(i).map_or(0, |u| u.spec);
    let ship = !f.strike_unit
        && f.strike
            .and_then(|t| sim.objects().get(t as usize))
            .is_some_and(|o| o.flags & oflags::SHIP != 0);
    match spec {
        AH1Z | RAH66 | OH58 => Munition::Hellfire,
        _ if ship => Munition::Harpoon,
        A10 if f.strike_unit => Munition::Cbu105,
        _ if f.strike_unit => Munition::Agm65,
        _ => Munition::Gbu12,
    }
}

/// Strike aircraft `f` fires an anti-radiation missile (at a SAM radar), not a missile at an
/// object.
fn harm(sim: &Sim, f: &stealth_sim::campaign::CampaignFlight) -> bool {
    f.strike_aaa
        || (!f.strike_unit
            && f.strike
                .and_then(|t| sim.objects().get(t as usize))
                .is_some_and(|o| o.unit_type != 0 && o.flags & oflags::AIRFIELD == 0))
}
