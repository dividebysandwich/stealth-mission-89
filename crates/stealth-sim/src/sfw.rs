//! The CBU-105 sensor fuzed weapon after its dispenser opens.
//!
//! The dispenser (a guided bomb, `missiles.rs`) opens at its burst altitude and throws out
//! ten BLU-108 submunitions, the forward five, then the aft five. Each falls on its chute,
//! spreading out to some 70 m from the next; low down it lets the chute go, fires a rocket
//! that stops its fall and spins it, and throws out its four skeets in pairs, 90 degrees
//! apart. A skeet falls coning, its sensors looking at the ground in a cone below it; finding
//! a vehicle it fires an explosively formed penetrator down at it. One that finds nothing
//! destroys itself 50 ft above the ground.
//!
//! Positions are in world units (ft: x east, y south like the map, true altitude up), like
//! [`crate::types::BombFall`] without the fine scale.

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::Sim;
use crate::aaa::GroundKind;
use crate::ballistics;
use crate::types::*;

/// Seconds from the dispenser opening to the chutes (the aft bay's later).
const EJECT_SECS: f64 = 0.4;
const AFT_BAY_SECS: f64 = 0.15;
/// The chute: the time a submunition's speed over the ground halves and its sink rate (ft/s)
/// settles in, and the sink rate.
const CHUTE_TAU: f64 = 1.5;
const CHUTE_TAU_SINK: f64 = 1.0;
const CHUTE_SINK: f64 = 50.0;
/// Height above the ground (ft) the rocket fires at, how long it burns, and when the two
/// pairs of skeets go (s into the burn).
const ROCKET_FT: f64 = 330.0;
const ROCKET_SECS: f64 = 1.0;
const SKEET_PAIRS: [f64; 2] = [0.3, 0.6];
/// A spent submunition falls on to the ground (ft/s at the most).
const SPENT_TERMINAL: f64 = 150.0;
/// The pattern of the ten submunitions at the full burst altitude: along the path five
/// apart by `PATTERN_ALONG`, across it two rows `PATTERN_ACROSS` from the path (ft).
const PATTERN_ALONG: f64 = 230.0;
const PATTERN_ACROSS: f64 = 60.0;
/// A skeet: thrown out at this speed (ft/s), slowing over the ground like a chute's, sinking
/// at `SKEET_SINK`, looking after `SKEET_ARM_SECS`.
const SKEET_THROW: f64 = 50.0;
const SKEET_TAU: f64 = 1.5;
const SKEET_SINK: f64 = 65.0;
const SKEET_ARM_SECS: f64 = 0.2;
/// Its sensors see the ground this far from straight below per foot of height (tan 30
/// degrees).
const SKEET_CONE: f64 = 0.577;
/// Height (ft) it destroys itself at when it found nothing.
const SKEET_SELF_DESTRUCT_FT: f64 = 50.0;
/// Odds (percent) its penetrator hits a vehicle it fired at, and that a hit destroys a tank
/// or another vehicle.
const SKEET_HIT: i32 = 90;
const SKEET_KILL_TANK: i32 = 80;
const SKEET_KILL_OTHER: i32 = 95;
/// How far (ft) from the point the submunitions settle over the targets a salvo looks at:
/// vehicles (they drive on while the chutes come down) and world objects.
const SALVO_UNITS_FT: f64 = 3500.0;
const SALVO_OBJECTS_FT: f64 = 2500.0;
/// World object categories a skeet takes for a vehicle: aircraft, launcher, radar, armour.
const VEHICLE_CATEGORIES: [i32; 4] = [2, 7, 8, 15];

/// Where a submunition is in its fall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Phase {
    /// Thrown out of the dispenser, its chute not out yet.
    #[default]
    Eject,
    Chute,
    /// Its rocket burning, throwing out the skeets.
    Rocket,
    /// Done: falling to the ground.
    Spent,
}

/// A BLU-108 submunition.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Blu108 {
    /// Drawn as the same one from frame to frame.
    pub id: u16,
    /// Position and velocity (ft, ft/s: east, south, up).
    pub p: [f64; 3],
    pub v: [f64; 3],
    pub phase: Phase,
    /// Seconds in the phase (the aft bay's start below 0, still in the dispenser).
    pub t: f64,
    /// Turned about its axis (rad), spun by the rocket.
    pub spin: f64,
    /// Skeets still on it.
    pub skeets: u8,
    /// The pilot who dropped it.
    pub owner: u8,
    pub salvo: u16,
}

