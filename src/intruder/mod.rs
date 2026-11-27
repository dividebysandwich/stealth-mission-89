//! The A-6 Intruder's own cockpit equipment: the controls of its two seats and the state they
//! latch in ([`IntruderPanel`]), its lights and counters, and its displays (`vdi`: the pilot's
//! vertical display indicator; `scope`: the pilot's horizontal display, the
//! bombardier/navigator's direct view radar indicator and his sensor scope).
//!
//! The cockpit models name these controls `Intruder(<Key>)` (`cockpit3d::layout::Effect`), as
//! scripts do (`press=Intruder(Vdi(TerrainClearance))`). The square pushbuttons latch in turned
//! 45 degrees (`Key::kind`: `twist`), the toggle switches and knobs show their position
//! (`IntruderPanel::position`).
//!
//! Behind the panel: `computer` (the attack-navigation computer and the panel's hold on the
//! weapons, the radar and the landing system), `terrain` (the track radar's view of the
//! ground ahead) and `sight` (the pilot's optical sight). `SM89_A6_PNG=<dir>` writes the
//! displays out as they are drawn.

pub mod computer;
pub mod crew;
pub mod scope;
pub mod sight;
pub mod terrain;
pub mod vdi;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::canvas::Canvas;
use crate::font::{Align, Fonts, stroke_text_at};
use crate::settings::AppState;

pub struct IntruderPlugin;

impl Plugin for IntruderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntruderPanel>()
            .init_resource::<computer::Computer>()
            .init_resource::<terrain::TerrainScan>()
            .init_resource::<vdi::Stream>()
            .add_message::<PressKey>()
            .add_systems(OnEnter(AppState::Flight), reset)
            .add_systems(
                Update,
                (
                    crew::share,
                    crew::camera,
                    crew::automatic,
                    computer::drive,
                    computer::compute,
                    terrain::scan,
                    stream,
                )
                    .chain()
                    .after(crate::combat::CombatSystems)
                    .run_if(in_state(AppState::Flight)),
            );
    }
}

/// A new flight: the panel as the crew finds it.
fn reset(mut panel: ResMut<IntruderPanel>) {
    *panel = IntruderPanel::default();
}

/// The ground's pattern on the vertical display streams with the flight.
fn stream(
    time: Res<Time>,
    aircraft: Query<&crate::flight::Aircraft, With<crate::flight::Player>>,
    mut stream: ResMut<vdi::Stream>,
) {
    let Ok(a) = aircraft.single() else { return };
    let v = a.path_dir().normalize_or_zero() * a.ground_knots().max(a.knots() * 0.3) * 0.5144;
    let height = ((a.altitude - a.ground_altitude) as f64 * crate::world::WU) as f32;
    stream.advance(Vec2::new(v.x, -v.z), height, time.delta_secs());
}

/// Test aid `SM89_A6_PNG=<dir>`: a display's canvas written as `<name>.png` in its own
/// colours, about once a second.
pub fn dump_png(c: &Canvas, colours: &[u32; 256], name: &'static str) {
    use std::sync::Mutex;
    static LAST: Mutex<Vec<(&'static str, std::time::Instant)>> = Mutex::new(Vec::new());
    let Ok(dir) = std::env::var("SM89_A6_PNG") else { return };
    let Ok(mut last) = LAST.lock() else { return };
    let now = std::time::Instant::now();
    match last.iter_mut().find(|l| l.0 == name) {
        Some(l) if now.duration_since(l.1).as_secs_f32() < 1.0 => return,
        Some(l) => l.1 = now,
        None => last.push((name, now)),
    }
    let px: Vec<u8> =
        c.px.iter().flat_map(|i| colours[*i as usize].to_le_bytes().into_iter().take(3)).collect();
    if let Some(img) = image::RgbImage::from_raw(c.pw() as u32, c.ph() as u32, px) {
        let _ = img.save(format!("{dir}/{name}.png"));
    }
}

/// Presses an A-6 control (scripts: `press=Intruder(Flight(Land))`), in whichever seat it is.
#[derive(Message, Debug, Clone, Copy)]
pub struct PressKey(pub Key);

/// The pilot's vertical display's picture (the DISPLAY buttons).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VdiDisplay {
    #[default]
    ContactAnalog,
    TerrainClearance,
    Standby,
    Off,
}

/// The vertical display's terrain clearance range (the RANGE MILES buttons), or its test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VdiRange {
    Mi4,
    Mi3,
    #[default]
    Mi2,
    Mi1,
    Test,
}

impl VdiRange {
    /// Nautical miles (None: the test pattern).
    pub fn miles(self) -> Option<u8> {
        Some(match self {
            VdiRange::Mi4 => 4,
            VdiRange::Mi3 => 3,
            VdiRange::Mi2 => 2,
            VdiRange::Mi1 => 1,
            VdiRange::Test => return None,
        })
    }
}

/// The pilot's horizontal display's picture (MODE SEL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PhdMode {
    Tcppi,
    EScan,
    #[default]
    Search,
}

/// A three-position power switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Power {
    #[default]
    On,
    Standby,
    Off,
}

/// The track radar's terrain clearance height (CLEAR, feet).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Clearance {
    Ft200,
    #[default]
    Ft500,
    Ft1000,
}

impl Clearance {
    pub fn feet(self) -> u16 {
        match self {
            Clearance::Ft200 => 200,
            Clearance::Ft500 => 500,
            Clearance::Ft1000 => 1000,
        }
    }
}

/// The track radar's function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TrackFunction {
    #[default]
    Display,
    Track,
    Boresight,
}

/// The track radar's display scale (DISPLAY SCALE MILES).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DisplayScale {
    Mi5,
    #[default]
    Mi10,
}

impl DisplayScale {
    pub fn miles(self) -> u8 {
        match self {
            DisplayScale::Mi5 => 5,
            DisplayScale::Mi10 => 10,
        }
    }
}

/// The flight mode buttons of the bombardier/navigator's mode panel: one is in at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FlightMode {
    Land,
    #[default]
    Flight,
    TakeOff,
    PreFlight,
    Off,
    Test,
}

/// The attack mode buttons of the armament panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackMode {
    LabTarget,
    LabIp,
    Rocket,
    HiLoft,
    Straight,
    General,
}

/// The armament panel's G buttons: the pull-up of the attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PullG {
    G5,
    G4_5,
    #[default]
    G4,
    G3_5,
    G3,
}

impl PullG {
    pub fn g(self) -> f32 {
        match self {
            PullG::G5 => 5.0,
            PullG::G4_5 => 4.5,
            PullG::G4 => 4.0,
            PullG::G3_5 => 3.5,
            PullG::G3 => 3.0,
        }
    }
}

/// A station's select switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StationSelect {
    Bombs,
    #[default]
    Off,
    Rockets,
}

/// How the selected stations release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ReleaseMode {
    Salvo,
    Train,
    #[default]
    Step,
}

/// The search radar's range scale (nautical miles), by its knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SearchRange {
    Mi15,
    #[default]
    Mi30,
    Mi60,
    Mi150,
}

