//! Anti-submarine warfare: the MH-60R Seahawk's hunt for submarines.
//!
//! Submarines run under water toward a goal: through a blockade line, at a ship, or at the
//! carrier. The crew finds them with sonobuoys (the SSQ-53 DIFAR listens and gives a bearing,
//! the SSQ-62 DICASS listens or pings for a range and bearing, the SSQ-36 BT measures the
//! water's temperature profile: where the layer is), the AQS-13F dipping sonar lowered from
//! the hover (pinging or listening), and the MAD towed body close over a submarine. Crossing the
//! passive bearings is the operator's work (his marks on the waterfalls, `src/asw/marks.rs`);
//! the automatic tracker keeps the active echoes' and the MAD's fixes as tracks (S1, S2...). The Mk 54
//! torpedo is dropped on a chute, runs out on its programmed heading, searches in its pattern
//! after its enable delay and homes on the first submarine it hears; a hit sinks it. A
//! submarine at periscope depth leaves a feather wake a Hellfire can hit.
//!
//! The host's simulation alone knows the submarines: the crews see only what their sensors
//! report ([`Asw::for_clients`]). Positions are metres east and north (as [`crate::gunship`]'s
//! shells, `M_PER_MAP` per map unit), depths metres down from the surface.
//!
//! Sound in the water, in decibels: a submarine's noise (louder fast, much louder when it
//! cavitates near the surface or snorkels) less the spreading and absorption loss over the
//! range, less another `LAYER_LOSS` when the sensor and the submarine are on either side of the
//! layer, against the sea's noise (higher with the sea state). An active ping's echo goes both
//! ways.

use serde::{Deserialize, Serialize};
use stealth_i18n::{Text, text};

use crate::Sim;
use crate::aaa::M_PER_MAP;
use crate::rng::Rng;
use crate::types::*;

/// Mission type: hunt the submarines (`stealth_mission::asw::HUNT`).
pub const HUNT_MISSION: i32 = 12;
/// The Mk 54 torpedo's weapon type.
pub const MK54_WEAPON: i32 = 24;
/// Sonobuoys in the water a crew monitors at most.
pub const MAX_BUOYS: usize = 4;
/// The sonobuoys a Seahawk carries (DIFAR, DICASS, BT).
pub const BUOY_LOAD: [u8; 3] = [4, 2, 2];

const M_PER_WU: f64 = M_PER_MAP / 32.0;
const KT: f64 = 0.514_444;
const FT: f64 = 0.3048;
const DEG: f64 = std::f64::consts::PI / 180.0;
const TAU: f64 = std::f64::consts::TAU;

/// Seconds a buoy lives in the water (its scuttle timer), the BT's.
const BUOY_LIFE: f32 = 1800.0;
const BT_LIFE: f32 = 90.0;
/// The BT reports the layer this long after it splashes (its probe sinking through it).
const BT_REPORT: f32 = 25.0;
/// A buoy falls on its chute at this rate (m/s), at least this long (s).
const BUOY_SINK: f64 = 15.0;
const BUOY_FALL_MIN: f64 = 2.0;
/// Hydrophone depths (m): shallow and deep.
pub const BUOY_DEPTHS: [f32; 2] = [27.0, 120.0];
/// DICASS: pings it holds, seconds between pings.
const DICASS_PINGS: u8 = 24;
const PING_EVERY: f32 = 10.0;
/// The most points the crew's route has.
pub const MAX_ROUTE: usize = 16;

/// A route point (world units east, north) of a point (m east, north), and back.
pub fn wu_point(p: [f64; 2]) -> (i32, i32) {
    ((p[0] / M_PER_WU).round() as i32, (p[1] / M_PER_WU).round() as i32)
}

pub fn wu_metres(p: (i32, i32)) -> [f64; 2] {
    [p.0 as f64 * M_PER_WU, p.1 as f64 * M_PER_WU]
}

/// The buoys of a kind a crew has left.
fn c_left(sim: &Sim, owner: u8, kind: BuoyKind) -> usize {
    sim.asw.crew(owner).map_or(0, |c| c.buoys_left[kind.index()] as usize)
}

/// A pending drop goes when the aircraft passes this close to its point (m).
const DROP_RADIUS: f64 = 150.0;

/// The dipping sonar: highest hover (m above the sea) and fastest (m/s) it may be lowered at;
/// the cable parts above these with the sonar in the water; how fast it reels (m/s); the
/// deepest it goes (m, 440 ft).
const DIP_HOVER: f64 = 70.0 * FT;
const DIP_SPEED: f64 = 5.0 * KT;
const DIP_BREAK_SPEED: f64 = 15.0 * KT;
const DIP_BREAK_HEIGHT: f64 = 150.0 * FT;
const DIP_REEL: f32 = 3.0;
pub const DIP_MAX: f32 = 134.0;
/// The transducer is under the sea this far below the hover's height (m).
const DIP_CABLE_SLACK: f32 = 5.0;

/// MAD: it is towed below this height and speed (m, m/s); the signal at 100 m slant range
/// (tenths of a nanotesla), falling off with the cube of the range; nothing beyond `MAD_RANGE`.
const MAD_HEIGHT: f64 = 300.0 * FT;
const MAD_SPEED: f64 = 120.0 * KT;
const MAD_AT_100: f64 = 400.0;
const MAD_RANGE: f64 = 700.0;
/// Its noise (tenths of nT) and the threshold that drops an automatic mark.
const MAD_NOISE: i32 = 6;
const MAD_MARK: i16 = 30;

/// Sound (dB): the ping's source level, a submarine's target strength, the detection
/// thresholds, the directivity of DIFAR and of the active transducers, the layer's loss.
const PING_SL: f64 = 215.0;
/// The dipping sonar's ping: a bigger transducer, far more power than a buoy's battery (dB).
const DIP_SL: f64 = 228.0;
const TARGET_STRENGTH: f64 = 15.0;
const PASSIVE_DT: f64 = 0.0;
const ACTIVE_DT: f64 = 10.0;
const DIFAR_DI: f64 = 5.0;
const ACTIVE_DI: f64 = 10.0;
const LAYER_LOSS: f64 = 15.0;
/// Active ranges (m): the DICASS's and (below) the dipping sonar's.
const DICASS_RANGE: f64 = 12_000.0;
/// Ships farther than this (m) from every sensor are not worked out (their noise is lost there).
const SHIP_HEARD: f64 = 40_000.0;
const DIP_RANGE: f64 = 30_000.0;
/// A submarine hears a ping within this range (m) and evades.
const COUNTER_DETECT: f64 = 25_000.0;
/// Sensor reports a second (each a waterfall's row).
pub const SENSE_HZ: f64 = 4.0;
/// A torpedo running in the water: its noise (dB), its screw's line (Hz) on the waterfalls.
const TORPEDO_LEVEL: f64 = 158.0;
const TORPEDO_LINE_HZ: u16 = 150;
/// A sensor hears another's ping within this range (m).
const PING_HEARD: f64 = 25_000.0;
/// Where another sonar's ping shows on a waterfall (Hz): the pings are kHz, above its scale, so
/// each kind of sonar has its line there (a DICASS's, a dipping sonar's).
pub const PING_LINE_HZ: [u16; 2] = [220, 200];
/// Sound in sea water (m/s): an echo comes this much later than the ping it is of.
const SOUND_SPEED: f64 = 1500.0;

/// The tracker: fixes fitted (s), a track dropped without a fix (s): both fade from the map
/// over five minutes.
pub const FIX_KEEP: f64 = 300.0;
pub const TRACK_DROP: f64 = 300.0;
/// Fixes the tracker keeps at most, and sends the others.
const MAX_FIXES: usize = 300;
const CLIENT_FIXES: usize = 60;

/// The submarines: turn (rad/s), depth change (m/s), speeds (m/s), the range they fire their
/// torpedoes at their target from (m), the seconds those run.
const SUB_TURN: f64 = 1.5 * DEG;
const SUB_DIVE: f64 = 1.0;
const QUIET: f64 = 4.0 * KT;
const TRANSIT: f64 = 7.0 * KT;
const SPRINT: f64 = 20.0 * KT;
const ATTACK_RANGE: f64 = 8000.0;
const ENEMY_TORPEDO_RUN: f64 = 300.0;
/// Periscope depth (m): reached every so often for a while (s).
pub const PERISCOPE_DEPTH: f64 = 15.0;
const PD_EVERY: (i32, i32) = (600, 1100);
const PD_FOR: f64 = 180.0;
/// Every so often (s) a submarine stops dead and shuts down all it can for a while (s): no
/// noise a passive sensor hears (a ping still finds it).
const SILENT_EVERY: (i32, i32) = (700, 1300);
const SILENT_FOR: (i32, i32) = (150, 330);
/// A helicopter low and slow within this range (m) is heard by a submarine (its rotor, a
/// dipping sonar's splash): it goes silent till the helicopter has been gone this long (s).
const OVERHEAD_RANGE: f64 = 2500.0;
const OVERHEAD_HEIGHT: f64 = 250.0 * FT;
const OVERHEAD_SPEED: f64 = 30.0 * KT;
const OVERHEAD_HOLD: f64 = 150.0;
/// A submarine stopped and silent: its noise this far under its quietest running (dB).
const SILENT_LOSS: f64 = 35.0;
/// The AI's hunt (a campaign's ASW mission nobody flew): its helicopters search round the datum
/// this far out (m); within `AI_SEARCH_RANGE` of the submarine for `AI_LOCALIZE` seconds (its
/// buoys and dips) they hold it, within `AI_FIX_ERR` (m, one sigma), and refresh that every
/// `AI_REFIX` s; they drop a torpedo from the hover this close to the fix (m), `AI_TORPEDOES` an
/// aircraft, `AI_DROP_EVERY` s apart.
const AI_ORBIT: f64 = 2000.0;
const AI_SEARCH_RANGE: f64 = 6000.0;
const AI_LOCALIZE: f64 = 120.0;
const AI_FIX_ERR: f64 = 200.0;
const AI_REFIX: f64 = 90.0;
const AI_DROP_RANGE: f64 = 150.0;
const AI_TORPEDOES: u8 = 2;
const AI_DROP_EVERY: f64 = 120.0;
/// The owner of an AI helicopter's torpedo (no player's).
pub const AI_OWNER: u8 = 255;
/// A Hellfire striking this close to a submarine at periscope depth sinks it (m).
pub const HELLFIRE_SUB_RADIUS: f64 = 25.0;

/// The Mk 54: speeds (m/s) running out, searching, homing; its seeker's range (m) and half
/// cone (rad); its endurance (s); a hit's distance (m); the odds (percent) an evading
/// submarine breaks its lock each time it acquires.
const TORP_RUN: f64 = 40.0 * KT;
const TORP_SEARCH: f64 = 28.0 * KT;
const TORP_HOME: f64 = 45.0 * KT;
const TORP_ACQUIRE: f64 = 1500.0;
const TORP_CONE: f64 = 35.0 * DEG;
const TORP_LIFE: f64 = 480.0;
const TORP_HIT: f64 = 15.0;
const TORP_BREAK: i32 = 25;
/// Radius of the circle and helix patterns (m), the snake's period (s) and swing (rad).
const CIRCLE_R: f64 = 350.0;
const SNAKE_PERIOD: f64 = 40.0;
const SNAKE_SWING: f64 = 45.0 * DEG;
/// Release limits: height above the sea (m) and speed (m/s).
pub const TORP_MAX_HEIGHT: f64 = 300.0 * FT;
pub const TORP_MAX_SPEED: f64 = 100.0 * KT;

/// The water of a mission: the mixed layer's depth (m), the sea state (0 calm .. 6), the
/// depth of the bottom (m).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ocean {
    pub layer: f32,
    pub sea_state: u8,
    pub bottom: f32,
}

impl Default for Ocean {
    fn default() -> Self {
        Ocean { layer: 60.0, sea_state: 2, bottom: 600.0 }
    }
}

