//! Realistic missile guidance (`SimOptions::realistic_guidance`).
//!
//! Classic guidance steers a missile's heading and pitch straight at its target, a fixed angle per
//! frame, at a speed that only grows, from positions kept in whole map units (32 world units):
//! its flight path zigzags in steps of several degrees and it rolls back and forth with every
//! correction. Here a missile flies in 3D with exact positions:
//!
//! - **Motor**: a boost to full speed and, on missiles with a sustainer, a burn that holds it,
//!   for the real motor's burn time; afterwards it glides, slowed by drag (thinner up high) and
//!   by every turn it makes, and sped up by gravity diving (burning or not: a lobbed missile
//!   comes down on its target with the energy of its climb). Its smoke trail ends with the burn
//!   ([`Projectile::motor_visible`]).
//!   Once too slow to fly it stalls: no more lift or steering, it falls to the ground, so the
//!   launch altitude stretches its reach.
//! - **Turning**: the lift it can pull falls with the speed squared and the air density, so a
//!   slow missile high up can barely turn. It is roll stabilised and turns by skidding.
//! - **Guidance**: pure proportional navigation (N = 4): the velocity turns four times as fast
//!   as the line of sight to the target, plus a lift that carries the missile's weight, so it
//!   flies to the collision point instead of chasing the target.
//! - **Launch**: it flies straight for a moment first (the Hellfire along the aircraft's nose
//!   for up to 1 s against a distant target, so it can be lobbed). SAMs come off a rail
//!   (vertically from the vertical launchers). Against a distant target SAMs and the AIM-120
//!   climb at a loft angle that grows with the range, and only dive on the target once it is
//!   below them, so they reach it with the most energy left (pull up before firing an AIM-120
//!   and it has less to turn).
//! - **Fuse**: a proximity fuse that goes off at the closest approach within its radius.

use crate::Sim;
use crate::ballistics::GRAVITY;
use crate::math::{s16, view_z_f};
use crate::types::*;

/// A missile's motor and airframe: motor burn and smoke trail (s), seconds to full speed,
/// lateral acceleration limit (g) at full lift, launched vertically, and its drag as a multiple
/// of [`DRAG`].
#[derive(Debug, Clone, Copy)]
pub struct Profile {
    pub burn: f64,
    pub smoke: f64,
    pub boost: f64,
    pub g: f64,
    pub vertical: bool,
    pub drag: f64,
}

const fn rocket(burn: f64, boost: f64, g: f64) -> Profile {
    Profile { burn, smoke: burn, boost, g, vertical: false, drag: 1.0 }
}

const fn vertical(burn: f64, boost: f64, g: f64) -> Profile {
    Profile { burn, smoke: burn, boost, g, vertical: true, drag: 1.0 }
}

/// Burns the whole way (a turbojet or a long sustainer); the smoke shows for `smoke` s.
const fn cruise(smoke: f64, g: f64) -> Profile {
    Profile { burn: 1e9, smoke, boost: 3.0, g, vertical: false, drag: 1.0 }
}

/// The profile of a missile table entry; None for bombs and the other projectiles (they keep
/// the classic flight).
pub fn profile(spec: i32) -> Option<Profile> {
    Some(match spec {
        1 => rocket(25.0, 4.0, 12.0), // SA-2: booster, then a long sustainer
        2 => rocket(55.0, 5.0, 10.0), // SA-5
        3 | 8 => rocket(15.0, 2.0, 25.0), // SA-8, SA-N-4
        4 | 10 => vertical(12.0, 3.0, 25.0), // SA-10, SA-N-6
        5 | 11 | 42 => rocket(15.0, 3.0, 22.0), // SA-11, SA-N-7, SA-17
        6 => vertical(15.0, 4.0, 22.0), // SA-12
        7 => rocket(6.0, 1.5, 20.0),  // SA-13
        9 | 33 | 39 => rocket(6.0, 1.0, 16.0), // SA-N-5, SA-14, SA-9
        12 => rocket(26.0, 5.0, 15.0), // Hawk
        13 => rocket(3.0, 2.0, 25.0), // Rapier
        14 | 15 => rocket(10.0, 2.0, 10.0), // Tigercat, Seacat
        16 => rocket(2.2, 2.0, 20.0), // AA-2
        17 => rocket(3.0, 2.0, 35.0), // AA-8
        18 | 34 => rocket(8.0, 3.0, 15.0), // AA-6
        19 | 35 => rocket(6.0, 3.0, 20.0), // AA-7
        20 | 36 => rocket(12.0, 4.0, 15.0), // AA-9
        21 | 37 => rocket(6.0, 3.0, 30.0), // AA-10
        22 => rocket(8.0, 3.0, 35.0), // AIM-120
        23 | 32 => rocket(5.0, 2.0, 35.0), // AIM-9
        24 => rocket(6.0, 3.0, 20.0), // HARM
        25 => cruise(1e9, 10.0),      // Penguin: a sustainer the whole way
        26 => cruise(3.0, 10.0),      // Harpoon: booster, then its turbojet
        27 => rocket(4.0, 1.0, 15.0), // Maverick
        // Hellfire: light and blunt, it slows fast once burnt out (some 7 km from low down).
        47 => Profile { drag: 12.0, ..rocket(2.5, 1.0, 20.0) },
        31 => rocket(15.0, 4.0, 25.0),   // AIM-7
        41 => vertical(15.0, 4.0, 25.0), // SA-21
        43 => rocket(2.0, 2.0, 30.0),    // SA-22: a booster, then the dart glides
        44 => vertical(5.0, 2.0, 30.0),  // SA-15
        _ => return None,
    })
}

