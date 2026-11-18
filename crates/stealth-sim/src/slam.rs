//! The SLAM-ER (the F/A-18E's stand-off land attack missile), simplified for the game.
//!
//! The pilot programs four data link channels on the SLAM page: the cruise altitude (FLT
//! LO/MED/HI), the range to the target at which the seeker comes on and the target (the
//! mission's primary or secondary target). Fired, a missile takes the selected channel (or the
//! first free one) and cruises at its altitude to its target at a steady speed, always level.
//! Once its target is within the seeker range it turns toward it and its seeker picture (FLIR)
//! goes to the SLAM page. From then on the pilot steers it with the slew controls (pitch and
//! yaw only). Let go, it locks onto the ground target nearest the middle of its picture (an
//! object or a ground unit, within [`SNAP_DEG`]), else it holds the ground point under the
//! crosshair, and flies into it.

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::types::*;
use stealth_i18n::text;

/// Data link channels: missiles in flight at once (the pilot's four weapon slots).
pub const CHANNELS: usize = 4;
/// FLT: cruise altitudes (altitude units, "ft"): LO over the ground under it, MED and HI over
/// sea level (and at least LO over the ground).
pub const FLT_ALT: [i32; 3] = [500, 8000, 20000];
pub const FLT_LABELS: [&str; 3] = ["LO", "MED", "HI"];
/// The seeker ranges the page offers (nautical miles) and the one it starts with.
pub const SEEKER_NM: [u8; 5] = [2, 3, 5, 8, 10];
pub const SEEKER_DEFAULT_NM: u8 = 5;
/// Speed (wu/s, about 340 m/s).
pub const SPEED: f64 = 700.0;
/// Seconds it flies at most.
const MAX_SECS: i32 = 1500;
/// Cruise: turn rate (degrees/s) and the steepest climb or dive (degrees).
const CRUISE_TURN_DEG: f64 = 8.0;
const CRUISE_GAMMA_DEG: f64 = 30.0;
/// Terminal: the fastest the slew turns it (degrees/s at full deflection) and the fastest it
/// turns by itself onto its aim point.
const SLEW_DEG: f64 = 10.0;
const HOME_DEG: f64 = 15.0;
/// The cone around the crosshair it snaps onto a ground target in (degrees) and how far it
/// looks for one (wu).
pub const SNAP_DEG: f64 = 3.0;
const SNAP_RANGE: f64 = 60_000.0;
/// How far down the boresight the ground point under the crosshair is searched (wu) and the
/// step.
const MARCH_RANGE: f64 = 80_000.0;
const MARCH_STEP: f64 = 32.0;
/// Blast radius against world objects (map units).
const BLAST_MAP: i32 = 6;

const NM_WU: f64 = 1852.0 / WU_METRES;

/// One channel's program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlamProgram {
    /// 0 LO, 1 MED, 2 HI.
    pub flt: u8,
    /// Range to the target at which the seeker comes on (nautical miles).
    pub seeker_nm: u8,
    /// 0 the mission's primary target, 1 its secondary.
    pub target: u8,
}

impl Default for SlamProgram {
    fn default() -> Self {
        SlamProgram { flt: 0, seeker_nm: SEEKER_DEFAULT_NM, target: 0 }
    }
}

/// The SLAM page's report to the simulation each frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlamInput {
    /// The channel selected (0..4).
    pub sel: u8,
    pub prog: [SlamProgram; CHANNELS],
    /// The slew (right, up; -127..127) steering the selected missile once its seeker is on.
    pub steer: (i8, i8),
}

/// A SLAM-ER in flight: its channel and program, and its terminal guidance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlamGuide {
    pub ch: u8,
    /// The target point (wu: east, south).
    pub tx: i32,
    pub ty: i32,
    pub flt: u8,
    pub seeker_nm: u8,
    /// The seeker is on: the pilot steers it.
    pub active: bool,
    /// The point it flies into (wu: east, south, up): the target waypoint as the seeker comes
    /// on, then what it snapped onto or the ground under the crosshair; None while slewed.
    pub aim: Option<(i32, i32, i32)>,
    /// `aim` is a target it snapped onto (drawn boxed on the page).
    pub snapped: bool,
}

impl SlamGuide {
    /// The seeker range (wu).
    pub fn seeker_wu(&self) -> f64 {
        self.seeker_nm as f64 * NM_WU
    }
}

/// Seconds until missile `m`'s seeker comes on (0 once it is on).
pub fn time_to_seeker(m: &Projectile) -> Option<f64> {
    let g = m.slam?;
    if g.active {
        return Some(0.0);
    }
    let f = m.flight?;
    let fine = ROUND_FINE as f64;
    let d = (g.tx as f64 - f.x as f64 / fine).hypot(g.ty as f64 - f.y as f64 / fine);
    Some(((d - g.seeker_wu()) / SPEED).max(0.0))
}