impl SearchRange {
    pub fn miles(self) -> u16 {
        match self {
            SearchRange::Mi15 => 15,
            SearchRange::Mi30 => 30,
            SearchRange::Mi60 => 60,
            SearchRange::Mi150 => 150,
        }
    }
}

/// The computer keyboard's address keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Address {
    Base,
    CheckPoint,
    PresLoca,
    RelAdv,
    Target,
    AimPointOne,
    AimPointTwo,
    CruiseAlt,
    MapCenter,
    Store,
    Wind,
    GlidePath,
}

impl Address {
    pub const ALL: [Address; 12] = [
        Address::Base,
        Address::CheckPoint,
        Address::PresLoca,
        Address::RelAdv,
        Address::Target,
        Address::AimPointOne,
        Address::AimPointTwo,
        Address::CruiseAlt,
        Address::MapCenter,
        Address::Store,
        Address::Wind,
        Address::GlidePath,
    ];
}

/// The stations with a select switch (the A-6C's pod hangs on the third, the centre line).
pub const STATIONS: u8 = 5;
/// The navigation mode buttons.
pub const NAV_BUTTONS: u8 = 6;
/// The keyboard's quantity keys (N/+, S/-, 1..5, E, W, 6..9, 0) and action keys.
pub const QUANTITY_KEYS: u8 = 14;
pub const ACTION_KEYS: u8 = 6;
/// The interval counter's largest value (feet or seconds).
pub const INTERVAL_MAX: u16 = 999;
/// The pitch trim knob's stops either side of the middle.
pub const PITCH_TRIM: i8 = 3;

/// Every control of the A-6's two cockpits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// Pilot's control box, VERTICAL: the DISPLAY and RANGE MILES buttons, the path brightness
    /// and pitch trim knobs.
    Vdi(VdiDisplay),
    Range(VdiRange),
    PathBright,
    PitchTrim,
    /// HORIZONTAL: the MODE SEL and power switches, ONE SCAN and ERASE.
    PhdMode,
    PhdPower,
    OneScan,
    Erase,
    /// TRACK RADAR: the CLEAR, function and DISPLAY SCALE MILES switches.
    Clearance,
    TrackFunction,
    DisplayScale,
    /// The bombardier/navigator's mode panel: the flight mode buttons, the six navigation
    /// buttons (0 the top one), the transfer switch that gives them their second meanings and
    /// the NAV CHK PT / NORM switch.
    Flight(FlightMode),
    Nav(u8),
    NavTransfer,
    NavCheckPoint,
    /// The armament panel: the attack mode and G buttons, the station select switches (1 to
    /// `STATIONS`), the release mode switch, the interval counter's INCR and DECR, master arm.
    Attack(AttackMode),
    PullG(PullG),
    Station(u8),
    ReleaseMode,
    IntervalUp,
    IntervalDown,
    MasterArm,
    /// The search radar: its power switch, range knob and moving target indication.
    SearchPower,
    SearchRange,
    Amti,
    /// The computer keyboard: an address key (in until RELEASE), the release bar, CLEAR, the
    /// quantity keys (0 to `QUANTITY_KEYS` - 1) and the action keys.
    Address(Address),
    Release,
    Clear,
    Quantity(u8),
    Action(u8),
    /// The bombardier/navigator's camera button: a frame from the camera pod, whatever is
    /// selected.
    Camera,
}

pub(crate) const VDI_DISPLAYS: [VdiDisplay; 4] =
    [VdiDisplay::ContactAnalog, VdiDisplay::TerrainClearance, VdiDisplay::Standby, VdiDisplay::Off];
pub(crate) const VDI_RANGES: [VdiRange; 5] =
    [VdiRange::Mi4, VdiRange::Mi3, VdiRange::Mi2, VdiRange::Mi1, VdiRange::Test];
pub const FLIGHT_MODES: [FlightMode; 6] = [
    FlightMode::Land,
    FlightMode::Flight,
    FlightMode::TakeOff,
    FlightMode::PreFlight,
    FlightMode::Off,
    FlightMode::Test,
];
pub const ATTACK_MODES: [AttackMode; 6] = [
    AttackMode::LabTarget,
    AttackMode::LabIp,
    AttackMode::Rocket,
    AttackMode::HiLoft,
    AttackMode::Straight,
    AttackMode::General,
];
pub const PULL_GS: [PullG; 5] = [PullG::G5, PullG::G4_5, PullG::G4, PullG::G3_5, PullG::G3];
pub const QUANTITY_LABELS: [&str; QUANTITY_KEYS as usize] =
    ["N/+", "S/-", "1", "2", "3", "4", "5", "E", "W", "6", "7", "8", "9", "0"];
pub const ACTION_LABELS: [&str; ACTION_KEYS as usize] =
    ["POS", "ALT", "DIST", "BRG\nDIR", "VEL", "WT"];
/// The navigation buttons' legends with the transfer switch left, and right.
pub const NAV_LABELS: [[&str; NAV_BUTTONS as usize]; 2] = [
    ["BASE\nNO AP", "BASE\nAP 1", "BASE\nAP 2", "DATA\nLINK", "TACAN", "UHF DF"],
    ["TGT\nNO AP", "TGT\nAP 1", "TGT\nAP 2", "OPPORT\nTGT", "STAB", "UNSTAB"],
];

impl Key {
    /// Every key.
    pub fn all() -> Vec<Key> {
        let mut v = Vec::new();
        v.extend(VDI_DISPLAYS.map(Key::Vdi));
        v.extend(VDI_RANGES.map(Key::Range));
        v.extend([
            Key::PathBright,
            Key::PitchTrim,
            Key::PhdMode,
            Key::PhdPower,
            Key::OneScan,
            Key::Erase,
            Key::Clearance,
            Key::TrackFunction,
            Key::DisplayScale,
        ]);
        v.extend(FLIGHT_MODES.map(Key::Flight));
        v.extend((0..NAV_BUTTONS).map(Key::Nav));
        v.extend([Key::NavTransfer, Key::NavCheckPoint]);
        v.extend(ATTACK_MODES.map(Key::Attack));
        v.extend(PULL_GS.map(Key::PullG));
        v.extend((1..=STATIONS).map(Key::Station));
        v.extend([
            Key::ReleaseMode,
            Key::IntervalUp,
            Key::IntervalDown,
            Key::MasterArm,
            Key::SearchPower,
            Key::SearchRange,
            Key::Amti,
        ]);
        v.extend(Address::ALL.map(Key::Address));
        v.extend([Key::Release, Key::Clear]);
        v.extend((0..QUANTITY_KEYS).map(Key::Quantity));
        v.extend((0..ACTION_KEYS).map(Key::Action));
        v.push(Key::Camera);
        v
    }