/// Seconds a launched missile flies straight before steering, with its target `range` wu away
/// (level): a moment to clear the launcher; the Hellfire up to 1 s against a distant target (it
/// can be lobbed), only the moment close by, where it has to dive on the target.
fn straight_secs(spec: i32, range: Option<f64>) -> f64 {
    match (spec, range) {
        (47, Some(r)) => (r / LOB_RANGE).clamp(STRAIGHT, 1.0),
        _ => STRAIGHT,
    }
}

/// Seconds a missile flies straight off the launcher at least; the range (wu) from which the
/// Hellfire flies straight for 1 s.
const STRAIGHT: f64 = 0.4;
const LOB_RANGE: f64 = 10_000.0;

/// Navigation constant.
const PN_GAIN: f64 = 4.0;
/// Cosine of the seeker's field of view off the flight path (60°): a target outside it (one
/// overshot, behind or passed above or below) is lost and the missile flies on unguided.
const GIMBAL_COS: f64 = 0.5;
/// Cosine of the cone (30°) the target has to come into before the seeker counts it as seen: one
/// at the edge of the view on turning over would soon slip out of it again.
const ACQUIRE_COS: f64 = 0.87;
/// Loft: turn rate toward the climb angle (1/s), the widest loft angle (radians), where it
/// starts (share of the lock range) and the share of the time to go that turning down to the
/// target may take before the missile pushes over.
const LOFT_GAIN: f64 = 1.5;
const LOFT_MAX: f64 = 0.6;
const LOFT_FROM: f64 = 0.3;
const PUSHOVER: f64 = 0.4;
/// Drag: deceleration per (speed² x air density) (1/wu), and per lateral acceleration.
const DRAG: f64 = 1.0e-5;
const INDUCED: f64 = 0.12;
/// Speed (wu/s) at which a missile pulls its full g at sea level; the least share of it left.
const FULL_LIFT_SPEED: f64 = 2500.0;
const MIN_LIFT: f64 = 0.08;
/// After burnout, a missile slower than this (wu/s) stalls and falls.
const MIN_SPEED: f64 = 500.0;
/// Seconds a missile flies at most.
const MAX_SECS: i32 = 150;
/// Proximity fuse radius (wu): SAMs, air-to-air missiles and against ground targets.
const FUSE_SAM: f64 = 110.0;
pub(crate) const FUSE_AAM: f64 = 50.0;
pub(crate) const FUSE_GROUND: f64 = 35.0;
/// Speed at launch from the ground (wu/s) and height of the rail.
const RAIL_SPEED: f64 = 150.0;
const RAIL_HEIGHT: f64 = 5.0;
/// SAM rail elevation above the line of sight to the target, and the lowest one (radians).
const RAIL_ABOVE: f64 = 0.25;
const RAIL_LOWEST: f64 = 0.3;

const BAM: f64 = 65536.0 / std::f64::consts::TAU;

type V3 = [f64; 3];

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len(a: V3) -> f64 {
    dot(a, a).sqrt()
}
/// The part of `a` across the unit vector `n`.
fn across(a: V3, n: V3) -> V3 {
    sub(a, mul(n, dot(a, n)))
}

