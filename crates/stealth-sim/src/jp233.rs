//! The Tornado's JP233 airfield attack dispenser, simplified for the game.
//!
//! A JP233 stays on its pylon: the fire button with it selected sets it going, and for
//! `DISPENSE_SECS` it throws its submunitions down out of its tubes as the aircraft flies on,
//! laying them in a strip some 500 m long behind it. Each comes down on a small chute that
//! takes its speed off; the empty dispenser stays on its pylon (the station has no round left).
//!
//! - The SG-357 runway cratering bomblets, along the middle of the strip, go off as they hit:
//!   a runway they crater often enough is put out of action, and anything they land by
//!   (vehicles, parked aircraft, launchers) is likely destroyed.
//! - The HB-876 area denial mines, spread wider, lie where they land and fire at the first
//!   enemy vehicle to come near; one landing next to a vehicle fires at once. Those left blow
//!   themselves up after a while.
//!
//! It is meant for a level run low over the target: above `MAX_FT` over the ground the
//! submunitions would drift all over the place, so it refuses.
//!
//! Positions are in metres east, north and up (like [`crate::paradrop`]).

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::Sim;
use crate::aaa::{GroundKind, M_PER_MAP};
use crate::types::*;

/// The JP233's weapon type: a dispenser a round.
pub const JP233_WEAPON: i32 = 30;
/// The highest it may be used from (ft over the ground).
pub const MAX_FT: i32 = 1000;
/// Seconds a dispenser throws out its load.
pub const DISPENSE_SECS: f64 = 2.0;
/// Submunitions in a dispenser: the cratering bomblets and the mines (fewer than the real
/// one's 215).
pub const CRATERING: u8 = 30;
pub const MINES: u8 = 60;
/// Seconds from leaving the tube to the chute opening; the chute's sink (m/s) and how fast it
/// takes the forward speed off (1/s).
const CHUTE_SECS: f32 = 0.25;
const CHUTE_SINK: f32 = 18.0;
const CHUTE_DRAG: f32 = 1.6;
const GRAVITY: f32 = 9.81;
/// The dispenser under its lugs (where the station point is): the middle of its tubes behind
/// them and its depth (m).
const TUBES_AFT_M: f64 = 1.9;
const DEPTH_M: f64 = 0.6;
/// Thrown out of the tubes: down (m/s), and across the strip at most (m/s): the craters
/// narrowly, the mines wide.
const EJECT_DOWN: f32 = 12.0;
const SPREAD_CRATER: f32 = 22.0;
const SPREAD_MINE: f32 = 60.0;
/// A cratering bomblet's blast: vehicles destroyed within this (m), the odds (percent) for a
/// tank and anything else, and the reach (m) to an object it may wreck.
const CRATER_BLAST_M: f64 = 18.0;
const CRATER_KILL_TANK: i32 = 35;
const CRATER_KILL_OTHER: i32 = 85;
const CRATER_OBJECT_M: f64 = 30.0;
/// Craters a runway takes before it is out of action, and the margin (m) round a runway a
/// crater still counts in.
pub const RUNWAY_CRATERS: u8 = 3;
const RUNWAY_MARGIN_M: f64 = 10.0;
/// Without the airfields' layout (the classic theaters): how near (m) a runway object a crater
/// must land.
const RUNWAY_OBJECT_M: f64 = 600.0;
/// A mine: a vehicle within this (m) sets it off; the odds (percent) its charge destroys a
/// tank and anything else; the reach (m) to an object it lands by.
const MINE_TRIGGER_M: f64 = 9.0;
const MINE_KILL_TANK: i32 = 70;
const MINE_KILL_OTHER: i32 = 95;
const MINE_OBJECT_M: f64 = 12.0;
/// Seconds a mine lies before it destroys itself: at least, and the spread over that.
const MINE_LIFE_SECS: i32 = 40;
const MINE_LIFE_SPREAD: i32 = 80;

const FT: f64 = 0.3048;
const M_PER_WU: f64 = M_PER_MAP / 32.0;

/// What a submunition is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Kind {
    /// An SG-357 runway cratering bomblet.
    #[default]
    Crater,
    /// An HB-876 area denial mine.
    Mine,
}