    /// A key by its `Debug` text (`Flight(Land)`, `Station(2)`), as the cockpit models and
    /// scripts write it.
    pub fn parse(s: &str) -> Option<Key> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        Key::all().into_iter().find(|k| format!("{k:?}") == s)
    }

    /// How the control is modelled (the cockpit model's `kind`): `twist` a square pushbutton
    /// that latches in turned 45 degrees, `switch` a toggle switch, `knob` a rotary knob, `cap`
    /// a key.
    pub fn kind(self) -> &'static str {
        match self {
            Key::Vdi(_)
            | Key::Range(_)
            | Key::Flight(_)
            | Key::Nav(_)
            | Key::Attack(_)
            | Key::PullG(_) => "twist",
            Key::PhdMode
            | Key::PhdPower
            | Key::Clearance
            | Key::TrackFunction
            | Key::DisplayScale
            | Key::NavTransfer
            | Key::NavCheckPoint
            | Key::Station(_)
            | Key::ReleaseMode
            | Key::MasterArm
            | Key::SearchPower
            | Key::Amti => "switch",
            Key::PathBright | Key::PitchTrim | Key::SearchRange => "knob",
            Key::OneScan
            | Key::Erase
            | Key::IntervalUp
            | Key::IntervalDown
            | Key::Address(_)
            | Key::Release
            | Key::Clear
            | Key::Quantity(_)
            | Key::Action(_)
            | Key::Camera => "cap",
        }
    }

    /// The control's legend as the panel has it (lines separated by '\n').
    pub fn label(self) -> &'static str {
        match self {
            Key::Vdi(VdiDisplay::ContactAnalog) => "CONT\nANALOG",
            Key::Vdi(VdiDisplay::TerrainClearance) => "TC",
            Key::Vdi(VdiDisplay::Standby) => "STBY",
            Key::Vdi(VdiDisplay::Off) => "OFF",
            Key::Range(VdiRange::Mi4) => "4",
            Key::Range(VdiRange::Mi3) => "3",
            Key::Range(VdiRange::Mi2) => "2",
            Key::Range(VdiRange::Mi1) => "1",
            Key::Range(VdiRange::Test) => "TEST",
            Key::PathBright => "PATH\nBRIGHT",
            Key::PitchTrim => "PITCH\nTRIM",
            Key::PhdMode => "MODE\nSEL",
            Key::PhdPower => "PHD\nPOWER",
            Key::OneScan => "ONE SCAN",
            Key::Erase => "ERASE",
            Key::Clearance => "CLEAR",
            Key::TrackFunction => "TRACK\nRADAR",
            Key::DisplayScale => "SCALE\nMILES",
            Key::Flight(FlightMode::Land) => "LAND",
            Key::Flight(FlightMode::Flight) => "FLIGHT",
            Key::Flight(FlightMode::TakeOff) => "TAKE\nOFF",
            Key::Flight(FlightMode::PreFlight) => "PRE\nFLT",
            Key::Flight(FlightMode::Off) => "OFF",
            Key::Flight(FlightMode::Test) => "TEST",
            Key::Nav(n) => NAV_LABELS[0][n.min(NAV_BUTTONS - 1) as usize],
            Key::NavTransfer => "NAV\nXFER",
            Key::NavCheckPoint => "NAV\nCHK PT",
            Key::Attack(AttackMode::LabTarget) => "LAB\nTGT",
            Key::Attack(AttackMode::LabIp) => "LAB\nIP",
            Key::Attack(AttackMode::Rocket) => "ROCKET",
            Key::Attack(AttackMode::HiLoft) => "HI\nLOFT",
            Key::Attack(AttackMode::Straight) => "STRAIGHT",
            Key::Attack(AttackMode::General) => "GENERAL",
            Key::PullG(PullG::G5) => "5",
            Key::PullG(PullG::G4_5) => "4.5",
            Key::PullG(PullG::G4) => "4",
            Key::PullG(PullG::G3_5) => "3.5",
            Key::PullG(PullG::G3) => "3",
            Key::Station(1) => "STA 1",
            Key::Station(2) => "STA 2",
            Key::Station(3) => "STA 3",
            Key::Station(4) => "STA 4",
            Key::Station(_) => "STA 5",
            Key::ReleaseMode => "RELEASE\nMODE",
            Key::IntervalUp => "INCR",
            Key::IntervalDown => "DECR",
            Key::MasterArm => "MASTER\nARM",
            Key::SearchPower => "SEARCH\nRADAR",
            Key::SearchRange => "RANGE",
            Key::Amti => "AMTI",
            Key::Address(Address::Base) => "BASE",
            Key::Address(Address::CheckPoint) => "CHECK\nPOINT",
            Key::Address(Address::PresLoca) => "PRES\nLOCA",
            Key::Address(Address::RelAdv) => "REL\nADV",
            Key::Address(Address::Target) => "TARGET",
            Key::Address(Address::AimPointOne) => "AIM\nPOINT\nONE",
            Key::Address(Address::AimPointTwo) => "AIM\nPOINT\nTWO",
            Key::Address(Address::CruiseAlt) => "CRUISE\nALT",
            Key::Address(Address::MapCenter) => "MAP\nCENTER",
            Key::Address(Address::Store) => "STORE",
            Key::Address(Address::Wind) => "WIND",
            Key::Address(Address::GlidePath) => "GLIDE\nPATH",
            Key::Release => "RELEASE",
            Key::Clear => "CLEAR",
            Key::Quantity(n) => QUANTITY_LABELS[n.min(QUANTITY_KEYS - 1) as usize],
            Key::Action(n) => ACTION_LABELS[n.min(ACTION_KEYS - 1) as usize],
            Key::Camera => "CAMERA",
        }
    }

    /// The tooltip's text (a message of `assets/i18n/*/tooltips.ftl`): what the control does
    /// in the game, in a few words.
    pub fn tip(self) -> &'static str {
        match self {
            Key::Vdi(VdiDisplay::ContactAnalog) => "tip-a6-vdi-picture",
            Key::Vdi(VdiDisplay::TerrainClearance) => "tip-a6-vdi-terrain",
            Key::Vdi(_) => "tip-a6-vdi-blank",
            Key::Range(VdiRange::Test) => "tip-a6-vdi-test",
            Key::Range(_) => "tip-a6-vdi-miles",
            Key::PathBright => "tip-a6-path-bright",
            Key::PitchTrim => "tip-a6-pitch-trim",
            Key::PhdMode => "tip-a6-phd-mode",
            Key::PhdPower => "tip-a6-phd-power",
            Key::Clearance => "tip-a6-clearance",
            Key::DisplayScale => "tip-a6-display-scale",
            Key::Flight(FlightMode::Land) => "tip-a6-land",
            Key::Flight(FlightMode::Flight) => "tip-a6-flight",
            Key::Flight(FlightMode::Off) => "tip-a6-computer-off",
            Key::Flight(FlightMode::Test) => "tip-a6-lamp-test",
            Key::Attack(AttackMode::Rocket) => "tip-a6-rocket",
            Key::Attack(AttackMode::HiLoft) => "tip-a6-hi-loft",
            Key::Attack(_) => "tip-a6-attack",
            Key::PullG(_) => "tip-a6-g-buttons",
            Key::Station(_) => "tip-a6-station-switch",
            Key::ReleaseMode => "tip-a6-release-mode",
            Key::IntervalUp => "tip-a6-interval-up",
            Key::IntervalDown => "tip-a6-interval-down",
            Key::MasterArm => "tip-a6-arm",
            Key::SearchPower => "tip-a6-search-power",
            Key::SearchRange => "tip-a6-search-range",
            Key::Amti => "tip-a6-amti",
            Key::Address(Address::Target) => "tip-a6-target",
            Key::Release => "tip-a6-release-bar",
            Key::Clear => "tip-a6-clear",
            Key::Quantity(n) => match QUANTITY_LABELS.get(n as usize).copied() {
                Some("N/+") => "tip-a6-north",
                Some("E") => "tip-a6-east",
                Some("S/-" | "W") => "tip-a6-south-west",
                _ => "tip-a6-digit",
            },
            Key::Action(0) => "tip-a6-pos",
            Key::Camera => "tip-a6-camera",
            // They latch or move, and that is all.
            Key::OneScan
            | Key::Erase
            | Key::TrackFunction
            | Key::Flight(_)
            | Key::Nav(_)
            | Key::NavTransfer
            | Key::NavCheckPoint
            | Key::Address(_)
            | Key::Action(_) => "tip-a6-no-effect",
        }
    }
}

