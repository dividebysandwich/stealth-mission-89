//! Resupply of the friendly air defence: a player at a Patriot battery's or Gepard's console
//! asks for a new stockpile (or the stockpile ran out and one is ordered), and the request goes
//! on the board for the UH-60 pilots (hover
//! low at the unit with the load). Nobody takes it for `SUPPLY_AI_WAIT`: an AI helicopter
//! flies it from the nearest airfield, and it leaves the board.

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, Report, WorldView};
use stealth_i18n::{Text, text};

impl CampaignState {
    /// A player at an air defence console asks for a new stockpile for its unit (None: it ran
    /// out and the request goes out by itself).
    pub(crate) fn request_resupply(
        &mut self,
        player: Option<PlayerId>,
        aaa: u32,
        x: i32,
        y: i32,
        kind: DefenceKind,
    ) -> Result<Vec<CampaignEvent>, Text> {
        if let Some(r) = self.supply_requests.iter().find(|r| r.aaa == aaa) {
            return Err(match r.state {
                SupplyState::Open { .. } => text!("ce-resupply-requested"),
                SupplyState::Ai { .. } => text!("ce-resupply-on-way"),
            });
        }
        if self.supply_base(x, y).is_none() {
            return Err(text!("ce-no-supply-airfield"));
        }
        let name = self
            .places
            .iter()
            .min_by_key(|p| dist(p.x - x, p.y - y))
            .map_or(String::new(), |p| p.name.clone());
        let id = self.id();
        let now = self.clock;
        let r = SupplyRequest {
            id,
            aaa,
            x,
            y,
            kind,
            name,
            by: player,
            posted: now,
            state: SupplyState::Open { since: now },
        };
        let text = match player {
            Some(p) => text!("cl-resupply-requested", who = self.who(p), unit = r.title()),
            None => {
                let id = match kind {
                    DefenceKind::Gepard => "cl-resupply-out-of-rounds",
                    DefenceKind::Patriot => "cl-resupply-out-of-missiles",
                    DefenceKind::Drones => "cl-resupply-out-of-drones",
                };
                text!(id, unit = r.title())
            }
        };
        self.supply_requests.push(r);
        self.note(text.clone());
        self.board_at = f64::MIN;
        Ok(vec![CampaignEvent::Message(text)])
    }

    /// An air defence unit (AAA index, where it stands, what it is) has used
    /// up its stockpile: a resupply is ordered unless one is already under way (or no airfield
    /// is left to fly it from).
    pub fn order_resupply(
        &mut self,
        aaa: u32,
        x: i32,
        y: i32,
        kind: DefenceKind,
    ) -> Vec<CampaignEvent> {
        if self.outcome.is_some()
            || self.supply_requests.iter().any(|r| r.aaa == aaa || (r.x, r.y) == (x, y))
        {
            return Vec::new();
        }
        self.request_resupply(None, aaa, x, y, kind).unwrap_or_default()
    }

    /// The friendly airfield a resupply for (x, y) is flown from: the nearest intact one, else
    /// the nearest base.
    pub(crate) fn supply_base(&self, x: i32, y: i32) -> Option<ObjectId> {
        let near = |b: &ObjectId| self.site(*b).map_or(i32::MAX, |s| dist(s.x - x, s.y - y));
        let bases = self.bases(Side::Blue);
        bases
            .iter()
            .copied()
            .filter(|&b| self.site(b).is_some_and(|s| s.kind == SiteKind::Airfield))
            .min_by_key(near)
            .or_else(|| bases.iter().copied().min_by_key(near))
    }