impl Ocean {
    /// The sea's noise in the band (dB).
    fn noise(&self) -> f64 {
        45.0 + 3.0 * self.sea_state as f64
    }

    /// Spreading, absorption and the layer's loss (dB) from a sensor at `d1` to a source at
    /// `d2` (m deep) `r` m apart.
    fn loss(&self, r: f64, d1: f64, d2: f64) -> f64 {
        let r = r.max(1.0);
        let layer = if (d1 < self.layer as f64) != (d2 < self.layer as f64) { LAYER_LOSS } else { 0.0 };
        20.0 * r.log10() + 0.1 * r / 1000.0 + layer
    }
}

/// A sonobuoy's kind (console key `Buoy(n)`: 1 DIFAR, 2 DICASS, 3 BT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BuoyKind {
    /// AN/SSQ-53G/O: passive, directional.
    #[default]
    Difar,
    /// AN/SSQ-62F: passive (omnidirectional) or pinging.
    Dicass,
    /// AN/SSQ-36: the bathythermograph.
    Bt,
}

impl BuoyKind {
    pub fn from_key(n: u8) -> Option<Self> {
        match n {
            1 => Some(BuoyKind::Difar),
            2 => Some(BuoyKind::Dicass),
            3 => Some(BuoyKind::Bt),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            BuoyKind::Difar => 0,
            BuoyKind::Dicass => 1,
            BuoyKind::Bt => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BuoyKind::Difar => "DIFAR",
            BuoyKind::Dicass => "DICASS",
            BuoyKind::Bt => "BT",
        }
    }
}

/// One contact in a sensor's report: its signal over the noise (dB), the bearing to it (degrees
/// true from the sensor; DIFAR, the dipping sonar and an active ping), the range (m, an active
/// ping's echo), its closing speed (kt, the echo's Doppler) and the frequencies of its tonal
/// lines (Hz: the machinery and the blade rate).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Line {
    pub snr: i8,
    pub bearing: Option<u16>,
    pub range: Option<u16>,
    pub doppler: i8,
    pub freqs: [u16; 2],
    /// A surface ship's noise (else a submarine's): the operator tells them apart by ear and on
    /// the waterfall; the tracker takes only the submarines'.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ship: bool,
    /// Another sonar's ping (`freqs[0]` its line on the waterfall: `PING_LINE_HZ`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ping: bool,
    /// A torpedo running: its fast screws cavitating.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub torpedo: bool,
    /// Another sonar's ping come back off a submarine (at the submarine's bearing): heard
    /// whatever its own noise, after the extra way the sound went round by it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub echo: bool,
}

/// A sensor's report of the last second: a waterfall's row.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Sample {
    /// Counts up with every report (a new row).
    pub seq: u16,
    /// The noise level (dB).
    pub noise: u8,
    pub lines: Vec<Line>,
    /// It pinged in this second.
    pub ping: bool,
    /// It heard another sonar's ping in this second (a DICASS or a dipping sonar near it).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub heard: bool,
    /// The host's: other sonars' pings and their echoes still on their way (seconds to go, the
    /// line).
    #[serde(skip)]
    pub due: Vec<(f32, Line)>,
}

/// A sonobuoy.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Buoy {
    pub id: u16,
    pub owner: u8,
    pub kind: BuoyKind,
    /// Where it floats (m east, north).
    pub at: [f64; 2],
    /// Seconds still falling on its chute (0: in the water).
    pub fall: f32,
    /// Its hydrophone's depth (m).
    pub depth: f32,
    /// A DICASS commanded to ping.
    pub active: bool,
    pub pings: u8,
    pub ping_in: f32,
    /// Seconds it still lives.
    pub life: f32,
    pub sample: Sample,
}

/// The dipping sonar's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DipState {
    /// Stowed in the cabin.
    #[default]
    Up,
    Lowering,
    /// At its set depth, listening or pinging.
    Down,
    Raising,
    /// Its cable parted: lost for the sortie.
    Broken,
}

/// A Seahawk's dipping sonar.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Dip {
    pub state: DipState,
    /// Its depth now and the depth set (m).
    pub depth: f32,
    pub set: f32,
    pub active: bool,
    pub ping_in: f32,
    /// Where it is in the water (m east, north).
    pub at: [f64; 2],
    pub sample: Sample,
}

impl Dip {
    pub fn wet(&self) -> bool {
        self.depth > 0.0 && self.state != DipState::Broken
    }
}

/// The Mk 54's search pattern after its enable delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Pattern {
    Straight,
    #[default]
    Circle,
    Snake,
    /// Circling while it moves up and down between its ceiling and floor.
    Helix,
}

impl Pattern {
    pub const ALL: [Pattern; 4] = [Pattern::Straight, Pattern::Circle, Pattern::Snake, Pattern::Helix];

    pub fn name(self) -> &'static str {
        match self {
            Pattern::Straight => "STRAIGHT",
            Pattern::Circle => "CIRCLE",
            Pattern::Snake => "SNAKE",
            Pattern::Helix => "HELIX",
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

/// A torpedo's program, set on the operator's weapons page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TorpProgram {
    /// Its initial heading (degrees true; None: the aircraft's heading at release).
    pub heading: Option<u16>,
    /// Seconds it runs out before it searches.
    pub delay: u8,
    pub pattern: Pattern,
    /// Its search depths (m): ceiling and floor.
    pub ceiling: u16,
    pub floor: u16,
}

impl Default for TorpProgram {
    fn default() -> Self {
        TorpProgram { heading: None, delay: 10, pattern: Pattern::Circle, ceiling: 30, floor: 250 }
    }
}

/// Where a torpedo is in its run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TorpPhase {
    /// Under its chute.
    #[default]
    Air,
    /// Running out on its heading.
    Run,
    Search,
    /// Homing on a submarine.
    Home,
}

/// An Mk 54.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Torpedo {
    pub id: u16,
    pub owner: u8,
    /// Position (m east, north, up: negative under the sea).
    pub p: [f64; 3],
    /// Heading (rad clockwise from north) and speed (m/s).
    pub heading: f64,
    pub speed: f64,
    pub phase: TorpPhase,
    /// Seconds in the water, seconds still in the air.
    pub t: f64,
    pub air: f64,
    pub program: TorpProgram,
    /// The search's start (m east, north) and heading.
    pub base: [f64; 2],
    pub base_heading: f64,
    /// The submarine it homes on (its id).
    pub target: Option<u16>,
}

/// What a submarine is out to do.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Intent {
    /// Slip through the line from `a` to `b` (m east, north) to `goal`, beyond it.
    Blockade { a: [f64; 2], b: [f64; 2], goal: [f64; 2] },
    /// Close on a ship (world object) and torpedo it.
    Approach { ship: usize },
    /// The same against the carrier.
    Attack { ship: usize },
    /// The same against one of this carrier's escorts: the nearest ship of its group, picked
    /// as the hunt starts.
    Escort { carrier: usize },
}

/// What a submarine is doing now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Posture {
    #[default]
    Transit,
    /// Up at periscope depth, snorkelling: noisy, its periscope's wake on the sea.
    Periscope,
    /// Heard a ping or a torpedo: sprinting away, deep under the layer.
    Evade,
    /// Its torpedoes away at its target.
    Attack,
    /// Stopped, everything shut down that can be: silent to a passive sensor (a ping still
    /// finds it). Every so often, and while a helicopter hovers near it.
    Silent,
}

/// A submarine (the host's alone).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sub {
    pub id: u16,
    pub at: [f64; 2],
    pub depth: f64,
    pub heading: f64,
    pub speed: f64,
    pub intent: Intent,
    pub posture: Posture,
    /// Seconds in the posture, seconds to the next periscope depth.
    pub posture_t: f64,
    pub pd_in: f64,
    /// Where the threat it evades is (m east, north).
    pub threat: Option<[f64; 2]>,
    /// Its noise at 5 kt (dB) and its machinery's tone (Hz).
    pub quiet: f64,
    pub machinery: u16,
    pub alive: bool,
    /// Through the blockade, or its torpedoes run home.
    pub done: bool,
    /// A campaign's: the player whose sortie hunts it (its sinking his objective).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hunter: Option<u8>,
    /// Seconds to its next silent stop, and how long it stays silent (from `posture_t` 0).
    #[serde(default)]
    pub silent_in: f64,
    #[serde(default)]
    pub silent_for: f64,
    /// A campaign's: the AI mission (its id) whose helicopters hunt it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_hunt: Option<u32>,
}

/// The AI's hunt of a campaign ASW mission (the host's): its submarine, its helicopters
/// (campaign tags), what they know of it.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AiHunt {
    pub mission: u32,
    pub sub: u16,
    pub tags: Vec<u32>,
    /// Where the submarine was reported (m east, north).
    pub datum: [f64; 2],
    /// One of its helicopters has been up.
    pub up: bool,
    /// Seconds searched near the submarine since the last fix or drop; where they hold it
    /// (m east, north), and since when.
    pub search: f64,
    pub fix: Option<[f64; 2]>,
    pub fix_t: f64,
    /// Torpedoes each aircraft (tag) dropped; the clock when the next may go.
    pub dropped: Vec<(u32, u8)>,
    pub next_drop: f64,
}

/// A fix on a submarine (m east, north): its error (m), when (s), how (1 an echo, 2 a MAD
/// mark).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Fix {
    pub at: [f64; 2],
    pub err: f32,
    pub t: f64,
    pub how: u8,
}

/// A track the automatic tracker keeps: its number (S1...), position and velocity (m, m/s
/// east, north) now, its error (m), the time of its last fix.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Track {
    pub n: u8,
    pub at: [f64; 2],
    pub vel: [f64; 2],
    pub err: f32,
    pub last: f64,
}

/// A Seahawk crew's ASW state: its route (shared by the pilot and the operator), the torpedo
/// program, the buoys left, the dipping sonar, the MAD, the tracker's work.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AswCrew {
    /// The pilot whose aircraft it is.
    pub owner: u8,
    /// The route (world units east, north) and its revision (counts up with each change).
    pub route: Vec<(i32, i32)>,
    pub route_rev: u16,
    /// The mission's own waypoints after it, as the pilot's aircraft has them (for the operator).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mission: Vec<(i32, i32)>,
    /// A campaign barrier hunt's line (m east, north): the submarine must not slip past it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barrier: Option<([f64; 2], [f64; 2])>,
    pub program: TorpProgram,
    /// The release is armed (the operator's ARM; the pilot's own pickle needs none).
    pub armed: bool,
    pub buoys_left: [u8; 3],
    /// Drops waiting for the aircraft to reach their point: kind, deep, where (m east, north).
    pub pending: Vec<(BuoyKind, bool, [f64; 2])>,
    pub dip: Dip,
    /// The operator asks for the dip hover (the AI flying comes down to it; a player pilot is
    /// told).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hover_req: bool,
    pub mad_down: bool,
    /// MAD signal (tenths of nT) now, and its marks.
    pub mad: i16,
    pub fixes: Vec<Fix>,
    pub tracks: Vec<Track>,
    /// The layer's depth (m), once a BT reported it.
    pub layer: Option<u16>,
    /// The torpedoes' tracks for the map (where each one is, id).
    pub next_track: u8,
    /// Was the MAD over the mark threshold (an automatic mark per pass).
    #[serde(default)]
    pub mad_over: bool,
}

/// The outcome of the hunt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AswResult {
    #[default]
    Hunting,
    /// Every submarine sunk.
    Won,
    /// A ship was torpedoed.
    ShipLost,
    /// A submarine got through the blockade.
    Escaped,
}

/// A mission's submarine: where it starts (m east, north), its heading (rad) and what it is
/// out to do; a nuclear boat is louder and faster to sprint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SubSetup {
    pub at: [f64; 2],
    pub heading: f64,
    pub intent: Intent,
    pub nuclear: bool,
}

