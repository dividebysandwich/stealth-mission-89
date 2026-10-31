//! New friendly air defence sites: a player designates a spot on the map for a Gepard, a
//! Patriot battery or a drone command center, a helipad goes there, and logistics missions go on the board for the UH-60
//! pilots (land the cargo from the nearest airfield on the pad). A load nobody takes for
//! `DEPLOY_AI_WAIT` goes by AI helicopter. All loads in within `DEPLOY_TIMEOUT`, the unit
//! stands beside the pad; else the helipad is gone.

use stealth_i18n::Text;

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, GroundView, Report, WorldView};

/// The ground a site takes (map units from its unit).
pub fn footprint(kind: DefenceKind) -> i32 {
    match kind {
        DefenceKind::Gepard => DEPLOY_FOOTPRINT_GEPARD,
        DefenceKind::Patriot => DEPLOY_FOOTPRINT_PATRIOT,
        DefenceKind::Drones => DEPLOY_FOOTPRINT_DRONES,
    }
}

/// The loads of cargo a site needs.
pub fn loads(kind: DefenceKind) -> u8 {
    match kind {
        DefenceKind::Gepard => DEPLOY_LOADS_GEPARD,
        DefenceKind::Patriot => DEPLOY_LOADS_PATRIOT,
        DefenceKind::Drones => DEPLOY_LOADS_DRONES,
    }
}

impl CampaignState {
    /// Why a new site may not go to (x, y) (the helipad; a site of `kind`), None
    /// if it may: on land in friendly territory, clear of the towns, airfields and other world
    /// objects, and of the units on the ground (`aaa`, the simulation's air defence).
    pub fn deploy_refusal(
        &self,
        x: i32,
        y: i32,
        kind: DefenceKind,
        aaa: &[GroundView],
    ) -> Option<String> {
        if self.outcome.is_some() {
            return Some("The war is over".into());
        }
        if !(0x200..0x7E00).contains(&x) || !(0x200..0x7E00).contains(&y) {
            return Some("Too close to the edge of the map".into());
        }
        let open = self.deployments.iter().filter(|d| !d.built).count();
        if open >= DEPLOY_OPEN_MAX {
            return Some(format!("{DEPLOY_OPEN_MAX} sites are waiting for their cargo already"));
        }
        let foot = footprint(kind);
        let wet = |x: i32, y: i32| self.terrain.is_water(x, y) == Some(true);
        let ring = (0..8).map(|k| {
            let t = k as f64 * std::f64::consts::FRAC_PI_4;
            (x + (t.cos() * foot as f64) as i32, y + (t.sin() * foot as f64) as i32)
        });
        if wet(x, y) || ring.into_iter().any(|(px, py)| wet(px, py)) {
            return Some("Not on dry land".into());
        }
        let near = |px: i32, py: i32, d: i32| dist(px - x, py - y) < d;
        let nearest = self.places.iter().min_by_key(|p| dist(p.x - x, p.y - y));
        if nearest.is_none_or(|p| p.owner != Side::Blue) {
            return Some("Not in friendly territory".into());
        }
        if let Some(p) = self.places.iter().find(|p| near(p.x, p.y, DEPLOY_PLACE_CLEAR + foot)) {
            let what = if p.airfield.is_some() { "airfield" } else { "town" };
            return Some(format!("Too close to the {what} of {}", p.name));
        }
        if self.sites.iter().any(|s| !s.removed && near(s.x, s.y, DEPLOY_SITE_CLEAR + foot)) {
            return Some("Too close to a city, an airfield or a site".into());
        }
        let (ux, uy) = (x, y - DEPLOY_UNIT_OFFSET);
        let unit_near = |px: i32, py: i32, d: i32| dist(px - ux, py - uy) < d;
        if aaa.iter().any(|a| a.alive && unit_near(a.x, a.y, foot + 8)) {
            return Some("Too close to units on the ground".into());
        }
        if self.batteries.iter().any(|b| unit_near(b.x, b.y, BATTERY_RADIUS + foot)) {
            return Some("Too close to an enemy battery".into());
        }
        if self.deployments.iter().any(|d| near(d.x, d.y, footprint(d.kind) + foot)) {
            return Some("Too close to another new site".into());
        }
        if self.supply_base(x, y).is_none() {
            return Some("No friendly airfield to fly the cargo from".into());
        }
        None
    }

