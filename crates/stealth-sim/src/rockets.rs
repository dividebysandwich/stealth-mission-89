//! Rockets: the 70 mm Hydra 70 (unguided) and the APKWS (the same rocket with a laser seeker),
//! fired from 7- or 19-tube pods, and the unguided 5 in Zuni from the four-tube LAU-10: fewer,
//! heavier and longer ranged, with a bigger warhead ([`Motor`], [`Warhead`]).
//!
//! A press of the fire button with a rocket selected fires a pair, one from each side's pod in
//! turn; holding the trigger ripples on, a pair every `RIPPLE_SECS`. A rocket leaves its tube
//! along the aircraft's nose, its motor burns for its [`Motor::burn`], then it coasts and falls; it goes
//! off where it meets the ground. An unguided one scatters a little (`DISPERSION`): it is an
//! area weapon, aimed by the CCIP rocket reticle ([`impact`] predicts where one fired now lands).
//! An APKWS steers after `SEEKER_DELAY` towards the targeting pod's laser spot (the firing
//! pilot's pod's ground point, while it has one).
//!
//! Positions are in metres east, north and up (like [`crate::jp233`]).

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::Sim;
use crate::aaa::M_PER_MAP;
use crate::types::*;

/// The Hydra 70's, the APKWS's and the Zuni's weapon types and missile table entries.
pub const HYDRA_WEAPON: i32 = 31;
pub const APKWS_WEAPON: i32 = 32;
pub const ZUNI_WEAPON: i32 = 36;
pub const HYDRA_SPEC: i32 = 56;
pub const APKWS_SPEC: i32 = 57;
pub const ZUNI_SPEC: i32 = 59;

/// How a kind of rocket flies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motor {
    /// Speed off the tube (m/s, on top of the aircraft's).
    pub launch: f64,
    /// The motor's burn (s) and its push (m/s²).
    pub burn: f64,
    pub thrust: f64,
    /// Air drag: deceleration per (m/s)².
    pub drag: f64,
    /// The range it is of use to (m): the reticle's "in range".
    pub reach: f64,
}

/// The Hydra 70's Mk 66 motor: some 700 m/s at burnout from a hover.
pub const HYDRA_MOTOR: Motor =
    Motor { launch: 45.0, burn: 1.05, thrust: 640.0, drag: 7.0e-5, reach: 5000.0 };
/// The Zuni's Mk 71 motor: a longer burn to much the same speed (some 720 m/s), and five times
/// the weight on three times the frontal area, so it keeps its speed longer.
pub const ZUNI_MOTOR: Motor =
    Motor { launch: 40.0, burn: 1.5, thrust: 500.0, drag: 3.5e-5, reach: 8000.0 };

/// A warhead: units within `kill_m` are likely destroyed, within `blast_m` maybe (odds in
/// percent for a tank and anything else); objects (buildings, parked aircraft, launchers)
/// within `object_m` may be wrecked.
struct Warhead {
    kill_m: f64,
    blast_m: f64,
    kill_tank: i32,
    kill_other: i32,
    blast_tank: i32,
    blast_other: i32,
    object_m: f64,
}

/// The Hydra 70's and the APKWS's M151 (4.5 kg) and the Zuni's Mk 24 general purpose warhead
/// (22 kg).
const M151: Warhead = Warhead {
    kill_m: 4.0,
    blast_m: 11.0,
    kill_tank: 45,
    kill_other: 95,
    blast_tank: 5,
    blast_other: 55,
    object_m: 15.0,
};
const MK24: Warhead = Warhead {
    kill_m: 7.0,
    blast_m: 18.0,
    kill_tank: 70,
    kill_other: 98,
    blast_tank: 12,
    blast_other: 70,
    object_m: 25.0,
};

pub const GRAVITY: f64 = 9.81;
/// Seconds a rocket flies at most.
pub const MAX_SECS: f64 = 25.0;
/// An unguided rocket's scatter: its direction off the tube is off by up to this (rad).
const DISPERSION: f64 = 0.003;
/// Seconds between pairs while the trigger is held.
pub const RIPPLE_SECS: f64 = 0.25;
/// The APKWS: its seeker steers from this long after launch (s), at most this hard (m/s²).
const SEEKER_DELAY: f64 = 0.4;
const SEEKER_ACCEL: f64 = 200.0;
/// How fast it closes the angle to the spot (1/s).
const SEEKER_GAIN: f64 = 10.0;
/// The frame step of the CCIP prediction (s).
const PREDICT_DT: f64 = 1.0 / 30.0;