/// The state the A-6's controls latch in, for both seats (one panel a crew: the other crew
/// member's copy is to follow it).
#[derive(Resource, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct IntruderPanel {
    /// Pilot's control box, VERTICAL.
    pub vdi: VdiDisplay,
    pub range: VdiRange,
    pub path_bright: bool,
    /// The pitch trim knob (-`PITCH_TRIM` ..= `PITCH_TRIM` steps).
    pub pitch_trim: i8,
    /// HORIZONTAL: the mode and power switches, ONE SCAN latched, the ERASE presses so far.
    pub phd_mode: PhdMode,
    pub phd_power: Power,
    pub one_scan: bool,
    pub erases: u8,
    /// The camera button's presses so far (the pilot's client takes each new one, in either
    /// seat).
    pub photos: u8,
    /// TRACK RADAR.
    pub clearance: Clearance,
    pub track: TrackFunction,
    pub scale: DisplayScale,
    /// The mode panel: the flight mode, the navigation button in (0 the top one), the transfer
    /// switch to the right (the buttons' second meanings), NAV CHK PT.
    pub flight: FlightMode,
    pub nav: u8,
    pub nav_transfer: bool,
    pub nav_checkpoint: bool,
    /// The armament panel.
    pub attack: Option<AttackMode>,
    pub pull_g: PullG,
    pub stations: [StationSelect; STATIONS as usize],
    pub release: ReleaseMode,
    /// The FT OR SEC counter (0 ..= `INTERVAL_MAX`).
    pub interval: u16,
    pub master_arm: bool,
    /// The search radar.
    pub search_power: Power,
    pub search_range: SearchRange,
    pub amti: bool,
    /// The keyboard's address keys that are in (bit `Address as u16`; two can be), and the
    /// quantity, action and CLEAR presses so far.
    pub addresses: u16,
    pub key_presses: u16,
    /// The number last keyed (1 to 9, 0), until the computer takes it.
    #[serde(default)]
    pub entry: Option<u8>,
    /// Target coordinates being keyed (TARGET in, N or S, the north, E or W, the east), and
    /// the target's grid coordinates (hundredths of a kilometre east and north, `grid_of`)
    /// that POS entered: the computer steers to them.
    #[serde(default)]
    pub keying: Option<Keying>,
    #[serde(default)]
    pub keyed: Option<[u16; 2]>,
}

/// Coordinates as they are keyed: the field the digits go to, each field's value so far (None
/// until its N, S, E or W is pressed) and its sign (S, W), and whether POS refused them (until
/// the next quantity key or CLEAR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Keying {
    /// 0 north, 1 east.
    pub field: usize,
    pub value: [Option<u32>; 2],
    pub negative: [bool; 2],
    #[serde(default)]
    pub error: bool,
}

/// The largest grid coordinate (the map's 512 km, in hundredths of a kilometre).
pub const GRID_MAX: u32 = 51_200;
/// The largest value a field takes (five digits).
const FIELD_MAX: u32 = 99_999;

/// A map point's grid coordinates, as the kneeboard gives them and the keyboard takes them:
/// hundredths of a kilometre east and north of the map's south-west corner.
pub fn grid_of(x: i32, y: i32) -> [u16; 2] {
    let km100 = |u: i32| ((u.clamp(0, 0x8000) as u32 * 25 + 8) / 16).min(GRID_MAX) as u16;
    [km100(x), km100(0x8000 - y)]
}

/// Grid coordinates as written on the kneeboard and keyed: `N12345 E06789`.
pub fn grid_text(g: [u16; 2]) -> String {
    format!("N{:05} E{:05}", g[1], g[0])
}

/// The point at grid coordinates `g`, in world units.
pub fn grid_waypoint(g: [u16; 2]) -> crate::hud::Waypoint {
    // A map unit is 32 world units and 1/64 km.
    let wu = |v: u16| v as f64 * 0.64 * 32.0;
    crate::hud::Waypoint { east: wu(g[0]), north: wu(g[1]) }
}

impl Default for IntruderPanel {
    fn default() -> Self {
        Self {
            vdi: VdiDisplay::default(),
            range: VdiRange::default(),
            path_bright: true,
            pitch_trim: 0,
            phd_mode: PhdMode::default(),
            phd_power: Power::On,
            one_scan: false,
            erases: 0,
            photos: 0,
            clearance: Clearance::default(),
            track: TrackFunction::default(),
            scale: DisplayScale::default(),
            flight: FlightMode::default(),
            nav: 0,
            nav_transfer: false,
            nav_checkpoint: false,
            attack: None,
            pull_g: PullG::default(),
            stations: [StationSelect::Off; STATIONS as usize],
            release: ReleaseMode::default(),
            interval: 100,
            master_arm: false,
            search_power: Power::On,
            search_range: SearchRange::default(),
            amti: false,
            addresses: 0,
            key_presses: 0,
            entry: None,
            keying: None,
            keyed: None,
        }
    }
}

/// The next of `all` after `v`, round again.
fn next<T: Copy + PartialEq>(all: &[T], v: T) -> T {
    let i = all.iter().position(|a| *a == v).unwrap_or(0);
    all[(i + 1) % all.len()]
}

pub(crate) const PHD_MODES: [PhdMode; 3] = [PhdMode::Tcppi, PhdMode::EScan, PhdMode::Search];
pub(crate) const POWERS: [Power; 3] = [Power::On, Power::Standby, Power::Off];
pub(crate) const CLEARANCES: [Clearance; 3] =
    [Clearance::Ft200, Clearance::Ft500, Clearance::Ft1000];
pub(crate) const TRACKS: [TrackFunction; 3] =
    [TrackFunction::Display, TrackFunction::Track, TrackFunction::Boresight];
pub(crate) const SCALES: [DisplayScale; 2] = [DisplayScale::Mi5, DisplayScale::Mi10];
pub(crate) const SELECTS: [StationSelect; 3] =
    [StationSelect::Bombs, StationSelect::Off, StationSelect::Rockets];
pub(crate) const RELEASES: [ReleaseMode; 3] =
    [ReleaseMode::Salvo, ReleaseMode::Train, ReleaseMode::Step];
