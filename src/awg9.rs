//! The F-14's AWG-9 weapons control system: its radar's modes, the track files of its
//! track-while-scan and the Phoenix shots it guides.
//!
//! - **Pulse search** (`Mode::PulseSearch`): ranges at every aspect but blind looking down into
//!   the ground's clutter; **pulse STT** (`PulseStt`) the same, locked on one aircraft.
//! - **Pulse Doppler search** (`PdSearch`): the longest reach against closing aircraft (less
//!   against one going away), closure only (no range), and looking down a target flying across
//!   the beam hides in the Doppler notch. **Range while search** (`Rws`): the same with ranges.
//!   **PD STT** (`PdStt`): locked on one aircraft, lost in the notch.
//! - **Track while scan** (`TwsAuto`, `TwsMan`): pulse Doppler with ranges over a fixed scan
//!   (+-40 degrees and 2 bars, or +-20 and 4), keeping a track file of each aircraft seen (up
//!   to 24): numbered, its velocity estimated from its returns, extrapolated for a while after
//!   it is no longer seen. These tracks guide the Phoenixes (`PlayerState::support`). In AUTO
//!   the WCS ranks the hostile tracks for the shots (the six most urgent: soonest at us first),
//!   in MAN the RIO's hooked track goes first (`Awg9::set_priority`).
//! - **ACM** (`Plm`, `VslHigh`, `VslLow`, `Mrl`): the pilot's close-in modes, a boresight beam,
//!   a vertical scan above or around the nose, or a wide one, locking on the first aircraft
//!   within 5 nmi (pulse Doppler STT).
//!
//! It runs for the F-14 only, driven by the RIO's panel (`Awg9Controls`, written by the cockpit's
//! controls; defaults until then) instead of an MFD format.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::aa_radar::{BEAM_HALF, BURN_M, FADE_IN, JAM_HEARD, NOTCH, SCAN_RATE, Scan, Strobe};
use crate::aa_radar::{Target, Track, geometry, seed, sight_open};
use crate::ground_radar::{TheaterGround, angle_diff, hash, unit};
use crate::mfd::{ContactKind, ContactRef, Contacts};
use crate::world::{WU, WorldPos};

pub struct Awg9Plugin;

impl Plugin for Awg9Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Awg9>()
            .init_resource::<Awg9Controls>()
            .add_systems(OnEnter(crate::settings::AppState::Flight), reset)
            .add_systems(
                Update,
                scan.after(crate::combat::CombatSystems)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            );
    }
}

const DEG: f32 = std::f32::consts::PI / 180.0;
/// The antenna's gimbals either side of the nose (radians).
pub const GIMBAL: f32 = 65.0 * DEG;
/// A fighter's detection range (metres): pulse Doppler against one closing, pulse; an attack
/// drone's. Going away (opening) a pulse Doppler mode sees it this much less far.
const FIGHTER_PD: f64 = 150_000.0;
const FIGHTER_PULSE: f64 = 100_000.0;
const DRONE_PD: f64 = 36_000.0;
const DRONE_PULSE: f64 = 24_000.0;
const OPENING: f64 = 0.6;
/// Pulse modes see nothing lower than this under the horizon (radians): the ground's clutter.
const PULSE_LOOKDOWN: f32 = -0.5 * DEG;
/// A track holds out to this many detection ranges.
const TRACK_REACH: f64 = 1.2;
/// Track files: at most this many; extrapolated after this many scan frames unseen, dropped
/// after this many seconds unseen; the seconds over which the velocity estimate settles.
pub const MAX_TRACKS: usize = 24;
const COAST_FRAMES: f32 = 1.5;
const DROP_SECS: f32 = 14.0;
const TRACK_TAU: f64 = 2.0;
/// The tracks the WCS ranks for the Phoenix shots in TWS AUTO.
pub const PRIORITIES: usize = 6;
/// ACM: the range (metres, 5 nmi) inside which it locks on, the boresight beam's half width,
/// the vertical scans' elevations (radians: low and high from the nose) and the wide scan's
/// half width.
const ACM_RANGE: f64 = 9_260.0;
pub const PLM_HALF: f32 = 2.3 * DEG;
pub const VSL_HIGH: (f32, f32) = (15.0 * DEG, 55.0 * DEG);
pub const VSL_LOW: (f32, f32) = (-15.0 * DEG, 25.0 * DEG);
const MRL_HALF: f32 = 20.0 * DEG;
/// Returns kept on the DDD for this many scan frames.
const KEEP_FRAMES: f32 = 2.0;

/// The radar's mode (the RIO's WCS buttons, the pilot's ACM buttons).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Mode {
    /// Not transmitting.
    #[default]
    Standby,
    PulseSearch,
    PdSearch,
    Rws,
    TwsAuto,
    TwsMan,
    PdStt,
    PulseStt,
    /// ACM: pilot lock (boresight), vertical scan lock high / low, the wide scan.
    Plm,
    VslHigh,
    VslLow,
    Mrl,
}

