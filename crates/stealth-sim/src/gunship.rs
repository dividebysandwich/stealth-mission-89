//! The AC-130 gunship's side guns: a 40 mm Bofors and a 105 mm howitzer firing from the left
//! side at the point the targeting pod looks at.
//!
//! The fire control lays the selected gun on the pod's ground point (a vehicle in point track:
//! where it will be when the shell gets there), the aircraft's own speed and the shell's drop
//! included. The guns only traverse a little either side of the left wing's line and depress
//! between `GIMBAL_EL`: to bring them to bear the pilot flies a left-hand orbit round the
//! target. Each round leaves with a little dispersion, so from an orbit a few kilometres out
//! it takes a few rounds to hit a vehicle.
//!
//! Shells fly in metres (east, north, up: Bevy's world, the altitude's world units at 0.49 m)
//! with gravity and no drag; where they come down they
//! wear down the vehicles and guns they hit ([`SIDE_GUN_DAMAGE`]): the 40 mm takes 5 to 10 hits
//! to destroy a tank and 2 or 3 for anything softer, the 105 mm destroys anything but a tank
//! with one hit and a tank with 2 or 3.

use serde::{Deserialize, Serialize};

use stealth_i18n::text;

use crate::Sim;
use crate::aaa::{GroundKind, M_PER_MAP};
use crate::math::approx;
use crate::types::*;

/// The two guns: the 40 mm Bofors (0) and the 105 mm howitzer (1).
pub const GUNS: usize = 2;
/// Rounds carried.
pub const LOAD: [i32; GUNS] = [256, 100];
/// Muzzle speed (m/s).
pub const SHELL_SPEED: [f64; GUNS] = [880.0, 494.0];
/// Seconds between rounds.
pub const INTERVAL: [f64; GUNS] = [0.5, 4.0];
/// The rounds' dispersion: standard deviation of the angle off the laid line (rad).
pub const SPREAD: [f64; GUNS] = [0.0010, 0.0008];
/// The muzzles in the aircraft's body frame (metres right, forward, up from its position):
/// tools/models/c130.py `GUNS`, `gun_muzzle`.
pub const MUZZLE: [[f64; 3]; GUNS] = [[-3.05, -3.02, 2.17], [-3.10, -5.07, 2.50]];
/// How far either way of the left wing's line the guns traverse (rad, 20 degrees).
pub const GIMBAL_AZ: f64 = 0.349;
/// The guns' depression below the wings' plane: from, to (rad, 5 and 55 degrees).
pub const GIMBAL_EL: (f64, f64) = (0.087, 0.960);
/// A shell is lost after this many seconds in flight.
const SHELL_LIFE: f64 = 45.0;
/// A shell coming down this close to a vehicle hits it (metres): 40 mm, 105 mm on a tank,
/// 105 mm blast on anything softer.
const HIT_RADIUS: [f64; GUNS] = [3.0, 4.0];
const BLAST_RADIUS_105: f64 = 10.0;
/// Damage per hit (a unit has 100 points): (tank, anything else) ranges for each gun.
pub const SIDE_GUN_DAMAGE: [[(u8, u8); 2]; GUNS] = [[(10, 20), (34, 50)], [(34, 50), (100, 100)]];
/// Shells in flight at most.
const MAX_SIDE_SHELLS: usize = 96;

/// A gun's name for messages and the cockpit.
pub fn gun_name(gun: usize) -> &'static str {
    if gun == 0 { "40MM" } else { "105MM" }
}

/// A pilot's side guns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SideGuns {
    /// Rounds left in each gun.
    pub rounds: [i32; GUNS],
    /// The selected gun.
    pub gun: u8,
    /// Frames until each gun has its next round loaded.
    pub reload: [i32; GUNS],
    /// The laid gun's line is outside the gimbal (the trigger does nothing).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub out_of_gimbal: bool,
    /// The trigger was held outside the gimbal (one message per press).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub warned: bool,
    /// The fire control's solution for the selected gun: the shell's time of flight (tenths
    /// of a second) and the slant range (m); 0 without one.
    #[serde(default)]
    pub tof: i32,
    #[serde(default)]
    pub range: i32,
}

