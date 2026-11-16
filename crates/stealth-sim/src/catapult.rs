//! A carrier's four catapults: where each holds an aircraft back, which way it shoots it, and
//! the launch an AI aircraft makes from one (the player's is its flight model's).
//!
//! The tracks are measured on the ship's painted deck (tools/models/nimitz.py's nimitz.blend,
//! metres, bow +y, starboard +x, the ship's middle at 0): 1 and 2 on the bow (starboard and
//! port), 3 and 4 in the waist on the angled deck (tools/models/carrier.py paints those two).
//! The ship is drawn at the game's carrier size: 480 model units for its 304 m, at the tile
//! level's 4 world units each; its middle is its tile object (`PIVOTS`), the bow north where the
//! theater's take-off runway points north (`Sim::ns`).

use crate::Sim;
use crate::types::{fflags, oflags};

/// Catapults per carrier.
pub const CATAPULTS: usize = 4;

/// World units per metre of nimitz.blend.
pub const WU_PER_M: f64 = 480.0 / 304.0 * 4.0;

/// The tracks (m, nimitz.blend): the shuttle's start, and the track's forward end.
pub const TRACKS: [((f64, f64), (f64, f64)); CATAPULTS] = [
    ((16.7, 28.2), (5.8, 148.0)),
    ((-5.1, 27.7), (-5.1, 148.0)),
    ((-9.3, -33.4), (-22.3, 50.6)),
    ((-17.6, -44.9), (-30.6, 39.1)),
];

/// The aircraft's reference point stands this far behind the shuttle (wu).
const HOLD_BACK_WU: f64 = 12.0;

/// The carriers' tile objects (their ships' middles) off their world objects: world object
/// (map x, y) -> (east, north) wu. Without one the world object is the middle. The client's
/// `sea::Deck::find` finds the same (a test there keeps the two in step).
pub const PIVOTS: [((i32, i32), (f64, f64)); 10] = [
    ((19200, 9594), (20.0, 16.0)),
    ((30160, 27174), (12.0, 0.0)),
    ((24777, 22417), (28.0, 24.0)),
    ((5188, 16566), (16.0, 4.0)),
    ((11021, 3336), (28.0, 24.0)),
    ((29521, 12841), (4.0, 0.0)),
    ((29109, 19674), (4.0, 28.0)),
    ((7438, 23719), (28.0, 12.0)),
    ((21758, 27860), (24.0, 4.0)),
    ((25850, 17651), (4.0, 4.0)),
];

/// An aircraft (or a pilot) this close to a catapult's hold-back point (wu) occupies it.
pub const OCCUPIED_WU: f64 = 48.0;
/// AI launch: the run-up, the shooter's signal and the stroke (seconds), and the speed the
/// stroke ends at (wu/s; the classic take-off lifts the nose from 150).
pub const AI_RUNUP_S: i32 = 4;
pub const SHOOT_S: f32 = 1.0;
pub const STROKE_S: f32 = 2.5;
pub const AI_STROKE_SPEED: i32 = 150;
/// A waiting AI aircraft stands this far behind the hold-back point, per place in the queue
/// (wu), and taxies up at this speed (wu/s).
pub const QUEUE_WU: f64 = 64.0;
pub const TAXI_WU_S: f64 = 30.0;

/// A carrier's middle (wu east, north) from its world object (map units).
pub fn pivot(obj_x: i32, obj_y: i32) -> (f64, f64) {
    let (de, dn) = PIVOTS
        .iter()
        .find(|((x, y), _)| (*x, *y) == (obj_x, obj_y))
        .map_or((0.0, 0.0), |&(_, d)| d);
    (obj_x as f64 * 32.0 + de, (0x8000 - obj_y) as f64 * 32.0 + dn)
}

/// A catapult on a carrier: the aircraft's hold-back point (wu east, north), the launch
/// heading (BAM, clockwise from north) and the track's length ahead of it (wu).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatPose {
    pub east: f64,
    pub north: f64,
    pub heading: u16,
    pub length: f64,
}

impl CatPose {
    /// The heading in radians (clockwise from north).
    pub fn heading_rad(&self) -> f64 {
        self.heading as f64 / 65536.0 * std::f64::consts::TAU
    }

