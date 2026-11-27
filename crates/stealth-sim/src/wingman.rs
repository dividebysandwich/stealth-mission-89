//! The player's AI wingman: an aircraft of his own type that joins him once he is up and flies
//! formation on him (spread abeam, or in trail), defends him and itself against the enemy's
//! aircraft and (with anti-radiation missiles: HARMs, Shrikes, Sidearms) the SAM sites that fire
//! at them, and takes his radio calls: engage my target, engage SAMs (the site firing at them,
//! else the best radar ahead), cover me, rejoin, RTB, go trail / spread.
//!
//! It is one of the campaign's AI flights (in a single mission too, hosted by the campaign's
//! bookkeeping as an escort is, [`Sim::launch_escorts`]), with a [`Wing`] that says whose
//! wingman it is and what it was told. Its missiles come from its lead's air-to-air stores, its
//! anti-radiation shots from his anti-radiation missiles and its attacks on the ground from his
//! other stores.
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
const SNAP_HELI: f64 = 250.0;
/// How fast it closes on its station there (1/s), at most this part of its lead's speed and
/// this much more (m/s: a jet, a helicopter).
const STATION_GAIN: f64 = 0.8;
const CLOSE_MAX: f64 = 0.15;
const CLOSE_MIN: f64 = 10.0;
const CLOSE_MIN_HELI: f64 = 15.0;
/// A helicopter wingman farther off flies for where its station will be this long (s, at
/// most) on, this much faster than its lead per wu off it (1/s), up to this part over its own
/// top speed or its lead's (it catches up with a lead faster than the AI's helicopters fly).
const HELI_LEAD_S: f64 = 20.0;
const HELI_CLOSE_GAIN: f64 = 0.05;
const HELI_DASH: f64 = 1.25;
/// It holds its lead's height over the ground, at least this high (altitude units) over the
/// ground under it and these far ahead (m), climbing and sinking at most this fast (units/s).
const HELI_MIN_AGL: i32 = 40;
const HELI_LOOK: [f64; 3] = [0.0, 150.0, 400.0];
const HELI_CLIMB: f64 = 40.0;
/// A helicopter wingman goes for an enemy aircraft only this close to it (map units, unless
/// told to engage it), and only with missiles to fire at it.
pub(crate) const HELI_AIR_REACH: i32 = 0x100;
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
/// "Engage SAMs" takes a site this far either side of its lead's heading (BAM, 75°).
const AHEAD: i32 = 0x3555;
/// Map units added to a site's distance when weighing it (sites much closer than this count as
/// about as close), ~6 km.
const SAM_NEAR: i32 = 0x180;
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
    /// An enemy SAM site with an anti-radiation missile.
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

/// Where the wingman is with its lead: up with him, or on the ground and in the circuit with
/// him ([`crate::wing_ground`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WingPhase {
    /// In the air: formation, its orders.
    #[default]
    Air,
    /// On a carrier's catapult, launched after its lead.
    Catapult,
    /// On the runway with its lead, rolling and lifting off with him.
    Takeoff,
    /// Landing with him: beside him, or behind him on his track.
    Landing,
    /// Stopped after the landing.
    Parked,
}

impl WingPhase {
    /// It flies its lead's track ([`crate::wing_ground`]), not as the AI flies.
    pub fn on_track(self) -> bool {
        matches!(self, WingPhase::Takeoff | WingPhase::Landing | WingPhase::Parked)
    }
}

/// A wingman's record (its [`crate::campaign::CampaignFlight`]'s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Wing {
    /// Its lead (pilot id).
    pub lead: u8,
    pub order: WingOrder,
    pub form: Form,
    /// Anti-radiation missiles and ground attacks left.
    pub harms: u8,
    pub ground: u8,
    /// The missile (type) of its anti-radiation shots: its lead's HARMs, Shrikes or Sidearms (0:
    /// a HARM, as before there were others).
    #[serde(default)]
    pub arm: i32,
    /// The aircraft it was told to engage (flight unit).
    pub air: Option<u16>,
    /// Frames its lead has been on the ground (or out of the world).
    pub lead_down: i32,
    #[serde(default)]
    pub phase: WingPhase,
}

/// A wingman waiting for its lead (on the ground: it starts beside him; in the air: it joins
/// him once he is up).
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
    /// No anti-radiation missile left, or no SAM site in reach.
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

