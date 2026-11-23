//! The fleet's own defence and what it defends against, simplified for the game.
//!
//! - Sea-skimming anti-ship missiles (a submarine's salvo, `asw.rs`): they take an enemy
//!   missile slot and fly here, not as the classic missiles do: up off the water, down to skim
//!   the sea to their ship, a pop-up and dive at the end. A hit sinks an escort; the carrier
//!   takes it.
//! - The ships' Phalanx close-in weapon systems: three mounts on each carrier, one on each of
//!   its escorts. A mount engages the nearest threat in its reach (a sea-skimmer, an attack boat)
//!   with bursts of its 20 mm gun; every second of fire may knock the threat down. Its rounds
//!   are drawn as tracers.
//!
//! The fleet comes alive (its mounts are placed) with the first threat at sea, so a mission
//! without one keeps its state as it was. The host flies everything; the other simulations get
//! the picture ([`Naval`] in the fast world).

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::aaa::{GroundKind, M_PER_MAP, Shell};
use crate::types::*;
use crate::{ENEMY_SLOTS, Sim};

/// The sea-skimmer's missile table entry (`assets/sim/units.ron`).
pub const SEA_MISSILE_SPEC: i32 = 55;
/// Sea-skimmer: speeds (wu/s) cruising and in the last run in, the height it climbs to off
/// the water and the one it skims at (wu), the range at which it pops up and dives (wu), its
/// turn rate (degrees/s), and the seconds it flies at most.
const CRUISE: f64 = 600.0;
const TERMINAL: f64 = 1100.0;
const BOOST_ALT: f64 = 300.0;
const SKIM_ALT: f64 = 20.0;
const POP_UP: f64 = 5_000.0;
const POP_ALT: f64 = 250.0;
const SEA_TURN_DEG: f64 = 6.0;
const SEA_SECS: i32 = 400;
/// It hits a ship this close (wu over the water), the carrier this close, under this height.
const HIT_SHIP: f64 = 70.0;
const HIT_CARRIER: f64 = 260.0;
const HIT_HEIGHT: f64 = 200.0;
/// Escorts of a carrier: ships this close to it (map units).
const ESCORT_REACH: i32 = 768;
/// Phalanx: reach (m), rounds in its magazine and seconds to reload it, rounds a second, the
/// muzzle speed (m/s), the chance a second of fire has to knock down a sea-skimmer and an
/// attack boat (percent), and how far a tracer flies (s).
const CIWS_REACH: f64 = 2_000.0;
/// Against a boat low on the water, among the waves: closer in.
const CIWS_BOAT_REACH: f64 = 1_200.0;
const CIWS_ROUNDS: i32 = 1_550;
const CIWS_RELOAD: f32 = 60.0;
const CIWS_RATE: i32 = 75;
const CIWS_MUZZLE: f64 = 1_100.0;
const CIWS_KILL_MISSILE_PCT: i32 = 30;
const CIWS_KILL_BOAT_PCT: i32 = 2;
const TRACER_SECS: f64 = 1.6;
/// A tracer every this many frames a mount fires.
const TRACER_EVERY: i32 = 2;
/// Mounts on the carrier (nimitz.blend metres: the middles of the source's own Phalanx mounts on
/// the deck-edge sponsons, port and starboard of the bow and low on the port quarter, and the
/// sponson floor each stands on; tools/models/nimitz.py `CIWS` leaves those out for these).
const CARRIER_MOUNTS: [(f64, f64, f64); 3] =
    [(-22.5, 89.4, 14.64), (22.9, 89.3, 14.64), (-15.5, -147.0, 9.44)];
/// The mount on an escort (ft).
const ESCORT_MOUNT_ALT: f64 = 30.0;

/// The fleet's blasts go off this high (viewZ): above where a blast hurts the ground units round
/// it (`aaa_damage`), so a boat blowing up does not take its neighbours with it.
const SEA_BLAST_ALT: i32 = 60;

/// Metres per wu (over the ground), per altitude unit (up: feet, as the flight models take it).
const WU_M: f64 = M_PER_MAP / 32.0;
const FT_M: f64 = 0.3048;