impl SideGuns {
    /// Both guns loaded.
    pub fn loaded() -> Self {
        SideGuns { rounds: LOAD, ..Default::default() }
    }

    pub fn is_default(&self) -> bool {
        *self == SideGuns::default()
    }
}

/// A side gun's shell in flight (metres east, north, up; m/s).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SideShell {
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub gun: u8,
    /// The pilot who fired it.
    pub pilot: u8,
    /// Seconds in flight.
    pub age: f64,
}

/// What the fire control has for the selected gun: the line to lay it on (world: east, north,
/// up), the shell's time of flight and slant range, and whether the gimbal reaches it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solution {
    pub dir: [f64; 3],
    pub tof: f64,
    pub range: f64,
    pub in_gimbal: bool,
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn norm(a: [f64; 3]) -> [f64; 3] {
    mul(a, 1.0 / len(a).max(1e-9))
}

/// The body axes (right, forward, up) of an aircraft at `heading`, `pitch`, `roll` (BAM, roll
/// right wing down) in the world's (east, north, up).
pub fn body_axes(heading: i32, pitch: i32, roll: i32) -> [[f64; 3]; 3] {
    let axis = |o: [f64; 3]| {
        let w = crate::weapons::body_to_world(heading, pitch, roll, o);
        [w[0], -w[1], w[2]]
    };
    [axis([1.0, 0.0, 0.0]), axis([0.0, 1.0, 0.0]), axis([0.0, 0.0, 1.0])]
}

/// A direction (world: east, north, up) in the body frame as (azimuth right of the nose,
/// elevation above the wings' plane), rad.
pub fn body_angles(axes: &[[f64; 3]; 3], d: [f64; 3]) -> (f64, f64) {
    let (r, f, u) = (dot(d, axes[0]), dot(d, axes[1]), dot(d, axes[2]));
    (r.atan2(f), u.clamp(-1.0, 1.0).asin())
}

/// The body angles reach the guns: left of the nose within `GIMBAL_AZ` of the wing's line,
/// depressed within `GIMBAL_EL`.
pub fn in_gimbal(az: f64, el: f64) -> bool {
    let off = az + std::f64::consts::FRAC_PI_2;
    off.abs() <= GIMBAL_AZ && -el >= GIMBAL_EL.0 && -el <= GIMBAL_EL.1
}

/// Where to lay a gun at `from` firing shells of `speed` from an aircraft moving at `vel` to hit
/// `target` moving at `target_vel` (all metres, m/s; east, north, up): the barrel's direction
/// and the time of flight.
pub fn lay(
    from: [f64; 3],
    vel: [f64; 3],
    target: [f64; 3],
    target_vel: [f64; 3],
    speed: f64,
) -> ([f64; 3], f64) {
    let mut t = len(sub(target, from)) / speed;
    let mut d = sub(target, from);
    for _ in 0..8 {
        let at = add(target, mul(target_vel, t));
        // The shell carries the aircraft's speed and falls 0.5 g t^2 on the way.
        d = add(sub(sub(at, from), mul(vel, t)), [0.0, 0.0, 0.5 * G_METRES * t * t]);
        t = len(d) / speed;
    }
    (norm(d), t)
}

/// Gravity (m/s^2).
const G_METRES: f64 = 9.81;

/// A unit's armour class for the side guns: tanks (0), anything else (1).
fn class(kind: GroundKind) -> usize {
    if kind.tank() { 0 } else { 1 }
}