pub(crate) const SEARCH_RANGES: [SearchRange; 4] =
    [SearchRange::Mi15, SearchRange::Mi30, SearchRange::Mi60, SearchRange::Mi150];

impl IntruderPanel {
    /// A control pressed: a pushbutton goes in and the one of its group that was in comes out
    /// (an attack mode button pressed again comes out itself), a switch or knob goes to its next
    /// position, a key does what it does.
    pub fn press(&mut self, key: Key) {
        match key {
            Key::Vdi(d) => self.vdi = d,
            Key::Range(r) => self.range = r,
            Key::PathBright => self.path_bright = !self.path_bright,
            Key::PitchTrim => {
                self.pitch_trim =
                    if self.pitch_trim >= PITCH_TRIM { -PITCH_TRIM } else { self.pitch_trim + 1 }
            }
            Key::PhdMode => self.phd_mode = next(&PHD_MODES, self.phd_mode),
            Key::PhdPower => self.phd_power = next(&POWERS, self.phd_power),
            Key::OneScan => self.one_scan = !self.one_scan,
            Key::Erase => self.erases = self.erases.wrapping_add(1),
            Key::Camera => self.photos = self.photos.wrapping_add(1),
            Key::Clearance => self.clearance = next(&CLEARANCES, self.clearance),
            Key::TrackFunction => self.track = next(&TRACKS, self.track),
            Key::DisplayScale => self.scale = next(&SCALES, self.scale),
            Key::Flight(m) => self.flight = m,
            Key::Nav(n) => self.nav = n.min(NAV_BUTTONS - 1),
            Key::NavTransfer => self.nav_transfer = !self.nav_transfer,
            Key::NavCheckPoint => self.nav_checkpoint = !self.nav_checkpoint,
            Key::Attack(m) => self.attack = if self.attack == Some(m) { None } else { Some(m) },
            Key::PullG(g) => self.pull_g = g,
            Key::Station(n) => {
                if let Some(s) = self.stations.get_mut(n.wrapping_sub(1) as usize) {
                    *s = next(&SELECTS, *s);
                }
            }
            Key::ReleaseMode => self.release = next(&RELEASES, self.release),
            Key::IntervalUp => self.interval = (self.interval + 10).min(INTERVAL_MAX),
            Key::IntervalDown => self.interval = self.interval.saturating_sub(10),
            Key::MasterArm => self.master_arm = !self.master_arm,
            Key::SearchPower => self.search_power = next(&POWERS, self.search_power),
            Key::SearchRange => self.search_range = next(&SEARCH_RANGES, self.search_range),
            Key::Amti => self.amti = !self.amti,
            Key::Address(a) => self.addresses |= 1 << a as u16,
            Key::Release => {
                self.addresses = 0;
                self.keying = None;
            }
            Key::Quantity(n) => {
                self.key_presses = self.key_presses.wrapping_add(1);
                self.quantity(n);
            }
            Key::Clear => {
                self.key_presses = self.key_presses.wrapping_add(1);
                // A keying under way is cleared; with none, TARGET's keyed coordinates.
                if self.keying.take().is_none() && self.address_in(Address::Target) {
                    self.keyed = None;
                }
            }
            Key::Action(n) => {
                self.key_presses = self.key_presses.wrapping_add(1);
                if n == 0 && self.address_in(Address::Target) {
                    self.enter_position();
                }
            }
        }
    }

    /// A quantity key: with TARGET in, N, S, E and W start a coordinate and the digits go to
    /// it (shifted in from the right, five at most); a digit with none started is the route's
    /// point of that number.
    fn quantity(&mut self, n: u8) {
        let label = QUANTITY_LABELS.get(n as usize).copied().unwrap_or("");
        let digit: Option<u8> = label.parse().ok();
        self.entry = None;
        let letter = match label {
            "N/+" => Some((0, false)),
            "S/-" => Some((0, true)),
            "E" => Some((1, false)),
            "W" => Some((1, true)),
            _ => None,
        };
        if let Some((field, negative)) = letter
            && self.address_in(Address::Target)
        {
            let k = self.keying.get_or_insert_with(Keying::default);
            k.error = false;
            k.field = field;
            k.value[field] = Some(0);
            k.negative[field] = negative;
        } else if let Some(k) = self.keying.as_mut() {
            k.error = false;
            if let (Some(d), Some(v)) = (digit, k.value[k.field])
                && v * 10 + d as u32 <= FIELD_MAX
            {
                k.value[k.field] = Some(v * 10 + d as u32);
            }
        } else {
            self.entry = digit;
        }
    }

    /// POS: the coordinates keyed become the target, if both are and lie on the map.
    fn enter_position(&mut self) {
        let Some(k) = self.keying else { return };
        let field =
            |i: usize| k.value[i].filter(|v| !k.negative[i] || *v == 0).filter(|v| *v <= GRID_MAX);
        if let (Some(north), Some(east)) = (field(0), field(1)) {
            self.keyed = Some([east as u16, north as u16]);
            self.keying = None;
        } else if let Some(k) = self.keying.as_mut() {
            k.error = true;
        }
    }

    /// POS refused the coordinates keyed (TARGET in): the storage window shows ERROR and the
    /// COMPUTER lights come on.
    pub fn keying_error(&self) -> bool {
        self.address_in(Address::Target) && self.keying.is_some_and(|k| k.error)
    }

    /// The point the keyed coordinates are at (world units).
    pub fn keyed_waypoint(&self) -> Option<crate::hud::Waypoint> {
        self.keyed.map(grid_waypoint)
    }

    /// The control stays in (a pushbutton turned 45 degrees, an address key down).
    pub fn latched(&self, key: Key) -> bool {
        match key {
            Key::Vdi(d) => self.vdi == d,
            Key::Range(r) => self.range == r,
            Key::Flight(m) => self.flight == m,
            Key::Nav(n) => self.nav == n,
            Key::Attack(m) => self.attack == Some(m),
            Key::PullG(g) => self.pull_g == g,
            Key::Address(a) => self.address_in(a),
            Key::OneScan => self.one_scan,
            _ => false,
        }
    }

    /// An address key is in.
    pub fn address_in(&self, a: Address) -> bool {
        self.addresses & (1 << a as u16) != 0
    }

    /// A switch's or knob's position and how many it has: a switch's first is up, a knob's is
    /// fully anticlockwise. None for the buttons and keys.
    pub fn position(&self, key: Key) -> Option<(u8, u8)> {
        let at = |i: Option<usize>, n: usize| Some((i.unwrap_or(0) as u8, n as u8));
        fn index<T: PartialEq>(all: &[T], v: T) -> Option<usize> {
            all.iter().position(|a| *a == v)
        }
        match key {
            Key::PathBright => Some((self.path_bright as u8, 2)),
            Key::PitchTrim => {
                Some(((self.pitch_trim + PITCH_TRIM) as u8, 2 * PITCH_TRIM as u8 + 1))
            }
            Key::PhdMode => at(index(&PHD_MODES, self.phd_mode), 3),
            Key::PhdPower => at(index(&POWERS, self.phd_power), 3),
            Key::Clearance => at(index(&CLEARANCES, self.clearance), 3),
            Key::TrackFunction => at(index(&TRACKS, self.track), 3),
            Key::DisplayScale => at(index(&SCALES, self.scale), 2),
            // NAV CHK PT up, NORM down; the transfer switch's lever lies across (left: first).
            Key::NavTransfer => Some((self.nav_transfer as u8, 2)),
            Key::NavCheckPoint => Some((!self.nav_checkpoint as u8, 2)),
            Key::Station(n) => {
                at(index(&SELECTS, *self.stations.get(n.wrapping_sub(1) as usize)?), 3)
            }
            Key::ReleaseMode => at(index(&RELEASES, self.release), 3),
            // Armed: up.
            Key::MasterArm => Some((!self.master_arm as u8, 2)),
            Key::SearchPower => at(index(&POWERS, self.search_power), 3),
            Key::SearchRange => at(index(&SEARCH_RANGES, self.search_range), 4),
            Key::Amti => Some((!self.amti as u8, 2)),
            _ => None,
        }
    }
}

