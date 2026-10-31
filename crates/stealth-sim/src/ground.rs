//! Campaign armour: convoys driving their route to an enemy place, and the ground units that
//! see and fight each other (tanks, infantry fighting vehicles and the convoys' air defence
//! guns against whatever of the enemy's stands on the ground).
//!
//! A convoy's vehicles are ordinary [`GroundUnit`]s with a fine position ([`Fine`]); the air
//! defence guns among them keep firing at aircraft as anywhere else. Positions are metres
//! (east, north). A column halts while any of its vehicles can fire at an enemy it sees
//! within [`HALT_RANGE`]; its vehicles keep their place in it by the distance driven along
//! the route, so a wreck leaves its gap.

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{Fine, GroundKind, GroundUnit, M_PER_MAP};
use crate::rng::Rng;
use crate::types::SimEvent;

/// Road march speed (m/s) and cross-country speed in the assault.
pub const CONVOY_SPEED: f64 = 20.0;
const ASSAULT_SPEED: f64 = 6.0;
/// Metres between the column's vehicles.
pub const GAP: f64 = 60.0;
/// The convoy leaves the road this far (m) before its goal and attacks in line abreast, its
/// vehicles this far apart, stopping this short of the place's centre.
const ASSAULT_START: f64 = 2000.0;
const ASSAULT_SPACING: f64 = 90.0;
const ASSAULT_STOP: f64 = 400.0;
/// A ground unit sees enemy ground units this far (m), the line of sight over the relief
/// sampled this often (m) and clear by this much (m).
pub const SIGHT_RANGE: f64 = 4000.0;
/// A column drives on (its tanks firing as it goes) until an enemy it can hit is this close
/// (m): close enough that the infantry fighting vehicles at the back of the column (300 m
/// behind its head) reach the enemy's head with their autocannons too.
const HALT_RANGE: f64 = 1500.0;
/// A gun's chance to hit fired on the move, and at a target on the move (factors).
const MOVING_SHOOTER: f64 = 0.5;
const MOVING_TARGET: f64 = 0.8;
const LOS_STEP: f64 = 250.0;
const LOS_CLEAR: f64 = 10.0;
/// The most ground units in a campaign.
pub(crate) const MAX_GROUND: usize = 600;

/// What a convoy does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConvoyMode {
    /// Driving along the route.
    #[default]
    March,
    /// Stopped to fight an enemy in sight.
    Halt,
    /// Off the road, closing on the place in line abreast.
    Assault,
}

/// A column of vehicles on its way to an enemy place.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Convoy {
    /// The campaign's id for it.
    pub tag: u32,
    pub blue: bool,
    /// The way to drive (metres east, north), from the start to the goal.
    pub route: Vec<[f64; 2]>,
    /// How far (m) along the route the head of the column is.
    pub along: f64,
    /// Its vehicles in column order (`Sim::units` indices).
    pub units: Vec<u16>,
    pub mode: ConvoyMode,
}

impl Convoy {
    /// Point (metres east, north) and heading at distance `s` along its route.
    pub fn point(&self, s: f64) -> ([f64; 2], f64) {
        point_at(&self.route, s)
    }

    fn length(&self) -> f64 {
        self.route.windows(2).map(|w| dist(w[0], w[1])).sum()
    }
}

/// The campaign's ground war in the simulation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GroundState {
    pub convoys: Vec<Convoy>,
    /// Per unit: the convoy it came with (0: none).
    pub convoy_of: Vec<u32>,
    /// Per unit: the friendly (blue) and the enemy (red) side see it.
    pub seen_blue: Vec<bool>,
    pub seen_red: Vec<bool>,
    /// Per unit: frames until its gun may fire at a ground target again.
    pub reload: Vec<i32>,
    /// Per unit: rounds left of the burst it is firing, and at whom.
    #[serde(default)]
    pub burst: Vec<u8>,
    #[serde(default)]
    pub target: Vec<u16>,
    /// Per unit: the damage of the hits it took (percent; 100 or more: knocked out, or about
    /// to be as the round lands).
    #[serde(default)]
    pub damage: Vec<u8>,
    /// The rounds on their way that knock out a vehicle as they land: unit, frames to go.
    #[serde(default)]
    pub landing: Vec<(u16, u16)>,
    pub rng: Option<Rng>,
    /// Road march speed (m/s), 0: [`CONVOY_SPEED`] (a test aid).
    pub speed: f32,
}