impl Mode {
    /// It transmits.
    pub fn on(self) -> bool {
        self != Mode::Standby
    }

    /// Pulse (not pulse Doppler): ranges at every aspect, blind looking down.
    pub fn pulse(self) -> bool {
        matches!(self, Mode::PulseSearch | Mode::PulseStt)
    }

    pub fn tws(self) -> bool {
        matches!(self, Mode::TwsAuto | Mode::TwsMan)
    }

    pub fn stt(self) -> bool {
        matches!(self, Mode::PdStt | Mode::PulseStt)
    }

    pub fn acm(self) -> bool {
        matches!(self, Mode::Plm | Mode::VslHigh | Mode::VslLow | Mode::Mrl)
    }

    /// Its name on the displays.
    pub fn label(self) -> &'static str {
        match self {
            Mode::Standby => "STBY",
            Mode::PulseSearch => "P SRCH",
            Mode::PdSearch => "PD SRCH",
            Mode::Rws => "RWS",
            Mode::TwsAuto => "TWS AUTO",
            Mode::TwsMan => "TWS MAN",
            Mode::PdStt => "PD STT",
            Mode::PulseStt => "P STT",
            Mode::Plm => "PLM",
            Mode::VslHigh => "VSL HI",
            Mode::VslLow => "VSL LO",
            Mode::Mrl => "MRL",
        }
    }

    /// Its returns have no range: pulse Doppler search shows closure against azimuth.
    pub fn velocity_only(self) -> bool {
        self == Mode::PdSearch
    }
}

/// What the RIO (and the pilot's ACM buttons) set: the mode, the scan's half width (radians),
/// bars and elevation (radians over the horizon, the antenna knob), the displays' range.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Awg9Controls {
    pub mode: Mode,
    pub az_half: f32,
    pub bars: u8,
    pub elevation: f32,
    /// The DDD's range (metres).
    pub range: f32,
    /// PH ACT: the Phoenixes are fired active off the rail.
    pub ph_active: bool,
}

impl Default for Awg9Controls {
    fn default() -> Self {
        Self {
            mode: Mode::TwsAuto,
            az_half: 40.0 * DEG,
            bars: 2,
            elevation: 0.0,
            range: 185_200.0,
            ph_active: false,
        }
    }
}

impl Awg9Controls {
    /// The scan the mode sweeps: TWS its fixed +-40 x 2 or +-20 x 4, the ACM modes theirs.
    pub fn scan(&self) -> Scan {
        match self.mode {
            m if m.tws() => {
                if self.az_half <= 20.0 * DEG {
                    Scan { az_half: 20.0 * DEG, bars: 4, elevation: self.elevation }
                } else {
                    Scan { az_half: 40.0 * DEG, bars: 2, elevation: self.elevation }
                }
            }
            Mode::Mrl => Scan { az_half: MRL_HALF, bars: 2, elevation: 0.0 },
            _ => Scan {
                az_half: self.az_half.clamp(10.0 * DEG, GIMBAL),
                bars: self.bars.clamp(1, 8),
                elevation: self.elevation,
            },
        }
    }
}

/// A return on the DDD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// Azimuth from the nose (radians, right positive), range (metres) and closure (m/s,
    /// closing positive) when it was painted.
    pub az: f32,
    pub range: f32,
    pub closure: f32,
    pub target: ContactRef,
    /// Friend or foe, once the IFF has interrogated it (`Awg9::interrogate`); None until then.
    pub friendly: Option<bool>,
    /// Seconds since it was painted.
    pub age: f32,
}

/// A track file of the track while scan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackFile {
    /// Its number on the TID (1..).
    pub number: u16,
    pub target: ContactRef,
    /// Where it was last seen (metres east, north, up), its estimated velocity (m/s).
    pub pos: DVec3,
    pub vel: DVec3,
    /// Friend or foe, once the IFF has interrogated it (`Awg9::interrogate`); None until then.
    pub friendly: Option<bool>,
    /// Seconds since it was last seen, and since it was first.
    pub unseen: f32,
    pub age: f32,
    /// Its rank for the Phoenix shots (1 first), if it has one.
    pub priority: Option<u8>,
}