impl crate::cockpit3d::PanelKey for Key {
    fn kind(self) -> &'static str {
        Key::kind(self)
    }
    fn label(self) -> &'static str {
        Key::label(self)
    }
    fn tip(self) -> &'static str {
        Key::tip(self)
    }
}

impl crate::cockpit3d::PanelKeys for IntruderPanel {
    type Key = Key;
    fn press(&mut self, key: Key) -> Option<crate::cockpit3d::PanelOut> {
        IntruderPanel::press(self, key);
        None
    }
    fn latched(&self, key: Key) -> bool {
        IntruderPanel::latched(self, key)
    }
    fn position(&self, key: Key) -> Option<(u8, u8)> {
        IntruderPanel::position(self, key)
    }
}

/// The advisory lights of the A-6's panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Light {
    /// Under the vertical display.
    VdiAttack,
    VdiPlatform,
    VdiInRange,
    VdiComputer,
    /// The armament panel's.
    Reselect,
    /// On the direct view radar indicator's bezel.
    DvriEcm,
    DvriReselect,
    DvriComputer,
    DvriAttack,
    DvriGroundLock,
    DvriTrack,
    DvriDiscreteLock,
}

impl Light {
    /// The tooltip's text: when it lights.
    pub fn tip(self) -> &'static str {
        match self {
            Light::VdiAttack | Light::DvriAttack => "tip-a6-light-attack",
            Light::VdiInRange => "tip-a6-light-in-range",
            Light::VdiComputer | Light::DvriComputer => "tip-a6-light-computer",
            Light::VdiPlatform => "tip-a6-light-platform",
            Light::Reselect | Light::DvriReselect => "tip-a6-light-reselect",
            Light::DvriEcm => "tip-a6-light-ecm",
            Light::DvriGroundLock => "tip-a6-light-ground-lock",
            Light::DvriTrack => "tip-a6-light-track",
            Light::DvriDiscreteLock => "tip-a6-light-discrete-lock",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Light::VdiAttack | Light::DvriAttack => "ATTACK",
            Light::VdiPlatform => "PLATFORM",
            Light::VdiInRange => "IN RANGE",
            Light::VdiComputer | Light::DvriComputer => "COMPUTER",
            Light::Reselect | Light::DvriReselect => "RESELECT",
            Light::DvriEcm => "ECM",
            Light::DvriGroundLock => "GROUND\nLOCK ON",
            Light::DvriTrack => "TRACK",
            Light::DvriDiscreteLock => "DISCRETE\nLOCK ON",
        }
    }
}

/// Whether an advisory light is lit: ATTACK with an attack button in, IN RANGE from the
/// computer's solution, COMPUTER with the computer off, with no target to attack or with keyed coordinates refused, PLATFORM
/// in the test only, RESELECT with an attack on and no station to release, the radar's lock
/// lights by what the lock is on, ECM while the jammer radiates.
pub fn lit(light: Light, panel: &IntruderPanel, computer: &computer::Computer, ecm: bool) -> bool {
    let test = panel.flight == FlightMode::Test;
    match light {
        Light::VdiAttack | Light::DvriAttack => panel.attack.is_some() || test,
        Light::VdiInRange => computer.in_range || test,
        Light::VdiPlatform => test,
        Light::VdiComputer | Light::DvriComputer => {
            panel.flight == FlightMode::Off
                || test
                || panel.keying_error()
                || (panel.attack.is_some() && computer.target.is_none())
        }
        Light::Reselect | Light::DvriReselect => computer.reselect || test,
        Light::DvriEcm => ecm,
        Light::DvriTrack => computer.lock || computer.discrete,
        Light::DvriGroundLock => computer.designated && !computer.discrete,
        Light::DvriDiscreteLock => computer.discrete,
    }
}

/// The counters and readouts of the A-6's panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Readout {
    /// The armament panel's FT OR SEC counter.
    Interval,
    /// The direct view radar indicator's range window.
    DvriRange,
    /// The pilot's horizontal display's TIME and RANGE windows.
    PhdTime,
    PhdRange,
    /// The keyboard's TEMPORARY STORAGE readout.
    Storage,
}

impl Readout {
    /// Its legend on the panel and the tooltip's text: what it shows.
    pub fn label(self) -> &'static str {
        match self {
            Readout::Interval => "FT OR SEC",
            Readout::DvriRange | Readout::PhdRange => "RANGE",
            Readout::PhdTime => "TIME",
            Readout::Storage => "TEMPORARY STORAGE",
        }
    }

    pub fn tip(self) -> &'static str {
        match self {
            Readout::Interval => "tip-a6-readout-interval",
            Readout::DvriRange | Readout::PhdRange => "tip-a6-readout-range",
            Readout::PhdTime => "tip-a6-readout-time",
            Readout::Storage => "tip-a6-readout-storage",
        }
    }
}

/// What a readout shows: the interval counter; the range to the target in tenths of a
/// nautical mile and the seconds to it (to the release, with one coming); the keyboard's
/// storage, by the address key that is in: the coordinate being keyed, the target's bearing
/// and range, else nothing.
pub fn readout(r: Readout, panel: &IntruderPanel, computer: &computer::Computer) -> String {
    let tenths = (computer.range / 185.2).round().clamp(0.0, 9999.0) as u32;
    match r {
        Readout::Interval => format!("{:03}", panel.interval.min(INTERVAL_MAX)),
        Readout::DvriRange | Readout::PhdRange if computer.target.is_some() => {
            format!("{tenths:04}")
        }
        Readout::DvriRange | Readout::PhdRange => "0000".to_string(),
        Readout::PhdTime => {
            let secs = computer.release.unwrap_or(computer.time).clamp(0.0, 999.0);
            format!("{:03}", if computer.target.is_some() { secs as u32 } else { 0 })
        }
        Readout::Storage if panel.keying_error() => "ERROR ".to_string(),
        Readout::Storage if panel.address_in(Address::Target) && panel.keying.is_some() => {
            let k = panel.keying.unwrap_or_default();
            let letter = [["N", "S"], ["E", "W"]][k.field.min(1)][k.negative[k.field] as usize];
            format!("{letter}{:05}", k.value[k.field.min(1)].unwrap_or(0))
        }
        Readout::Storage if panel.address_in(Address::Target) && computer.target.is_some() => {
            let brg = computer.bearing.to_degrees().rem_euclid(360.0) as u32 % 360;
            format!("{brg:03}{:03}", (tenths / 10).min(999))
        }
        Readout::Storage => "N00000".to_string(),
    }
}

