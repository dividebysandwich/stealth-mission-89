//! Sound effects (assets/sounds/, synthesized by tools/gen_sfx.py and tools/gen_aircraft_sounds.py
//! unless noted).
//!
//! - Effects are requested by id with a priority (`PlaySound`); they play when
//!   `prio >= 3 - level` (level 3 = all sounds, 2 = no engine, 1 = firing and explosions only,
//!   0 = none; Alt-V cycles, with an extra step at level 3 that halves the engine first).
//!   Gear, bay doors, the error buzz and speech bypass the level.
//! - Samples are optional: `sfx_NN_*` / `speech_NN_*` (.flac) register under id NN, and a
//!   requested id with no sample stays silent, so speech can be added later by dropping files in.
//! - Radar warning tones come from `sounds/rwr/` (ids 100 and up).
//! - The cannon (sound 6) holds a loop of its burst while it fires and plays the burst's decay
//!   when it stops (`cannon_loop.flac`, `cannon_tail.flac`).
//! - A fire on board sounds the fire warning siren (`fire_siren.flac`, tools/gen_fire_siren.py)
//!   without a break until the fire is out (or the pilot has ejected).
//! - The master caution (its cockpit light or key) silences the alarms sounding: the fire
//!   siren until that fire is out, the stall horn and the radar warning tones while they keep
//!   repeating (one that pauses for `ALARM_REARM` sounds again).
//! - After ejecting the aircraft falls silent (engine, warnings, radio); the pilot hears the
//!   ejection, explosions and flares, and on the ground the emergency beacon (`beacon.flac`, two
//!   chirps every 3 s at half volume) until the rescue ends.
//! - Volumes (the OPTIONS menus): a master volume times the channel's, engine (the flown
//!   aircraft's and the others'), warnings (alarms, the fire siren, the Sidewinder tones) or
//!   effects (everything else) (`Channel`).
//! - Inside the aircraft (the cockpit views) the engine sounds (the jet's engine and wind, the
//!   helicopter's rotor and turbines), the other aircraft, SAM launches, vehicles' guns
//!   (`tank_gun`, `autocannon` and their `_far` variants), explosions, thunder
//!   and the world around (`ambient`) play through a low-pass filter (`muffle`).
//! - In the external views (chase, side, tactical, weapon, free) the flown aircraft's engine
//!   falls off with its distance from the eye, and the sounds heard only inside the cockpit
//!   (warning tones and the fire siren, the Sidewinder seeker, radar contact, camera shutter,
//!   error buzz and speech) are not played.
//! - Lightning's thunder (`event_thunder*.flac`, one at random) arrives after the strike at the
//!   speed of sound (weather.rs), quieter the further away.
//! - A jet plays `jet_engine.flac` and `jet_wind.flac`: the engine pitched with its rpm
//!   (winding up quickly from 0 when the throttle opens, idling in the air with it closed,
//!   running down when it closes on the ground or the fuel runs out), the wind louder with
//!   airspeed; the lit afterburner roars over the engine (`afterburner.flac`, louder with its
//!   setting, with a deeper thump as it lights). All only play at level 3.
//! - A helicopter's rotor (`heli_rotor.flac`) plays at the rotor speed, louder with the torque
//!   and in translational lift (the blade slap), over the turbines (`heli_turbine.flac`)
//!   pitched up with the torque. The CH-47 has its own: two rotors' double chop
//!   (`ch47_rotor.flac`) over bigger turbines (`ch47_turbine.flac`).
//! - The other aircraft are heard around, the AI's and the other players': jets and the Geran-3
//!   (`jet.flac`, a recording), the propeller-driven Geran-2 (`engine.flac`, a recording),
//!   helicopters (`helicopter.flac`, a CH-47 `chinook.flac`) and a drone command center's quadrotor interceptors
//!   (`interceptor.flac`, as at 20% throttle, its motors two octaves up at full throttle),
//!   placed by direction with stereo
//!   panning that follows the head, fading with distance, with a Doppler shift. Only at
//!   level 3.
//! - A SAM site's launch (`launch.flac`, a recording) is heard near the site, placed and fading
//!   like the aircraft, arriving at the speed of sound.
//! - At an air defence console the shelter's own sounds play (`station::sound`).
//! - With Sidewinders selected in the air-to-air HUD mode, the seeker growls (cut from a
//!   recording of the real tones by tools/cut_sidewinder_tones.py): the low growl
//!   (`aim9_growl.flac`) with nothing in its view, the higher one (`aim9_seen.flac`) with the
//!   locked target in the seeker (the LOCK oval), and the steady lock tone (`aim9_tone.flac`)
//!   once the target is in range.

use std::collections::HashMap;

use bevy::audio::{
    AddAudioSource, AudioSink, PlaybackMode, PlaybackSettings, SpatialAudioSink, SpatialListener,
    SpatialScale, Volume,
};
use bevy::prelude::*;
use stealth_sim::{AircraftKind, fflags};

use crate::camera::{EyeCamera, View, ViewState};
use crate::combat::{CombatSystems, unit_position};
use crate::flight::{Aircraft, Player};
use crate::input::{Action, ActionEvent};
use crate::settings::AppState;
use crate::world::{FloatingOrigin, WorldPos, wu_to_bevy};

mod ambient;
pub(crate) mod muffle;

use muffle::{Muffler, Pitch};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .add_message::<Thunder>()
            .add_message::<GunReport>()
            .init_resource::<SoundLevel>()
            .init_resource::<Alarms>()
            .init_resource::<muffle::MuffledSounds>()
            .init_resource::<ambient::Ambience>()
            .add_audio_source::<muffle::Muffled>()
            .add_systems(Startup, (load_bank, ambient::load))
            .add_systems(OnEnter(AppState::Flight), (muffle::reset, ambient::reset))
            .add_systems(
                Update,
                (
                    muffle::follow_view,
                    flight_sounds,
                    play_sounds,
                    cannon,
                    heli_sounds,
                    jet_sounds,
                    beacon,
                    fire_siren,
                    sidewinder_tones,
                    objective_sound,
                    thunder,
                    ambient::ambience,
                    remix,
                )
                    .chain()
                    .run_if(in_state(AppState::Flight)),
            )
            .add_systems(
                Update,
                (follow_head, distant_aircraft, sam_launches, gun_reports)
                    .chain()
                    .after(CombatSystems)
                    .run_if(in_state(AppState::Flight)),
            )
            .add_systems(OnExit(AppState::Flight), stop_all);
    }
}

/// Request to play a sound id. `priority` 255 = direct (bypasses the level).
#[derive(Message, Debug, Clone, Copy)]
pub struct PlaySound {
    pub id: u8,
    pub priority: u8,
}

pub const DIRECT: u8 = 255;

/// A vehicle's gun fired a round at `at` (Bevy world).
#[derive(Message, Debug, Clone, Copy)]
pub struct GunReport {
    pub at: bevy::math::DVec3,
    pub calibre: crate::explosions::Calibre,
}

/// Thunder of a lightning strike `distance` metres away, arriving now.
#[derive(Message, Debug, Clone, Copy)]
pub struct Thunder {
    pub distance: f32,
}

/// Sound ids of the radar warning tones (above the effects' ids).
pub fn rwr_id(tone: stealth_sim::RwrTone) -> u8 {
    use stealth_sim::RwrTone::*;
    match tone {
        Track => 100,
        Scan => 101,
        SearchWarning => 102,
        SamLaunch => 103,
        AirLaunch => 104,
    }
}

const RWR_FILES: [(stealth_sim::RwrTone, &str); 5] = {
    use stealth_sim::RwrTone::*;
    [
        (Track, "Track.flac"),
        (Scan, "Scan.flac"),
        (SearchWarning, "SearchWarning.flac"),
        (SamLaunch, "Launch.flac"),
        (AirLaunch, "LaunchWarningUS.flac"),
    ]
};

/// Which radar warning tones sound (`Settings::rwr_tones`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum RwrTones {
    /// Every tone: a radar painting, each sweep that detects, missile launches.
    #[default]
    All,
    /// A detection once (until the radars have let go for a while) and missile launches.
    Detect,
    /// Missile launches only.
    Launch,
    Off,
}

impl RwrTones {
    pub const ALL: [RwrTones; 4] =
        [RwrTones::All, RwrTones::Detect, RwrTones::Launch, RwrTones::Off];

