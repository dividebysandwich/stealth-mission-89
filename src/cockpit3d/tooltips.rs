//! Tooltips (desktop): with the mouse still over a cockpit control, display, instrument or
//! warning light, a box by the cursor names it, reads out what an instrument shows (updated
//! while the box is up), says what it does and which keys do the same.
//! It follows the view: whatever comes under the cursor while the head turns, unless something
//! in front hides it (`pick`). No tooltips when switched off (in-flight menu), outside the
//! cockpit views, while looking around (right button held), with the menu or the kneeboard over
//! the view.
//!
//! In VR the part is the control a fingertip touches, else one just ahead of the fingertip; the
//! box is drawn into an image shown on a small screen above the finger, facing the eye point,
//! drawn over the cockpit (readable where it goes into the panel).
//!
//! The texts are in `assets/i18n/*/tooltips.ftl`: the descriptions in the player's language,
//! the titles that are the cockpit's own labels (lamps, bezel and console buttons) in English.

use std::sync::LazyLock;

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, Extent3d, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    TextureDimension, TextureFormat, TextureUsages,
};

use super::controls::Control;
use super::displays::{CanvasRole, ConsoleScreen, Face};
use super::layout::{ConsoleKey, Dial, Effect, Elem, PanelStyle};
use super::pick::{Picker, Pointed};
use super::{CockpitRoot, Hands};
use crate::camera::{View, ViewState};
use crate::hotas::{ButtonTarget, InputConfig};
use crate::input::{Action, key_label, qwerty_key_label};
use crate::mfd::{CamDir, LeftMfd, MfdCmd, MfdState, OsbFn, RightMfd, osb};
use crate::panel::{LAMPS, LampId, LedId, PanelParams};
use crate::settings::{AppState, Settings};
use stealth_i18n::t;

/// Seconds the mouse rests on a part before its tooltip shows.
const DELAY: f32 = 0.4;
/// A tooltip shown this recently: the next one shows at once (moving along the panel).
const LINGER: f32 = 0.3;
/// Longest tooltip line (characters): longer texts are wrapped here, not by the text layout
/// (which would size the box for one line).
const LINE_CHARS: usize = 46;

/// VR: the box's image (pixels; the box is drawn twice as large as on the desktop), the screen's
/// width (metres), how far a part ahead of the fingertip is pointed at (metres) and where the
/// screen is from the fingertip (metres up and towards the eye).
const VR_IMAGE: UVec2 = UVec2::new(1024, 512);
const VR_SCALE: f32 = 2.0;
const VR_WIDTH: f32 = 0.26;
const VR_REACH: f32 = 0.05;
const VR_RAISE: f32 = 0.07;
const VR_TOWARDS_EYE: f32 = 0.04;

/// Tooltip of a cockpit part (the controls' come from their effect): its name, what it shows
/// now (instruments), what it does and the actions whose keys do the same.
#[derive(Component, Clone, Debug)]
pub struct Tip {
    title: String,
    readout: Option<Readout>,
    detail: String,
    actions: Vec<Action>,
}

impl Tip {
    pub fn new(title: impl Into<String>, detail: impl Into<String>, actions: &[Action]) -> Self {
        Self {
            title: title.into(),
            readout: None,
            detail: detail.into(),
            actions: actions.to_vec(),
        }
    }

    /// The tooltip reading out what the instrument shows.
    fn reading(self, readout: Readout) -> Self {
        Self { readout: Some(readout), ..self }
    }
}

/// An instrument whose tooltip reads out what it shows, as it shows it.
#[derive(Clone, Copy, Debug)]
enum Readout {
    Dial(Dial),
    Led(LedId),
    RadarBars,
    Attitude,
    Hsi,
}

/// What the instrument shows now (empty without an aircraft).
fn readout_text(readout: Readout, panel: &PanelParams) -> String {
    let Ok(a) = panel.aircraft.single() else { return String::new() };
    match readout {
        Readout::Dial(dial) => dial_text(dial, panel.dial_value(dial)),
        Readout::Led(led) => match led {
            LedId::Chaff => panel.cm.chaff.to_string(),
            LedId::Flares => panel.cm.flares.to_string(),
            LedId::Decoys => panel.cm.decoys.to_string(),
            // The fuel in lbs, not divided by 10 as on the LEDs.
            LedId::Fuel => dial_text(Dial::Fuel, a.fuel),
            LedId::Thrust => format!("{} %", a.thrust.max(0.0).round() as i32),
        },
        Readout::RadarBars => {
            let (emv, egr, ear) = panel.gauge_values();
            t!("tip-radar-bars-readout", emv = emv, egr = egr, ear = ear)
        }
        Readout::Attitude => {
            let (_, pitch, roll) = a.euler();
            let bank = roll.to_degrees().round() as i32;
            let bank = match bank {
                0 => t!("tip-wings-level").to_string(),
                1.. => t!("tip-bank-right", deg = bank),
                _ => t!("tip-bank-left", deg = -bank),
            };
            t!(
                "tip-attitude-readout",
                pitch = signed(pitch.to_degrees().round() as i32),
                bank = bank
            )
        }
        Readout::Hsi => {
            let Some((heading, bearing, dist)) = panel.hsi_values() else { return String::new() };
            let degrees = |rad: f32| rad.to_degrees().rem_euclid(360.0).round() as i32 % 360;
            let heading = t!("tip-heading", heading = format!("{:03}", degrees(heading)));
            match bearing.zip(dist) {
                Some((bearing, km)) => t!(
                    "tip-heading-waypoint",
                    heading = heading,
                    wp = panel.waypoints.current,
                    bearing = format!("{:03}", degrees(bearing)),
                    distance = km_text(km)
                ),
                None => heading,
            }
        }
    }
}

