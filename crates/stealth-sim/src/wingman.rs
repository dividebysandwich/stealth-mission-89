//! The player's AI wingman: an aircraft of his own type that joins him once he is up and flies
//! formation on him (spread abeam, or in trail), defends him and itself against the enemy's
//! aircraft and (with HARMs) the SAM sites that fire at them, and takes his radio calls:
//! engage my target, engage SAMs, cover me, rejoin, RTB, go trail / spread.
//!
//! It is one of the campaign's AI flights (in a single mission too, hosted by the campaign's
//! bookkeeping as an escort is, [`Sim::launch_escorts`]), with a [`Wing`] that says whose
//! wingman it is and what it was told. Its missiles come from its lead's air-to-air stores, its
//! anti-radiation shots from his HARMs and its attacks on the ground from his other stores.
//!
//! Close to its station it keeps it as a formation pilot does: on the station point, moving
//! with its lead ([`Sim::keep_station`]); farther off (joining up, after a fight) it flies there
//! as the AI flies anywhere.

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::ai_loadout::{AIM9, AIM120, AiLoadout, Seeker, Store};
use crate::ai_missions::AiTask;
use crate::campaign::Role;
use crate::math::{approx, s16};
use crate::types::*;
use crate::{AircraftKind, ENEMY_SLOTS, Sim};

/// Campaign tag of pilot `k`'s wingman: this plus `k`.
pub const WING_TAG: u32 = 0xE5C1_0000;

/// World units per metre.
const WU_PER_M: f64 = 32.0 / crate::aaa::M_PER_MAP;
/// Station off the lead (m: right, back): a jet's and a helicopter's, spread and in trail.
const SPREAD_JET: (f64, f64) = (400.0, 150.0);
const TRAIL_JET: (f64, f64) = (0.0, 1200.0);
const SPREAD_HELI: (f64, f64) = (120.0, 40.0);
const TRAIL_HELI: (f64, f64) = (0.0, 300.0);
/// Within this (m) of its station it keeps it as a formation pilot does (jet, helicopter).
const SNAP_JET: f64 = 1500.0;
const SNAP_HELI: f64 = 400.0;
/// How fast it closes on its station there (1/s).
const STATION_GAIN: f64 = 0.8;
/// It joins up from this far (m) behind its station, once its lead is this high (ft over the
/// ground: a jet, a helicopter).
const JOIN_BACK: f64 = 1500.0;
const JOIN_JET_FT: i32 = 300;
const JOIN_HELI_FT: i32 = 40;
/// The enemy aircraft it goes for round its lead (map units): defending him in formation,
/// covering him, and the target it was told to engage.
const DEFEND_BUBBLE: i32 = 0x200;
const COVER_BUBBLE: i32 = 0x500;
const ENGAGE_BUBBLE: i32 = 0x900;
/// How far (map units) from its lead an enemy SAM site may be for it to go for it.
const SAM_REACH: i32 = 0x800;
/// Its lead down this long (s): it goes home.
const LEAD_DOWN_SECS: i32 = 20;
/// Ground attacks it makes with each store of a weapon (at most).
const SHOTS_PER_STATION: u8 = 2;

/// What the wingman was told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WingOrder {
    /// On its station, fighting off what comes at its lead.
    #[default]
    Formation,
    /// Its lead's target (an aircraft, a ground object or point).
    Engage,
    /// An enemy SAM site with a HARM.
    Sams,
    /// Ranges wider round its lead after the enemy's aircraft.
    Cover,
    /// Home to land.
    Rtb,
}

/// Where its station is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Form {
    /// Abeam on the right, a little back.
    #[default]
    Spread,
    /// Behind.
    Trail,
}

/// A wingman's record (its [`crate::campaign::CampaignFlight`]'s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Wing {
    /// Its lead (pilot id).
    pub lead: u8,
    pub order: WingOrder,
    pub form: Form,
    /// HARMs and ground attacks left.
    pub harms: u8,
    pub ground: u8,
    /// The aircraft it was told to engage (flight unit).
    pub air: Option<u16>,
    /// Frames its lead has been on the ground (or out of the world).
    pub lead_down: i32,
}