    /// A player designates a new site at (x, y): its helipad goes there and its loads go on the
    /// board. (The host has checked the units on the ground.)
    pub(crate) fn designate(
        &mut self,
        player: PlayerId,
        x: i32,
        y: i32,
        kind: DefenceKind,
    ) -> Result<Vec<CampaignEvent>, Text> {
        if let Some(why) = self.deploy_refusal(x, y, kind, &[]) {
            return Err(Text::raw(why));
        }
        let name = self
            .places
            .iter()
            .min_by_key(|p| dist(p.x - x, p.y - y))
            .map_or(String::new(), |p| p.name.clone());
        let id = self.id();
        let now = self.clock;
        let loads = loads(kind);
        let d = Deployment {
            id,
            x,
            y,
            kind,
            name,
            by: Some(player),
            posted: now,
            expires: now + DEPLOY_TIMEOUT,
            loads,
            delivered: Vec::new(),
            ai: Vec::new(),
            open_since: now,
            built: false,
            seen: false,
            base: self.supply_base(x, y),
            crates: vec![CrateState::AtBase; loads as usize],
        };
        let who = self.player_name(player).unwrap_or("A pilot");
        let loads = if d.loads == 1 { "1 load".to_string() } else { format!("{} loads", d.loads) };
        let text =
            Text::raw(format!("{who} designates a new {}: {loads} of cargo needed", d.title()));
        self.deployments.push(d);
        self.note(text.clone());
        self.board_at = f64::MIN;
        Ok(vec![CampaignEvent::Message(text)])
    }