/// A needle instrument's value (`PanelParams::dial_value`) with its unit.
fn dial_text(dial: Dial, value: f32) -> String {
    let v = value.round() as i32;
    match dial {
        Dial::Airspeed => format!("{v} kt"),
        Dial::Altimeter => format!("{v} ft"),
        Dial::Aoa => format!("{value:.1}°"),
        Dial::Vvi => format!("{} ft/min", signed(v)),
        Dial::Rpm => format!("{v} %"),
        Dial::Ftit => format!("{v} °C"),
        Dial::Oil => format!("{v} psi"),
        Dial::Nozzle => t!("tip-nozzle-open", value = v),
        Dial::Fuel => format!("{} lbs", v.max(0)),
        Dial::Clock => {
            let (h, m, s) = crate::mfd::hms(value);
            format!("{h:02}:{m:02}:{s:02}")
        }
    }
}

/// A value with its sign ("+5", "-5", "0").
fn signed(v: i32) -> String {
    if v == 0 { "0".to_string() } else { format!("{v:+}") }
}

/// A distance as the HSI shows it (tenths under 100 km).
fn km_text(km: f32) -> String {
    if km < 100.0 { format!("{km:.1} km") } else { format!("{km:.0} km") }
}

/// An action's name and what it does (in the player's language).
pub fn action_text(a: Action) -> (String, &'static str) {
    use Action::*;
    let id = match a {
        Gear => "tip-gear",
        Autopilot => "tip-autopilot",
        BayDoors => "tip-bay-doors",
        Flaps => "tip-flaps",
        Brakes => "tip-brakes",
        Eject => "tip-eject",
        MasterCaution => "tip-master-caution",
        HudMode => "tip-hud-mode",
        BombMode => "tip-bomb-mode",
        HudDeclutter => "tip-hud-declutter",
        Maps => "tip-maps",
        Nvg => "tip-nvg",
        Ordnance => "tip-ordnance",
        Flir => "tip-flir",
        Ils => "tip-ils",
        MissionOrders => "tip-mission-orders",
        CamAhead => "tip-cam-ahead",
        CamRear => "tip-cam-rear",
        CamRight => "tip-cam-right",
        CamLeft => "tip-cam-left",
        DesignateTarget => "tip-designate-target",
        SelectTarget => "tip-select-target",
        NearestTarget => "tip-nearest-target",
        ClearTarget => "tip-clear-target",
        SelectWeapon => "tip-select-weapon",
        FireWeapon => "tip-fire-weapon",
        Flare => "tip-flare",
        Chaff => "tip-chaff",
        Decoy => "tip-decoy",
        Ecm => "tip-ecm",
        IrJammer => "tip-ir-jammer",
        SelectWaypoint => "tip-select-waypoint",
        ChangeWaypoint => "tip-change-waypoint",
        ResetWaypoint => "tip-reset-waypoint",
        LastWaypoint => "tip-last-waypoint",
        NextWaypoint => "tip-next-waypoint",
        Menu => "tip-menu",
        Volume => "tip-volume",
        Detail => "tip-detail",
        TimeAccel => "tip-time-accel",
        TimeNormal => "tip-time-normal",
        MapZoomIn => "tip-map-zoom-in",
        MapZoomOut => "tip-map-zoom-out",
        MapOrientation => "tip-map-orientation",
        LockCursor => "tip-lock-cursor",
        SwapSoi => "tip-swap-soi",
        Kneeboard => "tip-kneeboard",
        Chat => "tip-chat",
        CameraZoomIn => "tip-camera-zoom-in",
        CameraZoomOut => "tip-camera-zoom-out",
        _ => return (crate::hotas::action_name(a), ""),
    };
    (t!(id).to_string(), desc(id))
}

/// The `.desc` of message `id`: what it does ("" if it has none).
fn desc(id: &str) -> &'static str {
    let attr = format!("{id}.desc");
    if stealth_i18n::exists(&attr) { t!(&attr) } else { "" }
}

/// A tip from message `id` (its value the title, its `.desc` the detail).
fn tip(id: &str, actions: &[Action]) -> Tip {
    Tip::new(t!(id), desc(id), actions)
}

