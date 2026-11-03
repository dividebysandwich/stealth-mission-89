//! The air-to-air radar (`avionics::Format::Fcr`, after the F-16's fire control radar in range
//! while search): a beam sweeping left and right across the scan width, a row (bar) higher or
//! lower each way, paints the aircraft it sees as bricks on a B-scope (azimuth across, range up),
//! kept for two frames of the scan, dimmer the second.
//!
//! - The scan is stabilised: its azimuth goes from the nose's heading, its elevation from the
//!   horizon (`Scan::elevation`, the antenna knob, the bars stacked round it).
//! - An aircraft is seen within its detection range (a fighter's about 40 nm, a drone's much
//!   less; the edge fluctuates from pass to pass), with the ground not in the way, and not in
//!   the Doppler notch: looked down on against the ground, one flying across the beam (little
//!   speed towards or away from the radar) hides in the clutter.
//! - A lock on an aircraft the radar can follow (in its gimbals, seen) stops the scan: the
//!   antenna tracks it (single target track) and gives its aspect, heading, speed and closure.
//!
//! It transmits while its format is up and not in standby (`AvionicsState::fcr_on`); the
//! simulation hears it as it does the ground map (`PlayerState::radar`).

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::flight::{Aircraft, Player};
use crate::ground_radar::{Ground, TheaterGround, angle_diff, hash, unit};
use crate::mfd::{ContactKind, ContactRef, Contacts, MfdState};
use crate::world::{WU, WorldPos};

pub struct AaRadarPlugin;

impl Plugin for AaRadarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AaRadar>()
            .add_systems(OnEnter(crate::settings::AppState::Flight), reset)
            .add_systems(
                Update,
                scan.after(crate::combat::CombatSystems)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            );
    }
}

/// The antenna's gimbal limits either side of the nose and over and under the horizon
/// (radians): the B-scope's width.
pub const GIMBAL: f32 = 60.0 * std::f32::consts::PI / 180.0;
/// Bars apart (radians) and the beam's half width.
pub const BAR_SPACING: f32 = 2.2 * std::f32::consts::PI / 180.0;
pub const BEAM_HALF: f32 = 1.75 * std::f32::consts::PI / 180.0;
/// The scan rate (radians per second): a 4-bar scan across A6 takes about 7 s.
const SCAN_RATE: f32 = 70.0 * std::f32::consts::PI / 180.0;
/// Detection range (metres) of a fighter, and of an attack drone.
const FIGHTER_M: f64 = 74_000.0;
const DRONE_M: f64 = 18_000.0;
/// The detection range's edge from pass to pass: always seen inside this fraction of it,
/// never beyond `1 / FADE_IN` of it.
const FADE_IN: f64 = 0.8;
/// A track holds out to this many detection ranges.
const TRACK_RANGE: f64 = 1.2;
/// Speeds (m/s) towards or away from the radar under which a target looked down on is lost in
/// the ground clutter.
const NOTCH: f64 = 30.0;
/// The sight line to a target is checked against the ground every this many metres.
const SIGHT_STEP: f64 = 250.0;
/// Seconds over which the targets' velocities are smoothed (from their positions).
const VEL_TAU: f64 = 0.5;
/// Frames of the scan a brick is kept.
const KEEP_FRAMES: f32 = 2.0;

/// The scan the pilot set: its half width, bars and the antenna's elevation (radians over the
/// horizon).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scan {
    pub az_half: f32,
    pub bars: u8,
    pub elevation: f32,
}

impl Scan {
    /// Seconds a full frame (every bar once) takes.
    pub fn frame(self) -> f32 {
        self.bars.max(1) as f32 * 2.0 * self.az_half / SCAN_RATE
    }

    /// A bar's elevation (bar 0 the top one).
    pub fn bar_elevation(self, bar: u8) -> f32 {
        let n = self.bars.max(1) as f32;
        self.elevation + ((n - 1.0) / 2.0 - bar as f32) * BAR_SPACING
    }

    /// The elevations the whole scan covers (lowest, highest).
    pub fn coverage(self) -> (f32, f32) {
        let n = self.bars.max(1) - 1;
        (self.bar_elevation(n) - BEAM_HALF, self.bar_elevation(0) + BEAM_HALF)
    }
}

