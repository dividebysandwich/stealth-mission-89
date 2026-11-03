//! The AI flies the missions of the boards: an offer of a side's board that nobody flies gets
//! a flight of the aircraft made for it `AI_MISSION_WAIT` after it was posted. The enemy has a
//! board of its own (`red_board`) that only its AI flies; its strikes on the friendly
//! airfields go up as raids (the friendly side hears of them and defends). A player may take
//! over one of a friendly flight's aircraft in the air, and leave his aircraft to the AI.

use crate::state::*;
use crate::tuning::*;
use crate::{CampaignEvent, Report, WorldView};
use serde::{Deserialize, Serialize};
use stealth_i18n::{Text, text};

/// What flies a flight of the enemy (the simulation picks the type of the theater's era).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedJob {
    Strike,
    Fighter,
    Recon,
    Helicopter,
}

/// The aircraft of a flight: one of the players' for the friendly side, a job for the enemy's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiAirframe {
    Blue(Aircraft),
    Red(RedJob),
}

impl AiAirframe {
    pub fn helicopter(self) -> bool {
        match self {
            AiAirframe::Blue(a) => a.helicopter(),
            AiAirframe::Red(j) => j == RedJob::Helicopter,
        }
    }

    /// The type's name for the board (the enemy's is the simulation's to pick).
    pub fn name(self) -> &'static str {
        match self {
            AiAirframe::Blue(a) => a.name(),
            AiAirframe::Red(_) => "",
        }
    }
}

/// What a flight goes for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiTarget {
    /// A world object to bomb (an anti-radiation missile at a SAM site's radar).
    Object(ObjectId),
    /// The other side's ground units around (x, y): rockets at them (CAS).
    Ground { x: i32, y: i32 },
    /// A world object to photograph.
    Recon { object: ObjectId },
    /// Patrol over (x, y) and fight the other side's aircraft.
    Patrol { x: i32, y: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiMissionState {
    Enroute,
    Done,
}

/// A flight of the AI flying an offer of a side's board.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiMission {
    pub id: u32,
    pub side: Side,
    /// The offer as it was when the flight took off.
    pub offer: MissionOffer,
    pub airframe: AiAirframe,
    pub target: AiTarget,
    pub from: ObjectId,
    /// Campaign tags of its aircraft, which of them are still in the air, and the player
    /// flying one (taken over).
    pub tags: Vec<u32>,
    pub alive: Vec<bool>,
    pub controlled: Vec<Option<PlayerId>>,
    /// Out of the side's aircraft (else a player's own he left to the AI).
    pub pooled: Vec<bool>,
    pub launched: f64,
    pub state: AiMissionState,
    pub ended: Option<f64>,
    /// Its objective is achieved (the recon's photo taken).
    pub achieved: bool,
    pub photographed: bool,
}

impl AiMission {
    pub fn open(&self) -> bool {
        self.state == AiMissionState::Enroute
    }

    /// The tags the AI flies now (in the air, nobody took them over).
    pub fn flying(&self) -> impl Iterator<Item = u32> + '_ {
        (0..self.tags.len())
            .filter(|&k| self.alive[k] && self.controlled[k].is_none())
            .map(|k| self.tags[k])
    }

    /// The aircraft a player may take over (the first the AI flies).
    pub fn takeable(&self) -> Option<u32> {
        self.flying().next()
    }

    fn slot(&self, tag: u32) -> Option<usize> {
        self.tags.iter().position(|&t| t == tag)
    }
}

/// The missions the AI flies (the others fly themselves: insertions when ordered, the rescue
/// helicopter, the resupply and cargo helicopters).
pub fn ai_flown(kind: MissionKind) -> bool {
    !matches!(
        kind,
        MissionKind::Insertion | MissionKind::Csar | MissionKind::Resupply | MissionKind::Logistics
    )
}

/// The ground missions (the rest fight in the air).
fn ground(kind: MissionKind) -> bool {
    matches!(
        kind,
        MissionKind::Recon
            | MissionKind::Sead
            | MissionKind::Strike
            | MissionKind::Interdiction
            | MissionKind::AntiShip
            | MissionKind::Cas
    )
}

