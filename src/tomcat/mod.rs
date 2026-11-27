//! The F-14B Tomcat's own cockpit equipment: the controls of its two cockpits and the state they
//! latch in ([`TomcatPanel`]), its lights and readouts, and its displays (`displays`: the
//! pilot's vertical display indicator (VDI) and horizontal situation display (HSD), the radar
//! intercept officer's detail data display (DDD), tactical information display (TID) and
//! electronic countermeasures display (ECMD)).
//!
//! The cockpit models name these controls `Tomcat(<Key>)` (`cockpit3d::layout::Effect`), as
//! scripts do (`press=Tomcat(Wcs(TwsAuto))`). The pilot's DISPLAYS panel's steering buttons
//! latch in turned 45 degrees (`Key::kind`: `twist`, the A-6's), the RIO's lit pushbuttons sink
//! and light their legends (`lamp`), the toggle switches and knobs show their position
//! (`TomcatPanel::position`).
//!
//! The panel is only the controls' state: the radar, the displays and the weapons read it
//! (`avionics` sets the AWG-9 from it and gathers what `displays` draw). `SM89_F14_PNG=<dir>` writes the displays out as they
//! are drawn.

pub mod attack;
pub mod avionics;
pub mod crew;
pub mod displays;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::canvas::Canvas;
use crate::cockpit3d::{PanelKeys, PanelOut};
use crate::font::{Align, Fonts, stroke_text_at};
use crate::input::Action;
use crate::settings::AppState;

pub struct TomcatPlugin;

impl Plugin for TomcatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TomcatPanel>()
            .init_resource::<TomcatStatus>()
            .add_message::<PressKey>()
            .add_systems(OnEnter(AppState::Flight), reset)
            .add_systems(
                Update,
                sweep_switch.after(crate::combat::CombatSystems).run_if(in_state(AppState::Flight)),
            );
        attack::build(app);
        avionics::build(app);
        crew::build(app);
    }
}

/// A new flight: the panel as the crew finds it.
fn reset(mut panel: ResMut<TomcatPanel>, mut status: ResMut<TomcatStatus>) {
    *panel = TomcatPanel::default();
    *status = TomcatStatus::default();
}

/// The wing sweep keys take the wing out of AUTO: the sweep mode switch shows MAN.
fn sweep_switch(
    settings: Res<crate::settings::Settings>,
    aircraft: Query<&crate::flight::Aircraft, With<crate::flight::Player>>,
    mut panel: ResMut<TomcatPanel>,
) {
    if settings.aircraft != crate::aircraft::AircraftKind::F14 {
        return;
    }
    if let Ok(a) = aircraft.single()
        && a.sweep_command.is_some()
        && !panel.sweep_manual
    {
        panel.sweep_manual = true;
    }
}

/// What the rest of the aircraft tells the panel's lights (filled by the avionics; all dark
/// until they do): the stations with a weapon ready (bit `n` the station `STATIONS[n]`).
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct TomcatStatus {
    pub stations: u8,
}

/// The hooked aircraft in the panel's twelve bits (`TomcatPanel::hook_target`): 0 none, a flight
/// unit's index + 1, another pilot's id + 1 over 0x800.
pub fn hook_code(target: Option<crate::mfd::ContactRef>) -> u16 {
    match target {
        Some(crate::mfd::ContactRef::Unit(i)) if i < 0x7FF => i as u16 + 1,
        Some(crate::mfd::ContactRef::Pilot(k)) => 0x800 | (k as u16 + 1),
        _ => 0,
    }
}

/// The aircraft of a `hook_code`.
pub fn hook_ref(code: u16) -> Option<crate::mfd::ContactRef> {
    match code {
        0 => None,
        c if c & 0x800 != 0 => Some(crate::mfd::ContactRef::Pilot(((c & 0x7FF) - 1) as u8)),
        c => Some(crate::mfd::ContactRef::Unit(c as usize - 1)),
    }
}

/// Presses an F-14 control (scripts: `press=Tomcat(Wcs(TwsAuto))`), in whichever cockpit it is.
#[derive(Message, Debug, Clone, Copy)]
pub struct PressKey(pub Key);

/// Test aid `SM89_F14_PNG=<dir>`: a display's canvas written as `<name>.png` in its own
/// colours, about once a second.
pub fn dump_png(c: &Canvas, colours: &[u32; 256], name: &'static str) {
    use std::sync::Mutex;
    static LAST: Mutex<Vec<(&'static str, std::time::Instant)>> = Mutex::new(Vec::new());
    let Ok(dir) = std::env::var("SM89_F14_PNG") else { return };
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

// --- The pilot's controls.

/// The DISPLAYS panel's steering MODE buttons: what the HUD and the VDI steer to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SteerMode {
    TakeOff,
    #[default]
    Cruise,
    AirToAir,
    AirToGround,
    Landing,
}

/// The DISPLAYS panel's STEER CMD buttons: the steering's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SteerCmd {
    Tacan,
    #[default]
    Dest,
    Awl,
    Vec,
    Man,
}

/// The all-weather landing steering a display shows: the ILS's or the carrier's (ACL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Awl {
    #[default]
    Ils,
    Acl,
}

/// The VDI's picture: the flight picture or the TV (LANTIRN) video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VdiMode {
    #[default]
    Norm,
    Tv,
}

/// The HSD's picture: the navigation display, the TID's repeat, the ECM threats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum HsdMode {
    #[default]
    Nav,
    Tid,
    Ecm,
}

/// The ACM panel's master arm switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MasterArm {
    On,
    #[default]
    Off,
    Training,
}

// --- The radar intercept officer's controls.

/// The AWG-9's modes (the WCS MODE buttons).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WcsMode {
    PdSearch,
    #[default]
    Rws,
    TwsManual,
    TwsAuto,
    PdStt,
    PulseSearch,
    PulseStt,
}

/// The DDD's range buttons (nautical miles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DddRange {
    Nm5,
    Nm10,
    Nm20,
    Nm50,
    #[default]
    Nm100,
    Nm200,
}

impl DddRange {
    pub fn miles(self) -> u16 {
        match self {
            DddRange::Nm5 => 5,
            DddRange::Nm10 => 10,
            DddRange::Nm20 => 20,
            DddRange::Nm50 => 50,
            DddRange::Nm100 => 100,
            DddRange::Nm200 => 200,
        }
    }
}

/// The DDD's display mode buttons: the pulse radar's raw video, the pulse doppler's targets,
/// the antenna's track, the radar's range-rate display, the jamming's angle track, the IR
/// sensor's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DddMode {
    PulseVideo,
    #[default]
    Pd,
    AntTrk,
    Rdrot,
    Jat,
    Irot,
}

/// The target's expected aspect (the ASPECT switch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Aspect {
    #[default]
    Nose,
    Beam,
    Tail,
}

/// The radar panel's knobs that only adjust the picture (`KNOB_STEPS` positions each).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RioKnob {
    PdClear,
    PdClutter,
    JamJet,
    AcmThreshold,
    PulseGain,
    IrGain,
    Bright,
    Erase,
    Volume,
    MslAudio,
}

/// The counter-countermeasures buttons (CCM MODES), each in or out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ccm {
    Spl,
    AltOff,
    Vgs,
}

/// The TID's picture: ground or aircraft stabilised, the attack picture, the TV video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TidMode {
    GroundStab,
    #[default]
    AircraftStab,
    Attack,
    Tv,
}

