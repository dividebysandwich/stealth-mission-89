//! Low-level air defences (`SimOptions::air_defence`): radar-laid
//! ZSU-23-4 Shilkas and optically laid S-60 batteries around the hostile airfields and the
//! targets, and MANPADS teams (SA-14) in the towns. They make flying low over defended places
//! dangerous: the Shilka's Gun Dish radar sees the player like any other radar (EMV), the S-60
//! and the MANPADS need to see him (day or moonlight, clouds).
//!
//! The units live outside the world object list and draw from their own random stream, so the
//! classic behaviour is untouched while the option is off. Shells fly in metres (x east,
//! y north, z up from the ground) with gravity and no drag.

use serde::{Deserialize, Serialize};
use stealth_mission::Mission;

use crate::Sim;
use crate::math::{approx, bearing, s16};
use crate::radar::DetectCtx;
use crate::radar::detect;
use crate::rng::Rng;
use crate::types::*;

/// Metres per map unit (32 world units of 0.48828125 m).
pub const M_PER_MAP: f64 = 15.625;
/// A town's defenders stand round its edge: from and to these shares of its radius.
const TOWN_RING: (f64, f64) = (0.8, 1.05);
const M_PER_WU: f64 = M_PER_MAP / 32.0;
const FT: f64 = 0.3048;
const G: f64 = 9.81;

/// The Gun Dish radar for [`detect`]: lethality (km), threat tier, Doppler.
const GUN_DISH: (i32, i32, i32) = (12, 1, 1);
/// SA-14 (missile table index).
pub const SA14: i32 = 0x21;
/// The friendly side's missiles: Stinger (MANPADS) and Patriot (MIM-104F PAC-3).
pub const STINGER: i32 = 46;
pub const PATRIOT: i32 = 45;
/// Launcher of a MANPADS missile (fire and forget: no launcher radar to keep its lock).
pub const MANPADS_LAUNCHER: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GroundKind {
    /// ZSU-23-4 Shilka: four 23 mm guns, Gun Dish radar.
    #[default]
    Zsu,
    /// S-60: 57 mm gun with timed shells, optically laid.
    S60,
    /// MANPADS team with SA-14s (Stingers on the friendly side).
    Manpads,
    /// Patriot M901 launcher: four missiles, fires on its battery radar's track.
    PatriotLauncher,
    /// Patriot AN/MPQ-53 radar: tracks for the launchers around it.
    PatriotRadar,
    /// Gepard: two 35 mm guns, search and tracking radar.
    Gepard,
    /// Drone command center: a bunker with a launch ramp, flies surveillance and attack drones
    /// (no gun of its own).
    DroneCenter,
    /// T-90 main battle tank (125 mm gun).
    T90,
    /// Leopard 2 main battle tank (120 mm gun).
    Leopard2,
    /// M2 Bradley infantry fighting vehicle: 25 mm gun, TOW missiles and a rifle squad.
    Bradley,
    /// BMP-2 infantry fighting vehicle: 30 mm gun, Konkurs missiles and a rifle squad.
    Bmp2,
}