/// What a display command does.
fn cmd_text(c: MfdCmd) -> String {
    use crate::avionics::Format;
    t!(match c {
        MfdCmd::LeftPage(LeftMfd::Map) => "tip-cmd-map-page",
        MfdCmd::LeftPage(LeftMfd::Tac) => "tip-cmd-tac-page",
        MfdCmd::RightPage(RightMfd::Camera) => "tip-cmd-camera-page",
        MfdCmd::RightPage(RightMfd::Way) => "tip-cmd-way-page",
        MfdCmd::RightPage(RightMfd::Wpn) => "tip-cmd-wpn-page",
        MfdCmd::RightPage(RightMfd::Ord) => "tip-cmd-ord-page",
        MfdCmd::ZoomIn | MfdCmd::MapZoom(1..) => "tip-cmd-map-zoom-in",
        MfdCmd::ZoomOut | MfdCmd::MapZoom(..=0) => "tip-cmd-map-zoom-out",
        MfdCmd::ToggleOrient => "tip-cmd-orientation",
        MfdCmd::Flir => "tip-cmd-flir",
        MfdCmd::Cam(CamDir::Ahead) => "tip-cmd-cam-ahead",
        MfdCmd::Cam(CamDir::Right) => "tip-cmd-cam-right",
        MfdCmd::Cam(CamDir::Rear) => "tip-cmd-cam-rear",
        MfdCmd::Cam(CamDir::Left) => "tip-cmd-cam-left",
        MfdCmd::Ils => "tip-cmd-ils",
        MfdCmd::Format(_, f) => match f {
            Format::Hsd => "tip-cmd-hsd",
            Format::Harm => "tip-cmd-harm",
            Format::Tgp => "tip-cmd-tgp",
            Format::Map => "tip-cmd-map",
            Format::Sms => "tip-cmd-sms",
            Format::Hsi => "tip-cmd-hsi",
        },
        MfdCmd::Soi(_) => "tip-cmd-soi",
        MfdCmd::SwapSoi => "tip-cmd-swap-soi",
        MfdCmd::HsdRange(1..) => "tip-cmd-hsd-range-up",
        MfdCmd::HsdRange(..=0) => "tip-cmd-hsd-range-down",
        MfdCmd::HsdCentre => "tip-cmd-hsd-centre",
        MfdCmd::MapWay => "tip-cmd-map-way",
        MfdCmd::Sms(true) => "tip-cmd-sms-orders",
        MfdCmd::Sms(false) => "tip-cmd-sms-weapons",
        MfdCmd::HarmFilter => "tip-cmd-harm-filter",
        MfdCmd::CamZoom(1..) => "tip-cmd-cam-zoom-in",
        MfdCmd::CamZoom(..=0) => "tip-cmd-cam-zoom-out",
        MfdCmd::CamZoomCycle => "tip-cmd-cam-zoom-cycle",
        MfdCmd::TgpSp => "tip-cmd-tgp-sp",
        MfdCmd::TgpSlave => "tip-cmd-tgp-slave",
        MfdCmd::TgpCz => "tip-cmd-tgp-cz",
    })
    .to_string()
}

/// An air defence console button's title (its label, in English) and what it does.
fn console_text(k: ConsoleKey) -> (&'static str, &'static str) {
    let (title, id) = match k {
        ConsoleKey::Tracks => ("TRACKS", "tip-console-tracks"),
        ConsoleKey::Site => ("SITE", "tip-console-site"),
        ConsoleKey::Lock => ("LOCK", "tip-console-lock"),
        ConsoleKey::Drop => ("DROP", "tip-console-drop"),
        ConsoleKey::Guided => ("GUIDED", "tip-console-guided"),
        ConsoleKey::Semi => ("SEMI-ACTIVE", "tip-console-semi"),
        ConsoleKey::Fire => ("FIRE", "tip-console-fire"),
        ConsoleKey::Launcher(_) => ("LAUNCHER", "tip-console-launcher"),
        ConsoleKey::Refill(_) => ("RELOAD", "tip-console-refill"),
        ConsoleKey::Flir => ("FLIR", "tip-console-flir"),
        ConsoleKey::ZoomIn => ("ZOOM IN", "tip-console-zoom"),
        ConsoleKey::ZoomOut => ("ZOOM OUT", "tip-console-zoom"),
        ConsoleKey::RangeIn => ("RANGE -", "tip-console-range"),
        ConsoleKey::RangeOut => ("RANGE +", "tip-console-range"),
        ConsoleKey::Leave => ("LEAVE", "tip-console-leave"),
    };
    (title, t!(id))
}