/// The TID's range (nautical miles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TidRange {
    Nm25,
    Nm50,
    #[default]
    Nm100,
    Nm200,
    Nm400,
}

impl TidRange {
    pub fn miles(self) -> u16 {
        match self {
            TidRange::Nm25 => 25,
            TidRange::Nm50 => 50,
            TidRange::Nm100 => 100,
            TidRange::Nm200 => 200,
            TidRange::Nm400 => 400,
        }
    }
}

/// What the TID's data readout shows (the TID READOUT knob).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TidReadout {
    #[default]
    TgtData,
    OwnAircraft,
    Wind,
    Waypoint,
}

/// The TID's symbol buttons round its screen, each in or out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TidOption {
    Clsn,
    TidAuto,
    NonAttack,
    JamStrobe,
    DataLink,
    SymElem,
    AltNum,
    VelVector,
}

/// The armament panel's missile option knob: normal, or the radar missiles' launch modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MissileOption {
    #[default]
    Norm,
    PdStt,
    ActiveStt,
    PulseStt,
}

/// A three-position power switch (the LANTIRN's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Power {
    On,
    Standby,
    #[default]
    Off,
}

/// The LANTIRN pod's mode buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LtsMode {
    #[default]
    AirToGround,
    AirToAir,
}

/// The ALQ-126 deception jammer's mode knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Alq126 {
    Off,
    #[default]
    Standby,
    Receive,
    Repeat,
}

pub const STEER_MODES: [SteerMode; 5] = [
    SteerMode::TakeOff,
    SteerMode::Cruise,
    SteerMode::AirToAir,
    SteerMode::AirToGround,
    SteerMode::Landing,
];
pub const STEER_CMDS: [SteerCmd; 5] =
    [SteerCmd::Tacan, SteerCmd::Dest, SteerCmd::Awl, SteerCmd::Vec, SteerCmd::Man];
pub const WCS_MODES: [WcsMode; 7] = [
    WcsMode::PdSearch,
    WcsMode::Rws,
    WcsMode::TwsManual,
    WcsMode::TwsAuto,
    WcsMode::PdStt,
    WcsMode::PulseSearch,
    WcsMode::PulseStt,
];
pub const DDD_RANGES: [DddRange; 6] = [
    DddRange::Nm5,
    DddRange::Nm10,
    DddRange::Nm20,
    DddRange::Nm50,
    DddRange::Nm100,
    DddRange::Nm200,
];
pub const DDD_MODES: [DddMode; 6] = [
    DddMode::PulseVideo,
    DddMode::Pd,
    DddMode::AntTrk,
    DddMode::Rdrot,
    DddMode::Jat,
    DddMode::Irot,
];
pub const RIO_KNOBS: [RioKnob; 10] = [
    RioKnob::PdClear,
    RioKnob::PdClutter,
    RioKnob::JamJet,
    RioKnob::AcmThreshold,
    RioKnob::PulseGain,
    RioKnob::IrGain,
    RioKnob::Bright,
    RioKnob::Erase,
    RioKnob::Volume,
    RioKnob::MslAudio,
];
pub const CCMS: [Ccm; 3] = [Ccm::Spl, Ccm::AltOff, Ccm::Vgs];
pub const TID_OPTIONS: [TidOption; 8] = [
    TidOption::Clsn,
    TidOption::TidAuto,
    TidOption::NonAttack,
    TidOption::JamStrobe,
    TidOption::DataLink,
    TidOption::SymElem,
    TidOption::AltNum,
    TidOption::VelVector,
];
pub const LTS_MODES: [LtsMode; 2] = [LtsMode::AirToGround, LtsMode::AirToAir];
const AWLS: [Awl; 2] = [Awl::Ils, Awl::Acl];
// A switch's positions from its top (the first up), a knob's from fully anticlockwise.
const VDI_MODES: [VdiMode; 2] = [VdiMode::Tv, VdiMode::Norm];
const HSD_MODES: [HsdMode; 3] = [HsdMode::Nav, HsdMode::Tid, HsdMode::Ecm];
const MASTER_ARMS: [MasterArm; 3] = [MasterArm::On, MasterArm::Off, MasterArm::Training];
const ASPECTS: [Aspect; 3] = [Aspect::Nose, Aspect::Beam, Aspect::Tail];
const TID_MODES: [TidMode; 4] =
    [TidMode::GroundStab, TidMode::AircraftStab, TidMode::Attack, TidMode::Tv];
pub const TID_RANGES: [TidRange; 5] =
    [TidRange::Nm25, TidRange::Nm50, TidRange::Nm100, TidRange::Nm200, TidRange::Nm400];
const TID_READOUTS: [TidReadout; 4] =
    [TidReadout::TgtData, TidReadout::OwnAircraft, TidReadout::Wind, TidReadout::Waypoint];
const MISSILE_OPTIONS: [MissileOption; 4] =
    [MissileOption::Norm, MissileOption::PdStt, MissileOption::ActiveStt, MissileOption::PulseStt];
const POWERS: [Power; 3] = [Power::On, Power::Standby, Power::Off];
const ALQ126S: [Alq126; 4] = [Alq126::Off, Alq126::Standby, Alq126::Receive, Alq126::Repeat];

/// Positions of the radar panel's picture knobs and the pitch ladder's brightness knob.
pub const KNOB_STEPS: u8 = 5;
pub const LADDER_STEPS: u8 = 4;
/// The XMTR CHAN and MSL CHAN thumbwheels' channels (1 to `CHANNELS`).
pub const CHANNELS: u8 = 9;
/// The pilot's armament panel's station lights, left to right (`TomcatStatus::stations`).
pub const STATIONS: [&str; 8] = ["1A", "1B", "3", "4", "5", "6", "8B", "8A"];

/// Every control of the F-14's two cockpits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// The pilot's DISPLAYS panel: the steering MODE and STEER CMD buttons, the HUD's, VDI's
    /// and HSD's switches, the pitch ladder's brightness.
    Steer(SteerMode),
    SteerCmd(SteerCmd),
    HudDeclutter,
    HudAwl,
    VdiMode,
    VdiAwl,
    HsdMode,
    HsdEcmOverride,
    PitchLadder,
    /// The pilot's ACM panel: master arm, the ACM jettison button's guard, the lit GUN RATE,
    /// SW COOL, MSL PREP and MSL MODE buttons.
    MasterArm,
    AcmCover,
    GunRate,
    SwCool,
    MslPrep,
    MslMode,
    /// The wing sweep mode switch (AUTO / MAN; the sweep handle is `Effect::Action`'s keys),
    /// the hook handle.
    SweepMode,
    Hook,
    /// The RIO's radar panel: the WCS mode, DDD range and DDD mode buttons, the ASPECT and Vc
    /// switches, the picture knobs, the CCM buttons, the antenna elevation keys and the XMTR
    /// CHAN and MSL CHAN thumbwheels.
    Wcs(WcsMode),
    DddRange(DddRange),
    DddMode(DddMode),
    Aspect,
    VcScale,
    Knob(RioKnob),
    Ccm(Ccm),
    AntennaUp,
    AntennaDown,
    XmtrChan,
    MslChan,
    /// The IFF interrogator (lit while it interrogates: friendly returns get bars on the DDD).
    Iff,
    /// The RIO's TID: its mode, range and readout knobs, the symbol buttons round it, the hand
    /// controller's hook button.
    TidMode,
    TidRange,
    TidReadout,
    TidOption(TidOption),
    TidHook,
    /// The RIO's armament panel: Phoenix active off the rail, LAUNCH, the missile options.
    PhAct,
    Launch,
    MissileOption,
    /// The RIO's LANTIRN panel.
    LtsPower,
    LtsVideo,
    LaserArm,
    Polarity,
    Fov,
    LtsMode(LtsMode),
    LtsOper,
    /// The RIO's ECM panel: the ALQ-126's mode.
    Alq126,
}