impl GroundKind {
    pub fn name(self) -> &'static str {
        match self {
            GroundKind::Zsu => "ZSU-23-4",
            GroundKind::S60 => "S-60",
            GroundKind::Manpads => "SA-14",
            GroundKind::PatriotLauncher => "Patriot",
            GroundKind::PatriotRadar => "Patriot radar",
            GroundKind::Gepard => "Gepard",
            GroundKind::DroneCenter => "Drone command center",
            GroundKind::T90 => "T-90",
            GroundKind::Leopard2 => "Leopard 2",
            GroundKind::Bradley => "M2 Bradley",
            GroundKind::Bmp2 => "BMP-2",
        }
    }

    /// Name on a side: the friendly MANPADS teams carry Stingers.
    pub fn name_on(self, blue: bool) -> &'static str {
        match self {
            GroundKind::Manpads if blue => "Stinger",
            _ => self.name(),
        }
    }

    /// Patriot launcher or radar (no gun, no MANPADS).
    pub fn patriot(self) -> bool {
        matches!(self, GroundKind::PatriotLauncher | GroundKind::PatriotRadar)
    }

    /// No gun and no MANPADS of its own to fire at aircraft: a Patriot launcher or radar, a
    /// drone command center, a tank or an infantry fighting vehicle.
    pub fn unarmed(self) -> bool {
        self.patriot() || self == GroundKind::DroneCenter || self.armour()
    }

    /// A main battle tank.
    pub fn tank(self) -> bool {
        matches!(self, GroundKind::T90 | GroundKind::Leopard2)
    }

    /// An infantry fighting vehicle: carries the troops that take a place.
    pub fn carrier(self) -> bool {
        matches!(self, GroundKind::Bradley | GroundKind::Bmp2)
    }

    /// A tank or an infantry fighting vehicle: fights other ground units.
    pub fn armour(self) -> bool {
        self.tank() || self.carrier()
    }

    /// Goes with a convoy: armour and the tracked air defence guns.
    pub fn mobile(self) -> bool {
        self.armour() || matches!(self, GroundKind::Zsu | GroundKind::Gepard)
    }

    /// Muzzle velocity (m/s), slant range (m) and highest target (m) it engages.
    fn gun(self) -> (f64, f64, f64) {
        match self {
            GroundKind::Zsu => (970.0, 2500.0, 2000.0),
            GroundKind::S60 => (1000.0, 6000.0, 4500.0),
            GroundKind::Manpads => (0.0, 5000.0, 3500.0),
            GroundKind::Gepard => (1175.0, 3500.0, 3000.0),
            _ => (0.0, 0.0, 0.0),
        }
    }
}

/// The way air defence unit `k` stands (rad, clockwise from north): each its own, spread by the
/// golden angle. A drone command center's rail points this way.
pub fn unit_heading(k: usize) -> f32 {
    (k as f32 * 2.399_963).rem_euclid(std::f32::consts::TAU)
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct GroundUnit {
    pub kind: GroundKind,
    /// Map units (x east, y south).
    pub x: i32,
    pub y: i32,
    pub destroyed: bool,
    /// The Gun Dish holds the player (Shilka), or the crew sees him.
    pub tracking: bool,
    /// Where the guns or the launcher point: azimuth (radians clockwise from north) and
    /// elevation.
    pub aim_az: f32,
    pub aim_el: f32,
    /// Frames left of the burst being fired, and until the next one may start.
    pub burst: i32,
    pub cooldown: i32,
    /// Patriot launcher: missiles ready.
    #[serde(default)]
    pub rounds: i32,
    /// A vehicle's own position and the way its hull points (`x`, `y` are this rounded).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine: Option<Fine>,
    /// Destroyed without a wreck: left the world (a convoy that took its place hands over to
    /// the new garrison), its slot free for another unit.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gone: bool,
}

/// A vehicle's position in metres (east, north) and its hull's heading (rad, clockwise from
/// north).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Fine {
    pub e: f32,
    pub n: f32,
    pub heading: f32,
}

impl GroundUnit {
    /// Position in metres (east, north, up).
    pub fn metres(&self) -> [f64; 3] {
        match self.fine {
            Some(f) => [f.e as f64, f.n as f64, 2.0],
            None => [self.x as f64 * M_PER_MAP, (0x8000 - self.y) as f64 * M_PER_MAP, 2.0],
        }
    }

    /// Position in map units, between the units' grid.
    pub fn map_xy(&self) -> (f64, f64) {
        match self.fine {
            Some(f) => (f.e as f64 / M_PER_MAP, 0x8000 as f64 - f.n as f64 / M_PER_MAP),
            None => (self.x as f64, self.y as f64),
        }
    }
}

/// A shell in flight (metres, x east, y north, z up; m/s).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shell {
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    /// Frames left in flight; an S-60 shell bursts when they run out.
    pub ttl: i32,
    pub flak: bool,
    /// The pilot it was fired at.
    #[serde(default)]
    pub pilot: u8,
}