/// A skeet.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Skeet {
    pub id: u16,
    pub p: [f64; 3],
    pub v: [f64; 3],
    /// Seconds since it left its submunition.
    pub t: f64,
    pub owner: u8,
    pub salvo: u16,
}

/// What a skeet fires at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aim {
    /// A ground unit (`Sim::units` index).
    Unit(usize),
    /// A world object.
    Object(usize),
}

/// The targets one dispenser's skeets look for, and those already fired at.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Salvo {
    pub id: u16,
    pub units: Vec<usize>,
    pub objects: Vec<usize>,
    pub engaged: Vec<Aim>,
}

/// Every CBU-105 opened and not done yet.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Sfw {
    pub blus: Vec<Blu108>,
    pub skeets: Vec<Skeet>,
    /// Only the host's simulation keeps them (not sent to the others).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub salvos: Vec<Salvo>,
    #[serde(default)]
    pub next_id: u16,
}

impl Sfw {
    pub fn is_empty(&self) -> bool {
        self.blus.is_empty() && self.skeets.is_empty() && self.salvos.is_empty()
    }

    fn id(&mut self) -> u16 {
        self.next_id = self.next_id.wrapping_add(1);
        self.next_id
    }

    /// What the other pilots' simulations draw: the submunitions and the skeets.
    pub fn for_clients(&self) -> Sfw {
        Sfw { salvos: Vec::new(), ..self.clone() }
    }
}

/// How far (ft) from the marked point the submunitions may settle for the HUD to show the
/// CBU-105 in range: within the pattern, its skeets see the point.
const IN_RANGE_FT: f64 = 300.0;

/// The HUD's CBU-105 data: the burst altitude (ft), the aircraft below it, and whether one
/// released now reaches the marked point (None: no point marked).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SfwSolution {
    pub burst_ft: u16,
    pub low: bool,
    pub in_range: Option<bool>,
}

/// How hard the dispenser's tail kit steers (g).
pub(crate) const SFW_PULL_G: f64 = 2.0;

/// The height above the ground (ft) CBU-105 `m` opens at.
pub(crate) fn burst_of(m: &Projectile) -> u16 {
    match m.burst_ft {
        0 => SFW_DEFAULT_BURST_FT,
        b => b,
    }
}

/// Fine units of a world-unit coordinate.
fn fine(a: f64) -> i32 {
    (a * ROUND_FINE as f64).round() as i32
}

/// How far submunitions thrown out `height` ft above the ground, sinking at `vz` ft/s, go on
/// over the ground until their rockets fire, as seconds at the dispenser's speed (their eject
/// and chute phases at `dt` s a step, as [`Sim::update_sfw`] flies them).
pub fn drift_secs(height: f64, vz: f64, dt: f64) -> f64 {
    let g = ballistics::GRAVITY;
    let (mut h, mut vz, mut t, mut speed, mut secs) = (height, vz, 0.0, 1.0, 0.0);
    while t < EJECT_SECS {
        vz -= g * dt;
        h += vz * dt;
        secs += speed * dt;
        t += dt;
    }
    let (k, ks) = ((-dt / CHUTE_TAU).exp(), 1.0 - (-dt / CHUTE_TAU_SINK).exp());
    while h > ROCKET_FT && t < 600.0 {
        speed *= k;
        vz += (-CHUTE_SINK - vz) * ks;
        h += vz * dt;
        secs += speed * dt;
        t += dt;
    }
    secs
}

impl Sim {
    /// The CBU-105s' submunitions and skeets on their way down.
    pub fn sfw(&self) -> &Sfw {
        &self.sfw
    }