    pub fn name(self) -> &'static str {
        match self {
            RwrTones::All => "ALL",
            RwrTones::Detect => "DETECT ONLY",
            RwrTones::Launch => "LAUNCH ONLY",
            RwrTones::Off => "OFF",
        }
    }

    pub fn facts(self) -> &'static [&'static str] {
        match self {
            RwrTones::All => {
                &["A radar painting you, every sweep that detects you, missile launches"]
            }
            RwrTones::Detect => &[
                "One tone when a radar detects you (again after the radars let go for a while)",
                "And missile launches",
            ],
            RwrTones::Launch => &["Only missiles launched at you"],
            RwrTones::Off => &["No radar warning tones (the lights and displays still warn)"],
        }
    }

    /// The next (`dir` 1) or previous (-1) choice, wrapping.
    pub fn step(self, dir: i32) -> Self {
        let i = Self::ALL.iter().position(|&t| t == self).unwrap_or(0) as i32;
        Self::ALL[(i + dir).rem_euclid(Self::ALL.len() as i32) as usize]
    }
}

/// The radar warning tones' sound ids: painted, detected (Doppler, pulse), launches.
const RWR_PAINTED: u8 = 100;
const RWR_DETECTED: [u8; 2] = [101, 102];

/// Seconds without a detecting sweep before a detection sounds again in `RwrTones::Detect`.
const DETECT_REARM: f64 = 10.0;

/// Whether the tones setting lets radar warning tone `id` sound; `last_detected` is when a
/// detecting sweep last asked for its tone (real seconds).
fn rwr_sounds(tones: RwrTones, id: u8, now: f64, last_detected: &mut Option<f64>) -> bool {
    let detected = RWR_DETECTED.contains(&id);
    let fresh = last_detected.is_none_or(|t| now - t > DETECT_REARM);
    if detected {
        *last_detected = Some(now);
    }
    match tones {
        RwrTones::All => true,
        RwrTones::Detect => id != RWR_PAINTED && (!detected || fresh),
        RwrTones::Launch => id != RWR_PAINTED && !detected,
        RwrTones::Off => false,
    }
}

/// Alt-V sound level (3 = all sounds .. 0 = none); `engine_half` = level 3 with the engine
/// at half volume.
#[derive(Resource)]
pub struct SoundLevel(pub u8, pub bool);

impl Default for SoundLevel {
    fn default() -> Self {
        Self(3, false)
    }
}

/// What a sound is, for its volume (`Settings::*_volume`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Channel {
    Engine,
    Effects,
    Warnings,
    /// The radar warning tones (also under the warnings' volume).
    Rwr,
    /// The coop radio's voices.
    Voice,
}

impl Channel {
    /// The channel of an effect by id.
    fn of(id: u8) -> Self {
        if RWR_IDS.contains(&id) {
            Channel::Rwr
        } else if ALARM_TONES.contains(&id) {
            Channel::Warnings
        } else {
            Channel::Effects
        }
    }

    /// The channel's volume: the master volume times its own.
    pub fn gain(self, s: &crate::settings::Settings) -> f32 {
        s.master_volume
            * match self {
                Channel::Engine => s.engine_volume,
                Channel::Effects => s.effects_volume,
                Channel::Warnings => s.warning_volume,
                Channel::Rwr => s.warning_volume * s.rwr_volume,
                Channel::Voice => s.voice_volume,
            }
    }
}

/// A sound played at a fixed volume (before its channel's gain), set again when the volumes
/// change.
#[derive(Component)]
pub(crate) struct Mixed(pub(crate) Channel, pub(crate) f32);

impl Mixed {
    /// The playback settings with the volume mixed in.
    pub(crate) fn settings(
        &self,
        s: &crate::settings::Settings,
        p: PlaybackSettings,
    ) -> PlaybackSettings {
        p.with_volume(Volume::Linear(self.1 * self.0.gain(s)))
    }
}

/// The volumes changed: the sounds playing at a fixed volume follow (the others set theirs every
/// frame).
fn remix(settings: Res<crate::settings::Settings>, mut q: Query<(&Mixed, &mut AudioSink)>) {
    if !settings.is_changed() {
        return;
    }
    for (m, mut sink) in &mut q {
        sink.set_volume(Volume::Linear(m.1 * m.0.gain(&settings)));
    }
}

/// The sounds still heard after ejecting: impact, explosion, the ejection, signal flare.
const EJECTED_SOUNDS: [u8; 4] = [4, 5, 0x14, 0x1C];

/// The sounds heard only inside the cockpit (besides the alarm tones and speech): radar contact,
/// camera shutter, error buzz.
const COCKPIT_SOUNDS: [u8; 3] = [7, 0x12, 0x20];

/// Sound id of an explosion (through the cockpit's low-pass).
const EXPLOSION: u8 = 5;

/// The sounds that play again over themselves: a missile launch and a bomb release, one for
/// each store however fast they go.
const OVERLAPPING: [u8; 2] = [0x0D, 0x10];

/// Fire warning siren volume.
const SIREN_VOLUME: f32 = 0.35;

/// The alarm tones the master caution silences: the stall horn and the radar warning tones.
const ALARM_TONES: [u8; 6] = [14, 100, 101, 102, 103, 104];

/// The radar warning tones' ids (`rwr_id`).
const RWR_IDS: std::ops::RangeInclusive<u8> = 100..=104;

/// Seconds an alarm tone silenced by the master caution has to pause to sound again.
const ALARM_REARM: f64 = 5.0;

/// Alarms and the master caution.
#[derive(Resource, Default)]
struct Alarms {
    /// Alarm tone ids: when each was last asked for (real seconds), and whether the master
    /// caution silenced it.
    tones: HashMap<u8, (f64, bool)>,
    /// The fire siren is silenced until the fire is out.
    fire_silenced: bool,
}

impl Alarms {
    /// The master caution: silences the alarm tones heard lately, and the fire siren.
    fn silence(&mut self, now: f64) {
        self.fire_silenced = true;
        for (last, quiet) in self.tones.values_mut() {
            *quiet |= now - *last < ALARM_REARM;
        }
    }

    /// An alarm tone is asked for: false while it is silenced.
    fn sounds(&mut self, id: u8, now: f64) -> bool {
        let (last, quiet) = self.tones.entry(id).or_insert((now, false));
        *quiet &= now - *last < ALARM_REARM;
        *last = now;
        !*quiet
    }
}

/// Emergency beacon interval (real seconds) and volume.
const BEACON_INTERVAL: f32 = 3.0;
const BEACON_VOLUME: f32 = 0.5;

/// The other aircraft heard: jets, propeller drones and helicopters up to their range (m), at
/// full volume within the reference distance (m), then falling off with 1 / distance; the
/// nearest `DISTANT_MAX` of them.
const JET_RANGE: f32 = 12_000.0;
const JET_REF: f32 = 500.0;
const JET_VOLUME: f32 = 0.9;
const PROPELLER_RANGE: f32 = 4_000.0;
const PROPELLER_REF: f32 = 150.0;
const PROPELLER_VOLUME: f32 = 0.8;
const HELICOPTER_RANGE: f32 = 5_000.0;
const HELICOPTER_REF: f32 = 200.0;
const HELICOPTER_VOLUME: f32 = 0.8;
const QUAD_RANGE: f32 = 2_000.0;
const QUAD_REF: f32 = 60.0;
const QUAD_VOLUME: f32 = 0.7;
const DISTANT_MAX: usize = 6;
/// A SAM launch: heard up to this range (m), at full volume within the reference distance (m).
const LAUNCH_RANGE: f32 = 10_000.0;
const LAUNCH_REF: f32 = 400.0;
const LAUNCH_VOLUME: f32 = 1.0;
/// A tank's gun and an autocannon's round (the Shilka's lighter): heard up to this range (m),
/// at full volume within the reference distance (m); beyond `GUN_FAR` (m) the distant sound.
const TANK_GUN: (f32, f32, f32) = (12_000.0, 150.0, 1.0);
const AUTOCANNON: (f32, f32, f32) = (5_000.0, 50.0, 0.6);
const LIGHT_GUN: (f32, f32, f32) = (3_000.0, 30.0, 0.35);
const GUN_FAR: f32 = 800.0;
/// The flown aircraft's engine in the external views: at full volume within this distance (m)
/// of the eye, then falling off with 1 / distance.
const ENGINE_REF: f32 = 25.0;
/// Thunder at full volume within this distance (m), then falling off with 1 / distance, gone at
/// `weather::THUNDER_RANGE` (fading out over its last part).
const THUNDER_REF: f32 = 1500.0;
const THUNDER_VOLUME: f32 = 0.9;