    /// Every tick: sites not supplied in time are called off, built ones whose units are all
    /// gone are dropped, loads nobody flew for long enough get an AI helicopter.
    pub(crate) fn run_deployments(&mut self, view: &WorldView, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        let flown: Vec<(u32, u8)> = self
            .players
            .iter()
            .filter_map(|p| match p.sortie.as_ref()?.offer.objective {
                Objective::Logistics { site, leg } => Some((site, leg)),
                _ => None,
            })
            .collect();
        // A crate whose helicopter is gone (the sortie is over, the player left) is back at its
        // base.
        let flying: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|p| p.sortie.is_some())
            .map(|p| p.id)
            .filter(|id| view.players.iter().any(|p| p.id == *id))
            .collect();
        for d in &mut self.deployments {
            for leg in 0..d.loads {
                if let CrateState::Carried(p) = d.crate_state(leg)
                    && !flying.contains(&p)
                {
                    d.set_crate(leg, CrateState::AtBase);
                    self.board_at = f64::MIN;
                }
            }
        }
        for k in (0..self.deployments.len()).rev() {
            let d = &self.deployments[k];
            if d.built {
                let (ux, uy) = d.unit_at();
                let r = footprint(d.kind);
                let up = self.guns_within(view, ux, uy, Side::Blue, r).next().is_some();
                if up && !d.seen {
                    self.deployments[k].seen = true;
                } else if !up && d.seen {
                    let d = self.deployments.remove(k);
                    let text = Text::raw(format!("The {} is destroyed", d.title()));
                    self.note(text.clone());
                    ev.push(CampaignEvent::Message(text));
                }
                continue;
            }
            if now >= d.expires {
                // Its AI helicopters fly home with the cargo.
                let d = self.deployments.remove(k);
                for &(_, tag) in &d.ai {
                    ev.push(CampaignEvent::Recall { tag });
                }
                self.board_at = f64::MIN;
                let text = Text::raw(format!(
                    "The {} was not supplied in time: helipad removed",
                    d.title()
                ));
                self.note(text.clone());
                ev.push(CampaignEvent::Message(text));
                continue;
            }
            let id = d.id;
            let leg = d.open_legs().find(|l| !flown.contains(&(id, *l)));
            let Some(leg) = leg else {
                self.deployments[k].open_since = now;
                continue;
            };
            if now - d.open_since < DEPLOY_AI_WAIT {
                continue;
            }
            let (x, y) = (d.x, d.y);
            let Some(from) = self.supply_base(x, y) else { continue };
            let tag = self.id();
            let d = &mut self.deployments[k];
            d.ai.push((leg, tag));
            d.open_since = now;
            let title = d.title();
            let base = self.site(from).map_or(String::new(), |s| s.name.clone());
            let text =
                Text::raw(format!("A UH-60 takes a load of cargo for the {title} from {base}"));
            self.note(text.clone());
            self.board_at = f64::MIN;
            ev.push(CampaignEvent::Message(text));
            ev.push(CampaignEvent::CargoLaunched { id, from, x, y, tag });
        }
    }

    /// A campaign aircraft delivered, was shot down or landed: if it was a cargo helicopter,
    /// its load is in, or back on the board.
    pub(crate) fn cargo_report(&mut self, r: &Report, ev: &mut Vec<CampaignEvent>) {
        let (tag, delivered) = match *r {
            Report::SupplyDelivered { tag } => (tag, true),
            Report::FlightDestroyed { tag, .. } | Report::FlightLanded { tag, .. } => (tag, false),
            _ => return,
        };
        let now = self.clock;
        let Some(d) = self.deployments.iter_mut().find(|d| d.ai.iter().any(|a| a.1 == tag)) else {
            return;
        };
        let k = d.ai.iter().position(|a| a.1 == tag).expect("found");
        let (leg, _) = d.ai.remove(k);
        let id = d.id;
        if delivered {
            d.set_crate(leg, crate_slot(leg));
            self.logistics_delivered(id, leg, None, ev);
            return;
        }
        d.open_since = now;
        let text =
            Text::raw(format!("Cargo helicopter for the {} lost - back on the board", d.title()));
        self.note(text.clone());
        self.board_at = f64::MIN;
        ev.push(CampaignEvent::Message(text));
    }

    /// A load for site `id` is in (by a player, else by the AI); with the last one the site is
    /// built.
    pub(crate) fn logistics_delivered(
        &mut self,
        id: u32,
        leg: u8,
        by: Option<PlayerId>,
        ev: &mut Vec<CampaignEvent>,
    ) {
        let who =
            by.and_then(|p| self.player_name(p)).map(|n| format!(" by {n}")).unwrap_or_default();
        self.board_at = f64::MIN;
        let Some(d) = self.deployments.iter_mut().find(|d| d.id == id && !d.built) else { return };
        // Someone else brought this one already: it counts for the next.
        let leg = if d.delivered.contains(&leg) {
            match (0..d.loads).find(|l| !d.delivered.contains(l)) {
                Some(l) => l,
                None => return,
            }
        } else {
            leg
        };
        d.delivered.push(leg);
        // A player beat its AI helicopter to it: that one flies home.
        for &(_, tag) in d.ai.iter().filter(|a| a.0 == leg) {
            ev.push(CampaignEvent::Recall { tag });
        }
        d.ai.retain(|a| a.0 != leg);
        let (done, loads) = (d.delivered.len(), d.loads as usize);
        if done < loads {
            let text =
                Text::raw(format!("Cargo {done} of {loads} for the {} delivered{who}", d.title()));
            self.note(text.clone());
            ev.push(CampaignEvent::Message(text));
            return;
        }
        d.built = true;
        d.crates.clear();
        let ((x, y), kind, title) = (d.unit_at(), d.kind, d.title());
        for (_, tag) in std::mem::take(&mut d.ai) {
            ev.push(CampaignEvent::Recall { tag });
        }
        let text = Text::raw(format!("The {title} is supplied{who} and ready for action"));
        self.note(text.clone());
        ev.push(CampaignEvent::Message(text));
        ev.push(CampaignEvent::SiteBuilt { id, x, y, kind });
    }

    /// Why the player may not hook load `leg` of site `id` now, None if he may: in a
    /// helicopter on a sortie near the crate's base (`view`, where the players are), not carrying
    /// one already, and the crate still waiting there.
    pub fn hook_refusal(
        &self,
        player: PlayerId,
        id: u32,
        leg: u8,
        view: &WorldView,
    ) -> Option<String> {
        let Some(d) = self.deployment(id).filter(|d| leg < d.loads) else {
            return Some("No such cargo".into());
        };
        if !d.crate_waiting(leg) {
            return Some("That cargo is taken".into());
        }
        let heli = self
            .players
            .iter()
            .find(|p| p.id == player)
            .and_then(|p| p.sortie.as_ref())
            .is_some_and(|s| s.progress.heli);
        if !heli {
            return Some("Cargo is flown on a helicopter sortie".into());
        }
        if self.deployments.iter().any(|d| d.carried_by(player).is_some()) {
            return Some("Already carrying cargo".into());
        }
        let base = d.base.or_else(|| self.supply_base(d.x, d.y)).and_then(|b| self.site(b));
        let me = view.players.iter().find(|p| p.id == player);
        match (base, me) {
            (Some(b), Some(me)) if dist(me.x - b.x, me.y - b.y) <= CRATE_HOOK_RANGE => None,
            _ => Some("Too far from the cargo".into()),
        }
    }

    /// The player hooked load `leg` of site `id` under his helicopter.
    pub(crate) fn hook_crate(
        &mut self,
        player: PlayerId,
        id: u32,
        leg: u8,
    ) -> Result<Vec<CampaignEvent>, Text> {
        let Some(d) = self.deployments.iter_mut().find(|d| d.id == id && leg < d.loads) else {
            return Err(Text::raw("No such cargo"));
        };
        if !d.crate_waiting(leg) {
            return Err(Text::raw("That cargo is taken"));
        }
        d.set_crate(leg, CrateState::Carried(player));
        self.board_at = f64::MIN;
        Ok(vec![CampaignEvent::CrateHooked { id, leg, player }])
    }

    /// The player set load `leg` of site `id` down at (`east`, `north`) metres from its
    /// helipad: it is delivered (and his sortie's objective is met, if it was this site's).
    pub(crate) fn drop_crate(
        &mut self,
        player: PlayerId,
        id: u32,
        leg: u8,
        east: f32,
        north: f32,
        heading: f32,
    ) -> Result<Vec<CampaignEvent>, Text> {
        let Some(d) = self.deployments.iter_mut().find(|d| d.id == id && leg < d.loads) else {
            return Err(Text::raw("No such cargo"));
        };
        if d.crate_state(leg) != CrateState::Carried(player) {
            return Err(Text::raw("Not carrying that cargo"));
        }
        let off = east.hypot(north);
        if off.is_nan() || off > CRATE_DROP_METRES * 1.1 {
            return Err(Text::raw("Too far from the helipad"));
        }
        d.set_crate(leg, CrateState::Delivered { east, north, heading });
        let mut ev = vec![CampaignEvent::CrateDropped { id, leg, east, north, heading }];
        self.logistics_delivered(id, leg, Some(player), &mut ev);
        if let Some(s) =
            self.players.iter_mut().find(|p| p.id == player).and_then(|p| p.sortie.as_mut())
            && matches!(s.offer.objective, Objective::Logistics { site, .. } if site == id)
        {
            s.progress.delivered = true;
        }
        Ok(ev)
    }

    /// The board's logistics offers: the loads of the sites waiting for cargo that no AI
    /// helicopter is bringing.
    pub(crate) fn logistics_offers(&self) -> Vec<(Text, Objective, i32, i32)> {
        self.deployments
            .iter()
            .filter(|d| !d.built)
            .flat_map(|d| {
                d.open_legs().map(move |leg| {
                    (
                        Text::raw(format!(
                            "{} {}/{} - {}",
                            d.kind.short(),
                            leg + 1,
                            d.loads,
                            d.name
                        )),
                        Objective::Logistics { site: d.id, leg },
                        d.x,
                        d.y,
                    )
                })
            })
            .collect()
    }
}