impl Key {
    /// Every key.
    pub fn all() -> Vec<Key> {
        let mut v = Vec::new();
        v.extend(STEER_MODES.map(Key::Steer));
        v.extend(STEER_CMDS.map(Key::SteerCmd));
        v.extend([
            Key::HudDeclutter,
            Key::HudAwl,
            Key::VdiMode,
            Key::VdiAwl,
            Key::HsdMode,
            Key::HsdEcmOverride,
            Key::PitchLadder,
            Key::MasterArm,
            Key::AcmCover,
            Key::GunRate,
            Key::SwCool,
            Key::MslPrep,
            Key::MslMode,
            Key::SweepMode,
            Key::Hook,
        ]);
        v.extend(WCS_MODES.map(Key::Wcs));
        v.extend(DDD_RANGES.map(Key::DddRange));
        v.extend(DDD_MODES.map(Key::DddMode));
        v.extend([Key::Aspect, Key::VcScale]);
        v.extend(RIO_KNOBS.map(Key::Knob));
        v.extend(CCMS.map(Key::Ccm));
        v.extend([
            Key::AntennaUp,
            Key::AntennaDown,
            Key::XmtrChan,
            Key::MslChan,
            Key::Iff,
            Key::TidMode,
            Key::TidRange,
            Key::TidReadout,
        ]);
        v.extend(TID_OPTIONS.map(Key::TidOption));
        v.extend([
            Key::TidHook,
            Key::PhAct,
            Key::Launch,
            Key::MissileOption,
            Key::LtsPower,
            Key::LtsVideo,
            Key::LaserArm,
            Key::Polarity,
            Key::Fov,
        ]);
        v.extend(LTS_MODES.map(Key::LtsMode));
        v.extend([Key::LtsOper, Key::Alq126]);
        v
    }

    #[cfg_attr(not(test), allow(dead_code))]
    /// The pilot's controls (the others are the RIO's).
    pub fn pilot(self) -> bool {
        matches!(
            self,
            Key::Steer(_)
                | Key::SteerCmd(_)
                | Key::HudDeclutter
                | Key::HudAwl
                | Key::VdiMode
                | Key::VdiAwl
                | Key::HsdMode
                | Key::HsdEcmOverride
                | Key::PitchLadder
                | Key::MasterArm
                | Key::AcmCover
                | Key::GunRate
                | Key::SwCool
                | Key::MslPrep
                | Key::MslMode
                | Key::SweepMode
                | Key::Hook
        )
    }