/// Speed of sound (m/s) for the Doppler shift, and the shift's limits.
const SPEED_OF_SOUND: f32 = 340.0;
const DOPPLER_RANGE: (f32, f32) = (0.75, 1.35);
/// Distance between the listener's ears (m).
const EAR_GAP: f32 = 0.2;

#[derive(Resource, Default)]
struct SoundBank {
    /// Sound id -> variants (speech 43 has two).
    sfx: HashMap<u8, Vec<Handle<AudioSource>>>,
    /// The ids of speech.
    speech: Vec<u8>,
    thunder: Vec<Handle<AudioSource>>,
    beacon: Option<Handle<AudioSource>>,
    siren: Option<Handle<AudioSource>>,
    /// The cannon's loop and tail.
    cannon: Option<(Handle<AudioSource>, Handle<AudioSource>)>,
    /// The other aircraft's loops, by `Engine`.
    distant: [Option<Handle<AudioSource>>; 5],
    launch: Option<Handle<AudioSource>>,
    /// The vehicles' guns, near and far: the tank's, the autocannon's.
    guns: [Option<(Handle<AudioSource>, Handle<AudioSource>)>; 2],
    heli_rotor: Option<Handle<AudioSource>>,
    heli_turbine: Option<Handle<AudioSource>>,
    /// The CH-47's rotors and turbines.
    ch47_rotor: Option<Handle<AudioSource>>,
    ch47_turbine: Option<Handle<AudioSource>>,
    /// The flown jet's engine and wind loops, and its afterburner's.
    jet_loops: Option<(Handle<AudioSource>, Handle<AudioSource>)>,
    afterburner: Option<Handle<AudioSource>>,
    /// The Sidewinder seeker's loops, by `SeekerTone`.
    aim9: [Option<Handle<AudioSource>>; 3],
}

/// A playing one-shot sound (one instance per id at a time, like the single-voice driver, but
/// for `OVERLAPPING`).
#[derive(Component)]
struct OneShot(u8);

#[derive(Component)]
struct Beacon;

/// The cannon's loop, while it fires.
#[derive(Component)]
struct CannonLoop;

/// Sound id of the cannon.
const CANNON: u8 = 6;

/// The fire warning siren's loop.
#[derive(Component)]
struct FireSiren;

/// What another aircraft sounds like (`SoundBank::distant` index).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Engine {
    Jet,
    Propeller,
    Helicopter,
    /// A quadrotor interceptor.
    Quad,
    /// A tandem-rotor helicopter (the CH-47).
    Tandem,
}

impl Engine {
    /// Range, reference distance (m) and volume.
    fn reach(self) -> (f32, f32, f32) {
        match self {
            Engine::Jet => (JET_RANGE, JET_REF, JET_VOLUME),
            Engine::Propeller => (PROPELLER_RANGE, PROPELLER_REF, PROPELLER_VOLUME),
            Engine::Helicopter | Engine::Tandem => {
                (HELICOPTER_RANGE, HELICOPTER_REF, HELICOPTER_VOLUME)
            }
            Engine::Quad => (QUAD_RANGE, QUAD_REF, QUAD_VOLUME),
        }
    }
}

const DISTANT_FILES: [&str; 5] =
    ["jet.flac", "engine.flac", "helicopter.flac", "interceptor.flac", "chinook.flac"];

/// Another aircraft heard: an AI flight unit or another player.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Heard {
    Unit(usize),
    Pilot(u8),
}

impl Heard {
    /// A slightly different pitch per aircraft, so that several don't sound alike.
    fn pitch(self) -> f32 {
        let k = match self {
            Heard::Unit(i) => i as f32,
            Heard::Pilot(id) => 1000.0 + id as f32,
        };
        0.92 + 0.16 * (k * 0.618).fract()
    }
}

/// The loop of another aircraft heard.
#[derive(Component)]
struct DistantAircraft {
    who: Heard,
    engine: Engine,
    /// The engine's own pitch (an interceptor's motors with its speed).
    rev: f32,
    /// Distance (m) last frame, and the current Doppler factor.
    dist: f32,
    doppler: f32,
}

/// A SAM launch playing.
#[derive(Component)]
struct LaunchSound;

/// A vehicle's gun report playing.
#[derive(Component)]
struct GunSound;

/// A Sidewinder seeker loop: 0 the search growl, 1 the growl with the target in the seeker,
/// 2 the lock tone.
#[derive(Component)]
struct SeekerTone(usize);

const AIM9_FILES: [&str; 3] = ["aim9_growl.flac", "aim9_seen.flac", "aim9_tone.flac"];

/// Sidewinder seeker volume (the loops keep the recording's relative loudness).
const SEEKER_VOLUME: f32 = 0.3;
/// How long (real seconds) the seeker has to see or lose the target before its sound changes,
/// so the target at the edge of the seeker's view doesn't chop it.
const SEEKER_HOLD: f32 = 0.15;

/// The flown helicopter's loops: rotor (false) and turbines (true).
#[derive(Component)]
struct HeliLoop(bool);

/// Helicopter loop volumes at full level: the rotor at idle torque and its rise to full torque
/// and in translational lift, the turbines.
const ROTOR_VOLUME: (f32, f32, f32) = (0.45, 0.35, 0.2);
const TURBINE_VOLUME: f32 = 0.35;

/// The flown jet's loops: engine (0), wind (1) and afterburner (2).
#[derive(Component)]
struct JetLoop(usize);

/// Jet engine rpm (share of full): at idle thrust, its rise per second while starting, and
/// the time constants (s) following the thrust and running down without it.
const JET_IDLE: f32 = 0.6;
const JET_START_RATE: f32 = 0.45;
const JET_SPOOL_TAU: f32 = 0.6;
const JET_RUNDOWN_TAU: f32 = 1.8;
/// Jet loop volumes at full level: the engine at full rpm, the wind at `JET_WIND_KNOTS` and
/// above.
const JET_ENGINE_VOLUME: f32 = 0.55;
/// The rpm by which the engine's loop has faded in on start.
const JET_FADE_IN: f32 = 0.2;
const JET_WIND_VOLUME: f32 = 0.45;
const JET_WIND_KNOTS: f32 = 600.0;
/// The engine loop's playback speed at idle rpm (1 at full): an octave between them.
const JET_IDLE_PITCH: f32 = 0.5;
/// The afterburner's loop volume when fully lit, and the thump as it lights: its extra volume,
/// the loop's speed then (deeper) and the seconds it dies away in.
const AB_VOLUME: f32 = 0.75;
const AB_THUMP: f32 = 0.8;
const AB_THUMP_PITCH: f32 = 0.7;
const AB_THUMP_TAU: f32 = 0.35;

/// Sound id of an anti-aircraft shell burst nearby.
pub const AIRBURST: u8 = 110;
/// Sound id of an objective achieved (`sounds/objective_complete.flac`, optional).
pub const OBJECTIVE: u8 = 111;

fn load_bank(mut commands: Commands, server: Res<AssetServer>) {
    let mut bank = SoundBank::default();
    let names = crate::asset_path::list("sounds");
    if names.is_empty() {
        warn!("assets/sounds missing: no sounds");
        commands.insert_resource(bank);
        return;
    }
    for name in names {
        let Some(stem) = name.strip_suffix(".flac") else { continue };
        let path = format!("sounds/{name}");
        if stem.starts_with("event_thunder") {
            bank.thunder.push(server.load(path));
            continue;
        }
        let speech = stem.strip_prefix("speech_");
        if let Some(rest) = stem.strip_prefix("sfx_").or(speech) {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(id) = digits.parse::<u8>() {
                bank.sfx.entry(id).or_default().push(server.load(path));
                if speech.is_some() {
                    bank.speech.push(id);
                }
            }
        }
    }
    for (tone, file) in RWR_FILES {
        if crate::asset_path::exists(&format!("sounds/rwr/{file}")) {
            bank.sfx
                .entry(rwr_id(tone))
                .or_default()
                .push(server.load(format!("sounds/rwr/{file}")));
        }
    }
    let optional = |name: &str| {
        crate::asset_path::exists(&format!("sounds/{name}"))
            .then(|| server.load(format!("sounds/{name}")))
    };
    if let Some(h) = optional("airburst.flac") {
        bank.sfx.entry(AIRBURST).or_default().push(h);
    }
    if let Some(h) = optional("objective_complete.flac") {
        bank.sfx.entry(OBJECTIVE).or_default().push(h);
    }
    if let Some(h) = optional("radio_click.flac") {
        bank.sfx.entry(crate::atc::RADIO_CLICK).or_default().push(h);
    }
    bank.beacon = optional("beacon.flac");
    bank.siren = optional("fire_siren.flac");
    bank.cannon = optional("cannon_loop.flac").zip(optional("cannon_tail.flac"));
    bank.distant = DISTANT_FILES.map(optional);
    bank.launch = optional("launch.flac");
    bank.guns = [
        optional("tank_gun.flac").zip(optional("tank_gun_far.flac")),
        optional("autocannon.flac").zip(optional("autocannon_far.flac")),
    ];
    bank.heli_rotor = optional("heli_rotor.flac");
    bank.heli_turbine = optional("heli_turbine.flac");
    bank.ch47_rotor = optional("ch47_rotor.flac");
    bank.ch47_turbine = optional("ch47_turbine.flac");
    bank.jet_loops = optional("jet_engine.flac").zip(optional("jet_wind.flac"));
    bank.afterburner = optional("afterburner.flac");
    bank.aim9 = AIM9_FILES.map(optional);
    commands.insert_resource(bank);
}