impl TrackFile {
    /// Where it is now by its estimate (it flies on as it was seen).
    pub fn now(&self) -> DVec3 {
        self.pos + self.vel * self.unseen as f64
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Awg9 {
    pub mode: Mode,
    /// The beam's azimuth from the nose and elevation over the horizon (radians), the way it
    /// moves and the bar it scans.
    pub beam: f32,
    pub elevation: f32,
    dir: f32,
    pub bar: u8,
    pub hits: Vec<Hit>,
    pub strobes: Vec<Strobe>,
    pub tracks: Vec<TrackFile>,
    /// The single target track (STT, or the ACM's lock).
    pub stt: Option<Track>,
    /// An ACM mode found an aircraft to lock on: the lock it wants (`scan` asks the simulation
    /// for it).
    pub acquired: Option<ContactRef>,
    /// The aircraft the RIO hooked (a track on the TID, a return on the DDD), and its track.
    pub hook: Option<ContactRef>,
    pub hooked: Option<u16>,
    /// A scan frame's length (s).
    pub frame: f32,
    /// PH ACT (from the controls).
    pub ph_active: bool,
    next_number: u16,
    passes: u32,
    /// Own position and velocity (metres, m/s), own heading (radians).
    own: Option<(DVec3, DVec3)>,
    heading: f32,
    /// The targets' last positions and velocities (for closures and the STT).
    motions: Vec<(ContactRef, DVec3, DVec3)>,
    /// The aircraft the IFF has interrogated and what they answered (friendly), while the radar
    /// holds them; the IFF's button was pushed (the next frame interrogates).
    identified: Vec<(ContactRef, bool)>,
    asking: bool,
}

impl Default for Awg9 {
    fn default() -> Self {
        Self {
            mode: Mode::Standby,
            beam: 0.0,
            elevation: 0.0,
            dir: 1.0,
            bar: 0,
            hits: Vec::new(),
            strobes: Vec::new(),
            tracks: Vec::new(),
            stt: None,
            acquired: None,
            hook: None,
            hooked: None,
            frame: 1.0,
            ph_active: false,
            next_number: 1,
            passes: 0,
            own: None,
            heading: 0.0,
            motions: Vec::new(),
            identified: Vec::new(),
            asking: false,
        }
    }
}

fn reset(mut radar: ResMut<Awg9>) {
    *radar = default();
}

impl Awg9 {
    /// Off: nothing painted, nothing tracked (the track numbers go on).
    pub fn clear(&mut self) {
        let next = self.next_number;
        *self = Awg9 { next_number: next, ..default() };
    }

    /// Own heading (radians) at the last frame.
    pub fn own_heading(&self) -> f32 {
        self.heading
    }

    /// Own position (metres east, north, up) at the last frame.
    pub fn own_position(&self) -> Option<DVec3> {
        self.own.map(|o| o.0)
    }

    /// The IFF's button pushed: every aircraft the radar holds now (a return, a track, the
    /// single target track) is interrogated, and is known as friend or foe from then on until
    /// the radar loses it. One it finds later is unknown until the next push.
    pub fn interrogate(&mut self) {
        self.asking = true;
    }

    /// Friend (true) or foe as the IFF found an aircraft; None if it has not interrogated it.
    pub fn identity(&self, t: ContactRef) -> Option<bool> {
        self.identified.iter().find(|k| k.0 == t).map(|k| k.1)
    }

    /// The frame's end: the IFF's answers of the aircraft held if it was pushed, those of the
    /// aircraft lost forgotten, the returns and the tracks marked by what is known.
    fn identify(&mut self, targets: &[Target]) {
        let held = |r: &Awg9, t: ContactRef| {
            r.hits.iter().any(|h| h.target == t)
                || r.tracks.iter().any(|k| k.target == t)
                || r.stt.is_some_and(|s| s.target == t)
        };
        if std::mem::take(&mut self.asking) {
            for t in targets {
                if held(self, t.target) && self.identity(t.target).is_none() {
                    self.identified.push((t.target, t.friendly));
                }
            }
        }
        let mut known = std::mem::take(&mut self.identified);
        known.retain(|k| held(self, k.0));
        self.identified = known;
        for i in 0..self.hits.len() {
            self.hits[i].friendly = self.identity(self.hits[i].target);
        }
        for i in 0..self.tracks.len() {
            self.tracks[i].friendly = self.identity(self.tracks[i].target);
        }
    }

    /// The track numbered `n`.
    pub fn track(&self, n: u16) -> Option<&TrackFile> {
        self.tracks.iter().find(|t| t.number == n)
    }

    /// TWS MAN: the RIO ranks track `n` for the shots at `priority` (None takes it out).
    pub fn set_priority(&mut self, n: u16, priority: Option<u8>) {
        for t in &mut self.tracks {
            if t.number == n {
                t.priority = priority;
            } else if priority.is_some() && t.priority == priority {
                t.priority = None;
            }
        }
    }

    /// The aircraft its tracks guide missiles at: the TWS tracks still held (extrapolated ones
    /// too, a while), the STT's.
    pub fn supported(&self) -> Vec<ContactRef> {
        let mut out: Vec<ContactRef> = self.tracks.iter().map(|t| t.target).collect();
        if let Some(s) = self.stt {
            out.push(s.target);
        }
        out
    }

    /// The target of the next Phoenix in TWS: the first ranked track with no Phoenix of ours
    /// in flight at it (`in_flight`), else the hooked one.
    pub fn next_shot(&self, in_flight: &[ContactRef]) -> Option<ContactRef> {
        if !self.mode.tws() {
            return None;
        }
        let mut ranked: Vec<&TrackFile> =
            self.tracks.iter().filter(|t| t.priority.is_some()).collect();
        ranked.sort_by_key(|t| t.priority);
        ranked
            .into_iter()
            .map(|t| t.target)
            .find(|t| !in_flight.contains(t))
            .or_else(|| self.hooked.and_then(|n| self.track(n)).map(|t| t.target))
    }

    fn velocity(&self, t: ContactRef) -> DVec3 {
        self.motions.iter().find(|m| m.0 == t).map_or(DVec3::ZERO, |m| m.2)
    }

    /// A frame: the aircraft at `own` (metres east, north, up) heading `heading`, the controls
    /// `c`, the simulation's air lock `locked`; `sight` tells whether the ground leaves the line
    /// between two points open.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        dt: f32,
        own: DVec3,
        heading: f32,
        c: &Awg9Controls,
        targets: &[Target],
        locked: Option<ContactRef>,
        sight: impl Fn(DVec3, DVec3) -> bool,
    ) {
        self.sweep(dt, own, heading, c, targets, locked, sight);
        if c.mode.on() {
            self.identify(targets);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sweep(
        &mut self,
        dt: f32,
        own: DVec3,
        heading: f32,
        c: &Awg9Controls,
        targets: &[Target],
        locked: Option<ContactRef>,
        sight: impl Fn(DVec3, DVec3) -> bool,
    ) {
        if self.mode != c.mode {
            // A new mode: the search returns go, TWS tracks only stay in TWS, ACM starts over.
            self.hits.clear();
            if !c.mode.tws() {
                for t in &mut self.tracks {
                    t.priority = None;
                }
            }
            self.acquired = None;
        }
        self.mode = c.mode;
        self.heading = heading;
        self.ph_active = c.ph_active;
        if !c.mode.on() {
            self.clear();
            return;
        }
        let dt64 = dt as f64;
        // Velocities from the positions (own and the targets').
        let own_vel = match self.own {
            Some((p, v)) if dt64 > 0.0 => v + ((own - p) / dt64 - v) * smooth(dt64, 0.5),
            _ => DVec3::ZERO,
        };
        self.own = Some((own, own_vel));
        self.motions = targets
            .iter()
            .map(|t| match self.motions.iter().find(|m| m.0 == t.target) {
                Some(&(_, p, v)) if dt64 > 0.0 && p.distance(t.pos) < 2000.0 * dt64.max(0.05) => {
                    (t.target, t.pos, v + ((t.pos - p) / dt64 - v) * smooth(dt64, 0.5))
                }
                Some(&(_, _, v)) => (t.target, t.pos, v),
                None => (t.target, t.pos, DVec3::ZERO),
            })
            .collect();
        let scan = c.scan();
        self.frame = scan.frame();
        for h in &mut self.hits {
            h.age += dt;
        }
        for s in &mut self.strobes {
            s.age += dt;
        }
        let keep = KEEP_FRAMES * self.frame;
        self.hits.retain(|h| h.age < keep);
        self.strobes.retain(|s| s.age < keep);
        for t in &mut self.tracks {
            t.unseen += dt;
            t.age += dt;
        }
        self.tracks.retain(|t| t.unseen < DROP_SECS);
        if self.hooked.is_some_and(|n| self.track(n).is_none()) {
            self.hooked = None;
        }
        let pulse = c.mode.pulse();
        // Seen out to `reach` detection ranges in this mode.
        let seen = |r: &Awg9, t: &Target, el: f32, range: f64, reach: f64| {
            let los = (t.pos - own) / range.max(1.0);
            let closure = -(r.velocity(t.target) - own_vel).dot(los);
            let drone = t.detect < 50_000.0;
            let base = match (pulse, drone) {
                (true, false) => FIGHTER_PULSE,
                (true, true) => DRONE_PULSE,
                (false, false) => FIGHTER_PD,
                (false, true) => DRONE_PD,
            };
            let detect = if !pulse && closure < 0.0 { base * OPENING } else { base };
            if range > detect * reach {
                return false;
            }
            if pulse {
                // Looking down into the ground's clutter.
                if el < PULSE_LOOKDOWN {
                    return false;
                }
            } else if el < 0.0 && r.velocity(t.target).dot(los).abs() < NOTCH {
                return false;
            }
            sight(own, t.pos)
        };
        // The single target track: the lock while it is in the gimbals and seen.
        let want_stt = c.mode.stt() || c.mode.acm();
        self.stt = locked
            .filter(|_| want_stt)
            .and_then(|l| targets.iter().find(|t| t.target == l))
            .and_then(|t| {
                let (az, el, range) = geometry(own, heading, t.pos);
                if az.abs() > GIMBAL || el.abs() > GIMBAL || !seen(self, t, el, range, TRACK_REACH)
                {
                    return None;
                }
                if t.jam && range > BURN_M {
                    return None;
                }
                let v = self.velocity(t.target);
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
        if let Some(t) = self.stt {
            self.beam = t.az;
            self.elevation = t.el;
            self.acquired = None;
            return;
        }
        if c.mode.stt() {
            // Nothing to track: the antenna rests ahead.
            self.beam = 0.0;
            self.elevation = c.elevation;
            return;
        }
        if c.mode.acm() {
            self.acm(own, heading, c.mode, targets, &seen);
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
        let tws = c.mode.tws();
        for t in targets {
            let (az, el, range) = geometry(own, heading, t.pos);
            let Some(&(_, _, _, pass)) = spans.iter().find(|&&(lo, hi, bar, _)| {
                az >= lo - BEAM_HALF
                    && az <= hi + BEAM_HALF
                    && (el - scan.bar_elevation(bar)).abs() <= BEAM_HALF
            }) else {
                continue;
            };
            if t.jam && range > BURN_M {
                if range < FIGHTER_PD * JAM_HEARD && sight(own, t.pos) {
                    self.strobes.retain(|s| s.target != t.target);
                    self.strobes.push(Strobe { az, target: t.target, age: 0.0 });
                }
                self.hits.retain(|h| h.target != t.target);
                continue;
            }
            let edge =
                FADE_IN + (1.0 / FADE_IN - FADE_IN) * unit(hash(seed(t.target), pass, 9)) as f64;
            if !seen(self, t, el, range, edge) {
                continue;
            }
            let los = (t.pos - own) / range.max(1.0);
            let closure = -(self.velocity(t.target) - own_vel).dot(los) as f32;
            self.strobes.retain(|s| s.target != t.target);
            self.hits.retain(|h| h.target != t.target);
            self.hits.push(Hit {
                az,
                range: range as f32,
                closure,
                target: t.target,
                friendly: self.identity(t.target),
                age: 0.0,
            });
            if tws {
                self.update_track(t, own_vel);
            }
        }
        // Coasting tracks: extrapolated from their last return.
        let coast = COAST_FRAMES * self.frame;
        for tr in &mut self.tracks {
            if tr.unseen > coast {
                tr.priority = tr.priority.filter(|_| tr.unseen < DROP_SECS / 2.0);
            }
        }
        if c.mode == Mode::TwsAuto {
            self.rank(own);
        }
    }

    /// A return of target `t` in TWS: its track file updated (velocity estimated from the
    /// positions), or a new one.
    fn update_track(&mut self, t: &Target, _own_vel: DVec3) {
        if let Some(tr) = self.tracks.iter_mut().find(|tr| tr.target == t.target) {
            let dt = tr.unseen.max(0.05) as f64;
            let measured = (t.pos - tr.pos) / dt;
            let k = if tr.age < 0.5 { 1.0 } else { smooth(dt, TRACK_TAU) };
            tr.vel += (measured - tr.vel) * k;
            tr.pos = t.pos;
            tr.unseen = 0.0;
            return;
        }
        if self.tracks.len() >= MAX_TRACKS {
            return;
        }
        let number = self.next_number;
        self.next_number = self.next_number % 999 + 1;
        self.tracks.push(TrackFile {
            number,
            target: t.target,
            pos: t.pos,
            vel: self.velocity(t.target),
            friendly: self.identity(t.target),
            unseen: 0.0,
            age: 0.0,
            priority: None,
        });
    }

    /// TWS AUTO: the tracks not known as friends ranked for the shots, the soonest at us first
    /// (range over closure; those not closing by their range), the first `PRIORITIES`.
    fn rank(&mut self, own: DVec3) {
        let own_vel = self.own.map_or(DVec3::ZERO, |o| o.1);
        let mut order: Vec<(f64, u16)> = self
            .tracks
            .iter()
            .filter(|t| t.friendly != Some(true) && t.unseen < COAST_FRAMES * self.frame)
            .map(|t| {
                let d = t.now() - own;
                let range = d.length();
                let closure = -(t.vel - own_vel).dot(d / range.max(1.0));
                let urgency = if closure > 10.0 { range / closure } else { 1e6 + range };
                (urgency, t.number)
            })
            .collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        for t in &mut self.tracks {
            t.priority = None;
        }
        for (k, (_, n)) in order.into_iter().take(PRIORITIES).enumerate() {
            if let Some(t) = self.tracks.iter_mut().find(|t| t.number == n) {
                t.priority = Some(k as u8 + 1);
            }
        }
    }

    /// The ACM modes' search: the nearest aircraft within 5 nmi in its pattern, to lock on.
    fn acm(
        &mut self,
        own: DVec3,
        heading: f32,
        mode: Mode,
        targets: &[Target],
        seen: &impl Fn(&Awg9, &Target, f32, f64, f64) -> bool,
    ) {
        let inside = |az: f32, el: f32| match mode {
            Mode::Plm => az.abs() <= PLM_HALF && el.abs() <= PLM_HALF,
            Mode::VslHigh => az.abs() <= 2.0 * BEAM_HALF && (VSL_HIGH.0..=VSL_HIGH.1).contains(&el),
            Mode::VslLow => az.abs() <= 2.0 * BEAM_HALF && (VSL_LOW.0..=VSL_LOW.1).contains(&el),
            _ => az.abs() <= MRL_HALF && el.abs() <= 3.0 * BEAM_HALF,
        };
        // The beam's sweep for the displays.
        self.beam = match mode {
            Mode::Mrl => (self.beam + self.dir * SCAN_RATE / 60.0).clamp(-MRL_HALF, MRL_HALF),
            _ => 0.0,
        };
        if self.beam.abs() >= MRL_HALF {
            self.dir = -self.dir;
        }
        self.acquired = targets
            .iter()
            .filter_map(|t| {
                let (az, el, range) = geometry(own, heading, t.pos);
                (range < ACM_RANGE && inside(az, el) && seen(self, t, el, range, 1.0))
                    .then_some((range, t.target))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, t)| t);
    }
}

/// The share of the way an estimate moves towards a new value over `dt` seconds, settling
/// over `tau`.
fn smooth(dt: f64, tau: f64) -> f64 {
    1.0 - (-dt / tau).exp()
}

/// The AWG-9 in the simulation's report of the player: it transmits, the aircraft its tracks
/// hold guide the Phoenixes and the Sparrows (`PlayerState::support`), and the next Phoenix
/// goes for the first ranked track without one in flight at it (`launch_unit`).
pub fn report(radar: &Awg9, sim: &stealth_sim::Sim, ps: &mut stealth_sim::PlayerState) {
    if !radar.mode.on() {
        return;
    }
    ps.radar = true;
    ps.ph_active = radar.ph_active;
    for t in radar.supported() {
        match t {
            ContactRef::Unit(i) => ps.support.extend(u16::try_from(i).ok()),
            ContactRef::Pilot(k) => ps.support_pilots.push(k),
            _ => {}
        }
    }
    let in_flight: Vec<ContactRef> = sim.projectiles()[stealth_sim::ENEMY_SLOTS..]
        .iter()
        .filter(|m| m.ttl != 0 && m.spec == stealth_sim::PHOENIX_SPEC)
        .map(|m| match m.target_pilot {
            Some(k) => ContactRef::Pilot(k),
            None => ContactRef::Unit(m.target.max(0) as usize),
        })
        .collect();
    match radar.next_shot(&in_flight) {
        Some(ContactRef::Unit(i)) => ps.launch_unit = u16::try_from(i).ok(),
        Some(ContactRef::Pilot(k)) => ps.launch_pilot = Some(k),
        _ => {}
    }
}

/// The aircraft the radar may see: those on the contacts (drones with less reach).
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
                // The mode sets the reach; this tells a drone from a fighter.
                detect: if k.kind == ContactKind::Drone { DRONE_PD } else { FIGHTER_PD },
                jam: k.jammer,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn scan(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    controls: Res<Awg9Controls>,
    contacts: Res<Contacts>,
    mut radar: ResMut<Awg9>,
    aircraft: Query<(&crate::flight::Aircraft, &WorldPos), With<crate::flight::Player>>,
    combat: Option<Res<crate::combat::Combat>>,
    theater: Option<Res<crate::world::Theater>>,
    mut locks: MessageWriter<crate::avionics::LockRequest>,
) {
    let tomcat = settings.aircraft == crate::aircraft::AircraftKind::F14;
    let flying = aircraft.single().ok().filter(|(a, _)| !a.on_ground && a.ended.is_none());
    let (Some((a, pos)), Some(combat), true) = (flying, combat, tomcat) else {
        if radar.mode.on() {
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
    radar.step(time.delta_secs(), own, a.euler().0, &controls, &found, locked, |a, b| {
        sight_open(&ground, a, b)
    });
    // ACM: the aircraft it found, locked.
    if let Some(t) = radar.acquired.filter(|&t| locked != Some(t)) {
        locks.write(crate::avionics::LockRequest(t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fighter(i: usize, east: f64, north: f64, up: f64, heading: f32) -> Target {
        Target {
            target: ContactRef::Unit(i),
            pos: DVec3::new(east, north, up),
            heading,
            friendly: false,
            detect: FIGHTER_PD,
            jam: false,
        }
    }

    fn controls(mode: Mode) -> Awg9Controls {
        Awg9Controls { mode, ..default() }
    }

    /// Runs the radar `secs` seconds at 30 Hz, the aircraft at 9000 m heading north at 250 m/s
    /// (from the origin), the targets flying on at their headings at `speed` m/s.
    fn run(
        radar: &mut Awg9,
        secs: f32,
        c: &Awg9Controls,
        targets: &mut [Target],
        speed: f64,
        locked: Option<ContactRef>,
    ) -> DVec3 {
        let dt = 1.0 / 30.0;
        let mut own = DVec3::new(0.0, 0.0, 9000.0);
        for _ in 0..(secs / dt) as usize {
            own.y += 250.0 * dt as f64;
            for t in targets.iter_mut() {
                let (s, co) = (t.heading as f64).sin_cos();
                t.pos += DVec3::new(s, co, 0.0) * speed * dt as f64;
            }
            radar.step(dt, own, 0.0, c, targets, locked, |_, _| true);
        }
        own
    }

    /// TWS: three fighters coming head on far out each get a numbered track with their
    /// velocity, ranked soonest first; they all guide missiles.
    #[test]
    fn tws_tracks_and_ranks_the_closing_fighters() {
        let mut radar = Awg9::default();
        let c = controls(Mode::TwsAuto);
        let pi = std::f32::consts::PI;
        let mut t = [
            fighter(1, -10_000.0, 120_000.0, 9000.0, pi),
            fighter(2, 0.0, 90_000.0, 9000.0, pi),
            fighter(3, 10_000.0, 100_000.0, 9000.0, pi),
        ];
        run(&mut radar, 4.0 * c.scan().frame(), &c, &mut t, 250.0, None);
        assert_eq!(radar.tracks.len(), 3, "{:?}", radar.tracks);
        let speed = |n: usize| {
            let tr = radar.tracks.iter().find(|tr| tr.target == ContactRef::Unit(n)).unwrap();
            (tr.vel.y, tr.priority)
        };
        // Southbound at 250 m/s.
        assert!((speed(2).0 + 250.0).abs() < 25.0, "{:?}", speed(2));
        assert_eq!(speed(2).1, Some(1));
        assert_eq!(speed(3).1, Some(2));
        assert_eq!(speed(1).1, Some(3));
        let mut s = radar.supported();
        s.sort_by_key(|r| format!("{r:?}"));
        assert_eq!(s.len(), 3);
        // The next shot: the first ranked without a missile at it.
        assert_eq!(radar.next_shot(&[]), Some(ContactRef::Unit(2)));
        assert_eq!(radar.next_shot(&[ContactRef::Unit(2)]), Some(ContactRef::Unit(3)));
    }

    /// A track is unknown until the IFF interrogates it: then the friend shows as one and
    /// leaves the ranks, the foe shows as one; an aircraft found later is unknown again until
    /// the next push, and one the radar lost is forgotten.
    #[test]
    fn the_iff_tells_friend_from_foe_once_pushed() {
        let mut radar = Awg9::default();
        let c = controls(Mode::TwsAuto);
        let pi = std::f32::consts::PI;
        let mut t = vec![
            fighter(1, -10_000.0, 120_000.0, 9000.0, pi),
            Target { friendly: true, ..fighter(2, 0.0, 90_000.0, 9000.0, pi) },
        ];
        let frames = 4.0 * c.scan().frame();
        run(&mut radar, frames, &c, &mut t, 250.0, None);
        let track = |r: &Awg9, n: usize| {
            *r.tracks.iter().find(|tr| tr.target == ContactRef::Unit(n)).unwrap()
        };
        assert_eq!(track(&radar, 1).friendly, None);
        assert_eq!(track(&radar, 2).friendly, None);
        assert!(radar.hits.iter().all(|h| h.friendly.is_none()));
        // Unknown, the friend is ranked for a shot like the other.
        assert_eq!(track(&radar, 2).priority, Some(1));
        radar.interrogate();
        run(&mut radar, 0.2, &c, &mut t, 250.0, None);
        assert_eq!(track(&radar, 1).friendly, Some(false));
        assert_eq!(track(&radar, 2).friendly, Some(true));
        assert_eq!(track(&radar, 2).priority, None);
        assert_eq!(track(&radar, 1).priority, Some(1));
        assert_eq!(radar.identity(ContactRef::Unit(2)), Some(true));
        // It stays known on the returns of the later sweeps.
        run(&mut radar, frames, &c, &mut t, 250.0, None);
        assert_eq!(track(&radar, 2).friendly, Some(true));
        assert!(radar.hits.iter().any(|h| h.friendly == Some(true)));
        // A newcomer is unknown until the next push.
        t.push(Target { friendly: true, ..fighter(3, 10_000.0, 80_000.0, 9000.0, pi) });
        run(&mut radar, frames, &c, &mut t, 250.0, None);
        assert_eq!(track(&radar, 3).friendly, None);
        assert_eq!(track(&radar, 1).friendly, Some(false));
        radar.interrogate();
        run(&mut radar, 0.2, &c, &mut t, 250.0, None);
        assert_eq!(track(&radar, 3).friendly, Some(true));
        // Switched off, the radar forgets.
        run(&mut radar, 0.2, &controls(Mode::Standby), &mut t, 250.0, None);
        assert_eq!(radar.identity(ContactRef::Unit(2)), None);
    }

    /// Pulse search does not see an aircraft below the horizon (ground clutter), pulse Doppler
    /// does unless it flies across the beam.
    #[test]
    fn pulse_is_blind_looking_down_and_pd_has_its_notch() {
        let low = |mode| Awg9Controls { elevation: -4.0 * DEG, ..controls(mode) };
        let pi = std::f32::consts::PI;
        for (mode, closing, beaming) in [
            (Mode::PulseSearch, false, false),
            (Mode::Rws, true, false),
            (Mode::PdSearch, true, false),
        ] {
            let mut radar = Awg9::default();
            let c = low(mode);
            let mut t = [
                fighter(1, 0.0, 40_000.0, 6000.0, pi),
                fighter(2, 0.0, 45_000.0, 6000.0, std::f32::consts::FRAC_PI_2),
            ];
            run(&mut radar, 3.0 * c.scan().frame(), &c, &mut t, 230.0, None);
            let has = |n| radar.hits.iter().any(|h| h.target == ContactRef::Unit(n));
            assert_eq!(has(1), closing, "{mode:?} head on");
            assert_eq!(has(2), beaming, "{mode:?} beaming");
        }
    }

    /// Pulse Doppler reaches farther than pulse; and a target going away (faster than us) is
    /// seen less far.
    #[test]
    fn pd_reaches_farther_against_closing_targets() {
        let pi = std::f32::consts::PI;
        let reach = |mode, heading, speed| {
            let c = controls(mode);
            (60..200).step_by(10).rev().find(|&km| {
                let mut radar = Awg9::default();
                let mut t = [fighter(1, 0.0, km as f64 * 1000.0, 9000.0, heading)];
                run(&mut radar, 2.5 * c.scan().frame(), &c, &mut t, speed, None);
                radar.hits.iter().any(|h| h.target == ContactRef::Unit(1))
            })
        };
        let pd = reach(Mode::Rws, pi, 200.0);
        let pulse = reach(Mode::PulseSearch, pi, 200.0);
        let away = reach(Mode::Rws, 0.0, 350.0);
        assert!(pd > pulse && pd > away, "{pd:?} {pulse:?} {away:?}");
    }

    /// An ACM mode locks on the first aircraft inside 5 nmi in its pattern: PLM straight ahead,
    /// VSL high above the nose.
    #[test]
    fn acm_acquires_close_in() {
        let mut radar = Awg9::default();
        let mut t =
            [fighter(1, 0.0, 6_000.0, 9000.0, 0.0), fighter(2, 0.0, 4_000.0, 12_000.0, 0.0)];
        run(&mut radar, 0.5, &controls(Mode::Plm), &mut t, 250.0, None);
        assert_eq!(radar.acquired, Some(ContactRef::Unit(1)));
        run(&mut radar, 0.5, &controls(Mode::VslHigh), &mut t, 250.0, None);
        assert_eq!(radar.acquired, Some(ContactRef::Unit(2)));
        // The lock comes: tracked in STT.
        run(&mut radar, 0.5, &controls(Mode::VslHigh), &mut t, 250.0, Some(ContactRef::Unit(2)));
        assert!(radar.stt.is_some() && radar.acquired.is_none());
    }

    /// A track no longer seen is extrapolated, then dropped.
    #[test]
    fn lost_tracks_coast_then_drop() {
        let mut radar = Awg9::default();
        let c = controls(Mode::TwsAuto);
        let mut t = [fighter(1, 0.0, 60_000.0, 9000.0, std::f32::consts::PI)];
        run(&mut radar, 2.0 * c.scan().frame(), &c, &mut t, 250.0, None);
        assert_eq!(radar.tracks.len(), 1);
        run(&mut radar, 5.0, &c, &mut [], 250.0, None);
        assert_eq!(radar.tracks.len(), 1, "coasting");
        run(&mut radar, DROP_SECS, &c, &mut [], 250.0, None);
        assert!(radar.tracks.is_empty());
    }
}