    /// A key by its `Debug` text (`Wcs(TwsAuto)`, `Knob(Bright)`), as the cockpit models and
    /// scripts write it.
    pub fn parse(s: &str) -> Option<Key> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        Key::all().into_iter().find(|k| format!("{k:?}") == s)
    }

    /// How the control is modelled (the cockpit model's `kind`): `twist` a square pushbutton
    /// that latches in turned 45 degrees (the A-6's), `lamp` a square lit pushbutton that sinks
    /// while it is in and lights its legend, `switch` a toggle switch, `knob` a rotary knob,
    /// `cap` a key or thumbwheel.
    pub fn kind(self) -> &'static str {
        match self {
            Key::Steer(_) | Key::SteerCmd(_) => "twist",
            Key::GunRate
            | Key::SwCool
            | Key::MslPrep
            | Key::MslMode
            | Key::Wcs(_)
            | Key::DddRange(_)
            | Key::DddMode(_)
            | Key::Ccm(_)
            | Key::TidOption(_)
            | Key::TidHook
            | Key::Launch
            | Key::Iff
            | Key::LtsMode(_)
            | Key::LtsOper => "lamp",
            Key::HudDeclutter
            | Key::HudAwl
            | Key::VdiMode
            | Key::VdiAwl
            | Key::HsdMode
            | Key::HsdEcmOverride
            | Key::MasterArm
            | Key::AcmCover
            | Key::SweepMode
            | Key::Hook
            | Key::Aspect
            | Key::VcScale
            | Key::PhAct
            | Key::LtsPower
            | Key::LtsVideo
            | Key::LaserArm
            | Key::Polarity
            | Key::Fov => "switch",
            Key::PitchLadder
            | Key::Knob(_)
            | Key::TidMode
            | Key::TidRange
            | Key::TidReadout
            | Key::MissileOption
            | Key::Alq126 => "knob",
            Key::AntennaUp | Key::AntennaDown | Key::XmtrChan | Key::MslChan => "cap",
        }
    }

    /// The control's legend as the panel has it (lines separated by '\n'): a lit button's on
    /// its cap, the others' printed by it.
    pub fn label(self) -> &'static str {
        match self {
            Key::Steer(SteerMode::TakeOff) => "T.O.",
            Key::Steer(SteerMode::Cruise) => "CRUISE",
            Key::Steer(SteerMode::AirToAir) => "A/A",
            Key::Steer(SteerMode::AirToGround) => "A/G",
            Key::Steer(SteerMode::Landing) => "LDG",
            Key::SteerCmd(SteerCmd::Tacan) => "TACAN",
            Key::SteerCmd(SteerCmd::Dest) => "DEST",
            Key::SteerCmd(SteerCmd::Awl) => "AWL\nPCD",
            Key::SteerCmd(SteerCmd::Vec) => "VEC",
            Key::SteerCmd(SteerCmd::Man) => "MAN",
            Key::HudDeclutter => "DECLUTTER",
            Key::HudAwl => "HUD AWL",
            Key::VdiMode => "VDI MODE",
            Key::VdiAwl => "VDI AWL",
            Key::HsdMode => "HSD MODE",
            Key::HsdEcmOverride => "ECM\nORIDE",
            Key::PitchLadder => "PITCH\nLAD",
            Key::MasterArm => "MASTER\nARM",
            Key::AcmCover => "ACM\nJETT",
            Key::GunRate => "HIGH",
            Key::SwCool => "ON",
            Key::MslPrep => "ON",
            Key::MslMode => "BRSIT",
            Key::SweepMode => "SWEEP",
            Key::Hook => "HOOK",
            Key::Wcs(WcsMode::PdSearch) => "PD\nSRCH",
            Key::Wcs(WcsMode::Rws) => "RWS",
            Key::Wcs(WcsMode::TwsManual) => "TWS\nMAN",
            Key::Wcs(WcsMode::TwsAuto) => "TWS\nAUTO",
            Key::Wcs(WcsMode::PdStt) => "PD\nSTT",
            Key::Wcs(WcsMode::PulseSearch) => "PULSE\nSRCH",
            Key::Wcs(WcsMode::PulseStt) => "P\nSTT",
            Key::DddRange(DddRange::Nm5) => "5",
            Key::DddRange(DddRange::Nm10) => "10",
            Key::DddRange(DddRange::Nm20) => "20",
            Key::DddRange(DddRange::Nm50) => "50",
            Key::DddRange(DddRange::Nm100) => "100",
            Key::DddRange(DddRange::Nm200) => "200",
            Key::DddMode(DddMode::PulseVideo) => "PULSE\nVIDEO",
            Key::DddMode(DddMode::Pd) => "PD",
            Key::DddMode(DddMode::AntTrk) => "ANT\nTRK",
            Key::DddMode(DddMode::Rdrot) => "RDROT",
            Key::DddMode(DddMode::Jat) => "JAT",
            Key::DddMode(DddMode::Irot) => "IROT",
            Key::Aspect => "ASPECT",
            Key::VcScale => "Vc",
            Key::Knob(RioKnob::PdClear) => "CLEAR",
            Key::Knob(RioKnob::PdClutter) => "CLUTTER",
            Key::Knob(RioKnob::JamJet) => "JAM/JET",
            Key::Knob(RioKnob::AcmThreshold) => "ACM\nTHRLD",
            Key::Knob(RioKnob::PulseGain) => "PULSE\nGAIN",
            Key::Knob(RioKnob::IrGain) => "IR GAIN",
            Key::Knob(RioKnob::Bright) => "BRIGHT",
            Key::Knob(RioKnob::Erase) => "ERASE",
            Key::Knob(RioKnob::Volume) => "VOL",
            Key::Knob(RioKnob::MslAudio) => "MSL\nAUDIO",
            Key::Ccm(Ccm::Spl) => "SPL",
            Key::Ccm(Ccm::AltOff) => "ALT\nOFF",
            Key::Ccm(Ccm::Vgs) => "VGS",
            Key::AntennaUp => "EL UP",
            Key::AntennaDown => "EL DN",
            Key::XmtrChan => "XMTR\nCHAN",
            Key::MslChan => "MSL\nCHAN",
            Key::Iff => "IFF",
            Key::TidMode => "TID MODE",
            Key::TidRange => "TID RANGE",
            Key::TidReadout => "READOUT",
            Key::TidOption(TidOption::Clsn) => "CLSN",
            Key::TidOption(TidOption::TidAuto) => "TID\nAUTO",
            Key::TidOption(TidOption::NonAttack) => "NON\nATTK",
            Key::TidOption(TidOption::JamStrobe) => "JAM\nSTROBE",
            Key::TidOption(TidOption::DataLink) => "DATA\nLINK",
            Key::TidOption(TidOption::SymElem) => "SYM\nELEM",
            Key::TidOption(TidOption::AltNum) => "ALT\nNUM",
            Key::TidOption(TidOption::VelVector) => "VEL\nVECTOR",
            Key::TidHook => "HOOK",
            Key::PhAct => "PH ACT",
            Key::Launch => "LAUNCH",
            Key::MissileOption => "MSL\nOPTIONS",
            Key::LtsPower => "POWER",
            Key::LtsVideo => "VIDEO",
            Key::LaserArm => "LASER",
            Key::Polarity => "POLARITY",
            Key::Fov => "FOV",
            Key::LtsMode(LtsMode::AirToGround) => "A/G",
            Key::LtsMode(LtsMode::AirToAir) => "A/A",
            Key::LtsOper => "OPER",
            Key::Alq126 => "ALQ-126",
        }
    }

    /// The tooltip's text (a message of `assets/i18n/*/tooltips.ftl`): what the control does
    /// in the game, in a few words.
    pub fn tip(self) -> &'static str {
        match self {
            Key::Steer(_) => "tip-f14-steer-mode",
            Key::SteerCmd(_) => "tip-f14-steer-cmd",
            Key::HudDeclutter => "tip-f14-hud-declutter",
            Key::HudAwl | Key::VdiAwl => "tip-f14-awl",
            Key::VdiMode => "tip-f14-vdi-mode",
            Key::HsdMode => "tip-f14-hsd-mode",
            Key::HsdEcmOverride => "tip-f14-hsd-ecm",
            Key::PitchLadder => "tip-f14-pitch-ladder",
            Key::MasterArm => "tip-f14-master-arm",
            Key::AcmCover => "tip-f14-acm-cover",
            Key::GunRate => "tip-f14-gun-rate",
            Key::SwCool => "tip-f14-sw-cool",
            Key::MslPrep => "tip-f14-msl-prep",
            Key::MslMode => "tip-f14-msl-mode",
            Key::SweepMode => "tip-f14-sweep-mode",
            Key::Wcs(_) => "tip-f14-wcs-mode",
            Key::DddRange(_) => "tip-f14-ddd-range",
            Key::DddMode(_) => "tip-f14-ddd-mode",
            Key::Aspect => "tip-f14-aspect",
            Key::VcScale => "tip-f14-vc-scale",
            Key::Knob(RioKnob::Volume | RioKnob::MslAudio) => "tip-f14-volume",
            Key::Knob(_) => "tip-f14-radar-knob",
            Key::Ccm(_) => "tip-f14-ccm",
            Key::AntennaUp | Key::AntennaDown => "tip-f14-antenna",
            Key::XmtrChan | Key::MslChan => "tip-f14-channel",
            Key::Iff => "tip-f14-iff",
            Key::TidMode => "tip-f14-tid-mode",
            Key::TidRange => "tip-f14-tid-range",
            Key::TidReadout => "tip-f14-tid-readout",
            Key::TidOption(_) => "tip-f14-tid-option",
            Key::TidHook => "tip-f14-tid-hook",
            Key::PhAct => "tip-f14-ph-act",
            Key::Launch => "tip-f14-launch",
            Key::MissileOption => "tip-f14-missile-option",
            Key::LtsPower => "tip-f14-lantirn-power",
            Key::LtsVideo => "tip-f14-lantirn-video",
            Key::LaserArm => "tip-f14-laser-arm",
            Key::Polarity => "tip-f14-polarity",
            Key::Fov => "tip-f14-fov",
            Key::LtsMode(_) => "tip-f14-lantirn-mode",
            Key::LtsOper => "tip-f14-lantirn-oper",
            Key::Alq126 => "tip-f14-alq126",
            // It moves, and that is all.
            Key::Hook => "tip-a6-no-effect",
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

/// The state the F-14's controls latch in, for both cockpits (one panel a crew: the other crew
/// member's copy is to follow it, `pack`).
#[derive(Resource, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TomcatPanel {
    /// The pilot's DISPLAYS panel.
    pub steer: SteerMode,
    pub steer_cmd: SteerCmd,
    pub hud_declutter: bool,
    pub hud_awl: Awl,
    pub vdi_mode: VdiMode,
    pub vdi_awl: Awl,
    pub hsd_mode: HsdMode,
    pub hsd_ecm_override: bool,
    /// The pitch ladder's brightness (0 dark .. `LADDER_STEPS` - 1).
    pub pitch_ladder: u8,
    /// The ACM panel.
    pub master_arm: MasterArm,
    pub acm_cover: bool,
    /// GUN RATE high, SW COOL on, MSL PREP on, MSL MODE boresight.
    pub gun_high: bool,
    pub sw_cool: bool,
    pub msl_prep: bool,
    pub msl_boresight: bool,
    /// The wing sweep mode switch at MAN; the hook handle down.
    pub sweep_manual: bool,
    pub hook_down: bool,
    /// The RIO's radar panel.
    pub wcs: WcsMode,
    pub ddd_range: DddRange,
    pub ddd_mode: DddMode,
    pub aspect: Aspect,
    pub vc_2x: bool,
    /// The picture knobs' positions (0 .. `KNOB_STEPS` - 1), by `RIO_KNOBS`.
    pub knobs: [u8; 10],
    /// The CCM buttons in (bit `Ccm as u8`).
    pub ccm: u8,
    /// The thumbwheels' channels (1 ..= `CHANNELS`).
    pub xmtr_chan: u8,
    pub msl_chan: u8,
    /// The IFF interrogates.
    pub iff: bool,
    /// The TID.
    pub tid_mode: TidMode,
    pub tid_range: TidRange,
    pub tid_readout: TidReadout,
    /// The TID's symbol buttons in (bit `TidOption as u8`).
    pub tid_options: u8,
    /// The hook button's presses so far (the avionics take each new one; this seat's own: not
    /// shared).
    pub hooks: u8,
    /// The aircraft hooked (`hook_code`; 0 none): the crew's, so both seats' radars hook the same.
    pub hook_target: u16,
    /// The armament panel.
    pub ph_active: bool,
    pub missile_option: MissileOption,
    /// LAUNCH presses so far.
    pub launches: u8,
    /// The LANTIRN panel.
    pub lts_power: Power,
    pub lts_lantirn: bool,
    pub laser_armed: bool,
    pub black_hot: bool,
    pub narrow: bool,
    pub lts_mode: LtsMode,
    pub lts_oper: bool,
    /// The ECM panel.
    pub alq126: Alq126,
}