/// A sea-skimmer in flight (enemy missile slot): its ship (world object), the submarine that
/// fired it (its id), and its phase: 0 climbing off the water, 1 skimming, 2 popped up and
/// diving.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeaGuide {
    pub target: u16,
    pub sub: u16,
    pub phase: u8,
}

/// What a mount fires at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Threat {
    /// A sea-skimmer (enemy missile slot).
    Missile(u8),
    /// An attack boat (`Sim::units` index).
    Boat(u16),
}

/// A Phalanx mount: on its ship (world object), where it stands (m east, north, up), where
/// its gun points (azimuth clockwise from north, elevation; rad), firing, the rounds left,
/// seconds to its reload's end.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ciws {
    pub ship: u16,
    pub at: [f64; 3],
    pub az: f32,
    pub el: f32,
    pub firing: bool,
    pub rounds: i32,
    pub reload: f32,
    pub target: Option<Threat>,
    /// The way its sponson points out from the ship's side (rad clockwise from north).
    #[serde(default)]
    pub out: f32,
}

/// A campaign sortie's raid: the attack boats a player fights (`Sim::units` indices), one of
/// them has reached a ship.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Raid {
    pub player: u8,
    pub boats: Vec<u16>,
    pub hit: bool,
}

/// The mission type of a sortie's primary slot fighting a raid (after the hunt's 12).
pub const RAID_MISSION: i32 = 13;
/// A raid's boats: how many (the least, and up to so many more) and how far out (km).
const RAID_BOATS: (usize, usize) = (4, 3);
const RAID_KM: f64 = 8.0;

/// The fleet's defence.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Naval {
    /// The mounts were placed (at the start when the mission has a fleet, else on the first
    /// threat).
    pub armed: bool,
    pub mounts: Vec<Ciws>,
    /// The mounts' tracers (m, m/s: east, north, up).
    pub shells: Vec<Shell>,
    /// The campaign sorties' raids (the host's).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raids: Vec<Raid>,
}

impl Naval {
    pub fn is_empty(&self) -> bool {
        !self.armed
    }
}

/// A position in wu (east, south, up) in metres (east, north, up).
fn wu_metres(p: [f64; 3]) -> [f64; 3] {
    [p[0] * WU_M, (0x10_0000 as f64 - p[1]) * WU_M, p[2] * FT_M]
}

fn len(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = len(a).max(1e-9);
    [a[0] / l, a[1] / l, a[2] / l]
}

impl Sim {
    /// A world object's middle (wu east, south).
    fn object_wu(&self, o: usize) -> [f64; 2] {
        let g = self.objects[o];
        [g.x as f64 * 32.0 + 16.0, g.y as f64 * 32.0 + 16.0]
    }

    /// The ships a carrier's group defends: its carriers and the ships near them that are not
    /// the enemy's (armed ships are sites).
    pub(crate) fn fleet(&self) -> Vec<usize> {
        let carriers: Vec<usize> = (0..self.objects.len())
            .filter(|&i| self.objects[i].flags & oflags::CARRIER != 0)
            .collect();
        let mut out = carriers.clone();
        for i in 0..self.objects.len() {
            let o = self.objects[i];
            let enemy = i < self.ground_units && o.unit_type != 0;
            if out.contains(&i) || enemy || !self.is_ship(i as i32) {
                continue;
            }
            let near = carriers.iter().any(|&c| {
                let k = self.objects[c];
                (o.x - k.x).abs() < ESCORT_REACH && (o.y - k.y).abs() < ESCORT_REACH
            });
            if near {
                out.push(i);
            }
        }
        out
    }

    /// Places the fleet's mounts (once, at the start or on the first threat): three on each
    /// carrier, one on each escort.
    pub(crate) fn arm_fleet(&mut self) {
        if self.naval.armed {
            return;
        }
        self.naval.armed = true;
        let s = if self.ns < 0 { -1.0 } else { 1.0 };
        for o in self.fleet() {
            let g = self.objects[o];
            let mut put = |at: [f64; 3], out: f64| {
                self.naval.mounts.push(Ciws {
                    ship: o as u16,
                    at,
                    out: out as f32,
                    az: 0.0,
                    el: 0.0,
                    firing: false,
                    rounds: CIWS_ROUNDS,
                    reload: 0.0,
                    target: None,
                })
            };
            if g.flags & oflags::CARRIER != 0 {
                let (pe, pn) = crate::catapult::pivot(g.x, g.y);
                for (x, y, z) in CARRIER_MOUNTS {
                    let k = crate::catapult::WU_PER_M * s;
                    // Out from the side it stands on (the bow north for `s` 1, south for -1).
                    let out = std::f64::consts::FRAC_PI_2 * x.signum() * s;
                    let alt = crate::catapult::deck_height(z);
                    put([(pe + x * k) * WU_M, (pn + y * k) * WU_M, alt * FT_M], out);
                }
            } else {
                let [e, n] = crate::asw::object_metres(&g);
                put([e, n, ESCORT_MOUNT_ALT * FT_M], 0.0);
            }
        }
    }