/// Palette colours of the panel faces: black, the legends' white, the pushbuttons' amber lens
/// (brighter while the button is in; a lit light's too), the keys' grey and their ink.
const BLACK: u8 = 5;
const WHITE: u8 = 15;
const LENS: u8 = 18;
const LENS_LIT: u8 = 86;
const KEY_FACE: u8 = 7;
const KEY_INK: u8 = 5;

/// A control's face in the instrument atlas (rectangle `r`, `k` texels per millimetre): a
/// pushbutton's square cap with its round lens (bright while it is in), a switch's base plate,
/// a knob's top with its index, a key with its legend. True if it glows.
pub fn draw_control(
    c: &mut Canvas,
    fonts: &Fonts,
    r: [i32; 4],
    key: Key,
    panel: &IntruderPanel,
    k: i32,
) -> bool {
    let [x, y, w, h] = r;
    let centre = Vec2::new(x as f32 + w as f32 / 2.0, y as f32 + h as f32 / 2.0);
    let rad = w.min(h) as f32 / 2.0;
    let pen = k as f32;
    match key.kind() {
        "twist" => {
            // A grey cap with a pale edge (its square shows on the black panel), the lens in a
            // black ring.
            let latched = panel.latched(key);
            c.fill_rect(x, y, w, h, 7);
            c.fill_rect(x + k, y + k, w - 2 * k, h - 2 * k, 8);
            disc(c, centre, rad * 0.7, BLACK);
            disc(c, centre, rad * 0.54, if latched { LENS_LIT } else { LENS });
            latched
        }
        "switch" => {
            c.fill_rect(x, y, w, h, BLACK);
            c.ring(centre, rad * 0.55, pen, 8);
            false
        }
        "knob" => {
            c.fill_rect(x, y, w, h, 8);
            c.ring(centre, rad * 0.6, pen, BLACK);
            c.stroke(
                centre - Vec2::Y * rad * 0.2,
                centre - Vec2::Y * (rad - pen),
                pen * 1.5,
                WHITE,
            );
            false
        }
        _ => {
            let latched = panel.latched(key);
            c.fill_rect(x, y, w, h, BLACK);
            let face = if latched { LENS_LIT } else { KEY_FACE };
            c.fill_rect(x + k, y + k, w - 2 * k, h - 2 * k, face);
            let inner = [x + 2 * k, y + 2 * k, w - 4 * k, h - 4 * k];
            crate::panel::draw_label(c, fonts, inner, key.label(), KEY_INK, face, 3);
            latched
        }
    }
}

/// An advisory light's face: its legend dark on the lit lamp, pale on the dark one.
pub fn draw_light(c: &mut Canvas, fonts: &Fonts, r: [i32; 4], light: Light, on: bool) {
    let (ink, back) = if on { (BLACK, LENS_LIT) } else { (LENS, 38) };
    crate::panel::draw_label(c, fonts, r, light.label(), ink, back, 3);
}

/// A counter's face: white drum digits on black.
pub fn draw_readout(c: &mut Canvas, r: [i32; 4], text: &str) {
    let [x, y, w, h] = r;
    c.fill_rect(x, y, w, h, BLACK);
    let n = text.chars().count().max(1) as f32;
    let cell = w as f32 / n;
    let size = (h as f32 * 0.62).min(cell * 1.1);
    let pen = (size / 7.0).max(1.0);
    for (i, ch) in text.chars().enumerate() {
        let cx = x as f32 + cell * (i as f32 + 0.5);
        if i > 0 {
            c.fill_rect((x as f32 + cell * i as f32) as i32, y, 1, h, 6);
        }
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        stroke_text_at(c, cx, y as f32 + h as f32 / 2.0, s, size, pen, WHITE, Align::Centre);
    }
}

/// A filled disc.
pub fn disc(c: &mut Canvas, centre: Vec2, r: f32, colour: u8) {
    c.stroke(centre, centre + Vec2::X * 0.01, 2.0 * r, colour);
}

/// A phosphor's colours for a display's canvas (`CanvasTexture::colours`): index 0 the dark
/// tube, 1 to 100 a ramp to `bright` (8-bit sRGB) and on to nearly white.
pub fn phosphor(dark: [u8; 3], bright: [u8; 3]) -> std::sync::Arc<[u32; 256]> {
    let mut lut = [u32::from_le_bytes([dark[0], dark[1], dark[2], 255]); 256];
    for (i, v) in lut.iter_mut().enumerate().skip(1).take(100) {
        let t = i as f32 / 100.0;
        // Up to the phosphor's colour by 0.8, then towards white.
        let (a, b, f) = if t < 0.8 {
            (dark.map(f32::from), bright.map(f32::from), t / 0.8)
        } else {
            (bright.map(f32::from), [255.0, 255.0, 240.0], (t - 0.8) / 0.2 * 0.6)
        };
        let [r, g, b] = std::array::from_fn(|c| (a[c] + (b[c] - a[c]) * f).round() as u8);
        *v = u32::from_le_bytes([r, g, b, 255]);
    }
    std::sync::Arc::new(lut)
}

