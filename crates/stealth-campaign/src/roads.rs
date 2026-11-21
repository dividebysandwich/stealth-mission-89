//! Supply by road (theaters with supply roads: Ukraine): the enemy's bases along the roads get
//! their munitions by truck instead of by cargo aircraft. A convoy of three trucks (and, half of
//! the time, a pickup with a machine gun) drives from where the roads come in to the base; each
//! truck carries a third of the load, and only the trucks that get there deliver. A bridge's
//! span brought down cuts its road for good: the convoys go round by the other roads.

use serde::{Deserialize, Serialize};
use stealth_i18n::text;
use stealth_mission::roads::RoadNet;

use crate::CampaignEvent;
use crate::convoy::ConvoyUnit;
use crate::munitions::{CargoRun, CargoState, Munition};
use crate::state::*;
use crate::tuning::*;

/// Metres per map unit.
const M_PER_MAP: f64 = 15.625;

/// A cargo run by road: its convoy's vehicles, head first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadRun {
    pub units: Vec<ConvoyUnit>,
    /// Trucks that turned back at a cut road (their loads went back to the production base).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub turned_back: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

impl RoadRun {
    pub fn trucks(&self) -> u8 {
        self.units.iter().filter(|u| u.truck()).count() as u8
    }
}

/// Metres (east, north) of a map point.
fn metres(x: i32, y: i32) -> [f64; 2] {
    [x as f64 * M_PER_MAP, (0x8000 - y) as f64 * M_PER_MAP]
}

/// `n`'s share for `part` of `whole` trucks.
fn share(n: u32, part: u8, whole: u8) -> u32 {
    if whole == 0 { 0 } else { n * part as u32 / whole as u32 }
}

impl CampaignState {
    /// The theater's supply roads (the host's campaign; none on a coop client).
    pub fn road_net(&self) -> Option<&RoadNet> {
        self.terrain.0.as_ref()?.roads.as_ref().filter(|r| !r.is_empty())
    }