const FT: f64 = 0.3048;
const M_PER_WU: f64 = M_PER_MAP / 32.0;

/// A rocket in flight.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rocket {
    /// Drawn as the same one from frame to frame.
    pub id: u16,
    /// Position and velocity (m, m/s: east, north, up).
    pub p: [f32; 3],
    pub v: [f32; 3],
    /// Seconds since launch.
    pub t: f32,
    /// The pilot who fired it.
    pub owner: u8,
    /// An APKWS (it homes on its pilot's laser spot).
    pub guided: bool,
    /// A Zuni.
    #[serde(default)]
    pub zuni: bool,
}

impl Rocket {
    /// Its weapon type.
    pub fn weapon(&self) -> i32 {
        match (self.zuni, self.guided) {
            (true, _) => ZUNI_WEAPON,
            (false, true) => APKWS_WEAPON,
            (false, false) => HYDRA_WEAPON,
        }
    }

    pub fn motor(&self) -> &'static Motor {
        motor(self.weapon())
    }

    /// The motor is burning (its flame and smoke).
    pub fn burning(&self) -> bool {
        (self.t as f64) < self.motor().burn
    }
}

/// Every rocket in flight.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Rockets {
    pub rockets: Vec<Rocket>,
    #[serde(default)]
    pub next_id: u16,
}

impl Rockets {
    pub fn is_empty(&self) -> bool {
        self.rockets.is_empty()
    }

    fn id(&mut self) -> u16 {
        self.next_id = self.next_id.wrapping_add(1);
        self.next_id
    }
}

/// Weapon type `w` is a rocket pod.
pub fn rocket_weapon(w: i32) -> bool {
    matches!(w, HYDRA_WEAPON | APKWS_WEAPON | ZUNI_WEAPON)
}

/// The motor of rocket weapon type `w`.
pub fn motor(w: i32) -> &'static Motor {
    if w == ZUNI_WEAPON { &ZUNI_MOTOR } else { &HYDRA_MOTOR }
}

/// The motor of the rocket of missile table entry `spec`.
pub fn spec_motor(spec: i32) -> &'static Motor {
    if spec == ZUNI_SPEC { &ZUNI_MOTOR } else { &HYDRA_MOTOR }
}

fn warhead(w: i32) -> &'static Warhead {
    if w == ZUNI_WEAPON { &MK24 } else { &M151 }
}

/// Missile table entry `spec` is a rocket.
pub fn rocket_spec(spec: i32) -> bool {
    matches!(spec, HYDRA_SPEC | APKWS_SPEC | ZUNI_SPEC)
}

/// One step of a rocket's flight `t` s after launch: its motor `m` along its velocity, drag and
/// gravity (m, m/s: east, north, up), and the steering `steer` (m/s²) on top.
fn step(m: &Motor, p: &mut [f64; 3], v: &mut [f64; 3], t: f64, dt: f64, steer: [f64; 3]) {
    let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1.0);
    let push = if t < m.burn { m.thrust } else { 0.0 };
    let along = push - m.drag * speed * speed;
    for k in 0..3 {
        v[k] += (v[k] / speed * along + steer[k]) * dt;
    }
    v[2] -= GRAVITY * dt;
    for k in 0..3 {
        p[k] += v[k] * dt;
    }
}

/// Where a rocket with motor `m` fired from `p` with velocity `v` (m, m/s: east, north, up; the
/// tube's direction times [`Motor::launch`] plus the aircraft's velocity) meets the ground,
/// `ground` giving its height (m) at a point east, north: the point and the seconds it takes.
/// None if it does not come down within `MAX_SECS`.
pub fn impact(
    m: &Motor,
    p: [f64; 3],
    v: [f64; 3],
    ground: impl Fn(f64, f64) -> f64,
) -> Option<([f64; 3], f64)> {
    let (mut p, mut v) = (p, v);
    let mut t = 0.0;
    while t < MAX_SECS {
        let before = p;
        step(m, &mut p, &mut v, t, PREDICT_DT, [0.0; 3]);
        t += PREDICT_DT;
        let g = ground(p[0], p[1]);
        if p[2] <= g {
            // Back along the step to where it crossed.
            let g0 = ground(before[0], before[1]);
            let (a, b) = (before[2] - g0, g - p[2]);
            let f = if a + b > 1e-9 { a / (a + b) } else { 1.0 };
            let at = std::array::from_fn(|k| before[k] + (p[k] - before[k]) * f);
            return Some((at, t - PREDICT_DT * (1.0 - f)));
        }
    }
    None
}