/// The campaign tag of a gunship mission's enemy column.
pub const COLUMN_TAG: u32 = 0xC01_0000;
/// The column sets off when a gunship comes this close to its road's start (m).
pub const COLUMN_WAKE: f64 = 35_000.0;
/// Its march speed (m/s): a column picking its way across country.
pub const COLUMN_SPEED: f32 = 7.0;
/// The share of its vehicles destroyed that achieves the objective (mission type 11).
pub const COLUMN_DONE: f64 = 0.75;
/// Mission type: destroy the column (`stealth_mission::gunship::COLUMN`).
pub const COLUMN_MISSION: i32 = 11;

/// A gunship mission's enemy column: its vehicles and road (map units) until it sets off, and
/// whether it has.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MissionColumn {
    pub kinds: Vec<GroundKind>,
    pub route: Vec<(i32, i32)>,
    pub spawned: bool,
}

impl MissionColumn {
    /// The mission's column, if it has one.
    pub fn of(m: &stealth_mission::Mission) -> Option<Self> {
        let c = m.column.as_ref()?;
        let kinds = c
            .kinds
            .iter()
            .map(|&k| match k {
                stealth_mission::gunship::T90 => GroundKind::T90,
                stealth_mission::gunship::ZSU => GroundKind::Zsu,
                _ => GroundKind::Bmp2,
            })
            .collect();
        Some(MissionColumn { kinds, route: c.route.clone(), spawned: false })
    }
}

/// The AC-130's dispensers: flares and chaff.
pub const FLARES: i32 = 250;
pub const CHAFF: i32 = 250;

impl crate::pilot::Pilot {
    /// An AC-130 starts (or rearms) with both guns loaded and its big dispensers full.
    pub(crate) fn load_side_guns(&mut self) {
        if self.spectre {
            self.side_guns = SideGuns::loaded();
            self.stores[1] = FLARES;
            self.stores[2] = CHAFF;
        }
    }
}

impl Sim {
    /// The active pilot's side guns, the shells in flight.
    pub fn side_guns(&self) -> Option<SideGuns> {
        self.pl.spectre.then_some(self.pl.side_guns)
    }

    pub fn side_shells(&self) -> &[SideShell] {
        &self.side_shells
    }

    /// Selects the side gun `gun` (0: 40 mm, 1: 105 mm).
    pub fn select_side_gun(&mut self, gun: usize) {
        if self.pl.spectre && gun < GUNS {
            self.pl.side_guns.gun = gun as u8;
            self.msg(text!("sim-gun-selected", gun = gun_name(gun)));
        }
    }

    /// The mission's column ([`MissionColumn`]): it sets off once a pilot comes near its road;
    /// once most of it is destroyed, the objective of the pilots who have it.
    pub(crate) fn update_column(&mut self, present: &[usize]) {
        let Some(col) = self.column.as_ref() else { return };
        if !col.spawned {
            let Some(&(x, y)) = col.route.first() else { return };
            let (e, n) = (x as f64 * M_PER_MAP, (0x8000 - y) as f64 * M_PER_MAP);
            let m_per_wu = M_PER_MAP / 32.0;
            let near = present.iter().any(|&k| {
                let p = &self.pilot_ref(k).p;
                (p.wx as f64 * m_per_wu - e).hypot(p.wy as f64 * m_per_wu - n) < COLUMN_WAKE
            });
            if !near {
                return;
            }
            let (kinds, route) = (col.kinds.clone(), col.route.clone());
            if let Some(c) = self.column.as_mut() {
                c.spawned = true;
            }
            if self.campaign.is_none() {
                let stores = self.pl.stores;
                let gun = self.pl.gun_ammo;
                self.campaign = Some(Box::new(self.campaign_state(stores, gun)));
            }
            self.set_convoy_speed(COLUMN_SPEED);
            self.spawn_convoy(COLUMN_TAG, false, &kinds, &route);
            return;
        }
        let Some(c) = self.campaign.as_ref() else { return };
        let units: Vec<usize> = (0..self.units.len())
            .filter(|&k| c.ground.convoy_of.get(k) == Some(&COLUMN_TAG))
            .collect();
        if units.is_empty() {
            return;
        }
        let down = units.iter().filter(|&&k| self.units[k].destroyed).count();
        if (down as f64) < COLUMN_DONE * units.len() as f64 {
            return;
        }
        let back = self.pl.id as usize;
        for &k in present {
            self.activate(k);
            if self.pl.targets[0].mission_type == COLUMN_MISSION {
                self.mark_target_reached(0);
            }
        }
        self.activate(back);
    }