/// An aircraft the radar may see.
#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub target: ContactRef,
    /// Metres east, north and up.
    pub pos: DVec3,
    /// Its heading (radians clockwise from north).
    pub heading: f32,
    pub friendly: bool,
    /// Its detection range (metres).
    pub detect: f64,
}

/// A search return on the B-scope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brick {
    /// Azimuth from the nose (radians, right positive) and range (metres) when it was painted.
    pub az: f32,
    pub range: f32,
    pub target: ContactRef,
    pub friendly: bool,
    /// Seconds since it was painted.
    pub age: f32,
}

/// The locked aircraft the antenna follows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track {
    pub target: ContactRef,
    pub az: f32,
    pub el: f32,
    pub range: f32,
    /// Altitude (metres).
    pub alt: f32,
    /// Its heading off the nose's (radians, right positive), for the B-scope's symbol.
    pub rel_heading: f32,
    pub heading: f32,
    /// Ground speed and closure (m/s, closing positive).
    pub speed: f32,
    pub closure: f32,
    /// Aspect angle (radians, 0 from its tail, pi head on) and the side of its nose we are on.
    pub aspect: f32,
    pub right: bool,
}

/// A target's position and smoothed velocity (metres, m/s).
#[derive(Debug, Clone, Copy)]
struct Motion {
    target: ContactRef,
    pos: DVec3,
    vel: DVec3,
}

#[derive(Resource, Debug, Clone)]
pub struct AaRadar {
    /// Transmitting.
    pub on: bool,
    /// The beam's azimuth from the nose (radians, right positive), the way it moves (+1 right)
    /// and the bar it scans.
    pub beam: f32,
    dir: f32,
    pub bar: u8,
    /// The beam's elevation over the horizon (radians).
    pub elevation: f32,
    pub bricks: Vec<Brick>,
    pub track: Option<Track>,
    /// The frame's length (seconds), for the bricks' age.
    pub frame: f32,
    /// Passes so far (a pass is one way across).
    passes: u32,
    motions: Vec<Motion>,
    own: Option<Motion>,
}

impl Default for AaRadar {
    fn default() -> Self {
        Self {
            on: false,
            beam: 0.0,
            dir: 1.0,
            bar: 0,
            elevation: 0.0,
            bricks: Vec::new(),
            track: None,
            frame: 1.0,
            passes: 0,
            motions: Vec::new(),
            own: None,
        }
    }
}

fn reset(mut radar: ResMut<AaRadar>) {
    *radar = default();
}

/// Where a target is from the aircraft: azimuth off the nose, elevation over the horizon and
/// range.
fn geometry(own: DVec3, heading: f32, pos: DVec3) -> (f32, f32, f64) {
    let d = pos - own;
    let ground = d.x.hypot(d.y);
    let bearing = d.x.atan2(d.y) as f32;
    (angle_diff(bearing, heading), d.z.atan2(ground) as f32, d.length())
}

fn seed(t: ContactRef) -> u32 {
    match t {
        ContactRef::Unit(i) => i as u32,
        ContactRef::Pilot(k) => 0x1_0000 + k as u32,
        ContactRef::Object(i) => 0x2_0000 + i as u32,
        ContactRef::Point(x, y) => (x as u32) ^ (y as u32).rotate_left(16),
    }
}

impl AaRadar {
    /// Off: nothing painted, nothing tracked.
    fn clear(&mut self) {
        self.on = false;
        self.bricks.clear();
        self.track = None;
        self.motions.clear();
        self.own = None;
    }

    /// The smoothed velocity of a position seen again after `dt` seconds.
    fn follow(m: Option<Motion>, target: ContactRef, pos: DVec3, dt: f64) -> Motion {
        match m {
            Some(m) if dt > 0.0 && m.pos.distance(pos) < 2000.0 * dt.max(0.05) => {
                let k = 1.0 - (-dt / VEL_TAU).exp();
                Motion { target, pos, vel: m.vel + ((pos - m.pos) / dt - m.vel) * k }
            }
            Some(m) => Motion { pos, ..m },
            None => Motion { target, pos, vel: DVec3::ZERO },
        }
    }