/// The air-to-air missiles, anti-radiation missiles (and their missile type: the farthest
/// reaching of them) and ground attacks a wingman carries for its lead's stores (weapon type,
/// rounds per station).
fn wing_stores(sim: &Sim, bays: &[(i32, i32)]) -> (AiLoadout, u8, i32, u8) {
    let (mut radar, mut ir, mut stingers, mut harms, mut ground) = (0, 0, 0, 0u8, 0u8);
    let mut arm = 0;
    for &(w, n) in bays.iter().filter(|b| b.0 >= 0 && b.1 > 0) {
        match w {
            1 => radar += n,
            0 => ir += n,
            21 => stingers += n,
            HARM_WEAPON | SHRIKE_WEAPON | SIDEARM_WEAPON => {
                harms = harms.saturating_add(n as u8);
                let spec = sim.weapon_types.get(w as usize).map_or(0, |t| t.spec as i32);
                if arm == 0 || sim.missile(spec).lock_range > sim.missile(arm).lock_range {
                    arm = spec;
                }
            }
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
    l.stores[2] = Store { missile: STINGER_SPEC, count: stingers, seeker: Seeker::Ir };
    (l, harms, arm, ground)
}

impl Sim {
    /// Pilot `lead` takes a wingman of AI aircraft type `spec`: on the runway with him (it
    /// takes off with him), else it joins him once he is up.
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

    /// The lead on the runway: the wingman starts beside him. In the air and high enough: the
    /// wingman joins him.
    fn join_wingmen(&mut self) {
        let mut k = 0;
        while k < self.pending_wing.len() {
            let p = self.pending_wing[k].clone();
            let pilot = self.pilot_ref(p.lead as usize);
            if pilot.id != p.lead || pilot.absent || pilot.eject != 0 {
                k += 1;
                continue;
            }
            if pilot.on_ground {
                if self.spawn_wingman_on_ground(&p) {
                    self.pending_wing.remove(k);
                } else {
                    k += 1;
                }
                continue;
            }
            let heli = self.aircraft_kind(p.spec) == AircraftKind::Helicopter;
            let high = if heli { JOIN_HELI_FT } else { JOIN_JET_FT };
            if pilot.on_ground || pilot.p.height < high {
                k += 1;
                continue;
            }
            self.pending_wing.remove(k);
            let l = self.lead(p.lead);
            let at = self.station(&l, Form::Spread, p.spec, JOIN_BACK);
            self.spawn_wingman_at(p, Some(at));
        }
    }

    /// Wingman `p` joins its lead at `at` (wu east, south, viewZ; none: where he is, for its
    /// place on the ground to be set).
    pub(crate) fn spawn_wingman_at(
        &mut self,
        p: PendingWing,
        at: Option<(f64, f64, i32)>,
    ) -> usize {
        let l = self.lead(p.lead);
        let (wx, wy, alt) = at.unwrap_or((l.wx, l.wy, l.alt));
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
        let (loadout, harms, arm, ground) = wing_stores(self, &bays);
        let tag = WING_TAG + p.lead as u32;
        let mut f =
            self.mission_flight(p.spec, AiTask::Patrol(u.x, u.y), tag, true, p.home as usize);
        f.loadout = loadout;
        f.wing = Some(Wing { lead: p.lead, harms, ground, arm, ..Default::default() });
        u.missile = f.classic_missile();
        self.flights[i] = u;
        if self.ai_flight.len() > i {
            self.ai_flight[i] = Default::default();
        }
        self.campaign.as_mut().expect("campaign").flights[i] = Some(f);
        if at.is_some() {
            let back = self.pl.id as usize;
            self.activate(p.lead as usize);
            self.msg(text!("wing-joined"));
            self.activate(back);
        }
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
        if u.flags & fflags::TAKEOFF != 0 {
            return;
        }
        let l = self.lead(w.lead);
        let (x, y, z) = self.station(&l, w.form, u.spec, 0.0);
        let heli = self.aircraft_kind(u.spec) == AircraftKind::Helicopter;
        let snap = if heli { SNAP_HELI } else { SNAP_JET } * WU_PER_M;
        let (dx, dy) = (x - u.wx as f64, y - u.wy as f64);
        if dx.hypot(dy) > snap {
            return;
        }
        // A helicopter holds its lead's height over the ground under it.
        let z = if heli { self.heli_wing_alt(&u, w) } else { z };
        let dt = 1.0 / self.fs.max(1) as f64;
        let k = (STATION_GAIN * dt).min(1.0);
        let h = (l.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        // Where it was before the AI moved it this frame (along its own heading at its speed;
        // a helicopter close to its station the AI leaves where it is), then where its lead's
        // motion takes the station this frame, and onto it.
        let hu = (u.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let own = if heli { 0.0 } else { u.speed as f64 * dt };
        let (px, py) = (u.wx as f64 - hu.sin() * own, u.wy as f64 + hu.cos() * own);
        let (dx, dy) = (x - px, y - py);
        let mv = l.speed * dt;
        // Closing at most a fraction of its lead's speed (from beside him after the take-off,
        // out to its station).
        let close = if heli { CLOSE_MIN_HELI } else { CLOSE_MIN };
        let most = (l.speed * CLOSE_MAX + close * WU_PER_M) * dt;
        let c = (dx.hypot(dy) * k).max(1e-9);
        let k = k * (most / c).min(1.0);
        let nx = px + h.sin() * mv + dx * k;
        let ny = py - h.cos() * mv + dy * k;
        u.wx = nx.round() as i32;
        u.wy = ny.round() as i32;
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.alt += if heli {
            let climb = HELI_CLIMB * dt;
            ((z - u.alt) as f64 * (k * 4.0).min(1.0)).clamp(-climb, climb).round() as i32
        } else {
            ((z - u.alt) as f64 * k).round() as i32
        };
        let turn = s16(l.heading - u.heading);
        u.heading = s16(u.heading + (turn as f64 * (k * 4.0).min(1.0)).round() as i32);
        u.pitch = l.pitch;
        u.bank = l.roll;
        u.speed = (l.speed.round() as i32).max(1);
        u.flags |= fflags::AIRBORNE;
        u.flags &= !fflags::RTB;
        self.flights[i] = u;
    }

    /// The height (viewZ) a helicopter wingman `u` flies at: its lead's over the ground, at
    /// least [`HELI_MIN_AGL`] over the ground under it and just ahead.
    fn heli_wing_alt(&self, u: &FlightUnit, w: &Wing) -> i32 {
        let p = &self.pilot_ref(w.lead as usize).p;
        let agl = (p.view_z - self.ground_z(p.mx, p.my)).max(0);
        let h = (u.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let ahead = HELI_LOOK
            .iter()
            .map(|&m| {
                let d = m / crate::aaa::M_PER_MAP;
                self.ground_z(
                    u.x + (h.sin() * d).round() as i32,
                    u.y - (h.cos() * d).round() as i32,
                )
            })
            .max()
            .unwrap_or(0);
        (self.ground_z(u.x, u.y) + agl).max(ahead + HELI_MIN_AGL)
    }

    /// Flight unit `i` is a helicopter wingman flying formation on its lead (up, no enemy
    /// aircraft to go for, no strike).
    pub(crate) fn heli_wing_flies(&self, i: usize) -> bool {
        let Some(f) = self.campaign.as_ref().and_then(|c| c.flights.get(i).copied().flatten())
        else {
            return false;
        };
        let Some(w) = f.wing else { return false };
        let u = &self.flights[i];
        w.phase == WingPhase::Air
            && matches!(w.order, WingOrder::Formation | WingOrder::Cover)
            && f.role == Role::Fighter
            && f.target.is_none()
            && u.flags & fflags::TAKEOFF == 0
            && self.aircraft_kind(u.spec) == AircraftKind::Helicopter
    }

    /// One frame of helicopter wingman `i` in formation: far from its station it flies for
    /// where the station will be, faster than its lead, low over the ground as he is; close to
    /// it [`Sim::keep_station`] keeps it there.
    pub(crate) fn fly_wing_helo(&mut self, i: usize) {
        let Some(w) = self.campaign.as_ref().and_then(|c| c.flights[i]).and_then(|f| f.wing) else {
            return;
        };
        let mut u = self.flights[i];
        let max = self.ac_type(u.spec).max_speed as f64;
        let dt = 1.0 / self.fs.max(1) as f64;
        let l = self.lead(w.lead);
        let (sx, sy, _) = self.station(&l, w.form, u.spec, 0.0);
        let (dx, dy) = (sx - u.wx as f64, sy - u.wy as f64);
        let d = dx.hypot(dy);
        u.flags |= fflags::AIRBORNE;
        u.flags &= !fflags::RTB;
        if d <= SNAP_HELI * WU_PER_M {
            self.flights[i] = u;
            return;
        }
        // For where the station will be by the time it gets there (lead pursuit).
        let hl = (l.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let vmax = max.max(l.speed) * HELI_DASH;
        let ahead = (d / vmax.max(1.0)).min(HELI_LEAD_S);
        let (ax, ay) = (dx + hl.sin() * l.speed * ahead, dy - hl.cos() * l.speed * ahead);
        let want = (l.speed + d * HELI_CLOSE_GAIN).clamp(1.0, vmax);
        let acc = max / 6.0 * dt;
        let speed = u.speed as f64 + (want - u.speed as f64).clamp(-acc, acc);
        u.speed = (speed.round() as i32).max(1);
        let brg = s16((ax.atan2(-ay) / std::f64::consts::TAU * 65536.0).round() as i32);
        let most = (0x2000 as f64 * dt).round() as i32;
        let turn = s16(brg - u.heading).clamp(-most, most);
        u.heading = s16(u.heading + turn);
        let h = (u.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let mv = u.speed as f64 * dt;
        u.wx = (u.wx as f64 + h.sin() * mv).round() as i32;
        u.wy = (u.wy as f64 - h.cos() * mv).round() as i32;
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        let climb = HELI_CLIMB * dt;
        let z = self.heli_wing_alt(&u, &w);
        u.alt += ((z - u.alt) as f64).clamp(-climb, climb).round() as i32;
        // Nose down with speed, banked into its turns.
        u.pitch = -(u.speed * 0x600 / (max as i32).max(1)).min(0x900);
        u.bank = (s16(brg - u.heading) * 2).clamp(-0x1800, 0x1800) * (u.speed > 20) as i32;
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

    /// The enemy SAM site ahead of pilot `lead` (within `AHEAD` of his heading) and within
    /// reach (object index) most worth an anti-radiation missile: the most valuable radar (how
    /// far it sees) for how far off it is, a radar on the air before one that is off.
    fn sam_ahead(&self, lead: u8) -> Option<usize> {
        let p = &self.pilot_ref(lead as usize).p;
        (1..self.objects.len())
            .filter(|&o| self.enemy_site(o))
            .filter_map(|o| {
                let ob = &self.objects[o];
                let (dx, dy) = (s16(ob.x - p.mx), s16(ob.y - p.my));
                let d = approx(dx, dy);
                let off = s16(crate::math::bearing(dx, s16(-dy)) - p.heading).abs();
                if d >= SAM_REACH || off > AHEAD {
                    return None;
                }
                let value = self.threat_type(ob.unit_type).map_or(1, |t| t.lethality.max(1));
                let worth = value as i64 * 0x10000 / (d + SAM_NEAR) as i64;
                Some((o, ob.alert > 0, worth))
            })
            .max_by_key(|&(o, on, worth)| (on, worth, std::cmp::Reverse(o)))
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
            // Its lead down or gone a while: home (on the ground with him: once he is gone).
            let pilot = self.pilot_ref(w.lead as usize);
            let away = pilot.id != w.lead || pilot.absent || pilot.eject != 0;
            let with_him = w.phase != WingPhase::Air;
            w.lead_down = if away || (pilot.on_ground && !with_him) { w.lead_down + 1 } else { 0 };
            if w.lead_down > LEAD_DOWN_SECS * fs && w.order != WingOrder::Rtb {
                let low = u.alt <= self.ground_z(u.x, u.y) + 20;
                let down = matches!(w.phase, WingPhase::Takeoff | WingPhase::Parked)
                    || (w.phase == WingPhase::Landing && low)
                    || (w.phase == WingPhase::Catapult && u.flags & fflags::TAKEOFF != 0);
                if down {
                    // Never up, or down with him: it stays down.
                    self.wing_landed(i);
                    continue;
                }
                if w.phase != WingPhase::Air {
                    self.leave_track(i, WingPhase::Air);
                    w.phase = WingPhase::Air;
                }
                w.order = WingOrder::Rtb;
                self.wing_rejoin(i);
                self.recall_flight(WING_TAG + w.lead as u32);
            }
            if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
                f.wing = Some(w);
            }
            // With its lead on the ground and in the circuit.
            match w.phase {
                WingPhase::Takeoff | WingPhase::Landing | WingPhase::Parked => {
                    self.fly_track(i);
                    continue;
                }
                WingPhase::Catapult => {
                    if u.flags & fflags::TAKEOFF == 0 {
                        w.phase = WingPhase::Air;
                    } else {
                        continue;
                    }
                }
                WingPhase::Air => {
                    if self.wing_lands(i, &w) {
                        continue;
                    }
                }
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
                let Some(o) = self.sam_firing_at(lead, i).or_else(|| self.sam_ahead(lead)) else {
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
        // A helicopter without air-to-air missiles leaves the air lock to its lead.
        let missiles = self.campaign.as_ref().and_then(|c| c.flights[i]).map(|f| f.loadout);
        let heli = self.aircraft_kind(self.flights[i].spec) == AircraftKind::Helicopter;
        let air_ok = !heli || missiles.is_some_and(|l| l.missiles() > 0);
        let air = Some(self.pl.air_lock)
            .filter(|&j| air_ok && j >= 0 && self.wing_enemy(j as usize))
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