    /// The active pilot's aircraft: position (metres east, north, up) and velocity (m/s).
    fn own_motion(&self) -> ([f64; 3], [f64; 3]) {
        let p = &self.pl.p;
        let m_per_wu = M_PER_MAP / 32.0;
        // Altitude is in world units too (the game's "feet"), as the hills' heights are drawn.
        let pos = [p.wx as f64 * m_per_wu, p.wy as f64 * m_per_wu, p.alt as f64 * m_per_wu];
        let d = crate::guidance::direction(p.heading, p.pitch);
        let v = p.knots as f64 * 0.514_444;
        (pos, [d[0] * v, -d[1] * v, d[2] * v])
    }

    /// Where the pod's line of sight `aim` from `from` meets the ground (metres), within 15 km.
    fn pod_ground(&self, from: [f64; 3], aim: &PodAim) -> Option<[f64; 3]> {
        let d = crate::guidance::direction(aim.heading as i32, aim.pitch as i32);
        let d = [d[0], -d[1], d[2]];
        if d[2] >= -0.01 {
            return None;
        }
        let hills = self.hills();
        let mut prev = from;
        let mut s = 0.0;
        while s < 15_000.0 {
            s += 25.0;
            let p = add(from, mul(d, s));
            let g = hills.height_m(p[0], p[1]) as f64;
            if p[2] <= g {
                // Back up to the crossing.
                let gp = hills.height_m(prev[0], prev[1]) as f64;
                let (a, b) = (prev[2] - gp, g - p[2]);
                let k = if a + b > 0.0 { a / (a + b) } else { 1.0 };
                let at = add(prev, mul(sub(p, prev), k));
                return Some([at[0], at[1], hills.height_m(at[0], at[1]) as f64]);
            }
            prev = p;
        }
        None
    }

    /// The fire control's solution for the selected gun, from the pod's line of sight (None:
    /// no pod, or it looks at the sky).
    pub(crate) fn side_gun_solution(&self, ps: &PlayerState) -> Option<Solution> {
        let aim = ps.pod?;
        let gun = self.pl.side_guns.gun as usize;
        let p = &self.pl.p;
        let axes = body_axes(p.heading, p.pitch, p.roll);
        let (pos, vel) = self.own_motion();
        let muzzle = add(
            pos,
            add(
                add(mul(axes[0], MUZZLE[gun][0]), mul(axes[1], MUZZLE[gun][1])),
                mul(axes[2], MUZZLE[gun][2]),
            ),
        );
        // A vehicle in point track: lead it.
        let tracked = self
            .pl
            .spi_unit
            .map(|k| k as usize)
            .filter(|&k| self.units.get(k).is_some_and(|u| !u.destroyed && u.fine.is_some()));
        let (target, target_vel) = match tracked {
            Some(k) => {
                let u = &self.units[k];
                let at = u.metres_on(&self.hills());
                let v = if self.unit_moving(k) { self.convoy_speed() } else { 0.0 };
                let h = u.fine.map_or(0.0, |f| f.heading as f64);
                (at, [h.sin() * v, h.cos() * v, 0.0])
            }
            None => (self.pod_ground(pos, &aim)?, [0.0; 3]),
        };
        let (dir, tof) = lay(muzzle, vel, target, target_vel, SHELL_SPEED[gun]);
        let (az, el) = body_angles(&axes, dir);
        Some(Solution { dir, tof, range: len(sub(target, muzzle)), in_gimbal: in_gimbal(az, el) })
    }

