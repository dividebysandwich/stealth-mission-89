//! The wingman on the ground with its lead: it starts on the runway beside him (or behind him
//! when the runway has no room for two, on the next catapult on a carrier), rolls and lifts off
//! with him, and lands with him: beside him when there is room, else behind him on his track.
//!
//! On the ground and on the approach it flies its lead's own track ([`WingPath`]), shifted to
//! his side or some way back along it, so it lands where he lands and stops short of him.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::catapult::Phase;
use crate::math::s16;
use crate::types::*;
use crate::wingman::{PendingWing, Wing, WingOrder, WingPhase};
use crate::{AircraftKind, Sim};

/// World units per metre.
const WU_PER_M: f64 = 32.0 / crate::aaa::M_PER_MAP;
/// The sim holds an aircraft on the ground this far (viewZ) over it.
const GEAR_Z: i32 = 12;
/// Beside its lead: its wingspan apart and this much more (m), and this far back (jet, m).
const SIDE_CLEAR_M: f64 = 8.0;
const SIDE_BACK_JET_M: f64 = 10.0;
/// Room kept to the runway's edge (m).
const EDGE_M: f64 = 4.0;
/// Behind its lead on his track (m): taking off, landing on a runway, landing on a carrier.
const TRAIL_TAKEOFF_M: f64 = 150.0;
const TRAIL_LANDING_M: f64 = 450.0;
/// Stopped this far (m, more than its length) behind its lead, braking at most this hard
/// (m/s²) on a runway and on a carrier's wires.
const STOP_GAP_M: f64 = 30.0;
const STOP_GAP_BIG_M: f64 = 90.0;
const BRAKE_RUNWAY: f64 = 3.0;
const BRAKE_DECK: f64 = 45.0;
const DECK_GAP_M: f64 = 25.0;
/// Its lead slower than this (m/s) on his landing roll: it brakes to a stop on its own.
const TAXI_MS: f64 = 15.0;
/// How long (s) it eases from where it was onto its place on the track.
const EASE_AIR_S: f64 = 5.0;
const EASE_GROUND_S: f64 = 1.0;
/// A jet lead lands from below this height (ft) with his gear down, lined up with a runway
/// within this far (m) of its end, or near a carrier.
const LANDING_FT: i32 = 2000;
const APPROACH_M: f64 = 12_000.0;
const CARRIER_APPROACH_M: f64 = 8000.0;
/// A helicopter lead lands below this height (ft) and speed (kt).
const HELI_LANDING_FT: i32 = 100;
const HELI_LANDING_KT: i32 = 40;
/// It goes back to flying formation once its lead is up again (ft, jet and helicopter).
const ABORT_JET_FT: i32 = 500;
const ABORT_HELI_FT: i32 = 200;
const ABORT_HELI_KT: i32 = 60;
/// Its lead up this high (ft): a wingman parked after the landing is done.
const LEFT_BEHIND_FT: i32 = 1000;

/// A point of its lead's track.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
struct TrackPoint {
    /// Track length to here (wu).
    s: f64,
    /// wu east, wu south.
    e: f64,
    n: f64,
    /// viewZ as the sim holds an aircraft there.
    alt: i32,
    heading: i32,
    pitch: i32,
    roll: i32,
    /// wu/s.
    speed: f64,
    ground: bool,
}

/// A wingman flying its lead's track (on the ground, taking off, landing).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WingPath {
    pub lead: u8,
    pts: VecDeque<TrackPoint>,
    /// The lead's track length so far, his last position (wu east, south) and speed (wu/s).
    len: f64,
    last: (f64, f64),
    lead_v: f64,
    /// The wingman's place on the track (wu) and speed along it (wu/s).
    s: f64,
    v: f64,
    /// wu to the right of the track (beside its lead); 0: behind him.
    shift: f64,
    /// How far back on the track it keeps in the air (wu), and stops short of him (wu).
    back: f64,
    gap: f64,
    /// Its braking (wu/s²).
    brake: f64,
    /// Where it was off its place when it took it up (wu east, south, viewZ), easing to 0.
    off: [f64; 3],
}

/// How a wingman starts with its lead on the ground.
enum Start {
    /// On the runway (or the ground) at his side (wu right; negative: left).
    Side(f64),
    /// Behind him on the runway (wu).
    Trail(f64),
    /// On a catapult of carrier object `b`.
    Catapult(usize),
}