/// The aircraft for a friendly mission: the stealthy ones (and the Tornado low) into heavy air
/// defences, the fighters where the enemy's are up, the helicopters for targets close to the
/// base, else the one made for it.
pub fn pick_blue(kind: MissionKind, sam: i32, air: usize, close: bool) -> Aircraft {
    use Aircraft::*;
    let heavy_sam = sam >= HEAVY_SAM_THREAT;
    let heavy_air = air >= HEAVY_AIR_THREAT;
    let pick = if ground(kind) {
        match kind {
            MissionKind::Recon if close => {
                if heavy_sam {
                    RAH66
                } else {
                    OH58
                }
            }
            MissionKind::Cas if close => {
                if heavy_sam {
                    RAH66
                } else {
                    AH1Z
                }
            }
            MissionKind::Sead if heavy_sam => {
                if heavy_air {
                    F22
                } else {
                    Tornado
                }
            }
            MissionKind::Cas if heavy_sam => Tornado,
            _ if heavy_sam => F117,
            MissionKind::Cas if heavy_air => AV8B,
            _ if heavy_air => F16,
            _ => kind.suited()[0],
        }
    } else if close && matches!(kind, MissionKind::Intercept | MissionKind::Escort) && !heavy_air {
        AH1Z
    } else if heavy_air {
        F22
    } else {
        F16
    };
    if kind.flyable_by(pick) { pick } else { kind.suited()[0] }
}

/// The enemy's aircraft for its mission.
pub fn pick_red(kind: MissionKind, close: bool) -> RedJob {
    match kind {
        MissionKind::Recon => RedJob::Recon,
        MissionKind::Cas if close => RedJob::Helicopter,
        _ if ground(kind) => RedJob::Strike,
        _ => RedJob::Fighter,
    }
}

impl CampaignState {
    /// The AI flies the boards' missions (the campaign's rule; not in an older test state).
    pub fn ai_missions_on(&self) -> bool {
        self.rules.is_some_and(|r| !r.no_ai_missions)
    }

    /// The AI flight of an offer: the one in the air, else the last.
    pub fn ai_mission_for(&self, offer: u32) -> Option<&AiMission> {
        let mut all = self.ai_missions.iter().filter(|m| m.offer.id == offer);
        let last = all.clone().next_back();
        all.find(|m| m.open()).or(last)
    }

    /// The AI flight one of whose aircraft has campaign tag `tag`.
    pub fn ai_mission_of_tag(&self, tag: u32) -> Option<&AiMission> {
        self.ai_missions.iter().find(|m| m.tags.contains(&tag))
    }

    /// When the AI takes off for an offer (None: it does not, or its flight is up).
    pub fn ai_launch_at(&self, o: &MissionOffer) -> Option<f64> {
        if !self.ai_missions_on() || !ai_flown(o.kind) || self.flown_by_player(o.id) {
            return None;
        }
        match self.ai_mission_for(o.id) {
            Some(m) if m.open() => None,
            Some(m) => Some(m.ended.unwrap_or(m.launched).max(o.posted) + AI_MISSION_WAIT),
            None => Some(o.posted + AI_MISSION_WAIT),
        }
    }

    /// A player is in the air on it.
    fn flown_by_player(&self, offer: u32) -> bool {
        self.players.iter().any(|p| p.sortie.as_ref().is_some_and(|s| s.offer.id == offer))
    }

    /// Enemy aircraft of `side` up within reach of a point.
    fn air_threat(&self, side: Side, x: i32, y: i32, view: &WorldView) -> usize {
        view.flights
            .iter()
            .filter(|f| f.side == side.enemy() && dist(f.x - x, f.y - y) <= AIR_THREAT_RANGE)
            .count()
    }