    /// Fires a sea-skimmer from the sea at (m east, north) at ship `target` (world object),
    /// for submarine `sub`. False when no missile slot is free.
    pub(crate) fn launch_sea_missile(&mut self, at: [f64; 2], target: usize, sub: u16) -> bool {
        let Some(k) = (0..ENEMY_SLOTS).find(|&k| self.proj[k].ttl == 0) else { return false };
        self.arm_fleet();
        let p = [at[0] / WU_M, 0x10_0000 as f64 - at[1] / WU_M, 5.0];
        let to = self.object_wu(target);
        let heading = (to[0] - p[0]).atan2(-(to[1] - p[1]));
        let d = [heading.sin() * 0.5, -heading.cos() * 0.5, 0.87];
        let mut pm = Projectile {
            x: ((p[0] / 32.0) as i32) & 0xFFFF,
            y: ((p[1] / 32.0) as i32) & 0xFFFF,
            speed: 1,
            spec: SEA_MISSILE_SPEC,
            target: -1,
            sea: Some(SeaGuide { target: target as u16, sub, phase: 0 }),
            ..Default::default()
        };
        crate::guidance::start_flight(&mut pm, p, d.map(|c| c * 200.0), self.fs);
        // (start_flight sets the table missile's life.)
        pm.ttl = SEA_SECS * self.fs;
        self.proj[k] = pm;
        self.proj_pilot[k] = self.pl.id;
        self.emit(SimEvent::MissileLaunched {
            slot: k,
            spec: SEA_MISSILE_SPEC,
            launcher: Launcher::Sub(sub),
        });
        true
    }

    /// One frame of the fleet's war at sea: the sea-skimmers fly, the mounts fire, their
    /// tracers fly (the host's).
    pub(crate) fn update_naval(&mut self) {
        if !self.naval.armed || self.mirror {
            return;
        }
        for k in 0..ENEMY_SLOTS {
            if self.proj[k].ttl != 0 && self.proj[k].sea.is_some() {
                self.sea_frame(k);
            }
        }
        self.drive_boats();
        self.check_raids();
        self.update_mounts();
        let dt = 1.0 / self.fs.max(1) as f64;
        self.naval.shells.retain_mut(|s| {
            for a in 0..3 {
                s.pos[a] += s.vel[a] * dt;
            }
            s.vel[2] -= 9.81 * dt;
            s.ttl -= 1;
            s.ttl > 0 && s.pos[2] > 0.0
        });
    }