/// The wingspan (m) of AI aircraft type `spec` (its rotor's for a helicopter).
fn span_m(spec: i32) -> f64 {
    use crate::ai_missions as t;
    match spec {
        t::F16 | t::AV8B => 10.0,
        t::A10 => 17.5,
        t::UH60 | t::MH60R | t::CH47 => 18.5,
        t::OH58 => 11.0,
        crate::munitions::C5 => 68.0,
        _ => 14.0,
    }
}

impl Sim {
    /// The ground's height (viewZ) at map point (`x`, `y`).
    pub(crate) fn ground_z(&self, x: i32, y: i32) -> i32 {
        self.tiles.as_ref().map_or(0, |t| t.relief.height(x, y).max(t.hills.ground_alt(x, y)))
    }

    fn ground_at(&self, e: f64, n: f64) -> i32 {
        self.ground_z(((e as i32) >> 5) & 0xFFFF, ((n as i32) >> 5) & 0xFFFF)
    }

    /// Flight unit `i` is a wingman flying its lead's track (the AI does not move it).
    pub(crate) fn wing_on_track(&self, i: usize) -> bool {
        self.campaign
            .as_ref()
            .and_then(|c| c.flights.get(i).copied().flatten())
            .and_then(|f| f.wing)
            .is_some_and(|w| w.phase.on_track())
    }

    /// The runway (metres: across from its centreline, half width; the lead's metres right
    /// along it, unit) under or ahead of a point (m east, north) within `ahead` m of its end,
    /// lined up with `heading` (radians) when `aligned`.
    fn runway_near(
        &self,
        e: f64,
        n: f64,
        heading: f64,
        ahead: f64,
        aligned: bool,
    ) -> Option<(f64, f64, (f64, f64))> {
        let fwd = (heading.sin(), heading.cos());
        self.runways().iter().find_map(|r| {
            let (dx, dy) = (e - r.c.0, n - r.c.1);
            let along = dx * r.axis.0 + dy * r.axis.1;
            let across = -dx * r.axis.1 + dy * r.axis.0;
            let dot = fwd.0 * r.axis.0 + fwd.1 * r.axis.1;
            if aligned {
                // Lined up with it, on the near side of its far end, its centreline ahead.
                let toward = along * dot.signum();
                let ok = dot.abs() > 0.87
                    && toward < r.half_len
                    && toward > -(r.half_len + ahead)
                    && across.abs() < 300.0 + (-toward - r.half_len).max(0.0) * 0.15;
                ok.then_some((across, r.half_width, (-r.axis.1, r.axis.0)))
            } else {
                (along.abs() <= r.half_len && across.abs() <= r.half_width).then_some((
                    across,
                    r.half_width,
                    (-r.axis.1, r.axis.0),
                ))
            }
        })
    }

    /// Beside a lead at `e`, `n` (m) heading `heading` on a runway (`rw`, from
    /// [`Sim::runway_near`]): how far to his right (m; negative left) a wingman of type `spec`
    /// fits, the right first; none when there is no room for two.
    fn side_room(
        &self,
        spec: i32,
        heading: f64,
        rw: Option<(f64, f64, (f64, f64))>,
    ) -> Option<f64> {
        let span = span_m(spec);
        let sep = span * 1.5 + SIDE_CLEAR_M * 0.5;
        let Some((across, half, left)) = rw else { return Some(sep) };
        // His right in metres east, north; and how far along the runway's left that is.
        let right = (heading.cos(), -heading.sin());
        let r_left = right.0 * left.0 + right.1 * left.1;
        [sep, -sep].into_iter().find(|&d| (across + d * r_left).abs() + span / 2.0 <= half - EDGE_M)
    }