/// The view is one of the external cameras (not the cockpit, nor the ejected pilot's eyes).
fn external(view: Option<&ViewState>) -> bool {
    view.is_some_and(|v| {
        matches!(v.view, View::Chase | View::Side | View::Tactical | View::Weapon | View::Free)
    })
}

/// The flown aircraft's engine volume for the view: falling off with the eye's distance in the
/// external views.
#[derive(bevy::ecs::system::SystemParam)]
struct EngineDistance<'w, 's> {
    view: Option<Res<'w, ViewState>>,
    origin: Res<'w, FloatingOrigin>,
    eye: Query<'w, 's, &'static GlobalTransform, With<EyeCamera>>,
    player: Query<'w, 's, &'static WorldPos, With<Player>>,
}

impl EngineDistance<'_, '_> {
    fn gain(&self) -> f32 {
        if !external(self.view.as_deref()) {
            return 1.0;
        }
        let (Ok(eye), Ok(pos)) = (self.eye.single(), self.player.single()) else { return 1.0 };
        let d = (self.origin.origin + eye.translation().as_dvec3()).distance(pos.0) as f32;
        (ENGINE_REF / d.max(1.0)).min(1.0)
    }
}

/// Volume of thunder from `dist` metres away.
fn thunder_volume(dist: f32) -> f32 {
    let range = crate::weather::THUNDER_RANGE;
    let fade = ((range - dist) / (0.3 * range)).clamp(0.0, 1.0);
    THUNDER_VOLUME * (THUNDER_REF / dist.max(1.0)).min(1.0) * fade
}

/// Thunder arriving: one of the claps at random, as loud as its distance allows (at level 3).
fn thunder(
    mut commands: Commands,
    mut heard: MessageReader<Thunder>,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    settings: Res<crate::settings::Settings>,
    mut muffler: Muffler,
) {
    let Some(bank) = bank.filter(|b| !b.thunder.is_empty() && level.0 == 3) else {
        heard.clear();
        return;
    };
    for t in heard.read() {
        let handle = &bank.thunder[rand::random_range(0..bank.thunder.len())];
        let Some(handle) = muffler.once(handle) else { continue };
        debug!("thunder from {:.0} m", t.distance);
        let mix = Mixed(Channel::Effects, thunder_volume(t.distance));
        let playback = mix.settings(&settings, PlaybackSettings::DESPAWN);
        commands.spawn((AudioPlayer(handle), playback, mix, ThunderClap));
    }
}

/// A clap of thunder playing.
#[derive(Component)]
struct ThunderClap;

/// The pilot has ejected.
fn has_ejected(combat: Option<&crate::combat::Combat>) -> bool {
    combat.is_some_and(|c| c.sim.chute().is_some())
}

#[allow(clippy::too_many_arguments)]
fn play_sounds(
    mut commands: Commands,
    mut requests: MessageReader<PlaySound>,
    mut actions: MessageReader<ActionEvent>,
    mut level: ResMut<SoundLevel>,
    bank: Option<Res<SoundBank>>,
    combat: Option<Res<crate::combat::Combat>>,
    playing: Query<(Entity, &OneShot)>,
    mut silenced: Local<bool>,
    mut alarms: ResMut<Alarms>,
    mut last_detected: Local<Option<f64>>,
    real: Res<Time<Real>>,
    settings: Res<crate::settings::Settings>,
    view: Option<Res<ViewState>>,
    mut muffler: Muffler,
) {
    let now = real.elapsed_secs_f64();
    for ActionEvent(a) in actions.read() {
        match a {
            Action::Volume => {
                *level = match *level {
                    SoundLevel(3, false) => SoundLevel(3, true),
                    SoundLevel(0, _) => SoundLevel(3, false),
                    SoundLevel(l, _) => SoundLevel(l - 1, false),
                };
            }
            Action::MasterCaution => {
                info!("master caution: alarms silenced");
                alarms.silence(now);
                for (e, _) in playing.iter().filter(|(_, p)| ALARM_TONES.contains(&p.0)) {
                    commands.entity(e).despawn();
                }
            }
            _ => {}
        }
    }
    // Ejecting cuts off the aircraft's sounds at once.
    let ejected = has_ejected(combat.as_deref());
    let silence = ejected && !*silenced;
    *silenced = ejected;
    let survives = |id: u8| !ejected || EJECTED_SOUNDS.contains(&id);
    if silence {
        for (e, _) in playing.iter().filter(|(_, p)| !survives(p.0)) {
            commands.entity(e).despawn();
        }
    }
    let Some(bank) = bank else {
        requests.clear();
        return;
    };
    // Out in the external views the cockpit's own sounds go quiet.
    let outside = external(view.as_deref());
    let cockpit_only = |id: u8| {
        ALARM_TONES.contains(&id) || COCKPIT_SOUNDS.contains(&id) || bank.speech.contains(&id)
    };
    if outside {
        for (e, _) in playing.iter().filter(|(_, p)| cockpit_only(p.0)) {
            commands.entity(e).despawn();
        }
    }
    for r in requests.read() {
        if outside && cockpit_only(r.id) {
            continue;
        }
        if RWR_IDS.contains(&r.id) && !rwr_sounds(settings.rwr_tones, r.id, now, &mut last_detected)
        {
            continue;
        }
        if ALARM_TONES.contains(&r.id) && !alarms.sounds(r.id, now) {
            continue;
        }
        let allowed =
            r.priority == DIRECT || (level.0 > 0 && r.priority as i32 >= 3 - level.0 as i32);
        let busy = !OVERLAPPING.contains(&r.id)
            && playing.iter().any(|(_, p)| p.0 == r.id && survives(p.0));
        if !allowed || !survives(r.id) || busy {
            continue;
        }
        // The cannon's loop and tail take its place.
        if r.id == CANNON && bank.cannon.is_some() {
            continue;
        }
        let Some(variants) = bank.sfx.get(&r.id) else { continue };
        let handle = &variants[rand::random_range(0..variants.len())];
        debug!("sound {} (priority {})", r.id, r.priority);
        let mix = Mixed(Channel::of(r.id), 1.0);
        let playback = mix.settings(&settings, PlaybackSettings::DESPAWN);
        // Explosions through the cockpit's low-pass (once the sound has loaded).
        if r.id == EXPLOSION
            && let Some(muffled) = muffler.once(handle)
        {
            commands.spawn((AudioPlayer(muffled), playback, mix, OneShot(r.id)));
            continue;
        }
        commands.spawn((AudioPlayer(handle.clone()), playback, mix, OneShot(r.id)));
    }
}