impl Default for TomcatPanel {
    fn default() -> Self {
        Self {
            steer: SteerMode::default(),
            steer_cmd: SteerCmd::default(),
            hud_declutter: false,
            hud_awl: Awl::default(),
            vdi_mode: VdiMode::default(),
            vdi_awl: Awl::default(),
            hsd_mode: HsdMode::default(),
            hsd_ecm_override: false,
            pitch_ladder: LADDER_STEPS - 1,
            master_arm: MasterArm::default(),
            acm_cover: false,
            gun_high: true,
            sw_cool: false,
            msl_prep: false,
            msl_boresight: false,
            sweep_manual: false,
            hook_down: false,
            wcs: WcsMode::default(),
            ddd_range: DddRange::default(),
            ddd_mode: DddMode::default(),
            aspect: Aspect::default(),
            vc_2x: false,
            knobs: [KNOB_STEPS / 2; 10],
            ccm: 0,
            xmtr_chan: 1,
            msl_chan: 1,
            iff: false,
            tid_mode: TidMode::default(),
            tid_range: TidRange::default(),
            tid_readout: TidReadout::default(),
            // The aircraft's velocity vector and the targets' altitudes shown.
            tid_options: 1 << TidOption::AltNum as u8 | 1 << TidOption::VelVector as u8,
            hooks: 0,
            hook_target: 0,
            ph_active: false,
            missile_option: MissileOption::default(),
            launches: 0,
            lts_power: Power::Off,
            lts_lantirn: false,
            laser_armed: false,
            black_hot: false,
            narrow: false,
            lts_mode: LtsMode::default(),
            lts_oper: false,
            alq126: Alq126::default(),
        }
    }
}

/// The next of `all` after `v`, round again.
fn next<T: Copy + PartialEq>(all: &[T], v: T) -> T {
    let i = all.iter().position(|a| *a == v).unwrap_or(0);
    all[(i + 1) % all.len()]
}

#[cfg_attr(not(test), allow(dead_code))]
/// The one of `all` at index `i` (the last for a larger one).
fn pick<T: Copy>(all: &[T], i: u8) -> T {
    all[(i as usize).min(all.len() - 1)]
}

fn index<T: PartialEq>(all: &[T], v: T) -> u8 {
    all.iter().position(|a| *a == v).unwrap_or(0) as u8
}

impl TomcatPanel {
    /// A control pressed: a steering button or a radio group's lit button goes in and the one
    /// of its group that was in comes out, a lit toggle button goes in or out, a switch or knob
    /// goes to its next position, a key does what it does. What it does to the rest of the
    /// aircraft: the sweep mode switch to AUTO, the antenna keys, LAUNCH.
    pub fn press(&mut self, key: Key) -> Option<PanelOut> {
        match key {
            Key::Steer(m) => self.steer = m,
            Key::SteerCmd(c) => self.steer_cmd = c,
            Key::HudDeclutter => self.hud_declutter = !self.hud_declutter,
            Key::HudAwl => self.hud_awl = next(&AWLS, self.hud_awl),
            Key::VdiMode => self.vdi_mode = next(&VDI_MODES, self.vdi_mode),
            Key::VdiAwl => self.vdi_awl = next(&AWLS, self.vdi_awl),
            Key::HsdMode => self.hsd_mode = next(&HSD_MODES, self.hsd_mode),
            Key::HsdEcmOverride => self.hsd_ecm_override = !self.hsd_ecm_override,
            Key::PitchLadder => self.pitch_ladder = (self.pitch_ladder + 1) % LADDER_STEPS,
            Key::MasterArm => self.master_arm = next(&MASTER_ARMS, self.master_arm),
            Key::AcmCover => self.acm_cover = !self.acm_cover,
            Key::GunRate => self.gun_high = !self.gun_high,
            Key::SwCool => self.sw_cool = !self.sw_cool,
            Key::MslPrep => self.msl_prep = !self.msl_prep,
            Key::MslMode => self.msl_boresight = !self.msl_boresight,
            Key::SweepMode => {
                self.sweep_manual = !self.sweep_manual;
                if !self.sweep_manual {
                    return Some(PanelOut::Action(Action::WingSweepAuto));
                }
            }
            Key::Hook => self.hook_down = !self.hook_down,
            Key::Wcs(m) => self.wcs = m,
            Key::DddRange(r) => self.ddd_range = r,
            Key::DddMode(m) => self.ddd_mode = m,
            Key::Aspect => self.aspect = next(&ASPECTS, self.aspect),
            Key::VcScale => self.vc_2x = !self.vc_2x,
            Key::Knob(k) => {
                let v = &mut self.knobs[index(&RIO_KNOBS, k) as usize];
                *v = (*v + 1) % KNOB_STEPS;
            }
            Key::Ccm(c) => self.ccm ^= 1 << c as u8,
            Key::AntennaUp => return Some(PanelOut::Mfd(crate::mfd::MfdCmd::FcrElevation(1))),
            Key::AntennaDown => return Some(PanelOut::Mfd(crate::mfd::MfdCmd::FcrElevation(-1))),
            Key::XmtrChan => self.xmtr_chan = self.xmtr_chan % CHANNELS + 1,
            Key::MslChan => self.msl_chan = self.msl_chan % CHANNELS + 1,
            Key::Iff => self.iff = !self.iff,
            Key::TidMode => self.tid_mode = next(&TID_MODES, self.tid_mode),
            Key::TidRange => self.tid_range = next(&TID_RANGES, self.tid_range),
            Key::TidReadout => self.tid_readout = next(&TID_READOUTS, self.tid_readout),
            Key::TidOption(o) => self.tid_options ^= 1 << o as u8,
            Key::TidHook => self.hooks = self.hooks.wrapping_add(1),
            Key::PhAct => self.ph_active = !self.ph_active,
            Key::Launch => {
                self.launches = self.launches.wrapping_add(1);
                return Some(PanelOut::Action(Action::FireWeapon));
            }
            Key::MissileOption => self.missile_option = next(&MISSILE_OPTIONS, self.missile_option),
            Key::LtsPower => self.lts_power = next(&POWERS, self.lts_power),
            Key::LtsVideo => self.lts_lantirn = !self.lts_lantirn,
            Key::LaserArm => self.laser_armed = !self.laser_armed,
            Key::Polarity => self.black_hot = !self.black_hot,
            Key::Fov => self.narrow = !self.narrow,
            Key::LtsMode(m) => self.lts_mode = m,
            Key::LtsOper => self.lts_oper = !self.lts_oper,
            Key::Alq126 => self.alq126 = next(&ALQ126S, self.alq126),
        }
        None
    }