    /// The active pilot's side guns this frame: reloading, and a round from the selected gun
    /// while the trigger is held and the gun bears.
    pub(crate) fn update_side_guns(&mut self, ps: &PlayerState) {
        if !self.pl.spectre {
            return;
        }
        let fs = self.fs.max(1);
        for r in &mut self.pl.side_guns.reload {
            *r = (*r - 1).max(0);
        }
        let solution = self.side_gun_solution(ps);
        let gun = self.pl.side_guns.gun as usize;
        self.pl.side_guns.out_of_gimbal = !solution.is_some_and(|s| s.in_gimbal);
        self.pl.side_guns.tof = solution.map_or(0, |s| (s.tof * 10.0).round() as i32);
        self.pl.side_guns.range = solution.map_or(0, |s| s.range.round() as i32);
        let trigger = (ps.cmd.trigger || ps.cmd.gun || ps.cmd.fire)
            && self.pl.eject == 0
            && !self.pl.on_ground;
        if !trigger {
            self.pl.side_guns.warned = false;
            self.pl.gun_firing = false;
            return;
        }
        let Some(s) = solution.filter(|s| s.in_gimbal) else {
            if !self.pl.side_guns.warned {
                self.pl.side_guns.warned = true;
                if solution.is_some() {
                    self.msg(text!("sim-gun-gimbal"));
                } else {
                    self.msg(text!("sim-gun-no-pod"));
                }
            }
            self.pl.gun_firing = false;
            return;
        };
        if self.pl.side_guns.rounds[gun] <= 0 {
            if !self.pl.side_guns.warned {
                self.pl.side_guns.warned = true;
                self.msg(text!("sim-gun-empty", gun = gun_name(gun)));
            }
            return;
        }
        if self.pl.side_guns.reload[gun] > 0 || self.side_shells.len() >= MAX_SIDE_SHELLS {
            return;
        }
        // The round's dispersion: two roughly normal angles across the line.
        let gauss = |rng: &mut crate::rng::Rng| {
            (0..4).map(|_| rng.random(1000) as f64 / 1000.0).sum::<f64>() * 1.732 - 3.464
        };
        let (a, b) = (gauss(&mut self.rng) * SPREAD[gun], gauss(&mut self.rng) * SPREAD[gun]);
        let side = norm([s.dir[1], -s.dir[0], 0.0]);
        let up = [
            side[1] * s.dir[2] - side[2] * s.dir[1],
            side[2] * s.dir[0] - side[0] * s.dir[2],
            side[0] * s.dir[1] - side[1] * s.dir[0],
        ];
        let dir = norm(add(s.dir, add(mul(side, a), mul(up, b))));
        let p = &self.pl.p;
        let axes = body_axes(p.heading, p.pitch, p.roll);
        let (pos, vel) = self.own_motion();
        let m = MUZZLE[gun];
        let muzzle = add(pos, add(add(mul(axes[0], m[0]), mul(axes[1], m[1])), mul(axes[2], m[2])));
        let shell_vel = add(vel, mul(dir, SHELL_SPEED[gun]));
        self.side_shells.push(SideShell {
            pos: muzzle,
            vel: shell_vel,
            gun: gun as u8,
            pilot: self.pl.id,
            age: 0.0,
        });
        self.pl.side_guns.rounds[gun] -= 1;
        self.pl.side_guns.reload[gun] = (INTERVAL[gun] * fs as f64).round() as i32;
        self.pl.gun_firing = true;
        self.pl.fired_step = self.steps;
        let dm = |v: f64| (v * 10.0).round() as i32;
        self.emit(SimEvent::SideGunFired {
            gun: gun as u8,
            pilot: self.pl.id,
            from: [dm(muzzle[0]), dm(muzzle[1]), dm(muzzle[2])],
            vel: [dm(shell_vel[0]), dm(shell_vel[1]), dm(shell_vel[2])],
        });
    }