/// The cannon: its loop while it fires (held while the flight is paused), then the tail. Ejecting
/// cuts it off.
fn cannon(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    combat: Option<Res<crate::combat::Combat>>,
    virt: Res<Time<Virtual>>,
    playing: Query<(Entity, Option<&AudioSink>), With<CannonLoop>>,
    settings: Res<crate::settings::Settings>,
) {
    let mix = || Mixed(Channel::Effects, 1.0);
    let Some((looped, tail)) = bank.as_ref().and_then(|b| b.cannon.clone()) else { return };
    let ejected = has_ejected(combat.as_deref());
    let firing = combat
        .as_deref()
        .is_some_and(|c| c.sim.gun_firing() && c.end.is_none() && !c.sim.player_absent())
        && !ejected
        && level.0 > 0;
    if !firing {
        if !playing.is_empty() && !ejected && level.0 > 0 {
            let playback = mix().settings(&settings, PlaybackSettings::DESPAWN);
            commands.spawn((AudioPlayer(tail), playback, mix(), OneShot(CANNON)));
        }
        for (e, _) in &playing {
            commands.entity(e).despawn();
        }
        return;
    }
    match playing.single() {
        Ok((_, Some(sink))) => {
            if virt.is_paused() != sink.is_paused() {
                if virt.is_paused() { sink.pause() } else { sink.play() }
            }
        }
        Ok(_) => {}
        Err(_) => {
            let playback = mix().settings(&settings, PlaybackSettings::LOOP);
            commands.spawn((AudioPlayer(looped), playback, mix(), CannonLoop));
        }
    }
}

/// An objective just achieved (ticked in the mission orders): its sound, if there is one.
fn objective_sound(
    orders: Res<crate::mfd::MissionOrders>,
    mut was: Local<[bool; 2]>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if (0..2).any(|k| orders.done[k] && !was[k]) {
        sounds.write(PlaySound { id: OBJECTIVE, priority: 3 });
    }
    *was = orders.done;
}

/// Sounds triggered from the flight step and frame update.
#[allow(clippy::too_many_arguments)]
fn flight_sounds(
    mut out: MessageWriter<PlaySound>,
    mut actions: MessageReader<ActionEvent>,
    player: Query<(&Aircraft, &crate::world::WorldPos), With<Player>>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    mut prev: Local<Option<(bool, f32)>>,
    mut said_takeoff: Local<bool>,
    mut said_land: Local<bool>,
    mut been_away: Local<bool>,
    settings: Res<crate::settings::Settings>,
) {
    let Ok((a, pos)) = player.single() else { return };
    let helicopter = settings.aircraft.spec().helicopter;
    for ActionEvent(act) in actions.read() {
        match act {
            Action::Gear if !a.on_ground => {
                out.write(PlaySound { id: 19, priority: DIRECT });
            }
            Action::ThrottleIdle if !helicopter => {
                out.write(PlaySound { id: 12, priority: 0 });
            }
            _ => {}
        }
    }
    let (was_on_ground, prev_thrust) = prev.unwrap_or((a.on_ground, a.thrust));
    // Spool up when the throttle is raised from zero actual thrust (a helicopter's turbines
    // keep running).
    if prev_thrust == 0.0 && a.set_thrust > 0.0 && !helicopter {
        out.write(PlaySound { id: 11, priority: 2 });
    }
    // Touchdown in a landing corridor (a helicopter's anywhere).
    if a.on_ground && !was_on_ground && (a.in_landing_corridor || helicopter) {
        out.write(PlaySound { id: 10, priority: 2 });
    }
    // Stall horn while stalled (repeated every frame).
    if !a.on_ground && a.velocity < a.stall_speed {
        out.write(PlaySound { id: 14, priority: 1 });
    }
    // "Cleared for takeoff" about 2 s into the mission; "Cleared to land" once on approach.
    if a.time < 1.0 {
        *said_takeoff = false;
        *said_land = false;
        *been_away = false;
    }
    if !*said_takeoff && a.time > 2.0 {
        *said_takeoff = true;
        out.write(PlaySound { id: 44, priority: DIRECT });
    }
    if let Some(m) = mission
        && !*said_land
        && a.has_flown
        && !a.on_ground
    {
        let base = &m.0.landing.position;
        let (be, bn) = crate::world::map_to_wu(base.x as u16, base.y as u16);
        let (e, n, _) = crate::world::bevy_to_wu(pos.0);
        // On approach: back within range 5000 of the landing base, descending with the gear
        // down.
        let d = (e - be).hypot(n - bn);
        if d > 10_000.0 {
            *been_away = true;
        }
        if *been_away && d < 5000.0 && a.climb_rate < 0.0 && a.gear_down {
            *said_land = true;
            out.write(PlaySound { id: 45, priority: DIRECT });
        }
    }
    *prev = Some((a.on_ground, a.thrust));
}

/// An engine loop through the filter, its speed set with its `Pitch`.
fn spawn_loop(
    commands: &mut Commands,
    muffler: &mut Muffler,
    handle: &Handle<AudioSource>,
    speed: f32,
    volume: f32,
    marker: impl Bundle,
) {
    let Some((handle, pitch)) = muffler.looped(handle, speed) else { return };
    commands.spawn((
        AudioPlayer(handle),
        PlaybackSettings {
            // The filtered sound loops itself.
            mode: PlaybackMode::Once,
            volume: Volume::Linear(volume),
            ..default()
        },
        pitch,
        marker,
    ));
}

/// The flown helicopter's rotor and turbines (instead of the engine drone), while the rotor
/// turns.
#[allow(clippy::too_many_arguments)]
fn heli_sounds(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    player: Query<&Aircraft, With<Player>>,
    combat: Option<Res<crate::combat::Combat>>,
    settings: Res<crate::settings::Settings>,
    mut loops: Query<(Entity, &HeliLoop, &Pitch, Option<&mut AudioSink>)>,
    mut muffler: Muffler,
    distance: EngineDistance,
) {
    let Some(bank) = bank else { return };
    let Ok(a) = player.single() else { return };
    let nr = a.rotor_speed();
    let on = level.0 == 3
        && settings.aircraft.spec().helicopter
        && nr > 0.05
        && a.ended.is_none()
        && !has_ejected(combat.as_deref());
    if !on {
        for (e, ..) in &loops {
            commands.entity(e).despawn();
        }
        return;
    }
    let half = if level.1 { 0.5 } else { 1.0 } * Channel::Engine.gain(&settings) * distance.gain();
    let torque = a.torque().clamp(0.0, 1.2);
    let rotor = (
        nr.max(0.3),
        half * nr
            * (ROTOR_VOLUME.0 + ROTOR_VOLUME.1 * torque + ROTOR_VOLUME.2 * a.translational_lift()),
    );
    // The turbines run down with the fuel.
    let turbine = (0.85 + 0.2 * torque, if a.fuel > 0.0 { half * TURBINE_VOLUME } else { 0.0 });
    let mut have = [false; 2];
    for (_, l, pitch, sink) in &mut loops {
        have[l.0 as usize] = true;
        let (speed, volume) = if l.0 { turbine } else { rotor };
        pitch.set(speed);
        if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(volume));
        }
    }
    let (rotor_loop, turbine_loop) = match settings.aircraft {
        crate::aircraft::AircraftKind::CH47 => (&bank.ch47_rotor, &bank.ch47_turbine),
        _ => (&bank.heli_rotor, &bank.heli_turbine),
    };
    for (turbines, handle) in [(false, rotor_loop), (true, turbine_loop)] {
        let Some(handle) = handle.as_ref().filter(|_| !have[turbines as usize]) else { continue };
        let (speed, volume) = if turbines { turbine } else { rotor };
        spawn_loop(&mut commands, &mut muffler, handle, speed, volume, HeliLoop(turbines));
    }
}