    /// The control stays in: a steering button turned 45 degrees, a lit button sunk with its
    /// legend lit.
    pub fn latched(&self, key: Key) -> bool {
        match key {
            Key::Steer(m) => self.steer == m,
            Key::SteerCmd(c) => self.steer_cmd == c,
            Key::GunRate => self.gun_high,
            Key::SwCool => self.sw_cool,
            Key::MslPrep => self.msl_prep,
            Key::MslMode => self.msl_boresight,
            Key::Wcs(m) => self.wcs == m,
            Key::DddRange(r) => self.ddd_range == r,
            Key::DddMode(m) => self.ddd_mode == m,
            Key::Ccm(c) => self.ccm_in(c),
            Key::Iff => self.iff,
            Key::TidOption(o) => self.tid_option(o),
            Key::LtsMode(m) => self.lts_mode == m,
            Key::LtsOper => self.lts_oper,
            _ => false,
        }
    }

    /// A CCM button is in.
    pub fn ccm_in(&self, c: Ccm) -> bool {
        self.ccm & (1 << c as u8) != 0
    }

    /// A TID symbol button is in.
    pub fn tid_option(&self, o: TidOption) -> bool {
        self.tid_options & (1 << o as u8) != 0
    }

    /// A picture knob's position (0 .. `KNOB_STEPS` - 1).
    pub fn knob(&self, k: RioKnob) -> u8 {
        self.knobs[index(&RIO_KNOBS, k) as usize]
    }

    /// A switch's or knob's position and how many it has: a switch's first is up, a knob's is
    /// fully anticlockwise. None for the buttons and keys.
    pub fn position(&self, key: Key) -> Option<(u8, u8)> {
        // A two-position switch: its first position (up) when `up`.
        let two = |up: bool| Some((!up as u8, 2));
        Some(match key {
            Key::HudDeclutter => return two(self.hud_declutter),
            Key::HudAwl => (index(&AWLS, self.hud_awl), 2),
            Key::VdiMode => (index(&VDI_MODES, self.vdi_mode), 2),
            Key::VdiAwl => (index(&AWLS, self.vdi_awl), 2),
            Key::HsdMode => (index(&HSD_MODES, self.hsd_mode), 3),
            Key::HsdEcmOverride => return two(self.hsd_ecm_override),
            Key::PitchLadder => (self.pitch_ladder.min(LADDER_STEPS - 1), LADDER_STEPS),
            Key::MasterArm => (index(&MASTER_ARMS, self.master_arm), 3),
            Key::AcmCover => return two(self.acm_cover),
            // AUTO up, MAN down; the hook handle up, down.
            Key::SweepMode => (self.sweep_manual as u8, 2),
            Key::Hook => (self.hook_down as u8, 2),
            Key::Aspect => (index(&ASPECTS, self.aspect), 3),
            Key::VcScale => return two(self.vc_2x),
            Key::Knob(k) => (self.knob(k).min(KNOB_STEPS - 1), KNOB_STEPS),
            Key::TidMode => (index(&TID_MODES, self.tid_mode), 4),
            Key::TidRange => (index(&TID_RANGES, self.tid_range), 5),
            Key::TidReadout => (index(&TID_READOUTS, self.tid_readout), 4),
            Key::PhAct => return two(self.ph_active),
            Key::MissileOption => (index(&MISSILE_OPTIONS, self.missile_option), 4),
            Key::LtsPower => (index(&POWERS, self.lts_power), 3),
            Key::LtsVideo => return two(self.lts_lantirn),
            Key::LaserArm => return two(self.laser_armed),
            Key::Polarity => return two(self.black_hot),
            Key::Fov => return two(self.narrow),
            Key::Alq126 => (index(&ALQ126S, self.alq126), 4),
            _ => return None,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    /// The panel in 16 bytes, to share it with the other seat (`stealth_sim::crew::SharedPanel`):
    /// every control's state but the press counters' high bits.
    pub fn pack(&self) -> [u8; 16] {
        let mut b = Bits::default();
        b.put(index(&STEER_MODES, self.steer), 3);
        b.put(index(&STEER_CMDS, self.steer_cmd), 3);
        b.flag(self.hud_declutter);
        b.put(index(&AWLS, self.hud_awl), 1);
        b.put(index(&VDI_MODES, self.vdi_mode), 1);
        b.put(index(&AWLS, self.vdi_awl), 1);
        b.put(index(&HSD_MODES, self.hsd_mode), 2);
        b.flag(self.hsd_ecm_override);
        b.put(self.pitch_ladder, 2);
        b.put(index(&MASTER_ARMS, self.master_arm), 2);
        for f in [
            self.acm_cover,
            self.gun_high,
            self.sw_cool,
            self.msl_prep,
            self.msl_boresight,
            self.sweep_manual,
            self.hook_down,
        ] {
            b.flag(f);
        }
        b.put(index(&WCS_MODES, self.wcs), 3);
        b.put(index(&DDD_RANGES, self.ddd_range), 3);
        b.put(index(&DDD_MODES, self.ddd_mode), 3);
        b.put(index(&ASPECTS, self.aspect), 2);
        b.flag(self.vc_2x);
        for k in self.knobs {
            b.put(k, 3);
        }
        b.put(self.ccm, 3);
        b.put(self.xmtr_chan, 4);
        b.put(self.msl_chan, 4);
        b.put(index(&TID_MODES, self.tid_mode), 2);
        b.put(index(&TID_RANGES, self.tid_range), 3);
        b.put(index(&TID_READOUTS, self.tid_readout), 2);
        b.put(self.tid_options, 8);
        b.put(self.hook_target as u8, 8);
        b.put((self.hook_target >> 8) as u8 & 0xF, 4);
        b.flag(self.ph_active);
        b.put(index(&MISSILE_OPTIONS, self.missile_option), 2);
        b.put(self.launches, 4);
        b.put(index(&POWERS, self.lts_power), 2);
        for f in [self.lts_lantirn, self.laser_armed, self.black_hot, self.narrow] {
            b.flag(f);
        }
        b.put(index(&LTS_MODES, self.lts_mode), 1);
        b.flag(self.lts_oper);
        b.put(index(&ALQ126S, self.alq126), 2);
        b.flag(self.iff);
        b.buf
    }

    #[cfg_attr(not(test), allow(dead_code))]
    /// The panel as `pack` made it. The press counters keep their own high bits (only their
    /// low ones travel; a counter is only ever compared with its last value).
    pub fn unpack(&mut self, bits: &[u8; 16]) {
        let mut b = Bits { buf: *bits, at: 0 };
        self.steer = pick(&STEER_MODES, b.get(3));
        self.steer_cmd = pick(&STEER_CMDS, b.get(3));
        self.hud_declutter = b.bool();
        self.hud_awl = pick(&AWLS, b.get(1));
        self.vdi_mode = pick(&VDI_MODES, b.get(1));
        self.vdi_awl = pick(&AWLS, b.get(1));
        self.hsd_mode = pick(&HSD_MODES, b.get(2));
        self.hsd_ecm_override = b.bool();
        self.pitch_ladder = b.get(2).min(LADDER_STEPS - 1);
        self.master_arm = pick(&MASTER_ARMS, b.get(2));
        self.acm_cover = b.bool();
        self.gun_high = b.bool();
        self.sw_cool = b.bool();
        self.msl_prep = b.bool();
        self.msl_boresight = b.bool();
        self.sweep_manual = b.bool();
        self.hook_down = b.bool();
        self.wcs = pick(&WCS_MODES, b.get(3));
        self.ddd_range = pick(&DDD_RANGES, b.get(3));
        self.ddd_mode = pick(&DDD_MODES, b.get(3));
        self.aspect = pick(&ASPECTS, b.get(2));
        self.vc_2x = b.bool();
        for k in &mut self.knobs {
            *k = b.get(3).min(KNOB_STEPS - 1);
        }
        self.ccm = b.get(3);
        self.xmtr_chan = b.get(4).clamp(1, CHANNELS);
        self.msl_chan = b.get(4).clamp(1, CHANNELS);
        self.tid_mode = pick(&TID_MODES, b.get(2));
        self.tid_range = pick(&TID_RANGES, b.get(3));
        self.tid_readout = pick(&TID_READOUTS, b.get(2));
        self.tid_options = b.get(8);
        self.hook_target = b.get(8) as u16 | (b.get(4) as u16) << 8;
        self.ph_active = b.bool();
        self.missile_option = pick(&MISSILE_OPTIONS, b.get(2));
        self.launches = (self.launches & !0xf) | b.get(4);
        self.lts_power = pick(&POWERS, b.get(2));
        self.lts_lantirn = b.bool();
        self.laser_armed = b.bool();
        self.black_hot = b.bool();
        self.narrow = b.bool();
        self.lts_mode = pick(&LTS_MODES, b.get(1));
        self.lts_oper = b.bool();
        self.alq126 = pick(&ALQ126S, b.get(2));
        self.iff = b.bool();
    }
}

impl PanelKeys for TomcatPanel {
    type Key = Key;
    fn press(&mut self, key: Key) -> Option<PanelOut> {
        TomcatPanel::press(self, key)
    }
    fn latched(&self, key: Key) -> bool {
        TomcatPanel::latched(self, key)
    }
    fn position(&self, key: Key) -> Option<(u8, u8)> {
        TomcatPanel::position(self, key)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
/// Bits written one field after another into 16 bytes, least significant first.
#[derive(Default)]
struct Bits {
    buf: [u8; 16],
    at: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
impl Bits {
    fn put(&mut self, v: u8, n: usize) {
        for i in 0..n {
            assert!(self.at < 128, "the F-14 panel does not fit 16 bytes");
            if v >> i & 1 != 0 {
                self.buf[self.at / 8] |= 1 << (self.at % 8);
            }
            self.at += 1;
        }
    }

    fn flag(&mut self, f: bool) {
        self.put(f as u8, 1);
    }

    fn get(&mut self, n: usize) -> u8 {
        let mut v = 0;
        for i in 0..n {
            if self.at < 128 && self.buf[self.at / 8] >> (self.at % 8) & 1 != 0 {
                v |= 1 << i;
            }
            self.at += 1;
        }
        v
    }

    fn bool(&mut self) -> bool {
        self.get(1) != 0
    }
}

/// The lights of the F-14's panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Light {
    /// The pilot's station lights (0 the 1A, `STATIONS`): the station's weapon ready, master
    /// arm on.
    Station(u8),
    /// The RIO's repeat of the pilot's MSL PREP.
    MslPrep,
    /// The RIO's LANTIRN panel: the laser armed with the pod on.
    LaserArmed,
}

#[cfg_attr(not(test), allow(dead_code))]
pub const LIGHTS: [Light; 10] = [
    Light::Station(0),
    Light::Station(1),
    Light::Station(2),
    Light::Station(3),
    Light::Station(4),
    Light::Station(5),
    Light::Station(6),
    Light::Station(7),
    Light::MslPrep,
    Light::LaserArmed,
];

impl Light {
    pub fn label(self) -> &'static str {
        match self {
            Light::Station(n) => STATIONS[(n as usize).min(STATIONS.len() - 1)],
            Light::MslPrep => "MSL\nPREP",
            Light::LaserArmed => "LASER\nARMED",
        }
    }

    /// The tooltip's text: when it lights.
    pub fn tip(self) -> &'static str {
        match self {
            Light::Station(_) => "tip-f14-light-station",
            Light::MslPrep => "tip-f14-light-msl-prep",
            Light::LaserArmed => "tip-f14-light-laser",
        }
    }
}

/// Whether a light is lit.
pub fn lit(light: Light, panel: &TomcatPanel, status: &TomcatStatus) -> bool {
    match light {
        Light::Station(n) => {
            panel.master_arm != MasterArm::Off && status.stations & (1 << n.min(7)) != 0
        }
        Light::MslPrep => panel.msl_prep,
        Light::LaserArmed => panel.laser_armed && panel.lts_power == Power::On,
    }
}

/// The readouts of the F-14's panels: the pilot's fuel totaliser's TOTAL and BINGO counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Readout {
    FuelTotal,
    Bingo,
}

impl Readout {
    pub fn label(self) -> &'static str {
        match self {
            Readout::FuelTotal => "TOTAL",
            Readout::Bingo => "BINGO",
        }
    }

    pub fn tip(self) -> &'static str {
        match self {
            Readout::FuelTotal => "tip-f14-readout-total",
            Readout::Bingo => "tip-f14-readout-bingo",
        }
    }
}