/// Heading and pitch (BAM) of a direction (east, south, up).
pub fn heading_pitch(d: V3) -> (i32, i32) {
    (s16((d[0].atan2(-d[1]) * BAM) as i32), s16((d[2].atan2(d[0].hypot(d[1])) * BAM) as i32))
}

/// Unit vector of a heading and pitch (BAM): east, south, up.
pub fn direction(heading: i32, pitch: i32) -> V3 {
    let (h, p) = ((heading as u16) as f64 / BAM, s16(pitch) as f64 / BAM);
    [p.cos() * h.sin(), -p.cos() * h.cos(), p.sin()]
}

/// Air density relative to sea level at `alt` (wu).
fn density(alt: f64) -> f64 {
    (-alt.max(0.0) / 30_000.0).exp()
}

/// What a missile steers for or its fuse watches: position (wu: east, south, up) and velocity.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Target {
    pub p: V3,
    pub v: V3,
}

impl Projectile {
    /// The motor is burning, trailing smoke (always under classic guidance).
    pub fn motor_visible(&self) -> bool {
        self.flight.is_none_or(|f| f.smoke > 0)
    }
}

impl Sim {
    /// Realistic guidance flies missiles of type `spec`.
    pub(crate) fn guided(&self, spec: i32) -> bool {
        self.opts.realistic_guidance && profile(spec).is_some()
    }

    /// The player's position (wu: east, south, true altitude) and velocity.
    pub(crate) fn player_target(&self) -> Target {
        self.pilot_target(self.pl.id)
    }

    /// Pilot `k`'s aircraft as a target, like [`Self::player_target`].
    pub(crate) fn pilot_target(&self, k: u8) -> Target {
        let p = &self.pilot_ref(k as usize).p;
        Target {
            p: [p.wx as f64, (0x100000 - p.wy) as f64, p.alt as f64],
            v: crate::ballistics::release_velocity(p.heading, p.velocity, p.climb),
        }
    }

    /// Flight unit `j` as a target (at the altitude it is drawn at).
    pub(crate) fn unit_target(&self, j: usize) -> Target {
        let u = self.flights[j];
        Target {
            p: [u.wx as f64, u.wy as f64, u.alt as f64],
            v: mul(direction(u.heading, u.pitch), u.speed as f64),
        }
    }

    /// A world object (on the ground) as a target.
    pub(crate) fn object_target(&self, i: i32) -> Target {
        let o = self.obj(i);
        Target { p: [o.x as f64 * 32.0 + 16.0, o.y as f64 * 32.0 + 16.0, 0.0], v: [0.0; 3] }
    }

    /// Starts the realistic flight of the missile just put in slot `s` at `pos` (wu: east,
    /// south, true altitude) with velocity `vel`.
    pub(crate) fn start_flight(&mut self, s: usize, pos: V3, vel: V3) {
        let fs = self.fs;
        start_flight(&mut self.proj[s], pos, vel, fs);
    }

    /// A SAM leaves site `site` in slot `s` for a target at `tgt`: off its rail toward the
    /// target, lofted against a distant one.
    pub(crate) fn launch_sam_flight(&mut self, s: usize, site: usize, tgt: V3) {
        let o = self.objects[site];
        self.launch_flight_at(s, (o.x + 8, o.y), tgt);
    }

    /// [`Sim::launch_sam_flight`] from a launcher at a map position.
    pub(crate) fn launch_flight_at(&mut self, s: usize, (x, y): (i32, i32), tgt: V3) {
        let spec = self.proj[s].spec;
        let Some(prof) = profile(spec) else { return };
        let pos = [x as f64 * 32.0 + 16.0, y as f64 * 32.0 + 16.0, RAIL_HEIGHT];
        let r = sub(tgt, pos);
        let range = r[0].hypot(r[1]);
        let elev = r[2].atan2(range);
        // The lock range in world units (lock range x 32 map units).
        let loft = loft_angle(range, elev, self.reach(spec));
        let up = if prof.vertical { 1.5 } else { (elev + RAIL_ABOVE).max(RAIL_LOWEST).max(loft) };
        let h = self.proj[s].heading;
        let dir = direction(h, (up.min(1.5) * BAM) as i32);
        self.proj[s].pitch = s16((up * BAM) as i32);
        self.start_flight(s, pos, mul(dir, RAIL_SPEED));
        if let Some(f) = self.proj[s].flight.as_mut() {
            f.loft = (loft * BAM) as i32;
        }
    }