type V3 = [f64; 3];

fn len(a: V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn norm(a: V3) -> V3 {
    let l = len(a).max(1e-9);
    [a[0] / l, a[1] / l, a[2] / l]
}

/// Heading (radians clockwise from north) and climb angle of a direction (east, south, up).
fn angles(d: V3) -> (f64, f64) {
    (d[0].atan2(-d[1]), d[2].atan2(d[0].hypot(d[1])))
}

fn from_angles(h: f64, g: f64) -> V3 {
    [g.cos() * h.sin(), -g.cos() * h.cos(), g.sin()]
}

/// The shortest way from angle `a` to `b` (radians).
fn wrap(d: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    (d + std::f64::consts::PI).rem_euclid(tau) - std::f64::consts::PI
}

/// Turns direction `d` toward `want`, by at most `rate` radians in heading and in climb.
fn turn_toward(d: V3, want: V3, rate: f64) -> V3 {
    let (h, g) = angles(d);
    let (wh, wg) = angles(want);
    let h = h + wrap(wh - h).clamp(-rate, rate);
    let g = g + (wg - g).clamp(-rate, rate);
    from_angles(h, g)
}

impl Sim {
    /// The ground (altitude units) at a point in wu (east, south): the mountains and the hills.
    pub(crate) fn slam_ground(&self, x: f64, y: f64) -> f64 {
        let (mx, my) = ((x / 32.0).floor() as i32, (y / 32.0).floor() as i32);
        let hills = self.hill_alt(mx, my);
        let relief = self.tiles.as_ref().map_or(0, |t| t.relief.ground(mx, my));
        hills.max(relief) as f64
    }

    /// Channels with a missile of the active pilot in flight.
    fn slam_busy(&self) -> [bool; CHANNELS] {
        let mut busy = [false; CHANNELS];
        for s in Self::own_slots() {
            if let Some(g) = self.proj[s].slam.filter(|_| self.proj[s].ttl != 0) {
                busy[(g.ch as usize).min(CHANNELS - 1)] = true;
            }
        }
        busy
    }

    /// The target point of target slot `k` (wu: east, south), if the mission has one there.
    pub(crate) fn slam_target(&self, k: usize) -> Option<(i32, i32)> {
        let t = self.pl.targets.get(k)?;
        let o = self.objects.get(usize::try_from(t.object).ok()?)?;
        Some((o.x * 32 + 16, o.y * 32 + 16))
    }

    /// Fires a SLAM-ER on the selected channel (else the first free one) at its programmed
    /// target: not with four in flight, nor without a target.
    pub(crate) fn release_slam(&mut self, ps: &PlayerState) {
        let input = ps.slam.unwrap_or_default();
        let busy = self.slam_busy();
        let sel = (input.sel as usize).min(CHANNELS - 1);
        let Some(ch) = (!busy[sel]).then_some(sel).or_else(|| (0..CHANNELS).find(|&c| !busy[c]))
        else {
            self.sound(0x20, 1);
            self.msg(text!("sim-slam-channels-busy"));
            return;
        };
        let prog = input.prog[ch];
        let Some((tx, ty)) = self
            .slam_target(prog.target as usize)
            .or_else(|| self.slam_target(1 - prog.target.min(1) as usize))
        else {
            self.sound(0x20, 1);
            self.msg(text!("sim-no-target-designated"));
            return;
        };
        self.pl.last_missile_slot = -1;
        self.fire_missile();
        let Ok(slot) = usize::try_from(self.pl.last_missile_slot) else { return };
        let bay = self.proj[slot].station.map_or(0, |b| b as usize);
        let (pos, vel) = self.store_motion(bay, self.proj[slot].drop > 0);
        self.start_flight(slot, pos, vel);
        let fs = self.fs;
        let pm = &mut self.proj[slot];
        pm.ttl = MAX_SECS * fs;
        pm.target = -1;
        pm.slam = Some(SlamGuide {
            ch: ch as u8,
            tx,
            ty,
            flt: prog.flt.min(2),
            seeker_nm: prog.seeker_nm.clamp(1, 50),
            ..Default::default()
        });
    }

    /// One frame of the SLAM-ER in slot `s`.
    pub(crate) fn slam_frame(&mut self, s: usize) {
        let m = self.proj[s];
        let (Some(mut g), Some(mut f)) = (m.slam, m.flight) else {
            self.proj[s].ttl = 0;
            return;
        };
        let fs = self.fs.max(1) as f64;
        let dt = 1.0 / fs;
        let fine = ROUND_FINE as f64;
        let mut p = [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine];
        let v = [f.vx as f64 / fine, f.vy as f64 / fine, f.vz as f64 / fine];
        let mut d =
            if len(v) > 1.0 { norm(v) } else { crate::guidance::direction(m.heading, m.pitch) };
        let mut speed = len(v);
        if m.drop > 0 {
            // Falling clear of the pylon before its engine starts.
            f.sink += (crate::ballistics::GRAVITY * dt * fine) as i32;
            p[2] -= f.sink as f64 / fine * dt;
            self.proj[s].drop -= 1;
            if self.proj[s].drop == 0 {
                f.sink = 0;
            }
        } else {
            speed += (SPEED - speed).clamp(-80.0 * dt, 120.0 * dt);
            let to = [g.tx as f64 - p[0], g.ty as f64 - p[1]];
            if !g.active && to[0].hypot(to[1]) <= g.seeker_wu() {
                g.active = true;
                let z = self.slam_ground(g.tx as f64, g.ty as f64);
                g.aim = Some((g.tx, g.ty, z as i32));
                self.emit(SimEvent::CancelTimeAccel);
            }
            if g.active {
                d = self.slam_terminal(&mut g, p, d, dt);
            } else {
                // Cruise: to the target at the programmed altitude, the ground ahead cleared.
                let ahead = (0..4)
                    .map(|k| {
                        let r = k as f64 * speed * 2.0;
                        self.slam_ground(p[0] + d[0] * r, p[1] + d[1] * r)
                    })
                    .fold(0.0, f64::max);
                let floor = ahead + FLT_ALT[0] as f64;
                let want_alt = match g.flt {
                    0 => floor,
                    k => (FLT_ALT[k.min(2) as usize] as f64).max(floor),
                };
                let gmax = CRUISE_GAMMA_DEG.to_radians();
                let gamma = ((want_alt - p[2]) / (4.0 * speed.max(1.0))).clamp(-gmax, gmax);
                let h = to[0].atan2(-to[1]);
                d = turn_toward(d, from_angles(h, gamma), CRUISE_TURN_DEG.to_radians() * dt);
            }
            p = [p[0] + d[0] * speed * dt, p[1] + d[1] * speed * dt, p[2] + d[2] * speed * dt];
        }
        let vel = [d[0] * speed, d[1] * speed, d[2] * speed];
        let q = |a: f64| (a * fine).round() as i32;
        (f.x, f.y, f.alt, f.vx, f.vy, f.vz) =
            (q(p[0]), q(p[1]), q(p[2]), q(vel[0]), q(vel[1]), q(vel[2]));
        f.t += 1;
        let pm = &mut self.proj[s];
        pm.flight = Some(f);
        pm.slam = Some(g);
        pm.x = ((p[0] / 32.0).floor() as i32) & 0xFFFF;
        pm.y = ((p[1] / 32.0).floor() as i32) & 0xFFFF;
        pm.alt = crate::math::view_z_f(p[2]) as i32;
        pm.speed = (speed / 128.0).round() as i32;
        let (heading, pitch) = crate::guidance::heading_pitch(d);
        pm.heading = heading;
        pm.pitch = pitch;
        // Always level.
        pm.roll = 0;
        pm.ttl -= 1;
        if p[2] <= self.slam_ground(p[0], p[1]) {
            self.slam_impact(s, p);
        }
    }

    /// The terminal phase: the pilot's slew turns the missile; let go, it homes on its aim
    /// point (a target it snapped onto, else the ground under the crosshair).
    fn slam_terminal(&mut self, g: &mut SlamGuide, p: V3, d: V3, dt: f64) -> V3 {
        let input = self.pl.last_input.slam.filter(|i| i.sel == g.ch);
        let steer = input.map_or((0, 0), |i| i.steer);
        if steer != (0, 0) {
            g.aim = None;
            g.snapped = false;
            let rate = SLEW_DEG.to_radians() * dt / 127.0;
            let (h, gm) = angles(d);
            let gm = (gm + steer.1 as f64 * rate).clamp(-1.4, 0.6);
            return from_angles(h + steer.0 as f64 * rate, gm);
        }
        if g.aim.is_none() || !g.snapped {
            // Let go: the target nearest the crosshair, else the ground there.
            if let Some(t) = self.slam_snap(p, d) {
                g.aim = Some(t);
                g.snapped = true;
            } else if g.aim.is_none() {
                g.aim = self.slam_boresight_ground(p, d);
            }
        }
        match g.aim {
            Some((x, y, z)) => {
                let want = norm([x as f64 - p[0], y as f64 - p[1], z as f64 - p[2]]);
                turn_toward(d, want, HOME_DEG.to_radians() * dt)
            }
            None => d,
        }
    }

    /// The ground target nearest the crosshair of a seeker at `p` looking along `d`, within
    /// [`SNAP_DEG`] (wu: east, south, up).
    fn slam_snap(&self, p: V3, d: V3) -> Option<(i32, i32, i32)> {
        let cos = SNAP_DEG.to_radians().cos();
        let mut best: Option<(f64, (i32, i32, i32))> = None;
        let mut consider = |x: f64, y: f64, z: f64| {
            let r = [x - p[0], y - p[1], z - p[2]];
            let l = len(r);
            if l > SNAP_RANGE || l < 1.0 {
                return;
            }
            let c = (r[0] * d[0] + r[1] * d[1] + r[2] * d[2]) / l;
            if c > cos && best.is_none_or(|(b, _)| c > b) {
                best = Some((c, (x as i32, y as i32, z as i32)));
            }
        };
        for o in self.objects.iter().skip(3) {
            if o.flags & oflags::DESTROYED != 0 || o.shape == 0 {
                continue;
            }
            let (x, y) = (o.x as f64 * 32.0 + 16.0, o.y as f64 * 32.0 + 16.0);
            consider(x, y, self.slam_ground(x, y));
        }
        for u in self.units.iter().filter(|u| !u.destroyed) {
            let (ux, uy) = u.map_xy();
            let (x, y) = (ux * 32.0, uy * 32.0);
            consider(x, y, self.slam_ground(x, y));
        }
        best.map(|(_, t)| t)
    }

    /// Where the boresight of a seeker at `p` looking along `d` meets the ground.
    fn slam_boresight_ground(&self, p: V3, d: V3) -> Option<(i32, i32, i32)> {
        if d[2] >= -0.01 {
            return None;
        }
        let steps = (MARCH_RANGE / MARCH_STEP) as usize;
        (1..=steps).find_map(|k| {
            let r = k as f64 * MARCH_STEP;
            let q = [p[0] + d[0] * r, p[1] + d[1] * r, p[2] + d[2] * r];
            let ground = self.slam_ground(q[0], q[1]);
            (q[2] <= ground).then_some((q[0] as i32, q[1] as i32, ground as i32))
        })
    }

    /// The SLAM-ER in slot `s` hits the ground at `p` (wu): the world object there may be
    /// destroyed (as the weapon's effect against it allows), the blast kills ground units near.
    pub(crate) fn slam_impact(&mut self, s: usize, p: V3) {
        let m = self.proj[s];
        self.proj[s].ttl = 0;
        let (ex, ey) = (((p[0] / 32.0) as i32) & 0xFFFF, ((p[1] / 32.0) as i32) & 0xFFFF);
        let long = self.weapon_types[(m.weapon.max(0) as usize).min(self.weapon_types.len() - 1)]
            .long_name
            .clone();
        let mut text = text!("sim-misses", weapon = long.to_string());
        let o = self.find_object_at(ex, ey);
        if o > 0 && self.obj(o).flags & oflags::DESTROYED == 0 {
            let ob = self.obj(o);
            let (tx, ty) = self.pl.last_tile.map_or((ob.x, ob.y), |t| t.map_pos());
            let close = crate::math::approx(tx - ex, ty - ey) <= BLAST_MAP;
            if close && self.compat(m.weapon, o) > self.rng.random(4) {
                self.destroy_ground(o);
                let place = self.place_name(o as usize);
                text = text!("sim-destroyed-by", target = place, weapon = long.to_string());
            }
        }
        self.msg(text);
        self.flash = Flash { x: ex, y: ey, alt: 0, timer: 0x10 };
        self.emit(SimEvent::Explosion { x: ex, y: ey, alt: 0, kind: 0x10 });
    }

    /// The active pilot's SLAM-ERs in flight: slot and missile.
    pub fn slam_rounds(&self) -> Vec<(usize, Projectile)> {
        Self::own_slots()
            .filter(|&s| self.proj[s].ttl != 0 && self.proj[s].slam.is_some())
            .map(|s| (s, self.proj[s]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turning_is_rate_limited_and_level() {
        let d = from_angles(0.0, 0.0);
        let want = from_angles(1.0, -0.5);
        let t = turn_toward(d, want, 0.1);
        let (h, g) = angles(t);
        assert!((h - 0.1).abs() < 1e-9 && (g + 0.1).abs() < 1e-9);
        // Across north the short way.
        let (h, _) = angles(turn_toward(from_angles(-3.0, 0.0), from_angles(3.0, 0.0), 0.1));
        assert!(wrap(h - (-3.1)).abs() < 1e-9, "{h}");
    }

    #[test]
    fn time_to_seeker_counts_down_to_the_seeker_range() {
        let g = SlamGuide { tx: 0, ty: 0, seeker_nm: 5, ..Default::default() };
        let x = g.seeker_wu() + SPEED * 30.0;
        let f = MissileFlight { x: (x * ROUND_FINE as f64) as i32, ..Default::default() };
        let m = Projectile { slam: Some(g), flight: Some(f), ttl: 1, ..Default::default() };
        let t = time_to_seeker(&m).unwrap();
        assert!((t - 30.0).abs() < 0.01, "{t}");
    }
}