/// A control's tooltip (bezel buttons: on the current page; None for a blank one).
fn control_tip(effect: Effect, mfd: &MfdState) -> Option<Tip> {
    let (title, detail) = match effect {
        Effect::Action(a) => {
            let (title, detail) = action_text(a);
            (title, detail.to_string())
        }
        Effect::HudMode(m) => {
            let id = match m {
                0 => "tip-hud-nav",
                1 => "tip-hud-air",
                _ => "tip-hud-gnd",
            };
            (t!(id).to_string(), desc(id).to_string())
        }
        Effect::LeftPage(p) => (cmd_text(MfdCmd::LeftPage(p)), t!("tip-left-mfd").to_string()),
        Effect::CamCycle => (t!("tip-cam-cycle").to_string(), desc("tip-cam-cycle").to_string()),
        Effect::TimeToggle => {
            let (title, detail) = action_text(Action::TimeAccel);
            (title, t!("tip-on-off", detail = detail))
        }
        Effect::Console(k) => {
            let (title, detail) = console_text(k);
            (title.to_string(), detail.to_string())
        }
        Effect::Osb(side, slot) => {
            let (label, f, _) = osb(side, slot, mfd)?;
            let detail = match f {
                OsbFn::Cmd(c) => cmd_text(c),
                OsbFn::Action(a) => {
                    let (title, detail) = action_text(a);
                    if detail.is_empty() { title } else { format!("{title}: {detail}") }
                }
                // The only readout: the HSD's range between its RNG- and RNG+ buttons.
                OsbFn::Readout => t!("tip-hsd-range").to_string(),
            };
            (label.to_string(), detail)
        }
    };
    // The key a script `press=` names; both time keys; none for the camera cycle (one key per
    // direction).
    let actions = match effect {
        Effect::TimeToggle => vec![Action::TimeAccel, Action::TimeNormal],
        Effect::CamCycle => Vec::new(),
        _ => effect.action(mfd).into_iter().collect(),
    };
    Some(Tip { title, readout: None, detail, actions })
}

/// The tooltip of an instrument atlas face that is not a button (None: a plain label).
pub(super) fn face_tip(face: Face) -> Option<Tip> {
    let Face::Item(elem) = face else {
        return matches!(face, Face::LedBars).then(radar_bars);
    };
    let (title, detail): (String, &str) = match elem {
        Elem::Lamp(id, label) => {
            let lamp = match id {
                LampId::Stall => "tip-lamp-stall",
                LampId::Trak => "tip-lamp-trak",
                LampId::Ir => "tip-lamp-ir",
                LampId::Rad => "tip-lamp-rad",
                LampId::Fire => "tip-lamp-fire",
                LampId::Eng => "tip-lamp-eng",
                LampId::Fc => "tip-lamp-fc",
                LampId::Jam => "tip-lamp-jam",
                LampId::Av => "tip-lamp-av",
                LampId::Fuel => "tip-lamp-fuel",
                LampId::Bd => "tip-lamp-bd",
                LampId::ChaffAct => "tip-lamp-chaff-act",
                LampId::FlareAct => "tip-lamp-flare-act",
                LampId::DecoyAct => "tip-lamp-decoy-act",
                LampId::EmvWarn => "tip-lamp-emv-warn",
                LampId::LowFuel => "tip-lamp-low-fuel",
                _ => return None,
            };
            // The lamp's label (English, as on the panel), but for the unlabelled lights.
            let (title, detail) = match id {
                LampId::ChaffAct | LampId::FlareAct | LampId::DecoyAct | LampId::EmvWarn => {
                    (t!(lamp), desc(lamp))
                }
                _ => (label.unwrap_or(LAMPS[id as usize].1), t!(lamp)),
            };
            (title.replace('\n', " "), detail)
        }
        Elem::Led(led) => {
            let id = match led {
                LedId::Chaff => "tip-led-chaff",
                LedId::Flares => "tip-led-flares",
                LedId::Decoys => "tip-led-decoys",
                LedId::Fuel => "tip-led-fuel",
                LedId::Thrust => "tip-led-thrust",
            };
            return Some(tip(id, &[]).reading(Readout::Led(led)));
        }
        Elem::Gauge => return Some(radar_bars()),
        Elem::GearLights => (t!("tip-gear-lights").into(), desc("tip-gear-lights")),
        Elem::Dial(dial) => {
            let id = match dial {
                Dial::Airspeed => "tip-dial-airspeed",
                Dial::Altimeter => "tip-dial-altimeter",
                Dial::Aoa => "tip-dial-aoa",
                Dial::Vvi => "tip-dial-vvi",
                Dial::Rpm => "tip-dial-rpm",
                Dial::Ftit => "tip-dial-ftit",
                Dial::Oil => "tip-dial-oil",
                Dial::Nozzle => "tip-dial-nozzle",
                Dial::Fuel => "tip-dial-fuel",
                Dial::Clock => "tip-dial-clock",
            };
            return Some(tip(id, &[]).reading(Readout::Dial(dial)));
        }
        Elem::Label(..) | Elem::Hazard => return None,
    };
    Some(Tip::new(title, detail, &[]))
}

fn radar_bars() -> Tip {
    tip("tip-radar-bars", &[]).reading(Readout::RadarBars)
}