    /// The lock range of a missile of type `spec` in world units (lock range x 32 map units).
    fn reach(&self, spec: i32) -> f64 {
        self.missile(spec).lock_range as f64 * 1024.0
    }

    /// The AIM-120 `pm`, its flight just started, against a target at `tgt`: a distant one gets
    /// a loft, the missile climbing at the angle for the range before diving on the target (a
    /// pull up before firing leaves it less to turn).
    pub(crate) fn loft_launch(&self, pm: &mut Projectile, tgt: V3) {
        let Some(f) = pm.flight.as_mut().filter(|_| pm.spec == 22) else { return };
        let fine = ROUND_FINE as f64;
        let r = sub(tgt, [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine]);
        let range = r[0].hypot(r[1]);
        f.loft = (loft_angle(range, r[2].atan2(range), self.reach(pm.spec)) * BAM) as i32;
    }

    /// One frame of a realistic flight: steers for `guide` (when the seeker has its target),
    /// moves and checks the fuse against `fuse`. True when the fuse goes off; a missile that
    /// hits the ground or is spent ends here.
    pub(crate) fn fly_guided(
        &mut self,
        s: usize,
        guide: Option<Target>,
        fuse: Option<Target>,
    ) -> bool {
        let spec = self.proj[s].spec;
        let vmax = self.guided_vmax(spec);
        let radius = if s >= 8 && self.missile(spec).class != 7 {
            FUSE_GROUND
        } else if s < 8 && self.proj[s].launcher > 2 {
            // Site launchers are object indices > 2, aircraft are -flight index.
            FUSE_SAM
        } else {
            FUSE_AAM
        };
        let fs = self.fs;
        match flight_step(&mut self.proj[s], vmax, radius, fs, guide, fuse) {
            Step::Fused => true,
            Step::Flying => false,
            Step::Ground => {
                self.ground_impact(s);
                false
            }
        }
    }

    /// Top speed (wu/s) of a missile of type `spec` under realistic guidance.
    pub(crate) fn guided_vmax(&self, spec: i32) -> f64 {
        ((self.missile(spec).max_speed as i32 >> 6) * 128) as f64
    }
}

/// The loft angle (radians) against a target `range` wu away and `elev` radians above, for a
/// missile with lock range `reach` (wu): none close in, growing with the range. Not worth it
/// against a target above the loft path.
fn loft_angle(range: f64, elev: f64, reach: f64) -> f64 {
    let share = ((range / reach.max(1.0) - LOFT_FROM) / (1.0 - LOFT_FROM)).clamp(0.0, 1.0);
    let loft = if share > 0.0 { 0.25 + (LOFT_MAX - 0.25) * share } else { 0.0 };
    if elev > loft * 0.5 { 0.0 } else { loft }
}

/// Starts the realistic flight of missile `pm` at `pos` (wu: east, south, true altitude) with
/// velocity `vel`.
pub(crate) fn start_flight(pm: &mut Projectile, pos: V3, vel: V3, fs: i32) {
    let q = |a: f64| (a * ROUND_FINE as f64).round() as i32;
    pm.flight = Some(MissileFlight {
        x: q(pos[0]),
        y: q(pos[1]),
        alt: q(pos[2]),
        vx: q(vel[0]),
        vy: q(vel[1]),
        vz: q(vel[2]),
        t: 0,
        smoke: profile(pm.spec).map_or(0, |p| (p.smoke * fs as f64).min(1e6) as i32),
        loft: 0,
        seen: false,
        lost: false,
        stalled: false,
    });
    pm.ttl = MAX_SECS * fs;
    pm.roll = 0;
}

/// How a frame of a realistic flight ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Flying,
    /// The proximity fuse went off.
    Fused,
    /// It hit the ground (`alt` is -1).
    Ground,
}