/// A wingman waiting for its lead to get up.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PendingWing {
    pub lead: u8,
    /// Its AI aircraft type and home base (object index).
    pub spec: i32,
    pub home: i32,
    /// Its stores (weapon type, rounds per station; none: its lead's when it joins).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stores: Vec<(i32, i32)>,
}

/// A radio call to the wingman.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WingCmd {
    Engage,
    Sams,
    Cover,
    Rejoin,
    Rtb,
    Trail,
    Spread,
}

impl WingCmd {
    pub const ALL: [WingCmd; 7] = [
        WingCmd::Engage,
        WingCmd::Sams,
        WingCmd::Cover,
        WingCmd::Rejoin,
        WingCmd::Rtb,
        WingCmd::Trail,
        WingCmd::Spread,
    ];
}

/// The wingman's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WingReply {
    /// Doing it.
    Copy,
    /// Going for the target.
    Engaging,
    /// Nothing locked to engage.
    NoTarget,
    /// No HARM left, or no SAM site in reach.
    NoHarm,
    NoSam,
    /// Nothing left to attack the ground with.
    Winchester,
    /// No wingman (not up yet, or gone).
    NoWingman,
}

impl WingReply {
    /// The reply's message id (sim.ftl).
    pub fn id(self) -> &'static str {
        match self {
            WingReply::Copy => "wing-reply-copy",
            WingReply::Engaging => "wing-reply-engaging",
            WingReply::NoTarget => "wing-reply-no-target",
            WingReply::NoHarm => "wing-reply-no-harm",
            WingReply::NoSam => "wing-reply-no-sam",
            WingReply::Winchester => "wing-reply-winchester",
            WingReply::NoWingman => "wing-reply-none",
        }
    }
}

/// The lead's position for station keeping: wu east, wu south, viewZ, heading, speed (wu/s).
#[derive(Debug, Clone, Copy)]
struct Lead {
    wx: f64,
    wy: f64,
    alt: i32,
    heading: i32,
    pitch: i32,
    roll: i32,
    speed: f64,
}

/// The air-to-air missiles, HARMs and ground attacks a wingman carries for its lead's stores
/// (weapon type, rounds per station).
fn wing_stores(sim: &Sim, bays: &[(i32, i32)]) -> (AiLoadout, u8, u8) {
    let (mut radar, mut ir, mut harms, mut ground) = (0, 0, 0u8, 0u8);
    for &(w, n) in bays.iter().filter(|b| b.0 >= 0 && b.1 > 0) {
        match w {
            1 => radar += n,
            0 | 21 => ir += n,
            2 => harms = harms.saturating_add(n as u8),
            // The camera, tanks, cargo, paratroopers, decoys and jammers attack nothing.
            16 | 17 | 18 | 27 | 28 | 29 => {}
            _ if sim.weapon_types.get(w as usize).is_some_and(|t| t.spec != 0)
                || crate::rockets::rocket_weapon(w) =>
            {
                ground = ground.saturating_add(SHOTS_PER_STATION.min(n as u8));
            }
            _ => {}
        }
    }
    let mut l = AiLoadout::default();
    l.stores[0] = Store { missile: AIM120, count: radar, seeker: Seeker::Radar };
    l.stores[1] = Store { missile: AIM9, count: ir, seeker: Seeker::Ir };
    (l, harms, ground)
}

impl Sim {
    /// Pilot `lead` takes a wingman of AI aircraft type `spec`: it joins him once he is up.
    pub fn request_wingman(&mut self, lead: u8, spec: i32) {
        self.request_wingman_with(lead, spec, Vec::new());
    }

    /// [`Sim::request_wingman`] with the wingman's own stores (weapon type, rounds per station:
    /// what its base had for it).
    pub fn request_wingman_with(&mut self, lead: u8, spec: i32, stores: Vec<(i32, i32)>) {
        if self.campaign.is_none() {
            self.campaign = Some(Box::new(self.campaign_state([0; 4], 0)));
        }
        self.pending_wing.retain(|p| p.lead != lead);
        let home = self.pilot_ref(lead as usize).targets[0].base;
        self.pending_wing.push(PendingWing { lead, spec, home, stores });
    }