    /// The point `d` wu along the track (wu east, north).
    pub fn along(&self, d: f64) -> (f64, f64) {
        let h = self.heading_rad();
        (self.east + h.sin() * d, self.north + h.cos() * d)
    }
}

/// Catapult `cat` of the carrier whose world object is at (map) `obj`, its bow north for
/// `ns` 1 (south for -1).
pub fn pose(obj: (i32, i32), ns: i32, cat: usize) -> CatPose {
    let ((ax, ay), (fx, fy)) = TRACKS[cat % CATAPULTS];
    let s = if ns < 0 { -1.0 } else { 1.0 };
    let (dx, dy) = ((fx - ax) * s, (fy - ay) * s);
    let len = dx.hypot(dy);
    let (ux, uy) = (dx / len, dy / len);
    let (pe, pn) = pivot(obj.0, obj.1);
    let (e, n) = (pe + ax * s * WU_PER_M, pn + ay * s * WU_PER_M);
    let heading = (ux.atan2(uy) / std::f64::consts::TAU * 65536.0).round() as i64 as u16;
    CatPose {
        east: e - ux * HOLD_BACK_WU,
        north: n - uy * HOLD_BACK_WU,
        heading,
        length: len * WU_PER_M + HOLD_BACK_WU,
    }
}

/// Launch phases (as a pilot's report and an AI aircraft's state carry them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// AI only: queued behind the catapult for its turn.
    Wait = 1,
    /// On the catapult, the engines running up.
    Tension = 2,
    /// Full power: the shooter's signal, the launch a second later.
    Shoot = 3,
    /// The shuttle runs.
    Stroke = 4,
}

impl Phase {
    pub fn from_bits(b: i32) -> Option<Phase> {
        match b {
            1 => Some(Phase::Wait),
            2 => Some(Phase::Tension),
            3 => Some(Phase::Shoot),
            4 => Some(Phase::Stroke),
            _ => None,
        }
    }
}

/// A launch as a pilot's report carries it (`PlayerState::catapult`): 0 none, else the
/// catapult in the low two bits and the phase above them.
pub fn report(cat: usize, phase: Phase) -> u8 {
    (cat as u8 & 3) | (phase as u8) << 2
}

/// A pilot's report's launch: (catapult, phase).
pub fn from_report(r: u8) -> Option<(usize, Phase)> {
    Phase::from_bits((r >> 2) as i32).map(|p| ((r & 3) as usize, p))
}

/// An AI aircraft's launch (`FlightUnit::cat`): the catapult (2 bits), the phase (3 bits),
/// the frames in it (11 bits) and the carrier's object index (15 bits). 0: none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiLaunch {
    pub cat: usize,
    pub phase: Phase,
    pub frames: i32,
    pub carrier: usize,
}

impl AiLaunch {
    pub fn decode(v: i32) -> Option<AiLaunch> {
        let phase = Phase::from_bits((v >> 2) & 7)?;
        Some(AiLaunch {
            cat: (v & 3) as usize,
            phase,
            frames: (v >> 5) & 0x7FF,
            carrier: ((v >> 16) & 0x7FFF) as usize,
        })
    }

    pub fn encode(&self) -> i32 {
        (self.cat as i32 & 3)
            | (self.phase as i32) << 2
            | self.frames.clamp(0, 0x7FF) << 5
            | (self.carrier as i32 & 0x7FFF) << 16
    }
}

impl Sim {
    /// Catapult `cat` of carrier object `b`.
    pub fn catapult(&self, b: usize, cat: usize) -> CatPose {
        let o = self.objects.get(b).copied().unwrap_or_default();
        pose((o.x, o.y), self.ns, cat)
    }