impl Sim {
    /// The rockets in flight (for drawing).
    pub fn rockets(&self) -> &Rockets {
        &self.rockets
    }

    /// The ground (m) under a point (m east, north).
    fn rocket_ground(&self, e: f64, n: f64) -> f64 {
        let (x, y) = (e / M_PER_WU, 0x10_0000 as f64 - n / M_PER_WU);
        self.slam_ground(x, y) * FT
    }

    /// Where the active pilot's station `station` launches from (m east, north, up; the
    /// aircraft's position without a known station), the aircraft's velocity and the nose's
    /// direction (m/s, unit vector: east, north, up).
    fn rocket_launch(&self, station: Option<usize>) -> ([f64; 3], [f64; 3], [f64; 3]) {
        let p = &self.pl.p;
        let v = crate::ballistics::release_velocity(p.heading, p.velocity, p.climb);
        // The world's ft/s east, south, up to m/s east, north, up.
        let v = [v[0] * M_PER_WU, -v[1] * M_PER_WU, v[2] * FT];
        let at = station.and_then(|s| self.station_point(s)).unwrap_or([
            p.wx as f64,
            (0x10_0000 - p.wy) as f64,
            p.alt as f64 - 5.0,
        ]);
        let at = [at[0] * M_PER_WU, (0x10_0000 as f64 - at[1]) * M_PER_WU, at[2] * FT];
        let f = crate::weapons::body_to_world(p.heading, p.pitch, p.roll, [0.0, 1.0, 0.0]);
        let len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt().max(1e-9);
        (at, v, [f[0] / len, -f[1] / len, f[2] / len])
    }

    /// Where a rocket fired now would land (m east, north, up) and its time of flight (s): the
    /// CCIP rocket reticle's point. None without a rocket selected or if it would not come down.
    pub fn rocket_ccip(&self) -> Option<([f64; 3], f64)> {
        let w = self.selected_weapon().0;
        if !rocket_weapon(w) {
            return None;
        }
        let m = motor(w);
        let (p, v, nose) = self.rocket_launch(None);
        let v = std::array::from_fn(|k| v[k] + nose[k] * m.launch);
        impact(m, p, v, |e, n| self.rocket_ground(e, n))
    }

    /// The fire button with a rocket selected: a pair, one from each side, then a pair every
    /// `RIPPLE_SECS` while the trigger stays held.
    pub(crate) fn fire_rockets(&mut self) {
        let (w, count) = self.selected_weapon();
        if count <= 0 {
            return;
        }
        for _ in 0..2 {
            if !self.fire_rocket(w) {
                break;
            }
        }
        self.pl.rocket_ripple = (RIPPLE_SECS * self.fs as f64).round().max(1.0) as i32;
        self.sound(0x10, 1);
    }

    /// The trigger held after a pair: the next pair when its time comes.
    pub(crate) fn ripple_rockets(&mut self, held: bool) {
        if self.pl.rocket_ripple == 0 {
            return;
        }
        let w = self.selected_weapon().0;
        if !held || !rocket_weapon(w) || self.pl.eject != 0 {
            self.pl.rocket_ripple = 0;
            return;
        }
        self.pl.rocket_ripple -= 1;
        if self.pl.rocket_ripple == 0 {
            self.fire_rockets();
        }
    }

    /// One rocket of weapon `w` from the station with the most left (the sides take turns);
    /// false with none left.
    fn fire_rocket(&mut self, w: i32) -> bool {
        let station = self.firing_station(w);
        if self.pl.bays.get(station).is_none_or(|b| b.0 != w || b.1 <= 0) {
            return false;
        }
        self.pl.bays[station].1 -= 1;
        let (p, v, nose) = self.rocket_launch(Some(station));
        let guided = w == APKWS_WEAPON;
        let zuni = w == ZUNI_WEAPON;
        // An unguided rocket's scatter: a small random turn of its direction.
        let mut dir = nose;
        if !guided {
            let r = |s: &mut Self| (s.rng.random(2001) - 1000) as f64 / 1000.0 * DISPERSION;
            let (a, b) = (r(self), r(self));
            // Two directions square to the nose.
            let side = [nose[1], -nose[0], 0.0];
            let sl = (side[0] * side[0] + side[1] * side[1]).sqrt().max(1e-9);
            let side = side.map(|c| c / sl);
            let up = [
                nose[1] * side[2] - nose[2] * side[1],
                nose[2] * side[0] - nose[0] * side[2],
                nose[0] * side[1] - nose[1] * side[0],
            ];
            dir = std::array::from_fn(|k| nose[k] + side[k] * a + up[k] * b);
        }
        let id = self.rockets.id();
        self.rockets.rockets.push(Rocket {
            id,
            p: p.map(|c| c as f32),
            v: std::array::from_fn(|k| (v[k] + dir[k] * motor(w).launch) as f32),
            t: 0.0,
            owner: self.pl.id,
            guided,
            zuni,
        });
        self.emit(SimEvent::RocketFired { at: fine(p.map(|c| c as f32)), guided, zuni });
        true
    }