/// A submunition on its way down.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Bomblet {
    /// Drawn as the same one from frame to frame.
    pub id: u16,
    pub kind: Kind,
    /// Position and velocity (m, m/s: east, north, up).
    pub p: [f32; 3],
    pub v: [f32; 3],
    /// Seconds since it left the dispenser.
    pub t: f32,
    /// The pilot who dropped it.
    pub owner: u8,
}

impl Bomblet {
    /// Its chute is out.
    pub fn open(&self) -> bool {
        self.t >= CHUTE_SECS
    }
}

/// A mine lying on the ground.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Mine {
    pub id: u16,
    /// Where it lies (m: east, north, up).
    pub p: [f32; 3],
    pub owner: u8,
    /// Seconds before it destroys itself.
    pub left: f32,
}

/// A dispenser at work: its pilot, its station, the submunitions still in it, the time it has
/// been going.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Run {
    pub owner: u8,
    pub station: u8,
    pub craters: u8,
    pub mines: u8,
    pub t: f32,
}

/// Every JP233 at work, its submunitions in the air, its mines on the ground.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Jp233 {
    pub runs: Vec<Run>,
    pub bomblets: Vec<Bomblet>,
    pub mines: Vec<Mine>,
    /// Craters in each runway object so far (only the host's simulation keeps them).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cratered: Vec<(u16, u8)>,
    #[serde(default)]
    pub next_id: u16,
}

