//! Armour columns: each side sends one at a time from one of its places to the nearest enemy
//! place it can drive to. A column that destroys the place's garrison and still has an
//! infantry fighting vehicle takes the place; without one, a helicopter insertion is tried.

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, WorldView};
use stealth_i18n::text;

/// What a convoy's vehicle is (the simulation picks the side's model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvoyUnit {
    /// T-90, Leopard 2.
    Tank,
    /// BMP-2, Bradley: carries the troops that take a place.
    Carrier,
    /// ZSU-23-4, Gepard: covers the column against aircraft.
    Escort,
}

/// A convoy's column, head first: the air defence gun in the middle.
pub const COLUMN: [ConvoyUnit; 7] = [
    ConvoyUnit::Tank,
    ConvoyUnit::Tank,
    ConvoyUnit::Carrier,
    ConvoyUnit::Escort,
    ConvoyUnit::Carrier,
    ConvoyUnit::Carrier,
    ConvoyUnit::Tank,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvoyState {
    /// On its way, or fighting for the place.
    Moving,
    /// It cleared the place without troops left: helicopters are to take it.
    Insertion,
    /// It took the place (or its side had it by other means).
    Captured,
    /// Destroyed, or called off.
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Convoy {
    pub id: u32,
    pub side: Side,
    pub from: PlaceId,
    pub place: PlaceId,
    /// Its way (map units), from its place to the one it attacks.
    pub route: Vec<(i32, i32)>,
    /// Its vehicles still running, head first (as of the last look).
    pub units: Vec<ConvoyUnit>,
    /// Where its head was last seen (map units).
    pub at: (i32, i32),
    pub state: ConvoyState,
    pub launched_at: f64,
    /// Waiting for helicopters: since then, and the next try.
    #[serde(default)]
    pub retry_since: f64,
    #[serde(default)]
    pub retry_at: f64,
}

impl Convoy {
    pub fn active(&self) -> bool {
        self.state == ConvoyState::Moving
    }

    /// Nothing more comes of it.
    pub fn over(&self) -> bool {
        matches!(self.state, ConvoyState::Captured | ConvoyState::Failed)
    }

    /// The rest of its way from where its head is (to resume it after a load).
    pub fn route_left(&self) -> Vec<(i32, i32)> {
        let d2 = |a: (i32, i32), b: (i32, i32)| {
            let (dx, dy) = ((a.0 - b.0) as i64, (a.1 - b.1) as i64);
            dx * dx + dy * dy
        };
        // The leg it is on: the one whose end is nearer along it than its start.
        let leg = self
            .route
            .windows(2)
            .enumerate()
            .min_by_key(|(_, w)| {
                let (a, b) = (w[0], w[1]);
                let (ax, ay, bx, by) = (a.0 as f64, a.1 as f64, b.0 as f64, b.1 as f64);
                let (px, py) = (self.at.0 as f64, self.at.1 as f64);
                let l2 = ((bx - ax).powi(2) + (by - ay).powi(2)).max(1.0);
                let t = (((px - ax) * (bx - ax) + (py - ay) * (by - ay)) / l2).clamp(0.0, 1.0);
                let q = ((ax + (bx - ax) * t) as i32, (ay + (by - ay) * t) as i32);
                d2(q, self.at)
            })
            .map_or(0, |(k, _)| k);
        let mut out = vec![self.at];
        out.extend(self.route[leg + 1..].iter().copied());
        out.dedup();
        if out.len() < 2 {
            out.push(*self.route.last().unwrap_or(&self.at));
        }
        out
    }

    fn carriers(&self) -> usize {
        self.units.iter().filter(|&&u| u == ConvoyUnit::Carrier).count()
    }
}

fn index(side: Side) -> usize {
    (side == Side::Red) as usize
}

impl CampaignState {
    pub(crate) fn run_convoys(&mut self, view: &WorldView, ev: &mut Vec<CampaignEvent>) {
        for k in 0..self.convoys.len() {
            if self.convoys[k].active() {
                self.follow_convoy(k, view, ev);
            }
        }
        for side in [Side::Blue, Side::Red] {
            self.send_convoy(side, ev);
        }
        // A side whose convoy cleared a place without troops sends helicopters (tried again
        // every minute while it has none to send; given up after a while).
        for k in 0..self.convoys.len() {
            let c = &self.convoys[k];
            if c.state != ConvoyState::Insertion || self.clock < c.retry_at {
                continue;
            }
            let (side, place) = (c.side, c.place);
            if self.places[place as usize].owner != side.enemy()
                || self.clock - c.retry_since > CONVOY_INSERTION_WAIT
            {
                self.convoys[k].state = ConvoyState::Failed;
                continue;
            }
            if self.insertion_under_way(place) {
                self.convoys[k].retry_at = self.clock + 60.0;
                continue;
            }
            match self.order_insertion_for(side, None, place) {
                Ok(e) => {
                    ev.extend(e);
                    self.convoys[k].state = ConvoyState::Captured;
                }
                Err(_) => self.convoys[k].retry_at = self.clock + 60.0,
            }
        }
        // The last few that are over are kept for the record.
        while self.convoys.iter().filter(|c| c.over()).count() > 8 {
            let k = self.convoys.iter().position(Convoy::over).expect("one is over");
            self.convoys.remove(k);
        }
    }

    /// Convoy `k` as the simulation shows it: what is left of it, where it is, and whether it
    /// took its place.
    fn follow_convoy(&mut self, k: usize, view: &WorldView, ev: &mut Vec<CampaignEvent>) {
        let c = &self.convoys[k];
        let (id, side, place) = (c.id, c.side, c.place);
        let mine: Vec<_> = view.units.iter().filter(|u| u.convoy == id).collect();
        let p = &self.places[place as usize];
        let (px, py, name) = (p.x, p.y, p.name.clone());
        if p.owner == side {
            self.end_convoy(k, ConvoyState::Captured, ev);
            return;
        }
        // Not in the simulation yet (it is placed after this tick).
        if !mine.is_empty() {
            let alive: Vec<_> = mine.iter().filter(|u| u.alive).collect();
            let c = &mut self.convoys[k];
            c.units = alive.iter().filter_map(|u| u.vehicle).collect();
            if let Some(head) = alive.iter().min_by_key(|u| dist(u.x - px, u.y - py)) {
                c.at = (head.x, head.y);
            }
            if alive.is_empty() {
                self.note(if side == Side::Blue {
                    text!("cl-convoy-destroyed", place = name.as_str())
                } else {
                    text!("cl-enemy-convoy-destroyed", place = name.as_str())
                });
                ev.push(CampaignEvent::Message(if side == Side::Blue {
                    text!("cl-convoy-lost", place = name.as_str())
                } else {
                    text!(
                        "cl-exclaim",
                        text = text!("cl-enemy-convoy-destroyed", place = name.as_str())
                    )
                }));
                self.end_convoy(k, ConvoyState::Failed, ev);
                return;
            }
        }
        let c = &self.convoys[k];
        if self.clock - c.launched_at > CONVOY_TIMEOUT {
            self.end_convoy(k, ConvoyState::Failed, ev);
            return;
        }
        let near = dist(c.at.0 - px, c.at.1 - py) <= CONVOY_NEAR;
        if !near || mine.is_empty() {
            return;
        }
        // The place's armour, and what is left of convoys that stopped there.
        let moving: Vec<u32> = self.convoys.iter().filter(|c| c.active()).map(|c| c.id).collect();
        let garrison = view.units.iter().any(|u| {
            u.alive
                && u.side == side.enemy()
                && u.vehicle.is_some()
                && !moving.contains(&u.convoy)
                && dist(u.x - px, u.y - py) <= GARRISON_RADIUS
        });
        if garrison {
            return;
        }
        if self.convoys[k].carriers() > 0 {
            self.end_convoy(k, ConvoyState::Captured, ev);
            self.capture_place(place, side, ev);
        } else {
            self.note(if side == Side::Blue {
                text!("cl-convoy-cleared", place = name.as_str())
            } else {
                text!("cl-enemy-convoy-cleared", place = name.as_str())
            });
            self.convoys[k].retry_since = self.clock;
            self.convoys[k].retry_at = self.clock;
            self.end_convoy(k, ConvoyState::Insertion, ev);
        }
    }

    /// Convoy `k` is over: its vehicles stop where they are.
    fn end_convoy(&mut self, k: usize, state: ConvoyState, ev: &mut Vec<CampaignEvent>) {
        let c = &mut self.convoys[k];
        c.state = state;
        let (id, side) = (c.id, c.side);
        self.convoy_at[index(side)] = self.clock;
        self.board_at = f64::MIN;
        ev.push(CampaignEvent::ConvoyEnded { id });
    }

    fn insertion_under_way(&self, place: PlaceId) -> bool {
        self.insertions.iter().any(|i| {
            i.place == place
                && matches!(i.state, InsertionState::Pending { .. } | InsertionState::Enroute)
        })
    }

    /// A side with no convoy on the way sends the next when it is due: from one of its places
    /// to the nearest enemy place it can drive to.
    fn send_convoy(&mut self, side: Side, ev: &mut Vec<CampaignEvent>) {
        if self.convoys.iter().any(|c| c.side == side && c.active()) {
            return;
        }
        let skill = (self.setup.opponents as usize).min(3);
        let every = if side == Side::Blue { CONVOY_EVERY } else { ENEMY_CONVOY_EVERY[skill] };
        if self.clock - self.convoy_at[index(side)] < every {
            return;
        }
        // Another look in a while, whatever comes of this one.
        self.convoy_at[index(side)] = self.clock - every + 120.0;
        let Some(grid) = self.terrain.ground() else { return };
        let mut pairs: Vec<(i32, PlaceId, PlaceId)> = Vec::new();
        for (a, pa) in self.places.iter().enumerate() {
            if pa.owner != side {
                continue;
            }
            for (b, pb) in self.places.iter().enumerate() {
                let d = dist(pa.x - pb.x, pa.y - pb.y);
                if pb.owner == side.enemy() && d <= CONVOY_MAX_ROUTE {
                    pairs.push((d, a as PlaceId, b as PlaceId));
                }
            }
        }
        pairs.sort();
        let mut found = None;
        for (_, a, b) in pairs {
            let (pa, pb) = (&self.places[a as usize], &self.places[b as usize]);
            if grid.area(pa.x, pa.y).is_none() || grid.area(pa.x, pa.y) != grid.area(pb.x, pb.y) {
                continue;
            }
            if let Some(route) = grid.route((pa.x, pa.y), (pb.x, pb.y), CONVOY_MAX_ROUTE) {
                found = Some((a, b, route));
                break;
            }
        }
        let Some((from, place, route)) = found else { return };
        if !self.pay(side, CONVOY_COST, Spend::Operations) {
            return;
        }
        let id = self.id();
        let at = route[0];
        self.convoys.push(Convoy {
            id,
            side,
            from,
            place,
            route: route.clone(),
            units: COLUMN.to_vec(),
            at,
            state: ConvoyState::Moving,
            launched_at: self.clock,
            retry_since: 0.0,
            retry_at: 0.0,
        });
        self.convoy_at[index(side)] = self.clock;
        let (src, dst) =
            (self.places[from as usize].name.clone(), self.places[place as usize].name.clone());
        let text = if side == Side::Blue {
            text!("cl-convoy-launched", from = src.as_str(), place = dst.as_str())
        } else {
            text!("cl-enemy-convoy-intel", from = src.as_str(), place = dst.as_str())
        };
        self.note(text.clone());
        self.board_at = f64::MIN;
        ev.push(CampaignEvent::Message(text));
        ev.push(CampaignEvent::ConvoyLaunched { id, side, units: COLUMN.to_vec(), route });
    }

    /// The convoys to put in a new simulation (after a load): those on their way, from where
    /// they were, and the vehicles of those that stopped short of a place to stand there.
    pub fn convoys_to_place(&self) -> Vec<(u32, Side, Vec<ConvoyUnit>, Vec<(i32, i32)>, bool)> {
        self.convoys
            .iter()
            .filter(|c| matches!(c.state, ConvoyState::Moving | ConvoyState::Insertion))
            .filter(|c| !c.units.is_empty())
            .map(|c| (c.id, c.side, c.units.clone(), c.route_left(), c.active()))
            .collect()
    }

    /// Test aid: both sides' next convoys are due now.
    pub fn convoys_due(&mut self) {
        self.convoy_at = [f64::MIN / 2.0; 2];
    }
}