/// The tooltip of a live display (None: the HUD and the atlases).
pub(super) fn display_tip(role: CanvasRole, style: PanelStyle) -> Option<Tip> {
    let id = match role {
        CanvasRole::Left => "tip-display-left",
        CanvasRole::Right if style == PanelStyle::Modern => "tip-display-centre",
        CanvasRole::Right if style == PanelStyle::Tornado => "tip-display-centre-mfd",
        CanvasRole::Right => "tip-display-right",
        CanvasRole::Third => "tip-display-third",
        CanvasRole::Adi => "tip-display-adi",
        CanvasRole::Approach => "tip-display-approach",
        CanvasRole::Rwr => "tip-display-rwr",
        CanvasRole::Tfr => "tip-display-tfr",
        CanvasRole::Hsi => "tip-display-hsi",
        CanvasRole::Ded => "tip-display-ded",
        CanvasRole::Console(ConsoleScreen::Scope) => "tip-display-scope",
        CanvasRole::Console(ConsoleScreen::Board) => "tip-display-board",
        CanvasRole::Console(ConsoleScreen::Eo) => "tip-display-eo",
        CanvasRole::Console(ConsoleScreen::Info) => "tip-display-info",
        CanvasRole::Atlas | CanvasRole::Glow | CanvasRole::Hud => return None,
    };
    let tip = tip(id, &[]);
    Some(match role {
        CanvasRole::Adi => tip.reading(Readout::Attitude),
        CanvasRole::Hsi => tip.reading(Readout::Hsi),
        _ => tip,
    })
}

pub(super) fn stick_tip() -> Tip {
    use Action::*;
    tip("tip-stick", &[PitchDown, PitchUp, RollLeft, RollRight])
}

/// The air defence console's hand controller (`station`).
pub(super) fn console_stick_tip() -> Tip {
    use Action::*;
    tip("tip-console-stick", &[PitchDown, PitchUp, RollLeft, RollRight, FireWeapon])
}

pub(super) fn throttle_tip() -> Tip {
    use Action::*;
    tip("tip-throttle", &[ThrottleUp, ThrottleDown, ThrottleMax, ThrottleIdle])
}

/// The tooltips' font: egui's Hack (as the menus), which has the accented letters of the
/// keyboard layouts (Bevy's built-in font only has ASCII: "é" would show as a box).
const FONT: &[u8] = epaint_default_fonts::HACK_REGULAR;

/// Whether the tooltip font draws every character of `s`.
fn drawable(s: &str) -> bool {
    static FACE: LazyLock<Option<ttf_parser::Face<'static>>> =
        LazyLock::new(|| ttf_parser::Face::parse(FONT, 0).ok());
    FACE.as_ref().is_some_and(|f| s.chars().all(|c| f.glyph_index(c).is_some()))
}

/// The keys bound to the actions: "Key: 9", "Keys: Shift+Z, Shift+X", and "Keys: Up/Z, Down/S"
/// when an action has several keys (those of one action joined by "/", the actions' by ", "). A
/// key whose label on the layout the font can't draw (a non-Latin layout) is named as on a US
/// keyboard.
fn keys_text(actions: &[Action], input: &InputConfig) -> String {
    let groups = actions.iter().map(|&a| {
        input
            .keys_for(ButtonTarget::Action(a))
            .map(|m| {
                let k = key_label(m.mods, m.key);
                if drawable(&k) { k } else { qwerty_key_label(m.mods, m.key) }
            })
            .collect()
    });
    join_keys(groups)
}

/// The keys of each action as text (see `keys_text`); a key already shown is not repeated.
fn join_keys(groups: impl IntoIterator<Item = Vec<String>>) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut shown: Vec<String> = Vec::new();
    for group in groups {
        let start = seen.len();
        for k in group {
            if !seen.contains(&k) {
                seen.push(k);
            }
        }
        if seen.len() > start {
            shown.push(seen[start..].join("/"));
        }
    }
    match seen.as_slice() {
        [] => String::new(),
        [key] => t!("tip-key", key = key.as_str()),
        _ => t!("tip-keys", keys = shown.join(", ")),
    }
}

/// The text broken into lines of at most `LINE_CHARS` (at spaces; a longer word on its own).
fn wrap(text: &str) -> String {
    let mut out = String::new();
    let mut len = 0;
    for word in text.split(' ').filter(|w| !w.is_empty()) {
        let n = word.chars().count();
        if len > 0 && len + 1 + n > LINE_CHARS {
            out.push('\n');
            len = 0;
        } else if len > 0 {
            out.push(' ');
            len += 1;
        }
        out.push_str(word);
        len += n;
    }
    out
}

/// The tooltip box (title, readout, detail and keys lines).
#[derive(Component)]
pub(super) struct TipBox;

#[derive(Component)]
pub(super) struct TipLine(usize);

/// VR: the camera drawing the box into its image.
#[derive(Component)]
pub(super) struct TipCamera;

/// VR: the screen showing the box's image (a child of the cockpit root).
#[derive(Component)]
pub(super) struct TipScreen;

/// VR: the screen's material, drawn over whatever is in front of it.
pub(crate) type TipScreenMaterial = ExtendedMaterial<StandardMaterial, OnTop>;

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
pub(crate) struct OnTop {}

