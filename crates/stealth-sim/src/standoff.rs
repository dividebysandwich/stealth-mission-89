//! Stand-off weapons the air defences can shoot down, simplified for the game.
//!
//! - The AGM-154C JSOW: an unpowered glide bomb. Released, it glides to the point it was
//!   aimed at (the targeting pod's ground point or the ground lock, else the mission's primary
//!   target) at its best glide, and noses over into a steep dive once the target lies far
//!   enough below it. Its reach comes from the release height and speed ([`glide_range`]).
//! - The ADM-160 MALD: a small jet decoy. It flies at the release altitude to the point it was
//!   aimed at, then on along its heading until its fuel runs out, showing the air defences an
//!   aircraft's signature.
//!
//! The pilot's site radars see these weapons as they sweep (the MALD as far as their missiles
//! reach, the small JSOW much closer) and fire at them ([`Sim::standoff_sweep`]); a missile fired
//! at one flies its own pursuit ([`Sim::intercept_frame`]) and may knock it down.

use serde::{Deserialize, Serialize};

use crate::math::{approx, s16};
use crate::types::*;
use crate::{ENEMY_SLOTS, Sim};
use stealth_i18n::text;

/// Gravity (altitude units ≈ ft per s²: the flight models' units, see `ballistics`).
const G: f64 = crate::ballistics::GRAVITY;
/// JSOW: lift over drag, and the speed it glides farthest at (wu/s).
pub const LIFT_DRAG: f64 = 8.5;
pub const BEST_GLIDE: f64 = 550.0;
/// JSOW: how hard it zooms its speed above the best glide speed into height, and pitches down
/// below it (radians of path angle per share of the best glide speed), and its steepest zoom
/// (degrees).
const ZOOM: f64 = 1.0;
const SINK: f64 = 0.6;
const ZOOM_MAX_DEG: f64 = 20.0;
/// JSOW: the slowest it flies (it noses down to keep it), its turn rates gliding and diving
/// (degrees/s), and the angle down to the target at which it dives (degrees).
const SLOWEST: f64 = 380.0;
const GLIDE_TURN_DEG: f64 = 6.0;
const DIVE_TURN_DEG: f64 = 20.0;
const DIVE_DEG: f64 = 30.0;
/// JSOW: the closest it is released at (wu) and the seconds it flies at most.
pub const MIN_RANGE: f64 = 6_000.0;
const JSOW_SECS: i32 = 900;
/// MALD: cruise speed (wu/s), endurance (s), turn rate (degrees/s), steepest climb or dive
/// (degrees), the least height over the ground it cruises at, and how near its aim point
/// it turns no more (wu).
const MALD_SPEED: f64 = 880.0;
const MALD_SECS: i32 = 600;
const MALD_TURN_DEG: f64 = 4.0;
const MALD_GAMMA_DEG: f64 = 20.0;
const MALD_FLOOR: f64 = 1_500.0;
const MALD_PASS: f64 = 4_000.0;
/// MALD fired without an aim point: it flies this far ahead of the aircraft (wu).
const MALD_AHEAD: f64 = 200_000.0;
/// How far (share of their missiles' lock range, in 1/8) the sites see a MALD and a JSOW.
const MALD_SEEN_8THS: i32 = 8;
const JSOW_SEEN_8THS: i32 = 3;
/// Chance (percent, per sweep that sees it) that a ready site fires at a MALD, at a JSOW.
const MALD_FIRE_PCT: i32 = 80;
const JSOW_FIRE_PCT: i32 = 35;
/// A site that sees a MALD tracks it this hard (alert): it comes up emitting.
const MALD_ALERT: i32 = 0xD0;
/// Interceptor: speed (wu/s), turn rate (degrees/s), seconds straight up off the rail, the
/// distance it fuzes at (wu) and its kill chances (percent) against a JSOW and a MALD.
const SAM_SPEED: f64 = 2_600.0;
const SAM_TURN_DEG: f64 = 25.0;
const SAM_BOOST_SECS: f64 = 1.0;
const SAM_FUZE: f64 = 120.0;
const SAM_KILL_JSOW_PCT: i32 = 65;
const SAM_KILL_MALD_PCT: i32 = 80;
const SAM_SECS: i32 = 60;

/// A JSOW or a MALD in flight.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glide {
    /// The MALD (else a JSOW).
    pub mald: bool,
    /// The point it flies to (wu: east, south).
    pub tx: i32,
    pub ty: i32,
    /// MALD: the altitude it cruises at.
    pub alt: i32,
    /// JSOW: diving onto its target; MALD: past its aim point, flying on.
    pub late: bool,
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