    /// The AI flights: the ones done come home, the offers due take off.
    pub(crate) fn run_ai_missions(&mut self, view: &WorldView, ev: &mut Vec<CampaignEvent>) {
        let now = self.clock;
        for k in 0..self.ai_missions.len() {
            if !self.ai_missions[k].open() {
                continue;
            }
            if !self.ai_missions[k].achieved && self.ai_achieved(&self.ai_missions[k], view) {
                let m = &mut self.ai_missions[k];
                m.achieved = true;
                let (side, title) = (m.side, m.offer.title.clone());
                let home: Vec<u32> = m.flying().collect();
                ev.extend(home.into_iter().map(|tag| CampaignEvent::Recall { tag }));
                if side == Side::Blue {
                    self.note(text!("cl-ai-mission-done", title = title));
                }
            }
            let m = &self.ai_missions[k];
            let gone = m.alive.iter().all(|a| !a) && m.controlled.iter().all(Option::is_none);
            if gone || now - m.launched > AI_MISSION_TIMEOUT {
                self.end_ai_mission(k);
            }
        }
        self.ai_missions
            .retain(|m| m.open() || m.ended.is_some_and(|t| now - t < AI_MISSION_MEMORY));
        if !self.ai_missions_on() || self.outcome.is_some() {
            return;
        }
        for side in [Side::Blue, Side::Red] {
            let board = if side == Side::Blue { &self.board } else { &self.red_board };
            let mut due: Vec<MissionOffer> = board
                .iter()
                .filter(|o| self.ai_launch_at(o).is_some_and(|t| now >= t))
                // The enemy's strikes on airfields go up as raids (`enemy_offensive`).
                .filter(|o| side == Side::Blue || !matches!(o.objective, Objective::Strike { .. }))
                .cloned()
                .collect();
            due.sort_by(|a, b| a.posted.total_cmp(&b.posted));
            for o in due {
                let open = self.ai_missions.iter().filter(|m| m.side == side && m.open()).count();
                if open >= AI_MISSIONS_MAX {
                    break;
                }
                self.launch_ai(side, &o, view, ev);
            }
        }
    }

    /// The objective of an AI flight is achieved.
    fn ai_achieved(&self, m: &AiMission, view: &WorldView) -> bool {
        if let Objective::Recon { .. } = m.offer.objective {
            return m.photographed;
        }
        let aaa = match &m.offer.objective {
            Objective::Cas { aaa, .. }
            | Objective::Battery { aaa, .. }
            | Objective::Armour { aaa, .. }
            | Objective::Support { aaa, .. } => aaa
                .iter()
                .copied()
                .filter(|&k| view.units.get(k as usize).is_some_and(|u| !u.alive))
                .collect(),
            _ => Vec::new(),
        };
        let up = self.clock - m.launched;
        let p = Progress {
            aaa_destroyed: aaa,
            on_station: if up >= 2.0 * CAP_SECONDS { CAP_SECONDS } else { 0.0 },
            near_secs: PRESENCE_SECONDS,
            ..Progress::default()
        };
        self.objective_met(&m.offer.objective, &p)
    }

    /// AI flight `k` is over (its aircraft down, back, or out too long).
    fn end_ai_mission(&mut self, k: usize) {
        let m = &mut self.ai_missions[k];
        m.state = AiMissionState::Done;
        m.ended = Some(self.clock);
        if m.side == Side::Blue && !m.achieved {
            let title = m.offer.title.clone();
            self.note(text!("cl-ai-mission-failed", title = title));
        }
    }

    /// Flight of the AI for offer `o` of `side`'s board, if the side has the aircraft and the
    /// materiel for it.
    fn launch_ai(
        &mut self,
        side: Side,
        o: &MissionOffer,
        view: &WorldView,
        ev: &mut Vec<CampaignEvent>,
    ) -> bool {
        let from = match side {
            Side::Blue => Some(o.base).filter(|&b| self.player_bases().contains(&b)),
            _ => self.nearest_base(side, o.x, o.y),
        };
        let Some(from) = from else { return false };
        let Some(b) = self.site(from) else { return false };
        let d = dist(b.x - o.x, b.y - o.y);
        if side == Side::Red && d > RAID_RANGE {
            return false;
        }
        let close = d <= AI_HELO_RANGE;
        let airframe = match side {
            Side::Blue => AiAirframe::Blue(pick_blue(
                o.kind,
                o.threat,
                self.air_threat(side, o.x, o.y, view),
                close,
            )),
            _ => AiAirframe::Red(pick_red(o.kind, close)),
        };
        let Some(target) = self.ai_target(o) else { return false };
        let n = if o.kind == MissionKind::Recon { AI_RECON_AIRCRAFT } else { AI_FLIGHT_AIRCRAFT };
        let f = self.forces(side);
        let pool = if airframe.helicopter() { f.helicopters } else { f.fighters };
        if (pool as usize) < n || f.materiel < AI_MISSION_COST {
            return false;
        }
        self.pay(side, AI_MISSION_COST, Spend::Operations);
        let f = self.forces_mut(side);
        let pool = if airframe.helicopter() { &mut f.helicopters } else { &mut f.fighters };
        *pool -= n as u32;
        let id = self.id();
        let tags: Vec<u32> = (0..n).map(|_| self.id()).collect();
        self.ai_missions.push(AiMission {
            id,
            side,
            offer: o.clone(),
            airframe,
            target,
            from,
            tags: tags.clone(),
            alive: vec![true; n],
            controlled: vec![None; n],
            pooled: vec![true; n],
            launched: self.clock,
            state: AiMissionState::Enroute,
            ended: None,
            achieved: false,
            photographed: false,
        });
        if side == Side::Blue {
            self.note(text!(
                "cl-ai-mission-launched",
                title = o.title.clone(),
                aircraft = airframe.name(),
                count = n as i64
            ));
        }
        ev.push(CampaignEvent::AiMissionLaunched { id, side, from, airframe, target, tags });
        true
    }