    /// Who is on catapult `cat` of carrier `b` (or queued for it), other than pilot `except`
    /// and flight unit `except_unit`: how many.
    fn catapult_users(
        &self,
        b: usize,
        cat: usize,
        except: Option<u8>,
        except_unit: Option<usize>,
    ) -> usize {
        let p = self.catapult(b, cat);
        let near = |e: f64, n: f64| (e - p.east).abs().max((n - p.north).abs()) < OCCUPIED_WU;
        let pilots = self
            .present()
            .into_iter()
            .filter(|&k| Some(k as u8) != except)
            .filter_map(|k| self.pilot_view(k as u8))
            .filter(|v| v.alt < 400 && near(v.wx as f64, v.wy as f64))
            .count();
        let units = self
            .flights
            .iter()
            .enumerate()
            .filter(|&(i, u)| Some(i) != except_unit && u.flags & fflags::EXISTS != 0)
            .filter(|(_, u)| {
                match AiLaunch::decode(u.cat).filter(|_| u.flags & fflags::TAKEOFF != 0) {
                    Some(l) => l.carrier == b && l.cat == cat,
                    None => u.alt < 400 && near(u.wx as f64, (0x8000 * 32 - u.wy) as f64),
                }
            })
            .count();
        pilots + units
    }

    /// Where pilot `id` starts on carrier `b`: (x east, y north wu, heading BAM, the launch
    /// as his report carries it). On the first free catapult, else queued behind the least
    /// busy one (his flight model taxies him up when it is free).
    pub fn catapult_start(&self, b: usize, id: u8) -> (i32, i32, i32, u8) {
        let (cat, waiting) = (0..CATAPULTS)
            .map(|c| (c, self.catapult_users(b, c, Some(id), None)))
            .min_by_key(|&(c, n)| (n, c))
            .unwrap_or((0, 0));
        let p = self.catapult(b, cat);
        let (e, n) = p.along(-(waiting as f64) * QUEUE_WU);
        let phase = if waiting == 0 { Phase::Tension } else { Phase::Wait };
        (e.round() as i32, n.round() as i32, p.heading as i32, report(cat, phase))
    }

    /// Puts flight unit `i` on a catapult of carrier `b` (queued behind the least busy one when
    /// none is free).
    pub(crate) fn put_on_catapult(&mut self, i: usize, b: usize) {
        let (cat, waiting) = (0..CATAPULTS)
            .map(|c| (c, self.catapult_users(b, c, None, Some(i))))
            .min_by_key(|&(c, n)| (n, c))
            .unwrap_or((0, 0));
        let p = self.catapult(b, cat);
        let (e, n) = p.along(-(waiting as f64) * QUEUE_WU);
        let phase = if waiting == 0 { Phase::Tension } else { Phase::Wait };
        let u = &mut self.flights[i];
        u.wx = e.round() as i32;
        u.wy = (0x8000 as f64 * 32.0 - n).round() as i32;
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.heading = crate::math::s16(p.heading as i32);
        u.speed = 0;
        u.cat = AiLaunch { cat, phase, frames: 0, carrier: b }.encode();
    }