    /// One frame of the sea-skimmer in enemy slot `k`.
    fn sea_frame(&mut self, k: usize) {
        let m = self.proj[k];
        let (Some(mut g), Some(mut f)) = (m.sea, m.flight) else {
            self.proj[k].ttl = 0;
            return;
        };
        let dt = 1.0 / self.fs.max(1) as f64;
        let fine = ROUND_FINE as f64;
        let mut p = [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine];
        let v = [f.vx as f64 / fine, f.vy as f64 / fine, f.vz as f64 / fine];
        let mut d = norm(v);
        let t = g.target as usize;
        let to = self.object_wu(t.min(self.objects.len() - 1));
        let gone = self.objects.get(t).is_none_or(|o| o.flags & oflags::DESTROYED != 0);
        let flat = [to[0] - p[0], to[1] - p[1]];
        let range = flat[0].hypot(flat[1]);
        if g.phase == 0 && p[2] >= BOOST_ALT {
            g.phase = 1;
        }
        if g.phase == 1 && range < POP_UP {
            g.phase = 2;
        }
        let speed = if g.phase == 2 { TERMINAL } else { CRUISE }.min(len(v) + 300.0 * dt);
        let want = match g.phase {
            0 => d,
            1 => {
                let h = flat[0].atan2(-flat[1]);
                let climb = ((SKIM_ALT - p[2]) / (3.0 * speed)).clamp(-0.3, 0.3);
                norm([h.sin(), -h.cos(), climb])
            }
            _ => {
                // Up over the ship's side, then down onto it.
                let z = if range > POP_UP * 0.5 { POP_ALT } else { 40.0 };
                norm([flat[0], flat[1], z - p[2]])
            }
        };
        let rate = SEA_TURN_DEG.to_radians() * dt * if g.phase == 2 { 3.0 } else { 1.0 };
        d = norm([
            d[0] + (want[0] - d[0]).clamp(-rate, rate),
            d[1] + (want[1] - d[1]).clamp(-rate, rate),
            d[2] + (want[2] - d[2]).clamp(-rate, rate),
        ]);
        p = [p[0] + d[0] * speed * dt, p[1] + d[1] * speed * dt, p[2] + d[2] * speed * dt];
        let q = |a: f64| (a * fine).round() as i32;
        (f.x, f.y, f.alt, f.vx, f.vy, f.vz) =
            (q(p[0]), q(p[1]), q(p[2]), q(d[0] * speed), q(d[1] * speed), q(d[2] * speed));
        f.t += 1;
        let pm = &mut self.proj[k];
        pm.flight = Some(f);
        pm.sea = Some(g);
        pm.x = ((p[0] / 32.0).floor() as i32) & 0xFFFF;
        pm.y = ((p[1] / 32.0).floor() as i32) & 0xFFFF;
        pm.alt = crate::math::view_z_f(p[2]) as i32;
        pm.speed = (speed / 128.0).round() as i32;
        let (heading, pitch) = crate::guidance::heading_pitch(d);
        (pm.heading, pm.pitch, pm.roll) = (heading, pitch, 0);
        pm.ttl -= 1;
        let carrier = self.objects.get(t).is_some_and(|o| o.flags & oflags::CARRIER != 0);
        let reach = if carrier { HIT_CARRIER } else { HIT_SHIP };
        if !gone && range < reach && p[2] < HIT_HEIGHT {
            self.sea_missile_hit(k, t, g.sub);
        } else if p[2] <= 0.0 || (g.phase == 2 && range > POP_UP * 1.5) {
            // Into the sea, or past its ship.
            self.proj[k].ttl = 0;
            let (x, y) = (pm_map(p).0, pm_map(p).1);
            self.emit(SimEvent::Explosion { x, y, alt: SEA_BLAST_ALT, kind: -6 });
        }
    }

    /// Sea-skimmer `k` hits ship `t`: an escort sinks, the carrier takes it. The hunt of the
    /// submarine that fired it is lost.
    fn sea_missile_hit(&mut self, k: usize, t: usize, sub: u16) {
        let o = self.objects[t];
        self.proj[k].ttl = 0;
        self.emit(SimEvent::Explosion { x: o.x, y: o.y, alt: SEA_BLAST_ALT, kind: 0x10 });
        let name = self.place_name(t);
        if o.flags & oflags::CARRIER != 0 {
            self.msg(text!("sim-carrier-hit", ship = name));
        } else {
            self.msg(text!("sim-ship-sunk", ship = name));
        }
        let hunting = self.asw.result == crate::asw::AswResult::Hunting;
        let hunted = self.asw.subs.iter().any(|s| s.id == sub);
        if hunting && hunted {
            self.lose_hunt(crate::asw::AswResult::ShipLost, Some(t));
        } else if o.flags & oflags::CARRIER == 0 && o.flags & oflags::DESTROYED == 0 {
            self.destroy_ground(t as i32);
        }
    }