impl Jp233 {
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
            && self.bomblets.is_empty()
            && self.mines.is_empty()
            && self.cratered.is_empty()
    }

    fn id(&mut self) -> u16 {
        self.next_id = self.next_id.wrapping_add(1);
        self.next_id
    }

    /// What the other pilots' simulations draw: the dispensers, submunitions and mines.
    pub fn for_clients(&self) -> Jp233 {
        Jp233 { cratered: Vec::new(), ..self.clone() }
    }

    /// Pilot `owner`'s dispenser on station `station` is at work.
    pub fn running(&self, owner: u8, station: usize) -> bool {
        self.runs.iter().any(|r| r.owner == owner && r.station as usize == station)
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

/// A ground unit's position (m east, north).
fn unit_metres(u: &crate::aaa::GroundUnit) -> [f64; 2] {
    let (x, y) = u.map_xy();
    [x * M_PER_MAP, (0x8000 as f64 - y) * M_PER_MAP]
}

impl Sim {
    /// The JP233s at work and their submunitions (for drawing).
    pub fn jp233(&self) -> &Jp233 {
        &self.jp233
    }

    /// The fire button with the JP233 selected: the dispenser with the most left in it starts,
    /// low enough over the ground.
    pub(crate) fn release_jp233(&mut self) {
        let (w, count) = self.selected_weapon();
        if count <= 0 {
            return;
        }
        if self.height_above_ground() > MAX_FT {
            self.sound(0x20, 1);
            self.msg(text!("sim-jp233-too-high", feet = MAX_FT));
            return;
        }
        let me = self.pl.id;
        let Some(station) = (0..self.pl.bays.len()).find(|&i| {
            self.pl.bays[i].0 == w && self.pl.bays[i].1 > 0 && !self.jp233.running(me, i)
        }) else {
            return;
        };
        self.jp233.runs.push(Run {
            owner: me,
            station: station as u8,
            craters: CRATERING,
            mines: MINES,
            t: 0.0,
        });
        self.sound(0x10, 1);
        self.msg(text!("sim-jp233-dispensing"));
    }

    /// The ground (m) under a point (m east, north).
    fn jp233_ground(&self, e: f64, n: f64) -> f64 {
        let (x, y) = (e / M_PER_WU, 0x10_0000 as f64 - n / M_PER_WU);
        self.slam_ground(x, y) * FT
    }

    /// Where the active pilot's station `station` hangs its dispenser (m east, north, up) and
    /// the aircraft's velocity (m/s).
    fn jp233_motion(&self, station: usize) -> ([f64; 3], [f64; 3]) {
        let p = &self.pl.p;
        let v = crate::ballistics::release_velocity(p.heading, p.velocity, p.climb);
        // The world's ft/s east, south, up to m/s east, north, up.
        let v = [v[0] * M_PER_WU, -v[1] * M_PER_WU, v[2] * FT];
        let at = self.station_point(station).unwrap_or([
            p.wx as f64,
            (0x10_0000 - p.wy) as f64,
            p.alt as f64 - 5.0,
        ]);
        ([at[0] * M_PER_WU, (0x10_0000 as f64 - at[1]) * M_PER_WU, at[2] * FT], v)
    }

    /// One frame of the JP233s: the dispensers throwing out their loads, the submunitions
    /// coming down, the mines lying in wait (the host's; a mirror draws what it is sent).
    pub(crate) fn update_jp233(&mut self, present: &[usize]) {
        if self.jp233.is_empty() || self.mirror {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let back = self.pl.id as usize;
        // The dispensers at work, from their aircraft.
        for k in 0..self.jp233.runs.len() {
            let r = self.jp233.runs[k];
            let owner = r.owner as usize;
            if !present.contains(&owner) {
                self.jp233.runs[k].craters = 0;
                self.jp233.runs[k].mines = 0;
                continue;
            }
            self.activate(owner);
            let (p, v) = self.jp233_motion(r.station as usize);
            let t = r.t as f64 + dt;
            // What should be out by now, evenly over the run.
            let due = |all: u8| (all as f64 * (t / DISPENSE_SECS).min(1.0)).round() as u8;
            let craters = CRATERING - r.craters;
            let mines = MINES - r.mines;
            let out = [
                (Kind::Crater, due(CRATERING).saturating_sub(craters)),
                (Kind::Mine, due(MINES).saturating_sub(mines)),
            ];
            let h = v[0].atan2(v[1]);
            let (side_e, side_n) = (h.cos(), -h.sin());
            for (kind, n) in out {
                for _ in 0..n {
                    let spread = match kind {
                        Kind::Crater => SPREAD_CRATER,
                        Kind::Mine => SPREAD_MINE,
                    };
                    let across = (self.rng.random(2001) - 1000) as f32 / 1000.0 * spread;
                    // Out of the tubes along the dispenser's underside: it reaches from 1.4 m
                    // ahead of its lugs to 5.2 m behind them.
                    let along = (self.rng.random(500) - 250) as f64 / 100.0 - TUBES_AFT_M;
                    let speed = v[0].hypot(v[1]).max(1.0);
                    let at =
                        [p[0] + v[0] / speed * along, p[1] + v[1] / speed * along, p[2] - DEPTH_M];
                    let id = self.jp233.id();
                    self.jp233.bomblets.push(Bomblet {
                        id,
                        kind,
                        p: at.map(|c| c as f32),
                        v: [
                            v[0] as f32 + side_e as f32 * across,
                            v[1] as f32 + side_n as f32 * across,
                            v[2] as f32 - EJECT_DOWN,
                        ],
                        t: 0.0,
                        owner: r.owner,
                    });
                }
            }
            let run = &mut self.jp233.runs[k];
            run.t = t as f32;
            run.craters -= out[0].1.min(run.craters);
            run.mines -= out[1].1.min(run.mines);
        }
        // Empty dispensers are done (they stay on their pylons, no round left).
        let mut k = 0;
        while k < self.jp233.runs.len() {
            let r = self.jp233.runs[k];
            if r.craters > 0 || r.mines > 0 {
                k += 1;
                continue;
            }
            self.jp233.runs.remove(k);
            if present.contains(&(r.owner as usize)) {
                self.activate(r.owner as usize);
                if let Some(b) = self.pl.bays.get_mut(r.station as usize) {
                    b.1 = (b.1 - 1).max(0);
                }
            }
        }
        // The submunitions on their chutes.
        let mut landed = Vec::new();
        let dtf = dt as f32;
        let mut bomblets = std::mem::take(&mut self.jp233.bomblets);
        bomblets.retain_mut(|b| {
            b.t += dtf;
            if b.open() {
                let k = (-CHUTE_DRAG * dtf).exp();
                b.v[0] *= k;
                b.v[1] *= k;
                b.v[2] += (-CHUTE_SINK - b.v[2]) * (1.0 - (-3.0 * dtf).exp());
            } else {
                b.v[2] -= GRAVITY * dtf;
            }
            for a in 0..3 {
                b.p[a] += b.v[a] * dtf;
            }
            let ground = self.jp233_ground(b.p[0] as f64, b.p[1] as f64) as f32;
            if b.p[2] <= ground {
                b.p[2] = ground;
                landed.push(*b);
                return false;
            }
            true
        });
        self.jp233.bomblets = bomblets;
        for b in landed {
            match b.kind {
                Kind::Crater => self.crater(&b),
                Kind::Mine => self.mine_lands(&b),
            }
        }
        // The mines lying in wait.
        let mut k = 0;
        while k < self.jp233.mines.len() {
            let m = self.jp233.mines[k];
            let left = m.left - dtf;
            if left <= 0.0 {
                self.jp233.mines.remove(k);
                self.emit(SimEvent::Jp233Burst { at: fine(m.p), crater: false, kill: false });
                continue;
            }
            self.jp233.mines[k].left = left;
            if let Some(u) = self.vehicle_near(m.p, MINE_TRIGGER_M) {
                self.jp233.mines.remove(k);
                self.mine_fires(&m, u);
                continue;
            }
            k += 1;
        }
        self.activate(back);
    }

    /// The nearest enemy vehicle within `reach` m of `p`.
    fn vehicle_near(&self, p: [f32; 3], reach: f64) -> Option<usize> {
        let (e, n) = (p[0] as f64, p[1] as f64);
        // A quick test on the map units first.
        let (mx, my) = (e / M_PER_MAP, 0x8000 as f64 - n / M_PER_MAP);
        let r_map = reach / M_PER_MAP + 1.0;
        (0..self.units.len())
            .filter(|&k| {
                let u = &self.units[k];
                !u.destroyed
                    && !u.gone
                    && !self.aaa_blue(k)
                    && !matches!(u.kind, GroundKind::Manpads | GroundKind::DroneCenter)
                    && !u.kind.structure()
                    && (u.x as f64 - mx).abs() < r_map + 2.0
                    && (u.y as f64 - my).abs() < r_map + 2.0
            })
            .map(|k| {
                let at = unit_metres(&self.units[k]);
                (k, (at[0] - e).hypot(at[1] - n))
            })
            .filter(|&(_, d)| d < reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k)
    }

    /// Ground unit `k` destroyed by pilot `owner`'s JP233.
    fn jp233_kills_unit(&mut self, k: usize, owner: u8) {
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

    /// World object `k` destroyed by pilot `owner`'s JP233.
    fn jp233_kills_object(&mut self, k: usize, owner: u8) {
        if self.objects[k].flags & oflags::DESTROYED != 0 {
            return;
        }
        self.activate(owner as usize);
        let long = self.weapon_types[(JP233_WEAPON as usize).min(self.weapon_types.len() - 1)]
            .long_name
            .clone();
        self.credit = Some(owner);
        self.destroy_ground(k as i32);
        self.credit = None;
        let place = self.place_name(k);
        self.msg(text!("sim-destroyed-by", target = place, weapon = long));
    }

    /// The world objects (not runways) the JP233 can harm within `reach` m of `p`.
    fn objects_near(&self, p: [f32; 3], reach: f64) -> Vec<usize> {
        let (e, n) = (p[0] as f64, p[1] as f64);
        (1..self.objects.len())
            .filter(|&k| {
                let o = &self.objects[k];
                if o.flags & oflags::DESTROYED != 0
                    || self.category(k as i32) == 1
                    || self.compat(JP233_WEAPON, k as i32) <= 0
                {
                    return false;
                }
                let at = crate::asw::object_metres(o);
                (at[0] - e).hypot(at[1] - n) < reach
            })
            .collect()
    }

    /// A cratering bomblet goes off: vehicles and objects by it, and a crater in the runway
    /// under it.
    fn crater(&mut self, b: &Bomblet) {
        let (e, n) = (b.p[0] as f64, b.p[1] as f64);
        let mut kill = false;
        let near: Vec<usize> = (0..self.units.len())
            .filter(|&k| {
                let u = &self.units[k];
                !u.destroyed && !u.gone && {
                    let at = unit_metres(u);
                    (at[0] - e).hypot(at[1] - n) < CRATER_BLAST_M
                }
            })
            .collect();
        for k in near {
            let odds = if self.units[k].kind.tank() { CRATER_KILL_TANK } else { CRATER_KILL_OTHER };
            if self.rng.random(100) < odds {
                self.jp233_kills_unit(k, b.owner);
                kill = true;
            }
        }
        for k in self.objects_near(b.p, CRATER_OBJECT_M) {
            if self.rng.random(100) < self.compat(JP233_WEAPON, k as i32) * 10 {
                self.jp233_kills_object(k, b.owner);
                kill = true;
            }
        }
        if let Some(k) = self.runway_under(e, n) {
            let hits = match self.jp233.cratered.iter_mut().find(|c| c.0 as usize == k) {
                Some(c) => {
                    c.1 = c.1.saturating_add(1);
                    c.1
                }
                None => {
                    self.jp233.cratered.push((k as u16, 1));
                    1
                }
            };
            if hits >= RUNWAY_CRATERS {
                self.jp233_kills_object(k, b.owner);
                self.jp233.cratered.retain(|c| c.0 as usize != k);
            }
        }
        self.emit(SimEvent::Jp233Burst { at: fine(b.p), crater: true, kill });
    }

    /// The runway object (intact, one the JP233 can crater) a crater at `e`, `n` (m) is in.
    fn runway_under(&self, e: f64, n: f64) -> Option<usize> {
        let runways = self.runways();
        let candidates = (1..self.objects.len()).filter(|&k| {
            self.category(k as i32) == 1
                && self.objects[k].flags & oflags::DESTROYED == 0
                && self.compat(JP233_WEAPON, k as i32) > 0
        });
        let dist = |k: usize| {
            let at = crate::asw::object_metres(&self.objects[k]);
            (at[0] - e).hypot(at[1] - n)
        };
        if runways.is_empty() {
            return candidates
                .map(|k| (k, dist(k)))
                .filter(|&(_, d)| d < RUNWAY_OBJECT_M)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(k, _)| k);
        }
        // On a runway's surface: its object is the nearest runway object to the runway's
        // middle.
        let r = runways.iter().find(|r| {
            let (dx, dy) = (e - r.c.0, n - r.c.1);
            let along = dx * r.axis.0 + dy * r.axis.1;
            let across = -dx * r.axis.1 + dy * r.axis.0;
            along.abs() <= r.half_len + RUNWAY_MARGIN_M
                && across.abs() <= r.half_width + RUNWAY_MARGIN_M
        })?;
        candidates
            .map(|k| {
                let at = crate::asw::object_metres(&self.objects[k]);
                (k, (at[0] - r.c.0).hypot(at[1] - r.c.1))
            })
            .filter(|&(_, d)| d < r.half_len + 300.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k)
    }

    /// A mine comes down: by a vehicle it fires at once, by an object it may wreck it, else it
    /// lies in wait.
    fn mine_lands(&mut self, b: &Bomblet) {
        let mine = Mine {
            id: b.id,
            p: b.p,
            owner: b.owner,
            left: (MINE_LIFE_SECS + self.rng.random(MINE_LIFE_SPREAD)) as f32,
        };
        if let Some(u) = self.vehicle_near(b.p, MINE_TRIGGER_M * 0.5) {
            self.mine_fires(&mine, u);
            return;
        }
        if let Some(&k) = self.objects_near(b.p, MINE_OBJECT_M).first() {
            let kill = self.rng.random(100) < self.compat(JP233_WEAPON, k as i32) * 10;
            if kill {
                self.jp233_kills_object(k, b.owner);
            }
            self.emit(SimEvent::Jp233Burst { at: fine(b.p), crater: false, kill });
            return;
        }
        self.jp233.mines.push(mine);
    }

    /// Mine `m` fires at ground unit `u`.
    fn mine_fires(&mut self, m: &Mine, u: usize) {
        let odds = if self.units[u].kind.tank() { MINE_KILL_TANK } else { MINE_KILL_OTHER };
        let kill = self.rng.random(100) < odds;
        if kill {
            self.jp233_kills_unit(u, m.owner);
        }
        self.emit(SimEvent::Jp233Burst { at: fine(m.p), crater: false, kill });
    }
}