/// The flown jet's engine (pitched with its rpm) and wind (louder with airspeed), instead of
/// the engine drone.
#[allow(clippy::too_many_arguments)]
fn jet_sounds(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    player: Query<&Aircraft, With<Player>>,
    combat: Option<Res<crate::combat::Combat>>,
    settings: Res<crate::settings::Settings>,
    time: Res<Time>,
    mut rpm: Local<f32>,
    mut thump: Local<(f32, bool)>,
    mut loops: Query<(Entity, &JetLoop, &Pitch, Option<&mut AudioSink>)>,
    mut muffler: Muffler,
    distance: EngineDistance,
) {
    let Some((engine, wind)) = bank.as_ref().and_then(|b| b.jet_loops.clone()) else { return };
    let Ok(a) = player.single() else { return };
    let on = level.0 == 3
        && !settings.aircraft.spec().helicopter
        && a.ended.is_none()
        && !has_ejected(combat.as_deref());
    if !on {
        *rpm = 0.0;
        for (e, ..) in &loops {
            commands.entity(e).despawn();
        }
        return;
    }
    // The engine runs with fuel left and the throttle open (in the air also closed, at idle);
    // it winds up to idle at a steady rate, then follows the thrust.
    let dt = time.delta_secs();
    let running = (a.set_thrust > 0.0 || !a.on_ground) && a.fuel > 0.0;
    let target = JET_IDLE + (1.0 - JET_IDLE) * (a.thrust / 100.0).clamp(0.0, 1.0);
    // A flight starting in the air starts with the engine running.
    *rpm = if a.time < 0.2 {
        if running { target } else { 0.0 }
    } else if !running {
        *rpm * (-dt / JET_RUNDOWN_TAU).exp()
    } else if *rpm < JET_IDLE {
        (*rpm + JET_START_RATE * dt).min(JET_IDLE)
    } else {
        target + (*rpm - target) * (-dt / JET_SPOOL_TAU).exp()
    };
    let half = if level.1 { 0.5 } else { 1.0 } * Channel::Engine.gain(&settings) * distance.gain();
    let n = *rpm;
    // Mostly the pitch follows the rpm: the volume only fades in over the first part of the
    // start and swells a little towards full rpm.
    let pitch = if n < JET_IDLE {
        JET_IDLE_PITCH * n / JET_IDLE
    } else {
        JET_IDLE_PITCH + (1.0 - JET_IDLE_PITCH) * (n - JET_IDLE) / (1.0 - JET_IDLE)
    };
    let engine_mix = (
        pitch.max(0.05),
        half * JET_ENGINE_VOLUME * (n / JET_FADE_IN).min(1.0) * (0.85 + 0.15 * n),
    );
    let airspeed = (a.knots() / JET_WIND_KNOTS).min(1.0);
    let wind_mix = (0.9 + 0.15 * airspeed, half * JET_WIND_VOLUME * airspeed.powf(1.5));
    // The afterburner: a thump as it lights, dying away.
    let lit = a.afterburner > 0.0;
    if lit && !thump.1 {
        thump.0 = 1.0;
    }
    thump.0 *= (-dt / AB_THUMP_TAU).exp();
    thump.1 = lit;
    let ab_mix = (
        1.0 - (1.0 - AB_THUMP_PITCH) * thump.0,
        half * AB_VOLUME * (a.afterburner.sqrt() + AB_THUMP * thump.0),
    );
    let afterburner = bank.as_ref().and_then(|b| b.afterburner.clone());
    let mixes = [engine_mix, wind_mix, ab_mix];
    // Not playing: the engine run down, the afterburner out.
    let silent = [n < 0.01, false, !lit];
    let mut have = [false; 3];
    for (e, l, pitch, sink) in &mut loops {
        let (speed, volume) = mixes[l.0];
        if silent[l.0] {
            commands.entity(e).despawn();
            continue;
        }
        have[l.0] = true;
        pitch.set(speed);
        if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(volume));
        }
    }
    for (i, handle) in [(0, Some(engine)), (1, Some(wind)), (2, afterburner)] {
        let Some(handle) = handle.filter(|_| !have[i] && !silent[i]) else { continue };
        let (speed, volume) = mixes[i];
        spawn_loop(&mut commands, &mut muffler, &handle, speed, volume, JetLoop(i));
    }
}

/// The emergency beacon of the pilot on the ground waiting for the rescue: the first chirps on
/// landing, then every `BEACON_INTERVAL` (real time, held while the flight is paused).
#[allow(clippy::too_many_arguments)]
fn beacon(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    combat: Option<Res<crate::combat::Combat>>,
    real: Res<Time<Real>>,
    virt: Res<Time<Virtual>>,
    mut wait: Local<f32>,
    settings: Res<crate::settings::Settings>,
) {
    let Some(handle) = bank.as_ref().and_then(|b| b.beacon.clone()) else { return };
    let waiting = combat.is_some_and(|c| c.sim.csar().is_some() && c.end.is_none());
    if !waiting {
        *wait = 0.0;
        return;
    }
    if virt.is_paused() {
        return;
    }
    *wait -= real.delta_secs();
    if *wait <= 0.0 {
        *wait = BEACON_INTERVAL;
        if level.0 > 0 {
            let mix = Mixed(Channel::Effects, BEACON_VOLUME);
            let playback = mix.settings(&settings, PlaybackSettings::DESPAWN);
            commands.spawn((AudioPlayer(handle), playback, mix, Beacon));
        }
    }
}

/// The fire warning: a siren sounding without a break while the aircraft is on fire (held while
/// the flight is paused).
#[allow(clippy::too_many_arguments)]
fn fire_siren(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    combat: Option<Res<crate::combat::Combat>>,
    player: Query<&Aircraft, With<Player>>,
    virt: Res<Time<Virtual>>,
    siren: Query<(Entity, Option<&AudioSink>), With<FireSiren>>,
    mut alarms: ResMut<Alarms>,
    settings: Res<crate::settings::Settings>,
    view: Option<Res<ViewState>>,
) {
    let on_fire = combat
        .as_deref()
        .is_some_and(|c| c.sim.damage_effects().bits & stealth_sim::dmg::FIRE != 0);
    // The master caution silences this fire only.
    if !on_fire && alarms.fire_silenced {
        alarms.fire_silenced = false;
    }
    // Out of the aircraft: ejected, or (campaign) back at the board with the damage of the
    // sortie still on record until the next one.
    let burning = on_fire
        && combat.as_deref().is_some_and(|c| c.end.is_none() && !c.sim.player_absent())
        && !alarms.fire_silenced
        && !has_ejected(combat.as_deref())
        && player.single().is_ok_and(|a| a.ended.is_none())
        && !external(view.as_deref())
        && level.0 > 0;
    let Some(handle) = bank.as_ref().and_then(|b| b.siren.clone()).filter(|_| burning) else {
        for (e, _) in &siren {
            commands.entity(e).despawn();
        }
        return;
    };
    match siren.single() {
        Ok((_, Some(sink))) => {
            if virt.is_paused() != sink.is_paused() {
                if virt.is_paused() { sink.pause() } else { sink.play() }
            }
        }
        Ok(_) => {}
        Err(_) => {
            let mix = Mixed(Channel::Warnings, SIREN_VOLUME);
            let playback = mix.settings(&settings, PlaybackSettings::LOOP);
            commands.spawn((AudioPlayer(handle), playback, mix, FireSiren));
        }
    }
}

/// The Sidewinder seeker's growl and lock tone, while Sidewinders are selected in the air-to-air
/// HUD mode (held while the flight is paused).
#[allow(clippy::too_many_arguments)]
fn sidewinder_tones(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    combat: Option<Res<crate::combat::Combat>>,
    hud: Res<crate::hud::HudState>,
    player: Query<&Aircraft, With<Player>>,
    real: Res<Time<Real>>,
    virt: Res<Time<Virtual>>,
    mut loops: Query<(Entity, &SeekerTone, Option<&mut AudioSink>)>,
    mut held: Local<((bool, bool), f32)>,
    settings: Res<crate::settings::Settings>,
    view: Option<Res<ViewState>>,
) {
    let outside = external(view.as_deref());
    // None: silent; Some(false): growl; Some(true): lock tone.
    let state = combat.as_deref().and_then(|c| {
        let sim = &c.sim;
        let (groups, cur) = sim.weapon_groups();
        let (w, rounds) = *groups.get(cur?)?;
        let aim9 = sim
            .weapon_types()
            .get(w.max(0) as usize)
            .is_some_and(|t| stealth_sim::ir_aam(t.spec as i32));
        let on = aim9
            && rounds > 0
            && !sim.gun_selected()
            && hud.mode == crate::hud::HudMode::Air
            && c.end.is_none()
            && !sim.player_absent()
            && !has_ejected(Some(c))
            && player.single().is_ok_and(|a| a.ended.is_none())
            && !outside
            && level.0 > 0;
        let t = sim.targeting();
        on.then_some((t.lock_valid, t.lock_valid && t.in_range))
    });
    let Some(now) = state else {
        *held = default();
        for (e, ..) in &loops {
            commands.entity(e).despawn();
        }
        return;
    };
    // (seen, locked) as heard: follows the seeker once a change has lasted SEEKER_HOLD.
    if now == held.0 {
        held.1 = 0.0;
    } else if !virt.is_paused() {
        held.1 += real.delta_secs();
        if held.1 >= SEEKER_HOLD {
            *held = (now, 0.0);
        }
    }
    let (seen, locked) = held.0;
    let tone = if locked { 2 } else { seen as usize };
    let mut have = false;
    for (e, l, sink) in &mut loops {
        if l.0 != tone {
            commands.entity(e).despawn();
            continue;
        }
        have = true;
        if let Some(sink) = sink
            && virt.is_paused() != sink.is_paused()
        {
            if virt.is_paused() { sink.pause() } else { sink.play() }
        }
    }
    if have {
        return;
    }
    if let Some(handle) = bank.as_ref().and_then(|b| b.aim9[tone].clone()) {
        debug!("sidewinder {}", AIM9_FILES[tone]);
        let mix = Mixed(Channel::Warnings, SEEKER_VOLUME);
        let playback = mix.settings(&settings, PlaybackSettings::LOOP);
        commands.spawn((AudioPlayer(handle), playback, mix, SeekerTone(tone)));
    }
}