/// A phosphor ramp's ink at brightness `t` (0..1).
pub fn ink(t: f32) -> u8 {
    (1.0 + t.clamp(0.0, 1.0) * 99.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every control, light and counter has its tooltip text.
    #[test]
    fn tips_exist() {
        let lights = [
            Light::VdiAttack,
            Light::VdiPlatform,
            Light::VdiInRange,
            Light::VdiComputer,
            Light::Reselect,
            Light::DvriEcm,
            Light::DvriReselect,
            Light::DvriComputer,
            Light::DvriAttack,
            Light::DvriGroundLock,
            Light::DvriTrack,
            Light::DvriDiscreteLock,
        ];
        let readouts = [
            Readout::Interval,
            Readout::DvriRange,
            Readout::PhdTime,
            Readout::PhdRange,
            Readout::Storage,
        ];
        let ids = Key::all()
            .into_iter()
            .map(Key::tip)
            .chain(lights.map(Light::tip))
            .chain(readouts.map(Readout::tip));
        for id in ids {
            assert!(stealth_i18n::exists(id), "{id}");
        }
    }

    /// The models' and scripts' names of the keys read back.
    #[test]
    fn keys_parse() {
        let all = Key::all();
        for (i, k) in all.iter().enumerate() {
            assert_eq!(Key::parse(&format!("{k:?}")), Some(*k));
            assert!(!all[..i].contains(k), "{k:?} twice");
            assert!(!k.label().is_empty());
        }
        assert_eq!(Key::parse("Flight( Land )"), Some(Key::Flight(FlightMode::Land)));
        assert_eq!(Key::parse("Station(6)"), None);
        assert_eq!(Key::parse("Bogus"), None);
    }

    /// One button of a group is in at a time; an attack mode button pressed again comes out.
    #[test]
    fn radio_groups() {
        let mut p = IntruderPanel::default();
        let groups: [Vec<Key>; 5] = [
            VDI_DISPLAYS.map(Key::Vdi).to_vec(),
            VDI_RANGES.map(Key::Range).to_vec(),
            FLIGHT_MODES.map(Key::Flight).to_vec(),
            (0..NAV_BUTTONS).map(Key::Nav).collect(),
            PULL_GS.map(Key::PullG).to_vec(),
        ];
        for group in &groups {
            // One is in from the start.
            assert_eq!(group.iter().filter(|k| p.latched(**k)).count(), 1, "{group:?}");
            for &k in group {
                p.press(k);
                let latched: Vec<Key> = group.iter().copied().filter(|k| p.latched(*k)).collect();
                assert_eq!(latched, [k]);
                // Again: it stays in.
                p.press(k);
                assert!(p.latched(k));
            }
        }
        // The groups do not disturb each other.
        assert!(p.latched(Key::Vdi(VdiDisplay::Off)) && p.latched(Key::Flight(FlightMode::Test)));
        // The attack modes: none at first, one at a time, out again when pressed again.
        let attack = ATTACK_MODES.map(Key::Attack);
        assert!(attack.iter().all(|k| !p.latched(*k)));
        for k in attack {
            p.press(k);
            assert_eq!(attack.iter().filter(|a| p.latched(**a)).count(), 1);
            assert!(p.latched(k));
        }
        p.press(Key::Attack(AttackMode::General));
        assert_eq!(p.attack, None);
        assert!(attack.iter().all(|k| !p.latched(*k)));
    }

    /// The switches and knobs go round their positions; the counter stays in its range.
    #[test]
    fn switches_and_counter() {
        let mut p = IntruderPanel::default();
        for k in Key::all() {
            let Some((at, n)) = p.position(k) else {
                assert!(matches!(k.kind(), "twist" | "cap"), "{k:?}");
                continue;
            };
            assert!(matches!(k.kind(), "switch" | "knob"), "{k:?}");
            assert!(at < n, "{k:?}");
            let mut seen = vec![at];
            for _ in 1..n {
                p.press(k);
                seen.push(p.position(k).unwrap().0);
            }
            seen.sort();
            assert_eq!(seen, (0..n).collect::<Vec<_>>(), "{k:?}");
            p.press(k);
            assert_eq!(p.position(k).unwrap().0, at, "{k:?} round again");
        }
        assert_eq!(p, IntruderPanel::default());
        p.press(Key::Station(2));
        assert_eq!(p.stations[1], StationSelect::Rockets);
        assert_eq!(p.stations[0], StationSelect::Off);
        for _ in 0..200 {
            p.press(Key::IntervalUp);
        }
        assert_eq!(p.interval, INTERVAL_MAX);
        for _ in 0..200 {
            p.press(Key::IntervalDown);
        }
        assert_eq!(p.interval, 0);
    }

    /// Address keys stay in (two can be) until the release bar.
    #[test]
    fn address_keys() {
        let mut p = IntruderPanel::default();
        p.press(Key::Address(Address::Target));
        p.press(Key::Address(Address::Wind));
        p.press(Key::Quantity(3));
        assert!(p.latched(Key::Address(Address::Target)) && p.latched(Key::Address(Address::Wind)));
        assert!(!p.latched(Key::Address(Address::Base)));
        p.press(Key::Release);
        assert!(Address::ALL.iter().all(|a| !p.address_in(*a)));
    }

    /// TARGET, N and five digits, E and five digits, POS: the target's grid coordinates. The
    /// storage window shows the field being keyed; CLEAR drops a keying, then the target.
    #[test]
    fn coordinates_keyed() {
        let mut p = IntruderPanel::default();
        let digits = |p: &mut IntruderPanel, s: &str| {
            for ch in s.chars() {
                let n = QUANTITY_LABELS.iter().position(|l| *l == ch.to_string()).unwrap();
                p.press(Key::Quantity(n as u8));
            }
        };
        // Without TARGET, a letter starts nothing and a digit is the route's point.
        p.press(Key::Quantity(0));
        assert_eq!(p.keying, None);
        digits(&mut p, "3");
        assert_eq!(p.entry, Some(3));
        p.entry = None;
        p.press(Key::Address(Address::Target));
        p.press(Key::Quantity(0));
        digits(&mut p, "12345");
        assert_eq!(p.entry, None);
        let c = computer::Computer::default();
        assert_eq!(readout(Readout::Storage, &p, &c), "N12345");
        // POS with only the north keyed does nothing.
        p.press(Key::Action(0));
        assert_eq!(p.keyed, None);
        p.press(Key::Quantity(7));
        digits(&mut p, "0678");
        assert_eq!(readout(Readout::Storage, &p, &c), "E00678");
        digits(&mut p, "9");
        assert_eq!(readout(Readout::Storage, &p, &c), "E06789");
        p.press(Key::Action(0));
        assert_eq!(p.keyed, Some([6789, 12345]));
        assert_eq!(p.keying, None);
        // Off the map, or south: not taken.
        p.press(Key::Quantity(1));
        digits(&mut p, "5");
        // A sixth digit is not taken.
        p.press(Key::Quantity(0));
        digits(&mut p, "999999");
        assert_eq!(readout(Readout::Storage, &p, &c), "N99999");
        p.press(Key::Quantity(1));
        digits(&mut p, "5");
        p.press(Key::Quantity(7));
        digits(&mut p, "5");
        p.press(Key::Action(0));
        assert_eq!(p.keyed, Some([6789, 12345]));
        // The refusal shows until the next key.
        assert!(p.keying_error());
        assert_eq!(readout(Readout::Storage, &p, &c), "ERROR ");
        assert!(lit(Light::VdiComputer, &p, &c, false));
        p.press(Key::Quantity(1));
        assert!(!p.keying_error());
        assert_eq!(readout(Readout::Storage, &p, &c), "S00000");
        digits(&mut p, "1");
        p.press(Key::Action(0));
        assert!(p.keying_error());
        p.press(Key::Clear);
        assert_eq!((p.keying, p.keyed), (None, Some([6789, 12345])));
        p.press(Key::Clear);
        assert_eq!(p.keyed, None);
    }

    /// The kneeboard's coordinates key back to the point they were made of.
    #[test]
    fn grid_round_trip() {
        let (x, y) = (12_000u16, 20_000u16);
        let g = grid_of(x as i32, y as i32);
        assert_eq!(grid_text(g), format!("N{:05} E{:05}", g[1], g[0]));
        let w = grid_waypoint(g);
        let (east, north) = crate::world::map_to_wu(x, y);
        assert!((w.east - east).abs() < 32.0 && (w.north - north).abs() < 32.0);
        assert_eq!(grid_of(0x8000, 0), [GRID_MAX as u16, GRID_MAX as u16]);
    }

    /// The panel goes to the other crew member as it is.
    #[test]
    fn panel_serializes() {
        let mut p = IntruderPanel::default();
        p.press(Key::Attack(AttackMode::HiLoft));
        p.press(Key::Station(4));
        let text = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<IntruderPanel>(&text).unwrap(), p);
    }
}