    /// The flight unit of pilot `lead`'s wingman (alive).
    pub fn wingman_of(&self, lead: u8) -> Option<usize> {
        let c = self.campaign.as_ref()?;
        c.flights.iter().position(|f| f.is_some_and(|f| f.wing.is_some_and(|w| w.lead == lead)))
    }

    /// Pilot `lead`'s wingman's record.
    pub fn wing(&self, lead: u8) -> Option<Wing> {
        let i = self.wingman_of(lead)?;
        self.campaign.as_ref()?.flights[i]?.wing
    }

    /// Pilot `lead` still waits for his wingman to join him.
    pub fn wingman_pending(&self, lead: u8) -> bool {
        self.pending_wing.iter().any(|p| p.lead == lead)
    }

    fn lead(&self, k: u8) -> Lead {
        let p = &self.pilot_ref(k as usize).p;
        Lead {
            wx: p.wx as f64,
            wy: (0x10_0000 - p.wy) as f64,
            alt: p.view_z,
            heading: p.heading,
            pitch: p.pitch,
            roll: p.roll,
            speed: p.velocity as f64 / 16.0,
        }
    }

    /// The station (wu east, wu south, viewZ) of aircraft type `spec` in form `form` on `l`,
    /// `back` m further behind.
    fn station(&self, l: &Lead, form: Form, spec: i32, back: f64) -> (f64, f64, i32) {
        let heli = self.aircraft_kind(spec) == AircraftKind::Helicopter;
        let (right, aft) = match (form, heli) {
            (Form::Spread, false) => SPREAD_JET,
            (Form::Trail, false) => TRAIL_JET,
            (Form::Spread, true) => SPREAD_HELI,
            (Form::Trail, true) => TRAIL_HELI,
        };
        let h = (l.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        // Forward is east sin h, south -cos h; right is east cos h, south sin h.
        let (fe, fs) = (h.sin(), -h.cos());
        let (re, rs) = (h.cos(), h.sin());
        let aft = (aft + back) * WU_PER_M;
        let right = right * WU_PER_M;
        (l.wx + re * right - fe * aft, l.wy + rs * right - fs * aft, l.alt)
    }

    /// Lead and wingman are in the air and the lead is high enough: the wingman joins.
    fn join_wingmen(&mut self) {
        let mut k = 0;
        while k < self.pending_wing.len() {
            let p = self.pending_wing[k].clone();
            let pilot = self.pilot_ref(p.lead as usize);
            if pilot.id != p.lead || pilot.absent || pilot.eject != 0 {
                k += 1;
                continue;
            }
            let heli = self.aircraft_kind(p.spec) == AircraftKind::Helicopter;
            let high = if heli { JOIN_HELI_FT } else { JOIN_JET_FT };
            if pilot.on_ground || pilot.p.height < high {
                k += 1;
                continue;
            }
            self.pending_wing.remove(k);
            self.spawn_wingman(p);
        }
    }

    /// Wingman `p` joins its lead from behind its station.
    fn spawn_wingman(&mut self, p: PendingWing) -> usize {
        let l = self.lead(p.lead);
        let (wx, wy, alt) = self.station(&l, Form::Spread, p.spec, JOIN_BACK);
        let t = self.ac_type(p.spec).clone();
        let i = self.campaign_slot();
        let (wx, wy) = (wx.round() as i32, wy.round() as i32);
        let heli = t.kind == AircraftKind::Helicopter;
        let speed = l.speed.round() as i32;
        let mut u = FlightUnit {
            spec: p.spec,
            wx,
            wy,
            x: (wx >> 5) & 0xFFFF,
            y: (wy >> 5) & 0xFFFF,
            alt: alt.max(20),
            heading: s16(l.heading),
            speed: speed.max(if heli { 1 } else { t.max_speed as i32 / 2 }),
            flags: fflags::EXISTS | fflags::AIRBORNE,
            wp: p.home,
            // Fuel for the whole mission (as an escort's): the longest the timer runs.
            timer: 0x7FFF,
            ..Default::default()
        };
        let bays = if p.stores.is_empty() {
            self.pilot_ref(p.lead as usize).bays.clone()
        } else {
            p.stores.clone()
        };
        let (loadout, harms, ground) = wing_stores(self, &bays);
        let tag = WING_TAG + p.lead as u32;
        let mut f =
            self.mission_flight(p.spec, AiTask::Patrol(u.x, u.y), tag, true, p.home as usize);
        f.loadout = loadout;
        f.wing = Some(Wing { lead: p.lead, harms, ground, ..Default::default() });
        u.missile = f.classic_missile();
        self.flights[i] = u;
        if self.ai_flight.len() > i {
            self.ai_flight[i] = Default::default();
        }
        self.campaign.as_mut().expect("campaign").flights[i] = Some(f);
        let back = self.pl.id as usize;
        self.activate(p.lead as usize);
        self.msg(text!("wing-joined"));
        self.activate(back);
        i
    }

    /// Where the wingman's lead wants the enemy's aircraft engaged (around his position: map
    /// units, radius), and the aircraft he told it to engage (for target selection).
    pub(crate) fn wing_bubble(&self, w: &Wing) -> ((i32, i32), i32, Option<usize>) {
        let p = &self.pilot_ref(w.lead as usize).p;
        let r = match w.order {
            WingOrder::Formation => DEFEND_BUBBLE,
            WingOrder::Cover => COVER_BUBBLE,
            WingOrder::Engage => ENGAGE_BUBBLE,
            WingOrder::Sams | WingOrder::Rtb => DEFEND_BUBBLE / 2,
        };
        let air = w.air.map(usize::from).filter(|&j| self.wing_enemy(j));
        ((p.mx, p.my), r, air)
    }

    /// Flight unit `j` is the enemy's, alive and in the air.
    fn wing_enemy(&self, j: usize) -> bool {
        self.flights.get(j).is_some_and(|u| {
            u.flags & fflags::DESTROYED == 0 && u.flags & fflags::AIRBORNE != 0 && u.alt > 0
        }) && self.ai_side(j) == Some(false)
    }

    /// The station point a wingman in formation flies to (map units) and its altitude (viewZ).
    pub(crate) fn wing_goal(&self, i: usize) -> Option<(i32, i32, i32)> {
        let f = self.campaign.as_ref()?.flights.get(i)?.as_ref()?;
        let w = f.wing?;
        if f.target.is_some() {
            return None;
        }
        self.wing_goal_of(i, &w)
    }

    fn wing_goal_of(&self, i: usize, w: &Wing) -> Option<(i32, i32, i32)> {
        if !matches!(w.order, WingOrder::Formation | WingOrder::Cover) {
            return None;
        }
        let l = self.lead(w.lead);
        // A little ahead of the station, so it flies along with its lead.
        let (x, y, z) = self.station(&l, w.form, self.flights[i].spec, -300.0);
        Some((((x as i32) >> 5) & 0xFFFF, ((y as i32) >> 5) & 0xFFFF, z))
    }

    /// The speed (wu/s) a wingman flying to its station holds: its lead's, more when behind.
    pub(crate) fn wing_speed(&self, i: usize, u: &FlightUnit) -> Option<f64> {
        let f = self.campaign.as_ref()?.flights.get(i)?.as_ref()?;
        let w = f.wing?;
        if !matches!(w.order, WingOrder::Formation | WingOrder::Cover) || f.target.is_some() {
            return None;
        }
        let l = self.lead(w.lead);
        let (x, y, _) = self.station(&l, w.form, u.spec, 0.0);
        let h = (l.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        // Behind its station (along the lead's track): faster, ahead: slower.
        let behind = (x - u.wx as f64) * h.sin() - (y - u.wy as f64) * h.cos();
        let v = l.speed.max(1.0);
        Some((v + behind * 0.2).clamp(v * 0.7, v * 1.6))
    }

    /// A wingman in formation close to its station keeps it: it moves with its lead onto the
    /// station point, on his heading and attitude.
    fn keep_station(&mut self, i: usize, w: &Wing) {
        let mut u = self.flights[i];
        let l = self.lead(w.lead);
        let (x, y, z) = self.station(&l, w.form, u.spec, 0.0);
        let heli = self.aircraft_kind(u.spec) == AircraftKind::Helicopter;
        let snap = if heli { SNAP_HELI } else { SNAP_JET } * WU_PER_M;
        let (dx, dy) = (x - u.wx as f64, y - u.wy as f64);
        if dx.hypot(dy) > snap {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let k = (STATION_GAIN * dt).min(1.0);
        let h = (l.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        // Where it was before the AI moved it this frame (along its own heading at its speed),
        // then where its lead's motion takes the station this frame, and onto it.
        let hu = (u.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let own = u.speed as f64 * dt;
        let (px, py) = (u.wx as f64 - hu.sin() * own, u.wy as f64 + hu.cos() * own);
        let (dx, dy) = (x - px, y - py);
        let mv = l.speed * dt;
        let nx = px + h.sin() * mv + dx * k;
        let ny = py - h.cos() * mv + dy * k;
        u.wx = nx.round() as i32;
        u.wy = ny.round() as i32;
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.alt += ((z - u.alt) as f64 * k).round() as i32;
        let turn = s16(l.heading - u.heading);
        u.heading = s16(u.heading + (turn as f64 * (k * 4.0).min(1.0)).round() as i32);
        u.pitch = l.pitch;
        u.bank = l.roll;
        u.speed = (l.speed.round() as i32).max(1);
        u.flags |= fflags::AIRBORNE;
        u.flags &= !fflags::RTB;
        self.flights[i] = u;
    }

    /// The enemy SAM site nearest pilot `lead` that has a missile in the air at him or at
    /// flight unit `wing` (object index).
    fn sam_firing_at(&self, lead: u8, _wing: usize) -> Option<usize> {
        (0..ENEMY_SLOTS)
            .filter(|&s| self.proj[s].ttl != 0 && self.proj[s].launcher > 2)
            .filter(|&s| self.proj_pilot[s] == lead || self.proj[s].target_pilot == Some(lead))
            .map(|s| self.proj[s].launcher as usize)
            .find(|&o| self.enemy_site(o))
    }

    /// Object `o` is an enemy SAM site still standing.
    fn enemy_site(&self, o: usize) -> bool {
        self.objects.get(o).is_some_and(|ob| {
            ob.unit_type != 0
                && ob.flags & (oflags::FRIENDLY | oflags::DESTROYED | oflags::AIRFIELD) == 0
        })
    }

    /// The enemy SAM site nearest pilot `lead` within reach (object index), the active ones
    /// first.
    fn nearest_sam(&self, lead: u8) -> Option<usize> {
        let p = &self.pilot_ref(lead as usize).p;
        (1..self.objects.len())
            .filter(|&o| self.enemy_site(o))
            .map(|o| {
                let ob = &self.objects[o];
                let d = approx(ob.x - p.mx, ob.y - p.my);
                (o, if ob.alert > 0 { d } else { d + SAM_REACH }, d)
            })
            .filter(|&(_, _, d)| d < SAM_REACH)
            .min_by_key(|&(_, key, _)| key)
            .map(|(o, _, _)| o)
    }

    /// The wingman of flight unit `i` goes for object `o` (a SAM site with a HARM, else with
    /// its ground weapons), or the enemy's vehicles at point `at` (map units).
    fn wing_strike(&mut self, i: usize, o: Option<usize>, at: (i32, i32), order: WingOrder) {
        let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) else { return };
        f.role = Role::Strike;
        f.target = None;
        f.goal = at;
        match o {
            Some(o) => {
                f.strike = Some(o as i32);
                f.strike_unit = false;
            }
            None => {
                f.strike = Some(0);
                f.strike_unit = true;
                f.passes = 1;
                f.pass_wait = 0;
            }
        }
        f.strike_aaa = false;
        if let Some(w) = f.wing.as_mut() {
            w.order = order;
            w.air = None;
        }
        self.flights[i].flags &= !fflags::RTB;
    }

    /// Back to its lead's side, its strike (if any) called off.
    fn wing_rejoin(&mut self, i: usize) {
        let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) else { return };
        f.role = Role::Fighter;
        f.strike = None;
        f.strike_unit = false;
        f.target = None;
        if let Some(w) = f.wing.as_mut() {
            w.order = WingOrder::Formation;
            w.air = None;
        }
        self.flights[i].flags &= !fflags::RTB;
    }

    /// One frame of the wingmen: joining up, keeping station, their orders done and the
    /// stores they used.
    pub(crate) fn update_wingmen(&mut self) {
        if self.campaign.is_none() {
            return;
        }
        self.join_wingmen();
        let wings: Vec<(usize, Wing)> = self
            .campaign
            .as_ref()
            .map(|c| {
                c.flights
                    .iter()
                    .enumerate()
                    .filter_map(|(i, f)| Some((i, f.as_ref()?.wing?)))
                    .collect()
            })
            .unwrap_or_default();
        let fs = self.fs.max(1);
        for (i, mut w) in wings {
            let u = self.flights[i];
            if u.flags & fflags::DESTROYED != 0 || u.flags & fflags::EXISTS == 0 {
                continue;
            }
            // Its lead down or gone a while: home.
            let pilot = self.pilot_ref(w.lead as usize);
            let away = pilot.id != w.lead || pilot.absent || pilot.eject != 0;
            w.lead_down = if away || pilot.on_ground { w.lead_down + 1 } else { 0 };
            if w.lead_down > LEAD_DOWN_SECS * fs && w.order != WingOrder::Rtb {
                w.order = WingOrder::Rtb;
                self.wing_rejoin(i);
                self.recall_flight(WING_TAG + w.lead as u32);
            }
            let f = self.campaign.as_ref().and_then(|c| c.flights[i]).expect("wingman");
            match w.order {
                WingOrder::Rtb => {
                    self.flights[i].flags |= fflags::RTB;
                }
                WingOrder::Engage | WingOrder::Sams if f.role == Role::Strike => {
                    // Its shot away (or the target gone): it uses up a store and comes back.
                    let gone = f.strike.is_some_and(|o| {
                        !f.strike_unit && o > 0 && !self.enemy_target_object(o as usize)
                    });
                    if f.strike.is_none() || gone {
                        if f.strike.is_none() {
                            if w.order == WingOrder::Sams {
                                w.harms = w.harms.saturating_sub(1);
                            } else {
                                w.ground = w.ground.saturating_sub(1);
                            }
                        }
                        self.wing_rejoin(i);
                        w.order = WingOrder::Formation;
                        w.air = None;
                    } else {
                        self.flights[i].flags &= !fflags::RTB;
                    }
                }
                WingOrder::Engage if w.air.is_some() => {
                    let j = w.air.map(usize::from).filter(|&j| self.wing_enemy(j));
                    if j.is_none() {
                        w.order = WingOrder::Formation;
                        w.air = None;
                    }
                    if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
                        f.target = j;
                    }
                }
                _ => {
                    // In formation: a SAM site firing at its lead or at it gets a HARM.
                    if w.order == WingOrder::Engage {
                        w.order = WingOrder::Formation;
                    }
                    if w.harms > 0
                        && let Some(o) = self.sam_firing_at(w.lead, i)
                    {
                        let at = (self.objects[o].x, self.objects[o].y);
                        self.wing_strike(i, Some(o), at, WingOrder::Sams);
                        w.order = WingOrder::Sams;
                    } else if f.target.is_none() {
                        self.keep_station(i, &w);
                    }
                }
            }
            // In formation: its goal the station (the AI flies there when it is far off).
            let goal = self.wing_goal_of(i, &w);
            if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
                if let Some(g) = goal.filter(|_| f.target.is_none() && f.role == Role::Fighter) {
                    f.goal = (g.0, g.1);
                }
                f.wing = Some(w);
            }
        }
    }

    /// Object `o` is the enemy's and still standing (a target for the wingman).
    fn enemy_target_object(&self, o: usize) -> bool {
        self.objects.get(o).is_some_and(|ob| ob.flags & (oflags::FRIENDLY | oflags::DESTROYED) == 0)
    }

    /// Pilot `lead`'s radio call to his wingman: it answers him ([`SimEvent::WingReply`]).
    pub fn wing_command(&mut self, lead: u8, cmd: WingCmd) -> WingReply {
        let reply = self.wing_order(lead, cmd);
        self.wing_replies.push((lead, reply));
        reply
    }

    fn wing_order(&mut self, lead: u8, cmd: WingCmd) -> WingReply {
        let Some(i) = self.wingman_of(lead) else { return WingReply::NoWingman };
        let Some(w) = self.campaign.as_ref().and_then(|c| c.flights[i]).and_then(|f| f.wing) else {
            return WingReply::NoWingman;
        };
        let set = |sim: &mut Sim, f: &dyn Fn(&mut Wing)| {
            if let Some(Some(cf)) = sim.campaign.as_mut().map(|c| &mut c.flights[i])
                && let Some(w) = cf.wing.as_mut()
            {
                f(w);
            }
        };
        match cmd {
            WingCmd::Spread | WingCmd::Trail => {
                let form = if cmd == WingCmd::Spread { Form::Spread } else { Form::Trail };
                set(self, &|w| w.form = form);
                WingReply::Copy
            }
            WingCmd::Rejoin => {
                self.wing_rejoin(i);
                WingReply::Copy
            }
            WingCmd::Cover => {
                self.wing_rejoin(i);
                set(self, &|w| w.order = WingOrder::Cover);
                WingReply::Copy
            }
            WingCmd::Rtb => {
                self.wing_rejoin(i);
                set(self, &|w| w.order = WingOrder::Rtb);
                self.recall_flight(WING_TAG + lead as u32);
                WingReply::Copy
            }
            WingCmd::Sams => {
                if w.harms == 0 {
                    return WingReply::NoHarm;
                }
                let Some(o) = self.sam_firing_at(lead, i).or_else(|| self.nearest_sam(lead)) else {
                    return WingReply::NoSam;
                };
                let at = (self.objects[o].x, self.objects[o].y);
                self.wing_strike(i, Some(o), at, WingOrder::Sams);
                WingReply::Engaging
            }
            WingCmd::Engage => self.wing_engage(lead, i, &w),
        }
    }

    /// "Engage my target": the lead's air lock, else his ground lock or designated point.
    fn wing_engage(&mut self, lead: u8, i: usize, w: &Wing) -> WingReply {
        let back = self.pl.id as usize;
        self.activate(lead as usize);
        let air = Some(self.pl.air_lock)
            .filter(|&j| j >= 0 && self.wing_enemy(j as usize))
            .map(|j| j as usize);
        let ground = self.valid_gnd();
        let point = self.pl.spi.or_else(|| self.pl.last_input.pod.and_then(|p| p.ground));
        self.activate(back);
        if let Some(j) = air {
            self.wing_rejoin(i);
            if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
                f.target = Some(j);
                if let Some(fw) = f.wing.as_mut() {
                    fw.order = WingOrder::Engage;
                    fw.air = Some(j as u16);
                }
            }
            return WingReply::Engaging;
        }
        if w.ground == 0 {
            return if ground.is_some() || point.is_some() {
                WingReply::Winchester
            } else {
                WingReply::NoTarget
            };
        }
        if let Some(o) = ground.filter(|&o| o > 0 && self.enemy_target_object(o)) {
            let at = (self.objects[o].x, self.objects[o].y);
            self.wing_strike(i, Some(o), at, WingOrder::Engage);
            return WingReply::Engaging;
        }
        if let Some(at) = point {
            self.wing_strike(i, None, at, WingOrder::Engage);
            return WingReply::Engaging;
        }
        WingReply::NoTarget
    }
}