    /// Where a threat is now (m east, north, up) and its velocity (m/s), if it is still there.
    fn threat_at(&self, t: Threat) -> Option<([f64; 3], [f64; 3])> {
        match t {
            Threat::Missile(k) => {
                let m = self.proj.get(k as usize).filter(|m| m.ttl != 0 && m.sea.is_some())?;
                let f = m.flight?;
                let fine = ROUND_FINE as f64;
                let p = wu_metres([f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine]);
                let v = [
                    f.vx as f64 / fine * WU_M,
                    -f.vy as f64 / fine * WU_M,
                    f.vz as f64 / fine * FT_M,
                ];
                Some((p, v))
            }
            Threat::Boat(i) => {
                let u = self
                    .units
                    .get(i as usize)
                    .filter(|u| !u.destroyed && u.kind == GroundKind::Usv)?;
                let p = u.metres();
                let v = u.fine.map_or([0.0; 3], |f| {
                    let s = crate::naval::USV_SPEED;
                    [f.heading.sin() as f64 * s, f.heading.cos() as f64 * s, 0.0]
                });
                Some(([p[0], p[1], 0.5], v))
            }
        }
    }

    /// Every threat at sea now.
    fn threats(&self) -> Vec<Threat> {
        let mut out: Vec<Threat> = (0..ENEMY_SLOTS)
            .filter(|&k| self.proj[k].ttl != 0 && self.proj[k].sea.is_some())
            .map(|k| Threat::Missile(k as u8))
            .collect();
        out.extend(
            (0..self.units.len())
                .filter(|&i| !self.units[i].destroyed && self.units[i].kind == GroundKind::Usv)
                .map(|i| Threat::Boat(i as u16)),
        );
        out
    }

    /// The mounts pick their threats, aim and fire; a second of fire may knock one down.
    fn update_mounts(&mut self) {
        let fs = self.fs.max(1);
        let dt = 1.0 / fs as f32;
        let threats: Vec<(Threat, [f64; 3], [f64; 3])> = self
            .threats()
            .into_iter()
            .filter_map(|t| self.threat_at(t).map(|(p, v)| (t, p, v)))
            .collect();
        for i in 0..self.naval.mounts.len() {
            let c = self.naval.mounts[i];
            let ship_gone =
                self.objects.get(c.ship as usize).is_none_or(|o| o.flags & oflags::DESTROYED != 0);
            let mut c2 = c;
            c2.firing = false;
            if c2.reload > 0.0 {
                c2.reload = (c2.reload - dt).max(0.0);
                if c2.reload == 0.0 {
                    c2.rounds = CIWS_ROUNDS;
                }
            }
            let near = threats
                .iter()
                .map(|&(t, p, v)| (t, p, v, len([p[0] - c.at[0], p[1] - c.at[1], p[2] - c.at[2]])))
                .filter(|&(t, _, _, r)| {
                    r < if matches!(t, Threat::Boat(_)) { CIWS_BOAT_REACH } else { CIWS_REACH }
                })
                .min_by(|a, b| a.3.total_cmp(&b.3));
            c2.target = near.map(|n| n.0);
            if let Some((t, p, v, r)) =
                near.filter(|_| !ship_gone && c2.rounds > 0 && c2.reload == 0.0)
            {
                // Lead the threat by the rounds' time of flight.
                let tof = r / CIWS_MUZZLE;
                let aim = [
                    p[0] + v[0] * tof - c.at[0],
                    p[1] + v[1] * tof - c.at[1],
                    p[2] + v[2] * tof - c.at[2],
                ];
                c2.az = aim[0].atan2(aim[1]) as f32;
                c2.el = aim[2].atan2(aim[0].hypot(aim[1])) as f32;
                c2.firing = true;
                c2.rounds -= (CIWS_RATE / fs).max(1);
                if c2.rounds <= 0 {
                    c2.rounds = 0;
                    c2.reload = CIWS_RELOAD;
                }
                if !c.firing {
                    self.emit(SimEvent::CiwsFiring { mount: i as u8 });
                }
                if self.frame % TRACER_EVERY == 0 {
                    let d = norm(aim);
                    let mut j = || (self.rng.random(1000) - 500) as f64 / 500.0 * 0.004;
                    let spread = [j(), j(), j()];
                    let vel = [
                        (d[0] + spread[0]) * CIWS_MUZZLE,
                        (d[1] + spread[1]) * CIWS_MUZZLE,
                        (d[2] + spread[2]) * CIWS_MUZZLE,
                    ];
                    let ttl = (TRACER_SECS * fs as f64) as i32;
                    self.naval.shells.push(Shell { pos: c.at, vel, ttl, flak: false, pilot: 0 });
                }
                let pct = match t {
                    Threat::Missile(_) => CIWS_KILL_MISSILE_PCT,
                    Threat::Boat(_) => CIWS_KILL_BOAT_PCT,
                };
                // A second's chance spread over its frames (a roll of its own: the simulation's
                // generator is drawn in step with the tracers').
                let id = match t {
                    Threat::Missile(k) => k as u64,
                    Threat::Boat(b) => 0x100 + b as u64,
                };
                let roll = mix(self.rng.hold as u64
                    ^ ((self.frame as u64) << 20)
                    ^ ((i as u64) << 12)
                    ^ id);
                if (roll % (100 * fs as u64)) < pct as u64 {
                    self.ciws_kill(t, p);
                }
            }
            self.naval.mounts[i] = c2;
        }
    }

