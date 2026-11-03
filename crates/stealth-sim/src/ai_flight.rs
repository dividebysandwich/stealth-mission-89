//! Realistic AI flight ([`crate::SimOptions::realistic_ai_flight`]): the AI's jets fly a point-mass
//! model like the player's (`src/flight/modern.rs`): thrust against parasitic and induced drag
//! (the hard turn costs speed), a g limit and less lift below the corner speed and in thin air,
//! gravity along the flight path (the zoom climb trades speed for height). Their pilots steer it
//! for the point the AI picks ([`Sim::fly`](crate::Sim)) and defend against the missiles they
//! notice: cranking off a distant shot, breaking and diving away from a close one, with flares
//! or chaff.
//!
//! Takeoff, the landing approach, rollout, base traffic and falling wrecks keep the classic
//! moves; the model takes over from (and hands back to) the flight unit's record seamlessly.

use crate::Sim;
use crate::math::s16;
use crate::types::*;
use crate::{AircraftType, campaign::M_PER_WU};
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// Gravity (wu/s²).
const G: f64 = 9.81 / WU_METRES;
/// The altitude (alt units) the top speeds are set for.
const REF_ALT: f64 = 10_000.0;
/// Afterburner: thrust over full military power, and its top speed over `max_speed` (the
/// classic intercept dash).
const AB_GAIN: f64 = 2.0;
const AB_SPEED: f64 = 1.5;
/// Aircraft at least this fast (game speed, Mach 1.4) have afterburners.
const AB_MIN_SPEED: i16 = 560;
/// Corner speed: share of `max_speed`, at sea level.
const CORNER: f64 = 0.6;
/// Steepest climb and dive (rad) flown for a point, and the defensive dive.
const CLIMB_MAX: f64 = 0.45;
const DIVE_MAX: f64 = 0.6;
const DEFEND_DIVE: f64 = 0.75;
/// The climb flown clear of the mountains ahead (rad).
const TERRAIN_CLIMB: f64 = 0.35;
/// Steering: turn rate per heading error, climb-angle rate per climb-angle error (1/s).
const K_PSI: f64 = 0.8;
const K_GAMMA: f64 = 1.2;
/// Room (alt units) kept over the ground below when pulling out of a dive, and the height
/// under which it no longer dives (about 300 m and 1 km).
const PULL_OUT_ROOM: f64 = 600.0;
const DIVE_FLOOR: f64 = 2000.0;
/// Integration steps per sim frame.
const SUBSTEPS: usize = 2;

/// Missile defence: time to go (s) and range (wu, about 2.5 km) under which it defends instead
/// of cranking, the time to go it goes back to cranking at, the crank angle off the missile's
/// bearing (rad, 55°), seconds between turning to the other side (and the random extra), the
/// time to go under which it releases countermeasures, and seconds between two releases.
const T_CLOSE: f64 = 8.0;
const R_CLOSE: f64 = 5000.0;
const T_CRANK_AGAIN: f64 = 14.0;
const CRANK: f64 = 0.96;
const CRANK_S: i32 = 6;
const CRANK_S_RANDOM: i32 = 5;
const T_CM: f64 = 4.0;
const CM_EVERY_S: f64 = 1.5;
/// Seconds a cartridge lives, and the most cartridges of each kind carried.
const CM_LIFE_S: i32 = 3;
const CM_STOCK: u8 = 16;
/// An IR missile (no warning) is seen in a look (one a second) at this chance (%), plus this per
/// difficulty level, a third of it in the rear cone (cosine of 120° off the nose), half from
/// `EYE_FAR` (wu, 4 km) and not at all from `EYE_MAX` (10 km).
const EYE_PCT: i32 = 20;
const EYE_DIFF_PCT: i32 = 20;
const EYE_REAR_COS: f64 = -0.5;
const EYE_FAR: f64 = 8000.0;
const EYE_MAX: f64 = 20_000.0;
/// Seconds without a threat after which the pilot stops watching for it.
const FORGET_S: i32 = 3;
/// Chance (%) an AI aircraft's flare or chaff `r` map units from missile `spec` fools it: a third
/// of the player's chance, better the harder the opponents; the AIM-9M's seeker (IRCCM) rejects
/// flares `IRCCM` times as often.
fn ai_cm_pct(r: i32, diff: i32, spec: i32) -> i32 {
    let pct = crate::missiles::cm_fool_pct(r, 3 - diff.clamp(0, 3)) / 3;
    if matches!(spec, 23 | 32) { pct / IRCCM } else { pct }
}

/// How much less often a flare fools the player's AIM-9M than another missile.
const IRCCM: i32 = 2;