    /// The road junction at a base's gate (its spur road's end).
    fn road_node(&self, base: ObjectId) -> Option<usize> {
        let net = self.road_net()?;
        let s = self.site(base)?;
        let [e, n] = metres(s.x, s.y);
        net.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.name.ends_with(" base"))
            .map(|(k, node)| (k, (node.x - e).hypot(node.y - n)))
            .filter(|&(_, d)| d < ROAD_BASE_REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k)
    }

    /// A base the enemy supplies by road.
    pub fn road_served(&self, base: ObjectId, side: Side) -> bool {
        side == Side::Red && self.road_node(base).is_some()
    }

    /// The way the trucks drive to `base` (metres east, north): from the nearest of the roads'
    /// entries, over the roads not cut.
    pub(crate) fn road_route(&self, base: ObjectId) -> Option<Vec<[f64; 2]>> {
        let net = self.road_net()?;
        let to = self.road_node(base)?;
        let open = |k: usize, _: &stealth_mission::roads::RoadEdge| !self.cut_roads.contains(&k);
        net.entries
            .iter()
            .filter_map(|&from| net.path(from, to, open))
            .map(|p| {
                let len: f64 = p.iter().map(|&(k, _)| net.edges[k].length()).sum();
                (p, len)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(p, _)| net.points(&p).into_iter().map(|(x, y, _)| [x, y]).collect())
    }

    /// A road-served base low on something gets another convoy (a few on the road to it at
    /// once, a while apart): up to `CARGO_SHARE` of a full stock of each munition, less what
    /// is on its way already.
    pub(crate) fn schedule_road(&mut self, base: ObjectId, side: Side, from: ObjectId) {
        let runs: Vec<&CargoRun> =
            self.cargo_runs.iter().filter(|r| r.to == base && r.road.is_some()).collect();
        if runs.len() >= SUPPLY_CONVOYS_MAX
            || runs.iter().any(|r| self.clock - r.posted < SUPPLY_SPACING)
        {
            return;
        }
        let coming = |m: Munition| -> u32 {
            runs.iter().flat_map(|r| &r.load).filter(|l| l.0 == m).map(|l| l.1).sum()
        };
        let Some(s) = self.stocks.get(&base) else { return };
        let have: Vec<(Munition, f64, f64)> = Munition::of_side(side)
            .iter()
            .map(|&m| (m, (s.count(m) + coming(m)) as f64, self.full_stock(base, m)))
            .collect();
        let Some(pool) = self.stocks.get(&from) else { return };
        // Worth a convoy: something below `SUPPLY_LOW` of a full stock, with enough of it made.
        let worth = have.iter().any(|&(m, n, full)| {
            full >= 1.0
                && n < full * SUPPLY_LOW
                && pool.count(m) as f64 >= (full * CARGO_SHARE / 2.0).max(1.0)
        });
        if !worth {
            return;
        }
        let pool = self.stocks.get_mut(&from).expect("pool");
        let mut load = Vec::new();
        for (m, n, full) in have {
            let want = ((full * CARGO_SHARE) as u32).min((full - n).max(0.0) as u32);
            let got = pool.take(m, want);
            if got > 0 {
                load.push((m, got));
            }
        }
        if load.is_empty() {
            return;
        }
        let (id, tag) = (self.id(), self.id());
        // A mix of civilian and army trucks, a pickup with a machine gun with half of them
        // (from the run's id: the campaign's random stream is left alone).
        let h = id.wrapping_mul(0x9E37_79B9);
        let truck = |k: u32| {
            if (h >> (8 + k)) & 1 == 0 { ConvoyUnit::CivTruck } else { ConvoyUnit::ArmyTruck }
        };
        let mut units = Vec::new();
        if (h >> 20) % 1000 < (TECHNICAL_CHANCE * 1000.0) as u32 {
            units.push(ConvoyUnit::Technical);
        }
        units.extend((0..SUPPLY_TRUCKS as u32).map(truck));
        let now = self.clock;
        self.cargo_runs.push(CargoRun {
            id,
            side,
            from,
            to: base,
            tag,
            load,
            state: CargoState::Scheduled { launch_at: now + SUPPLY_WAIT },
            posted: now,
            escorts: Vec::new(),
            road: Some(RoadRun { units, turned_back: 0 }),
        });
    }

    /// Road run `k` is due: its convoy sets off (false: no way there by road).
    pub(crate) fn launch_road(&mut self, k: usize, ev: &mut Vec<CampaignEvent>) -> bool {
        let r = &self.cargo_runs[k];
        let Some(route) = self.road_route(r.to) else { return false };
        let units = r.road.as_ref().map(|w| w.units.clone()).unwrap_or_default();
        let r = &mut self.cargo_runs[k];
        r.state = CargoState::Airborne;
        ev.push(CampaignEvent::SupplyConvoyLaunched { id: r.id, side: r.side, tag: r.tag, units, route });
        true
    }

    /// The simulation's news of the convoys and the roads.
    pub(crate) fn road_report(&mut self, r: &crate::Report, ev: &mut Vec<CampaignEvent>) {
        match *r {
            crate::Report::SupplyConvoyArrived { tag, trucks } => self.supply_arrived(tag, trucks, ev),
            crate::Report::SupplyTurnedBack { tag, trucks } => {
                let Some(run) = self.cargo_runs.iter_mut().find(|c| c.tag == tag && c.road.is_some())
                else {
                    return;
                };
                let road = run.road.as_mut().expect("road");
                let whole = road.trucks();
                let back = trucks.min(whole - road.turned_back.min(whole));
                road.turned_back += back;
                let (from, load) = (run.from, run.load.clone());
                for (m, n) in load {
                    self.put_in(from, m, share(n, back, whole));
                }
            }
            crate::Report::RoadCut { edge } => {
                if self.cut_roads.contains(&edge) {
                    return;
                }
                self.cut_roads.push(edge);
                let name = self
                    .road_net()
                    .and_then(|n| n.edges.get(edge))
                    .map_or(String::new(), |e| e.name.clone());
                let t = text!("cl-road-cut", road = name);
                self.note(t.clone());
                ev.push(CampaignEvent::Message(t));
            }
            _ => {}
        }
    }

    /// Road run `tag`'s convoy is over: its trucks that got there deliver their thirds of the
    /// load (those lost on the way with theirs).
    fn supply_arrived(&mut self, tag: u32, trucks: u8, ev: &mut Vec<CampaignEvent>) {
        let Some(k) = self
            .cargo_runs
            .iter()
            .position(|c| c.tag == tag && c.road.is_some() && c.state == CargoState::Airborne)
        else {
            return;
        };
        let run = self.cargo_runs.remove(k);
        let road = run.road.clone().expect("road");
        let whole = road.trucks();
        let lost = whole.saturating_sub(trucks + road.turned_back);
        self.supply_log.trucks_lost += lost as u32;
        self.supply_log.trucks_arrived += trucks as u32;
        self.supply_log.convoys += 1;
        self.board_at = f64::MIN;
        if trucks > 0 && self.owner(run.to) == Some(run.side) {
            for &(m, n) in &run.load {
                self.put_in(run.to, m, share(n, trucks, whole));
            }
        }
        self.supply_log.items_lost += run.load.iter().map(|&(_, n)| share(n, lost, whole)).sum::<u32>();
        if trucks == 0 && road.turned_back == 0 {
            // The whole convoy destroyed: the other side hears of it.
            let base = self.site(run.to).map_or(String::new(), |s| s.name.clone());
            let id = if run.side == Side::Blue { "cl-supply-convoy-lost" } else { "cl-enemy-supply-convoy-lost" };
            let t = text!(id, base = base);
            self.note(t.clone());
            ev.push(CampaignEvent::Message(t));
        }
    }
}

/// The supply convoys' record (the soak test's numbers).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SupplyLog {
    pub convoys: u32,
    pub trucks_arrived: u32,
    pub trucks_lost: u32,
    pub items_lost: u32,
}

impl SupplyLog {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl ConvoyUnit {
    /// A supply truck (carries a share of its convoy's load).
    pub fn truck(self) -> bool {
        matches!(self, ConvoyUnit::CivTruck | ConvoyUnit::ArmyTruck)
    }
}