impl GroundState {
    /// What the clients are sent of it: who sees whom, the convoys' routes (they drive with
    /// the moving world), without the guns' and the damage's bookkeeping.
    pub(crate) fn for_clients(&self) -> GroundState {
        let mut g = GroundState {
            convoys: self.convoys.clone(),
            convoy_of: self.convoy_of.clone(),
            seen_blue: self.seen_blue.clone(),
            seen_red: self.seen_red.clone(),
            speed: self.speed,
            ..Default::default()
        };
        for v in &mut g.convoys {
            v.along = 0.0;
            v.mode = Default::default();
        }
        g
    }
}

/// A ground vehicle's gun firing at the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundGun {
    /// A tank's main gun (120 or 125 mm).
    Tank,
    /// An infantry fighting vehicle's autocannon (25 or 30 mm).
    Cannon,
    /// An air defence gun's (23 or 35 mm).
    Flak,
}

impl GroundGun {
    /// Muzzle velocity (m/s; the rounds fly flat over these ranges).
    pub fn speed(self) -> f64 {
        match self {
            GroundGun::Tank => 1650.0,
            GroundGun::Cannon => 1100.0,
            GroundGun::Flak => 1000.0,
        }
    }

    /// Seconds a round flies `d` metres.
    pub fn flight_time(self, d: f64) -> f64 {
        d / self.speed()
    }
}

/// A ground gun's fire at another ground unit.
struct Shot {
    gun: GroundGun,
    /// Reach (m).
    reach: f64,
    /// Rounds a burst (fewest, most), seconds between its rounds, seconds between bursts.
    rounds: (i32, i32),
    interval: f64,
    reload: f64,
    /// Chance a round hits at point blank and at full reach.
    hit: (f64, f64),
    /// Damage a hit does (percent, least and most), and the chance that it knocks the
    /// vehicle out at once (a tank round into another tank).
    damage: (i32, i32),
    knockout: f64,
}

/// How unit kind `a` fights ground unit kind `b`, if it does. A tank round knocks out an
/// infantry fighting vehicle with a hit and another tank with one or two; it takes about a
/// dozen autocannon hits on an infantry fighting vehicle and 20 to 30 on a tank.
fn shot(a: GroundKind, b: GroundKind) -> Option<Shot> {
    let light = matches!(b, GroundKind::Zsu | GroundKind::Gepard);
    if a.tank() {
        // The T-90's autoloader is a little slower than the Leopard's loader.
        let reload = if a == GroundKind::T90 { 7.5 } else { 6.0 };
        let (damage, knockout) = if b.tank() { ((55, 75), 0.35) } else { ((100, 100), 0.0) };
        return Some(Shot {
            gun: GroundGun::Tank,
            reach: 3000.0,
            rounds: (1, 1),
            interval: 0.0,
            reload,
            hit: (0.9, 0.5),
            damage,
            knockout,
        });
    }
    if a.carrier() {
        // Bursts of 3 to 5 at the low rate (M242 Bushmaster, 2A42).
        let damage = if b.tank() {
            (3, 5)
        } else if b.carrier() {
            (6, 11)
        } else if light {
            (20, 35)
        } else {
            (35, 60)
        };
        return Some(Shot {
            gun: GroundGun::Cannon,
            reach: 2000.0,
            rounds: (3, 5),
            interval: if a == GroundKind::Bradley { 0.3 } else { 0.25 },
            reload: 3.0,
            hit: (0.6, 0.2),
            damage,
            knockout: 0.0,
        });
    }
    if matches!(a, GroundKind::Zsu | GroundKind::Gepard) && !b.tank() {
        let damage = if b.carrier() {
            (4, 8)
        } else if light {
            (15, 30)
        } else {
            (30, 50)
        };
        return Some(Shot {
            gun: GroundGun::Flak,
            reach: 1500.0,
            rounds: (4, 8),
            interval: if a == GroundKind::Zsu { 0.1 } else { 0.12 },
            reload: 3.5,
            hit: (0.5, 0.15),
            damage,
            knockout: 0.0,
        });
    }
    None
}