/// The other aircraft around: each one's loop plays from its position through the cockpit's
/// filter, panned by its direction from the eye (the ears follow the head) and shifted by its
/// closing speed; out of range or beyond the nearest few, it stops.
#[allow(clippy::too_many_arguments)]
fn distant_aircraft(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    combat: Option<Res<crate::combat::Combat>>,
    origin: Res<FloatingOrigin>,
    virt: Res<Time<Virtual>>,
    eye: Query<&GlobalTransform, With<EyeCamera>>,
    mut playing: Query<(
        Entity,
        &mut DistantAircraft,
        &mut WorldPos,
        &Pitch,
        Option<&mut SpatialAudioSink>,
    )>,
    settings: Res<crate::settings::Settings>,
    mut muffler: Muffler,
) {
    let gain = Channel::Engine.gain(&settings);
    let heard = level.0 == 3 && combat.as_ref().is_some_and(|c| c.end.is_none());
    let (Some(bank), Some(combat), Ok(eye_tf)) = (bank.filter(|_| heard), combat, eye.single())
    else {
        for (e, ..) in &playing {
            commands.entity(e).despawn();
        }
        return;
    };
    let ear = origin.origin + eye_tf.translation().as_dvec3();
    let sim = &combat.sim;
    // The nearest aircraft in range: (who, engine, position, distance, the engine's pitch).
    let in_range = |who, engine: Engine, p: bevy::math::DVec3, rev: f32| {
        let d = (p - ear).length() as f32;
        (d < engine.reach().0).then_some((who, engine, p, d, rev))
    };
    let mut heard: Vec<(Heard, Engine, bevy::math::DVec3, f32, f32)> = sim
        .active_aircraft()
        .into_iter()
        .filter(|(_, u, _)| {
            u.flags & fflags::DESTROYED == 0
                && (u.wx != 0 || u.wy != 0)
                // Parked (speed 0, the AI never stops in the air): engines off.
                && u.speed > 0
        })
        .filter_map(|(i, u, _)| {
            let quad =
                sim.uav_of_flight(i).filter(|d| d.kind == stealth_sim::uav::UavKind::Interceptor);
            let engine = match sim.aircraft_kind(u.spec) {
                AircraftKind::Plane => Engine::Jet,
                AircraftKind::Helicopter => Engine::Helicopter,
                AircraftKind::Drone if u.spec == stealth_mission::drones::GERAN3 as i32 => {
                    Engine::Jet
                }
                AircraftKind::Drone if quad.is_some() => Engine::Quad,
                AircraftKind::Drone => Engine::Propeller,
            };
            // An interceptor's motors spin up with the throttle (the loop is at its hover
            // throttle, 20%; two octaves up at full), and stop with its battery flat.
            let rev = match quad {
                Some(d) if d.battery <= 0.0 => return None,
                Some(d) => {
                    let hover = d.kind.perf().hover;
                    4f32.powf(((d.throttle() - hover) / (1.0 - hover)) as f32)
                }
                None => 1.0,
            };
            let (east, north, alt) = unit_position(combat.prev_flights.get(i), &u, combat.alpha);
            in_range(Heard::Unit(i), engine, wu_to_bevy(east, north, alt), rev)
        })
        .collect();
    let me = sim.active_pilot();
    for p in sim.remote_pilots().iter().filter(|p| p.id != me && p.chute.is_none()) {
        let v = combat.remote_track.view_at(p.id, combat.remote_time, p.view);
        if p.on_ground && v.knots == 0 {
            continue;
        }
        let engine = match (p.helicopter, p.chinook) {
            (true, true) => Engine::Tandem,
            (true, false) => Engine::Helicopter,
            _ => Engine::Jet,
        };
        let at = wu_to_bevy(v.wx as f64, v.wy as f64, v.alt as f64);
        heard.extend(in_range(Heard::Pilot(p.id), engine, at, 1.0));
    }
    heard.sort_by(|a, b| a.3.total_cmp(&b.3));
    heard.truncate(DISTANT_MAX);

    let dt = virt.delta_secs();
    let mut have = Vec::new();
    for (e, mut d, mut pos, pitch, sink) in &mut playing {
        let Some(&(_, engine, p, dist, rev)) =
            heard.iter().find(|h| h.0 == d.who && h.1 == d.engine)
        else {
            commands.entity(e).despawn();
            continue;
        };
        have.push(d.who);
        pos.0 = p;
        if dt > 0.0 {
            let away = (dist - d.dist) / dt;
            let target = (SPEED_OF_SOUND / (SPEED_OF_SOUND + away).max(1.0))
                .clamp(DOPPLER_RANGE.0, DOPPLER_RANGE.1);
            d.doppler += (target - d.doppler) * (dt * 4.0).min(1.0);
        }
        d.dist = dist;
        d.rev += (rev - d.rev) * (dt * 3.0).min(1.0);
        pitch.set(d.who.pitch() * d.rev * d.doppler);
        let Some(mut sink) = sink else { continue };
        if virt.is_paused() {
            sink.pause();
            continue;
        }
        sink.play();
        sink.set_volume(Volume::Linear(heard_volume(engine.reach(), dist) * gain));
    }
    for &(who, engine, p, dist, rev) in heard.iter().filter(|h| !have.contains(&h.0)) {
        let Some(handle) = &bank.distant[engine as usize] else { continue };
        let Some((handle, pitch)) = muffler.looped(handle, who.pitch() * rev) else { continue };
        debug!("heard {who:?} ({engine:?}) {dist:.0} m away");
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings {
                // The filtered sound loops itself.
                mode: PlaybackMode::Once,
                volume: Volume::Linear(heard_volume(engine.reach(), dist) * gain),
                spatial: true,
                // Positions shrunk to within rodio's unit distance: it only pans, the volume
                // falls off here.
                spatial_scale: Some(SpatialScale::new(1.0 / engine.reach().0)),
                ..default()
            },
            WorldPos(p),
            Transform::from_translation((p - origin.origin).as_vec3()),
            pitch,
            DistantAircraft { who, engine, rev, dist, doppler: 1.0 },
        ));
    }
}

/// The ears the positioned sounds are panned for: a child of the eye camera at the head, which
/// in VR (where the eye camera is the tracking root, not turning with the head) follows the
/// headset's pose.
#[derive(Component)]
struct Ears;

/// Keeps the ears on the eye camera (again after the rig is rebuilt) and at the head's pose.
fn follow_head(
    mut commands: Commands,
    eye: Query<Entity, With<EyeCamera>>,
    mut ears: Query<(Entity, &ChildOf, &mut Transform), With<Ears>>,
    heads: Res<crate::camera::HeadPoses>,
) {
    let Ok(eye) = eye.single() else { return };
    // Between the eyes, facing as the first one does (flat screen: the camera itself).
    let head = match heads.0.as_slice() {
        [] => Transform::IDENTITY,
        poses => Transform::from_translation(
            poses.iter().map(|p| p.translation).sum::<Vec3>() / poses.len() as f32,
        )
        .with_rotation(poses[0].rotation),
    };
    let mut have = false;
    for (e, parent, mut tf) in &mut ears {
        if parent.parent() != eye {
            commands.entity(e).despawn();
            continue;
        }
        have = true;
        *tf = head;
    }
    if !have {
        let ears = commands
            .spawn((
                Ears,
                head,
                // rodio's panning plays an emitter on the right louder in the left channel, so
                // the ears are swapped to come out right.
                SpatialListener {
                    left_ear_offset: Vec3::X * EAR_GAP / 2.0,
                    right_ear_offset: Vec3::X * EAR_GAP / -2.0,
                },
            ))
            .id();
        commands.entity(eye).add_child(ears);
    }
}