    /// What a flight on an offer goes for (None: nothing left to go for).
    fn ai_target(&self, o: &MissionOffer) -> Option<AiTarget> {
        let alive = |k: ObjectId| self.site(k).is_some_and(Site::alive);
        Some(match &o.objective {
            Objective::Recon { object, .. } => AiTarget::Recon { object: *object },
            Objective::Sead { sites, .. } => {
                AiTarget::Object(sites.iter().copied().find(|&s| alive(s))?)
            }
            Objective::Strike { airfield, .. } => AiTarget::Object(*airfield),
            Objective::Interdict { object, .. } => AiTarget::Object(*object),
            Objective::AntiShip { ship } => AiTarget::Object(*ship),
            Objective::Cas { .. }
            | Objective::Battery { .. }
            | Objective::Armour { .. }
            | Objective::Support { .. } => AiTarget::Ground { x: o.x, y: o.y },
            Objective::Cap { .. }
            | Objective::Defend { .. }
            | Objective::Intercept { .. }
            | Objective::Escort { .. }
            | Objective::Awacs { .. } => AiTarget::Patrol { x: o.x, y: o.y },
            _ => return None,
        })
    }

    /// What happened to the AI flights' aircraft.
    pub(crate) fn ai_mission_report(&mut self, r: &Report, ev: &mut Vec<CampaignEvent>) {
        let (tag, landed) = match *r {
            Report::FlightDestroyed { tag, .. } => (tag, false),
            Report::FlightLanded { tag, .. } => (tag, true),
            Report::Photographed { tag } => {
                self.ai_photo(tag, ev);
                return;
            }
            _ => return,
        };
        let Some(m) = self.ai_missions.iter_mut().find(|m| m.tags.contains(&tag)) else { return };
        let k = m.slot(tag).expect("contains");
        if !m.alive[k] || m.controlled[k].is_some() {
            return;
        }
        m.alive[k] = false;
        let (side, heli, pooled) = (m.side, m.airframe.helicopter(), m.pooled[k]);
        if landed && pooled {
            self.aircraft_back(side, !heli as u32, heli as u32);
        }
    }

    /// An AI recon flight's aircraft photographed its object: its side has the intelligence.
    fn ai_photo(&mut self, tag: u32, ev: &mut Vec<CampaignEvent>) {
        let Some(m) = self.ai_missions.iter_mut().find(|m| m.tags.contains(&tag)) else { return };
        if m.photographed {
            return;
        }
        m.photographed = true;
        let (side, objective) = (m.side, m.offer.objective.clone());
        let Objective::Recon { place, .. } = objective else { return };
        match side {
            Side::Blue => self.gain_intel(place, ev),
            _ => {
                let until = self.clock + RED_INTEL_SECONDS;
                if let Some(p) = self.places.get_mut(place as usize) {
                    p.red_intel_until = Some(until);
                }
                self.board_at = f64::MIN;
            }
        }
    }