/// `Projectile::cm_fooled` bit: a player missile a flare or chaff fooled; it flies on unguided.
pub(crate) const DECOYED: u8 = 0x80;

/// The flight model's parameters for an aircraft type.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Aero {
    /// Load limit (g) and roll rate (rad/s).
    pub n_max: f64,
    pub roll_rate: f64,
    /// Thrust over weight at full military power at sea level; afterburner fitted.
    pub thrust: f64,
    pub ab: bool,
    /// Drag over weight: `cd0 σ v² + k n² / (σ v²)`.
    pub cd0: f64,
    pub k: f64,
    /// Corner speed at sea level and `max_speed` (wu/s).
    pub v_corner: f64,
    pub v_max: f64,
}

impl Aero {
    /// The most g its wings can pull at speed `v` (wu/s) in air of density `sigma`.
    pub fn n_avail(&self, v: f64, sigma: f64) -> f64 {
        (self.n_max * sigma * (v / self.v_corner).powi(2)).min(self.n_max)
    }

    /// Thrust over weight at throttle `thr` (0..1 military power, to 2 with the afterburner).
    pub fn thrust(&self, thr: f64, sigma: f64) -> f64 {
        let t = if thr <= 1.0 { thr.max(0.0) } else { 1.0 + (AB_GAIN - 1.0) * (thr - 1.0) };
        self.thrust * t * sigma.powf(0.8)
    }

    pub fn drag(&self, v: f64, n: f64, sigma: f64) -> f64 {
        let q = sigma * v * v;
        self.cd0 * q + self.k * n * n / q.max(1.0)
    }

    pub fn thr_max(&self) -> f64 {
        if self.ab { 2.0 } else { 1.0 }
    }
}

/// The model of aircraft type `t`, from its speed and agility. Full thrust holds its top speed
/// level at `REF_ALT` (`max_speed`, with an afterburner 1.5 times that) and half its g limit
/// at its corner speed at sea level.
pub(crate) fn aero(t: &AircraftType) -> Aero {
    let v_max = (t.max_speed as f64).max(150.0);
    let agil = t.agility.clamp(1, 5) as f64;
    let ab = t.max_speed >= AB_MIN_SPEED;
    let heavy = !ab && t.agility <= 1;
    let n_max = if heavy { 2.5 } else { 4.5 + 0.9 * agil };
    let roll_rate = if heavy { 0.5 } else { (40.0 + 30.0 * agil).to_radians() };
    let full = if ab {
        0.75 + 0.08 * agil
    } else if heavy {
        0.3
    } else {
        0.45
    };
    let thrust = if ab { full / AB_GAIN } else { full };
    let v_top = if ab { AB_SPEED * v_max } else { v_max };
    let v_corner = CORNER * v_max;
    let n_sus = 0.5 * n_max;
    // cd0 σ v² + k n² / (σ v²) = T σ^0.8 at both points.
    let s = crate::guidance::density(REF_ALT);
    let (a1, b1, c1) = (s * v_top * v_top, 1.0 / (s * v_top * v_top), full * s.powf(0.8));
    let (a2, b2, c2) = (v_corner * v_corner, (n_sus / v_corner).powi(2), full);
    let det = a1 * b2 - a2 * b1;
    let cd0 = ((c1 * b2 - c2 * b1) / det).max(1e-9);
    let k = ((a1 * c2 - a2 * c1) / det).max(1.0);
    Aero { n_max, roll_rate, thrust, ab, cd0, k, v_corner, v_max }
}

/// A flight unit flown by the model.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct AiFlight {
    /// Flying under the model (else the classic AI moves it, and the model takes it up from
    /// the record again).
    pub active: bool,
    /// Position (wu east, wu south, alt units up), speed (wu/s), heading, climb angle and bank
    /// (rad), load (g), throttle (0..1 military power, to 2 with the afterburner) and the speed
    /// hold's integral.
    pub pos: [f64; 3],
    pub v: f64,
    pub psi: f64,
    pub gamma: f64,
    pub mu: f64,
    pub n: f64,
    pub thr: f64,
    pub hold: f64,
    #[serde(default)]
    pub evade: Evade,
    /// Its flares and chaff in the air, and those it has left (stocked when it takes off).
    #[serde(default)]
    pub cms: [AiCm; 2],
    pub flares: u8,
    pub chaff: u8,
    pub stocked: bool,
    /// Frames until it may release the next cartridge.
    pub cm_wait: i32,
}

/// An AI aircraft's flare (kind 1) or chaff (kind 2): map units, altitude, frames left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AiCm {
    pub x: i32,
    pub y: i32,
    pub alt: i32,
    pub kind: i32,
    pub ttl: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Phase {
    #[default]
    None,
    /// Turning off the missile's line, from side to side: it has to turn after it, and every
    /// turn costs a burnt-out missile speed.
    Crank,
    /// Beaming it in a steep dive, flares or chaff going.
    Defend,
}