    /// Every tick: requests whose unit is gone are dropped, the ones a player flies wait, the
    /// ones nobody took for long enough get an AI helicopter.
    pub(crate) fn run_resupply(&mut self, view: &WorldView, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        let flown: Vec<u32> = self
            .players
            .iter()
            .filter_map(|p| match p.sortie.as_ref()?.offer.objective {
                Objective::Resupply { request } => Some(request),
                _ => None,
            })
            .collect();
        for k in (0..self.supply_requests.len()).rev() {
            let r = &self.supply_requests[k];
            // The unit where it stands (a reloaded simulation may number it anew).
            let alive =
                |a: &crate::GroundView| a.alive && a.side == Side::Blue && (a.x, a.y) == (r.x, r.y);
            let unit = match view.units.get(r.aaa as usize) {
                Some(a) if alive(a) => Some(r.aaa),
                _ => view.units.iter().position(alive).map(|i| i as u32),
            };
            let Some(unit) = unit else {
                // Its helicopter flies home with the load.
                let r = self.supply_requests.remove(k);
                self.board_at = f64::MIN;
                self.note(text!("cl-resupply-lost-unit", unit = r.title()));
                continue;
            };
            let r = &mut self.supply_requests[k];
            r.aaa = unit;
            let SupplyState::Open { since } = r.state else { continue };
            if flown.contains(&r.id) {
                r.state = SupplyState::Open { since: now };
                continue;
            }
            if now - since < SUPPLY_AI_WAIT {
                continue;
            }
            let (x, y) = (r.x, r.y);
            let Some(from) = self.supply_base(x, y) else {
                self.supply_requests[k].state = SupplyState::Open { since: now };
                continue;
            };
            let tag = self.id();
            let r = &mut self.supply_requests[k];
            r.state = SupplyState::Ai { tag };
            let (id, aaa, title) = (r.id, r.aaa, r.title());
            let base = self.site(from).map_or(String::new(), |s| s.name.clone());
            let text = text!("cl-resupply-ai", unit = title, base = base);
            self.note(text.clone());
            self.board_at = f64::MIN;
            ev.push(CampaignEvent::Message(text));
            ev.push(CampaignEvent::SupplyLaunched { id, from, aaa, tag });
        }
    }

    /// A campaign aircraft delivered, was shot down (`lost`) or landed: if it was a resupply
    /// helicopter, its request is done, or back on the board.
    pub(crate) fn supply_report(&mut self, r: &Report, ev: &mut Vec<CampaignEvent>) {
        let (tag, delivered) = match *r {
            Report::SupplyDelivered { tag } => (tag, true),
            Report::FlightDestroyed { tag, .. } | Report::FlightLanded { tag, .. } => (tag, false),
            _ => return,
        };
        let Some(k) = self
            .supply_requests
            .iter()
            .position(|q| matches!(q.state, SupplyState::Ai { tag: t } if t == tag))
        else {
            return;
        };
        if delivered {
            let id = self.supply_requests[k].id;
            self.resupplied(id, None, ev);
            return;
        }
        let now = self.clock;
        let r = &mut self.supply_requests[k];
        r.state = SupplyState::Open { since: now };
        let text = text!("cl-resupply-heli-lost", unit = r.title());
        self.note(text.clone());
        self.board_at = f64::MIN;
        ev.push(CampaignEvent::Message(text));
    }

    /// Request `id` delivered (by a player, else by the AI): the unit's stockpile is full again.
    pub(crate) fn resupplied(
        &mut self,
        id: u32,
        by: Option<PlayerId>,
        ev: &mut Vec<CampaignEvent>,
    ) {
        let Some(k) = self.supply_requests.iter().position(|r| r.id == id) else { return };
        let r = self.supply_requests.remove(k);
        let text = match by.and_then(|p| self.player_name(p)) {
            Some(who) => text!("cl-resupplied-by", unit = r.title(), who = who),
            None => text!("cl-resupplied", unit = r.title()),
        };
        self.note(text.clone());
        self.board_at = f64::MIN;
        ev.push(CampaignEvent::Message(text));
        ev.push(CampaignEvent::Restocked { aaa: r.aaa });
    }

    /// The board's resupply offers: the requests nobody's helicopter has taken yet.
    pub(crate) fn supply_offers(&self) -> Vec<(Text, Objective, i32, i32)> {
        self.supply_requests
            .iter()
            .filter(|r| matches!(r.state, SupplyState::Open { .. }))
            .map(|r| {
                (
                    text!("cb-resupply", place = r.title()),
                    Objective::Resupply { request: r.id },
                    r.x,
                    r.y,
                )
            })
            .collect()
    }
}