    /// A mount knocked threat `t` down at `p` (m).
    fn ciws_kill(&mut self, t: Threat, p: [f64; 3]) {
        let (x, y) = ((p[0] / M_PER_MAP) as i32, (0x8000 as f64 - p[1] / M_PER_MAP) as i32);
        let alt = (crate::math::view_z_f(p[2] / FT_M) as i32).max(SEA_BLAST_ALT);
        match t {
            Threat::Missile(k) => {
                self.proj[k as usize].ttl = 0;
                self.emit(SimEvent::Explosion { x, y, alt, kind: 0x10 });
                self.msg(text!("sim-ciws-missile-down"));
            }
            Threat::Boat(i) => {
                let i = i as usize;
                self.units[i].destroyed = true;
                self.emit(SimEvent::Explosion { x, y, alt: SEA_BLAST_ALT, kind: 0x10 });
                self.emit(SimEvent::AaaDestroyed { unit: i, by: None });
            }
        }
    }

    /// The fleet's mounts and their tracers (for drawing).
    pub fn naval(&self) -> &Naval {
        &self.naval
    }
}

/// Attack boats: how far ahead of a ship they stop weaving and run straight in (m), how
/// widely they weave (rad) and how fast (rad/s of the weave's phase), the distance at which
/// they blow up against a ship's side and the carrier's (m), and how far out they look for
/// open water to start from (m).
const RUN_IN: f64 = 700.0;
const WEAVE: f64 = 0.45;
const WEAVE_RATE: f64 = 0.35;
const BOAT_HIT_SHIP: f64 = 45.0;
const BOAT_HIT_CARRIER: f64 = 150.0;
const USV_TURN: f64 = 0.35;

impl Sim {
    /// Open water `km` from the fleet's carrier (else its first ship): the ship's middle (m
    /// east, north), the bearing out to it (rad) and the ship. None without a fleet or water.
    fn fleet_sea_spot(&mut self, km: f64) -> Option<([f64; 2], f64, usize)> {
        let home = *self.fleet().first()?;
        self.sea_spot_off(home, km)
    }

    /// Open water `km` from ship `home`: its middle (m east, north), the bearing out (rad).
    fn sea_spot_off(&mut self, home: usize, km: f64) -> Option<([f64; 2], f64, usize)> {
        let c = crate::asw::object_metres(&self.objects[home]);
        let wet = |s: &Sim, p: [f64; 2]| {
            !s.water.known()
                || s.water.wet((p[0] / M_PER_MAP) as i32, (0x8000 as f64 - p[1] / M_PER_MAP) as i32)
        };
        let r = km * 1000.0;
        let start = self.rng.random(16) as f64;
        let dir =
            (0..16).map(|k| (start + k as f64) / 16.0 * std::f64::consts::TAU).find(|&a| {
                let p = [c[0] + a.sin() * r, c[1] + a.cos() * r];
                let mid = [c[0] + a.sin() * r * 0.5, c[1] + a.cos() * r * 0.5];
                wet(self, p) && wet(self, mid)
            })?;
        Some((c, dir, home))
    }

    /// Test aid: a salvo of `count` sea-skimmers fired from open water `km` out at the fleet's
    /// carrier (as a submarine there would). The number fired.
    pub fn test_sea_salvo(&mut self, count: usize, km: f64) -> usize {
        let Some((c, dir, home)) = self.fleet_sea_spot(km) else { return 0 };
        let r = km * 1000.0;
        let at = [c[0] + dir.sin() * r, c[1] + dir.cos() * r];
        (0..count).filter(|_| self.launch_sea_missile(at, home, 0)).count()
    }