/// The damage (percent) of one of `s`'s hits.
fn hit_damage(s: &Shot, rng: &mut Rng) -> i32 {
    if s.knockout > 0.0 && (rng.random(1000) as f64) < s.knockout * 1000.0 {
        return 100;
    }
    s.damage.0 + rng.random(s.damage.1 - s.damage.0 + 1)
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Point and heading (rad, clockwise from north) at distance `s` along a route.
fn point_at(route: &[[f64; 2]], s: f64) -> ([f64; 2], f64) {
    let mut left = s.max(0.0);
    for w in route.windows(2) {
        let d = dist(w[0], w[1]);
        let heading = (w[1][0] - w[0][0]).atan2(w[1][1] - w[0][1]);
        if left <= d {
            let k = if d > 0.0 { left / d } else { 0.0 };
            return (
                [w[0][0] + (w[1][0] - w[0][0]) * k, w[0][1] + (w[1][1] - w[0][1]) * k],
                heading,
            );
        }
        left -= d;
    }
    match route {
        [.., a, b] => (*b, (b[0] - a[0]).atan2(b[1] - a[1])),
        [a] => (*a, 0.0),
        [] => ([0.0; 2], 0.0),
    }
}

/// Map units (x east, y south) of a point in metres.
fn map_of(p: [f64; 2]) -> (i32, i32) {
    ((p[0] / M_PER_MAP).round() as i32, 0x8000 - (p[1] / M_PER_MAP).round() as i32)
}

/// Metres (east, north) of a map point.
pub fn metres_of(x: i32, y: i32) -> [f64; 2] {
    [x as f64 * M_PER_MAP, (0x8000 - y) as f64 * M_PER_MAP]
}

fn place(u: &mut GroundUnit, p: [f64; 2], heading: f64) {
    let (x, y) = map_of(p);
    u.x = x.clamp(0x40, 0x7FC0);
    u.y = y.clamp(0x40, 0x7FC0);
    u.fine = Some(Fine { e: p[0] as f32, n: p[1] as f32, heading: heading as f32 });
}

impl Sim {
    fn ground(&self) -> Option<&GroundState> {
        self.campaign.as_ref().map(|c| &c.ground)
    }

    /// The campaign's convoys on the move.
    pub fn convoys(&self) -> &[Convoy] {
        self.ground().map_or(&[], |g| &g.convoys)
    }

    /// The convoy unit `k` came with (0: none).
    pub fn unit_convoy(&self, k: usize) -> u32 {
        self.ground().and_then(|g| g.convoy_of.get(k).copied()).unwrap_or(0)
    }

    /// Unit `k` is in a convoy on the move.
    pub fn unit_moving(&self, k: usize) -> bool {
        let tag = self.unit_convoy(k);
        tag != 0 && self.convoys().iter().any(|c| c.tag == tag)
    }

    /// The side (`blue` or not) sees unit `k` now: one of its ground units or drones has it in
    /// sight. A side always knows its own.
    pub fn unit_seen_by(&self, k: usize, blue: bool) -> bool {
        if self.aaa_blue(k) == blue {
            return true;
        }
        self.ground().is_some_and(|g| {
            let seen = if blue { &g.seen_blue } else { &g.seen_red };
            seen.get(k).copied().unwrap_or(false)
        })
    }

    /// Unit `k` is on the move, or firing or reloading after a shot at a ground target (its
    /// turret matters).
    pub(crate) fn ground_busy(&self, k: usize) -> bool {
        self.unit_moving(k)
            || self.ground().is_some_and(|g| {
                g.reload.get(k).is_some_and(|&r| r > self.fs)
                    || g.burst.get(k).is_some_and(|&b| b > 0)
            })
    }

    /// The convoys' road march speed (m/s).
    pub fn convoy_speed(&self) -> f64 {
        self.ground().filter(|g| g.speed > 0.0).map_or(CONVOY_SPEED, |g| g.speed as f64)
    }

    /// Test aid: the convoys' road march speed (m/s).
    pub fn set_convoy_speed(&mut self, speed: f32) {
        if let Some(c) = self.campaign.as_mut() {
            c.ground.speed = speed;
        }
    }

    /// Campaign: a convoy of `kinds` sets off along `route` (map units) toward the enemy place
    /// at its end, its column strung out behind the first point.
    pub fn spawn_convoy(
        &mut self,
        tag: u32,
        blue: bool,
        kinds: &[GroundKind],
        route: &[(i32, i32)],
    ) {
        let Some(mut c) = self.campaign.take() else { return };
        let route: Vec<[f64; 2]> = route.iter().map(|&(x, y)| metres_of(x, y)).collect();
        let n = self.units.len();
        c.aaa_blue.resize(n, false);
        c.aaa_target.resize(n, None);
        c.ground.convoy_of.resize(n, 0);
        c.ground.convoys.retain(|v| v.tag != tag);
        if route.len() >= 2 && !kinds.is_empty() {
            let active: Vec<u32> = c.ground.convoys.iter().map(|v| v.tag).collect();
            // The wrecks of convoys that are over make room.
            let mut free: Vec<usize> = (0..n)
                .filter(|&k| {
                    let of = c.ground.convoy_of[k];
                    self.units[k].destroyed
                        && of != 0
                        && !active.contains(&of)
                        && !c.defence.stations.iter().any(|s| s.unit as usize == k)
                })
                .rev()
                .collect();
            let mut units = Vec::new();
            let along = (kinds.len() - 1) as f64 * GAP;
            for (i, &kind) in kinds.iter().enumerate() {
                let (p, heading) = point_at(&route, along - i as f64 * GAP);
                let mut u = GroundUnit {
                    kind,
                    aim_az: heading as f32,
                    aim_el: 0.05,
                    rounds: if kind == GroundKind::Gepard {
                        crate::station::GEPARD_ROUNDS
                    } else {
                        0
                    },
                    ..Default::default()
                };
                place(&mut u, p, heading);
                let k = match free.pop() {
                    Some(k) => k,
                    None if self.units.len() < MAX_GROUND => {
                        self.units.push(u);
                        c.aaa_blue.push(blue);
                        c.aaa_target.push(None);
                        c.ground.convoy_of.push(0);
                        self.units.len() - 1
                    }
                    None => continue,
                };
                self.units[k] = u;
                c.aaa_blue[k] = blue;
                c.aaa_target[k] = None;
                c.ground.convoy_of[k] = tag;
                if let Some(s) = c.defence.stock.get_mut(k) {
                    *s = crate::station::initial_stock(kind);
                }
                if let Some(r) = c.defence.refill.get_mut(k) {
                    *r = 0.0;
                }
                units.push(k as u16);
            }
            c.ground.convoys.push(Convoy {
                tag,
                blue,
                route,
                along,
                units,
                mode: ConvoyMode::March,
            });
        }
        self.campaign = Some(c);
        if self.aaa_rng.is_none() {
            self.aaa_rng = Some(Rng::new(self.rng.hold ^ 0x5EAD_AAA0));
        }
    }

    /// Campaign: convoy `tag` is over (it took its place, or the campaign called it off); its
    /// vehicles stay where they are.
    pub fn end_convoy(&mut self, tag: u32) {
        if let Some(c) = self.campaign.as_mut() {
            c.ground.convoys.retain(|v| v.tag != tag);
        }
    }

    /// Where the head of convoy `tag` is (map units): its first vehicle still running.
    pub fn convoy_head(&self, tag: u32) -> Option<(i32, i32)> {
        let v = self.convoys().iter().find(|v| v.tag == tag)?;
        let k = *v.units.iter().find(|&&k| !self.units[k as usize].destroyed)?;
        Some((self.units[k as usize].x, self.units[k as usize].y))
    }

    pub(crate) fn update_ground(&mut self) {
        let Some(mut c) = self.campaign.take() else { return };
        let n = self.units.len();
        let g = &mut c.ground;
        g.convoy_of.resize(n, 0);
        g.seen_blue.resize(n, false);
        g.seen_red.resize(n, false);
        g.reload.resize(n, 0);
        g.burst.resize(n, 0);
        g.target.resize(n, 0);
        g.damage.resize(n, 0);
        // A wreck's damage is done with (a unit put back in its place starts whole).
        for (d, u) in g.damage.iter_mut().zip(&self.units) {
            if u.destroyed {
                *d = 0;
            }
        }
        let fs = self.fs.max(1);
        let look = self.frame % fs == fs / 2;
        let mut convoys = std::mem::take(&mut g.convoys);
        let mut rng = g.rng.unwrap_or_else(|| Rng::new(self.rng.hold ^ 0x6A0D_7A2C));
        let speed = if g.speed > 0.0 { g.speed as f64 } else { CONVOY_SPEED };
        self.campaign = Some(c);
        if look {
            self.ground_look();
        }
        let dt = 1.0 / fs as f64;
        for v in &mut convoys {
            self.drive(v, speed, dt, look);
        }
        // The vehicles on the move (their convoy not halted) fire and are hit less well.
        let driving: Vec<u32> =
            convoys.iter().filter(|v| v.mode != ConvoyMode::Halt).map(|v| v.tag).collect();
        self.ground_fire(&mut rng, &driving);
        if let Some(c) = self.campaign.as_mut() {
            c.ground.convoys = convoys;
            c.ground.rng = Some(rng);
        }
    }

    /// Who sees whom (once a second): a side sees an enemy ground unit while one of its live
    /// ground units has it in sight within [`SIGHT_RANGE`], or one of its drones within its
    /// sensor's reach.
    fn ground_look(&mut self) {
        let n = self.units.len();
        let pos: Vec<[f64; 2]> = self
            .units
            .iter()
            .map(|u| {
                let m = u.metres();
                [m[0], m[1]]
            })
            .collect();
        let blue: Vec<bool> = (0..n).map(|k| self.aaa_blue(k)).collect();
        let drones: Vec<([f64; 2], f64)> =
            self.uavs().iter().map(|u| ([u.pos[0], u.pos[1]], u.kind.sensor())).collect();
        let mut seen_blue = vec![false; n];
        let mut seen_red = vec![false; n];
        for j in 0..n {
            if self.units[j].destroyed {
                continue;
            }
            let by_blue = !blue[j];
            if by_blue && drones.iter().any(|&(p, r)| dist(p, pos[j]) < r) {
                seen_blue[j] = true;
                continue;
            }
            let seen = (0..n).any(|i| {
                blue[i] == by_blue
                    && !self.units[i].destroyed
                    && dist(pos[i], pos[j]) < SIGHT_RANGE
                    && self.ground_los(pos[i], pos[j])
            });
            if by_blue {
                seen_blue[j] = seen;
            } else {
                seen_red[j] = seen;
            }
        }
        if let Some(c) = self.campaign.as_mut() {
            c.ground.seen_blue = seen_blue;
            c.ground.seen_red = seen_red;
        }
    }

    /// The relief leaves a line of sight from `a` to `b` (metres) open.
    fn ground_los(&self, a: [f64; 2], b: [f64; 2]) -> bool {
        let Some(r) = self.tiles.as_ref().map(|t| &t.relief) else { return true };
        let h = |p: [f64; 2]| r.smooth_m(p[0] / M_PER_MAP, 0x8000 as f64 - p[1] / M_PER_MAP) as f64;
        let (ha, hb) = (h(a) + 3.0, h(b) + 2.0);
        let steps = (dist(a, b) / LOS_STEP) as usize;
        (1..steps).all(|i| {
            let t = i as f64 / steps as f64;
            let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            h(p) <= ha + (hb - ha) * t + LOS_CLEAR
        })
    }

    /// Unit `k` has an enemy ground unit in sight that it can fire at.
    fn in_contact(&self, k: usize) -> bool {
        let a = &self.units[k];
        let m = a.metres();
        let blue = self.aaa_blue(k);
        (0..self.units.len()).any(|j| {
            let b = &self.units[j];
            let mb = b.metres();
            !b.destroyed
                && self.aaa_blue(j) != blue
                && shot(a.kind, b.kind)
                    .is_some_and(|s| dist([m[0], m[1]], [mb[0], mb[1]]) < s.reach.min(HALT_RANGE))
                && self.unit_seen_by(j, blue)
        })
    }

    /// One frame of a convoy's driving.
    fn drive(&mut self, v: &mut Convoy, speed: f64, dt: f64, look: bool) {
        let alive: Vec<usize> =
            v.units.iter().map(|&k| k as usize).filter(|&k| !self.units[k].destroyed).collect();
        if alive.is_empty() {
            return;
        }
        let length = v.length();
        if look && v.mode != ConvoyMode::Assault {
            let contact = alive.iter().any(|&k| self.in_contact(k));
            v.mode = if contact { ConvoyMode::Halt } else { ConvoyMode::March };
            if v.along >= length - ASSAULT_START {
                v.mode = ConvoyMode::Assault;
            }
        }
        match v.mode {
            ConvoyMode::Halt => {}
            ConvoyMode::March => {
                v.along = (v.along + speed * dt).min(length);
                for (i, &k) in v.units.iter().enumerate() {
                    let k = k as usize;
                    if self.units[k].destroyed {
                        continue;
                    }
                    let (p, heading) = point_at(&v.route, v.along - i as f64 * GAP);
                    let u = &mut self.units[k];
                    place(u, p, heading);
                    if u.kind.armour() {
                        u.aim_az = heading as f32;
                    }
                }
            }
            ConvoyMode::Assault => {
                // Line abreast across the way in, short of the place's centre.
                let goal = *v.route.last().unwrap();
                let (from, _) = point_at(&v.route, length - ASSAULT_START);
                let d = dist(from, goal).max(1.0);
                let dir = [(goal[0] - from[0]) / d, (goal[1] - from[1]) / d];
                let across = [dir[1], -dir[0]];
                let mid = (v.units.len() as f64 - 1.0) / 2.0;
                for (i, &k) in v.units.iter().enumerate() {
                    let k = k as usize;
                    if self.units[k].destroyed {
                        continue;
                    }
                    let off = (i as f64 - mid) * ASSAULT_SPACING;
                    let slot = [
                        goal[0] - dir[0] * ASSAULT_STOP + across[0] * off,
                        goal[1] - dir[1] * ASSAULT_STOP + across[1] * off,
                    ];
                    let m = self.units[k].metres();
                    let at = [m[0], m[1]];
                    let left = dist(at, slot);
                    let step = ASSAULT_SPEED * dt;
                    if left > step {
                        let heading = (slot[0] - at[0]).atan2(slot[1] - at[1]);
                        let p = [
                            at[0] + (slot[0] - at[0]) / left * step,
                            at[1] + (slot[1] - at[1]) / left * step,
                        ];
                        place(&mut self.units[k], p, heading);
                    }
                }
            }
        }
    }

    /// The ground guns fire at the enemy's ground units they see, a round at a time (bursts
    /// for the autocannons); the rounds that knock a vehicle out do so as they land.
    fn ground_fire(&mut self, rng: &mut Rng, driving: &[u32]) {
        let on_move = |sim: &Sim, k: usize| {
            let tag = sim.ground().and_then(|g| g.convoy_of.get(k).copied()).unwrap_or(0);
            tag != 0 && driving.contains(&tag)
        };
        let fs = self.fs.max(1) as f64;
        let n = self.units.len();
        let credit = self.credit.take();
        self.ground_landings();
        for i in 0..n {
            let a = self.units[i];
            let Some(c) = self.campaign.as_mut() else { break };
            let g = &mut c.ground;
            if a.destroyed
                || !(a.kind.armour() || matches!(a.kind, GroundKind::Zsu | GroundKind::Gepard))
            {
                g.burst[i] = 0;
                continue;
            }
            if g.reload[i] > 0 {
                g.reload[i] -= 1;
                continue;
            }
            // An air defence gun busy with an aircraft has no time for the ground.
            let busy = c.aaa_target.get(i).copied().flatten().is_some() || a.burst > 0;
            if busy || self.manned_by(i).is_some() {
                if let Some(c) = self.campaign.as_mut() {
                    c.ground.burst[i] = 0;
                }
                continue;
            }
            let m = a.metres();
            let me = [m[0], m[1]];
            // A burst keeps on at its target while it can.
            let mut aim = None;
            if let Some(c) = self.campaign.as_ref()
                && c.ground.burst[i] > 0
            {
                let j = c.ground.target[i] as usize;
                if j < n && self.ground_target_ok(i, j, me) {
                    aim = Some(j);
                }
            }
            let Some(j) = aim.or_else(|| self.ground_pick(i, me)) else {
                if let Some(c) = self.campaign.as_mut() {
                    c.ground.reload[i] = fs as i32;
                    c.ground.burst[i] = 0;
                }
                continue;
            };
            let b = self.units[j];
            let s = shot(a.kind, b.kind).unwrap();
            let mb = b.metres();
            let d = dist(me, [mb[0], mb[1]]);
            let k = d / s.reach;
            let mut p_hit = s.hit.0 + (s.hit.1 - s.hit.0) * k;
            if on_move(self, i) {
                p_hit *= MOVING_SHOOTER;
            }
            if on_move(self, j) {
                p_hit *= MOVING_TARGET;
            }
            let hit = (rng.random(1000) as f64) < p_hit * 1000.0;
            self.units[i].aim_az = (mb[0] - me[0]).atan2(mb[1] - me[1]) as f32;
            self.units[i].aim_el = 0.01;
            let Some(c) = self.campaign.as_mut() else { break };
            let g = &mut c.ground;
            if g.burst[i] == 0 || g.target[i] as usize != j {
                g.burst[i] = (s.rounds.0 + rng.random(s.rounds.1 - s.rounds.0 + 1)) as u8;
                g.target[i] = j as u16;
            }
            g.burst[i] -= 1;
            // Frames to wait before the next round (it fires on the frame after).
            g.reload[i] = if g.burst[i] > 0 {
                (s.interval * fs).round().max(1.0) as i32 - 1
            } else {
                let jitter = 0.8 + rng.random(400) as f64 / 1000.0;
                (s.reload * jitter * fs) as i32 - 1
            };
            let mut kill = false;
            let mut miss = (0, 0);
            if hit {
                let before = g.damage[j] as i32;
                let after = (before + hit_damage(&s, rng)).min(255);
                g.damage[j] = after as u8;
                kill = before < 100 && after >= 100;
                if kill {
                    let frames = (s.gun.flight_time(d) * fs).round().max(1.0) as u16;
                    g.landing.push((j as u16, frames));
                }
            } else {
                // Mostly short (the dirt kicked up in front of it), some over or wide.
                let along =
                    if rng.random(10) < 7 { -(8 + rng.random(40)) } else { 10 + rng.random(50) }
                        as f64;
                let wide = (rng.random(25) - 12) as f64;
                let (dx, dy) = ((mb[0] - me[0]) / d.max(1.0), (mb[1] - me[1]) / d.max(1.0));
                miss = (
                    (dx * along + dy * wide).round() as i16,
                    (dy * along - dx * wide).round() as i16,
                );
            }
            self.emit(SimEvent::GroundShot { from: i, to: j, gun: s.gun, hit, kill, miss });
        }
        self.credit = credit;
    }

    /// Unit `i` (at `me`) can fire at ground unit `j`: an enemy its side sees, in its gun's
    /// reach, not knocked out yet.
    fn ground_target_ok(&self, i: usize, j: usize, me: [f64; 2]) -> bool {
        let (a, b) = (&self.units[i], &self.units[j]);
        let blue = self.aaa_blue(i);
        if b.destroyed || self.aaa_blue(j) == blue || !self.unit_seen_by(j, blue) {
            return false;
        }
        if self.ground().is_some_and(|g| g.damage.get(j).is_some_and(|&d| d >= 100)) {
            return false;
        }
        let Some(s) = shot(a.kind, b.kind) else { return false };
        let mb = b.metres();
        dist(me, [mb[0], mb[1]]) <= s.reach
    }

    /// The enemy ground unit that unit `i` (at `me`) fires at next: tanks first for the
    /// tanks, the nearest otherwise.
    fn ground_pick(&self, i: usize, me: [f64; 2]) -> Option<usize> {
        let a = self.units[i];
        let mut best: Option<(f64, usize)> = None;
        for j in 0..self.units.len() {
            if !self.ground_target_ok(i, j, me) {
                continue;
            }
            let b = &self.units[j];
            let mb = b.metres();
            let d = dist(me, [mb[0], mb[1]]);
            let score = if b.kind.tank() == a.kind.tank() { d } else { d * 1.5 + 500.0 };
            if best.is_none_or(|(sc, _)| score < sc) {
                best = Some((score, j));
            }
        }
        best.map(|(_, j)| j)
    }

    /// The rounds that knock out a vehicle land: it is a wreck now.
    fn ground_landings(&mut self) {
        let Some(c) = self.campaign.as_mut() else { return };
        let mut done = Vec::new();
        c.ground.landing.retain_mut(|(k, frames)| {
            *frames = frames.saturating_sub(1);
            if *frames == 0 {
                done.push(*k as usize);
            }
            *frames > 0
        });
        for k in done {
            if self.units.get(k).is_some_and(|u| !u.destroyed) {
                self.units[k].destroyed = true;
                self.units[k].burst = 0;
                // The client draws the blast from the shot (an explosion event here would hit
                // the vehicles around it).
                self.emit(SimEvent::AaaDestroyed { unit: k, by: None });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_route_is_walked_by_distance() {
        let r = [[0.0, 0.0], [0.0, 100.0], [100.0, 100.0]];
        let (p, h) = point_at(&r, 50.0);
        assert_eq!(p, [0.0, 50.0]);
        assert!(h.abs() < 1e-9);
        let (p, h) = point_at(&r, 150.0);
        assert_eq!(p, [50.0, 100.0]);
        assert!((h - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
        assert_eq!(point_at(&r, 500.0).0, [100.0, 100.0]);
        assert_eq!(point_at(&r, -5.0).0, [0.0, 0.0]);
    }

    #[test]
    fn tanks_fight_armour_and_guns_leave_tanks_alone() {
        assert!(shot(GroundKind::T90, GroundKind::Leopard2).is_some());
        assert_eq!(shot(GroundKind::Bmp2, GroundKind::Leopard2).unwrap().gun, GroundGun::Cannon);
        assert!(shot(GroundKind::Zsu, GroundKind::Leopard2).is_none());
        assert!(shot(GroundKind::S60, GroundKind::Bradley).is_none());
    }

    /// Hits it takes to knock out a `b` with `a`'s gun, on average.
    fn hits_to_kill(a: GroundKind, b: GroundKind) -> f64 {
        let s = shot(a, b).unwrap();
        let mut rng = Rng::new(7);
        let tries = 2000;
        let mut hits = 0;
        for _ in 0..tries {
            let mut d = 0;
            while d < 100 {
                d += hit_damage(&s, &mut rng);
                hits += 1;
            }
        }
        hits as f64 / tries as f64
    }

    #[test]
    fn hits_to_knock_out() {
        use GroundKind::*;
        assert_eq!(hits_to_kill(T90, Bradley), 1.0);
        assert_eq!(hits_to_kill(Leopard2, Bmp2), 1.0);
        let tank = hits_to_kill(Leopard2, T90);
        assert!((1.2..=1.8).contains(&tank), "{tank}");
        let ifv = hits_to_kill(Bradley, Bmp2);
        assert!((10.5..=13.5).contains(&ifv), "{ifv}");
        let on_tank = hits_to_kill(Bmp2, Leopard2);
        assert!((20.0..=30.0).contains(&on_tank), "{on_tank}");
    }
}