impl MaterialExtension for OnTop {
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_compare = Some(CompareFunction::Always);
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

/// VR: the screen's mesh and material (the screen goes with the cockpit when it is rebuilt).
#[derive(Resource)]
pub(super) struct TipScreenLook {
    mesh: Handle<Mesh>,
    material: Handle<TipScreenMaterial>,
}

/// VR: the fingertip (cockpit frame) whose part's tooltip shows.
#[derive(Resource, Default)]
pub(super) struct TipAnchor(Option<Vec3>);

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_box(
    mut commands: Commands,
    settings: Res<Settings>,
    ui_font: Res<crate::i18n::UiFont>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TipScreenMaterial>>,
) {
    let font = ui_font.handle();
    let k = if settings.vr { VR_SCALE } else { 1.0 };
    let line = |i: usize, size: f32, color: Color| {
        (
            TipLine(i),
            Text::new(""),
            TextFont {
                font: font.clone().into(),
                font_size: bevy::text::FontSize::Px(size * k),
                ..default()
            },
            TextColor(color),
            TextLayout::no_wrap(),
            Node::default(),
        )
    };
    let tip_box = commands
        .spawn((
            TipBox,
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                row_gap: px(2.0 * k),
                padding: UiRect::axes(px(8.0 * k), px(5.0 * k)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.88)),
            Outline::new(px(k), Val::ZERO, Color::srgba(1.0, 0.85, 0.3, 0.5)),
            GlobalZIndex(10),
            Visibility::Hidden,
            DespawnOnExit(AppState::Flight),
            children![
                line(0, 17.0, Color::srgb(1.0, 0.85, 0.3)),
                line(1, 15.0, Color::srgb(0.6, 0.95, 0.65)),
                line(2, 14.0, Color::srgb(0.85, 0.87, 0.9)),
                line(3, 13.0, Color::srgb(0.55, 0.6, 0.65)),
            ],
        ))
        .id();
    if !settings.vr {
        return;
    }
    // VR: the box is drawn into an image (only while it shows) for the screen by the finger.
    let mut image = Image::new_fill(
        Extent3d { width: VR_IMAGE.x, height: VR_IMAGE.y, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
    let image = images.add(image);
    let camera = commands
        .spawn((
            TipCamera,
            Camera2d,
            Camera { order: -2, is_active: false, clear_color: Color::NONE.into(), ..default() },
            RenderTarget::Image(image.clone().into()),
            RenderLayers::none(),
            DespawnOnExit(AppState::Flight),
        ))
        .id();
    commands.entity(tip_box).insert(UiTargetCamera(camera));
    let size = Vec2::new(VR_WIDTH, VR_WIDTH * VR_IMAGE.y as f32 / VR_IMAGE.x as f32);
    commands.insert_resource(TipScreenLook {
        mesh: meshes.add(Rectangle::from_size(size)),
        material: materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture: Some(image),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                fog_enabled: false,
                ..default()
            },
            extension: OnTop {},
        }),
    });
}

pub(super) struct Hover {
    pointed: Option<Pointed>,
    /// Seconds on the same part.
    resting: f32,
    /// Seconds since a tooltip was up.
    since_shown: f32,
    /// The language the lines' font was set for.
    font_for: Option<stealth_i18n::Language>,
}

impl Default for Hover {
    fn default() -> Self {
        Self { pointed: None, resting: 0.0, since_shown: f32::INFINITY, font_for: None }
    }
}