    /// With a CBU-105 selected: the HUD's data for a release now at the last report's burst
    /// altitude, flying the dispenser's fall ahead in coarse steps to where it would open.
    pub fn sfw_solution(&self) -> Option<SfwSolution> {
        if self.selected_spec() != CBU105_SPEC || self.gun_selected() {
            return None;
        }
        let ps = &self.pl.last_input;
        let burst = match ps.burst_ft {
            0 => SFW_DEFAULT_BURST_FT,
            b => b,
        };
        let low = self.height_above_ground() < burst as i32;
        let aim = self.ccrp_aim_of(ps);
        let in_range = aim.map(|(x, y)| {
            if low {
                return false;
            }
            let m = Projectile {
                aim: Some(((x * 32.0) as i32, (y * 32.0) as i32)),
                burst_ft: burst,
                ..Default::default()
            };
            let (ax, ay) = (x * 32.0, y * 32.0);
            let mut p = [
                self.pl.p.wx as f64,
                (0x100000 - self.pl.p.wy) as f64,
                self.pl.p.alt as f64 - ballistics::RELEASE_DROP,
            ];
            let mut v = ballistics::release_velocity(
                self.pl.p.heading,
                self.pl.p.velocity,
                self.pl.p.climb,
            );
            let k = ballistics::drag(30, true).unwrap_or(0.0);
            let dt = 0.25;
            for _ in 0..600 {
                self.steer_sfw(&m, &p, &mut v, dt);
                ballistics::step(&mut p, &mut v, k, dt);
                let h = p[2] - self.sfw_ground(p[0], p[1]);
                if h <= burst as f64 {
                    let drift = drift_secs(h, v[2], dt);
                    let c = [p[0] + v[0] * drift, p[1] + v[1] * drift];
                    return (c[0] - ax).hypot(c[1] - ay) < IN_RANGE_FT;
                }
            }
            false
        });
        Some(SfwSolution { burst_ft: burst, low, in_range })
    }

    /// The ground under a point (world units, ft).
    fn sfw_ground(&self, x: f64, y: f64) -> f64 {
        self.hill_alt((x / 32.0) as i32, (y / 32.0) as i32) as f64
    }

    /// The active pilot's CBU-105 in slot `s` opens at `p` flying at `v` (ft, ft/s): its ten
    /// submunitions go out, laid out to settle in their pattern.
    pub(crate) fn sfw_dispense(&mut self, s: usize, p: [f64; 3], v: [f64; 3]) {
        let burst = burst_of(&self.proj[s]);
        self.proj[s].ttl = 0;
        self.emit(SimEvent::SfwDispense { x: fine(p[0]), y: fine(p[1]), alt: fine(p[2]) });
        self.sound(0x0D, 1);
        // Along the path and across it (right).
        let vh = v[0].hypot(v[1]).max(1.0);
        let along = [v[0] / vh, v[1] / vh];
        let right = [-along[1], along[0]];
        // A low burst leaves them less time to spread.
        let scale = (burst as f64 / SFW_DEFAULT_BURST_FT as f64).clamp(0.5, 1.0);
        let dt = 1.0 / self.fs.max(1) as f64;
        let drift = drift_secs(p[2] - self.sfw_ground(p[0], p[1]), v[2], dt);
        let centre = [p[0] + v[0] * drift, p[1] + v[1] * drift];
        let salvo = self.sfw.id();
        let owner = self.pl.id;
        for i in 0..10 {
            // The forward bay's five, then the aft bay's.
            let aft = i >= 5;
            let col = (i % 5) as f64 - 2.0;
            let row = if (i % 5) % 2 == 0 { -1.0 } else { 1.0 } * if aft { -1.0 } else { 1.0 };
            let jitter = [
                (self.rng.random(100) - 50) as f64 * 0.3,
                (self.rng.random(100) - 50) as f64 * 0.3,
            ];
            let off_along = (col * PATTERN_ALONG + jitter[0]) * scale;
            let off_right = (row * PATTERN_ACROSS + jitter[1]) * scale;
            let kick = |k: usize| (along[k] * off_along + right[k] * off_right) / drift;
            let id = self.sfw.id();
            let delay = if aft { AFT_BAY_SECS } else { 0.0 };
            self.sfw.blus.push(Blu108 {
                id,
                p: [p[0] - v[0] * delay, p[1] - v[1] * delay, p[2] - v[2] * delay],
                v: [v[0] + kick(0), v[1] + kick(1), v[2] - 10.0],
                phase: Phase::Eject,
                t: -delay,
                spin: self.rng.random(628) as f64 / 100.0,
                skeets: 4,
                owner,
                salvo,
            });
        }
        // What the skeets may find around where the submunitions come down.
        let units = (0..self.units.len())
            .filter(|&k| {
                let (ux, uy) = self.units[k].map_xy();
                (ux * 32.0 - centre[0]).hypot(uy * 32.0 - centre[1]) < SALVO_UNITS_FT
            })
            .collect();
        let objects = (1..self.objects.len())
            .filter(|&k| {
                let o = self.objects[k];
                let (ox, oy) = (o.x as f64 * 32.0 + 16.0, o.y as f64 * 32.0 + 16.0);
                (ox - centre[0]).hypot(oy - centre[1]) < SALVO_OBJECTS_FT
                    && VEHICLE_CATEGORIES.contains(&self.category(k as i32))
                    && self.compat(CBU105_WEAPON, k as i32) > 0
            })
            .collect();
        self.sfw.salvos.push(Salvo { id: salvo, units, objects, engaged: Vec::new() });
    }