    /// The enemy's own board, rebuilt with the friendly one: recon of the friendly places on
    /// its front, strikes on their airfields once it knows them, attacks on the friendly guns,
    /// armour and columns there. Offers keep their id while they stay.
    pub(crate) fn rebuild_red_board(&mut self, view: &WorldView) {
        let now = self.clock;
        let mut offers: Vec<MissionOffer> = Vec::new();
        let mut add = |c: &mut CampaignState,
                       kind,
                       title: Text,
                       objective: Objective,
                       x,
                       y,
                       threat| {
            let Some(base) = c.nearest_base(Side::Red, x, y) else { return };
            let (id, posted) = match c.red_board.iter().find(|o| o.objective == objective) {
                Some(o) => (o.id, o.posted),
                None => (c.id(), now),
            };
            offers.push(MissionOffer { id, kind, title, objective, x, y, base, threat, posted });
        };
        for pid in self.front(Side::Red) {
            let p = self.places[pid as usize].clone();
            let threat = self.threat_of(pid, Some(view));
            if !p.red_intel(now) {
                let object = p.airfield.or(p.members.first().copied()).unwrap_or(0);
                add(
                    self,
                    MissionKind::Recon,
                    text!("cb-recon", place = p.name.as_str()),
                    Objective::Recon { place: pid, object },
                    p.x,
                    p.y,
                    threat,
                );
            } else if let Some(a) = p.airfield.filter(|&a| self.site(a).is_some_and(Site::alive)) {
                add(
                    self,
                    MissionKind::Strike,
                    text!("cb-strike", place = p.name.as_str()),
                    Objective::Strike { place: pid, airfield: a },
                    p.x,
                    p.y,
                    threat,
                );
            }
            let aaa: Vec<u32> = (0..view.units.len() as u32)
                .filter(|&k| {
                    let u = &view.units[k as usize];
                    u.alive
                        && u.side == Side::Blue
                        && u.convoy == 0
                        && dist(u.x - p.x, u.y - p.y) <= GARRISON_RADIUS
                })
                .collect();
            if !aaa.is_empty() {
                add(
                    self,
                    MissionKind::Cas,
                    text!("cb-cas", place = p.name.as_str()),
                    Objective::Cas { place: pid, aaa },
                    p.x,
                    p.y,
                    threat,
                );
            }
        }
        for v in self.convoys.clone().into_iter().filter(|v| v.active() && v.side == Side::Blue) {
            let aaa: Vec<u32> = (0..view.units.len() as u32)
                .filter(|&k| {
                    let u = &view.units[k as usize];
                    u.convoy == v.id && u.alive
                })
                .collect();
            if aaa.is_empty() {
                continue;
            }
            let name = self.places[v.place as usize].name.clone();
            add(
                self,
                MissionKind::Cas,
                text!("cb-cas-armour", place = name.as_str()),
                Objective::Armour { convoy: v.id, aaa },
                v.at.0,
                v.at.1,
                0,
            );
        }
        self.red_board = offers;
    }

    /// A strike of the enemy's board on a friendly airfield that is due (its raid takes off
    /// `RAID_LEAD` after it is ordered, when the AI would fly it): place, airfield, base.
    pub(crate) fn red_strike_due(&self) -> Option<(PlaceId, ObjectId, ObjectId)> {
        let now = self.clock;
        self.red_board.iter().filter(|o| now >= o.posted + AI_MISSION_WAIT - RAID_LEAD).find_map(
            |o| {
                let Objective::Strike { place, airfield } = o.objective else { return None };
                if self.raids.iter().any(|r| r.open() && r.bombs(airfield)) || self.capped(place) {
                    return None;
                }
                let s = self.site(o.base)?;
                (dist(s.x - o.x, s.y - o.y) <= RAID_RANGE).then_some((place, airfield, o.base))
            },
        )
    }

