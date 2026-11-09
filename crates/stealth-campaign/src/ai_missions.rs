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
    /// Wait over (x, y) for the other side's aircraft `tag` (a cargo flight), then go after it
    /// wherever it is.
    Hunt { tag: u32, x: i32, y: i32 },
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
    /// When each one takes off (None: in the air): a side's aircraft take off one at a time,
    /// `AI_STAGGER` apart.
    #[serde(default)]
    pub due: Vec<Option<f64>>,
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
            .filter(|&k| self.alive[k] && self.controlled[k].is_none() && !self.waiting(k))
            .map(|k| self.tags[k])
    }

    /// Aircraft `k` waits for its turn to take off.
    fn waiting(&self, k: usize) -> bool {
        self.due.get(k).copied().flatten().is_some()
    }

    /// When its next aircraft takes off (None: all are up).
    pub fn next_takeoff(&self) -> Option<f64> {
        self.due.iter().flatten().copied().min_by(f64::total_cmp)
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

    /// When the AI takes off for an offer (None: it does not, or not before one of its side's
    /// flights is back): its place in the side's queue ([`CampaignState::ai_schedule`]), or its
    /// flight's next aircraft's turn on the runway.
    pub fn ai_launch_at(&self, o: &MissionOffer) -> Option<f64> {
        if let Some(m) = self.ai_mission_for(o.id).filter(|m| m.open()) {
            return m.next_takeoff();
        }
        let side = if self.red_board.iter().any(|r| r.id == o.id) { Side::Red } else { Side::Blue };
        self.ai_schedule(side).into_iter().find(|s| s.0 == o.id).map(|s| s.1)
    }

    /// The side's offers the AI flies next and when, in turn: each `AI_MISSION_WAIT` after it
    /// was posted (or its last flight came back), on its turn on the runway (`AI_STAGGER` after
    /// the one before), with no more than `AI_AIRCRAFT_PER_WINDOW` aircraft in
    /// `AI_LAUNCH_WINDOW`, while fewer than `AI_MISSIONS_MAX` flights are out.
    pub fn ai_schedule(&self, side: Side) -> Vec<(u32, f64)> {
        if !self.ai_missions_on() {
            return Vec::new();
        }
        let board = if side == Side::Blue { &self.board } else { &self.red_board };
        let mut queue: Vec<(f64, &MissionOffer)> = board
            .iter()
            .filter(|o| ai_flown(o.kind) && !self.flown_by_player(o.id))
            // The enemy's strikes on airfields go up as raids (`enemy_offensive`).
            .filter(|o| side == Side::Blue || !matches!(o.objective, Objective::Strike { .. }))
            // No hunting a cargo aircraft under heavy air defences (the players may).
            .filter(|o| {
                !matches!(o.objective, Objective::Cargo { .. }) || o.threat < HEAVY_SAM_THREAT
            })
            .filter_map(|o| {
                let since = match self.ai_mission_for(o.id) {
                    Some(m) if m.open() => return None,
                    Some(m) => m.ended.unwrap_or(m.launched).max(o.posted),
                    None => o.posted,
                };
                // The interceptors of a cargo flight go up soon after it was scheduled.
                let wait = if matches!(o.objective, Objective::Cargo { .. }) {
                    CARGO_INTERCEPT_WAIT
                } else {
                    AI_MISSION_WAIT
                };
                Some((since + wait, o))
            })
            .collect();
        queue.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.posted.total_cmp(&b.1.posted)));
        // The interceptors of a cargo flight go ahead of the queue (they do not count
        // against it).
        let (hunts, queue): (Vec<_>, Vec<_>) =
            queue.into_iter().partition(|(_, o)| matches!(o.objective, Objective::Cargo { .. }));
        let mut out: Vec<(u32, f64)> =
            hunts.into_iter().map(|(t, o)| (o.id, t.max(self.clock))).collect();
        let open = self
            .ai_missions
            .iter()
            .filter(|m| m.side == side && m.open() && !matches!(m.target, AiTarget::Hunt { .. }))
            .count();
        let mut window = self.launches_in_window(side);
        let mut runway = self.runway(side);
        for (wait, o) in queue.into_iter().take(AI_MISSIONS_MAX.saturating_sub(open)) {
            let n =
                if o.kind == MissionKind::Recon { AI_RECON_AIRCRAFT } else { AI_FLIGHT_AIRCRAFT };
            let mut t = wait.max(runway).max(self.clock);
            let over = (window.len() + n).saturating_sub(AI_AIRCRAFT_PER_WINDOW);
            if over > 0 {
                t = t.max(window[over - 1] + AI_LAUNCH_WINDOW);
            }
            out.push((o.id, t));
            runway = t + n as f64 * AI_STAGGER;
            window.extend(std::iter::repeat_n(t, n));
            window.sort_by(f64::total_cmp);
        }
        out.sort_by(|a, b| a.1.total_cmp(&b.1));
        out
    }

    /// When a side's aircraft took off (or will, a raid ordered) in the last
    /// `AI_LAUNCH_WINDOW`, earliest first.
    fn launches_in_window(&self, side: Side) -> Vec<f64> {
        let since = self.clock - AI_LAUNCH_WINDOW;
        let mut up: Vec<f64> = self
            .ai_missions
            .iter()
            .filter(|m| m.side == side && m.launched > since)
            .flat_map(|m| m.pooled.iter().filter(|&&p| p).map(|_| m.launched))
            .chain(self.raids.iter().filter(|r| r.side == side && !r.drones).flat_map(|r| {
                let at = match r.state {
                    RaidState::Pending { launch_at } => Some(launch_at),
                    _ => r.enroute_at,
                };
                at.filter(|&t| t > since).into_iter().flat_map(|t| [t; RAID_AIRCRAFT])
            }))
            .collect();
        up.sort_by(f64::total_cmp);
        up
    }

    /// When `n` more of a side's aircraft may take off: no more than `AI_AIRCRAFT_PER_WINDOW`
    /// in `AI_LAUNCH_WINDOW` (its raids' too), the first `AI_STAGGER` after the last one.
    pub(crate) fn ai_slot(&self, side: Side, n: usize) -> f64 {
        let up = self.launches_in_window(side);
        let over = (up.len() + n).saturating_sub(AI_AIRCRAFT_PER_WINDOW);
        let window = up.get(over.wrapping_sub(1)).map_or(f64::MIN, |t| t + AI_LAUNCH_WINDOW);
        window.max(self.runway(side))
    }

    /// When a side's next AI aircraft may take off (an AI flight's, a raid's): the last one's
    /// turn plus `AI_STAGGER`.
    pub(crate) fn runway(&self, side: Side) -> f64 {
        self.runway_at[(side == Side::Red) as usize] + AI_STAGGER
    }

    /// `n` of a side's aircraft take their turns on the runway from now: when each takes off.
    pub(crate) fn take_runway(&mut self, side: Side, n: usize) -> Vec<Option<f64>> {
        let first = self.runway(side).max(self.clock);
        let due: Vec<Option<f64>> = (0..n).map(|k| Some(first + k as f64 * AI_STAGGER)).collect();
        if let Some(&Some(last)) = due.last() {
            self.runway_at[(side == Side::Red) as usize] = last;
        }
        due
    }

    /// The AI has a flight on a friendly offer, up or with its turn to take off.
    pub fn ai_scheduled(&self, o: &MissionOffer) -> bool {
        self.ai_mission_for(o.id).is_some_and(AiMission::open)
            || self.ai_rush.contains(&o.id)
            || self.ai_launch_at(o).is_some()
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
            // Its cargo flight landed: nothing left to hunt, it comes home.
            if let AiTarget::Hunt { tag, .. } = self.ai_missions[k].target
                && !self.ai_missions[k].achieved
                && self.cargo_run(tag).is_none()
                && !self.cargo_lost.contains(&tag)
            {
                let m = &mut self.ai_missions[k];
                let home: Vec<u32> = m.flying().collect();
                ev.extend(home.into_iter().map(|tag| CampaignEvent::Recall { tag }));
                // The ones still waiting for their turn stay on the ground.
                let (side, heli) = (m.side, m.airframe.helicopter());
                let mut back = 0;
                for j in 0..m.tags.len() {
                    if m.due.get(j).copied().flatten().is_some() {
                        m.due[j] = None;
                        m.alive[j] = false;
                        back += m.pooled[j] as u32;
                    }
                }
                self.aircraft_back(side, if heli { 0 } else { back }, if heli { back } else { 0 });
                self.end_ai_mission(k);
                continue;
            }
            let m = &self.ai_missions[k];
            let gone = m.alive.iter().all(|a| !a) && m.controlled.iter().all(Option::is_none);
            if gone || now - m.launched > AI_MISSION_TIMEOUT {
                self.end_ai_mission(k);
            }
        }
        self.ai_missions
            .retain(|m| m.open() || m.ended.is_some_and(|t| now - t < AI_MISSION_MEMORY));
        // The aircraft whose turn it is take off.
        for m in self.ai_missions.iter_mut().filter(|m| m.open()) {
            let up: Vec<u32> = (0..m.tags.len())
                .filter(|&k| m.due.get(k).copied().flatten().is_some_and(|t| now >= t))
                .map(|k| m.tags[k])
                .collect();
            if up.is_empty() {
                continue;
            }
            for d in m.due.iter_mut().filter(|d| d.is_some_and(|t| now >= t)) {
                *d = None;
            }
            ev.push(CampaignEvent::AiMissionLaunched {
                id: m.id,
                side: m.side,
                from: m.from,
                airframe: m.airframe,
                target: m.target,
                tags: up,
            });
        }
        if !self.ai_missions_on() || self.outcome.is_some() {
            return;
        }
        // The offers a player asked for at once, out of the queue (no waiting for a turn in
        // `AI_LAUNCH_WINDOW`).
        for id in std::mem::take(&mut self.ai_rush) {
            if self.ai_mission_for(id).is_some_and(AiMission::open) {
                continue;
            }
            if let Some(o) = self.board.iter().find(|o| o.id == id).cloned() {
                self.launch_ai(Side::Blue, &o, view);
            }
        }
        // The offers whose turn it is (the queue's first; the next ones come later).
        for side in [Side::Blue, Side::Red] {
            for (id, t) in self.ai_schedule(side) {
                if now < t {
                    break;
                }
                let board = if side == Side::Blue { &self.board } else { &self.red_board };
                let Some(o) = board.iter().find(|o| o.id == id).cloned() else { continue };
                if !self.launch_ai(side, &o, view) {
                    // Not enough aircraft or materiel: the ones after it wait too.
                    break;
                }
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
    fn launch_ai(&mut self, side: Side, o: &MissionOffer, view: &WorldView) -> bool {
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
        // A cargo aircraft is the jets' to catch.
        let close = d <= AI_HELO_RANGE && !matches!(o.objective, Objective::Cargo { .. });
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
        // Each in its turn on the runway, `AI_STAGGER` apart.
        let due = self.take_runway(side, n);
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
            due,
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
            Objective::Cargo { tag } => {
                self.cargo_run(*tag)?;
                AiTarget::Hunt { tag: *tag, x: o.x, y: o.y }
            }
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
        for (title, objective, x, y, threat) in self.cargo_offers(Side::Red) {
            add(self, MissionKind::Intercept, title, objective, x, y, threat);
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
        // Its aircraft for the quarter hour (and the runway) when the raid would take off.
        if self.ai_slot(Side::Red, RAID_AIRCRAFT) > now + RAID_LEAD {
            return None;
        }
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

    /// After loading a save: the AI flights that were in the air are back on the ground (their
    /// offers go up again `AI_MISSION_WAIT` from now).
    pub(crate) fn ground_ai_missions(&mut self) {
        let now = self.clock;
        let mut back = Vec::new();
        for m in self.ai_missions.iter_mut().filter(|m| m.open()) {
            for k in 0..m.tags.len() {
                if m.alive[k] && m.pooled[k] {
                    back.push((m.side, m.airframe.helicopter()));
                }
                m.alive[k] = false;
                m.controlled[k] = None;
            }
            m.state = AiMissionState::Done;
            m.ended = Some(now);
        }
        for (side, heli) in back {
            self.aircraft_back(side, !heli as u32, heli as u32);
        }
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
        // A cargo run is flown to its base by the one who took it.
        if matches!(s.offer.objective, Objective::Airlift { .. }) {
            return Err(text!("ce-airlift-no-handover"));
        }
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
                            due: Vec::new(),
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
                m.due.push(None);
                (k, tag)
            }
        };
        let m = &self.ai_missions[k];
        // Done with it, or a mission the AI does not fly (insertions, rescues, cargo): the AI
        // flies it home.
        let home = s.progress.achieved || m.achieved || !ai_flown(s.offer.kind);
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