    /// A frame: the aircraft at `own` (metres east, north, up) heading `heading`, scanning
    /// `scan`, the simulation's air lock `locked`; `sight` tells whether the ground leaves the
    /// line between two points open.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        dt: f32,
        own: DVec3,
        heading: f32,
        scan: Scan,
        targets: &[Target],
        locked: Option<ContactRef>,
        sight: impl Fn(DVec3, DVec3) -> bool,
    ) {
        self.on = true;
        let dt64 = dt as f64;
        self.own = Some(Self::follow(self.own, ContactRef::Pilot(u8::MAX), own, dt64));
        let own_vel = self.own.map_or(DVec3::ZERO, |m| m.vel);
        self.motions = targets
            .iter()
            .map(|t| {
                let m = self.motions.iter().find(|m| m.target == t.target).copied();
                Self::follow(m, t.target, t.pos, dt64)
            })
            .collect();
        self.frame = scan.frame();
        for b in &mut self.bricks {
            b.age += dt;
        }
        let keep = KEEP_FRAMES * self.frame;
        self.bricks.retain(|b| b.age < keep);
        let motions = &self.motions;
        let vel = |t: &Target| {
            motions.iter().find(|m| m.target == t.target).map_or(DVec3::ZERO, |m| m.vel)
        };
        // Seen out to `reach` detection ranges: the ground clear and out of the notch.
        let seen = |t: &Target, el: f32, range: f64, reach: f64| {
            if range > t.detect * reach {
                return false;
            }
            let los = (t.pos - own) / range.max(1.0);
            if el < 0.0 && vel(t).dot(los).abs() < NOTCH {
                return false;
            }
            sight(own, t.pos)
        };
        // The lock: the antenna on it while it is in the gimbals and seen.
        self.track = locked.and_then(|l| targets.iter().find(|t| t.target == l)).and_then(|t| {
            let (az, el, range) = geometry(own, heading, t.pos);
            if az.abs() > GIMBAL || el.abs() > GIMBAL || !seen(t, el, range, TRACK_RANGE) {
                return None;
            }
            let v = vel(t);
            let d = (t.pos - own) / range.max(1.0);
            let to_us = angle_diff((own.x - t.pos.x).atan2(own.y - t.pos.y) as f32, t.heading);
            Some(Track {
                target: t.target,
                az,
                el,
                range: range as f32,
                alt: t.pos.z as f32,
                rel_heading: angle_diff(t.heading, heading),
                heading: t.heading,
                speed: v.x.hypot(v.y) as f32,
                closure: -(v - own_vel).dot(d) as f32,
                aspect: std::f32::consts::PI - to_us.abs(),
                right: to_us >= 0.0,
            })
        });
        if let Some(t) = self.track {
            self.beam = t.az;
            self.elevation = t.el;
            self.bricks.retain(|b| b.target != t.target);
            return;
        }
        // The search: the beam's sweep this frame, turning at the scan's edges onto the next
        // bar.
        let half = scan.az_half;
        let bars = scan.bars.max(1);
        self.bar %= bars;
        self.beam = self.beam.clamp(-half, half);
        let mut spans = Vec::with_capacity(2);
        let mut left = SCAN_RATE * dt;
        while left > 0.0 && spans.len() < 8 {
            let edge = half * self.dir;
            let room = (edge - self.beam).abs();
            let go = left.min(room);
            let next = self.beam + go * self.dir;
            spans.push((self.beam.min(next), self.beam.max(next), self.bar, self.passes));
            self.beam = next;
            left -= go;
            if go >= room {
                self.dir = -self.dir;
                self.bar = (self.bar + 1) % bars;
                self.passes = self.passes.wrapping_add(1);
            }
        }
        self.elevation = scan.bar_elevation(self.bar);
        for t in targets {
            let (az, el, range) = geometry(own, heading, t.pos);
            let Some(&(_, _, _, pass)) = spans.iter().find(|&&(lo, hi, bar, _)| {
                az >= lo - BEAM_HALF
                    && az <= hi + BEAM_HALF
                    && (el - scan.bar_elevation(bar)).abs() <= BEAM_HALF
            }) else {
                continue;
            };
            let edge =
                FADE_IN + (1.0 / FADE_IN - FADE_IN) * unit(hash(seed(t.target), pass, 7)) as f64;
            if !seen(t, el, range, edge) {
                continue;
            }
            self.bricks.retain(|b| b.target != t.target);
            self.bricks.push(Brick {
                az,
                range: range as f32,
                target: t.target,
                friendly: t.friendly,
                age: 0.0,
            });
        }
    }
}