/// A pilot as the air defences see him this frame.
struct Track {
    k: usize,
    me: [f64; 3],
    vel: [f64; 3],
    p: crate::PlayerView,
    emv: i32,
    on_ground: bool,
    lockheed: bool,
    comanche: bool,
}

/// What the AAA does to the player in one frame.
const ZSU_HIT_RADIUS: f64 = 9.0;
const ZSU_HIT_CHANCE: i32 = 450; // per mille, per tracer passing within the radius
const FLAK_LETHAL: f64 = 14.0;
const FLAK_REACH: f64 = 40.0;
/// A bomb's or missile's blast kills a tank this close (map units, ~16 m), an infantry fighting
/// vehicle this close (~25 m); one gun round in so many kills one.
const TANK_BLAST: f64 = 1.0;
const IFV_BLAST: f64 = 1.6;
const TANK_ROUND_ODDS: i32 = 30;
const IFV_ROUND_ODDS: i32 = 10;
/// Units and shells at most.
const MAX_UNITS: usize = 72;
const MAX_SHELLS: usize = 160;

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn len(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Smallest distance from the origin to the segment a .. a + d.
pub(crate) fn closest(a: [f64; 3], d: [f64; 3]) -> f64 {
    let k = (-dot(a, d) / dot(d, d).max(1e-9)).clamp(0.0, 1.0);
    len(add(a, mul(d, k)))
}

/// Unit vector from azimuth (clockwise from north) and elevation.
fn direction(az: f64, el: f64) -> [f64; 3] {
    [az.sin() * el.cos(), az.cos() * el.cos(), el.sin()]
}

/// Where to point a gun at `from` with shells of speed `v` to meet a target at `to` moving at
/// `vel`, gravity included: (azimuth, elevation, time of flight).
pub fn lead(from: [f64; 3], to: [f64; 3], vel: [f64; 3], v: f64) -> (f64, f64, f64) {
    let mut t = len(sub(to, from)) / v;
    let mut aim = to;
    for _ in 0..4 {
        let at = add(to, mul(vel, t));
        aim = add(at, [0.0, 0.0, 0.5 * G * t * t]);
        t = len(sub(at, from)) / v;
    }
    let d = sub(aim, from);
    (d[0].atan2(d[1]), d[2].atan2(d[0].hypot(d[1])), t)
}

impl Sim {
    pub fn units(&self) -> &[GroundUnit] {
        &self.units
    }

    /// The units (scenario tests place their own).
    pub fn units_mut(&mut self) -> &mut Vec<GroundUnit> {
        &mut self.units
    }

    /// The town at map point (`x`, `y`): the nearest whose disc holds it.
    pub fn town_at(&self, x: i32, y: i32) -> Option<crate::Town> {
        let towns = self.tiles.as_ref()?.towns.iter();
        towns
            .filter(|t| approx(t.x - x, t.y - y) <= t.r)
            .min_by_key(|t| approx(t.x - x, t.y - y))
            .copied()
    }

    /// Where the defenders of a place at (`x`, `y`) in a town stand: the ring round the town's
    /// edge (centre, inner and outer radius in map units).
    pub(crate) fn town_ring(&self, x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        self.town_at(x, y).map(|t| {
            let lo = (t.r as f64 * TOWN_RING.0) as i32;
            (t.x, t.y, lo, ((t.r as f64 * TOWN_RING.1) as i32).max(lo + 1))
        })
    }

    pub fn aaa_shells(&self) -> &[Shell] {
        &self.aaa_shells
    }

    /// Shells came down here (nobody may take them: a dedicated server keeps the last few).
    pub(crate) fn shells_down(&mut self, down: Vec<([f64; 2], bool)>) {
        self.shell_impacts.extend(down);
        if self.shell_impacts.len() > 256 {
            self.shell_impacts.drain(..128);
        }
    }

    /// The shells that came down on the ground since the last call (metres east, north; flak
    /// or not).
    pub fn take_shell_impacts(&mut self) -> Vec<([f64; 2], bool)> {
        std::mem::take(&mut self.shell_impacts)
    }

    /// The player in metres (east, north, up) and his velocity (m/s), as the gunners see it:
    /// from how he moved over the last frames (the flight model covers more ground than its
    /// airspeed says, and the flight path is not the nose's direction).
    fn player_metres(&mut self) -> ([f64; 3], [f64; 3]) {
        let p = &self.pl.p;
        let pos = [p.wx as f64 * M_PER_WU, p.wy as f64 * M_PER_WU, p.alt as f64 * FT];
        let fs = self.fs.max(1) as f64;
        let vel = match self.pl.aaa_track {
            // A jump (a new start position) is not a velocity.
            Some((last, v)) if len(sub(pos, last)) * fs < 1500.0 => {
                add(mul(v, 0.7), mul(sub(pos, last), fs * 0.3))
            }
            _ => [0.0; 3],
        };
        self.pl.aaa_track = Some((pos, vel));
        (pos, vel)
    }

    /// A random number from about -1 to 1, roughly normal.
    fn aaa_gauss(rng: &mut Rng) -> f64 {
        (0..3).map(|_| rng.random(1000) as f64 / 1000.0).sum::<f64>() * 1.15 - 1.73
    }

    /// Places the units (`Sim::with_options` with the option on).
    pub(crate) fn place_aaa(&mut self, m: &Mission) {
        let mut rng = Rng::new(m.rng.holdrand_exit ^ 0x5EAD_AAA0);
        let war = self.roe;
        let mut units = Vec::new();
        let around =
            |rng: &mut Rng, units: &mut Vec<GroundUnit>, x: i32, y: i32, kind, lo: i32, hi: i32| {
                let a = rng.random(0x4000) * 4;
                let r = (lo + rng.random(hi - lo)) as f64;
                let t = a as f64 / 65536.0 * std::f64::consts::TAU;
                units.push(GroundUnit {
                    kind,
                    x: (x + (t.sin() * r) as i32).clamp(0x40, 0x7FC0),
                    y: (y - (t.cos() * r) as i32).clamp(0x40, 0x7FC0),
                    destroyed: false,
                    tracking: false,
                    aim_az: t as f32,
                    aim_el: 0.3,
                    burst: 0,
                    cooldown: 0,
                    rounds: 0,
                    fine: None,
                    gone: false,
                });
            };
        let hostile = |o: &GroundObject| o.flags & (oflags::FRIENDLY | oflags::NEUTRAL) == 0;
        // Airfields: Shilkas, and in a war an S-60 battery (2 to 4 guns around one spot).
        for i in self.ground_units..self.objects.len() {
            let o = self.objects[i];
            if o.flags & oflags::AIRFIELD == 0 || !hostile(&o) || (o.x, o.y) == (0, 0) {
                continue;
            }
            for _ in 0..1 + rng.random(war + 1) {
                around(&mut rng, &mut units, o.x, o.y, GroundKind::Zsu, 26, 90);
            }
            if war > 0 {
                let (bx, by) = (o.x + rng.random(120) - 60, o.y + rng.random(120) - 60);
                for _ in 0..2 + rng.random(3) {
                    around(&mut rng, &mut units, bx, by, GroundKind::S60, 3, 8);
                }
            }
        }
        // The targets.
        for t in self.pl.targets {
            let Some(o) = usize::try_from(t.object).ok().and_then(|i| self.objects.get(i)) else {
                continue;
            };
            if (o.x, o.y) == (0, 0) || !hostile(o) {
                continue;
            }
            let (x, y) = (o.x, o.y);
            for _ in 0..1 + rng.random(war + 1) {
                around(&mut rng, &mut units, x, y, GroundKind::Zsu, 20, 70);
            }
            if rng.random(3) < war {
                let (bx, by) = (x + rng.random(100) - 50, y + rng.random(100) - 50);
                for _ in 0..2 + rng.random(2) {
                    around(&mut rng, &mut units, bx, by, GroundKind::S60, 3, 8);
                }
            }
        }
        // MANPADS in and around the towns (the other places of the world).
        for i in self.ground_units..self.objects.len() {
            let o = self.objects[i];
            if o.flags & (oflags::AIRFIELD | oflags::SHIP) != 0
                || !hostile(&o)
                || (o.x, o.y) == (0, 0)
            {
                continue;
            }
            let (x, y, lo, hi) = self.town_ring(o.x, o.y).unwrap_or((o.x, o.y, 15, 80));
            for _ in 0..rng.random(war + 2) {
                around(&mut rng, &mut units, x, y, GroundKind::Manpads, lo, hi);
            }
        }
        units.truncate(MAX_UNITS);
        self.units = units;
        self.aaa_rng = Some(rng);
    }

    /// One frame of the air defences: aiming, firing, the shells and the MANPADS launches.
    /// Each unit goes for the nearest pilot in the world.
    pub(crate) fn update_aaa(&mut self, present: &[usize]) {
        let Some(mut rng) = self.aaa_rng else { return };
        let fs = self.fs.max(1);
        let dt = 1.0 / fs as f64;
        let tracks: Vec<Track> = present
            .iter()
            .map(|&k| {
                self.activate(k);
                let (me, vel) = self.player_metres();
                Track {
                    k,
                    me,
                    vel,
                    p: self.pl.p,
                    emv: self.pl.emv,
                    on_ground: self.pl.on_ground,
                    lockheed: self.pl.lockheed,
                    comanche: self.pl.comanche,
                }
            })
            .collect();
        if tracks.is_empty() {
            return;
        }
        let mut units = std::mem::take(&mut self.units);
        self.aaa_pilot.resize(units.len(), tracks[0].k as u8);
        let vis_km = self.visual_range();
        // Accuracy: one standard deviation (radians) with a radar track / laid by eye.
        let skill = 1.5 - 0.2 * self.diff as f64;
        let mut fired: Vec<Shell> = Vec::new();
        let mut manpads: Vec<usize> = Vec::new();
        let mut rwr: Vec<(usize, RwrTone)> = Vec::new();
        for (i, u) in units.iter_mut().enumerate() {
            // Only the classic guns fight the player (the friendly ones never do).
            if u.destroyed || self.aaa_blue(i) || u.kind.unarmed() || u.kind == GroundKind::Gepard {
                continue;
            }
            u.cooldown = (u.cooldown - 1).max(0);
            let t = if tracks.len() == 1 {
                &tracks[0]
            } else {
                tracks
                    .iter()
                    .min_by_key(|t| approx(s16(t.p.mx - u.x), s16(t.p.my - u.y)))
                    .expect("a pilot")
            };
            self.aaa_pilot[i] = t.k as u8;
            let (me, vel, on_ground) = (t.me, t.vel, t.on_ground);
            let ctx = DetectCtx {
                px: t.p.mx,
                py: t.p.my,
                view_z: t.p.view_z,
                height: t.p.height,
                heading: t.p.heading,
                velocity: t.p.velocity,
                diff: self.diff,
                lockheed: t.lockheed,
                comanche: t.comanche,
                grid: &self.grid,
            };
            let from = u.metres();
            let d = sub(me, from);
            let dist = len(d);
            let (v0, reach, ceiling) = u.kind.gun();
            let range_km = approx(s16(t.p.mx - u.x), s16(t.p.my - u.y)) >> 6;
            let seen = range_km < vis_km
                && !on_ground
                && (!self.weather.any()
                    || self.weather.los_clear((u.x, u.y, 0), (t.p.mx, t.p.my, t.p.alt.max(0))));
            let radar = u.kind == GroundKind::Zsu && {
                let (score, _, _) = detect(u.x, u.y, 0, 1, Some(GUN_DISH), &ctx);
                score + t.emv > 100
            };
            if radar && !u.tracking && !rwr.iter().any(|r| r.0 == t.k) {
                rwr.push((t.k, RwrTone::Scan));
            }
            u.tracking = radar || seen;
            let engage =
                u.tracking && !on_ground && dist < reach && me[2] < ceiling && me[2] > 15.0;
            if !engage {
                u.burst = 0;
                continue;
            }
            let pilot = t.k as u8;
            let (az, el, t) = if u.kind == GroundKind::Manpads {
                (d[0].atan2(d[1]), d[2].atan2(d[0].hypot(d[1])), 0.0)
            } else {
                lead(from, me, vel, v0)
            };
            u.aim_az = az as f32;
            u.aim_el = el as f32;
            match u.kind {
                GroundKind::Manpads => {
                    if u.cooldown == 0 {
                        manpads.push(i);
                        u.cooldown = 60 * fs;
                    }
                }
                GroundKind::Zsu => {
                    if u.burst == 0 && u.cooldown == 0 {
                        u.burst = fs * 3 / 2 + rng.random(fs);
                    }
                    if u.burst > 0 {
                        u.burst -= 1;
                        if u.burst == 0 {
                            u.cooldown = fs * 2 + rng.random(fs * 2);
                        }
                        // One tracer (of the many rounds) every other frame.
                        if u.burst & 1 == 0 {
                            let sigma = if radar { 0.004 } else { 0.010 } * skill;
                            let a = az + Self::aaa_gauss(&mut rng) * sigma;
                            let e = el + Self::aaa_gauss(&mut rng) * sigma;
                            let muzzle = add(from, [0.0, 0.0, 1.5]);
                            fired.push(Shell {
                                pos: muzzle,
                                vel: mul(direction(a, e), v0),
                                ttl: (reach * 1.3 / v0 * fs as f64) as i32,
                                flak: false,
                                pilot,
                            });
                        }
                    }
                }
                GroundKind::S60 => {
                    if u.cooldown == 0 {
                        u.cooldown = fs + rng.random(fs);
                        let sigma = 0.008 * skill;
                        let a = az + Self::aaa_gauss(&mut rng) * sigma;
                        let e = el + Self::aaa_gauss(&mut rng) * sigma;
                        // The fuse is set for the predicted time, a little off.
                        let fuse = t * (1.0 + Self::aaa_gauss(&mut rng) * 0.03);
                        fired.push(Shell {
                            pos: add(from, [0.0, 0.0, 1.5]),
                            vel: mul(direction(a, e), v0),
                            ttl: ((fuse * fs as f64) as i32).max(1),
                            flak: true,
                            pilot,
                        });
                    }
                }
                GroundKind::PatriotLauncher
                | GroundKind::PatriotRadar
                | GroundKind::Gepard
                | GroundKind::DroneCenter
                | GroundKind::T90
                | GroundKind::Leopard2
                | GroundKind::Bradley
                | GroundKind::Bmp2 => {}
            }
        }
        self.units = units;
        for (k, tone) in rwr {
            self.activate(k);
            self.emit(SimEvent::Rwr(tone));
        }
        let room = MAX_SHELLS.saturating_sub(self.aaa_shells.len());
        fired.truncate(room);
        if self.record_shells {
            self.new_shells.extend_from_slice(&fired);
        }
        self.aaa_shells.extend(fired);
        for i in manpads {
            self.activate(self.aaa_pilot[i] as usize);
            self.launch_manpads(i);
        }

        // Shells: gravity, the pilot passing through a burst of tracers, timed bursts.
        let mut hits = vec![(0, 0); tracks.len()];
        let mut bursts = Vec::new();
        let mut shells = std::mem::take(&mut self.aaa_shells);
        let mut down = Vec::new();
        shells.retain_mut(|s| {
            let n = tracks.iter().position(|t| t.k == s.pilot as usize);
            let (me, vel) = n.map_or(([f64::MAX; 3], [0.0; 3]), |n| (tracks[n].me, tracks[n].vel));
            let step = mul(s.vel, dt);
            let rel = sub(s.pos, me);
            let rel_step = sub(step, mul(vel, dt));
            s.pos = add(s.pos, step);
            s.vel[2] -= G * dt;
            s.ttl -= 1;
            if s.pos[2] < 0.0 {
                down.push(([s.pos[0], s.pos[1]], s.flak));
                return false;
            }
            let near = if n.is_some() { closest(rel, rel_step) } else { f64::MAX };
            if !s.flak && near < ZSU_HIT_RADIUS && rng.random(1000) < ZSU_HIT_CHANCE {
                hits[n.unwrap_or(0)].0 += 1;
                return false;
            }
            if s.flak && (s.ttl <= 0 || near < FLAK_LETHAL * 0.5) {
                let d = len(sub(s.pos, me));
                let x = (s.pos[0] / M_PER_MAP) as i32;
                let y = 0x8000 - (s.pos[1] / M_PER_MAP) as i32;
                bursts.push((n.map_or(0, |n| tracks[n].k), x, y, (s.pos[2] / FT) as i32));
                if d < FLAK_LETHAL
                    || (d < FLAK_REACH
                        && rng.random(1000)
                            < ((FLAK_REACH - d) / (FLAK_REACH - FLAK_LETHAL) * 600.0) as i32)
                {
                    hits[n.unwrap_or(0)].1 += 1;
                }
                return false;
            }
            s.ttl > 0
        });
        self.aaa_shells = shells;
        self.aaa_rng = Some(rng);
        self.shells_down(down);
        for (k, x, y, alt) in bursts {
            self.activate(k);
            self.emit(SimEvent::Flak { x, y, alt });
        }
        for (n, &(hits, flak_hits)) in hits.iter().enumerate() {
            self.activate(tracks[n].k);
            for (n, text) in [(hits, "Hit by AAA"), (flak_hits, "Hit by flak")] {
                if n > 0 && !self.training() {
                    self.msg(text);
                    self.player_hit(false);
                    self.emit(SimEvent::Explosion {
                        x: self.pl.p.mx,
                        y: self.pl.p.my,
                        alt: self.pl.p.view_z,
                        kind: -1,
                    });
                }
            }
        }
    }

    /// A MANPADS team fires an SA-14 if a missile slot is free (never one in flight).
    fn launch_manpads(&mut self, i: usize) {
        let u = self.units[i];
        if !self.may_launch(SA14, (u.x, u.y, 0)) {
            return;
        }
        let Some(k) = (0..8).find(|&k| self.proj[k].ttl == 0) else { return };
        let brg = bearing(s16(self.pl.p.mx - u.x), s16(u.y - self.pl.p.my));
        let lock = self.missile(SA14).lock_range as i32;
        self.proj[k] = Projectile {
            x: u.x,
            y: u.y,
            alt: 0,
            speed: 1,
            heading: s16(brg),
            pitch: 0x2000,
            ttl: s16(lock * self.fs * 16),
            spec: SA14,
            target: -1,
            launcher: MANPADS_LAUNCHER,
            ..Default::default()
        };
        if self.guided(SA14) {
            let tgt = self.player_target().p;
            self.launch_flight_at(k, (u.x, u.y), tgt);
        }
        self.msg("Missile launch");
        self.proj_pilot[k] = self.pl.id;
        self.pl.launches += 1;
        self.emit(SimEvent::MissileLaunched {
            slot: k,
            spec: SA14,
            launcher: Launcher::Manpads(i),
        });
    }

    /// Player bombs, missiles and gun rounds that went off on the ground this frame destroy the
    /// units nearby.
    pub(crate) fn aaa_damage(&mut self) {
        let fine = (32 * ROUND_FINE) as f64;
        let blasts: Vec<(f64, f64, bool, Option<u8>)> = self
            .events
            .iter()
            .zip(&self.event_credit)
            .filter_map(|(e, &by)| match *e {
                SimEvent::Explosion { x, y, alt, kind }
                    if alt < 60 && kind != -1 && kind != -16 =>
                {
                    Some((x as f64, y as f64, false, by))
                }
                SimEvent::RoundImpact { x, y, .. } => {
                    Some((x as f64 / fine, y as f64 / fine, true, by))
                }
                _ => None,
            })
            .collect();
        if blasts.is_empty() {
            return;
        }
        let Some(mut rng) = self.aaa_rng else { return };
        for k in 0..self.units.len() {
            let u = self.units[k];
            if u.destroyed {
                continue;
            }
            let (ux, uy) = u.map_xy();
            // Blast radius and the odds against a gun round (map units, 1 in n).
            let (r, odds) = match u.kind {
                k if k.tank() => (TANK_BLAST, TANK_ROUND_ODDS),
                k if k.carrier() => (IFV_BLAST, IFV_ROUND_ODDS),
                _ => (4.0, 6),
            };
            for &(x, y, round, by) in &blasts {
                let d = (x - ux).hypot(y - uy);
                // A gun round kills only now and then.
                if (round && d < 1.0 && rng.random(odds) == 0) || (!round && d < r) {
                    self.units[k].destroyed = true;
                    self.units[k].burst = 0;
                    if let Some(p) = by {
                        self.activate(p as usize);
                    }
                    self.msg(format!("{} destroyed", u.kind.name()));
                    self.emit(SimEvent::Explosion { x: u.x, y: u.y, alt: 0, kind: 0x10 });
                    self.emit(SimEvent::AaaDestroyed { unit: k, by });
                    break;
                }
            }
        }
        self.aaa_rng = Some(rng);
    }

    /// ZSU-23-4 radars for the radar warning receiver (range km, bearing from the player).
    pub(crate) fn aaa_emitters(&self, out: &mut Vec<Emitter>) {
        let (px, py) = (self.pl.p.mx, self.pl.p.my);
        for (i, u) in self.units.iter().enumerate() {
            if u.kind != GroundKind::Zsu || u.destroyed || self.aaa_blue(i) {
                continue;
            }
            let range = approx(s16(px - u.x), s16(py - u.y)) >> 6;
            if range >= GUN_DISH.0 {
                continue;
            }
            let launch = u.burst > 0;
            out.push(Emitter {
                source: EmitterSource::Gun(i),
                x: u.x,
                y: u.y,
                alt: 0,
                radar_type: 0,
                range,
                bearing: bearing(s16(u.x - px), s16(py - u.y)),
                lock_range: 3,
                painted: u.tracking,
                tracking: u.tracking,
                launch,
                priority: 8
                    + if launch { 300 } else { 0 }
                    + if u.tracking { 120 } else { 0 }
                    + if range < 3 { 60 } else { 0 }
                    + (GUN_DISH.0 - range) / 4,
                signal: detect(u.x, u.y, 0, 1, Some(GUN_DISH), &self.detect_ctx()).0.max(0),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lead_meets_a_crossing_target() {
        // A target 2 km north at 300 m flying east at 200 m/s: the shell meets it.
        let from = [0.0, 0.0, 0.0];
        let to = [0.0, 2000.0, 300.0];
        let vel = [200.0, 0.0, 0.0];
        let (az, el, t) = lead(from, to, vel, 970.0);
        let mut p = from;
        let mut v = mul(direction(az, el), 970.0);
        let dt = 0.001;
        let mut best = f64::MAX;
        let mut tt = 0.0;
        while tt < t * 1.5 {
            p = add(p, mul(v, dt));
            v[2] -= G * dt;
            tt += dt;
            best = best.min(len(sub(p, add(to, mul(vel, tt)))));
        }
        assert!(best < 2.0, "missed by {best} m");
        assert!(az > 0.0 && t > 2.0 && t < 2.3);
    }

    #[test]
    fn closest_approach() {
        assert!((closest([-5.0, 3.0, 0.0], [10.0, 0.0, 0.0]) - 3.0).abs() < 1e-9);
        assert!((closest([5.0, 0.0, 0.0], [10.0, 0.0, 0.0]) - 5.0).abs() < 1e-9);
    }
}