/// What the pilot knows of the missile coming for him.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Evade {
    /// The threat watched, its range (wu) and closing speed (wu/s, smoothed).
    pub key: u32,
    pub range: f64,
    pub closing: f64,
    /// It has been noticed, and the frames before the pilot reacts.
    pub noticed: bool,
    pub react: i32,
    pub phase: Phase,
    /// The side it cranks or beams to (+1 right of the missile's bearing), frames until it
    /// cranks to the other side, until its next look out for a missile without a warning, and
    /// without any threat.
    pub side: i8,
    pub flip: i32,
    pub look: i32,
    pub lost: i32,
}

/// A missile homing on a flight unit this frame: the unit, which missile (pilot and slot, or a
/// campaign missile), where (wu east, south, up) and whether it is an IR missile (no warning).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnitThreat {
    pub unit: u16,
    pub key: u32,
    pub pos: [f64; 3],
    pub ir: bool,
}

/// What the pilot flies for: the heading error and climb angle (rad), the speed (wu/s), the
/// afterburner allowed, the share of the available g and a pull-out from a dive that has to
/// happen now.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Cmd {
    pub dpsi: f64,
    pub gamma: f64,
    pub speed: f64,
    pub ab: bool,
    pub g: f64,
    pub pull_out: bool,
}

/// What the classic AI decided this frame ([`Sim::fly`](crate::Sim)): its point, mode, range
/// (map units), the dogfight table's roll and pitch commands, and whether it turns hard away
/// (too many fighters on the player, or near his base).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Steer {
    pub tx: i32,
    pub ty: i32,
    pub tz: i32,
    pub mode: i32,
    pub rng: i32,
    pub dogfight: bool,
    pub roll_cmd: i32,
    pub pitch_cmd: i32,
    pub circle: bool,
}

pub(crate) fn rad(bam: i32) -> f64 {
    s16(bam) as f64 * PI / 32768.0
}

pub(crate) fn bam(a: f64) -> i32 {
    s16((a * 32768.0 / PI).round() as i32)
}

fn wrap(a: f64) -> f64 {
    let a = (a + PI).rem_euclid(TAU) - PI;
    if a <= -PI { a + TAU } else { a }
}

impl AiFlight {
    /// Takes the aircraft up from its record.
    pub fn sync(&mut self, u: &FlightUnit) {
        self.pos = [u.wx as f64, u.wy as f64, u.alt as f64];
        self.v = u.speed.max(1) as f64;
        self.psi = rad(u.heading);
        self.gamma = rad(u.pitch);
        self.mu = rad(u.bank);
        self.n = 1.0;
        self.thr = 0.8;
        self.hold = 0.0;
        self.active = true;
    }

    /// Whether the record still shows where the model left it.
    fn matches(&self, u: &FlightUnit) -> bool {
        self.pos[0].round() as i32 == u.wx
            && self.pos[1].round() as i32 == u.wy
            && self.pos[2].round() as i32 == u.alt
    }

    /// Writes the aircraft to its record.
    fn store(&self, u: &mut FlightUnit) {
        u.wx = self.pos[0].round() as i32;
        u.wy = self.pos[1].round() as i32;
        u.alt = self.pos[2].round() as i32;
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.heading = bam(self.psi);
        u.pitch = bam(self.gamma);
        u.bank = bam(self.mu);
        u.speed = (self.v.round() as i32).max(1);
    }

    /// The height (alt units) a pull-out from the dive now takes: rolling upright, then the
    /// arc at the most g there is.
    fn pull_out_height(&self, ae: &Aero, sigma: f64) -> f64 {
        if self.gamma >= 0.0 {
            return 0.0;
        }
        let n = (ae.n_avail(self.v, sigma) - 1.0).max(0.5);
        let arc = self.v * self.v / (G * n) * (1.0 - self.gamma.cos());
        let roll = (self.mu.abs() - FRAC_PI_2).max(0.0) / ae.roll_rate;
        arc + self.v * (-self.gamma).sin() * roll
    }