/// What a campaign's hunted submarine is out to do: close on a carrier's group (the carrier's
/// world object), or slip out past the line between two friendly bases (world objects), from
/// the side of the enemy's point `enemy` (map units).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuntTarget {
    Carrier(usize),
    Barrier { a: usize, b: usize, enemy: (i32, i32) },
}

/// What a mission's hunt starts with.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AswSetup {
    pub ocean: Ocean,
    pub subs: Vec<SubSetup>,
    pub seed: u32,
}

/// The whole anti-submarine picture.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Asw {
    pub ocean: Ocean,
    /// The host's alone (not sent).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subs: Vec<Sub>,
    pub buoys: Vec<Buoy>,
    pub torpedoes: Vec<Torpedo>,
    pub crews: Vec<AswCrew>,
    /// Periscope wakes on the sea (m east, north): a submarine at periscope depth.
    pub wakes: Vec<[f64; 2]>,
    pub result: AswResult,
    /// Seconds since the hunt began; the second the sensors last reported.
    pub clock: f64,
    pub next_id: u16,
    pub rng: Option<Rng>,
    /// The host's: the AI's hunts of a campaign's ASW missions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ai_hunts: Vec<AiHunt>,
}

impl Asw {
    pub fn is_empty(&self) -> bool {
        self.crews.is_empty() && self.subs.is_empty() && self.buoys.is_empty() && self.torpedoes.is_empty()
    }

    /// What the other simulations get: no submarines.
    pub fn for_clients(&self) -> Asw {
        let mut a = Asw { subs: Vec::new(), rng: None, ai_hunts: Vec::new(), ..self.clone() };
        for c in &mut a.crews {
            // The newest fixes for the map.
            let n = c.fixes.len();
            c.fixes.drain(..n.saturating_sub(CLIENT_FIXES));
        }
        a
    }

    fn id(&mut self) -> u16 {
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.next_id
    }

    pub fn crew(&self, owner: u8) -> Option<&AswCrew> {
        self.crews.iter().find(|c| c.owner == owner)
    }

    fn crew_mut(&mut self, owner: u8) -> &mut AswCrew {
        if let Some(i) = self.crews.iter().position(|c| c.owner == owner) {
            return &mut self.crews[i];
        }
        self.crews.push(AswCrew { owner, buoys_left: BUOY_LOAD, ..Default::default() });
        self.crews.last_mut().expect("pushed")
    }

    /// The buoys of `owner` in the water or falling.
    pub fn buoys_of(&self, owner: u8) -> impl Iterator<Item = &Buoy> {
        self.buoys.iter().filter(move |b| b.owner == owner)
    }

    fn rand(&mut self, n: i32) -> i32 {
        self.rng.get_or_insert(Rng::new(0x5EA_4A3C)).random(n)
    }

    /// A normal-ish draw (mean 0, standard deviation 1).
    fn gauss(&mut self) -> f64 {
        (0..4).map(|_| self.rand(1000) as f64 / 1000.0).sum::<f64>() * 1.732 - 3.464
    }
}

/// A sensor command from a Seahawk's crew (the pilot or the operator).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AswCmd {
    /// Drop a buoy (deep: its hydrophone at the deeper setting) here, or at a point (m east,
    /// north), dropped as the aircraft passes over it (the point added to the route's end).
    DropBuoy { kind: BuoyKind, deep: bool, at: Option<(i32, i32)> },
    /// A DICASS listens or pings.
    BuoyMode { id: u16, active: bool },
    Scuttle { id: u16 },
    /// The operator asks the pilot for the dip hover (40 ft over the sea), or no longer.
    Hover(bool),
    /// Lower or raise the dipping sonar.
    Dip(bool),
    /// Its depth set (m).
    DipDepth(u16),
    DipMode { active: bool },
    /// The MAD towed body out or in.
    Mad(bool),
    /// Mark the submarine under the aircraft (a fix where it is).
    Mark,
    Program(TorpProgram),
    Arm(bool),
    /// The operator drops the torpedo on this station (armed).
    Release { station: u8 },
    /// The route, whole (world units east, north), at this revision, and the buoys to drop at
    /// its points (the point's index, the kind, deep): these replace the drops waiting.
    Route { rev: u16, points: Vec<(i32, i32)>, drops: Vec<(u8, BuoyKind, bool)> },
    /// The pilot's client: the mission's waypoints after the route (world units east, north).
    Mission { points: Vec<(i32, i32)> },
}