/// What a readout shows: the fuel (lbs) and the bingo fuel, to the nearest 10 lbs.
pub fn readout(r: Readout, fuel: f32, bingo: f32) -> String {
    let lbs = |v: f32| ((v.max(0.0) / 10.0).round() as u32 * 10).min(99_990);
    match r {
        Readout::FuelTotal => format!("{:05}", lbs(fuel)),
        Readout::Bingo => format!("{:05}", lbs(bingo)),
    }
}

/// Palette colours of the panel faces: black, the legends' white, the lit buttons' dark cap,
/// their legend unlit and lit, the switches' plates, the knobs' grey.
const BLACK: u8 = 5;
const WHITE: u8 = 15;
const CAP: u8 = 38;
const LEGEND_OFF: u8 = 7;
const LEGEND_ON: u8 = 47;
const KNOB_TOP: u8 = 8;

/// A control's face in the instrument atlas (rectangle `r`, `k` texels per millimetre): the
/// steering buttons as the A-6's pushbuttons, a lit button's legend on its dark cap (lit while
/// it is in), a switch's base plate, a knob's top with its index, a key with its legend, a
/// thumbwheel's channel. True if it glows.
pub fn draw_control(
    c: &mut Canvas,
    fonts: &Fonts,
    r: [i32; 4],
    key: Key,
    panel: &TomcatPanel,
    k: i32,
) -> bool {
    let [x, y, w, h] = r;
    let centre = Vec2::new(x as f32 + w as f32 / 2.0, y as f32 + h as f32 / 2.0);
    let rad = w.min(h) as f32 / 2.0;
    let pen = k as f32;
    match key.kind() {
        "twist" => {
            // The A-6's: a grey cap with a pale edge, the lens in a black ring.
            let latched = panel.latched(key);
            c.fill_rect(x, y, w, h, 7);
            c.fill_rect(x + k, y + k, w - 2 * k, h - 2 * k, 8);
            crate::intruder::disc(c, centre, rad * 0.7, BLACK);
            crate::intruder::disc(c, centre, rad * 0.54, if latched { 86 } else { 18 });
            latched
        }
        "lamp" => {
            // A dark cap in a black frame, its legend dim, bright green while it is in.
            let on = panel.latched(key);
            c.fill_rect(x, y, w, h, BLACK);
            c.fill_rect(x + k, y + k, w - 2 * k, h - 2 * k, CAP);
            let inner = [x + 2 * k, y + 2 * k, w - 4 * k, h - 4 * k];
            let ink = if on { LEGEND_ON } else { LEGEND_OFF };
            crate::panel::draw_label(c, fonts, inner, key.label(), ink, CAP, 3);
            on
        }
        "switch" => {
            c.fill_rect(x, y, w, h, BLACK);
            c.ring(centre, rad * 0.55, pen, KNOB_TOP);
            false
        }
        "knob" => {
            c.fill_rect(x, y, w, h, KNOB_TOP);
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
            c.fill_rect(x, y, w, h, BLACK);
            let channel = match key {
                Key::XmtrChan => Some(panel.xmtr_chan),
                Key::MslChan => Some(panel.msl_chan),
                _ => None,
            };
            if let Some(n) = channel {
                // A thumbwheel: its channel's digit, white on its black drum.
                c.fill_rect(x + k, y + k, w - 2 * k, h - 2 * k, 6);
                let size = h as f32 * 0.6;
                let s = n.to_string();
                stroke_text_at(c, centre.x, centre.y, &s, size, size / 7.0, WHITE, Align::Centre);
            } else {
                c.fill_rect(x + k, y + k, w - 2 * k, h - 2 * k, 7);
                let inner = [x + 2 * k, y + 2 * k, w - 4 * k, h - 4 * k];
                crate::panel::draw_label(c, fonts, inner, key.label(), BLACK, 7, 3);
            }
            false
        }
    }
}