/// One frame of the realistic flight of `pm` with top speed `vmax` (wu/s) and fuse `radius`:
/// steers for `guide`, moves and checks the fuse against `fuse`.
pub(crate) fn flight_step(
    pm: &mut Projectile,
    vmax: f64,
    radius: f64,
    fs: i32,
    guide: Option<Target>,
    fuse: Option<Target>,
) -> Step {
    let m = *pm;
    let (Some(mut f), Some(prof)) = (m.flight, profile(m.spec)) else { return Step::Flying };
    let fs = fs.max(1) as f64;
    let dt = 1.0 / fs;
    let fine = ROUND_FINE as f64;
    let mut p = [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine];
    let mut v = [f.vx as f64 / fine, f.vy as f64 / fine, f.vz as f64 / fine];
    let t = f.t as f64 / fs;
    let speed = len(v);
    let vh = if speed > 1.0 { mul(v, 1.0 / speed) } else { direction(m.heading, m.pitch) };
    let sigma = density(p[2]);
    // Lateral acceleration: carry the weight, then the loft or the navigation.
    let lift = (sigma * (speed / FULL_LIFT_SPEED).powi(2)).clamp(MIN_LIFT, 1.0);
    let a_max = prof.g * GRAVITY * lift;
    let up = [0.0, 0.0, 1.0];
    let mut cmd = mul(up, GRAVITY);
    // The seeker, once it has had the target in view, loses it for good when it leaves the view
    // (until then, turning over from a vertical launch, a lob or climbing a loft, it is steered
    // to it). Flying straight off the launcher it does not look yet. Stalled: nothing steers it
    // any more.
    let guide = guide.filter(|_| !f.stalled);
    let range = guide.map(|tg| (tg.p[0] - p[0]).hypot(tg.p[1] - p[1]));
    let straight = t < straight_secs(m.spec, range);
    let guide = guide.filter(|tg| {
        if straight {
            return true;
        }
        let r = sub(tg.p, p);
        let off = dot(r, vh) / len(r).max(1.0);
        f.lost |= f.seen && off < GIMBAL_COS && f.loft == 0;
        f.seen |= off > ACQUIRE_COS;
        !f.lost
    });
    if !straight && let Some(tg) = guide {
        let r = sub(tg.p, p);
        if f.loft != 0 {
            let loft = f.loft as f64 / BAM;
            let range = r[0].hypot(r[1]);
            let elev = r[2].atan2(range);
            // Push over toward the target once turning down to it would take a good part
            // of the time left (the missile turns slowly when fast or up high), or when the
            // target has climbed above the loft path. Both at the speed it is boosting to.
            let fast = speed.max(vmax);
            let a_fast = prof.g * GRAVITY * (sigma * (fast / FULL_LIFT_SPEED).powi(2)).min(1.0);
            let turn = (vh[2].clamp(-1.0, 1.0).asin() - elev).max(0.0) * fast / a_fast;
            let to_go = len(r) / fast;
            if range < 1.0 || elev > loft || turn > PUSHOVER * to_go {
                f.loft = 0;
            } else {
                let want = [r[0] / range * loft.cos(), r[1] / range * loft.cos(), loft.sin()];
                cmd = add(cmd, mul(across(want, vh), speed * LOFT_GAIN));
            }
        }
        if f.loft == 0 {
            // Pure proportional navigation: the velocity turns N times as fast as the line
            // of sight.
            let omega = mul(cross(r, sub(tg.v, v)), 1.0 / dot(r, r).max(1.0));
            cmd = add(cmd, mul(cross(omega, v), PN_GAIN));
        }
    }
    // Stalled: no lift, it falls.
    let mut a_lat = if f.stalled { [0.0; 3] } else { across(cmd, vh) };
    let a = len(a_lat);
    if a > a_max {
        a_lat = mul(a_lat, a_max / a);
    }

    // Along the flight path: thrust while the motor burns, drag. The motor holds the top speed
    // level and climbing; diving it pushes on as in level flight and gravity speeds it up.
    let induced = INDUCED * len(a_lat);
    let drag = prof.drag * DRAG * sigma * speed * speed + induced;
    let thrust = if t < prof.burn {
        let most = vmax / prof.boost + prof.drag * DRAG * sigma * vmax * vmax + GRAVITY;
        let hold = prof.drag * DRAG * sigma * speed.min(vmax).powi(2) + induced;
        ((vmax - speed).max(0.0) / 0.5 + hold + GRAVITY * vh[2].max(0.0)).clamp(0.0, most)
    } else {
        0.0
    };
    let acc = add(add(a_lat, mul(vh, thrust - drag)), [0.0, 0.0, -GRAVITY]);
    let p0 = p;
    v = add(v, mul(acc, dt));
    p = add(p, mul(v, dt));
    f.t += 1;
    f.smoke = (f.smoke - 1).max(0);

    // Proximity fuse: the closest approach during the step (the target goes on from where
    // it is now, like the missile).
    let fused = fuse.is_some_and(|tg| {
        let a = sub(p0, tg.p);
        let b = sub(p, add(tg.p, mul(tg.v, dt)));
        let d = sub(b, a);
        let k = (-dot(a, d) / dot(d, d).max(1e-9)).clamp(0.0, 1.0);
        len(add(a, mul(d, k))) < radius
    });

    // Back into the record.
    let speed = len(v);
    f.stalled |= t > prof.burn && speed < MIN_SPEED;
    let q = |a: f64| (a * fine).round() as i32;
    f.x = q(p[0]);
    f.y = q(p[1]);
    f.alt = q(p[2]);
    f.vx = q(v[0]);
    f.vy = q(v[1]);
    f.vz = q(v[2]);
    pm.flight = Some(f);
    pm.x = ((p[0] / 32.0).floor() as i32) & 0xFFFF;
    pm.y = ((p[1] / 32.0).floor() as i32) & 0xFFFF;
    pm.alt = view_z_f(p[2]) as i32;
    pm.speed = (speed / 128.0).round() as i32;
    if v[0].hypot(v[1]) > 1.0 {
        pm.heading = s16((v[0].atan2(-v[1]) * BAM) as i32);
    }
    pm.pitch = s16((v[2].atan2(v[0].hypot(v[1])) * BAM) as i32);
    pm.roll = 0;
    pm.ttl -= 1;
    if fused {
        Step::Fused
    } else if p[2] < 0.0 {
        pm.alt = -1;
        Step::Ground
    } else {
        Step::Flying
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;
    use crate::SimData;
    use crate::math::abs16;
    use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};

    fn new_sim() -> Sim {
        static D: OnceLock<(SimData, stealth_mission::Mission)> = OnceLock::new();
        let (data, m) = D.get_or_init(|| {
            let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
            let data = SimData::load_json(&assets.join("sim")).expect("assets/sim");
            let g = GameData::load_json(&assets.join("mission")).expect("assets/mission");
            let s = MissionSettings { theater: 3, kind: MissionKind::Strike, ..Default::default() };
            (data, stealth_mission::generate(&g, &s, Seed::Srand(0x1234)).unwrap())
        });
        let opts = SimOptions { realistic_guidance: true, ..Default::default() };
        Sim::with_options(data, m, opts)
    }

    /// A missile of type `spec` in slot `s` at `pos` with velocity `vel`.
    fn missile(sim: &mut Sim, s: usize, spec: i32, pos: V3, vel: V3) {
        sim.proj[s] = Projectile { spec, ttl: 1, target: -1, launcher: 5, ..Default::default() };
        sim.start_flight(s, pos, vel);
    }

    /// Flies slot `s` at a target moving from `tgt` with velocity `tv` for up to `secs`; calls
    /// `each` every frame. The time of the fuse going off, None if it never does.
    fn fly(
        sim: &mut Sim,
        s: usize,
        mut tgt: V3,
        tv: V3,
        secs: f64,
        mut each: impl FnMut(&Projectile, f64),
    ) -> Option<f64> {
        for i in 0..(secs * 15.0) as i32 {
            let t = Target { p: tgt, v: tv };
            if sim.fly_guided(s, Some(t), Some(t)) {
                return Some(i as f64 / 15.0);
            }
            tgt = add(tgt, mul(tv, 1.0 / 15.0));
            if sim.proj[s].ttl == 0 {
                return None;
            }
            each(&sim.proj[s], i as f64 / 15.0);
        }
        None
    }

    fn speed(m: &Projectile) -> f64 {
        let f = m.flight.unwrap();
        len([f.vx as f64, f.vy as f64, f.vz as f64]) / ROUND_FINE as f64
    }

    #[test]
    fn hits_a_crossing_target() {
        let mut sim = new_sim();
        // An AIM-120 fired north at a fighter 20 kft ahead crossing east at 900 wu/s.
        missile(&mut sim, 8, 22, [0.0, 0.0, 10000.0], [0.0, -900.0, 0.0]);
        let hit =
            fly(&mut sim, 8, [-3000.0, -20000.0, 10000.0], [900.0, 0.0, 0.0], 60.0, |_, _| {});
        assert!(hit.is_some_and(|t| t > 3.0 && t < 15.0), "{hit:?}");
    }

    /// Highest altitude of an AIM-120 fired level from 10000 at a target `range` wu ahead at
    /// 10000, and whether it hits.
    fn aim120_apex(range: f64) -> (f64, bool) {
        let mut sim = new_sim();
        missile(&mut sim, 8, 22, [0.0, 0.0, 10000.0], [0.0, -900.0, 0.0]);
        let tgt = [0.0, -range, 10000.0];
        let mut m = sim.proj[8];
        sim.loft_launch(&mut m, tgt);
        sim.proj[8] = m;
        let mut apex: f64 = 0.0;
        let hit = fly(&mut sim, 8, tgt, [0.0; 3], 60.0, |m, _| {
            apex = apex.max(m.flight.unwrap().alt as f64 / ROUND_FINE as f64);
        });
        (apex, hit.is_some())
    }

    #[test]
    fn aim120_lofts_against_a_distant_target() {
        let (near_apex, near) = aim120_apex(8000.0);
        let (far_apex, far) = aim120_apex(30000.0);
        assert!(near && far, "{near} {far}");
        assert!(near_apex < 10500.0, "near shot flies at the target: {near_apex}");
        assert!(far_apex > 12000.0, "far shot climbs first: {far_apex}");
    }

    #[test]
    fn a_lobbed_hellfire_flies_straight_for_1_s() {
        let mut sim = new_sim();
        // Lobbed 20 degrees up at a tank on the ground: no turn for 1 s, then it dives on it.
        let dir = direction(0, (20.0f64.to_radians() * BAM) as i32);
        missile(&mut sim, 8, 47, [0.0, 0.0, 300.0], mul(dir, 300.0));
        let hit = fly(&mut sim, 8, [0.0, -15000.0, 0.0], [0.0; 3], 60.0, |m, t| {
            if t < 0.95 {
                assert!((m.pitch - 3641).abs() < 300, "t {t}: pitch {}", m.pitch);
            }
        });
        assert!(hit.is_some(), "the lobbed Hellfire reaches the target");
    }

    #[test]
    fn a_hellfire_at_a_close_target_turns_onto_it_soon() {
        let mut sim = new_sim();
        // Fired level from 500 at a tank 4500 ahead: it pushes over after a moment and hits.
        missile(&mut sim, 8, 47, [0.0, 0.0, 500.0], [0.0, -100.0, 0.0]);
        let mut pitch_at = 0;
        let hit = fly(&mut sim, 8, [0.0, -4500.0, 0.0], [0.0; 3], 30.0, |m, t| {
            if (0.6..0.7).contains(&t) {
                pitch_at = m.pitch;
            }
        });
        assert!(pitch_at < -200, "diving by 0.6 s: {pitch_at}");
        assert!(hit.is_some(), "it reaches the target");
    }

    #[test]
    fn motor_burns_out_and_the_missile_glides() {
        let mut sim = new_sim();
        // An AIM-9 fired at a target far ahead: 5 s of smoke, top speed, then slowing.
        missile(&mut sim, 8, 23, [0.0, 0.0, 10000.0], [0.0, -800.0, 0.0]);
        let (mut at_burnout, mut later) = (0.0, 0.0);
        fly(&mut sim, 8, [0.0, -200_000.0, 10000.0], [0.0; 3], 20.0, |m, t| {
            assert_eq!(m.motor_visible(), t < 4.9, "t {t}");
            if (4.9..5.0).contains(&t) {
                at_burnout = speed(m);
            }
            if (15.0..15.1).contains(&t) {
                later = speed(m);
            }
        });
        // At top speed when the motor burns out.
        assert!(at_burnout > 0.9 * sim.guided_vmax(23), "{at_burnout} of {}", sim.guided_vmax(23));
        assert!(later < at_burnout * 0.85 && later > 1000.0, "{at_burnout} -> {later}");
    }

    #[test]
    fn diving_missiles_gain_speed() {
        // AIM-120s fired level and 45 degrees down at targets on their flight path: the diving
        // one goes faster than the top speed, with the motor burning and once it has burnt out.
        let speeds = |pitch: f64| {
            let mut sim = new_sim();
            let dir = direction(0, (pitch.to_radians() * BAM) as i32);
            missile(&mut sim, 8, 22, [0.0, 0.0, 40000.0], mul(dir, 900.0));
            let tgt = add([0.0, 0.0, 40000.0], mul(dir, 1e6));
            let (mut burning, mut gliding) = (0.0, 0.0);
            fly(&mut sim, 8, tgt, [0.0; 3], 10.0, |m, t| {
                if (6.0..6.1).contains(&t) {
                    burning = speed(m);
                }
                if (9.0..9.1).contains(&t) {
                    gliding = speed(m);
                }
            });
            (burning, gliding)
        };
        let vmax = new_sim().guided_vmax(22);
        let (level_burning, level_gliding) = speeds(0.0);
        let (dive_burning, dive_gliding) = speeds(-45.0);
        assert!((level_burning - vmax).abs() < 50.0, "{level_burning} {vmax}");
        assert!(dive_burning > vmax + 30.0, "{dive_burning} {vmax}");
        assert!(dive_gliding > level_gliding + 10.0, "{dive_gliding} {level_gliding}");
    }

    /// Highest altitude of an SA-2 launched at a target `range` wu away at 6000.
    fn sa2_apex(range: f64) -> (f64, Option<f64>) {
        let mut sim = new_sim();
        let o = sim.objects[3];
        sim.proj[0] = Projectile {
            spec: 1,
            ttl: 1,
            target: -1,
            launcher: 3,
            heading: 0x8000,
            ..Default::default()
        };
        let site = [(o.x + 8) as f64 * 32.0 + 16.0, o.y as f64 * 32.0 + 16.0];
        let tgt = [site[0], site[1] + range, 6000.0];
        sim.launch_sam_flight(0, 3, tgt);
        let mut apex: f64 = 0.0;
        let hit = fly(&mut sim, 0, tgt, [0.0; 3], 140.0, |m, _| {
            apex = apex.max(m.flight.unwrap().alt as f64 / ROUND_FINE as f64);
        });
        (apex, hit)
    }

    #[test]
    fn distant_targets_get_a_lofted_sam() {
        let (near_apex, near) = sa2_apex(20_000.0);
        let (far_apex, far) = sa2_apex(110_000.0);
        assert!(near.is_some() && far.is_some(), "{near:?} {far:?}");
        assert!(near_apex < 9000.0, "near shot flies at the target: {near_apex}");
        assert!(far_apex > 25_000.0, "far shot climbs first: {far_apex}");
    }

    #[test]
    fn slow_missiles_high_up_barely_turn() {
        let mut sim = new_sim();
        // Gliding slowly at 40 kft (motor out), the target well off the nose: a slow turn.
        missile(&mut sim, 8, 23, [0.0, 0.0, 40000.0], [0.0, -900.0, 0.0]);
        sim.proj[8].flight.as_mut().unwrap().t = 15 * 30;
        let mut h = 0;
        fly(&mut sim, 8, [20000.0, -20000.0, 40000.0], [0.0; 3], 2.0, |m, _| h = m.heading);
        let fast = {
            let mut sim = new_sim();
            missile(&mut sim, 8, 23, [0.0, 0.0, 10000.0], [0.0, -3500.0, 0.0]);
            sim.proj[8].flight.as_mut().unwrap().t = 15 * 30;
            let mut h = 0;
            fly(&mut sim, 8, [20000.0, -20000.0, 10000.0], [0.0; 3], 2.0, |m, _| h = m.heading);
            h
        };
        assert!(s16(h) < s16(fast) / 3, "slow {h} fast {fast}");
    }

    #[test]
    fn an_overshot_target_is_lost() {
        let mut sim = new_sim();
        // An AIM-9 passing 3000 wu over a target just ahead, too steep to turn down to: it flies
        // on instead of looping back.
        missile(&mut sim, 8, 23, [0.0, 0.0, 12000.0], [0.0, -3500.0, 0.0]);
        sim.proj[8].flight.as_mut().unwrap().t = 15 * 30;
        let mut h = 0;
        let hit = fly(&mut sim, 8, [0.0, -1000.0, 9000.0], [0.0; 3], 10.0, |m, _| {
            h = h.max(abs16(s16(m.heading)));
        });
        assert!(hit.is_none() && h < 0x400, "turned back: {h:#x}");
    }

    #[test]
    fn a_vertical_launch_turns_over_to_a_low_target() {
        let mut sim = new_sim();
        // An SA-10 off its vertical launcher at a target low and 15 km out.
        missile(&mut sim, 0, 4, [0.0, 0.0, 5.0], [0.0, 0.0, 150.0]);
        let hit = fly(&mut sim, 0, [0.0, -50_000.0, 500.0], [0.0; 3], 60.0, |_, _| {});
        assert!(hit.is_some(), "missed");
    }
}