/// Shows the tooltip of the part under the cursor.
#[allow(clippy::too_many_arguments)]
pub(super) fn hover(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    view: Res<ViewState>,
    menu: Res<crate::flight_menu::FlightMenu>,
    hide: Option<Res<crate::camera::HideOwnAircraft>>,
    kneeboard: Option<Res<crate::kneeboard::Kneeboard>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut picker: Picker,
    controls: Query<(Entity, &Control, &Transform), Without<super::controls::Unused>>,
    tips: Query<&Tip>,
    panel: PanelParams,
    input: Res<InputConfig>,
    mut state: Local<Hover>,
    (hands, mut anchor, ui_font): (Res<Hands>, ResMut<TipAnchor>, Res<crate::i18n::UiFont>),
    mut boxes: Query<(&mut Node, &mut Visibility, &ComputedNode), With<TipBox>>,
    mut lines: Query<(&TipLine, &mut Text, &mut Node, &mut TextFont), Without<TipBox>>,
) {
    let dt = time.delta_secs();
    let cockpit_view =
        matches!(view.view, View::Cockpit | View::LookLeft | View::LookRight | View::LookRear);
    let pressing = if settings.vr {
        hands.hands.iter().any(|h| h.trigger > 0.7)
    } else {
        kneeboard.is_some_and(|k| k.overlay_shown())
            || buttons.pressed(MouseButton::Right)
            || buttons.pressed(MouseButton::Left)
    };
    let active =
        settings.tooltips && cockpit_view && !menu.open && !hide.is_some_and(|h| h.0) && !pressing;
    let all: Vec<_> = if active { controls.iter().collect() } else { Vec::new() };
    let (pointed, finger) = if !active {
        (None, None)
    } else if settings.vr {
        finger_pointed(&hands, &mut picker, &all).unzip()
    } else {
        (picker.cursor_ray().and_then(|ray| picker.pick(ray, &all, |_| true)), None)
    };
    if buttons.just_pressed(MouseButton::Left) {
        // A click puts the tooltip away until the mouse rests again.
        state.since_shown = f32::INFINITY;
    }
    if pointed == state.pointed {
        state.resting += dt;
    } else {
        state.pointed = pointed;
        state.resting = 0.0;
    }
    let tip = pointed.and_then(|p| match p {
        Pointed::Control(e) => {
            controls.get(e).ok().and_then(|(_, c, _)| control_tip(c.effect, &panel.mfd))
        }
        Pointed::Tip(e) => tips.get(e).ok().cloned(),
    });
    let show = tip.is_some() && (state.resting >= DELAY || state.since_shown < LINGER);
    state.since_shown = if show { 0.0 } else { state.since_shown + dt };
    anchor.0 = finger.filter(|_| show);

    let Ok((mut node, mut vis, computed)) = boxes.single_mut() else { return };
    vis.set_if_neq(if show { Visibility::Inherited } else { Visibility::Hidden });
    let (Some(tip), true) = (tip, show) else { return };
    let readout = tip.readout.map_or(String::new(), |r| readout_text(r, &panel));
    let texts = [
        wrap(&tip.title),
        wrap(&readout),
        wrap(&tip.detail),
        wrap(&keys_text(&tip.actions, &input)),
    ];
    let language = stealth_i18n::language();
    let new_font = state.font_for != Some(language);
    state.font_for = Some(language);
    for (TipLine(i), mut text, mut line, mut text_font) in &mut lines {
        if new_font {
            text_font.font = ui_font.handle().into();
        }
        let s = &texts[*i];
        if text.0 != *s {
            text.0.clone_from(s);
        }
        let display = if s.is_empty() { Display::None } else { Display::Flex };
        if line.display != display {
            line.display = display;
        }
    }
    let size = computed.size() * computed.inverse_scale_factor();
    let (x, y) = if settings.vr {
        // In the middle of the image, so in the middle of the screen.
        ((VR_IMAGE.as_vec2() - size) / 2.0).max(Vec2::ZERO).into()
    } else {
        // By the cursor, inside the window (above the cursor near the bottom).
        let (Some(cursor), Some(window)) = (picker.cursor(), picker.window_size()) else { return };
        let x = (cursor.x + 14.0).min(window.x - size.x - 4.0).max(4.0);
        let below = cursor.y + 22.0;
        let y = if below + size.y > window.y - 4.0 { cursor.y - size.y - 8.0 } else { below };
        (x, y.max(4.0))
    };
    if node.left != px(x) || node.top != px(y) {
        node.left = px(x);
        node.top = px(y);
    }
}

/// VR: what a fingertip points at, and the fingertip: the control it touches, else the part
/// just ahead of it (along the controller from the grip to the fingertip). Not while the hand
/// holds a lever or the kneeboard.
fn finger_pointed(
    hands: &Hands,
    picker: &mut Picker,
    controls: &[(Entity, &Control, &Transform)],
) -> Option<(Pointed, Vec3)> {
    let free = || hands.hands.iter().filter(|h| h.active && !h.lever && !h.carrying);
    if let Some(&(e, ..)) = controls.iter().find(|(_, c, _)| c.touched())
        && let Some(hand) = free().find(|h| h.touching)
    {
        return Some((Pointed::Control(e), hand.finger));
    }
    free().find_map(|hand| {
        let along = (hand.finger - hand.grip.translation).normalize_or_zero();
        let (ray, reach) =
            picker.segment(hand.finger - along * VR_REACH, hand.finger + along * VR_REACH)?;
        // Controls are pointed at by touching them only.
        match picker.pick_within(ray, reach, controls, |_| false)? {
            p @ Pointed::Tip(_) => Some((p, hand.finger)),
            Pointed::Control(_) => None,
        }
    })
}