    /// The laser spot pilot `owner`'s APKWS steers at (m east, north, up): its pod's ground
    /// point, if the pilot is here and the pod has one.
    fn laser_spot(&mut self, owner: usize, present: &[usize]) -> Option<[f64; 3]> {
        if !present.contains(&owner) {
            return None;
        }
        self.activate(owner);
        let (x, y) = self.pl.last_input.pod.and_then(|p| p.ground)?;
        // On the map point as a ground unit standing on it is (`GroundUnit::map_xy`).
        let [e, n] = crate::asw::map_metres((x, y));
        Some([e, n, self.rocket_ground(e, n)])
    }

    /// One frame of the rockets in flight (the host's; a mirror draws what it is sent).
    pub(crate) fn update_rockets(&mut self, present: &[usize]) {
        if self.rockets.is_empty() || self.mirror {
            return;
        }
        let back = self.pl.id as usize;
        let dt = 1.0 / self.fs.max(1) as f64;
        let mut rockets = std::mem::take(&mut self.rockets.rockets);
        let mut burst = Vec::new();
        for r in &mut rockets {
            let mut p = r.p.map(|c| c as f64);
            let mut v = r.v.map(|c| c as f64);
            let t = r.t as f64;
            let mut steer = [0.0; 3];
            if r.guided
                && t >= SEEKER_DELAY
                && let Some(spot) = self.laser_spot(r.owner as usize, present)
            {
                // Turn the velocity towards the spot (gravity made up for), as hard as it can.
                let to: [f64; 3] = std::array::from_fn(|k| spot[k] - p[k]);
                let d = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1.0);
                let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1.0);
                let want: [f64; 3] = std::array::from_fn(|k| to[k] / d * speed);
                let mut a: [f64; 3] = std::array::from_fn(|k| (want[k] - v[k]) * SEEKER_GAIN);
                a[2] += GRAVITY;
                let am = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
                if am > SEEKER_ACCEL {
                    a = a.map(|c| c * SEEKER_ACCEL / am);
                }
                steer = a;
            }
            let before = p;
            step(r.motor(), &mut p, &mut v, t, dt, steer);
            r.t = (t + dt) as f32;
            let g = self.rocket_ground(p[0], p[1]);
            if p[2] <= g {
                let g0 = self.rocket_ground(before[0], before[1]);
                let (a, b) = (before[2] - g0, g - p[2]);
                let f = if a + b > 1e-9 { a / (a + b) } else { 1.0 };
                p = std::array::from_fn(|k| before[k] + (p[k] - before[k]) * f);
                r.p = p.map(|c| c as f32);
                burst.push(*r);
                r.t = f32::MAX;
                continue;
            }
            r.p = p.map(|c| c as f32);
            r.v = v.map(|c| c as f32);
        }
        rockets.retain(|r| r.t != f32::MAX && (r.t as f64) < MAX_SECS);
        self.rockets.rockets = rockets;
        for r in burst {
            self.rocket_burst(&r);
        }
        self.activate(back);
    }

    /// Ground unit `k` destroyed by pilot `owner`'s rocket.
    fn rocket_kills_unit(&mut self, k: usize, owner: u8) {
        if self.units[k].destroyed {
            return;
        }
        self.activate(owner as usize);
        let u = self.units[k];
        self.units[k].destroyed = true;
        self.units[k].burst = 0;
        let blue = self.aaa_blue(k);
        self.msg(text!("sim-destroyed", target = u.kind.name_on(blue)));
        self.emit(SimEvent::AaaDestroyed { unit: k, by: Some(owner) });
    }

    /// World object `k` destroyed by pilot `owner`'s rocket of weapon `w`.
    fn rocket_kills_object(&mut self, k: usize, owner: u8, w: i32) {
        if self.objects[k].flags & oflags::DESTROYED != 0 {
            return;
        }
        self.activate(owner as usize);
        let long =
            self.weapon_types[(w as usize).min(self.weapon_types.len() - 1)].long_name.clone();
        self.credit = Some(owner);
        self.destroy_ground(k as i32);
        self.credit = None;
        let place = self.place_name(k);
        self.msg(text!("sim-destroyed-by", target = place, weapon = long));
    }

    /// A rocket's warhead goes off where it came down: the vehicles and objects by it.
    fn rocket_burst(&mut self, r: &Rocket) {
        let (e, n) = (r.p[0] as f64, r.p[1] as f64);
        let w = r.weapon();
        let head = warhead(w);
        let mut kill = false;
        let near: Vec<(usize, f64)> = (0..self.units.len())
            .filter_map(|k| {
                let u = &self.units[k];
                if u.destroyed || u.gone {
                    return None;
                }
                let (x, y) = u.map_xy();
                let at = [x * M_PER_MAP, (0x8000 as f64 - y) * M_PER_MAP];
                let d = (at[0] - e).hypot(at[1] - n);
                (d < head.blast_m).then_some((k, d))
            })
            .collect();
        for (k, d) in near {
            let tank = self.units[k].kind.tank();
            let odds = match (d < head.kill_m, tank) {
                (true, true) => head.kill_tank,
                (true, false) => head.kill_other,
                (false, true) => head.blast_tank,
                (false, false) => head.blast_other,
            };
            if self.rng.random(100) < odds {
                self.rocket_kills_unit(k, r.owner);
                kill = true;
            }
        }
        let objects: Vec<usize> = (1..self.objects.len())
            .filter(|&k| {
                let o = &self.objects[k];
                if o.flags & oflags::DESTROYED != 0 || self.compat(w, k as i32) <= 0 {
                    return false;
                }
                let at = crate::asw::object_metres(o);
                (at[0] - e).hypot(at[1] - n) < head.object_m
            })
            .collect();
        for k in objects {
            if self.rng.random(100) < self.compat(w, k as i32) * 10 {
                self.rocket_kills_object(k, r.owner, w);
                kill = true;
            }
        }
        self.emit(SimEvent::RocketBurst { at: fine(r.p), kill, zuni: r.zuni });
    }
}