    /// One frame of the side guns' shells: gravity, and what they hit where they come down.
    pub(crate) fn update_side_shells(&mut self) {
        if self.side_shells.is_empty() {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let hills = self.hills();
        let mut down = Vec::new();
        self.side_shells.retain_mut(|s| {
            let last = s.pos;
            // Exact under constant gravity, whatever the frame rate.
            s.pos = add(s.pos, mul(s.vel, dt));
            s.pos[2] -= 0.5 * G_METRES * dt * dt;
            s.vel[2] -= G_METRES * dt;
            s.age += dt;
            let g = hills.height_m(s.pos[0], s.pos[1]) as f64;
            if s.pos[2] <= g {
                let gl = hills.height_m(last[0], last[1]) as f64;
                let (a, b) = (last[2] - gl, g - s.pos[2]);
                let k = if a + b > 0.0 { a / (a + b) } else { 1.0 };
                let at = add(last, mul(sub(s.pos, last), k));
                down.push((s.pilot, s.gun as usize, [at[0], at[1], g]));
                return false;
            }
            s.age < SHELL_LIFE
        });
        for (pilot, gun, at) in down {
            self.side_shell_impact(pilot, gun, at);
        }
    }

    /// Test aid: a shell of pilot `pilot`'s gun `gun` comes down at `at` (metres east, north,
    /// up).
    pub fn drop_side_shell(&mut self, pilot: u8, gun: usize, at: [f64; 3]) {
        self.side_shell_impact(pilot, gun, at);
    }

    /// A shell of pilot `pilot`'s gun `gun` came down at `at` (metres east, north, up): the
    /// vehicles and guns it hits, the objects it destroys.
    fn side_shell_impact(&mut self, pilot: u8, gun: usize, at: [f64; 3]) {
        let back = self.pl.id as usize;
        let shooter =
            (0..self.pilot_count()).find(|&k| self.pilot_ref(k).id == pilot).unwrap_or(back);
        self.activate(shooter);
        self.credit = Some(pilot);
        let hills = self.hills();
        let mut armour = false;
        if self.side_gun_wear.len() < self.units.len() {
            self.side_gun_wear.resize(self.units.len(), 0);
        }
        for k in 0..self.units.len() {
            let u = self.units[k];
            if u.destroyed {
                continue;
            }
            let m = u.metres_on(&hills);
            let d = (m[0] - at[0]).hypot(m[1] - at[1]);
            let class = class(u.kind);
            let reach = match (gun, class) {
                (0, _) => HIT_RADIUS[0],
                (_, 0) => HIT_RADIUS[1],
                _ => BLAST_RADIUS_105,
            };
            if d > reach {
                continue;
            }
            armour |= u.kind.armour() || matches!(u.kind, GroundKind::Zsu | GroundKind::Gepard);
            let (lo, hi) = SIDE_GUN_DAMAGE[gun][class];
            let dmg = lo as i32 + self.rng.random((hi - lo) as i32 + 1);
            let wear = self.side_gun_wear[k] as i32 + dmg;
            if wear >= 100 {
                self.side_gun_wear[k] = 0;
                self.units[k].destroyed = true;
                self.units[k].burst = 0;
                self.msg(text!("sim-destroyed", target = u.kind.name()));
                self.emit(SimEvent::Explosion { x: u.x, y: u.y, alt: 0, kind: 0x10 });
                self.emit(SimEvent::AaaDestroyed { unit: k, by: Some(pilot) });
            } else {
                self.side_gun_wear[k] = wear as u8;
            }
        }
        let dm = |v: f64| (v * 10.0).round() as i32;
        self.emit(SimEvent::SideShellImpact {
            gun: gun as u8,
            at: [dm(at[0]), dm(at[1]), dm(at[2])],
            armour,
        });
        // A building or a site: the 105 mm as a cannon's heavy round always, the 40 mm now and
        // then.
        let (bx, by) = ((at[0] / M_PER_MAP) as i32, 0x8000 - (at[1] / M_PER_MAP) as i32);
        if gun == 1 || self.rng.random(3) == 0 {
            self.side_gun_object(bx, by);
        }
        self.credit = None;
        self.activate(back);
    }

    /// A side gun's shell at map point (bx, by) destroys the object there (as a heavy cannon
    /// round would).
    fn side_gun_object(&mut self, bx: i32, by: i32) {
        let o = self.find_object_at(bx, by);
        if o == -1 || self.obj(o).flags & oflags::DESTROYED != 0 {
            return;
        }
        let t = self.pl.last_tile;
        if !self.tile_is_intact(&t) {
            return;
        }
        let (tx, ty) = t.map_or((self.obj(o).x, self.obj(o).y), |t| t.map_pos());
        let d = approx(tx - bx, ty - by);
        if self.compat(18, o) * 6 / (self.diff + 2) > d {
            self.destroy_ground(o);
            let n = self.place_name(o as usize);
            self.msg(text!("sim-destroyed-gunfire", target = n.to_string()));
            self.emit(SimEvent::Explosion { x: tx, y: ty, alt: 0, kind: 0x10 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gimbal_is_off_the_left_wing() {
        // Level: straight out of the left wing, 30 degrees down.
        let axes = body_axes(0, 0, 0);
        let d = [-(30f64.to_radians().cos()), 0.0, -(30f64.to_radians().sin())];
        let (az, el) = body_angles(&axes, d);
        assert!(in_gimbal(az, el), "{az} {el}");
        // Ahead and down: no.
        let (az, el) = body_angles(&axes, [0.0, 0.7, -0.7]);
        assert!(!in_gimbal(az, el));
        // Out of the right wing: no.
        let (az, el) = body_angles(&axes, [0.86, 0.0, -0.5]);
        assert!(!in_gimbal(az, el));
        // Banked 30 degrees left, a target 60 degrees below the horizon to the left is 30
        // degrees below the wings: in; straight down is 60 below them: out.
        let bank = -(30.0f64 / 360.0 * 65536.0) as i32;
        let axes = body_axes(0, 0, bank);
        let d = [-(60f64.to_radians().cos()), 0.0, -(60f64.to_radians().sin())];
        let (az, el) = body_angles(&axes, d);
        assert!(in_gimbal(az, el), "{az} {el}");
        let (az, el) = body_angles(&axes, [0.0, 0.0, -1.0]);
        assert!(!in_gimbal(az, el), "{az} {el}");
    }

    #[test]
    fn laid_shells_hit_the_aim_point() {
        // An orbit 3 km up flying north at 150 m/s, the target 2 km to the west.
        let from = [0.0, 0.0, 3000.0];
        let vel = [0.0, 150.0, 0.0];
        for (gun, &speed) in SHELL_SPEED.iter().enumerate() {
            for target_vel in [[0.0; 3], [12.0, -9.0, 0.0]] {
                let target = [-2000.0, 300.0, 0.0];
                let (dir, tof) = lay(from, vel, target, target_vel, speed);
                // Fly it.
                let mut p = from;
                let mut v = add(vel, mul(dir, speed));
                let dt = 1.0 / 60.0;
                let mut t = 0.0;
                while p[2] > 0.0 {
                    let last = p;
                    p = add(p, mul(v, dt));
                    v[2] -= G_METRES * dt;
                    t += dt;
                    if p[2] <= 0.0 {
                        // Back to where it crossed the ground.
                        let k = last[2] / (last[2] - p[2]);
                        p = add(last, mul(sub(p, last), k));
                        t -= dt * (1.0 - k);
                    }
                }
                let at = add(target, mul(target_vel, t));
                let miss = (p[0] - at[0]).hypot(p[1] - at[1]);
                assert!(miss < 3.0, "gun {gun}: missed by {miss} m (tof {tof} vs {t})");
            }
        }
    }
}