    /// Launches `count` attack boats `km` out from the fleet's carrier (else its first ship)
    /// on open water, at the carrier's group. The number placed (none without a fleet or open
    /// water).
    pub fn spawn_usv_swarm(&mut self, count: usize, km: f64) -> usize {
        let Some(&home) = self.fleet().first() else { return 0 };
        self.spawn_swarm_at(home, count, km).len()
    }

    /// A campaign sortie's raid: attack boats racing in at carrier `carrier`'s group for
    /// player `player` to fight (his primary objective: sink them all before one reaches a
    /// ship). The one he fought before is gone.
    pub fn start_raid_for(&mut self, player: u8, carrier: usize, seed: u32) {
        self.end_raid_of(player);
        let count = RAID_BOATS.0 + (seed as usize) % (RAID_BOATS.1 + 1);
        let boats = self.spawn_swarm_at(carrier, count, RAID_KM);
        if !boats.is_empty() {
            let boats = boats.into_iter().map(|b| b as u16).collect();
            self.naval.raids.push(Raid { player, boats, hit: false });
        }
    }

    /// Player `player`'s raid ends: its boats still going leave.
    fn end_raid_of(&mut self, player: u8) {
        let Some(k) = self.naval.raids.iter().position(|r| r.player == player) else { return };
        let r = self.naval.raids.remove(k);
        for b in r.boats {
            if let Some(u) = self.units.get_mut(b as usize).filter(|u| !u.destroyed) {
                u.destroyed = true;
                u.gone = true;
            }
        }
    }

    /// The raids: one whose boats are all sunk, none on a ship, is its player's objective; one
    /// whose player has gone home ends.
    fn check_raids(&mut self) {
        for k in (0..self.naval.raids.len()).rev() {
            let r = self.naval.raids[k].clone();
            let p = r.player as usize;
            if !self.pilot_exists(p) || self.pilot_absent(r.player) {
                self.end_raid_of(r.player);
                continue;
            }
            // (A boat's slot taken by another unit since: that boat is long gone.)
            let gone = |b: u16| {
                self.units.get(b as usize).is_none_or(|u| u.destroyed || u.kind != GroundKind::Usv)
            };
            if !r.boats.iter().all(|&b| gone(b)) {
                continue;
            }
            self.naval.raids.remove(k);
            if r.hit {
                continue;
            }
            let back = self.pl.id as usize;
            self.activate(p);
            if self.pl.targets[0].mission_type == RAID_MISSION {
                self.mark_target_reached(0);
            }
            self.msg(text!("sim-raid-beaten"));
            self.activate(back);
        }
    }

    /// `count` attack boats `km` out from ship `home` on open water, racing in at its group
    /// (`Sim::units` indices; none without open water).
    fn spawn_swarm_at(&mut self, home: usize, count: usize, km: f64) -> Vec<usize> {
        let Some((c, dir, _)) = self.sea_spot_off(home, km) else { return Vec::new() };
        self.arm_fleet();
        if self.aaa_rng.is_none() {
            self.aaa_rng = Some(crate::rng::Rng::new(self.rng.hold ^ 0x05EA_BAB7));
        }
        let r = km * 1000.0;
        let mut placed = Vec::new();
        for k in 0..count {
            // Abreast, some 60 m apart.
            let side = (k as f64 - (count as f64 - 1.0) / 2.0) * 60.0;
            let (e, n) =
                (c[0] + dir.sin() * r + dir.cos() * side, c[1] + dir.cos() * r - dir.sin() * side);
            let heading = (dir + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) as f32;
            let fine = crate::aaa::Fine { e: e as f32, n: n as f32, heading };
            let x = (e / M_PER_MAP) as i32;
            let y = (0x8000 as f64 - n / M_PER_MAP) as i32;
            self.units.push(crate::aaa::GroundUnit {
                kind: GroundKind::Usv,
                x,
                y,
                fine: Some(fine),
                // The weave's phase.
                cooldown: (k as i32) * 37,
                ..Default::default()
            });
            placed.push(self.units.len() - 1);
        }
        self.msg(text!("sim-usv-swarm"));
        placed
    }