/// Fine units (`ROUND_FINE` world units; true altitude in fine ft) of a point in metres.
fn fine(p: [f32; 3]) -> (i32, i32, i32) {
    let f = ROUND_FINE as f64;
    let wu = |m: f32| m as f64 / M_PER_WU;
    (
        (wu(p[0]) * f).round() as i32,
        ((0x10_0000 as f64 - wu(p[1])) * f).round() as i32,
        (p[2] as f64 / FT * f).round() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fired level from a hover 100 m up over flat ground, it comes down some 1.5 to 4 km
    /// ahead in a few seconds; pointed down more, it lands nearer and sooner.
    #[test]
    fn a_rocket_falls_ahead() {
        let m = &HYDRA_MOTOR;
        let level = impact(m, [0.0, 0.0, 100.0], [0.0, m.launch, 0.0], |_, _| 0.0).unwrap();
        assert!((1500.0..4500.0).contains(&level.0[1]), "{level:?}");
        assert!(level.0[0].abs() < 1e-6);
        let dive = [0.0, m.launch * 0.94, -m.launch * 0.34];
        let down = impact(m, [0.0, 0.0, 100.0], dive, |_, _| 0.0).unwrap();
        assert!(down.0[1] < level.0[1] && down.1 < level.1, "{down:?} {level:?}");
        // Straight up it does not come down in time.
        assert!(impact(m, [0.0, 0.0, 100.0], [0.0, 0.0, m.launch], |_, _| 0.0).is_none());
    }

    /// The Zuni burns out about as fast as the Hydra 70 and carries farther: fired level from
    /// 1,500 m up it comes down well beyond it.
    #[test]
    fn a_zuni_outranges_a_hydra() {
        let from = [0.0, 0.0, 1500.0];
        let fall = |m: &Motor| impact(m, from, [0.0, 150.0 + m.launch, 0.0], |_, _| 0.0).unwrap();
        let (hydra, zuni) = (fall(&HYDRA_MOTOR), fall(&ZUNI_MOTOR));
        assert!(zuni.0[1] > 1.1 * hydra.0[1] && zuni.1 < hydra.1, "{zuni:?} {hydra:?}");
        // Burnout speed from rest, without drag: thrust times burn.
        let burnout = |m: &Motor| m.thrust * m.burn;
        assert!((650.0..800.0).contains(&burnout(&ZUNI_MOTOR)));
        assert!((burnout(&ZUNI_MOTOR) / burnout(&HYDRA_MOTOR) - 1.0).abs() < 0.2);
    }
}