    /// A player takes over aircraft `tag` of a friendly AI flight in the air: a sortie on its
    /// mission in its aircraft (the host puts him where it is).
    pub(crate) fn take_control(
        &mut self,
        player: PlayerId,
        tag: u32,
    ) -> Result<Vec<CampaignEvent>, Text> {
        if self.sortie(player).is_some() {
            return Err(text!("ce-already-flying"));
        }
        let Some(k) = self.ai_missions.iter().position(|m| m.open() && m.tags.contains(&tag))
        else {
            return Err(text!("ce-no-ai-aircraft"));
        };
        let m = &self.ai_missions[k];
        let slot = m.slot(tag).expect("contains");
        let AiAirframe::Blue(aircraft) = m.airframe else {
            return Err(text!("ce-no-ai-aircraft"));
        };
        if m.side != Side::Blue || !m.alive[slot] || m.controlled[slot].is_some() {
            return Err(text!("ce-no-ai-aircraft"));
        }
        let offer = m.offer.clone();
        let base = if self.player_bases().contains(&m.from) {
            m.from
        } else {
            self.nearest_player_base(offer.x, offer.y).unwrap_or(m.from)
        };
        let (flight, position) = self.callsign_numbers(offer.id);
        let sortie = Sortie {
            id: self.id(),
            offer: offer.clone(),
            base,
            aircraft,
            started: self.clock,
            progress: Progress { heli: aircraft.cargo(), ..Progress::default() },
            flight,
            position,
        };
        self.ai_missions[k].controlled[slot] = Some(player);
        self.note(text!("cl-took-control", title = offer.title.clone()));
        self.slot_mut(player).sortie = Some(sortie.clone());
        self.leave_plans(player);
        match self.plans.iter_mut().find(|p| p.offer == offer.id) {
            Some(p) => p.flying.push(player),
            None => self.plans.push(MissionPlan {
                offer: offer.id,
                base,
                route: Vec::new(),
                crew: Vec::new(),
                flying: vec![player],
            }),
        }
        Ok(vec![CampaignEvent::ControlTaken { player, tag, sortie }, self.plans_changed()])
    }

    /// A player in the air leaves his aircraft to the AI: it flies his mission on (the
    /// aircraft he took over goes back to its flight). His sortie ends when the host takes him
    /// out ([`SortieEnd::HandedOver`]).
    pub(crate) fn leave_aircraft(&mut self, player: PlayerId) -> Result<Vec<CampaignEvent>, Text> {
        let Some(s) = self.sortie(player).cloned() else {
            return Err(text!("ce-not-flying"));
        };
        let held = self.ai_missions.iter().position(|m| m.controlled.contains(&Some(player)));
        let (k, tag) = match held {
            Some(k) => {
                let m = &mut self.ai_missions[k];
                let slot = m.controlled.iter().position(|&c| c == Some(player)).expect("held");
                m.controlled[slot] = None;
                (k, m.tags[slot])
            }
            None => {
                let tag = self.id();
                let open = self
                    .ai_missions
                    .iter()
                    .position(|m| m.open() && m.side == Side::Blue && m.offer.id == s.offer.id);
                let k = match open {
                    Some(k) => k,
                    None => {
                        let target = self
                            .ai_target(&s.offer)
                            .unwrap_or(AiTarget::Patrol { x: s.offer.x, y: s.offer.y });
                        let id = self.id();
                        self.ai_missions.push(AiMission {
                            id,
                            side: Side::Blue,
                            offer: s.offer.clone(),
                            airframe: AiAirframe::Blue(s.aircraft),
                            target,
                            from: s.base,
                            tags: Vec::new(),
                            alive: Vec::new(),
                            controlled: Vec::new(),
                            pooled: Vec::new(),
                            launched: self.clock,
                            state: AiMissionState::Enroute,
                            ended: None,
                            achieved: false,
                            photographed: false,
                        });
                        self.ai_missions.len() - 1
                    }
                };
                let m = &mut self.ai_missions[k];
                m.tags.push(tag);
                m.alive.push(true);
                m.controlled.push(None);
                m.pooled.push(false);
                (k, tag)
            }
        };
        let m = &self.ai_missions[k];
        // Done with it (or nothing left to go for): the AI flies it home.
        let home = s.progress.achieved || m.achieved;
        Ok(vec![CampaignEvent::ControlLeft {
            player,
            tag,
            aircraft: s.aircraft,
            from: m.from,
            target: m.target,
            home,
        }])
    }

    /// A player's sortie in an aircraft he took over ended (not handed back): it is down, or
    /// landed and back with its side.
    pub(crate) fn controlled_ended(&mut self, player: PlayerId, end: SortieEnd) {
        let Some(m) = self.ai_missions.iter_mut().find(|m| m.controlled.contains(&Some(player)))
        else {
            return;
        };
        let k = m.controlled.iter().position(|&c| c == Some(player)).expect("held");
        m.controlled[k] = None;
        m.alive[k] = false;
        let (heli, pooled) = (m.airframe.helicopter(), m.pooled[k]);
        if end == SortieEnd::Landed && pooled {
            self.aircraft_back(Side::Blue, !heli as u32, heli as u32);
        }
    }
}