fn wrap(d: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    (d + std::f64::consts::PI).rem_euclid(tau) - std::f64::consts::PI
}

/// Turns direction `d` toward `want`, by at most `rate` radians in heading and in climb.
fn turn_toward(d: V3, want: V3, rate: f64) -> V3 {
    let (h, g) = angles(d);
    let (wh, wg) = angles(want);
    from_angles(h + wrap(wh - h).clamp(-rate, rate), g + (wg - g).clamp(-rate, rate))
}

/// The JSOW's glide path angle (radians, negative down) at `speed`: its best glide, zooming
/// up while it is fast (trading the release speed for height), steeper when it is slow.
fn glide_gamma(speed: f64) -> f64 {
    let d = (speed - BEST_GLIDE) / BEST_GLIDE;
    let gamma = -(1.0 / LIFT_DRAG).atan() + if d > 0.0 { ZOOM * d } else { SINK * d };
    gamma.min(ZOOM_MAX_DEG.to_radians())
}

/// The JSOW's speed change (per s) flying at climb angle `gamma`: gravity along the path
/// against drag, which balances it at the best glide speed on the best glide path.
fn glide_accel(speed: f64, gamma: f64) -> f64 {
    let r = speed / BEST_GLIDE;
    G * (-gamma.sin() - r * r / LIFT_DRAG)
}

/// How far (wu over the ground) a JSOW released `height` above its target at `speed` (wu/s)
/// glides, flying the same path as in flight over flat ground.
pub fn glide_range(height: f64, speed: f64) -> f64 {
    let dt = 0.5;
    let (mut h, mut v, mut x) = (height, speed, 0.0);
    for _ in 0..(JSOW_SECS as f64 / dt) as usize {
        if h <= 0.0 {
            break;
        }
        let gamma = glide_gamma(v);
        v = (v + glide_accel(v, gamma) * dt).max(SLOWEST * 0.5);
        x += v * gamma.cos() * dt;
        h += v * gamma.sin() * dt;
    }
    x
}

impl Sim {
    /// The point a JSOW or a MALD fired now flies to (wu: east, south): the pod's ground point
    /// or the ground lock, else the mission's primary target.
    fn standoff_aim(&self, ps: &PlayerState) -> Option<(i32, i32)> {
        if let Some((x, y)) = self.ccrp_aim_of(ps) {
            return Some(((x * 32.0) as i32, (y * 32.0) as i32));
        }
        self.slam_target(0)
    }

    /// The JSOW's reach now against the point it would fly to: range there and the most it
    /// glides (wu), or None without an aim point.
    pub fn jsow_reach(&self) -> Option<(f64, f64)> {
        let (tx, ty) = self.standoff_aim(&self.pl.last_input)?;
        let (pos, vel) = self.launch_motion();
        let range = (tx as f64 - pos[0]).hypot(ty as f64 - pos[1]);
        let height = pos[2] - self.slam_ground(tx as f64, ty as f64);
        Some((range, glide_range(height, len(vel))))
    }

    /// Releases a JSOW or a MALD (the selected weapon) at its aim point. A JSOW out of its reach
    /// or too close stays on its pylon; a MALD without an aim point flies on ahead.
    pub(crate) fn release_standoff(&mut self, ps: &PlayerState) {
        let mald = self.selected_spec() == MALD_SPEC;
        let (pos, vel) = self.launch_motion();
        let aim = self.standoff_aim(ps);
        let (tx, ty) = if mald {
            aim.unwrap_or_else(|| {
                let d = norm([vel[0], vel[1], 0.0]);
                ((pos[0] + d[0] * MALD_AHEAD) as i32, (pos[1] + d[1] * MALD_AHEAD) as i32)
            })
        } else {
            let Some((tx, ty)) = aim else {
                self.sound(0x20, 1);
                self.msg(text!("sim-no-target-designated"));
                return;
            };
            let range = (tx as f64 - pos[0]).hypot(ty as f64 - pos[1]);
            let height = pos[2] - self.slam_ground(tx as f64, ty as f64);
            if range > glide_range(height, len(vel)) {
                self.sound(0x20, 1);
                self.msg(text!("sim-jsow-out-of-range"));
                return;
            }
            if range < MIN_RANGE {
                self.sound(0x20, 1);
                self.msg(text!("sim-jsow-too-close"));
                return;
            }
            (tx, ty)
        };
        self.pl.last_missile_slot = -1;
        self.fire_missile();
        let Ok(slot) = usize::try_from(self.pl.last_missile_slot) else { return };
        let bay = self.proj[slot].station.map_or(0, |b| b as usize);
        let (pos, vel) = self.store_motion(bay, self.proj[slot].drop > 0);
        self.start_flight(slot, pos, vel);
        let fs = self.fs;
        let pm = &mut self.proj[slot];
        pm.ttl = if mald { MALD_SECS } else { JSOW_SECS } * fs;
        pm.target = -1;
        pm.glide = Some(Glide { mald, tx, ty, alt: pos[2] as i32, late: false });
    }