    /// A frame of the submunitions and skeets on their way down (the host's; a mirror draws
    /// what it is sent).
    pub(crate) fn update_sfw(&mut self) {
        if self.sfw.is_empty() || self.mirror {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let g = ballistics::GRAVITY;
        let mut blus = std::mem::take(&mut self.sfw.blus);
        let mut thrown = Vec::new();
        blus.retain_mut(|b| {
            b.t += dt;
            if b.t < 0.0 {
                // Still in the aft bay: carried along.
                for k in 0..3 {
                    b.p[k] += b.v[k] * dt;
                }
                return true;
            }
            let ground = self.sfw_ground(b.p[0], b.p[1]);
            match b.phase {
                Phase::Eject => {
                    b.v[2] -= g * dt;
                    if b.t >= EJECT_SECS {
                        b.phase = Phase::Chute;
                        b.t = 0.0;
                    }
                }
                Phase::Chute => {
                    let k = (-dt / CHUTE_TAU).exp();
                    b.v[0] *= k;
                    b.v[1] *= k;
                    b.v[2] += (-CHUTE_SINK - b.v[2]) * (1.0 - (-dt / CHUTE_TAU_SINK).exp());
                    if b.p[2] - ground <= ROCKET_FT {
                        b.phase = Phase::Rocket;
                        b.t = 0.0;
                        b.v = [b.v[0] * 0.5, b.v[1] * 0.5, 0.0];
                    }
                }
                Phase::Rocket => {
                    // The rocket holds it up and spins it.
                    b.v[2] += (5.0 - b.v[2]) * (1.0 - (-dt / 0.2f64).exp());
                    b.spin = (b.spin + 40.0 * dt) % std::f64::consts::TAU;
                    for (n, &at) in SKEET_PAIRS.iter().enumerate() {
                        if b.t >= at && b.skeets as usize > 2 * (1 - n) {
                            b.skeets -= 2;
                            let a = b.spin + n as f64 * std::f64::consts::FRAC_PI_2;
                            for a in [a, a + std::f64::consts::PI] {
                                let (s, c) = a.sin_cos();
                                thrown.push(Skeet {
                                    id: 0,
                                    p: b.p,
                                    v: [b.v[0] + s * SKEET_THROW, b.v[1] + c * SKEET_THROW, -5.0],
                                    t: 0.0,
                                    owner: b.owner,
                                    salvo: b.salvo,
                                });
                            }
                        }
                    }
                    if b.t >= ROCKET_SECS {
                        b.phase = Phase::Spent;
                        b.t = 0.0;
                    }
                }
                Phase::Spent => {
                    let k = g / (SPENT_TERMINAL * SPENT_TERMINAL);
                    ballistics::step(&mut b.p, &mut b.v, k, dt);
                    return b.p[2] > ground;
                }
            }
            for k in 0..3 {
                b.p[k] += b.v[k] * dt;
            }
            // One that came down before its rocket fired (a burst too low) is a dud.
            b.p[2] > ground
        });
        self.sfw.blus = blus;
        for mut s in thrown {
            s.id = self.sfw.id();
            self.sfw.skeets.push(s);
        }
        let mut skeets = std::mem::take(&mut self.sfw.skeets);
        skeets.retain_mut(|s| {
            s.t += dt;
            let k = (-dt / SKEET_TAU).exp();
            s.v[0] *= k;
            s.v[1] *= k;
            s.v[2] += (-SKEET_SINK - s.v[2]) * (1.0 - (-dt / 0.5f64).exp());
            for k in 0..3 {
                s.p[k] += s.v[k] * dt;
            }
            let height = s.p[2] - self.sfw_ground(s.p[0], s.p[1]);
            if s.t >= SKEET_ARM_SECS
                && let Some((aim, to)) = self.skeet_finds(s, height)
            {
                self.skeet_fires(s, aim, to);
                return false;
            }
            if height <= SKEET_SELF_DESTRUCT_FT {
                self.emit(SimEvent::SkeetBurst {
                    x: fine(s.p[0]),
                    y: fine(s.p[1]),
                    alt: fine(s.p[2]),
                });
                return false;
            }
            true
        });
        self.sfw.skeets = skeets;
        let live: Vec<u16> = self
            .sfw
            .blus
            .iter()
            .map(|b| b.salvo)
            .chain(self.sfw.skeets.iter().map(|s| s.salvo))
            .collect();
        self.sfw.salvos.retain(|v| live.contains(&v.id));
    }

    /// What skeet `s`, `height` ft up, sees in its cone, and where (ft): the vehicles first,
    /// one no other skeet of its salvo fired at before one that was, the nearest to straight
    /// below first.
    fn skeet_finds(&self, s: &Skeet, height: f64) -> Option<(Aim, [f64; 3])> {
        let salvo = self.sfw.salvos.iter().find(|v| v.id == s.salvo)?;
        let reach = height.max(0.0) * SKEET_CONE;
        let below = |x: f64, y: f64| (x - s.p[0]).hypot(y - s.p[1]);
        let units = salvo.units.iter().filter_map(|&k| {
            let u = self.units.get(k)?;
            if u.destroyed
                || u.gone
                || matches!(u.kind, GroundKind::Manpads | GroundKind::DroneCenter)
            {
                return None;
            }
            let (ux, uy) = u.map_xy();
            let (x, y) = (ux * 32.0, uy * 32.0);
            Some((Aim::Unit(k), [x, y, self.sfw_ground(x, y) + 6.0]))
        });
        let objects = salvo.objects.iter().filter_map(|&k| {
            let o = self.objects.get(k)?;
            if o.flags & oflags::DESTROYED != 0 {
                return None;
            }
            let (x, y) = (o.x as f64 * 32.0 + 16.0, o.y as f64 * 32.0 + 16.0);
            Some((Aim::Object(k), [x, y, self.sfw_ground(x, y) + 6.0]))
        });
        let seen: Vec<(Aim, [f64; 3])> =
            units.chain(objects).filter(|(_, p)| below(p[0], p[1]) <= reach).collect();
        let rank = |a: &(Aim, [f64; 3])| {
            (
                matches!(a.0, Aim::Object(_)),
                salvo.engaged.contains(&a.0),
                (below(a.1[0], a.1[1]) * 16.0) as i64,
            )
        };
        seen.into_iter().min_by_key(rank)
    }

    /// Skeet `s` fires its penetrator at `aim` at `to`.
    fn skeet_fires(&mut self, s: &Skeet, aim: Aim, to: [f64; 3]) {
        if let Some(v) = self.sfw.salvos.iter_mut().find(|v| v.id == s.salvo) {
            v.engaged.push(aim);
        }
        let hit = self.rng.random(100) < SKEET_HIT;
        let odds = match aim {
            Aim::Unit(k) if self.units[k].kind.tank() => SKEET_KILL_TANK,
            Aim::Unit(_) => SKEET_KILL_OTHER,
            Aim::Object(k) => self.compat(CBU105_WEAPON, k as i32) * 12,
        };
        let kill = hit && self.rng.random(100) < odds;
        // The kill is the pilot's who dropped it (his message, his score).
        let owner = s.owner as usize;
        if kill {
            self.activate(owner);
        }
        self.credit = Some(s.owner);
        let at = |p: [f64; 3]| (fine(p[0]), fine(p[1]), fine(p[2]));
        self.emit(SimEvent::SkeetFire { from: at(s.p), to: at(to), hit, kill });
        if kill {
            match aim {
                Aim::Unit(k) => {
                    let u = self.units[k];
                    self.units[k].destroyed = true;
                    self.units[k].burst = 0;
                    let blue = self.aaa_blue(k);
                    self.msg(text!("sim-destroyed", target = u.kind.name_on(blue)));
                    self.emit(SimEvent::AaaDestroyed { unit: k, by: Some(s.owner) });
                }
                Aim::Object(k) => {
                    let long = self.weapon_types
                        [(CBU105_WEAPON as usize).min(self.weapon_types.len() - 1)]
                    .long_name
                    .clone();
                    self.destroy_ground(k as i32);
                    let place = self.place_name(k);
                    self.msg(text!("sim-destroyed-by", target = place, weapon = long));
                }
            }
        }
        self.credit = None;
    }
}