    /// One frame of the attack boats: weaving in at the nearest ship of the fleet, straight in
    /// at the end; against its side they blow up.
    fn drive_boats(&mut self) {
        let dt = 1.0 / self.fs.max(1) as f64;
        let fleet: Vec<(usize, [f64; 2], bool)> = self
            .fleet()
            .into_iter()
            .filter(|&o| self.objects[o].flags & oflags::DESTROYED == 0)
            .map(|o| {
                (
                    o,
                    crate::asw::object_metres(&self.objects[o]),
                    self.objects[o].flags & oflags::CARRIER != 0,
                )
            })
            .collect();
        let water = self.water.clone();
        let t = self.frame as f64 * dt;
        let mut blasts = Vec::new();
        for i in 0..self.units.len() {
            let u = self.units[i];
            if u.kind != GroundKind::Usv || u.destroyed {
                continue;
            }
            let Some(mut f) = u.fine else { continue };
            let at = [f.e as f64, f.n as f64];
            let Some(&(ship, to, carrier)) =
                fleet.iter().min_by(|a, b| dist(a.1, at).total_cmp(&dist(b.1, at)))
            else {
                continue;
            };
            let d = dist(to, at);
            let reach = if carrier { BOAT_HIT_CARRIER } else { BOAT_HIT_SHIP };
            if d < reach {
                blasts.push((i, ship));
                continue;
            }
            let bearing = (to[0] - at[0]).atan2(to[1] - at[1]);
            let weave = if d > RUN_IN {
                WEAVE * (t * WEAVE_RATE * 6.0 + u.cooldown as f64).sin()
            } else {
                0.0
            };
            let want = bearing + weave;
            let turn = ((want - f.heading as f64 + std::f64::consts::PI)
                .rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI)
                .clamp(-USV_TURN * dt, USV_TURN * dt);
            f.heading = (f.heading as f64 + turn).rem_euclid(std::f64::consts::TAU) as f32;
            let next = [
                at[0] + f.heading.sin() as f64 * USV_SPEED * dt,
                at[1] + f.heading.cos() as f64 * USV_SPEED * dt,
            ];
            let (mx, my) =
                ((next[0] / M_PER_MAP) as i32, (0x8000 as f64 - next[1] / M_PER_MAP) as i32);
            if !water.known() || water.wet(mx, my) {
                f.e = next[0] as f32;
                f.n = next[1] as f32;
            } else {
                // Land ahead: round it.
                f.heading = (f.heading as f64 + USV_TURN * 2.0 * dt)
                    .rem_euclid(std::f64::consts::TAU) as f32;
            }
            let v = &mut self.units[i];
            v.fine = Some(f);
            v.x = (f.e as f64 / M_PER_MAP) as i32;
            v.y = (0x8000 as f64 - f.n as f64 / M_PER_MAP) as i32;
        }
        for (i, ship) in blasts {
            self.boat_hit(i, ship);
        }
    }

    /// Attack boat `i` blows up against ship `t`: an escort sinks, the carrier takes it.
    fn boat_hit(&mut self, i: usize, t: usize) {
        let u = self.units[i];
        self.units[i].destroyed = true;
        self.units[i].gone = true;
        for r in &mut self.naval.raids {
            r.hit |= r.boats.contains(&(i as u16));
        }
        self.emit(SimEvent::Explosion { x: u.x, y: u.y, alt: SEA_BLAST_ALT, kind: 0x10 });
        let o = self.objects[t];
        let name = self.place_name(t);
        if o.flags & oflags::CARRIER != 0 {
            self.msg(text!("sim-carrier-hit-boat", ship = name));
        } else {
            self.msg(text!("sim-ship-sunk-boat", ship = name));
            self.destroy_ground(t as i32);
        }
    }
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// A well-mixed hash of `x` (splitmix64's finaliser).
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A wu position's map point.
fn pm_map(p: [f64; 3]) -> (i32, i32) {
    (((p[0] / 32.0) as i32) & 0xFFFF, ((p[1] / 32.0) as i32) & 0xFFFF)
}

/// An attack boat's top speed (m/s).
pub const USV_SPEED: f64 = 22.0;