    /// One frame (`dt` s) of flight under the pilot's command.
    pub(crate) fn step(&mut self, ae: &Aero, cmd: &Cmd, dt: f64) {
        let sigma = crate::guidance::density(self.pos[2]);
        let n_av = ae.n_avail(self.v, sigma);
        let n_lim = (n_av * cmd.g).max(1.0f64.min(n_av));
        let v = self.v.max(1.0);
        let (mu_cmd, mut n_cmd);
        if cmd.pull_out {
            mu_cmd = 0.0;
            n_cmd = n_av;
        } else {
            // The turn rate and climb-angle change wanted, as the lift it takes.
            let w_max = G * (n_lim * n_lim - 1.0).max(0.0).sqrt() / v;
            let omega = (cmd.dpsi * K_PSI).clamp(-w_max, w_max);
            let a_lat = v * omega * self.gamma.cos();
            let a_vert = v * (cmd.gamma - self.gamma) * K_GAMMA + G * self.gamma.cos();
            if a_vert < 0.0 && a_vert > -G && a_lat.abs() < -a_vert {
                // A gentle push over: negative g, wings near level.
                n_cmd = a_vert / G;
                mu_cmd =
                    if n_cmd < -0.05 { (a_lat / (n_cmd * G)).clamp(-1.0, 1.0).asin() } else { 0.0 };
            } else {
                n_cmd = a_lat.hypot(a_vert) / G;
                mu_cmd = a_lat.atan2(a_vert);
            }
            n_cmd = n_cmd.min(n_lim);
        }
        // Roll at its rate; pull only as the lift comes round to where it is wanted.
        let err = wrap(mu_cmd - self.mu);
        self.mu = wrap(self.mu + err.clamp(-ae.roll_rate * dt, ae.roll_rate * dt));
        let err = wrap(mu_cmd - self.mu);
        let n_t = if n_cmd > 0.0 { n_cmd * err.cos().max(0.0) } else { n_cmd };
        self.n += (n_t - self.n) * (dt * 5.0).min(1.0);
        self.n = self.n.clamp(-1.5, n_av.max(0.0));
        // Throttle: hold the speed, the afterburner when allowed.
        let e = (cmd.speed - self.v) / cmd.speed.max(1.0);
        self.hold = (self.hold + e * dt).clamp(-1.0, 1.0);
        let top = if cmd.ab { ae.thr_max() } else { 1.0 };
        let thr = if cmd.ab && e > 0.0 {
            top
        } else {
            (0.7 + 4.0 * e + 0.5 * self.hold).clamp(0.05, top)
        };
        self.thr += (thr - self.thr) * (dt * 2.0).min(1.0);
        let h = dt / SUBSTEPS as f64;
        for _ in 0..SUBSTEPS {
            let sigma = crate::guidance::density(self.pos[2]);
            let v = self.v.max(1.0);
            let vdot =
                G * (ae.thrust(self.thr, sigma) - ae.drag(v, self.n, sigma) - self.gamma.sin());
            let gdot = G / v * (self.n * self.mu.cos() - self.gamma.cos());
            let pdot = G / v * self.n * self.mu.sin() / self.gamma.cos().max(0.05);
            self.v = (self.v + vdot * h).max(40.0);
            self.gamma += gdot * h;
            self.psi = wrap(self.psi + pdot * h);
            if self.gamma.abs() > FRAC_PI_2 {
                // Over the top or through the vertical: on the other heading, the other way up.
                self.gamma = self.gamma.signum() * PI - self.gamma;
                self.psi = wrap(self.psi + PI);
                self.mu = wrap(self.mu + PI);
            }
            let hv = self.v * self.gamma.cos();
            self.pos[0] += hv * self.psi.sin() * h;
            self.pos[1] -= hv * self.psi.cos() * h;
            self.pos[2] += self.v * self.gamma.sin() * h;
        }
    }
}

/// An IR missile (no launch warning to the target) of missile table entry `spec`.
pub(crate) fn ir_seeker(sim: &Sim, spec: i32) -> bool {
    match sim.missile(spec).class {
        7 => matches!(spec, 23 | 32),
        c => c <= 0,
    }
}

impl Sim {
    /// Flight unit `i` (not taking off, landing or falling) flies the model.
    pub(crate) fn ai_physics(&self, i: usize, u: &FlightUnit) -> bool {
        let ground = fflags::TAKEOFF | fflags::FINAL | fflags::ROLLOUT | fflags::DESTROYED;
        self.opts.realistic_ai_flight && u.flags & ground == 0 && !self.is_traffic(i)
    }

    /// Flight unit `i` is busy defending against a missile (it fires nothing).
    pub(crate) fn ai_defending(&self, i: usize) -> bool {
        self.ai_flight.get(i).is_some_and(|a| a.active && a.evade.phase == Phase::Defend)
    }

    /// Flight unit `i` flies the classic moves this frame: the model lets it go, and stocks up
    /// anew once it is down or taking off.
    pub(crate) fn ai_classic(&mut self, i: usize, u: &FlightUnit) {
        if let Some(a) = self.ai_flight.get_mut(i) {
            a.active = false;
            if u.flags & (fflags::TAKEOFF | fflags::ROLLOUT) != 0 {
                a.stocked = false;
                a.evade = Evade::default();
            }
        }
    }