/// A Seahawk as the sensors need it: position (m east, north), height above the sea (m), speed
/// (m/s), heading (rad) and velocity (m/s east, north).
#[derive(Debug, Clone, Copy)]
struct Heli {
    id: u8,
    at: [f64; 2],
    height: f64,
    speed: f64,
    heading: f64,
    vel: [f64; 2],
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Bearing (rad clockwise from north) from `a` to `b`.
fn bearing(a: [f64; 2], b: [f64; 2]) -> f64 {
    (b[0] - a[0]).atan2(b[1] - a[1]).rem_euclid(TAU)
}

fn wrap(a: f64) -> f64 {
    (a + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI
}

fn deg(a: f64) -> u16 {
    ((a.rem_euclid(TAU) / DEG).round() as u16) % 360
}

/// A world object's position (m east, north).
pub fn object_metres(o: &GroundObject) -> [f64; 2] {
    [(o.x as f64 + 0.5) * M_PER_MAP, ((0x8000 - o.y) as f64 - 0.5) * M_PER_MAP]
}

/// A map point (map units, x east, y south) in metres east, north.
pub fn map_metres(p: (i32, i32)) -> [f64; 2] {
    [p.0 as f64 * M_PER_MAP, (0x8000 - p.1) as f64 * M_PER_MAP]
}

impl AswSetup {
    /// A mission's hunt (`stealth_mission::asw`), if it has one.
    pub fn of(m: &stealth_mission::Mission) -> Option<Self> {
        use stealth_mission::asw::SubIntent as I;
        let h = m.hunt.as_ref()?;
        let subs = h
            .subs
            .iter()
            .map(|s| SubSetup {
                at: map_metres(s.at),
                heading: (s.heading as f64).to_radians(),
                intent: match s.intent {
                    I::Blockade { a, b, goal } => {
                        Intent::Blockade { a: map_metres(a), b: map_metres(b), goal: map_metres(goal) }
                    }
                    I::Approach { object } => Intent::Approach { ship: object.max(0) as usize },
                    I::Attack { object } => Intent::Attack { ship: object.max(0) as usize },
                    I::Escort { carrier } => Intent::Escort { carrier: carrier.max(0) as usize },
                },
                nuclear: s.nuclear,
            })
            .collect();
        let seed = (m.rng.holdrand_entry as u32) ^ (m.rng.holdrand_exit as u32).rotate_left(16);
        Some(AswSetup {
            ocean: Ocean {
                layer: h.layer as f32,
                sea_state: h.sea_state,
                bottom: if h.bottom > 0 { h.bottom as f32 } else { 600.0 },
            },
            subs,
            seed,
        })
    }
}

impl Sub {
    fn from_setup(id: u16, s: &SubSetup) -> Self {
        Sub {
            id,
            at: s.at,
            depth: 90.0,
            heading: s.heading,
            speed: TRANSIT,
            intent: s.intent,
            posture: Posture::Transit,
            posture_t: 0.0,
            pd_in: 900.0,
            threat: None,
            quiet: if s.nuclear { 122.0 } else { 116.0 },
            machinery: if s.nuclear { 60 } else { 50 },
            alive: true,
            done: false,
            hunter: None,
            silent_in: 1000.0,
            silent_for: 0.0,
            ai_hunt: None,
        }
    }

    /// Its noise (dB): louder with speed, much louder cavitating or snorkelling; stopped and
    /// silent next to nothing.
    pub fn level(&self) -> f64 {
        let kt = self.speed / KT;
        if self.posture == Posture::Silent {
            return self.quiet - SILENT_LOSS + 20.0 * (kt.max(0.1) / 5.0).log10().max(-3.0) * 0.5;
        }
        let mut sl = self.quiet + 20.0 * (kt.max(2.0) / 5.0).log10();
        if kt > 12.0 && self.depth < 80.0 {
            sl += 15.0;
        }
        if self.posture == Posture::Periscope {
            sl += 20.0;
        }
        sl
    }

    /// Its tonal lines (Hz): the machinery, the blade rate.
    fn freqs(&self) -> [u16; 2] {
        [self.machinery, (6.0 + self.speed / KT * 1.4).round() as u16]
    }
}

impl Sim {
    /// The anti-submarine picture (a client's: without the submarines).
    pub fn asw(&self) -> &Asw {
        &self.asw
    }

    /// Starts a mission's hunt: its water and submarines.
    pub fn start_asw(&mut self, s: &AswSetup) {
        self.asw.ocean = s.ocean;
        self.asw.rng = Some(Rng::new(s.seed | 1));
        for sub in &s.subs {
            let id = self.asw.id();
            let mut sub = *sub;
            if let Intent::Escort { carrier } = sub.intent {
                sub.intent = match self.escort_of(carrier, sub.at) {
                    Some(ship) => Intent::Approach { ship },
                    None => Intent::Attack { ship: carrier },
                };
            }
            self.asw.subs.push(Sub::from_setup(id, &sub));
        }
        for i in 0..self.asw.subs.len() {
            self.asw.subs[i].pd_in = (PD_EVERY.0 + self.asw.rand(PD_EVERY.1 - PD_EVERY.0)) as f64;
            self.asw.subs[i].silent_in = (SILENT_EVERY.0 + self.asw.rand(SILENT_EVERY.1 - SILENT_EVERY.0)) as f64;
        }
    }

    /// The carrier's escort nearest `from` (m east, north): a ship of its group, the theater's
    /// ships near it (`ESCORT_REACH` map units).
    fn escort_of(&self, carrier: usize, from: [f64; 2]) -> Option<usize> {
        const ESCORT_REACH: i32 = 768;
        let c = *self.objects.get(carrier)?;
        (0..self.objects.len())
            .filter(|&i| i != carrier && self.is_ship(i as i32))
            .filter(|&i| {
                let o = &self.objects[i];
                (o.x - c.x).abs() < ESCORT_REACH && (o.y - c.y).abs() < ESCORT_REACH && o.flags & oflags::CARRIER == 0
            })
            .min_by(|&a, &b| {
                dist(object_metres(&self.objects[a]), from).total_cmp(&dist(object_metres(&self.objects[b]), from))
            })
    }

    /// A campaign sortie's hunt: a submarine for player `player`, closing on a carrier's group
    /// or making for a barrier line (`spawn_hunted`).
    pub fn start_hunt_for(&mut self, player: u8, target: HuntTarget, seed: u32) {
        // Gone: the one hunted before it.
        self.asw.subs.retain(|s| s.hunter != Some(player));
        let Some((i, datum)) = self.spawn_hunted(target, seed) else { return };
        self.asw.subs[i].hunter = Some(player);
        let barrier = match self.asw.subs[i].intent {
            Intent::Blockade { a, b, .. } => Some((a, b)),
            _ => None,
        };
        // Where it was reported: the crew's first waypoint.
        let c = self.asw.crew_mut(player);
        c.route = vec![((datum[0] / M_PER_WU) as i32, (datum[1] / M_PER_WU) as i32)];
        c.route_rev = c.route_rev.wrapping_add(1);
        c.barrier = barrier;
    }

    /// A campaign's ASW mission the AI flies (`mission` its id): a submarine as for a player,
    /// hunted by the mission's helicopters (campaign tags; each joins the hunt as it takes
    /// off).
    pub fn start_ai_hunt(&mut self, mission: u32, tag: u32, target: HuntTarget, seed: u32) {
        if let Some(h) = self.asw.ai_hunts.iter_mut().find(|h| h.mission == mission) {
            if !h.tags.contains(&tag) {
                h.tags.push(tag);
            }
            return;
        }
        let Some((i, datum)) = self.spawn_hunted(target, seed) else { return };
        self.asw.subs[i].ai_hunt = Some(mission);
        let sub = self.asw.subs[i].id;
        self.asw.ai_hunts.push(AiHunt { mission, sub, tags: vec![tag], datum, ..Default::default() });
    }

    /// A campaign's hunted submarine: some 22-28 km off the carrier, closing on one of its
    /// escorts (else the carrier); or as far out on the enemy's side of the barrier line,
    /// making for the open sea beyond it. Its index, and where it is reported (a few km off).
    fn spawn_hunted(&mut self, target: HuntTarget, seed: u32) -> Option<(usize, [f64; 2])> {
        let barrier = match target {
            HuntTarget::Barrier { a, b, enemy } => {
                let (a, b) = (self.objects.get(a).map(object_metres)?, self.objects.get(b).map(object_metres)?);
                Some((a, b, map_metres(enemy)))
            }
            HuntTarget::Carrier(_) => None,
        };
        if self.asw.rng.is_none() {
            self.asw.rng = Some(Rng::new(seed | 1));
            let layer = 30 + self.asw.rand(90);
            self.asw.ocean = Ocean { layer: layer as f32, sea_state: 1 + self.asw.rand(4) as u8, bottom: 600.0 };
            // A barrier across a narrow sea: shallow water (the Gulf of Finland's), the layer
            // high in it.
            if barrier.is_some() {
                self.asw.ocean.layer = (15 + self.asw.rand(20)) as f32;
                self.asw.ocean.bottom = 100.0;
            }
        }
        let open = |sim: &Sim, p: [f64; 2]| {
            sim.sea_at(p)
                && (0..8).all(|k| {
                    let b = k as f64 * 45.0 * DEG;
                    sim.sea_at([p[0] + b.sin() * 1500.0, p[1] + b.cos() * 1500.0])
                })
        };
        let nuclear = self.asw.rand(2) == 0;
        let (at, sub) = match (target, barrier) {
            (HuntTarget::Barrier { .. }, Some((a, b, enemy))) => {
                let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                let len = dist(a, b).max(1.0);
                let along = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
                // Across the line toward the enemy.
                let mut n = [-along[1], along[0]];
                if (enemy[0] - mid[0]) * n[0] + (enemy[1] - mid[1]) * n[1] < 0.0 {
                    n = [-n[0], -n[1]];
                }
                let at = (0..64).find_map(|_| {
                    let s = (self.asw.rand(700) as f64 / 1000.0 - 0.35) * len;
                    let d = 22_000.0 + self.asw.rand(6000) as f64;
                    let p = [mid[0] + along[0] * s + n[0] * d, mid[1] + along[1] * s + n[1] * d];
                    open(self, p).then_some(p)
                })?;
                // Beyond the line: the open sea it makes for.
                let goal = [20_000.0, 35_000.0, 10_000.0]
                    .iter()
                    .map(|&d| [mid[0] - n[0] * d, mid[1] - n[1] * d])
                    .find(|&g| self.sea_at(g))
                    .unwrap_or([mid[0] - n[0] * 20_000.0, mid[1] - n[1] * 20_000.0]);
                let setup = SubSetup { at, heading: bearing(at, goal), intent: Intent::Blockade { a, b, goal }, nuclear };
                (at, setup)
            }
            (HuntTarget::Carrier(carrier), _) => {
                let c = self.objects.get(carrier).map(object_metres)?;
                let at = (0..64).find_map(|_| {
                    let a = self.asw.rand(3600) as f64 / 10.0 * DEG;
                    let d = 22_000.0 + self.asw.rand(6000) as f64;
                    let p = [c[0] + a.sin() * d, c[1] + a.cos() * d];
                    open(self, p).then_some(p)
                })?;
                let intent = match self.escort_of(carrier, at) {
                    Some(ship) => Intent::Approach { ship },
                    None => Intent::Attack { ship: carrier },
                };
                (at, SubSetup { at, heading: bearing(at, c), intent, nuclear })
            }
            _ => return None,
        };
        let id = self.asw.id();
        let mut s = Sub::from_setup(id, &sub);
        s.depth = s.depth.min(self.asw.ocean.bottom as f64 - 20.0);
        s.pd_in = (PD_EVERY.0 + self.asw.rand(PD_EVERY.1 - PD_EVERY.0)) as f64;
        s.silent_in = (SILENT_EVERY.0 + self.asw.rand(SILENT_EVERY.1 - SILENT_EVERY.0)) as f64;
        self.asw.subs.push(s);
        let off = self.asw.rand(3000) as f64;
        let a = self.asw.rand(3600) as f64 / 10.0 * DEG;
        Some((self.asw.subs.len() - 1, [at[0] + a.sin() * off, at[1] + a.cos() * off]))
    }

    /// Where a flight unit is (m east, north).
    fn flight_metres(&self, i: usize) -> [f64; 2] {
        let u = &self.flights[i];
        map_metres((u.x, u.y))
    }

    /// Where an AI helicopter of a hunt flies (map units), and whether it hovers there (over
    /// its fix: dipping, then dropping); None for other aircraft.
    pub(crate) fn asw_ai_goal(&self, i: usize) -> Option<((i32, i32), bool)> {
        let tag = self.campaign_flight(i)?.tag;
        let h = self.asw.ai_hunts.iter().find(|h| h.tags.contains(&tag))?;
        let to_map = |p: [f64; 2]| ((p[0] / M_PER_MAP) as i32, 0x8000 - (p[1] / M_PER_MAP) as i32);
        if let Some(f) = h.fix {
            return Some((to_map(f), true));
        }
        // Round the datum, each of them on its own side of the circle.
        let k = h.tags.iter().position(|&t| t == tag).unwrap_or(0) as f64;
        let a = self.asw.clock * TAU / 160.0 + k * std::f64::consts::PI;
        Some((to_map([h.datum[0] + a.sin() * AI_ORBIT, h.datum[1] + a.cos() * AI_ORBIT]), false))
    }

    /// Once a second: the AI's hunts. Its helicopters search, hold the submarine and drop
    /// their torpedoes from the hover over it; with the submarine sunk (the mission's
    /// objective) or through, or all its helicopters gone, the hunt is over.
    fn update_ai_hunts(&mut self, dt: f64) {
        let clock = self.asw.clock;
        let mut k = 0;
        while k < self.asw.ai_hunts.len() {
            let mut h = self.asw.ai_hunts[k].clone();
            let sub = self.asw.subs.iter().position(|s| s.id == h.sub);
            let flying: Vec<(u32, usize)> = h.tags.iter().filter_map(|&t| Some((t, self.flight_of_tag(t)?))).collect();
            let over = sub.is_none_or(|i| !self.asw.subs[i].alive || self.asw.subs[i].done);
            if over || h.up && flying.is_empty() {
                // Home; a submarine nobody hunts any more withdraws.
                for &(_, i) in &flying {
                    let home = self.campaign_flight(i).map_or(self.flights[i].wp, |f| f.home);
                    self.flights[i].flags |= fflags::RTB;
                    self.flights[i].wp = home;
                }
                if let Some(i) = sub.filter(|&i| self.asw.subs[i].alive && !self.asw.subs[i].done) {
                    self.asw.subs.remove(i);
                }
                self.asw.ai_hunts.remove(k);
                continue;
            }
            let at = self.asw.subs[sub.expect("not over")].at;
            h.up |= !flying.is_empty();
            if flying.iter().any(|&(_, i)| dist(self.flight_metres(i), at) < AI_SEARCH_RANGE) {
                h.search += dt;
            }
            let stale = h.fix.is_none_or(|_| clock - h.fix_t > AI_REFIX);
            if stale && h.search >= AI_LOCALIZE {
                let (e, n) = (self.asw.gauss(), self.asw.gauss());
                h.fix = Some([at[0] + e * AI_FIX_ERR, at[1] + n * AI_FIX_ERR]);
                h.fix_t = clock;
                h.search = 0.0;
            }
            if let Some(f) = h.fix.filter(|_| clock >= h.next_drop) {
                for &(tag, i) in &flying {
                    let n = h.dropped.iter().find(|d| d.0 == tag).map_or(0, |d| d.1);
                    let p = self.flight_metres(i);
                    if n >= AI_TORPEDOES || dist(p, f) > AI_DROP_RANGE {
                        continue;
                    }
                    let id = self.asw.id();
                    let heading = bearing(p, f);
                    let program = TorpProgram { heading: None, delay: 2, pattern: Pattern::Circle, ceiling: 30, floor: 250 };
                    self.asw.torpedoes.push(Torpedo {
                        id,
                        owner: AI_OWNER,
                        p: [p[0], p[1], 20.0],
                        heading,
                        speed: 0.0,
                        phase: TorpPhase::Air,
                        t: 0.0,
                        air: 1.5,
                        program,
                        base: p,
                        base_heading: heading,
                        target: None,
                    });
                    match h.dropped.iter_mut().find(|d| d.0 == tag) {
                        Some(d) => d.1 += 1,
                        None => h.dropped.push((tag, 1)),
                    }
                    h.next_drop = clock + AI_DROP_EVERY;
                    h.search = 0.0;
                    break;
                }
            }
            // Out of torpedoes: home (the submarine runs on, the mission failed).
            let left = flying.iter().any(|&(t, _)| h.dropped.iter().find(|d| d.0 == t).map_or(0, |d| d.1) < AI_TORPEDOES);
            let running = self.asw.torpedoes.iter().any(|t| t.owner == AI_OWNER);
            if !flying.is_empty() && !left && !running {
                for &(_, i) in &flying {
                    let home = self.campaign_flight(i).map_or(self.flights[i].wp, |f| f.home);
                    self.flights[i].flags |= fflags::RTB;
                    self.flights[i].wp = home;
                }
            }
            self.asw.ai_hunts[k] = h;
            k += 1;
        }
    }

    /// Test aid: a buoy of `player`'s put in the water at `at` (m east, north) at once.
    pub fn test_buoy_at(&mut self, player: u8, kind: BuoyKind, deep: bool, at: [f64; 2]) -> Result<(), Text> {
        let mut h = self.heli(player as usize);
        h.at = at;
        h.height = 0.0;
        h.vel = [0.0, 0.0];
        self.drop_buoy(&h, kind, deep)
    }

    /// Moves submarine `i` (test aid).
    pub fn place_sub(&mut self, i: usize, at: [f64; 2]) {
        if let Some(s) = self.asw.subs.get_mut(i) {
            s.at = at;
        }
    }

    /// The Seahawk crew `player` belongs to: his own aircraft's, or the one he is the
    /// operator of.
    pub fn asw_owner(&self, player: u8) -> Option<u8> {
        let flies = |id: u8| self.pilot_exists(id as usize) && self.pilot_ref(id as usize).seahawk;
        if flies(player) && !self.pilot_absent(player) {
            return Some(player);
        }
        self.crew_of(player).map(|(o, _)| o).filter(|&o| flies(o))
    }

    /// Pilot `k` is in the simulation.
    fn pilot_exists(&self, k: usize) -> bool {
        k < self.pilot_count() && self.pilot_ref(k).id as usize == k
    }

    /// A crew member's sensor command.
    pub fn asw_command(&mut self, player: u8, cmd: AswCmd) -> Result<(), Text> {
        let Some(owner) = self.asw_owner(player) else { return Err(text!("asw-not-seahawk")) };
        let heli = self.heli(owner as usize);
        match cmd {
            AswCmd::DropBuoy { kind, deep, at } => {
                let c = self.asw.crew_mut(owner);
                if c.buoys_left[kind.index()] == 0 {
                    return Err(text!("asw-no-buoys", kind = kind.name()));
                }
                let live = self.asw.buoys_of(owner).count() + self.asw.crew(owner).map_or(0, |c| c.pending.len());
                if live >= MAX_BUOYS {
                    return Err(text!("asw-buoys-live", n = MAX_BUOYS as i64));
                }
                let waiting = self
                    .asw
                    .crew(owner)
                    .map_or(0, |c| c.pending.iter().filter(|d| d.0 == kind).count());
                if at.is_some() && waiting >= c_left(self, owner, kind) {
                    return Err(text!("asw-no-buoys", kind = kind.name()));
                }
                let point = at.map(|(e, n)| [e as f64, n as f64]);
                match point {
                    Some(p) if dist(p, heli.at) > DROP_RADIUS => {
                        // A waypoint for it: the drop waits at that point of the route.
                        let c = self.asw.crew_mut(owner);
                        if c.route.len() >= MAX_ROUTE {
                            return Err(text!("asw-route-full"));
                        }
                        let wp = wu_point(p);
                        c.route.push(wp);
                        c.route_rev = c.route_rev.wrapping_add(1);
                        c.pending.push((kind, deep, wu_metres(wp)));
                        Ok(())
                    }
                    _ => self.drop_buoy(&heli, kind, deep),
                }
            }
            AswCmd::BuoyMode { id, active } => {
                let b = self
                    .asw
                    .buoys
                    .iter_mut()
                    .find(|b| b.id == id && b.owner == owner && b.kind == BuoyKind::Dicass)
                    .ok_or(text!("asw-no-such-buoy"))?;
                b.active = active;
                b.ping_in = b.ping_in.min(1.0);
                Ok(())
            }
            AswCmd::Scuttle { id } => {
                let n = self.asw.buoys.len();
                self.asw.buoys.retain(|b| !(b.id == id && b.owner == owner));
                if self.asw.buoys.len() == n { Err(text!("asw-no-such-buoy")) } else { Ok(()) }
            }
            AswCmd::Hover(on) => {
                let c = self.asw.crew_mut(owner);
                let changed = c.hover_req != on;
                c.hover_req = on;
                // Another player at the console: the pilot is told.
                if changed && player != owner {
                    let back = self.pl.id as usize;
                    self.activate(owner as usize);
                    self.msg(if on { text!("asw-hover-request") } else { text!("asw-hover-cancelled") });
                    self.activate(back);
                }
                Ok(())
            }
            AswCmd::Dip(down) => {
                let d = &mut self.asw.crew_mut(owner).dip;
                match (down, d.state) {
                    (_, DipState::Broken) => Err(text!("asw-dip-broken")),
                    (true, DipState::Up | DipState::Raising) => {
                        if heli.height > DIP_HOVER || heli.speed > DIP_SPEED {
                            return Err(text!("asw-dip-limits"));
                        }
                        if !self.sea_at(heli.at) {
                            return Err(text!("asw-not-over-water"));
                        }
                        let d = &mut self.asw.crew_mut(owner).dip;
                        d.state = DipState::Lowering;
                        if d.set <= 0.0 {
                            d.set = 60.0;
                        }
                        Ok(())
                    }
                    (false, DipState::Down | DipState::Lowering) => {
                        d.state = DipState::Raising;
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            AswCmd::DipDepth(m) => {
                let bottom = self.asw.ocean.bottom;
                let d = &mut self.asw.crew_mut(owner).dip;
                d.set = (m as f32).clamp(10.0, DIP_MAX.min(bottom - 5.0));
                if d.state == DipState::Down {
                    d.state = DipState::Lowering;
                }
                Ok(())
            }
            AswCmd::DipMode { active } => {
                let d = &mut self.asw.crew_mut(owner).dip;
                d.active = active;
                d.ping_in = d.ping_in.min(1.0);
                Ok(())
            }
            AswCmd::Mad(down) => {
                if down && heli.speed > MAD_SPEED {
                    return Err(text!("asw-mad-limits"));
                }
                self.asw.crew_mut(owner).mad_down = down;
                Ok(())
            }
            AswCmd::Mark => {
                let t = self.asw.clock;
                let c = self.asw.crew_mut(owner);
                c.fixes.push(Fix { at: heli.at, err: 150.0, t, how: 2 });
                self.tracker(owner as usize);
                Ok(())
            }
            AswCmd::Program(p) => {
                let p = TorpProgram {
                    heading: p.heading.map(|h| h % 360),
                    delay: p.delay.min(60),
                    ceiling: p.ceiling.clamp(10, 400),
                    floor: p.floor.clamp(p.ceiling.clamp(10, 400) + 10, 450),
                    ..p
                };
                self.asw.crew_mut(owner).program = p;
                Ok(())
            }
            AswCmd::Arm(on) => {
                self.asw.crew_mut(owner).armed = on;
                Ok(())
            }
            AswCmd::Release { station } => {
                if !self.asw.crew(owner).is_some_and(|c| c.armed) {
                    return Err(text!("asw-not-armed"));
                }
                let back = self.pl.id as usize;
                self.activate(owner as usize);
                let ok = self
                    .pl
                    .bays
                    .get(station as usize)
                    .is_some_and(|&(w, n)| w == MK54_WEAPON && n > 0);
                if ok {
                    self.pl.crew_launch = Some(station);
                }
                self.activate(back);
                if ok { Ok(()) } else { Err(text!("asw-no-torpedo")) }
            }
            AswCmd::Mission { points } => {
                self.asw.crew_mut(owner).mission = points.into_iter().take(32).collect();
                Ok(())
            }
            AswCmd::Route { rev, points, drops } => {
                let points: Vec<(i32, i32)> = points.into_iter().take(MAX_ROUTE).collect();
                let drops: Vec<(BuoyKind, bool, [f64; 2])> = drops
                    .into_iter()
                    .filter_map(|(i, k, d)| points.get(i as usize).map(|&p| (k, d, wu_metres(p))))
                    .collect();
                // Only the buoys left, and no more in the water and waiting than the receivers
                // follow.
                for kind in [BuoyKind::Difar, BuoyKind::Dicass, BuoyKind::Bt] {
                    if drops.iter().filter(|d| d.0 == kind).count() > c_left(self, owner, kind) {
                        return Err(text!("asw-no-buoys", kind = kind.name()));
                    }
                }
                if self.asw.buoys_of(owner).count() + drops.len() > MAX_BUOYS {
                    return Err(text!("asw-buoys-live", n = MAX_BUOYS as i64));
                }
                let c = self.asw.crew_mut(owner);
                // Last writer wins: a change made on the older route is still taken.
                c.route = points;
                c.route_rev = rev.max(c.route_rev.wrapping_add(1));
                c.pending = drops;
                Ok(())
            }
        }
    }

    /// The sea at a point (m east, north): the host's water (a test's without any: all sea).
    fn sea_at(&self, at: [f64; 2]) -> bool {
        if !self.water.known() {
            return true;
        }
        let x = (at[0] / M_PER_MAP) as i32;
        let y = 0x8000 - (at[1] / M_PER_MAP) as i32;
        self.water.wet(x, y)
    }

    fn heli(&self, k: usize) -> Heli {
        let p = &self.pilot_ref(k).p;
        let at = [p.wx as f64 * M_PER_WU, p.wy as f64 * M_PER_WU];
        let ground = self.hill_alt(p.mx, p.my);
        let height = ((p.alt - ground).max(0)) as f64 * M_PER_WU;
        let heading = (p.heading as u16) as f64 / 65536.0 * TAU;
        let speed = p.knots as f64 * KT;
        Heli { id: k as u8, at, height, speed, heading, vel: [heading.sin() * speed, heading.cos() * speed] }
    }

    fn drop_buoy(&mut self, h: &Heli, kind: BuoyKind, deep: bool) -> Result<(), Text> {
        if !self.sea_at(h.at) {
            return Err(text!("asw-not-over-water"));
        }
        let fall = (h.height / BUOY_SINK).max(BUOY_FALL_MIN);
        // It drifts on with the aircraft's speed while the chute slows it.
        let drift = fall.min(3.0) * 0.5;
        let at = [h.at[0] + h.vel[0] * drift, h.at[1] + h.vel[1] * drift];
        let id = self.asw.id();
        let c = self.asw.crew_mut(h.id);
        c.buoys_left[kind.index()] -= 1;
        self.asw.buoys.push(Buoy {
            id,
            owner: h.id,
            kind,
            at,
            fall: fall as f32,
            depth: BUOY_DEPTHS[deep as usize].min(self.asw.ocean.bottom - 5.0),
            active: false,
            pings: if kind == BuoyKind::Dicass { DICASS_PINGS } else { 0 },
            ping_in: 0.0,
            life: if kind == BuoyKind::Bt { BT_LIFE } else { BUOY_LIFE },
            sample: Sample::default(),
        });
        self.sound(0x1F, 1);
        Ok(())
    }

    /// The Mk 54 leaves its station: under its chute, then into the sea.
    pub(crate) fn drop_torpedo(&mut self) {
        let k = self.pl.id as usize;
        let h = self.heli(k);
        let program = self.asw.crew(h.id).map(|c| c.program).unwrap_or_default();
        let id = self.asw.id();
        let air = (h.height / 12.0).max(1.5);
        let heading = program.heading.map_or(h.heading, |d| d as f64 * DEG);
        let alt = h.height;
        self.asw.torpedoes.push(Torpedo {
            id,
            owner: h.id,
            p: [h.at[0], h.at[1], alt],
            heading,
            speed: h.speed,
            phase: TorpPhase::Air,
            t: 0.0,
            air,
            program,
            base: h.at,
            base_heading: heading,
            target: None,
        });
        self.asw.crew_mut(h.id).armed = false;
        self.msg(text!("asw-torpedo-away"));
        self.sound(0x0D, 1);
    }

    /// Can the active pilot release a torpedo now (over the sea, low and slow enough)?
    pub(crate) fn torpedo_release_check(&self) -> Result<(), Text> {
        let h = self.heli(self.pl.id as usize);
        if !self.sea_at(h.at) {
            return Err(text!("asw-not-over-water"));
        }
        if h.height > TORP_MAX_HEIGHT || h.speed > TORP_MAX_SPEED {
            return Err(text!("asw-torpedo-limits"));
        }
        Ok(())
    }

    /// A Hellfire struck the sea at (m east, north): a submarine at periscope depth under it
    /// sinks.
    pub(crate) fn hellfire_on_sea(&mut self, at: [f64; 2]) -> bool {
        let hit = self.asw.subs.iter().position(|s| {
            s.alive && s.depth <= PERISCOPE_DEPTH + 2.0 && dist(s.at, at) < HELLFIRE_SUB_RADIUS
        });
        if let Some(i) = hit {
            self.sink(i);
            return true;
        }
        false
    }

    fn sink(&mut self, i: usize) {
        let s = &mut self.asw.subs[i];
        s.alive = false;
        let (at, hunter, ai) = (s.at, s.hunter, s.ai_hunt);
        // The AI's: its mission achieved.
        if let Some(tag) = ai.and_then(|m| self.asw.ai_hunts.iter().find(|h| h.mission == m)).and_then(|h| h.tags.first().copied()) {
            self.emit(SimEvent::CampaignSubSunk { tag });
        }
        // A campaign's: the hunter's objective.
        if let Some(p) = hunter.filter(|&p| self.pilot_exists(p as usize)) {
            let back = self.pl.id as usize;
            self.activate(p as usize);
            if self.pl.targets[0].mission_type == HUNT_MISSION {
                self.mark_target_reached(0);
            }
            self.activate(back);
        }
        self.emit(SimEvent::AswSplash { at: [(at[0] * 10.0) as i32, (at[1] * 10.0) as i32], kind: 2 });
        self.tell_crews(text!("asw-sub-sunk"));
    }

    fn tell_crews(&mut self, t: Text) {
        let owners: Vec<u8> = self.asw.crews.iter().map(|c| c.owner).collect();
        let back = self.pl.id as usize;
        for o in owners {
            if self.pilot_exists(o as usize) {
                self.activate(o as usize);
                self.msg(t.clone());
                // Its operator hears it too.
                if let Some(g) = self.gunner_of(o) {
                    self.activate(g as usize);
                    self.msg(t.clone());
                }
            }
        }
        self.activate(back);
    }

    /// One frame of the hunt: the submarines, buoys, the dipping sonar, the MAD, the torpedoes;
    /// once a second the sensors report and the tracker works.
    pub(crate) fn update_asw(&mut self, present: &[usize]) {
        if self.asw.is_empty() || self.mirror {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let before = self.asw.clock;
        self.asw.clock += dt;
        let second = before.floor() != self.asw.clock.floor();
        let helis: Vec<Heli> =
            present.iter().filter(|&&k| self.pilot_ref(k).seahawk).map(|&k| self.heli(k)).collect();
        for h in &helis {
            self.asw.crew_mut(h.id);
        }
        self.move_subs(dt, &helis);
        self.move_buoys(dt, &helis);
        self.move_dips(dt, &helis);
        self.move_torpedoes(dt);
        self.update_mad(&helis);
        // The sensors report `SENSE_HZ` times a second; the tracker works once a second.
        let tick = (before * SENSE_HZ).floor() != (self.asw.clock * SENSE_HZ).floor();
        if tick {
            self.sense((1.0 / SENSE_HZ) as f32);
        }
        if second {
            for h in &helis {
                self.tracker(h.id as usize);
            }
            self.update_ai_hunts(1.0);
        }
        self.asw.wakes = self
            .asw
            .subs
            .iter()
            .filter(|s| s.alive && s.depth <= PERISCOPE_DEPTH + 2.0)
            .map(|s| s.at)
            .collect();
        self.judge_hunt(present);
    }

    fn move_subs(&mut self, dt: f64, helis: &[Heli]) {
        // A campaign's submarine whose hunter has gone home withdraws.
        let gone: Vec<bool> = self
            .asw
            .subs
            .iter()
            .map(|s| s.hunter.is_some_and(|p| !self.pilot_exists(p as usize) || self.pilot_absent(p)))
            .collect();
        let mut k = 0;
        self.asw.subs.retain(|_| {
            k += 1;
            !gone[k - 1]
        });
        for i in 0..self.asw.subs.len() {
            if !self.asw.subs[i].alive || self.asw.subs[i].done {
                continue;
            }
            let goal = match self.asw.subs[i].intent {
                Intent::Blockade { goal, .. } => goal,
                Intent::Approach { ship } | Intent::Attack { ship } | Intent::Escort { carrier: ship } => {
                    self.objects.get(ship).map_or(self.asw.subs[i].at, object_metres)
                }
            };
            // A torpedo in the water near it: evade.
            let torp = self
                .asw
                .torpedoes
                .iter()
                .filter(|t| t.phase != TorpPhase::Air)
                .map(|t| [t.p[0], t.p[1]])
                .find(|&p| dist(p, self.asw.subs[i].at) < 3000.0);
            let layer = self.asw.ocean.layer as f64;
            let bottom = self.asw.ocean.bottom as f64;
            let r = self.asw.rand(1000);
            let r2 = self.asw.rand(1000);
            // A helicopter low and slow near it: heard (its rotor on the water).
            let overhead = helis.iter().any(|h| {
                h.height < OVERHEAD_HEIGHT && h.speed < OVERHEAD_SPEED && dist(h.at, self.asw.subs[i].at) < OVERHEAD_RANGE
            });
            let s = &mut self.asw.subs[i];
            if let Some(p) = torp
                && s.posture != Posture::Attack
            {
                s.posture = Posture::Evade;
                s.posture_t = 0.0;
                s.threat = Some(p);
            }
            s.posture_t += dt;
            // Silent: stopped every so often, and while a helicopter hovers near (till it has
            // been gone a while); running for it once hunted in earnest (a ping, a torpedo).
            s.silent_in -= dt;
            if matches!(s.posture, Posture::Transit | Posture::Periscope) && (s.silent_in <= 0.0 || overhead) {
                s.posture = Posture::Silent;
                s.posture_t = 0.0;
                s.silent_for = if overhead {
                    OVERHEAD_HOLD
                } else {
                    (SILENT_FOR.0 + r2 * (SILENT_FOR.1 - SILENT_FOR.0) / 1000) as f64
                };
            }
            if s.posture == Posture::Silent {
                if overhead {
                    s.silent_for = s.silent_for.max(s.posture_t + OVERHEAD_HOLD);
                }
                if s.posture_t > s.silent_for {
                    s.posture = Posture::Transit;
                    s.posture_t = 0.0;
                    s.silent_in = (SILENT_EVERY.0 + r2 * (SILENT_EVERY.1 - SILENT_EVERY.0) / 1000) as f64;
                }
            }
            let to_goal = bearing(s.at, goal);
            let (want_heading, want_speed, want_depth) = match s.posture {
                Posture::Transit | Posture::Attack => (to_goal, TRANSIT, layer + 40.0),
                Posture::Periscope => (to_goal, QUIET, PERISCOPE_DEPTH),
                Posture::Silent => (s.heading, 0.0, s.depth),
                Posture::Evade => {
                    let away = s.threat.map_or(to_goal + std::f64::consts::FRAC_PI_2, |t| bearing(t, s.at));
                    // Away from the threat, but bending back toward its goal.
                    let h = away + wrap(to_goal - away).clamp(-1.2, 1.2) * 0.5;
                    let fast = s.posture_t < 90.0;
                    (h, if fast { SPRINT } else { QUIET }, (layer + 80.0).min(self.asw.ocean.bottom as f64 - 30.0))
                }
            };
            if s.posture == Posture::Evade && s.posture_t > 300.0 {
                s.posture = Posture::Transit;
                s.posture_t = 0.0;
                s.threat = None;
            }
            // Up to periscope depth every so often, while nothing hunts it.
            s.pd_in -= dt;
            if s.posture == Posture::Transit && s.pd_in <= 0.0 {
                s.posture = Posture::Periscope;
                s.posture_t = 0.0;
            } else if s.posture == Posture::Periscope && s.posture_t > PD_FOR {
                s.posture = Posture::Transit;
                s.posture_t = 0.0;
                s.pd_in = (PD_EVERY.0 + r * (PD_EVERY.1 - PD_EVERY.0) / 1000) as f64;
            }
            let turn = wrap(want_heading - s.heading).clamp(-SUB_TURN * dt, SUB_TURN * dt);
            s.heading = (s.heading + turn).rem_euclid(TAU);
            s.speed += (want_speed - s.speed).clamp(-0.3 * dt, 0.3 * dt);
            // Off the bottom (a shallow sea's).
            let want_depth = want_depth.min(bottom - 20.0).max(PERISCOPE_DEPTH.min(want_depth));
            s.depth += (want_depth - s.depth).clamp(-SUB_DIVE * dt, SUB_DIVE * dt);
            let next = [s.at[0] + s.heading.sin() * s.speed * dt, s.at[1] + s.heading.cos() * s.speed * dt];
            let ahead = [s.at[0] + s.heading.sin() * 1500.0, s.at[1] + s.heading.cos() * 1500.0];
            let (s_heading, s_at) = (s.heading, s.at);
            // Keep to the sea: turn away from land ahead.
            if !self.sea_at(ahead) {
                let left = [s_at[0] + (s_heading - 0.8).sin() * 1500.0, s_at[1] + (s_heading - 0.8).cos() * 1500.0];
                let turn = if self.sea_at(left) { -1.0 } else { 1.0 };
                let s = &mut self.asw.subs[i];
                s.heading = (s.heading + turn * SUB_TURN * 3.0 * dt).rem_euclid(TAU);
            }
            if self.sea_at(next) {
                self.asw.subs[i].at = next;
            }
            // At its goal: through the blockade, or in range of its target.
            let s = self.asw.subs[i].clone();
            match s.intent {
                Intent::Blockade { a, b, .. } => {
                    let side = |p: [f64; 2]| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
                    let start_side = side(goal);
                    if side(s.at) * start_side > 0.0 && dist(s.at, goal) < dist(a, b) {
                        self.asw.subs[i].done = true;
                        self.lose_hunt(AswResult::Escaped, None);
                    }
                }
                Intent::Approach { ship } | Intent::Attack { ship } | Intent::Escort { carrier: ship } => {
                    if s.posture != Posture::Attack && dist(s.at, goal) < ATTACK_RANGE {
                        let sub = &mut self.asw.subs[i];
                        sub.posture = Posture::Attack;
                        sub.posture_t = 0.0;
                    } else if s.posture == Posture::Attack {
                        let run = dist(s.at, goal) / (45.0 * KT);
                        if s.posture_t > run.min(ENEMY_TORPEDO_RUN) {
                            self.asw.subs[i].done = true;
                            self.lose_hunt(AswResult::ShipLost, Some(ship));
                        }
                    }
                }
            }
        }
    }

    fn lose_hunt(&mut self, why: AswResult, ship: Option<usize>) {
        if let Some(o) = ship
            && o < self.objects.len()
        {
            let at = object_metres(&self.objects[o]);
            self.emit(SimEvent::AswSplash { at: [(at[0] * 10.0) as i32, (at[1] * 10.0) as i32], kind: 3 });
            // A carrier takes it and stays afloat; anything smaller sinks.
            if self.objects[o].flags & oflags::CARRIER == 0 && self.objects[o].flags & oflags::DESTROYED == 0 {
                self.destroy_ground(o as i32);
            }
        }
        if self.asw.result == AswResult::Hunting {
            self.asw.result = why;
        }
        self.tell_crews(match why {
            AswResult::Escaped => text!("asw-sub-escaped"),
            _ => text!("asw-ship-torpedoed"),
        });
    }

    fn move_buoys(&mut self, dt: f64, helis: &[Heli]) {
        // Drops waiting for their point.
        for h in helis {
            let due: Vec<(BuoyKind, bool)> = {
                let c = self.asw.crew_mut(h.id);
                let mut due = Vec::new();
                c.pending.retain(|&(k, d, p)| {
                    if dist(p, h.at) < DROP_RADIUS {
                        due.push((k, d));
                        false
                    } else {
                        true
                    }
                });
                due
            };
            for (k, d) in due {
                if let Err(t) = self.drop_buoy(h, k, d) {
                    let back = self.pl.id as usize;
                    self.activate(h.id as usize);
                    self.msg(t);
                    self.activate(back);
                }
            }
        }
        let layer = self.asw.ocean.layer;
        let mut splashes = Vec::new();
        let mut reports = Vec::new();
        let dt32 = dt as f32;
        self.asw.buoys.retain_mut(|b| {
            if b.fall > 0.0 {
                b.fall -= dt32;
                if b.fall <= 0.0 {
                    splashes.push(b.at);
                }
                return true;
            }
            b.life -= dt32;
            if b.kind == BuoyKind::Bt && b.life <= BT_LIFE - BT_REPORT && b.life + dt32 > BT_LIFE - BT_REPORT {
                reports.push(b.owner);
            }
            b.life > 0.0
        });
        for at in splashes {
            if self.sea_at(at) {
                self.emit(SimEvent::AswSplash { at: [(at[0] * 10.0) as i32, (at[1] * 10.0) as i32], kind: 0 });
            }
        }
        // Buoys that came down on land are lost.
        let wet: Vec<bool> = self.asw.buoys.iter().map(|b| b.fall > 0.0 || self.sea_at(b.at)).collect();
        let mut k = 0;
        self.asw.buoys.retain(|_| {
            k += 1;
            wet[k - 1]
        });
        for o in reports {
            self.asw.crew_mut(o).layer = Some(layer.round() as u16);
            let back = self.pl.id as usize;
            if self.pilot_exists(o as usize) {
                self.activate(o as usize);
                self.msg(text!("asw-layer", m = layer.round() as i64));
                self.activate(back);
            }
        }
    }

    fn move_dips(&mut self, dt: f64, helis: &[Heli]) {
        for h in helis {
            let c = self.asw.crew_mut(h.id);
            let d = &mut c.dip;
            let reel = DIP_REEL * dt as f32;
            match d.state {
                DipState::Lowering => {
                    // The cable pays out from the hover: the sonar is in the sea once it has
                    // come down the hover's height.
                    d.depth = (d.depth + reel).min(d.set);
                    if (d.depth - d.set).abs() < 0.01 {
                        d.state = DipState::Down;
                    }
                }
                DipState::Raising => {
                    d.depth = (d.depth - reel).max(0.0);
                    if d.depth <= 0.0 {
                        d.state = DipState::Up;
                        d.active = false;
                    }
                }
                _ => {}
            }
            if d.wet() {
                d.at = h.at;
            }
            let wet = d.wet();
            if wet && (h.speed > DIP_BREAK_SPEED || h.height > DIP_BREAK_HEIGHT + d.depth as f64) {
                let d = &mut self.asw.crew_mut(h.id).dip;
                d.state = DipState::Broken;
                d.depth = 0.0;
                d.active = false;
                let back = self.pl.id as usize;
                self.activate(h.id as usize);
                self.msg(text!("asw-cable-parted"));
                self.sound(0x20, 2);
                self.activate(back);
            }
        }
        let _ = DIP_CABLE_SLACK;
    }

    fn update_mad(&mut self, helis: &[Heli]) {
        for h in helis {
            let down = self.asw.crew(h.id).is_some_and(|c| c.mad_down);
            if !down {
                self.asw.crew_mut(h.id).mad = 0;
                continue;
            }
            // The bird trails some 50 m behind and below the aircraft.
            let bird = [h.at[0] - h.heading.sin() * 50.0, h.at[1] - h.heading.cos() * 50.0];
            let height = (h.height - 30.0).max(5.0);
            let mut signal = 0.0;
            if h.height <= MAD_HEIGHT {
                for s in self.asw.subs.iter().filter(|s| s.alive) {
                    let slant = (dist(bird, s.at).powi(2) + (height + s.depth).powi(2)).sqrt();
                    if slant < MAD_RANGE {
                        signal += MAD_AT_100 * (100.0 / slant).powi(3);
                    }
                }
            }
            let noise = self.asw.rand(2 * MAD_NOISE + 1) - MAD_NOISE;
            let mad = (signal.min(3000.0) as i32 + noise) as i16;
            let t = self.asw.clock;
            let c = self.asw.crew_mut(h.id);
            c.mad = mad;
            // A pass over a submarine: an automatic mark at its peak.
            if mad > MAD_MARK && !c.mad_over {
                c.mad_over = true;
                c.fixes.push(Fix { at: bird, err: 200.0, t, how: 2 });
                let back = self.pl.id as usize;
                self.activate(h.id as usize);
                self.msg(text!("asw-mad-contact"));
                self.activate(back);
            } else if mad < MAD_MARK / 2 {
                self.asw.crew_mut(h.id).mad_over = false;
            }
        }
    }

    fn move_torpedoes(&mut self, dt: f64) {
        let mut hits = Vec::new();
        let mut entries = Vec::new();
        let layer = self.asw.ocean.layer as f64;
        let subs: Vec<(u16, [f64; 2], f64, bool)> =
            self.asw.subs.iter().map(|s| (s.id, s.at, s.depth, s.alive && s.posture == Posture::Evade)).collect();
        let mut torps = std::mem::take(&mut self.asw.torpedoes);
        let mut lost = Vec::new();
        for t in torps.iter_mut() {
            match t.phase {
                TorpPhase::Air => {
                    t.air -= dt;
                    t.speed *= (-dt / 1.0).exp();
                    t.p[0] += t.heading.sin() * t.speed * dt;
                    t.p[1] += t.heading.cos() * t.speed * dt;
                    t.p[2] = (t.p[2] - 15.0 * dt).max(0.0);
                    if t.air <= 0.0 {
                        t.phase = TorpPhase::Run;
                        t.p[2] = 0.0;
                        t.speed = TORP_RUN;
                        t.heading = t.base_heading;
                        entries.push([t.p[0], t.p[1]]);
                        if !self.sea_at([t.p[0], t.p[1]]) {
                            lost.push(t.id);
                        }
                    }
                    continue;
                }
                TorpPhase::Run => {
                    if t.t >= t.program.delay as f64 {
                        t.phase = TorpPhase::Search;
                        t.base = [t.p[0], t.p[1]];
                        t.base_heading = t.heading;
                        t.speed = TORP_SEARCH;
                    }
                }
                TorpPhase::Search => {
                    let st = t.t - t.program.delay as f64;
                    let w = TORP_SEARCH / CIRCLE_R;
                    t.heading = match t.program.pattern {
                        Pattern::Straight => t.base_heading,
                        Pattern::Circle | Pattern::Helix => (t.base_heading + w * st).rem_euclid(TAU),
                        Pattern::Snake => {
                            (t.base_heading + SNAKE_SWING * (TAU * st / SNAKE_PERIOD).sin()).rem_euclid(TAU)
                        }
                    };
                }
                TorpPhase::Home => {
                    if let Some(&(_, at, depth, _)) = subs.iter().find(|s| Some(s.0) == t.target) {
                        let want = bearing([t.p[0], t.p[1]], at);
                        let turn = wrap(want - t.heading).clamp(-15.0 * DEG * dt, 15.0 * DEG * dt);
                        t.heading = (t.heading + turn).rem_euclid(TAU);
                        t.p[2] += (-depth - t.p[2]).clamp(-4.0 * dt, 4.0 * dt);
                        let d3 = (dist([t.p[0], t.p[1]], at).powi(2) + (t.p[2] + depth).powi(2)).sqrt();
                        if d3 < TORP_HIT {
                            hits.push((t.id, t.target.unwrap_or(0)));
                        }
                    } else {
                        t.phase = TorpPhase::Search;
                        t.target = None;
                    }
                }
            }
            t.t += dt;
            // Depth: down to its search band (the helix up and down through it).
            if t.phase != TorpPhase::Home {
                let (c, f) = (t.program.ceiling as f64, t.program.floor as f64);
                let want = if t.phase == TorpPhase::Search && t.program.pattern == Pattern::Helix {
                    c + (f - c) * 0.5 * (1.0 - (TAU * t.t / 60.0).cos())
                } else {
                    (c + f) * 0.5
                };
                t.p[2] += (-want - t.p[2]).clamp(-3.0 * dt, 3.0 * dt);
            }
            t.p[0] += t.heading.sin() * t.speed * dt;
            t.p[1] += t.heading.cos() * t.speed * dt;
            if t.phase == TorpPhase::Home {
                t.speed = TORP_HOME;
            }
            if t.t > TORP_LIFE {
                lost.push(t.id);
            }
        }
        // Acquisitions (searching torpedoes listen and ping in their cone).
        for t in torps.iter_mut().filter(|t| t.phase == TorpPhase::Search) {
            for &(id, at, depth, evading) in &subs {
                if !self.asw.subs.iter().any(|s| s.id == id && s.alive) {
                    continue;
                }
                let r = dist([t.p[0], t.p[1]], at);
                let across = (-t.p[2] < layer) != (depth < layer);
                let range = if across { TORP_ACQUIRE * 0.5 } else { TORP_ACQUIRE };
                let off = wrap(bearing([t.p[0], t.p[1]], at) - t.heading).abs();
                if r < range && off < TORP_CONE {
                    let breaks = evading && self.asw.rand(100) < TORP_BREAK;
                    if !breaks {
                        t.phase = TorpPhase::Home;
                        t.target = Some(id);
                    }
                    break;
                }
            }
        }
        torps.retain(|t| !lost.contains(&t.id) && !hits.iter().any(|h| h.0 == t.id));
        self.asw.torpedoes = torps;
        for at in entries {
            self.emit(SimEvent::AswSplash { at: [(at[0] * 10.0) as i32, (at[1] * 10.0) as i32], kind: 1 });
        }
        for (_, sub) in hits {
            if let Some(i) = self.asw.subs.iter().position(|s| s.id == sub && s.alive) {
                self.sink(i);
            }
        }
    }

    /// `SENSE_HZ` times a second: every sensor's report, the echoes' fixes; pings alert the
    /// submarines that hear them.
    fn sense(&mut self, dt: f32) {
        let ocean = self.asw.ocean;
        let noise = ocean.noise();
        let clock = self.asw.clock;
        let subs: Vec<Sub> = self.asw.subs.iter().filter(|s| s.alive && !s.done).cloned().collect();
        // The torpedoes in the water: where, how deep, which.
        let torps: Vec<([f64; 2], f64, u16)> = self
            .asw
            .torpedoes
            .iter()
            .filter(|t| t.phase != TorpPhase::Air)
            .map(|t| ([t.p[0], t.p[1]], -t.p[2], t.id))
            .collect();
        // The ships under way within reach of a sensor: (where, noise, tones).
        let ships: Vec<([f64; 2], f64, [u16; 2])> = (0..self.objects.len())
            .filter(|&i| {
                let o = &self.objects[i];
                o.flags & oflags::DESTROYED == 0 && (o.flags & oflags::CARRIER != 0 || self.is_ship(i as i32))
            })
            .map(|i| {
                let o = &self.objects[i];
                let carrier = o.flags & oflags::CARRIER != 0;
                let level = if carrier { 168.0 } else { 156.0 + (i % 5) as f64 };
                let tones = if carrier { [60, 11] } else { [47 + (i % 9) as u16 * 3, 14 + (i % 6) as u16 * 2] };
                (object_metres(o), level, tones)
            })
            .filter(|(at, ..)| self.asw.buoys.iter().any(|b| dist(*at, b.at) < SHIP_HEARD) || self.asw.crews.iter().any(|c| c.dip.wet() && dist(*at, c.dip.at) < SHIP_HEARD))
            .collect();
        let mut alerts: Vec<[f64; 2]> = Vec::new();
        let mut fixes: Vec<(u8, Fix)> = Vec::new();
        // Noise in the measurements: drawn first (the borrow of the buoys below).
        let draws: Vec<f64> = (0..(self.asw.buoys.len() + self.asw.crews.len()) * subs.len().max(1) * 4)
            .map(|_| self.asw.gauss())
            .collect();
        let mut di = 0usize;
        let mut draw = || {
            let d = draws.get(di).copied().unwrap_or(0.0);
            di += 1;
            d
        };
        let mut sensors: Vec<(u8, u16, [f64; 2], f64, Option<f64>, bool, &mut Sample)> = Vec::new();
        // The pings this second: who (owner, sensor), where, how deep, how far they reach, how
        // loud.
        let mut pinged: Vec<(u8, u16, [f64; 2], f64, f64, f64)> = Vec::new();
        for b in self.asw.buoys.iter_mut().filter(|b| b.fall <= 0.0 && b.kind != BuoyKind::Bt) {
            let ping = b.kind == BuoyKind::Dicass && b.active && b.pings > 0 && {
                b.ping_in -= dt;
                b.ping_in <= 0.0
            };
            if ping {
                b.ping_in = PING_EVERY;
                b.pings -= 1;
                alerts.push(b.at);
                pinged.push((b.owner, b.id, b.at, b.depth as f64, DICASS_RANGE, PING_SL));
            }
            let di = if b.kind == BuoyKind::Difar { Some(DIFAR_DI) } else { None };
            sensors.push((b.owner, b.id, b.at, b.depth as f64, di, ping, &mut b.sample));
        }
        for c in self.asw.crews.iter_mut() {
            let d = &mut c.dip;
            if !d.wet() {
                continue;
            }
            let ping = d.active && {
                d.ping_in -= dt;
                d.ping_in <= 0.0
            };
            if ping {
                d.ping_in = PING_EVERY;
                alerts.push(d.at);
                pinged.push((c.owner, 0, d.at, d.depth as f64, DIP_RANGE, DIP_SL));
            }
            sensors.push((c.owner, 0, d.at, d.depth as f64, Some(DIFAR_DI), ping, &mut d.sample));
        }
        for (owner, sensor, at, depth, di, ping, sample) in sensors {
            let mut lines = Vec::new();
            for s in &subs {
                let r = dist(at, s.at).max(10.0);
                let tl = ocean.loss(r, depth, s.depth);
                let passive = s.level() - tl - noise + di.unwrap_or(0.0) - PASSIVE_DT;
                let (n1, n2) = (draw(), draw());
                let true_b = bearing(at, s.at);
                let mut line = Line { snr: passive.clamp(-60.0, 60.0) as i8, freqs: s.freqs(), ..Default::default() };
                let mut heard = passive > 0.0;
                if heard && di.is_some() {
                    let err = (2.0 + 25.0 / (1.0 + passive)) * DEG;
                    let b = true_b + n1 * err;
                    line.bearing = Some(deg(b));
                }
                if ping {
                    let (reach, sl) = if sensor == 0 { (DIP_RANGE, DIP_SL) } else { (DICASS_RANGE, PING_SL) };
                    let excess = sl - 2.0 * tl + TARGET_STRENGTH - (noise - ACTIVE_DI) - ACTIVE_DT;
                    if excess > 0.0 && r < reach {
                        let err_b = (1.5 + 10.0 / (1.0 + excess)) * DEG;
                        let b = true_b + n1 * err_b;
                        let range = r + n2 * (10.0 + r * 0.01);
                        line.bearing = Some(deg(b));
                        line.range = Some(range.clamp(0.0, 65_000.0) as u16);
                        let closing = -(s.heading.sin() * (at[0] - s.at[0]) + s.heading.cos() * (at[1] - s.at[1])) / r * s.speed;
                        line.doppler = (-closing / KT).round().clamp(-60.0, 60.0) as i8;
                        line.snr = line.snr.max(excess.clamp(0.0, 60.0) as i8);
                        let fix = [at[0] + b.sin() * range, at[1] + b.cos() * range];
                        fixes.push((owner, Fix { at: fix, err: (30.0 + r * err_b) as f32, t: clock, how: 1 }));
                        heard = true;
                    }
                }
                if heard {
                    lines.push(line);
                }
            }
            // The torpedoes running: their fast screws, cavitating.
            for &(t_at, t_depth, id) in &torps {
                let r = dist(at, t_at).max(20.0);
                let passive = TORPEDO_LEVEL - ocean.loss(r, depth, t_depth) - noise + di.unwrap_or(0.0) - PASSIVE_DT;
                if passive <= 0.0 {
                    continue;
                }
                let err = (2.0 + 25.0 / (1.0 + passive)) * DEG;
                let b = bearing(at, t_at) + draw() * err;
                lines.push(Line {
                    snr: passive.clamp(-60.0, 60.0) as i8,
                    bearing: di.map(|_| deg(b)),
                    freqs: [TORPEDO_LINE_HZ + (id % 6) * 4, 0],
                    torpedo: true,
                    ..Default::default()
                });
            }
            // The ships: heard (never pinged: the sonars' pings are set for submarines).
            for &(ship_at, level, freqs) in &ships {
                let r = dist(at, ship_at).max(50.0);
                let passive = level - ocean.loss(r, depth, 5.0) - noise + di.unwrap_or(0.0) - PASSIVE_DT;
                if passive <= 0.0 {
                    continue;
                }
                let err = (2.0 + 25.0 / (1.0 + passive)) * DEG;
                let b = bearing(at, ship_at) + draw() * err;
                lines.push(Line {
                    snr: passive.clamp(-60.0, 60.0) as i8,
                    bearing: di.map(|_| deg(b)),
                    freqs,
                    ship: true,
                    ..Default::default()
                });
            }
            lines.sort_by(|a, b| b.snr.cmp(&a.snr));
            lines.truncate(4);
            // Other sonars' pings: a line each (with its bearing on a directional sensor), and
            // their echoes off the submarines in their reach, however quiet the submarine runs.
            // Both on their way at the speed of sound: the ping straight from the pinging sonar,
            // the echo the longer way round by the submarine. What was on its way comes in
            // first, at the report nearest its time.
            let mut due = std::mem::take(&mut sample.due);
            for d in due.iter_mut() {
                d.0 -= dt;
            }
            for &(o, id, p, p_depth, reach, sl) in &pinged {
                let r = dist(p, at);
                if (o, id) == (owner, sensor) || r >= PING_HEARD {
                    continue;
                }
                let snr = sl - ocean.loss(r, depth, p_depth) - noise;
                let line = Line {
                    snr: snr.clamp(0.0, 60.0) as i8,
                    bearing: di.map(|_| deg(bearing(at, p))),
                    freqs: [PING_LINE_HZ[(id == 0) as usize], 0],
                    ping: true,
                    ..Default::default()
                };
                due.push(((r / SOUND_SPEED) as f32, line));
                for s in &subs {
                    let (r1, r2) = (dist(p, s.at).max(10.0), dist(s.at, at).max(10.0));
                    if r1 >= reach {
                        continue;
                    }
                    let excess = sl - ocean.loss(r1, p_depth, s.depth) - ocean.loss(r2, depth, s.depth)
                        + TARGET_STRENGTH
                        - noise
                        + di.unwrap_or(0.0)
                        - ACTIVE_DT;
                    if excess <= 0.0 {
                        continue;
                    }
                    let err = (1.5 + 10.0 / (1.0 + excess)) * DEG;
                    let b = bearing(at, s.at) + draw() * err;
                    let line = Line {
                        snr: excess.clamp(1.0, 60.0) as i8,
                        bearing: di.map(|_| deg(b)),
                        freqs: [PING_LINE_HZ[(id == 0) as usize], 0],
                        echo: true,
                        ..Default::default()
                    };
                    due.push((((r1 + r2) / SOUND_SPEED) as f32, line));
                }
            }
            // The pings and echoes arriving now.
            let arrived = |d: &(f32, Line)| d.0 < dt / 2.0;
            lines.extend(due.iter().filter(|d| arrived(d)).map(|d| d.1));
            let heard = due.iter().any(|d| arrived(d) && d.1.ping);
            due.retain(|d| !arrived(d));
            // The pings and echoes always shown, the four loudest contacts besides.
            lines.sort_by_key(|l| (!(l.ping || l.echo), -(l.snr as i16)));
            let pings = lines.iter().filter(|l| l.ping || l.echo).count();
            lines.truncate(pings + 4);
            *sample = Sample { seq: sample.seq.wrapping_add(1), noise: noise as u8, lines, ping, heard, due };
        }
        for (o, f) in fixes {
            self.asw.crew_mut(o).fixes.push(f);
        }
        // The pinged submarines run.
        for s in self.asw.subs.iter_mut().filter(|s| s.alive && s.posture != Posture::Attack) {
            if let Some(&p) = alerts.iter().find(|&&p| dist(p, s.at) < COUNTER_DETECT) {
                if s.posture != Posture::Evade {
                    s.posture_t = 0.0;
                }
                s.posture = Posture::Evade;
                s.threat = Some(p);
            }
        }
    }

    /// The tracker of pilot `k`'s crew: fits its tracks to its fixes (the echoes', the MAD's).
    fn tracker(&mut self, k: usize) {
        let t = self.asw.clock;
        let Some(ci) = self.asw.crews.iter().position(|c| c.owner == k as u8) else { return };
        let c = &mut self.asw.crews[ci];
        c.fixes.retain(|f| t - f.t <= FIX_KEEP);
        let n = c.fixes.len();
        c.fixes.drain(..n.saturating_sub(MAX_FIXES));
        // Each fix to the nearest track it fits, else a new one.
        let mut groups: Vec<Vec<Fix>> = vec![Vec::new(); c.tracks.len()];
        let mut fresh: Vec<Vec<Fix>> = Vec::new();
        for f in &c.fixes {
            let near = c
                .tracks
                .iter()
                .enumerate()
                .map(|(i, tr)| {
                    let p = [tr.at[0] + tr.vel[0] * (f.t - t), tr.at[1] + tr.vel[1] * (f.t - t)];
                    (i, dist(p, f.at), tr.err as f64)
                })
                .filter(|&(_, d, e)| d < 3.0 * (e + f.err as f64) + 500.0)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match near {
                Some((i, _, _)) => groups[i].push(*f),
                None => match fresh.iter_mut().find(|g| dist(g[0].at, f.at) < 3.0 * f.err as f64 + 500.0) {
                    Some(g) => g.push(*f),
                    None => fresh.push(vec![*f]),
                },
            }
        }
        let mut tracks: Vec<Track> = Vec::new();
        for (i, g) in groups.into_iter().enumerate() {
            let tr = c.tracks[i];
            if g.is_empty() {
                if t - tr.last < TRACK_DROP {
                    tracks.push(tr);
                }
                continue;
            }
            tracks.push(fit(tr.n, &g, t));
        }
        for g in fresh {
            c.next_track = c.next_track.wrapping_add(1).max(1);
            tracks.push(fit(c.next_track, &g, t));
        }
        c.tracks = tracks;
    }

    /// The hunt's objective: every submarine sunk (mission type 12).
    fn judge_hunt(&mut self, present: &[usize]) {
        if self.asw.subs.is_empty() || self.asw.result != AswResult::Hunting {
            return;
        }
        // A campaign's hunts are each sortie's own (`sink`).
        if self.asw.subs.iter().any(|s| s.hunter.is_some() || s.ai_hunt.is_some()) {
            return;
        }
        if self.asw.subs.iter().all(|s| !s.alive) {
            self.asw.result = AswResult::Won;
            let back = self.pl.id as usize;
            for &k in present {
                self.activate(k);
                for t in 0..2 {
                    if self.pl.targets[t].mission_type == HUNT_MISSION {
                        self.mark_target_reached(t);
                    }
                }
            }
            self.activate(back);
        }
    }
}

/// A track fitted to its fixes: weighted least squares of a straight run at a steady speed.
fn fit(n: u8, fixes: &[Fix], now: f64) -> Track {
    let w: Vec<f64> = fixes.iter().map(|f| 1.0 / (f.err as f64).powi(2)).collect();
    let sw: f64 = w.iter().sum();
    let tm = fixes.iter().zip(&w).map(|(f, w)| f.t * w).sum::<f64>() / sw;
    let mean = |k: usize| fixes.iter().zip(&w).map(|(f, w)| f.at[k] * w).sum::<f64>() / sw;
    let (mx, my) = (mean(0), mean(1));
    let stt: f64 = fixes.iter().zip(&w).map(|(f, w)| w * (f.t - tm).powi(2)).sum();
    let span = fixes.iter().map(|f| f.t).fold(f64::MIN, f64::max) - fixes.iter().map(|f| f.t).fold(f64::MAX, f64::min);
    let mut vel = [0.0, 0.0];
    if fixes.len() >= 3 && span > 30.0 && stt > 0.0 {
        for (k, m) in [(0, mx), (1, my)] {
            vel[k] = fixes.iter().zip(&w).map(|(f, w)| w * (f.t - tm) * (f.at[k] - m)).sum::<f64>() / stt;
        }
        // No submarine runs faster than this: a wild fit is no speed at all.
        let v = vel[0].hypot(vel[1]);
        if v > 25.0 * KT {
            vel = [0.0, 0.0];
        }
    }
    let at = [mx + vel[0] * (now - tm), my + vel[1] * (now - tm)];
    let spread = (fixes
        .iter()
        .zip(&w)
        .map(|(f, w)| {
            let p = [mx + vel[0] * (f.t - tm), my + vel[1] * (f.t - tm)];
            w * dist(p, f.at).powi(2)
        })
        .sum::<f64>()
        / sw)
        .sqrt();
    let best = fixes.iter().map(|f| f.err as f64).fold(f64::MAX, f64::min);
    let err = (spread + best / (fixes.len() as f64).sqrt()).max(30.0);
    let last = fixes.iter().map(|f| f.t).fold(f64::MIN, f64::max);
    Track { n, at, vel, err: err as f32, last }
}
