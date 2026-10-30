//! Campaign armour: convoys driving their route to an enemy place, and the ground units that
//! see and fight each other (tanks, infantry fighting vehicles and the convoys' air defence
//! guns against whatever of the enemy's stands on the ground).
//!
//! A convoy's vehicles are ordinary [`GroundUnit`]s with a fine position ([`Fine`]); the air
//! defence guns among them keep firing at aircraft as anywhere else. Positions are metres
//! (east, north). A column halts while any of its vehicles can fire at an enemy it sees
//! within [`HALT_RANGE`]; its
//! vehicles keep their place in it by the distance driven along the
//! route, so a wreck leaves its gap.

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{Fine, GroundKind, GroundUnit, M_PER_MAP};
use crate::rng::Rng;
use crate::types::SimEvent;

/// Road march speed (m/s) and cross-country speed in the assault.
pub const CONVOY_SPEED: f64 = 20.0;
const ASSAULT_SPEED: f64 = 6.0;
/// Metres between the column's vehicles.
const GAP: f64 = 60.0;
/// The convoy leaves the road this far (m) before its goal and attacks in line abreast, its
/// vehicles this far apart, stopping this short of the place's centre.
const ASSAULT_START: f64 = 2000.0;
const ASSAULT_SPACING: f64 = 90.0;
const ASSAULT_STOP: f64 = 400.0;
/// A ground unit sees enemy ground units this far (m), the line of sight over the relief
/// sampled this often (m) and clear by this much (m).
pub const SIGHT_RANGE: f64 = 4000.0;
/// A column drives on (firing as it goes) until an enemy it can hit is this close (m).
const HALT_RANGE: f64 = 2500.0;
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
    pub rng: Option<Rng>,
    /// Road march speed (m/s), 0: [`CONVOY_SPEED`] (a test aid).
    pub speed: f32,
}

/// A ground gun's shot at another ground unit.
struct Shot {
    /// Reach (m), seconds to reload, chance to hit at point blank and at full reach, chance
    /// that a hit kills.
    reach: f64,
    reload: f64,
    hit: (f64, f64),
    kill: f64,
    missile: bool,
}

/// How unit kind `a` fights ground unit kind `b`, if it does.
fn shot(a: GroundKind, b: GroundKind) -> Option<Shot> {
    let soft = !b.armour() && !matches!(b, GroundKind::Zsu | GroundKind::Gepard);
    let kill = |tank: f64, ifv: f64, light: f64, other: f64| {
        if b.tank() {
            tank
        } else if b.carrier() {
            ifv
        } else if soft {
            other
        } else {
            light
        }
    };
    if a.tank() {
        let k = kill(0.5, 0.8, 0.9, 0.9);
        return Some(Shot {
            reach: 3000.0,
            reload: 9.0,
            hit: (0.6, 0.15),
            kill: k,
            missile: false,
        });
    }
    if a.carrier() {
        if b.tank() {
            // TOW or Konkurs.
            return Some(Shot {
                reach: 3500.0,
                reload: 30.0,
                hit: (0.7, 0.4),
                kill: 0.6,
                missile: true,
            });
        }
        let k = kill(0.0, 0.35, 0.6, 0.7);
        return Some(Shot { reach: 2000.0, reload: 5.0, hit: (0.4, 0.1), kill: k, missile: false });
    }
    if matches!(a, GroundKind::Zsu | GroundKind::Gepard) && !b.tank() {
        let k = kill(0.0, 0.25, 0.5, 0.6);
        return Some(Shot {
            reach: 1500.0,
            reload: 4.0,
            hit: (0.4, 0.15),
            kill: k,
            missile: false,
        });
    }
    None
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

    /// Unit `k` is on the move or reloading after a shot at a ground target (its turret
    /// matters).
    pub(crate) fn ground_busy(&self, k: usize) -> bool {
        self.unit_moving(k)
            || self.ground().is_some_and(|g| g.reload.get(k).is_some_and(|&r| r > self.fs))
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
        self.ground_fire(&mut rng);
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
                && shot(a.kind, b.kind).is_some_and(|s| {
                    s.kill > 0.0 && dist([m[0], m[1]], [mb[0], mb[1]]) < s.reach.min(HALT_RANGE)
                })
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

    /// The ground guns fire at the enemy's ground units they see.
    fn ground_fire(&mut self, rng: &mut Rng) {
        let fs = self.fs.max(1) as f64;
        let n = self.units.len();
        let credit = self.credit.take();
        for i in 0..n {
            let a = self.units[i];
            if a.destroyed
                || !(a.kind.armour() || matches!(a.kind, GroundKind::Zsu | GroundKind::Gepard))
            {
                continue;
            }
            let Some(c) = self.campaign.as_mut() else { break };
            let reload = &mut c.ground.reload[i];
            if *reload > 0 {
                *reload -= 1;
                continue;
            }
            // An air defence gun busy with an aircraft has no time for the ground.
            let busy = c.aaa_target.get(i).copied().flatten().is_some() || a.burst > 0;
            if busy || self.manned_by(i).is_some() {
                continue;
            }
            let blue = self.aaa_blue(i);
            let m = a.metres();
            let me = [m[0], m[1]];
            let mut best: Option<(f64, usize)> = None;
            for j in 0..n {
                let b = &self.units[j];
                if b.destroyed || self.aaa_blue(j) == blue || !self.unit_seen_by(j, blue) {
                    continue;
                }
                let Some(s) = shot(a.kind, b.kind) else { continue };
                let mb = b.metres();
                let d = dist(me, [mb[0], mb[1]]);
                if d > s.reach || s.kill <= 0.0 {
                    continue;
                }
                // Tanks first for the tanks and the missiles, the nearest otherwise.
                let score =
                    if b.kind.tank() == (a.kind.tank() || s.missile) { d } else { d * 1.5 + 500.0 };
                if best.is_none_or(|(sc, _)| score < sc) {
                    best = Some((score, j));
                }
            }
            let Some((_, j)) = best else {
                if let Some(c) = self.campaign.as_mut() {
                    c.ground.reload[i] = fs as i32;
                }
                continue;
            };
            let b = self.units[j];
            let s = shot(a.kind, b.kind).unwrap();
            let mb = b.metres();
            let d = dist(me, [mb[0], mb[1]]);
            let k = d / s.reach;
            let p_hit = s.hit.0 + (s.hit.1 - s.hit.0) * k;
            let hit = (rng.random(1000) as f64) < p_hit * 1000.0;
            let kill = hit && (rng.random(1000) as f64) < s.kill * 1000.0;
            self.units[i].aim_az = (mb[0] - me[0]).atan2(mb[1] - me[1]) as f32;
            self.units[i].aim_el = 0.01;
            let jitter = 0.7 + rng.random(600) as f64 / 1000.0;
            if let Some(c) = self.campaign.as_mut() {
                c.ground.reload[i] = (s.reload * jitter * fs) as i32;
            }
            self.emit(SimEvent::GroundShot { from: i, to: j, hit, missile: s.missile });
            // The client draws the blast from the shot (an explosion event here would hit the
            // vehicles around it).
            if kill {
                self.units[j].destroyed = true;
                self.units[j].burst = 0;
                self.emit(SimEvent::AaaDestroyed { unit: j, by: None });
            }
        }
        self.credit = credit;
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
        assert!(shot(GroundKind::Bmp2, GroundKind::Leopard2).unwrap().missile);
        assert!(shot(GroundKind::Zsu, GroundKind::Leopard2).is_none());
        assert!(shot(GroundKind::S60, GroundKind::Bradley).is_none());
    }
}