/// SAM launches near the eye: each one's sound plays once from the site through the cockpit's
/// filter, after the time the sound takes to get here, as loud as the distance allows.
#[allow(clippy::too_many_arguments)]
fn sam_launches(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    combat: Option<Res<crate::combat::Combat>>,
    origin: Res<FloatingOrigin>,
    virt: Res<Time<Virtual>>,
    eye: Query<&GlobalTransform, With<EyeCamera>>,
    settings: Res<crate::settings::Settings>,
    mut muffler: Muffler,
    // Launches on their way: seconds until heard, position (Bevy world).
    mut coming: Local<Vec<(f32, bevy::math::DVec3)>>,
) {
    let (Some(bank), Some(combat), Ok(eye_tf)) = (bank, combat, eye.single()) else {
        coming.clear();
        return;
    };
    let ear = origin.origin + eye_tf.translation().as_dvec3();
    for &p in &combat.sam_launches {
        let d = (p - ear).length() as f32;
        debug!("SAM launch {d:.0} m away");
        if d < LAUNCH_RANGE {
            coming.push((d / SPEED_OF_SOUND, p));
        }
    }
    let dt = virt.delta_secs();
    let Some(handle) = bank.launch.as_ref().filter(|_| level.0 >= 1) else {
        coming.clear();
        return;
    };
    for c in coming.iter_mut() {
        c.0 -= dt;
    }
    let mut i = 0;
    while i < coming.len() {
        if coming[i].0 > 0.0 {
            i += 1;
            continue;
        }
        let Some(muffled) = muffler.once(handle) else {
            i += 1;
            continue;
        };
        let (_, p) = coming.swap_remove(i);
        let dist = (p - ear).length() as f32;
        let reach = (LAUNCH_RANGE, LAUNCH_REF, LAUNCH_VOLUME);
        commands.spawn((
            AudioPlayer(muffled),
            PlaybackSettings {
                volume: Volume::Linear(
                    heard_volume(reach, dist) * Channel::Effects.gain(&settings),
                ),
                spatial: true,
                spatial_scale: Some(SpatialScale::new(1.0 / LAUNCH_RANGE)),
                ..PlaybackSettings::DESPAWN
            },
            WorldPos(p),
            Transform::from_translation((p - origin.origin).as_vec3()),
            LaunchSound,
        ));
    }
}

/// The vehicles' guns firing: each round heard when its sound gets here (the far sound beyond
/// `GUN_FAR`), as loud as its distance allows, from where it was fired.
#[allow(clippy::too_many_arguments)]
fn gun_reports(
    mut commands: Commands,
    mut fired: MessageReader<GunReport>,
    bank: Option<Res<SoundBank>>,
    level: Res<SoundLevel>,
    origin: Res<FloatingOrigin>,
    virt: Res<Time<Virtual>>,
    eye: Query<&GlobalTransform, With<EyeCamera>>,
    settings: Res<crate::settings::Settings>,
    mut muffler: Muffler,
    // Reports on their way: seconds until heard, position (Bevy world), calibre.
    mut coming: Local<Vec<(f32, bevy::math::DVec3, crate::explosions::Calibre)>>,
) {
    use crate::explosions::Calibre;
    let (Some(bank), Ok(eye_tf)) = (bank.filter(|_| level.0 >= 1), eye.single()) else {
        fired.clear();
        coming.clear();
        return;
    };
    let ear = origin.origin + eye_tf.translation().as_dvec3();
    let reach = |c: Calibre| match c {
        Calibre::Tank => TANK_GUN,
        Calibre::Cannon => AUTOCANNON,
        Calibre::Small => LIGHT_GUN,
    };
    for g in fired.read() {
        let d = (g.at - ear).length() as f32;
        // Only so many rounds on their way at once (a few columns firing bursts).
        if d < reach(g.calibre).0 && coming.len() < 48 {
            coming.push((d / SPEED_OF_SOUND, g.at, g.calibre));
        }
    }
    let dt = virt.delta_secs();
    for c in coming.iter_mut() {
        c.0 -= dt;
    }
    let mut i = 0;
    while i < coming.len() {
        if coming[i].0 > 0.0 {
            i += 1;
            continue;
        }
        let (_, p, calibre) = coming.swap_remove(i);
        let dist = (p - ear).length() as f32;
        let Some((near, far)) = bank.guns[(calibre != Calibre::Tank) as usize].as_ref() else {
            continue;
        };
        let Some(muffled) = muffler.once(if dist > GUN_FAR { far } else { near }) else {
            continue;
        };
        let (range, ..) = reach(calibre);
        // The Shilka's 23 mm: the autocannon's report, smaller and quicker.
        let speed = if calibre == Calibre::Small { 1.3 } else { rand::random_range(0.95..1.05) };
        commands.spawn((
            AudioPlayer(muffled),
            PlaybackSettings {
                volume: Volume::Linear(
                    heard_volume(reach(calibre), dist) * Channel::Effects.gain(&settings),
                ),
                speed,
                spatial: true,
                spatial_scale: Some(SpatialScale::new(1.0 / range)),
                ..PlaybackSettings::DESPAWN
            },
            WorldPos(p),
            Transform::from_translation((p - origin.origin).as_vec3()),
            GunSound,
        ));
    }
}

/// Volume of a sound with reach (range, reference distance, volume) heard `dist` metres away,
/// fading out over the last quarter of the range.
fn heard_volume((range, reference, volume): (f32, f32, f32), dist: f32) -> f32 {
    let fade = ((range - dist) / (0.25 * range)).clamp(0.0, 1.0);
    volume * (reference / dist.max(1.0)).min(1.0) * fade
}

type FlightSound = Or<(
    With<OneShot>,
    With<HeliLoop>,
    With<JetLoop>,
    With<Beacon>,
    With<FireSiren>,
    With<CannonLoop>,
    With<DistantAircraft>,
    With<LaunchSound>,
    With<GunSound>,
    With<ThunderClap>,
    With<ambient::AmbientLoop>,
)>;

fn stop_all(mut commands: Commands, q: Query<Entity, FlightSound>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Detect only: one tone per detection, again after the radars let go; no painted tone.
    #[test]
    fn rwr_detect_only() {
        let mut last = None;
        let detect = |id, t, last: &mut _| rwr_sounds(RwrTones::Detect, id, t, last);
        assert!(!detect(100, 0.0, &mut last));
        assert!(detect(101, 1.0, &mut last));
        assert!(!detect(102, 4.0, &mut last));
        assert!(!detect(101, 12.0, &mut last));
        assert!(detect(103, 13.0, &mut last));
        assert!(detect(101, 30.0, &mut last));
        assert!(!rwr_sounds(RwrTones::Launch, 101, 60.0, &mut last));
        assert!(rwr_sounds(RwrTones::Launch, 104, 60.0, &mut last));
        assert!(!rwr_sounds(RwrTones::Off, 104, 60.0, &mut last));
        assert_eq!(RwrTones::Off.step(1), RwrTones::All);
        assert_eq!(RwrTones::All.step(-1), RwrTones::Off);
    }

    /// The master caution silences the tones sounding while they repeat; a tone that pauses,
    /// or one not heard before, sounds again.
    #[test]
    fn master_caution_silences_alarms() {
        let mut a = Alarms::default();
        assert!(a.sounds(14, 0.0));
        assert!(a.sounds(103, 0.5));
        a.silence(1.0);
        assert!(a.fire_silenced);
        assert!(!a.sounds(14, 1.1));
        assert!(!a.sounds(14, 4.0));
        assert!(!a.sounds(103, 5.0));
        assert!(a.sounds(101, 5.0), "a new tone");
        assert!(a.sounds(14, 4.0 + ALARM_REARM + 0.1), "the stall horn after a pause");
        assert!(a.sounds(103, 20.0), "a new launch");
    }

    /// Thunder is loud close by, falls off with distance and is gone at the edge of its range.
    #[test]
    fn thunder_falls_off() {
        let range = crate::weather::THUNDER_RANGE;
        assert_eq!(thunder_volume(500.0), THUNDER_VOLUME);
        let (near, far) = (thunder_volume(3000.0), thunder_volume(6000.0));
        assert!((near / far - 2.0).abs() < 1e-4);
        assert!(thunder_volume(0.9 * range) < thunder_volume(0.6 * range) / 1.5);
        assert_eq!(thunder_volume(range), 0.0);
    }
}