/// A light's face: its legend dark on the lit lamp, pale on the dark one.
pub fn draw_light(c: &mut Canvas, fonts: &Fonts, r: [i32; 4], light: Light, on: bool) {
    let (ink, back) = if on { (BLACK, 47) } else { (44, CAP) };
    crate::panel::draw_label(c, fonts, r, light.label(), ink, back, 3);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every control, light and readout has its tooltip text.
    #[test]
    fn tips_exist() {
        let readouts = [Readout::FuelTotal, Readout::Bingo];
        let ids = Key::all()
            .into_iter()
            .map(Key::tip)
            .chain(LIGHTS.map(Light::tip))
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
        assert_eq!(Key::parse("Wcs( TwsAuto )"), Some(Key::Wcs(WcsMode::TwsAuto)));
        assert_eq!(Key::parse("Knob(Bright)"), Some(Key::Knob(RioKnob::Bright)));
        assert_eq!(Key::parse("Bogus"), None);
    }

    /// One button of a group is in at a time.
    #[test]
    fn radio_groups() {
        let mut p = TomcatPanel::default();
        let groups: [Vec<Key>; 6] = [
            STEER_MODES.map(Key::Steer).to_vec(),
            STEER_CMDS.map(Key::SteerCmd).to_vec(),
            WCS_MODES.map(Key::Wcs).to_vec(),
            DDD_RANGES.map(Key::DddRange).to_vec(),
            DDD_MODES.map(Key::DddMode).to_vec(),
            LTS_MODES.map(Key::LtsMode).to_vec(),
        ];
        for group in &groups {
            assert_eq!(group.iter().filter(|k| p.latched(**k)).count(), 1, "{group:?}");
            for &k in group {
                p.press(k);
                let latched: Vec<Key> = group.iter().copied().filter(|k| p.latched(*k)).collect();
                assert_eq!(latched, [k]);
                p.press(k);
                assert!(p.latched(k));
            }
        }
        assert!(
            p.latched(Key::Steer(SteerMode::Landing)) && p.latched(Key::Wcs(WcsMode::PulseStt))
        );
        // The toggles go in and out on their own.
        for k in CCMS.map(Key::Ccm).into_iter().chain(TID_OPTIONS.map(Key::TidOption)) {
            let was = p.latched(k);
            p.press(k);
            assert_ne!(p.latched(k), was, "{k:?}");
        }
    }

    /// The switches and knobs go round their positions; only they have one; every control
    /// changes the panel or does something.
    #[test]
    fn switches_and_knobs() {
        let mut p = TomcatPanel::default();
        for k in Key::all() {
            let Some((at, n)) = p.position(k) else {
                assert!(matches!(k.kind(), "twist" | "lamp" | "cap"), "{k:?}");
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
        assert_eq!(p, TomcatPanel::default());
        for k in Key::all() {
            let mut q = TomcatPanel::default();
            // (A radio group's default button pressed again changes nothing.)
            let out = q.press(k);
            let pressed_default = q.latched(k) && TomcatPanel::default().latched(k);
            assert!(q != TomcatPanel::default() || out.is_some() || pressed_default, "{k:?}");
        }
        assert_eq!(
            p.press(Key::AntennaUp),
            Some(PanelOut::Mfd(crate::mfd::MfdCmd::FcrElevation(1)))
        );
        assert_eq!(p.press(Key::Launch), Some(PanelOut::Action(Action::FireWeapon)));
        // The sweep switch back to AUTO puts the wing on its schedule.
        assert_eq!(p.press(Key::SweepMode), None);
        assert_eq!(p.press(Key::SweepMode), Some(PanelOut::Action(Action::WingSweepAuto)));
        for _ in 0..20 {
            p.press(Key::XmtrChan);
            assert!((1..=CHANNELS).contains(&p.xmtr_chan));
        }
    }

    /// The panel goes to the other seat in 16 bytes and comes back as it was.
    #[test]
    fn pack_round_trip() {
        let mut p = TomcatPanel::default();
        let mut q = TomcatPanel::default();
        q.unpack(&p.pack());
        assert_eq!(q, p);
        // Every control in turn, pressed a few times: each state goes across.
        for (i, k) in Key::all().into_iter().enumerate() {
            for _ in 0..(i % 3 + 1) {
                p.press(k);
            }
            let mut q = TomcatPanel::default();
            q.unpack(&p.pack());
            q.hooks = p.hooks;
            assert_eq!(q, p, "after {k:?}");
        }
        // The counters' low bits.
        for _ in 0..300 {
            p.press(Key::TidHook);
            p.press(Key::Launch);
        }
        p.hook_target = hook_code(Some(crate::mfd::ContactRef::Pilot(7)));
        let mut q = p.clone();
        q.unpack(&p.pack());
        assert_eq!(q, p);
        assert_eq!(hook_ref(q.hook_target), Some(crate::mfd::ContactRef::Pilot(7)));
        let unit = Some(crate::mfd::ContactRef::Unit(300));
        assert_eq!(hook_ref(hook_code(unit)), unit);
        assert_eq!(hook_ref(hook_code(None)), None);
    }

    /// The fuel counters.
    #[test]
    fn readouts() {
        assert_eq!(readout(Readout::FuelTotal, 16_204.0, 0.0), "16200");
        assert_eq!(readout(Readout::Bingo, 0.0, 3_000.0), "03000");
        assert_eq!(readout(Readout::FuelTotal, -5.0, 0.0), "00000");
    }

    /// The panel goes to the other crew member as it is.
    #[test]
    fn panel_serializes() {
        let mut p = TomcatPanel::default();
        p.press(Key::Wcs(WcsMode::TwsAuto));
        p.press(Key::Knob(RioKnob::Bright));
        let text = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<TomcatPanel>(&text).unwrap(), p);
    }
}