    /// Where a wingman of pilot `lead` starts while he is on the ground: beside him on his
    /// runway, else behind him, a jet on a carrier on a catapult. None: not on a runway (it
    /// joins him once he is up).
    fn ground_start(&self, lead: u8, spec: i32) -> Option<Start> {
        let pilot = self.pilot_ref(lead as usize);
        if !pilot.on_ground || pilot.p.knots > 30 {
            return None;
        }
        let heli = self.aircraft_kind(spec) == AircraftKind::Helicopter;
        let carrier = self.carrier_near(pilot.p.mx, pilot.p.my, 400.0);
        if let Some(b) =
            carrier.filter(|_| !heli && self.ac_type(spec).model != crate::ai::HARRIER_MODEL)
        {
            return Some(Start::Catapult(b));
        }
        let (e, n) = Self::pilot_metres(&pilot.p);
        let h = (pilot.p.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let rw = self.runway_near(e, n, h, 0.0, false);
        if heli || carrier.is_some() {
            return Some(Start::Side(span_m(spec) * 1.5 * WU_PER_M));
        }
        if rw.is_none() && !self.runways().is_empty() {
            return None;
        }
        Some(match self.side_room(spec, h, rw) {
            Some(d) => Start::Side(d * WU_PER_M),
            None => Start::Trail(TRAIL_TAKEOFF_M.max(span_m(spec) * 2.0) * WU_PER_M),
        })
    }

    /// A pilot's position in metres east, north.
    fn pilot_metres(p: &crate::PlayerView) -> (f64, f64) {
        let m = crate::aaa::M_PER_MAP / 32.0;
        (p.wx as f64 * m, p.wy as f64 * m)
    }

    /// The carrier (object) within `m` metres of map point (`x`, `y`).
    fn carrier_near(&self, x: i32, y: i32, m: f64) -> Option<usize> {
        let r = (m / crate::aaa::M_PER_MAP) as i32;
        (1..self.objects.len()).find(|&b| {
            let o = &self.objects[b];
            o.flags & oflags::CARRIER != 0
                && o.flags & oflags::DESTROYED == 0
                && crate::math::approx(s16(o.x - x), s16(o.y - y)) < r
        })
    }

    /// Wingman `p` starts with its lead on the ground (true), or waits for him to get up.
    pub(crate) fn spawn_wingman_on_ground(&mut self, p: &PendingWing) -> bool {
        let Some(start) = self.ground_start(p.lead, p.spec) else { return false };
        let i = self.spawn_wingman_at(p.clone(), None);
        let u = &mut self.flights[i];
        u.speed = 0;
        u.pitch = 0;
        u.bank = 0;
        u.flags = fflags::EXISTS | fflags::AIRBORNE | fflags::FINAL;
        match start {
            Start::Catapult(b) => {
                self.spawn(i, b);
                self.flights[i].flags &= !fflags::FINAL;
                self.flights[i].wp = p.home;
                self.set_wing_phase(i, WingPhase::Catapult);
            }
            Start::Side(shift) => self.start_track(i, p.lead, WingPhase::Takeoff, shift, 0.0),
            Start::Trail(back) => self.start_track(i, p.lead, WingPhase::Takeoff, 0.0, back),
        }
        true
    }

    fn set_wing_phase(&mut self, i: usize, phase: WingPhase) {
        if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i])
            && let Some(w) = f.wing.as_mut()
        {
            w.phase = phase;
        }
    }

    /// Flight unit `i`, pilot `lead`'s wingman, takes up its place on his track from here on:
    /// `shift` wu to his right (beside him), else `back` wu behind him.
    fn start_track(&mut self, i: usize, lead: u8, phase: WingPhase, shift: f64, back: f64) {
        let spec = self.flights[i].spec;
        let heli = self.aircraft_kind(spec) == AircraftKind::Helicopter;
        let carrier = {
            let p = &self.pilot_ref(lead as usize).p;
            self.carrier_near(p.mx, p.my, CARRIER_APPROACH_M).is_some()
        };
        let big = span_m(spec) > 40.0;
        let back =
            if shift != 0.0 { if heli { 0.0 } else { SIDE_BACK_JET_M * WU_PER_M } } else { back };
        let gap = if carrier {
            DECK_GAP_M
        } else if big {
            STOP_GAP_BIG_M
        } else {
            STOP_GAP_M
        } * WU_PER_M;
        let brake = if carrier { BRAKE_DECK } else { BRAKE_RUNWAY } * WU_PER_M;
        let l = self.lead_point(lead);
        let mut path = WingPath {
            lead,
            pts: VecDeque::from([TrackPoint { s: 0.0, ..l }]),
            len: 0.0,
            last: (l.e, l.n),
            lead_v: l.speed,
            s: -back,
            v: if phase == WingPhase::Landing { self.flights[i].speed as f64 } else { 0.0 },
            shift,
            back,
            gap,
            brake,
            off: [0.0; 3],
        };
        if phase == WingPhase::Landing {
            // From where it is now, easing onto its place.
            let u = self.flights[i];
            let (e, n, z, ..) = self.track_place(&path, path.s);
            path.off = [u.wx as f64 - e, u.wy as f64 - n, (u.alt - z) as f64];
        }
        self.wing_paths.retain(|w| w.lead != lead);
        self.wing_paths.push(path);
        self.set_wing_phase(i, phase);
        if phase != WingPhase::Landing {
            self.fly_track(i);
        }
    }

    /// Pilot `lead` as a point of his track.
    fn lead_point(&self, lead: u8) -> TrackPoint {
        let pilot = self.pilot_ref(lead as usize);
        let p = &pilot.p;
        let (e, n) = (p.wx as f64, (0x10_0000 - p.wy) as f64);
        let ground = pilot.on_ground;
        let gz = self.ground_at(e, n);
        // On the ground (not a deck): the sim's ground height for an aircraft there.
        let alt = if ground && p.view_z < gz + 30 { gz + GEAR_Z } else { p.view_z + GEAR_Z };
        TrackPoint {
            s: 0.0,
            e,
            n,
            alt,
            heading: s16(p.heading),
            pitch: if ground { 0 } else { p.pitch },
            roll: if ground { 0 } else { p.roll },
            speed: 0.0,
            ground,
        }
    }

    /// The point of a track at `s` (before its start: back along its first heading): wu east,
    /// south, viewZ, heading, pitch, roll, speed, on the ground.
    fn track_point(path: &WingPath, s: f64) -> TrackPoint {
        let first = path.pts.front().copied().unwrap_or_default();
        if s <= first.s {
            let h = (first.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
            let d = first.s - s;
            return TrackPoint { e: first.e - h.sin() * d, n: first.n + h.cos() * d, ..first };
        }
        let k = path.pts.partition_point(|p| p.s <= s);
        let (a, b) = (path.pts[k - 1], path.pts.get(k).copied().unwrap_or(path.pts[k - 1]));
        let t = if b.s > a.s { (s - a.s) / (b.s - a.s) } else { 0.0 };
        let lerp = |x: f64, y: f64| x + (y - x) * t;
        let turn = |x: i32, y: i32| s16(x + (s16(y - x) as f64 * t).round() as i32);
        TrackPoint {
            s,
            e: lerp(a.e, b.e),
            n: lerp(a.n, b.n),
            alt: lerp(a.alt as f64, b.alt as f64).round() as i32,
            heading: turn(a.heading, b.heading),
            pitch: turn(a.pitch, b.pitch),
            roll: turn(a.roll, b.roll),
            speed: lerp(a.speed, b.speed),
            ground: if t < 0.5 { a.ground } else { b.ground },
        }
    }

    /// The wingman's place at `s` on its track: wu east, south, viewZ (over the ground
    /// there as its lead was there), and the track point.
    fn track_place(&self, path: &WingPath, s: f64) -> (f64, f64, i32, TrackPoint) {
        let p = Self::track_point(path, s);
        let h = (p.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        // Right is east cos h, south sin h.
        let (e, n) = (p.e + h.cos() * path.shift, p.n + h.sin() * path.shift);
        let (g0, g1) = (self.ground_at(p.e, p.n), self.ground_at(e, n));
        let z = (p.alt + g1 - g0).max(g1 + GEAR_Z);
        (e, n, z, p)
    }

    /// One frame of a wingman on its lead's track: his track drawn on, its place on it, and
    /// its phase done.
    pub(crate) fn fly_track(&mut self, i: usize) {
        let Some(w) = self.campaign.as_ref().and_then(|c| c.flights[i]).and_then(|f| f.wing) else {
            return;
        };
        let Some(k) = self.wing_paths.iter().position(|p| p.lead == w.lead) else {
            // No track (an old save): it flies on.
            self.leave_track(i, WingPhase::Air);
            return;
        };
        let fs = self.fs.max(1) as f64;
        let dt = 1.0 / fs;
        let pilot = self.pilot_ref(w.lead as usize);
        let (on_ground, height, knots, gear_up) =
            (pilot.on_ground, pilot.p.height, pilot.p.knots, pilot.pflags & pflags::GEAR_UP != 0);
        let mut l = self.lead_point(w.lead);
        let mut path = std::mem::take(&mut self.wing_paths[k]);
        // His track drawn on (a jump: drawn anew from here).
        let d = (l.e - path.last.0).hypot(l.n - path.last.1);
        if d > 2000.0 * WU_PER_M {
            path.pts.clear();
            path.len = 0.0;
            path.s = -path.back;
            path.lead_v = 0.0;
        }
        path.lead_v += (d.min(1000.0 * WU_PER_M * dt) * fs - path.lead_v) * 0.3;
        path.last = (l.e, l.n);
        l.speed = path.lead_v;
        let moved = path.pts.back().map_or(f64::MAX, |b| (l.e - b.e).hypot(l.n - b.n));
        if moved >= 0.5 || path.pts.is_empty() {
            path.len += if path.pts.is_empty() { 0.0 } else { moved };
            path.pts.push_back(TrackPoint { s: path.len, ..l });
        } else if let Some(b) = path.pts.back_mut() {
            *b = TrackPoint { s: b.s, e: b.e, n: b.n, ..l };
        }
        // Its place on the track.
        let phase = w.phase;
        let fast_lead = !on_ground || path.lead_v > TAXI_MS * WU_PER_M;
        match phase {
            WingPhase::Takeoff => {
                path.s = path.len - path.back;
                path.v = path.lead_v;
            }
            WingPhase::Landing if path.shift != 0.0 => {
                if fast_lead {
                    path.s = path.len - path.back;
                    path.v = path.lead_v;
                } else {
                    // His landing roll near its end: it brakes to a stop on its own.
                    path.v = (path.v - path.brake * dt).max(0.0);
                    path.s = (path.s + path.v * dt).min(path.len - path.back);
                }
            }
            WingPhase::Landing => {
                // Behind him: as fast as he flew here, closing up on its distance; on the
                // ground it stops short of him.
                let at = Self::track_point(&path, path.s);
                let vs = if path.s <= 0.0 { path.lead_v } else { at.speed };
                let keep = if on_ground { path.gap } else { path.back };
                let err = path.len - keep - path.s;
                let mut v = (vs + 0.25 * err).clamp(0.8 * vs, 1.25 * vs + 20.0);
                let room = (path.len - path.gap - path.s).max(0.0);
                v = v.min((2.0 * path.brake * room).sqrt()).max(0.0);
                path.v = v;
                path.s += v * dt;
            }
            _ => path.v = 0.0,
        }
        let (e, n, z, p) = self.track_place(&path, path.s);
        let ease = if p.ground { EASE_GROUND_S } else { EASE_AIR_S };
        for o in path.off.iter_mut() {
            *o *= 1.0 - (dt / ease).min(1.0);
        }
        let (e, n) = (e + path.off[0], n + path.off[1]);
        let g = self.ground_at(e, n);
        let z = (z + path.off[2].round() as i32).max(g + GEAR_Z);
        // The track behind it kept for its replay (a little).
        while path.pts.len() > 2 && path.pts[1].s < path.s - 50.0 * WU_PER_M {
            path.pts.pop_front();
        }
        let (s, v, ground) = (path.s, path.v, p.ground);
        let behind = path.shift == 0.0;
        self.wing_paths[k] = path;
        let was = self.flights[i];
        let heli = self.aircraft_kind(was.spec) == AircraftKind::Helicopter;
        let u = &mut self.flights[i];
        u.wx = e.round() as i32;
        u.wy = n.round() as i32;
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.alt = z;
        u.heading = p.heading;
        u.pitch = p.pitch;
        u.bank = p.roll;
        u.speed = v.round() as i32;
        u.flags |= fflags::EXISTS | fflags::AIRBORNE;
        // Its gear as its lead's.
        if gear_up && !ground {
            u.flags &= !fflags::FINAL;
        } else {
            u.flags |= fflags::FINAL;
        }
        if let Some(fv) = self.flight_vel.get_mut(i) {
            *fv = [
                (u.wx - was.wx) as f32 * fs as f32,
                (u.wy - was.wy) as f32 * fs as f32,
                (u.alt - was.alt) as f32 * fs as f32,
            ];
        }
        // Its phase done.
        let up = !ground && z > g + if heli { 30 } else { 100 };
        match phase {
            WingPhase::Takeoff => {
                let join = if heli { 40 } else { 300 };
                if !on_ground && height >= join && up {
                    self.leave_track(i, WingPhase::Air);
                }
            }
            WingPhase::Landing => {
                let again = if heli {
                    height > ABORT_HELI_FT || knots > ABORT_HELI_KT
                } else {
                    (gear_up && height > ABORT_JET_FT) || height > LANDING_FT + 500
                };
                if !on_ground && again && up {
                    self.leave_track(i, WingPhase::Air);
                } else if !heli && on_ground && ground && v < 0.5 && (!behind || s > 0.0) {
                    self.flights[i].speed = 0;
                    self.set_wing_phase(i, WingPhase::Parked);
                }
            }
            WingPhase::Parked if !on_ground && height > LEFT_BEHIND_FT => self.wing_landed(i),
            _ => {}
        }
    }

    /// The wingman leaves its lead's track and flies on (`phase`: in the air).
    pub(crate) fn leave_track(&mut self, i: usize, phase: WingPhase) {
        let lead =
            self.campaign.as_ref().and_then(|c| c.flights[i]).and_then(|f| f.wing).map(|w| w.lead);
        if let Some(lead) = lead {
            self.wing_paths.retain(|p| p.lead != lead);
        }
        let u = &mut self.flights[i];
        u.flags &= !(fflags::FINAL | fflags::TAKEOFF | fflags::ROLLOUT | fflags::RTB);
        u.flags |= fflags::EXISTS | fflags::AIRBORNE;
        u.speed = u.speed.max(1);
        if self.ai_flight.len() > i {
            self.ai_flight[i] = Default::default();
        }
        self.set_wing_phase(i, phase);
    }

    /// The wingman is down for good (stopped after the landing, its lead gone): it counts as
    /// landed and leaves the world.
    pub(crate) fn wing_landed(&mut self, i: usize) {
        let lead =
            self.campaign.as_ref().and_then(|c| c.flights[i]).and_then(|f| f.wing).map(|w| w.lead);
        if let Some(lead) = lead {
            self.wing_paths.retain(|p| p.lead != lead);
        }
        let u = &mut self.flights[i];
        u.flags |= fflags::ROLLOUT;
        u.speed = 0;
    }

    /// Its lead coming in to land (gear down, low, lined up with a runway or near a carrier;
    /// a helicopter low and slow): a wingman in formation lands with him.
    pub(crate) fn wing_lands(&mut self, i: usize, w: &Wing) -> bool {
        if !matches!(w.order, WingOrder::Formation | WingOrder::Cover) {
            return false;
        }
        let u = self.flights[i];
        if u.flags & fflags::TAKEOFF != 0 {
            return false;
        }
        let pilot = self.pilot_ref(w.lead as usize);
        if pilot.on_ground {
            return false;
        }
        let p = pilot.p;
        let heli = self.aircraft_kind(u.spec) == AircraftKind::Helicopter;
        // Coming down (not just up).
        if p.climb >= 0 {
            return false;
        }
        if heli {
            if p.height >= HELI_LANDING_FT || p.knots >= HELI_LANDING_KT {
                return false;
            }
            let shift = span_m(u.spec) * 1.5 * WU_PER_M;
            self.start_track(i, w.lead, WingPhase::Landing, shift, 0.0);
            return true;
        }
        if pilot.pflags & pflags::GEAR_UP != 0 || p.height >= LANDING_FT {
            return false;
        }
        let (e, n) = Self::pilot_metres(&p);
        let h = (p.heading as u16) as f64 * std::f64::consts::TAU / 65536.0;
        let trail = |m: f64| (m.max(span_m(u.spec) * 6.0)) * WU_PER_M;
        if let Some(rw) = self.runway_near(e, n, h, APPROACH_M, true) {
            match self.side_room(u.spec, h, Some(rw)) {
                Some(d) => self.start_track(i, w.lead, WingPhase::Landing, d * WU_PER_M, 0.0),
                None => {
                    self.start_track(i, w.lead, WingPhase::Landing, 0.0, trail(TRAIL_LANDING_M))
                }
            }
            return true;
        }
        if self.carrier_near(p.mx, p.my, CARRIER_APPROACH_M).is_some() {
            self.start_track(i, w.lead, WingPhase::Landing, 0.0, trail(TRAIL_LANDING_M));
            return true;
        }
        false
    }

    /// A wingman on a carrier's catapult holds until its lead is launched.
    pub(crate) fn wing_holds_launch(&self, i: usize) -> bool {
        let Some(w) = self
            .campaign
            .as_ref()
            .and_then(|c| c.flights.get(i).copied().flatten())
            .and_then(|f| f.wing)
        else {
            return false;
        };
        let pilot = self.pilot_ref(w.lead as usize);
        if pilot.id != w.lead || pilot.absent || pilot.eject != 0 || !pilot.on_ground {
            return false;
        }
        crate::catapult::from_report(pilot.last_input.catapult)
            .is_none_or(|(_, phase)| matches!(phase, Phase::Wait | Phase::Tension))
    }
}