/// The aircraft the radar may see: those on the MFD contacts (drones closer in).
fn targets(contacts: &Contacts) -> Vec<Target> {
    contacts
        .list
        .iter()
        .filter(|k| matches!(k.kind, ContactKind::Aircraft | ContactKind::Drone))
        .filter_map(|k| {
            Some(Target {
                target: k.target?,
                pos: DVec3::new(k.east * WU, k.north * WU, k.alt as f64 * WU),
                heading: (k.heading as f64 / 65536.0 * std::f64::consts::TAU) as f32,
                friendly: k.friendly,
                detect: if k.kind == ContactKind::Drone { DRONE_M } else { FIGHTER_M },
            })
        })
        .collect()
}

/// The ground leaves the line from `a` to `b` (metres east, north, up) open.
fn sight_open(ground: &impl Ground, a: DVec3, b: DVec3) -> bool {
    let n = ((a.distance(b) / SIGHT_STEP) as usize).min(600);
    (1..n).all(|i| {
        let p = a.lerp(b, i as f64 / n as f64);
        p.z > ground.height(p.x, p.y) as f64
    })
}

#[allow(clippy::too_many_arguments)]
fn scan(
    time: Res<Time>,
    state: Res<MfdState>,
    contacts: Res<Contacts>,
    mut radar: ResMut<AaRadar>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    combat: Option<Res<crate::combat::Combat>>,
    theater: Option<Res<crate::world::Theater>>,
) {
    let flying = aircraft.single().ok().filter(|(a, _)| !a.on_ground && a.ended.is_none());
    let (Some((a, pos)), Some(combat), true) = (flying, combat, state.modern && state.av.fcr_on())
    else {
        if radar.on {
            radar.clear();
        }
        return;
    };
    let own = DVec3::new(pos.0.x, -pos.0.z, pos.0.y);
    let hills = theater.as_ref().map(|t| t.hills.clone()).unwrap_or_default();
    let relief = combat.sim.relief();
    let ground = TheaterGround { hills: &hills, relief: &relief, scenery: None };
    let locked =
        contacts.locked.filter(|l| matches!(l, ContactRef::Unit(_) | ContactRef::Pilot(_)));
    let found = targets(&contacts);
    let scan = state.av.fcr_scan();
    radar.step(time.delta_secs(), own, a.euler().0, scan, &found, locked, |a, b| {
        sight_open(&ground, a, b)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const A6_4B: Scan = Scan { az_half: GIMBAL, bars: 4, elevation: 0.0 };

    fn fighter(i: usize, east: f64, north: f64, up: f64, heading: f32) -> Target {
        Target {
            target: ContactRef::Unit(i),
            pos: DVec3::new(east, north, up),
            heading,
            friendly: false,
            detect: FIGHTER_M,
        }
    }

    /// Runs the radar `secs` seconds at 60 Hz, the aircraft at 3000 m heading north, the
    /// targets flying on at their headings at `speed` m/s.
    fn run(
        radar: &mut AaRadar,
        secs: f32,
        scan: Scan,
        targets: &mut [Target],
        speed: f64,
        locked: Option<ContactRef>,
        sight: impl Fn(DVec3, DVec3) -> bool + Copy,
    ) {
        let dt = 1.0 / 60.0;
        for _ in 0..(secs / dt) as usize {
            for t in targets.iter_mut() {
                let (s, c) = (t.heading as f64).sin_cos();
                t.pos += DVec3::new(s, c, 0.0) * speed * dt as f64;
            }
            radar.step(dt, DVec3::new(0.0, 0.0, 3000.0), 0.0, scan, targets, locked, sight);
        }
    }

    #[test]
    fn a_fighter_ahead_is_painted_in_a_frame() {
        let mut radar = AaRadar::default();
        let mut t = [fighter(1, 5_000.0, 30_000.0, 3500.0, 3.0)];
        run(&mut radar, A6_4B.frame() + 0.2, A6_4B, &mut t, 200.0, None, |_, _| true);
        let b = radar.bricks.iter().find(|b| b.target == ContactRef::Unit(1)).expect("painted");
        assert!((b.range - 30_400.0).abs() < 1500.0, "{b:?}");
        assert!(b.az > 0.1 && b.az < 0.25, "{b:?}");
    }

    #[test]
    fn out_of_range_behind_or_hidden_targets_are_not_painted() {
        let mut radar = AaRadar::default();
        let mut t = [
            fighter(1, 0.0, 120_000.0, 3000.0, 3.0),
            fighter(2, 0.0, -20_000.0, 3000.0, 0.0),
            fighter(3, 0.0, 20_000.0, 3000.0, 3.0),
        ];
        // A ridge hides the one ahead.
        let hidden = |_: DVec3, b: DVec3| !(b.y > 15_000.0 && b.y < 25_000.0);
        run(&mut radar, 2.0 * A6_4B.frame(), A6_4B, &mut t, 200.0, None, hidden);
        assert!(radar.bricks.is_empty(), "{:?}", radar.bricks);
    }

    #[test]
    fn a_beaming_target_below_is_lost_in_the_notch() {
        let mut radar = AaRadar::default();
        // Low ahead: one flying across the beam, one coming head on.
        let mut t = [
            fighter(1, 0.0, 25_000.0, 500.0, std::f32::consts::FRAC_PI_2),
            fighter(2, 3_000.0, 25_000.0, 500.0, std::f32::consts::PI),
        ];
        let low = Scan { elevation: -6f32.to_radians(), ..A6_4B };
        run(&mut radar, 2.0 * low.frame(), low, &mut t, 220.0, None, |_, _| true);
        assert!(!radar.bricks.iter().any(|b| b.target == ContactRef::Unit(1)));
        assert!(radar.bricks.iter().any(|b| b.target == ContactRef::Unit(2)));
    }

    #[test]
    fn a_target_above_the_bars_is_not_painted_until_the_antenna_looks_up() {
        let mut radar = AaRadar::default();
        // 20 degrees up.
        let mut t = [fighter(1, 0.0, 20_000.0, 3000.0 + 7_300.0, 3.0)];
        run(&mut radar, 2.0 * A6_4B.frame(), A6_4B, &mut t, 0.0, None, |_, _| true);
        assert!(radar.bricks.is_empty());
        let up = Scan { elevation: 20f32.to_radians(), ..A6_4B };
        run(&mut radar, up.frame() + 0.2, up, &mut t, 0.0, None, |_, _| true);
        assert_eq!(radar.bricks.len(), 1);
    }

    #[test]
    fn a_lock_tracks_with_aspect_and_closure() {
        let mut radar = AaRadar::default();
        // Ahead, coming straight at us.
        let mut t = [fighter(1, 0.0, 30_000.0, 3000.0, std::f32::consts::PI)];
        run(&mut radar, 2.0, A6_4B, &mut t, 250.0, Some(ContactRef::Unit(1)), |_, _| true);
        let tr = radar.track.expect("tracked");
        assert!(tr.aspect > 3.0, "{tr:?}");
        assert!((tr.closure - 250.0).abs() < 10.0, "{tr:?}");
        assert!((tr.speed - 250.0).abs() < 10.0, "{tr:?}");
        assert!(tr.az.abs() < 0.01 && radar.beam.abs() < 0.01);
        // Behind the gimbals: back to the search.
        let mut t = [fighter(1, 0.0, -30_000.0, 3000.0, 0.0)];
        run(&mut radar, 0.5, A6_4B, &mut t, 250.0, Some(ContactRef::Unit(1)), |_, _| true);
        assert!(radar.track.is_none());
    }

    #[test]
    fn the_scan_steps_through_its_bars() {
        let mut radar = AaRadar::default();
        let mut bars = std::collections::BTreeSet::new();
        let dt = 1.0 / 60.0;
        for _ in 0..(A6_4B.frame() / dt) as usize + 10 {
            radar.step(dt, DVec3::ZERO, 0.0, A6_4B, &[], None, |_, _| true);
            assert!(radar.beam.abs() <= GIMBAL + 1e-4);
            bars.insert(radar.bar);
        }
        assert_eq!(bars.len(), 4);
    }
}