    /// One frame of the JSOW or MALD in slot `s`.
    pub(crate) fn glide_frame(&mut self, s: usize) {
        let m = self.proj[s];
        let (Some(mut g), Some(mut f)) = (m.glide, m.flight) else {
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
        let to = [g.tx as f64 - p[0], g.ty as f64 - p[1]];
        let dist = to[0].hypot(to[1]);
        let ground = self.slam_ground(g.tx as f64, g.ty as f64);
        if m.drop > 0 {
            // Falling clear of the pylon (the MALD's engine, the JSOW's wings not yet out).
            f.sink += (G * dt * fine) as i32;
            p[2] -= f.sink as f64 / fine * dt;
            self.proj[s].drop -= 1;
            if self.proj[s].drop == 0 {
                f.sink = 0;
            }
        } else if g.mald {
            speed += (MALD_SPEED - speed).clamp(-60.0 * dt, 80.0 * dt);
            if dist < MALD_PASS {
                g.late = true;
            }
            let ahead = (0..4)
                .map(|k| {
                    let r = k as f64 * speed * 2.0;
                    self.slam_ground(p[0] + d[0] * r, p[1] + d[1] * r)
                })
                .fold(0.0, f64::max);
            let want_alt = (g.alt as f64).max(ahead + MALD_FLOOR);
            let gmax = MALD_GAMMA_DEG.to_radians();
            let gamma = ((want_alt - p[2]) / (4.0 * speed.max(1.0))).clamp(-gmax, gmax);
            let (h, _) = angles(d);
            let h = if g.late { h } else { to[0].atan2(-to[1]) };
            d = turn_toward(d, from_angles(h, gamma), MALD_TURN_DEG.to_radians() * dt);
        } else {
            if !g.late && (p[2] - ground).atan2(dist.max(1.0)) > DIVE_DEG.to_radians() {
                g.late = true;
            }
            if g.late {
                let want = norm([to[0], to[1], ground - p[2]]);
                d = turn_toward(d, want, DIVE_TURN_DEG.to_radians() * dt);
            } else {
                let want = from_angles(to[0].atan2(-to[1]), glide_gamma(speed));
                d = turn_toward(d, want, GLIDE_TURN_DEG.to_radians() * dt);
            }
            let (_, gamma) = angles(d);
            speed = (speed + glide_accel(speed, gamma) * dt).max(SLOWEST * 0.5);
        }
        if m.drop == 0 {
            p = [p[0] + d[0] * speed * dt, p[1] + d[1] * speed * dt, p[2] + d[2] * speed * dt];
        }
        let vel = [d[0] * speed, d[1] * speed, d[2] * speed];
        let q = |a: f64| (a * fine).round() as i32;
        (f.x, f.y, f.alt, f.vx, f.vy, f.vz) =
            (q(p[0]), q(p[1]), q(p[2]), q(vel[0]), q(vel[1]), q(vel[2]));
        f.t += 1;
        let pm = &mut self.proj[s];
        pm.flight = Some(f);
        pm.glide = Some(g);
        set_pose(pm, p, d, speed);
        pm.ttl -= 1;
        if p[2] <= self.slam_ground(p[0], p[1]) {
            if g.mald {
                // Flown into the ground: it was only a decoy.
                self.proj[s].ttl = 0;
            } else {
                self.slam_impact(s, p);
            }
        }
    }

    /// Site `i` looks for its pilot's stand-off weapons as it sweeps and may fire at one:
    /// a MALD it sees as far as its missiles reach (and it tracks it, emitting), a JSOW only
    /// close in. One missile at a time at each weapon.
    pub(crate) fn standoff_sweep(&mut self, i: usize) {
        let o = self.objects[i];
        if o.flags & oflags::NEUTRAL != 0 {
            return;
        }
        let spec = self.site_missile(o.unit_type);
        if spec <= 0 {
            return;
        }
        let lock = self.missile(spec).lock_range as i32;
        let id = self.pl.id;
        for s in Self::own_slots() {
            let m = self.proj[s];
            let Some(g) = m.glide.filter(|_| m.ttl != 0 && m.drop == 0) else { continue };
            let chased = (0..ENEMY_SLOTS).any(|k| {
                self.proj[k].ttl != 0
                    && self.proj[k].intercept == Some(s as u8)
                    && self.proj_pilot[k] == id
            });
            if chased {
                continue;
            }
            let range = approx(s16(m.x - o.x), s16(m.y - o.y)) >> 6;
            let seen = lock * if g.mald { MALD_SEEN_8THS } else { JSOW_SEEN_8THS } / 8;
            if range > seen {
                continue;
            }
            if g.mald {
                let a = &mut self.objects[i];
                a.alert = a.alert.max(MALD_ALERT);
                a.flags |= oflags::TRACKING;
            }
            let pct = if g.mald { MALD_FIRE_PCT } else { JSOW_FIRE_PCT };
            if self.rng.random(100) >= pct {
                continue;
            }
            let k = self.enemy_slot(i);
            if self.proj[k].ttl == 0
                && self.may_launch(spec, (o.x, o.y, 0))
                && self.site_ready(i)
                && !self.salvo_held(spec, i)
            {
                self.launch_intercept(k, i, s);
                self.site_fired(i);
            }
            return;
        }
    }

    /// Site `i` fires missile slot `k` at the stand-off weapon in own slot `s`.
    fn launch_intercept(&mut self, k: usize, i: usize, s: usize) {
        let o = self.objects[i];
        let spec = self.site_missile(o.unit_type);
        let (x, y) = (o.x * 32 + 16, o.y * 32 + 16);
        let alt = self.hill_alt(o.x, o.y) as f64 + 20.0;
        let mut pm = Projectile {
            x: o.x,
            y: o.y,
            alt: crate::math::view_z_f(alt) as i32,
            speed: 1,
            pitch: 0x4000,
            ttl: SAM_SECS * self.fs,
            spec,
            target: -1,
            launcher: i as i32,
            intercept: Some(s as u8),
            ..Default::default()
        };
        crate::guidance::start_flight(
            &mut pm,
            [x as f64, y as f64, alt],
            [0.0, 0.0, 200.0],
            self.fs,
        );
        // (start_flight sets the table missile's life.)
        pm.ttl = SAM_SECS * self.fs;
        self.proj[k] = pm;
        self.proj_pilot[k] = self.pl.id;
        let n = self.place_name(i);
        let sam = self.missile(spec).name.clone();
        self.msg(text!("sim-firing-at-standoff", site = n.to_string(), missile = sam.to_string()));
        self.emit(SimEvent::MissileLaunched { slot: k, spec, launcher: Launcher::Site(i) });
    }

    /// One frame of enemy missile `k` fired at a stand-off weapon: straight up off the rail,
    /// then a lead pursuit of the weapon; close enough, it may knock it down. Its weapon gone,
    /// it flies on until it burns out.
    pub(crate) fn intercept_frame(&mut self, k: usize) {
        let m = self.proj[k];
        let Some(mut f) = m.flight else {
            self.proj[k].ttl = 0;
            return;
        };
        let fs = self.fs.max(1) as f64;
        let dt = 1.0 / fs;
        let fine = ROUND_FINE as f64;
        let mut p = [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine];
        let v = [f.vx as f64 / fine, f.vy as f64 / fine, f.vz as f64 / fine];
        let mut d = if len(v) > 1.0 { norm(v) } else { [0.0, 0.0, 1.0] };
        let speed = (len(v) + 1_500.0 * dt).min(SAM_SPEED);
        let s = m.intercept.map_or(usize::MAX, |s| s as usize);
        let target = self
            .proj
            .get(s)
            .filter(|t| t.ttl != 0 && t.glide.is_some())
            .and_then(|t| t.flight.map(|tf| (t, tf)));
        let mut killed = None;
        if let Some((t, tf)) = target {
            let tp = [tf.x as f64 / fine, tf.y as f64 / fine, tf.alt as f64 / fine];
            let tv = [tf.vx as f64 / fine, tf.vy as f64 / fine, tf.vz as f64 / fine];
            let r = [tp[0] - p[0], tp[1] - p[1], tp[2] - p[2]];
            let dist = len(r);
            if dist < SAM_FUZE + speed * dt {
                let mald = t.glide.is_some_and(|g| g.mald);
                let pct = if mald { SAM_KILL_MALD_PCT } else { SAM_KILL_JSOW_PCT };
                if self.rng.random(100) < pct {
                    killed = Some((s, tp));
                }
                // Fuzed: hit or miss, it is spent.
                self.proj[k].ttl = 1;
            } else if f.t as f64 >= SAM_BOOST_SECS * fs {
                let tgo = dist / speed.max(1.0);
                let lead = [tp[0] + tv[0] * tgo, tp[1] + tv[1] * tgo, tp[2] + tv[2] * tgo];
                let want = norm([lead[0] - p[0], lead[1] - p[1], lead[2] - p[2]]);
                d = turn_toward(d, want, SAM_TURN_DEG.to_radians() * dt);
            }
        }
        p = [p[0] + d[0] * speed * dt, p[1] + d[1] * speed * dt, p[2] + d[2] * speed * dt];
        let q = |a: f64| (a * fine).round() as i32;
        let vel = [d[0] * speed, d[1] * speed, d[2] * speed];
        (f.x, f.y, f.alt, f.vx, f.vy, f.vz) =
            (q(p[0]), q(p[1]), q(p[2]), q(vel[0]), q(vel[1]), q(vel[2]));
        f.t += 1;
        let pm = &mut self.proj[k];
        pm.flight = Some(f);
        set_pose(pm, p, d, speed);
        pm.ttl -= 1;
        if p[2] <= self.slam_ground(p[0], p[1]) {
            self.proj[k].ttl = 0;
        }
        if let Some((s, tp)) = killed {
            self.shoot_down(s, tp);
        }
    }

    /// The stand-off weapon in own slot `s` is shot down at `p` (wu).
    fn shoot_down(&mut self, s: usize, p: V3) {
        let m = self.proj[s];
        self.proj[s].ttl = 0;
        let (x, y) = (((p[0] / 32.0) as i32) & 0xFFFF, ((p[1] / 32.0) as i32) & 0xFFFF);
        let alt = crate::math::view_z_f(p[2]) as i32;
        let long = self.weapon_types[(m.weapon.max(0) as usize).min(self.weapon_types.len() - 1)]
            .long_name
            .clone();
        self.msg(text!("sim-standoff-shot-down", weapon = long.to_string()));
        self.emit(SimEvent::WeaponShotDown { spec: m.spec, x, y, alt });
    }

    /// The active pilot's JSOWs and MALDs in flight: slot and weapon.
    pub fn standoff_rounds(&self) -> Vec<(usize, Projectile)> {
        Self::own_slots()
            .filter(|&s| self.proj[s].ttl != 0 && self.proj[s].glide.is_some())
            .map(|s| (s, self.proj[s]))
            .collect()
    }
}

/// Sets a projectile's map position and attitude from its exact flight (`p` wu, `d` its
/// direction, `speed` wu/s).
fn set_pose(pm: &mut Projectile, p: V3, d: V3, speed: f64) {
    pm.x = ((p[0] / 32.0).floor() as i32) & 0xFFFF;
    pm.y = ((p[1] / 32.0).floor() as i32) & 0xFFFF;
    pm.alt = crate::math::view_z_f(p[2]) as i32;
    pm.speed = (speed / 128.0).round() as i32;
    let (heading, pitch) = crate::guidance::heading_pitch(d);
    pm.heading = heading;
    pm.pitch = pitch;
    pm.roll = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gliding at its best speed on its best path, it neither speeds up nor slows down.
    #[test]
    fn best_glide_is_steady() {
        let a = glide_accel(BEST_GLIDE, glide_gamma(BEST_GLIDE));
        assert!(a.abs() < 0.2, "{a}");
    }

    /// High and fast it glides several times farther than low and slow: some 130 km from
    /// 30,000 ft at 470 kt, some 20 km from 1,500 ft at 450 kt.
    #[test]
    fn range_grows_with_height_and_speed() {
        let km = |h, v| glide_range(h, v) * WU_METRES / 1000.0;
        let high = km(30_000.0, 800.0);
        let low = km(1_500.0, 760.0);
        assert!((100.0..170.0).contains(&high), "{high}");
        assert!((10.0..30.0).contains(&low), "{low}");
        assert!(km(15_000.0, 760.0) > low && km(15_000.0, 760.0) < high);
        assert!(km(15_000.0, 900.0) > km(15_000.0, 600.0));
    }
}