/// VR: the screen above the fingertip, facing the eye point (the cockpit's origin), drawn while
/// the tooltip shows.
pub(super) fn place_screen(
    mut commands: Commands,
    anchor: Res<TipAnchor>,
    look: Option<Res<TipScreenLook>>,
    roots: Query<Entity, With<CockpitRoot>>,
    mut screens: Query<(Entity, &ChildOf, &mut Transform, &mut Visibility), With<TipScreen>>,
    mut cams: Query<&mut Camera, With<TipCamera>>,
) {
    let Some(look) = look else { return };
    let Ok(root) = roots.single() else { return };
    if let Ok(mut cam) = cams.single_mut()
        && cam.is_active != anchor.0.is_some()
    {
        cam.is_active = anchor.0.is_some();
    }
    let Some(finger) = anchor.0 else {
        for (.., mut vis) in &mut screens {
            vis.set_if_neq(Visibility::Hidden);
        }
        return;
    };
    let to_eye = (-finger).normalize_or_zero();
    let at = finger + to_eye * VR_TOWARDS_EYE + Vec3::Y * VR_RAISE;
    // The screen's front (+Z) faces the eye.
    let place = Transform::from_translation(at).looking_to(at, Vec3::Y);
    match screens.iter_mut().find(|(_, parent, ..)| parent.parent() == root) {
        Some((_, _, mut tf, mut vis)) => {
            *tf = place;
            vis.set_if_neq(Visibility::Inherited);
        }
        // The first time, and after the cockpit was rebuilt (the old screen went with it).
        None => {
            commands.spawn((
                TipScreen,
                Mesh3d(look.mesh.clone()),
                MeshMaterial3d(look.material.clone()),
                place,
                NotShadowCaster,
                ChildOf(root),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_of_one_action_are_joined_by_a_slash() {
        let g = |keys: &[&str]| keys.iter().map(|k| k.to_string()).collect::<Vec<_>>();
        assert_eq!(join_keys(vec![]), "");
        assert_eq!(join_keys(vec![g(&[]), g(&[])]), "");
        assert_eq!(join_keys(vec![g(&["9"])]), "Key: 9");
        assert_eq!(join_keys(vec![g(&["ArrowUp", "Z"])]), "Keys: ArrowUp/Z");
        assert_eq!(
            join_keys(vec![g(&["ArrowUp", "Z"]), g(&[]), g(&["ArrowDown", "S"])]),
            "Keys: ArrowUp/Z, ArrowDown/S"
        );
        // A key shared by two actions, or bound twice to one, shows once.
        assert_eq!(join_keys(vec![g(&["A", "A"]), g(&["B"])]), "Keys: A, B");
        assert_eq!(join_keys(vec![g(&["A", "B"]), g(&["B"]), g(&["C"])]), "Keys: A/B, C");
    }

    /// Every key shows: the US names, the accented letters and symbols of the European layouts
    /// (AZERTY, QWERTZ, Nordic, Spanish...); what the font lacks falls back to the US name.
    #[test]
    fn key_labels_are_drawable() {
        for m in crate::input::default_key_mappings() {
            let k = qwerty_key_label(m.mods, m.key);
            assert!(drawable(&k), "{k}");
        }
        for c in crate::keyboard_layout::EUROPEAN_KEYCAPS.chars() {
            assert!(drawable(&c.to_string()), "{c}");
        }
        assert!(!drawable("你"));
    }

    /// Every panel button has a tooltip, and every bezel button on the first pages but the blank
    /// ones.
    #[test]
    fn every_control_has_a_tooltip() {
        use super::super::layout::PanelLayout;
        let mfd = MfdState::default();
        for style in [
            PanelStyle::Classic,
            PanelStyle::Modern,
            PanelStyle::F16,
            PanelStyle::Tornado,
            PanelStyle::A10,
        ] {
            for effect in PanelLayout::for_style(style).items.iter().filter_map(|p| p.cap) {
                let tip = control_tip(effect, &mfd).unwrap_or_else(|| panic!("{effect:?}"));
                assert!(!tip.title.is_empty() && !tip.title.contains('_'), "{effect:?}");
            }
        }
        for side in [crate::mfd::MfdSide::Left, crate::mfd::MfdSide::Right] {
            for slot in 0..crate::mfd::OSB_COUNT {
                let effect = Effect::Osb(side, slot);
                assert_eq!(control_tip(effect, &mfd).is_some(), osb(side, slot, &mfd).is_some());
            }
        }
    }

    /// Instrument readouts: units, signs, the clock's time of day; the font draws them all.
    #[test]
    fn dial_readouts() {
        assert_eq!(dial_text(Dial::Clock, 3725.4), "01:02:05");
        assert_eq!(dial_text(Dial::Clock, 86400.0 + 60.0), "00:01:00");
        assert_eq!(dial_text(Dial::Altimeter, 12449.6), "12450 ft");
        assert_eq!(dial_text(Dial::Vvi, -1200.0), "-1200 ft/min");
        assert_eq!(dial_text(Dial::Vvi, 0.3), "0 ft/min");
        assert_eq!(dial_text(Dial::Vvi, 600.0), "+600 ft/min");
        assert_eq!(dial_text(Dial::Aoa, 4.26), "4.3°");
        assert_eq!(km_text(12.34), "12.3 km");
        assert_eq!(km_text(123.4), "123 km");
        assert_eq!(dial_text(Dial::Fuel, -3.0), "0 lbs");
        use Dial::*;
        for dial in [Airspeed, Altimeter, Aoa, Vvi, Rpm, Ftit, Oil, Nozzle, Fuel, Clock] {
            let text = dial_text(dial, 950.0);
            assert!(drawable(&text), "{text}");
        }
    }

    #[test]
    fn long_texts_wrap_at_spaces() {
        let text = "Horizontal situation indicator: heading, bearing and distance to the waypoint";
        let wrapped = wrap(text);
        assert!(wrapped.lines().count() > 1);
        assert!(wrapped.lines().all(|l| l.chars().count() <= LINE_CHARS), "{wrapped}");
        assert_eq!(wrapped.replace('\n', " "), text);
        assert_eq!(wrap("Flaps"), "Flaps");
        assert_eq!(wrap(""), "");
    }
}