    /// One frame of flight unit `i`'s launch (`u` its state, taken out of `flights`): true
    /// while it is on the catapult (its moves are the launch's).
    pub(crate) fn catapult_step(&mut self, i: usize, u: &mut crate::types::FlightUnit) -> bool {
        let Some(mut l) = AiLaunch::decode(u.cat) else { return false };
        let fs = self.fs.max(1);
        let p = self.catapult(l.carrier, l.cat);
        let carrier_ok =
            self.objects.get(l.carrier).is_some_and(|o| o.flags & oflags::CARRIER != 0);
        if !carrier_ok || u.flags & fflags::DESTROYED != 0 {
            u.cat = 0;
            return false;
        }
        let set = |u: &mut crate::types::FlightUnit, (e, n): (f64, f64)| {
            u.wx = e.round() as i32;
            u.wy = (0x8000 as f64 * 32.0 - n).round() as i32;
            u.x = (u.wx >> 5) & 0xFFFF;
            u.y = (u.wy >> 5) & 0xFFFF;
        };
        u.pitch = 0;
        u.bank = 0;
        u.heading = crate::math::s16(p.heading as i32);
        l.frames += 1;
        match l.phase {
            Phase::Wait => {
                u.speed = 0;
                // Its turn once nobody is on the catapult (the first of those queued).
                let ahead = self.flights.iter().enumerate().any(|(k, f)| {
                    k != i
                        && f.flags & fflags::EXISTS != 0
                        && AiLaunch::decode(f.cat).is_some_and(|o| {
                            o.carrier == l.carrier
                                && o.cat == l.cat
                                && (o.phase != Phase::Wait || k < i)
                        })
                });
                let pilot_on = self.catapult_users(l.carrier, l.cat, None, Some(i))
                    > self
                        .flights
                        .iter()
                        .enumerate()
                        .filter(|&(k, f)| {
                            k != i
                                && f.flags & fflags::EXISTS != 0
                                && AiLaunch::decode(f.cat)
                                    .is_some_and(|o| o.carrier == l.carrier && o.cat == l.cat)
                        })
                        .count();
                if !ahead && !pilot_on {
                    // Taxi up to the hold-back point.
                    let (e, n) = (u.wx as f64, 0x8000 as f64 * 32.0 - u.wy as f64);
                    let (de, dn) = (p.east - e, p.north - n);
                    let d = de.hypot(dn);
                    let step = TAXI_WU_S / fs as f64;
                    if d <= step {
                        set(u, (p.east, p.north));
                        l = AiLaunch { phase: Phase::Tension, frames: 0, ..l };
                    } else {
                        set(u, (e + de / d * step, n + dn / d * step));
                    }
                }
            }
            Phase::Tension => {
                u.speed = 0;
                set(u, (p.east, p.north));
                if l.frames >= AI_RUNUP_S * fs {
                    l = AiLaunch { phase: Phase::Shoot, frames: 0, ..l };
                }
            }
            Phase::Shoot => {
                u.speed = 0;
                set(u, (p.east, p.north));
                if l.frames as f32 >= SHOOT_S * fs as f32 {
                    l = AiLaunch { phase: Phase::Stroke, frames: 0, ..l };
                }
            }
            Phase::Stroke => {
                let t = l.frames as f32 / fs as f32;
                let accel = AI_STROKE_SPEED as f32 / STROKE_S;
                let t = t.min(STROKE_S);
                u.speed = (accel * t).round() as i32;
                set(u, p.along((0.5 * accel * t * t) as f64));
                if t >= STROKE_S {
                    u.cat = 0;
                    u.speed = AI_STROKE_SPEED;
                    return false;
                }
            }
        }
        u.cat = l.encode();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_with_the_bow() {
        for c in 0..CATAPULTS {
            let n = pose((0x4000, 0x4000), 1, c);
            let s = pose((0x4000, 0x4000), -1, c);
            let (pe, pn) = pivot(0x4000, 0x4000);
            assert!(((n.east - pe) + (s.east - pe)).abs() < 1e-6);
            assert!(((n.north - pn) + (s.north - pn)).abs() < 1e-6);
            assert_eq!(n.heading.wrapping_add(0x8000), s.heading);
            // On the deck (the landing box's 8 x 30 map units half sizes), the track too.
            for p in [(n.east, n.north), n.along(n.length)] {
                assert!(
                    (p.0 - pe).abs() < 8.0 * 32.0 && (p.1 - pn).abs() < 30.0 * 32.0,
                    "{c} {p:?}"
                );
            }
        }
        // The bow catapults point ahead (a few degrees to port), the waist ones along the
        // angled deck.
        let h = |c: usize| (pose((0, 0), 1, c).heading as i16 as f64) / 65536.0 * 360.0;
        assert!((h(1)).abs() < 0.1 && (-6.0..-4.0).contains(&h(0)), "{} {}", h(0), h(1));
        assert!((-10.0..-8.0).contains(&h(2)) && (-10.0..-8.0).contains(&h(3)));
    }

    #[test]
    fn launch_state_round_trips() {
        let l = AiLaunch { cat: 3, phase: Phase::Stroke, frames: 77, carrier: 1234 };
        assert_eq!(AiLaunch::decode(l.encode()), Some(l));
        assert_eq!(AiLaunch::decode(0), None);
        assert_eq!(from_report(report(2, Phase::Shoot)), Some((2, Phase::Shoot)));
        assert_eq!(from_report(0), None);
    }
}