    /// The AI's cartridges burn out.
    pub(crate) fn ai_cms_age(&mut self) {
        for a in &mut self.ai_flight {
            for c in &mut a.cms {
                c.ttl = (c.ttl - 1).max(0);
            }
            a.cm_wait = (a.cm_wait - 1).max(0);
        }
    }

    /// The AI aircraft's flares and chaff in the air: flight unit, slot, cartridge.
    pub fn ai_countermeasures(&self) -> impl Iterator<Item = (usize, usize, AiCm)> + '_ {
        self.ai_flight.iter().enumerate().flat_map(|(i, a)| {
            a.cms.iter().enumerate().filter(|(_, c)| c.ttl > 0).map(move |(k, c)| (i, k, *c))
        })
    }

    /// Flight unit `i` flies one frame of the model for the classic AI's decision `st`.
    pub(crate) fn fly_physics(
        &mut self,
        i: usize,
        u: &mut FlightUnit,
        t: &AircraftType,
        st: Steer,
    ) {
        let fs = self.fs.max(1);
        let dt = 1.0 / fs as f64;
        let ae = aero(t);
        if self.ai_flight.len() < self.flights.len() {
            self.ai_flight.resize(self.flights.len(), AiFlight::default());
        }
        let mut a = self.ai_flight[i];
        if !a.active || !a.matches(u) {
            a.sync(u);
        }
        if !a.stocked {
            let n = if ae.ab || t.agility >= 3 { CM_STOCK } else { CM_STOCK / 2 };
            (a.flares, a.chaff, a.stocked) = (n, n, true);
        }
        let sigma = crate::guidance::density(a.pos[2]);
        // --- what the pilot flies for ---
        let dx = s16(st.tx - u.x) as f64;
        let dy = s16(st.ty - u.y) as f64;
        let brg = dx.atan2(-dy);
        let mut cmd = Cmd {
            dpsi: wrap(brg - a.psi),
            gamma: (st.tz as f64 - a.pos[2])
                .atan2(st.rng.max(1) as f64 * 32.0)
                .clamp(-DIVE_MAX, CLIMB_MAX),
            speed: 0.75 * ae.v_max,
            ab: false,
            g: 0.6,
            pull_out: false,
        };
        match st.mode {
            1 if st.dogfight => {
                cmd.speed = AB_SPEED * ae.v_max;
                cmd.ab = true;
                cmd.g = 1.0;
                if st.pitch_cmd == 0x6000 {
                    cmd.gamma = 1.0;
                    cmd.dpsi = 0.0;
                } else if st.pitch_cmd == s16(0xA000) {
                    // The split-S: rolled over, pulling down and round.
                    cmd.gamma = -1.0;
                    cmd.dpsi = 2.8 * (st.roll_cmd.signum() as f64);
                } else if st.roll_cmd != 0 {
                    cmd.dpsi = (rad(st.roll_cmd) * 1.5).clamp(-2.8, 2.8);
                }
            }
            1 => {
                let dash = self.diff >= self.pl.threat_count_last;
                cmd.speed = if dash { AB_SPEED * ae.v_max } else { 0.9 * ae.v_max };
                cmd.ab = dash;
                cmd.g = 0.8;
            }
            2 => cmd.speed = 0.7 * ae.v_max,
            4 => {
                cmd.speed = 1.2 * ae.v_max;
                cmd.ab = true;
                cmd.g = 0.9;
            }
            _ => {}
        }
        if st.circle {
            cmd.dpsi = 1.2;
        }
        if u.flags & fflags::ESCORT != 0
            && self.escort_leader >= 0
            && let Some(l) = self.flights.get(self.escort_leader as usize)
        {
            // Keeping up with the leader, catching up when behind.
            cmd.speed = l.speed as f64 * (1.0 + (st.rng as f64 / 512.0).min(0.3));
            cmd.ab = ae.ab;
        }
        if let Some(e) = self.evade_cmd(i, &mut a, &ae) {
            cmd = e;
        }
        // --- the ground and the ceiling ---
        let ground = self
            .tiles
            .as_ref()
            .map_or(0, |t| t.relief.height(u.x, u.y).max(t.hills.ground_alt(u.x, u.y)))
            as f64;
        let h = a.pos[2] - ground;
        if u.flags & fflags::TERRAIN != 0 {
            cmd.gamma = cmd.gamma.max(TERRAIN_CLIMB);
        }
        if h < DIVE_FLOOR {
            cmd.gamma = cmd.gamma.max(-0.05 - 0.3 * (h / DIVE_FLOOR).max(0.0));
        }
        if a.pos[2] > 28_000.0 {
            cmd.gamma = cmd.gamma.min(0.0);
        }
        if a.gamma < 0.0 && h < a.pull_out_height(&ae, sigma) * 1.2 + PULL_OUT_ROOM {
            cmd.pull_out = true;
        }
        a.step(&ae, &cmd, dt);
        a.store(u);
        self.ai_flight[i] = a;
    }

    /// The missiles homing on flight unit `i`: the pilots' (as they homed last frame) and the
    /// campaign's.
    fn threats_on(&self, i: usize) -> Vec<UnitThreat> {
        let mut v: Vec<UnitThreat> =
            self.unit_threats.iter().filter(|t| t.unit as usize == i).copied().collect();
        if let Some(c) = self.campaign.as_ref() {
            for (k, m) in c.missiles.iter().enumerate() {
                if m.target != i || m.ground.is_some() || m.decoyed || m.arm_lost {
                    continue;
                }
                let pos = [
                    m.pos[0] / M_PER_WU,
                    0x10_0000 as f64 - m.pos[1] / M_PER_WU,
                    m.pos[2] / M_PER_WU,
                ];
                let key = 0x1_0000 | ((k as u32) << 8) | (m.spec as u32 & 0xFF);
                v.push(UnitThreat { unit: i as u16, key, pos, ir: ir_seeker(self, m.spec) });
            }
        }
        v
    }

    /// The pilot of flight unit `i` against the missiles coming for him: None while he knows
    /// of none (or has yet to react), else what he flies.
    fn evade_cmd(&mut self, i: usize, a: &mut AiFlight, ae: &Aero) -> Option<Cmd> {
        if self.ai_no_evasion {
            return None;
        }
        let fs = self.fs.max(1);
        let diff = self.diff.clamp(0, 3);
        let me = a.pos;
        let dist = |p: [f64; 3]| {
            let d = [p[0] - me[0], p[1] - me[1], p[2] - me[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        let (psi, gamma) = (a.psi, a.gamma);
        let side = |brg: f64| if wrap(psi - brg) >= 0.0 { 1 } else { -1 };
        let th = self.threats_on(i).into_iter().min_by(|x, y| dist(x.pos).total_cmp(&dist(y.pos)));
        let ev = &mut a.evade;
        let Some(th) = th else {
            ev.lost += 1;
            if ev.lost > FORGET_S * fs {
                *ev = Evade { look: ev.look, ..Evade::default() };
            } else if ev.lost > fs / 2 {
                ev.phase = Phase::None;
            }
            return None;
        };
        ev.lost = 0;
        let r = dist(th.pos);
        if th.key == ev.key && ev.range > 0.0 {
            let c = (ev.range - r) * fs as f64;
            ev.closing = ev.closing * 0.7 + c * 0.3;
        } else {
            ev.key = th.key;
            ev.closing = 0.0;
        }
        ev.range = r;
        let (dx, dy, dz) = (th.pos[0] - me[0], th.pos[1] - me[1], th.pos[2] - me[2]);
        let brg = dx.atan2(-dy);
        if !ev.noticed {
            let seen = if th.ir {
                // No warning: he has to see it coming.
                ev.look -= 1;
                if ev.look > 0 {
                    false
                } else {
                    ev.look = fs;
                    let nose = [gamma.cos() * psi.sin(), -gamma.cos() * psi.cos(), gamma.sin()];
                    let off = (nose[0] * dx + nose[1] * dy + nose[2] * dz) / r.max(1.0);
                    let mut pct = EYE_PCT + EYE_DIFF_PCT * diff;
                    if off < EYE_REAR_COS {
                        pct /= 3;
                    }
                    if r > EYE_MAX {
                        pct = 0;
                    } else if r > EYE_FAR {
                        pct /= 2;
                    }
                    self.rng.random(100) < pct
                }
            } else {
                true
            };
            if seen {
                ev.noticed = true;
                ev.react = (4 - diff) * fs / 2;
            }
            return None;
        }
        if ev.react > 0 {
            ev.react -= 1;
            return None;
        }
        if ev.closing < -20.0 {
            // It is falling behind or has gone past: his own business again.
            ev.phase = Phase::None;
            return None;
        }
        let tgo = if ev.closing > 1.0 { r / ev.closing } else { f64::INFINITY };
        let phase = match ev.phase {
            Phase::Defend if tgo > T_CRANK_AGAIN && r > 1.5 * R_CLOSE => Phase::Crank,
            Phase::Defend => Phase::Defend,
            _ if tgo < T_CLOSE || r < R_CLOSE => Phase::Defend,
            _ => Phase::Crank,
        };
        if phase != ev.phase {
            ev.side = side(brg);
            ev.flip = (CRANK_S + self.rng.random(CRANK_S_RANDOM)) * fs;
        }
        ev.phase = phase;
        let mut cmd =
            Cmd { dpsi: 0.0, gamma: 0.0, speed: ae.v_max, ab: ae.ab, g: 0.6, pull_out: false };
        let s = ev.side as f64;
        match phase {
            Phase::Crank => {
                ev.flip -= 1;
                if ev.flip <= 0 {
                    ev.side = -ev.side;
                    ev.flip = (CRANK_S + self.rng.random(CRANK_S_RANDOM)) * fs;
                }
                // Off its line ahead, or off the line away from it when it comes from behind
                // (a weaving drag: it has the long chase and the turns).
                let line = if wrap(psi - brg).abs() > FRAC_PI_2 { brg + PI } else { brg };
                cmd.dpsi = wrap(line + ev.side as f64 * CRANK - psi);
                cmd.gamma = -0.09;
            }
            Phase::Defend => {
                cmd.dpsi = wrap(brg + s * FRAC_PI_2 - psi);
                cmd.gamma = -DEFEND_DIVE;
                cmd.speed = AB_SPEED * ae.v_max;
                cmd.g = 1.0;
                if tgo < T_CM && a.cm_wait == 0 {
                    self.ai_release(i, a, if th.ir { 1 } else { 2 });
                }
            }
            Phase::None => return None,
        }
        Some(cmd)
    }

    /// Flight unit `i` releases a flare (kind 1) or chaff (kind 2), if it has one left: the
    /// pilots' missiles roll against it as they see it, the campaign's missiles on it now.
    fn ai_release(&mut self, i: usize, a: &mut AiFlight, kind: i32) {
        let left = if kind == 1 { &mut a.flares } else { &mut a.chaff };
        if *left == 0 || self.ai_no_cms {
            return;
        }
        *left -= 1;
        a.cm_wait = (CM_EVERY_S * self.fs as f64).ceil() as i32;
        let k = if a.cms[0].ttl <= a.cms[1].ttl { 0 } else { 1 };
        let u = self.flights[i];
        a.cms[k] = AiCm { x: u.x, y: u.y, alt: u.alt, kind, ttl: CM_LIFE_S * self.fs };
        for s in Self::own_slots() {
            let m = &mut self.proj[s];
            m.cm_rolled &= !(1 << k);
            m.cm_fooled &= !(1 << k);
        }
        let diff = self.diff;
        let Some(mut c) = self.campaign.take() else { return };
        for m in c.missiles.iter_mut() {
            if m.target != i
                || m.ground.is_some()
                || m.decoyed
                || ir_seeker(self, m.spec) != (kind == 1)
            {
                continue;
            }
            let d = [
                m.pos[0] / M_PER_WU - a.pos[0],
                0x10_0000 as f64 - m.pos[1] / M_PER_WU - a.pos[1],
                m.pos[2] / M_PER_WU - a.pos[2],
            ];
            // Map units, as the pilots' missiles roll.
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / 32.0;
            if r <= crate::missiles::CM_FAR as f64
                && self.rng.random(100) < ai_cm_pct(r as i32, diff, m.spec)
            {
                m.decoyed = true;
            }
        }
        self.campaign = Some(c);
    }

    /// A pilot's air-to-air missile in slot `s` homes on flight unit `j`: unless a cartridge of
    /// the unit's fools it (it then flies on unguided, true when fooled), the unit learns of it.
    pub(crate) fn unit_homed(&mut self, s: usize, j: usize, cls: i32) -> bool {
        let spec = self.proj[s].spec;
        let ir = ir_seeker(self, spec);
        let cms = self.ai_flight.get(j).map(|a| a.cms).unwrap_or_default();
        for (k, c) in cms.iter().enumerate() {
            if c.ttl <= 0 || (c.kind == 1) != ir || !self.acquire(s, c.x, c.y, c.alt, cls) {
                continue;
            }
            let bit = 1u8 << k;
            if self.proj[s].cm_rolled & bit == 0 {
                self.proj[s].cm_rolled |= bit;
                if self.rng.random(100) < ai_cm_pct(self.acq_range, self.diff, spec) {
                    self.proj[s].cm_fooled |= bit | DECOYED;
                    return true;
                }
            }
        }
        let (x, y, alt) = self.proj[s].map_pos();
        let key = ((self.pl.id as u32) << 8) | s as u32;
        self.unit_threats.push(UnitThreat {
            unit: j as u16,
            key,
            pos: [x * 32.0, y * 32.0, alt],
            ir,
        });
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fighter() -> AircraftType {
        AircraftType {
            name: "MiG-29".into(),
            nato: String::new(),
            max_speed: 730,
            range: 520,
            agility: 5,
            radar: 1,
            model: 16,
            unknown: 2,
            kills: 0,
            kind: crate::AircraftKind::Plane,
            stealth: 100,
        }
    }

    fn level(alt: f64, v: f64) -> AiFlight {
        AiFlight { active: true, pos: [0.0, 0.0, alt], v, n: 1.0, thr: 0.8, ..Default::default() }
    }

    fn cmd(speed: f64, ab: bool) -> Cmd {
        Cmd { dpsi: 0.0, gamma: 0.0, speed, ab, g: 1.0, pull_out: false }
    }

    fn fly(a: &mut AiFlight, ae: &Aero, c: &Cmd, secs: f64) {
        for _ in 0..(secs * 15.0) as usize {
            a.step(ae, c, 1.0 / 15.0);
        }
    }

    #[test]
    fn military_power_holds_max_speed_and_the_afterburner_half_as_much_again() {
        let ae = aero(&fighter());
        let mut a = level(REF_ALT, 500.0);
        fly(&mut a, &ae, &cmd(5000.0, false), 240.0);
        assert!((a.v / 730.0 - 1.0).abs() < 0.12, "military power {:.0}", a.v);
        let mut a = level(REF_ALT, 500.0);
        fly(&mut a, &ae, &cmd(5000.0, true), 240.0);
        assert!((a.v / (AB_SPEED * 730.0) - 1.0).abs() < 0.05, "afterburner {:.0}", a.v);
        assert!((a.pos[2] - REF_ALT).abs() < 300.0, "held its height: {:.0}", a.pos[2]);
    }

    #[test]
    fn a_hard_turn_costs_speed() {
        let ae = aero(&fighter());
        let mut a = level(5000.0, 700.0);
        let c = Cmd { dpsi: 3.0, ..cmd(700.0, true) };
        fly(&mut a, &ae, &c, 8.0);
        assert!(a.v < 520.0, "still {:.0} wu/s after 8 s at max g", a.v);
        assert!(a.n > 3.0, "pulling {:.1} g", a.n);
    }

    #[test]
    fn a_zoom_climb_trades_speed_for_height() {
        let ae = aero(&fighter());
        let mut a = level(5000.0, 900.0);
        let c = Cmd { gamma: 1.2, ..cmd(900.0, false) };
        fly(&mut a, &ae, &c, 10.0);
        assert!(a.pos[2] > 8000.0, "only up to {:.0}", a.pos[2]);
        assert!(a.v < 700.0, "kept {:.0} wu/s", a.v);
    }

    #[test]
    fn little_lift_when_slow() {
        let ae = aero(&fighter());
        let s = crate::guidance::density(5000.0);
        assert!(ae.n_avail(200.0, s) < 2.0);
        assert_eq!(ae.n_avail(700.0, s), ae.n_max);
    }

    #[test]
    fn a_split_s_loses_height_and_turns_it_round() {
        let ae = aero(&fighter());
        let mut a = level(15_000.0, 500.0);
        let c = Cmd { dpsi: 2.8, gamma: -1.0, ..cmd(500.0, false) };
        for _ in 0..15 * 12 {
            let c = Cmd { dpsi: wrap(PI - a.psi), ..c };
            a.step(&ae, &c, 1.0 / 15.0);
        }
        assert!(a.pos[2] < 13_500.0, "still at {:.0}", a.pos[2]);
        assert!(wrap(a.psi).abs() > 2.5, "heading {:.2}", a.psi);
    }

    #[test]
    fn it_pulls_out_over_the_ground() {
        let ae = aero(&fighter());
        let mut a = level(3000.0, 700.0);
        a.gamma = -0.7;
        for _ in 0..15 * 20 {
            let sigma = crate::guidance::density(a.pos[2]);
            let pull =
                a.gamma < 0.0 && a.pos[2] < a.pull_out_height(&ae, sigma) * 1.2 + PULL_OUT_ROOM;
            let c = Cmd { gamma: -0.7, pull_out: pull, ..cmd(700.0, true) };
            a.step(&ae, &c, 1.0 / 15.0);
            assert!(a.pos[2] > 0.0, "flew into the ground");
        }
    }

    #[test]
    fn every_type_has_a_sane_model() {
        for (v, ag) in [(400, 1), (392, 2), (750, 1), (600, 4), (780, 5)] {
            let t = AircraftType { max_speed: v, agility: ag, ..fighter() };
            let ae = aero(&t);
            assert!(ae.cd0 > 0.0 && ae.k > 0.0, "{v}/{ag}: {ae:?}");
            let s = crate::guidance::density(REF_ALT);
            assert!(ae.thrust(ae.thr_max(), s) > ae.drag(0.8 * v as f64, 1.0, s), "{v}/{ag}");
        }
    }
}
