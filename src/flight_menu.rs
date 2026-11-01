//! In-flight settings menu (Esc, Pause, gamepad Start or the MENU button of the left side panel):
//! a screen in front of the pilot, in the cockpit, so it works on the desktop and in VR. The
//! simulation pauses while it is open.
//!
//! The screen is a panel with cut corners in its own greens (not the flight palette): the page's
//! title in a tab at the top, the entries in framed rows (on the main page and OPTIONS with a
//! Lucide icon each), the highlighted one lit up, every page's BACK the ESC BACK button in the
//! footer.
//!
//! The main page: RESUME, SAVE MISSION, OPTIONS, KEY GUIDE, END MISSION (set apart). SAVE MISSION
//! saves a single mission's flight as it is (`savegame.rs`) and says SAVED beside it; a loaded
//! save starts with the menu open. OPTIONS has the pages of the main menu's OPTIONS window (`options.rs`) that
//! make sense in flight: GAME (cockpit, HUD and avionics style, auto trim, tooltips, the VR cockpit
//! size), GRAPHICS (shaders, MSAA, shadows, point lights, time of day), SOUND (volumes, the radio
//! and ATC voice ones too, the engine low-pass) and CONTROLS (head tracking, the key and joystick bindings, the key guide). The
//! bindings are the main menu's dialog (`input_dialog.rs`) over the view (in VR on the menu panel
//! in front of the head), the menu's screen hidden until its DONE; it takes every key, Esc
//! included. They apply at once and are saved to the preferences when the menu closes. Esc, BACK or
//! the gamepad's B go back a page. Choose an entry with the mouse, a VR fingertip and the trigger,
//! the arrow keys / gamepad d-pad / VR thumbstick and Enter / A / X. In VR a fingertip on the
//! screen highlights an entry and the trigger presses it (a trigger pulled away from the screen and
//! the cockpit controls presses the highlighted entry).
//!
//! KEY GUIDE shows the key bindings and the mouse or VR controllers (`key_guide.rs`) on the
//! screen, scrolled with the arrow keys, the mouse wheel, the gamepad d-pad or a VR thumbstick;
//! Esc, a click or the trigger go back to the entries. A search field at its top takes the
//! typed text and keeps only the rows whose description holds it (Backspace deletes, its X or
//! Esc clears it). In VR a fingertip on the field and the trigger open a keyboard under it,
//! whose keys the fingertip and the trigger press (DONE closes it); the trigger elsewhere still
//! goes back.
//!
//! END MISSION keeps the flight paused and shows EXITING... until the debrief takes over.
//!
//! In a dynamic campaign the world goes on while the menu is open (nothing pauses), and END
//! SORTIE ends the sortie: the campaign's debrief and mission board come up in the flight.
//!
//! In VR the end-of-mission overlay (a 2D overlay on the desktop) is a screen floating in front of
//! the head, also in the outside views, drawn over the cockpit.

use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::camera::{EyeCamera, View, ViewState};
use crate::canvas::{Canvas, CanvasTexture, TRANSPARENT};
use crate::cockpit3d::{
    BUTTON_PRIMARY, BUTTON_STICK_DOWN, BUTTON_STICK_LEFT, BUTTON_STICK_RIGHT, BUTTON_STICK_UP,
    CockpitRoot, Hands,
};
use crate::font::{Align, capital, stroke_has, stroke_text_at, stroke_width};
use crate::input::{Action, ActionEvent, InputBlock};
use crate::settings::{AppState, PLAYER_LABEL_SIZE_RANGE, Settings, VR_COCKPIT_SCALE_RANGE};

pub struct FlightMenuPlugin;

impl Plugin for FlightMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlightMenu>()
            .add_systems(OnExit(AppState::Flight), close_on_exit)
            .add_systems(
                Update,
                (
                    toggle,
                    spawn_screen,
                    type_search,
                    scroll_guide,
                    navigate,
                    bindings_overlay,
                    draw,
                    vr_end_screen,
                )
                    .chain()
                    .run_if(in_state(AppState::Flight)),
            )
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                bindings_ui.run_if(in_state(AppState::Flight)),
            );
    }
}

/// Menu state; `open` pauses the flight.
#[derive(Resource, Default)]
pub struct FlightMenu {
    pub open: bool,
    /// The page shown and its highlighted entry.
    page: Page,
    cursor: usize,
    /// Settings when the menu opened (saved to the preferences on close if changed).
    opened_with: Option<Settings>,
    /// END MISSION was chosen: the menu shows EXITING... and the flight ends without the end
    /// overlay.
    pub exiting: bool,
    /// The left mouse button was up since the menu opened: only then does a click choose an
    /// entry (the click on the cockpit's MENU button that opened it must not, and the desktop
    /// overlay's cursor position is stale until it is drawn again).
    mouse_armed: bool,
    /// The key guide is shown instead of the entries, scrolled down this far (canvas pixels).
    guide: bool,
    guide_scroll: f32,
    /// The key guide's search (as typed), whether it was clicked or typed into (its caret then
    /// blinks, even while empty), whether its VR keyboard is open, what of it the mouse or a
    /// fingertip is over.
    search: String,
    focused: bool,
    keyboard: bool,
    hover: Option<Spot>,
    /// The bindings dialog is up instead of the menu's screen (until its DONE).
    bindings: bool,
    /// SAVE MISSION can save the flight now (`savegame.rs` says so every frame), was chosen,
    /// and what came of it (shown beside it until the menu closes).
    pub can_save: bool,
    pub save_request: bool,
    pub save_note: Option<&'static str>,
    /// Opens the menu (pausing the flight) as soon as it can: a loaded save starts so.
    pub open_on_start: bool,
    /// Opens the menu on its OPTIONS page as soon as it can (the campaign's Ops screen asks);
    /// opened so, BACK there closes it (`options_only`).
    pub open_options: bool,
    options_only: bool,
}

/// The menu's pages: the main one, OPTIONS and its pages (as the main menu's OPTIONS window).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Page {
    #[default]
    Main,
    Options,
    Game,
    Graphics,
    Effects,
    Sound,
    Controls,
}

impl Page {
    /// The entries in rows, then BACK (the footer's button, `item_rects`).
    fn items(self) -> &'static [Item] {
        use Item::*;
        match self {
            Page::Main => &[Resume, SaveMission, Open(Page::Options), KeyGuide, EndMission, Back],
            Page::Options => &[
                Open(Page::Game),
                Open(Page::Graphics),
                Open(Page::Effects),
                Open(Page::Sound),
                Open(Page::Controls),
                Back,
            ],
            Page::Game => &[
                Cockpit,
                Hud,
                Avionics,
                AutoTrim,
                Tooltips,
                Atc,
                PlayerLabels,
                LabelSize,
                CockpitSize,
                Back,
            ],
            Page::Graphics => {
                use crate::effects::Effect as E;
                &[
                    Shaders,
                    Msaa,
                    Shadows,
                    Lighting,
                    TimeOfDay,
                    SunMoon,
                    AutoDetail,
                    Effect(E::Vga),
                    Effect(E::Scanlines),
                    Effect(E::Pixelize),
                    PixelSize,
                    Back,
                ]
            }
            Page::Effects => {
                use crate::effects::Effect as E;
                &[
                    Effect(E::Shake),
                    Effect(E::Flashes),
                    Effect(E::Barrage),
                    Effect(E::CockpitNight),
                    Effect(E::Nvg),
                    Effect(E::Snow),
                    Effect(E::Mist),
                    Effect(E::CloudShadows),
                    Effect(E::Vapour),
                    Effect(E::NightSky),
                    Back,
                ]
            }
            Page::Sound => &[
                Volume(Vol::Master),
                Volume(Vol::Engine),
                Volume(Vol::Effects),
                Volume(Vol::Warnings),
                Volume(Vol::Rwr),
                Volume(Vol::Radio),
                Volume(Vol::Atc),
                RwrTones,
                LowPass,
                Back,
            ],
            Page::Controls => &[HeadTracking, TrackingMouse, Bindings, KeyGuide, Back],
        }
    }

    /// The page BACK and Esc go to (None: they close the menu).
    fn back(self, options_only: bool) -> Option<Page> {
        self.parent().filter(|&p| !(options_only && p == Page::Main))
    }

    fn parent(self) -> Option<Page> {
        match self {
            Page::Main => None,
            Page::Options => Some(Page::Main),
            _ => Some(Page::Options),
        }
    }

    fn title(self) -> &'static str {
        match self {
            Page::Main => "MENU",
            Page::Options => "OPTIONS",
            Page::Game => "GAME",
            Page::Graphics => "GRAPHICS",
            Page::Effects => "EFFECTS",
            Page::Sound => "SOUND",
            Page::Controls => "CONTROLS",
        }
    }
}

/// The volumes of the SOUND page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Vol {
    Master,
    Engine,
    Effects,
    Warnings,
    Rwr,
    /// The coop radio's voices (and the spoken ATC), the spoken ATC's own.
    Radio,
    Atc,
}

impl Vol {
    fn read(self, s: &Settings) -> f32 {
        match self {
            Vol::Master => s.master_volume,
            Vol::Engine => s.engine_volume,
            Vol::Effects => s.effects_volume,
            Vol::Warnings => s.warning_volume,
            Vol::Rwr => s.rwr_volume,
            Vol::Radio => s.voice_volume,
            Vol::Atc => s.atc_volume,
        }
    }

    fn get(self, s: &mut Settings) -> &mut f32 {
        match self {
            Vol::Master => &mut s.master_volume,
            Vol::Engine => &mut s.engine_volume,
            Vol::Effects => &mut s.effects_volume,
            Vol::Warnings => &mut s.warning_volume,
            Vol::Rwr => &mut s.rwr_volume,
            Vol::Radio => &mut s.voice_volume,
            Vol::Atc => &mut s.atc_volume,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Item {
    Resume,
    SaveMission,
    /// Opens a page.
    Open(Page),
    KeyGuide,
    EndMission,
    Back,
    Shaders,
    Msaa,
    Shadows,
    Lighting,
    TimeOfDay,
    SunMoon,
    AutoDetail,
    Cockpit,
    Hud,
    Avionics,
    AutoTrim,
    Tooltips,
    Atc,
    PlayerLabels,
    LabelSize,
    CockpitSize,
    /// The size of the pixelize look's blocks.
    PixelSize,
    /// An atmosphere effect's switch.
    Effect(crate::effects::Effect),
    Volume(Vol),
    /// Which radar warning tones sound.
    RwrTones,
    LowPass,
    HeadTracking,
    /// The right mouse button's look and zoom while a head tracker sends.
    TrackingMouse,
    /// The key and joystick bindings dialog (`input_dialog.rs`).
    Bindings,
}

impl Item {
    fn label(self) -> &'static str {
        match self {
            Item::Resume => "RESUME",
            Item::SaveMission => "SAVE MISSION",
            Item::Open(p) => p.title(),
            Item::KeyGuide => "KEY GUIDE",
            Item::EndMission => "END MISSION",
            Item::Back => "BACK",
            Item::Shaders => "SHADERS",
            Item::Msaa => "ANTI-ALIASING",
            Item::Shadows => "SHADOWS",
            Item::Lighting => "POINT LIGHTS",
            Item::TimeOfDay => "TIME OF DAY",
            Item::SunMoon => "SUN AND MOON",
            Item::AutoDetail => "AUTO SCENERY DETAIL",
            Item::Cockpit => "COCKPIT",
            Item::Hud => "HUD",
            Item::Avionics => "AVIONICS",
            Item::AutoTrim => "AUTO TRIM",
            Item::Tooltips => "TOOLTIPS",
            Item::Atc => "AIR TRAFFIC CONTROL",
            Item::PlayerLabels => "PLAYER LABELS",
            Item::LabelSize => "LABEL SIZE",
            Item::CockpitSize => "COCKPIT SIZE",
            Item::PixelSize => "PIXEL SIZE",
            Item::Effect(e) => e.menu_label(),
            Item::Volume(Vol::Master) => "MASTER VOLUME",
            Item::Volume(Vol::Engine) => "ENGINE",
            Item::Volume(Vol::Effects) => "EFFECTS",
            Item::Volume(Vol::Warnings) => "WARNINGS",
            Item::Volume(Vol::Rwr) => "RWR VOLUME",
            Item::Volume(Vol::Radio) => "RADIO",
            Item::Volume(Vol::Atc) => "ATC VOICE",
            Item::RwrTones => "RWR TONES",
            Item::LowPass => "ENGINE LOW-PASS",
            Item::HeadTracking => "HEAD TRACKING",
            Item::TrackingMouse => "TRACKER MOUSE LOOK",
            Item::Bindings => "KEYS AND JOYSTICK",
        }
    }

    fn value(self, s: &Settings) -> String {
        let style = |classic: bool| if classic { "CLASSIC" } else { "MODERN" }.to_string();
        let on = |b: bool| if b { "ON" } else { "OFF" }.to_string();
        match self {
            Item::Shaders => style(!s.modern_shading),
            Item::Msaa => on(s.msaa && !crate::palette::pixelized(s)),
            Item::Shadows => on(s.shadows),
            Item::Lighting => on(s.local_lights),
            Item::TimeOfDay => style(!s.continuous_time),
            Item::SunMoon => on(s.sun_moon),
            Item::AutoDetail => on(s.vr_auto_detail),
            Item::Cockpit => style(s.classic_cockpit),
            Item::Hud => style(s.classic_hud),
            Item::Avionics => style(s.classic_avionics),
            Item::AutoTrim => on(s.auto_trim),
            Item::Tooltips => on(s.tooltips),
            Item::Atc => on(s.atc),
            Item::PlayerLabels => on(s.player_labels),
            Item::LabelSize => format!("< {:.0}% >", s.player_label_size * 100.0),
            Item::PixelSize => format!("< {} PX >", s.effects.pixel_block),
            Item::Effect(e) => on(e.get(&s.effects)),
            Item::CockpitSize => format!("< {:.0}% >", s.vr_cockpit_scale * 100.0),
            Item::Volume(v) => format!("< {:.0}% >", v.read(s) * 100.0),
            Item::RwrTones => format!("< {} >", s.rwr_tones.name()),
            Item::LowPass => on(s.engine_low_pass),
            Item::HeadTracking => on(s.head_tracking),
            Item::TrackingMouse => on(s.tracking_mouse_look),
            // The pages get a chevron instead (`draw`).
            Item::Open(_)
            | Item::KeyGuide
            | Item::Bindings
            | Item::Resume
            | Item::SaveMission
            | Item::EndMission
            | Item::Back => String::new(),
        }
    }

    /// The cockpit size and the automatic scenery detail only matter in the headset, the label
    /// size only with the player labels on, the pixel size only with the pixelize look on, the
    /// cockpit style only for aircraft with two cockpits, the sun and moon switch only with the
    /// classic time of day, head tracking only on the desktop, MSAA not on WebGL2 nor with the
    /// pixelize look, scanlines and pixelize not in VR; the bindings not over the campaign's Ops
    /// screen (its egui context is the one up then); SAVE MISSION while the flight can be saved.
    fn enabled(self, s: &Settings, ops: bool, can_save: bool) -> bool {
        match self {
            Item::Bindings => !ops,
            Item::SaveMission => can_save,
            Item::CockpitSize | Item::AutoDetail => s.vr,
            Item::LabelSize => s.player_labels,
            Item::PixelSize => crate::palette::pixelized(s),
            Item::Cockpit => s.aircraft.spec().cockpits.len() > 1,
            Item::SunMoon => !s.continuous_time,
            Item::HeadTracking => !s.vr && !cfg!(target_arch = "wasm32"),
            Item::TrackingMouse => s.head_tracking && !s.vr && !cfg!(target_arch = "wasm32"),
            Item::Msaa => !crate::WEBGL2 && !crate::palette::pixelized(s),
            Item::Effect(crate::effects::Effect::Scanlines | crate::effects::Effect::Pixelize) => {
                !s.vr
            }
            _ => true,
        }
    }

    /// Left / right change it (the others only take a press).
    fn adjustable(self) -> bool {
        !matches!(
            self,
            Item::Resume
                | Item::SaveMission
                | Item::Open(_)
                | Item::KeyGuide
                | Item::Bindings
                | Item::EndMission
                | Item::Back
        )
    }
}

/// Screen size (metres) and its distance in front of and height relative to the eye.
const SIZE: Vec2 = Vec2::new(0.42, 0.28);
const POSITION: Vec3 = Vec3::new(0.0, -0.02, -0.52);
/// Canvas size (drawing units) and texels per unit.
const W: usize = 768;
const H: usize = 512;
const SCALE: usize = 2;
/// The panel's edge (inset from the canvas, room for its glow) and its cut corners.
const EDGE: f32 = 5.0;
const CUT: f32 = 22.0;
/// The bottom of the title's tab and how far its right side slants; the header's line steps
/// up this far towards the right, as the footer's.
const HEADER_Y: f32 = 64.0;
const SLANT: f32 = 22.0;
const STEP: f32 = 12.0;
/// The footer's line (its left part), the BACK button below it (x0, y0, x1, y1).
const FOOT_Y: f32 = 448.0;
const BACK: [f32; 4] = [W as f32 - 176.0, 456.0, W as f32 - 30.0, 490.0];
/// The rows of the entries: between these heights, at most this far apart, this far between
/// two; the space above END MISSION.
const TOP: f32 = 86.0;
const BOTTOM: f32 = 420.0;
const MAX_PITCH: f32 = 60.0;
const ROW_GAP: f32 = 5.0;
const END_GAP: f32 = 14.0;
/// Left and right ends of the rows.
const ROW_X: [f32; 2] = [24.0, W as f32 - 24.0];
/// The key guide's area, below its search field.
const GUIDE: [f32; 4] = [24.0, 122.0, W as f32 - 24.0, FOOT_Y - 24.0];
/// The key guide's search field (x0, y0, x1, y1) and its clear button at the right end.
const SEARCH: [f32; 4] = [24.0, 80.0, W as f32 - 24.0, 110.0];
const CLEAR: [f32; 4] = [SEARCH[2] - 32.0, SEARCH[1], SEARCH[2], SEARCH[3]];
/// The search's VR keyboard: its keys' size and spacing, the top of its five rows (at the
/// bottom of the guide, above the footer).
const KEY_W: f32 = 64.0;
const KEY_H: f32 = 34.0;
const KEY_GAP: f32 = 6.0;
const KEYBOARD_TOP: f32 = GUIDE[3] - 5.0 * (KEY_H + KEY_GAP);
/// Flight palette colours of the VR end screen: background, bright and dim green (the panel's
/// lamp greens).
const BG: u8 = 5;
const BRIGHT: u8 = 47;
const DIM: u8 = 45;
/// A VR fingertip touches the screen from this far in front of it to this far behind it
/// (metres; a finger easily passes through the plane).
const TOUCH_FRONT: f32 = 0.05;
const TOUCH_BEHIND: f32 = 0.10;

/// The menu screen's colours (its own palette, `colours`): a ramp from near black to a pale
/// mint, by percent.
mod ink {
    /// The colour `pct` percent up the ramp.
    pub const fn shade(pct: u8) -> u8 {
        1 + (pct as u16 * 99 / 100) as u8
    }
    pub const PANEL: u8 = shade(10);
    pub const TAB: u8 = shade(18);
    pub const ROW: u8 = shade(15);
    /// The rows' frames, the icons' separators.
    pub const FRAME: u8 = shade(34);
    /// The header's and footer's lines, the slanted marks.
    pub const LINE: u8 = shade(50);
    pub const GLOW: u8 = shade(30);
    /// The highlighted row: its fill from top to bottom, its frame's glow.
    pub const LIT_TOP: u8 = shade(56);
    pub const LIT_BOTTOM: u8 = shade(42);
    pub const LIT_GLOW: u8 = shade(46);
    pub const EDGE: u8 = shade(76);
    pub const NOTE: u8 = shade(60);
    pub const TEXT: u8 = shade(88);
    pub const LIT_TEXT: u8 = shade(97);
    pub const OFF: u8 = shade(38);
}

/// The ramp of `ink`: index 0 see-through, 1 to 100 through these colours (8-bit sRGB).
fn colours() -> std::sync::Arc<[u32; 256]> {
    const STOPS: [(f32, [f32; 3]); 8] = [
        (0.0, [2.0, 8.0, 6.0]),
        (0.12, [6.0, 21.0, 15.0]),
        (0.2, [10.0, 34.0, 25.0]),
        (0.35, [20.0, 90.0, 60.0]),
        (0.55, [30.0, 154.0, 90.0]),
        (0.75, [58.0, 230.0, 138.0]),
        (0.9, [116.0, 255.0, 180.0]),
        (1.0, [212.0, 255.0, 232.0]),
    ];
    let mut lut = [0u32; 256];
    for (i, v) in lut.iter_mut().enumerate().skip(1) {
        let t = ((i - 1) as f32 / 99.0).min(1.0);
        let k = STOPS.windows(2).position(|s| t <= s[1].0).unwrap_or(STOPS.len() - 2);
        let ((t0, a), (t1, b)) = (STOPS[k], STOPS[k + 1]);
        let f = (t - t0) / (t1 - t0);
        let [r, g, b] = std::array::from_fn(|c| (a[c] + (b[c] - a[c]) * f).round() as u8);
        *v = u32::from_le_bytes([r, g, b, 255]);
    }
    std::sync::Arc::new(lut)
}

#[derive(Component)]
struct MenuScreen;

#[derive(Component)]
struct EndScreen;

/// The VR end screen: size (metres) and distance in front of the head.
const END_SIZE: Vec2 = Vec2::new(0.6, 0.4);
const END_DISTANCE: f32 = 0.8;

fn close_on_exit(
    mut menu: ResMut<FlightMenu>,
    mut time: ResMut<Time<Virtual>>,
    mut block: ResMut<InputBlock>,
    mut dialog: ResMut<crate::input_dialog::InputDialog>,
) {
    if menu.bindings {
        dialog.close();
    }
    menu.bindings = false;
    menu.open = false;
    menu.opened_with = None;
    menu.exiting = false;
    time.unpause();
    block.0 = false;
}

/// Opens and closes the menu (Action::Menu), pausing the flight and its sounds.
#[allow(clippy::too_many_arguments)]
fn toggle(
    mut actions: MessageReader<ActionEvent>,
    mut menu: ResMut<FlightMenu>,
    settings: Res<Settings>,
    mut time: ResMut<Time<Virtual>>,
    mut block: ResMut<InputBlock>,
    mut view: ResMut<ViewState>,
    sinks: Query<&AudioSink>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    boss: Res<crate::boss_key::BossKey>,
    mut kneeboard: Option<ResMut<crate::kneeboard::Kneeboard>>,
) {
    // Not behind the boss screen (closing the menu would resume the flight it paused).
    let pressed =
        actions.read().filter(|a| a.0 == Action::Menu).count() % 2 == 1 && !boss.showing();
    // The bindings dialog takes Esc (it cancels a capture) and every other key: only its DONE
    // closes it.
    if menu.bindings {
        return;
    }
    if menu.open_on_start {
        menu.open_on_start = false;
        let pause = campaign.is_none();
        set_open(&mut menu, true, pause, &settings, &mut time, &mut block, &sinks);
    }
    if std::mem::take(&mut menu.open_options) {
        let pause = campaign.is_none();
        set_open(&mut menu, true, pause, &settings, &mut time, &mut block, &sinks);
        go_to(&mut menu, Page::Options);
        menu.options_only = true;
    }
    // Esc in the key guide clears its search, else goes back to the entries; on a page it goes
    // to the page before.
    if pressed && menu.open && menu.guide {
        if menu.search.is_empty() {
            menu.guide = false;
        } else {
            clear_search(&mut menu);
        }
        menu.focused = false;
    } else if pressed
        && menu.open
        && !menu.exiting
        && let Some(parent) = menu.page.back(menu.options_only)
    {
        go_to(&mut menu, parent);
    } else if pressed
        && !menu.open
        && let Some(kb) = kneeboard.as_mut().filter(|kb| kb.overlay_shown())
    {
        // Esc puts the kneeboard shown over the view away first; the next one opens the menu.
        kb.close_overlay();
    } else if pressed && !menu.exiting {
        let open = !menu.open;
        let pause = campaign.is_none();
        set_open(&mut menu, open, pause, &settings, &mut time, &mut block, &sinks);
        if menu.open && view.view != View::Cockpit {
            view.view = View::Cockpit;
        }
    }
}

fn set_open(
    menu: &mut FlightMenu,
    open: bool,
    pause: bool,
    settings: &Settings,
    time: &mut Time<Virtual>,
    block: &mut InputBlock,
    sinks: &Query<&AudioSink>,
) {
    if menu.open == open {
        return;
    }
    menu.open = open;
    menu.guide = false;
    block.0 = open;
    if open {
        menu.page = Page::Main;
        menu.options_only = false;
        menu.cursor = 0;
        menu.save_note = None;
        menu.mouse_armed = false;
        menu.opened_with = Some(settings.clone());
        if pause {
            time.pause();
            for s in sinks {
                s.pause();
            }
        }
    } else {
        time.unpause();
        if pause {
            for s in sinks {
                s.play();
            }
        }
        save_options(menu, settings);
    }
}

/// The options changed since the menu opened: saved to the preferences.
fn save_options(menu: &mut FlightMenu, s: &Settings) {
    use crate::prefs::Prefs;
    if menu.opened_with.take().is_some_and(|before| Prefs::options(&before) != Prefs::options(s)) {
        Prefs::save_options(s);
    }
}

/// Shows a page, the first entry after BACK highlighted (on the way back the entry that opened
/// the page).
fn go_to(menu: &mut FlightMenu, page: Page) {
    let from = menu.page;
    let items = page.items();
    menu.cursor = items
        .iter()
        .position(|&i| i == Item::Open(from))
        .or_else(|| items.iter().position(|&i| i != Item::Back))
        .unwrap_or(0);
    menu.page = page;
}

/// The menu screen: in VR in front of the pilot in the cockpit (spawned again when the cockpit
/// is rebuilt, e.g. after switching its style), on the desktop a 2D overlay.
fn spawn_screen(
    mut commands: Commands,
    roots: Query<Entity, With<CockpitRoot>>,
    screens: Query<(), With<MenuScreen>>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(root) = roots.single() else { return };
    if !screens.is_empty() {
        return;
    }
    // See-through past the cut corners.
    let mut tex = CanvasTexture::new(Canvas::scaled(W, H, SCALE), &mut images, true);
    tex.colours = Some(colours());
    if let Some(mut img) = images.get_mut(&tex.image) {
        img.sampler = bevy::image::ImageSampler::linear();
    }
    if !settings.vr {
        // On the desktop a flat overlay in the middle of the window: in front of the eye it
        // would be cut by the camera's near plane.
        let image = ImageNode::new(tex.image.clone());
        let overlay = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                GlobalZIndex(10),
                DespawnOnExit(AppState::Flight),
            ))
            .id();
        let screen = commands
            .spawn((
                MenuScreen,
                tex,
                image,
                Node { height: vh(70), aspect_ratio: Some(W as f32 / H as f32), ..default() },
                RelativeCursorPosition::default(),
                Visibility::Hidden,
            ))
            .id();
        commands.entity(overlay).add_child(screen);
        return;
    }
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(tex.image.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    });
    // Only if the cockpit still stands when the commands apply (it can be rebuilt the frame
    // this runs, e.g. when a campaign mission starts), or the screen would be left without one.
    let screen = (
        MenuScreen,
        tex,
        Mesh3d(meshes.add(Rectangle::new(SIZE.x, SIZE.y))),
        MeshMaterial3d(material),
        Transform::from_translation(POSITION),
        Visibility::Hidden,
        NotShadowCaster,
    );
    commands.entity(root).queue_silenced(move |mut root: EntityWorldMut| {
        root.with_child(screen);
    });
}

/// Canvas pixel of a point in the screen's local frame (None off the screen).
fn to_canvas(local: Vec3) -> Option<Vec2> {
    let u = local.x / SIZE.x + 0.5;
    let v = 0.5 - local.y / SIZE.y;
    ((0.0..1.0).contains(&u) && (0.0..1.0).contains(&v))
        .then(|| Vec2::new(u * W as f32, v * H as f32))
}

/// Where the entries of a page are (x0, y0, x1, y1): BACK the footer's button, the others in
/// rows as tall as fit (up to `MAX_PITCH` apart), END MISSION set apart.
fn item_rects(page: Page) -> Vec<[f32; 4]> {
    let items = page.items();
    let rows = items.iter().filter(|&&i| i != Item::Back).count().max(1) as f32;
    let gap = if items.contains(&Item::EndMission) { END_GAP } else { 0.0 };
    let pitch = ((BOTTOM - TOP - gap) / rows).min(MAX_PITCH);
    let mut y = TOP;
    items
        .iter()
        .map(|item| match item {
            Item::Back => BACK,
            _ => {
                if *item == Item::EndMission {
                    y += END_GAP;
                }
                y += pitch;
                [ROW_X[0], y - pitch, ROW_X[1], y - ROW_GAP]
            }
        })
        .collect()
}

/// Entry of a page under a canvas pixel, and whether the pixel is on the right half (values:
/// larger).
fn item_at(p: Vec2, page: Page) -> Option<(usize, bool)> {
    item_rects(page).iter().position(|&r| inside(p, r)).map(|i| (i, p.x > W as f32 / 2.0))
}

/// The key guide's area, above the VR keyboard while it is open.
fn guide_area(keyboard: bool) -> [f32; 4] {
    if keyboard { [GUIDE[0], GUIDE[1], GUIDE[2], KEYBOARD_TOP - 12.0] } else { GUIDE }
}

/// Whether a canvas pixel is in a rectangle (x0, y0, x1, y1).
fn inside(p: Vec2, [x0, y0, x1, y1]: [f32; 4]) -> bool {
    (x0..x1).contains(&p.x) && (y0..y1).contains(&p.y)
}

/// A key of the search's VR keyboard.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum VKey {
    Char(char),
    Space,
    Delete,
    Clear,
    Done,
}

impl VKey {
    fn label(self) -> String {
        match self {
            VKey::Char(c) => c.to_string(),
            VKey::Space => "SPACE".into(),
            VKey::Delete => "DEL".into(),
            VKey::Clear => "CLEAR".into(),
            VKey::Done => "DONE".into(),
        }
    }
}

/// The VR keyboard's keys and their rectangles: the main menu's character rows (staggered as
/// a keyboard's), then SPACE, DEL, CLEAR and DONE.
fn keyboard_keys() -> Vec<(VKey, [f32; 4])> {
    const STAGGER: f32 = 10.0;
    let unit = KEY_W + KEY_GAP;
    let rows = crate::text_entry::ROWS;
    let x0 = (W as f32 - (10.0 * unit - KEY_GAP) - STAGGER * (rows.len() - 1) as f32) / 2.0;
    let mut keys = Vec::new();
    for (r, row) in rows.iter().enumerate() {
        let y = KEYBOARD_TOP + r as f32 * (KEY_H + KEY_GAP);
        for (i, ch) in row.chars().enumerate() {
            let x = x0 + r as f32 * STAGGER + i as f32 * unit;
            keys.push((VKey::Char(ch), [x, y, x + KEY_W, y + KEY_H]));
        }
    }
    let y = KEYBOARD_TOP + rows.len() as f32 * (KEY_H + KEY_GAP);
    let mut x = x0;
    for (key, units) in
        [(VKey::Space, 3.0), (VKey::Delete, 2.0), (VKey::Clear, 2.0), (VKey::Done, 3.0)]
    {
        keys.push((key, [x, y, x + units * unit - KEY_GAP, y + KEY_H]));
        x += units * unit;
    }
    keys
}

/// What of the key guide is at a canvas pixel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Spot {
    /// The search field's clear button (while there is text).
    Clear,
    Field,
    Key(VKey),
    /// Between the keys of the open keyboard.
    Keyboard,
    List,
}

fn spot_at(p: Vec2, empty: bool, keyboard: bool) -> Spot {
    if !empty && inside(p, CLEAR) {
        Spot::Clear
    } else if inside(p, SEARCH) {
        Spot::Field
    } else if keyboard && p.y >= KEYBOARD_TOP - KEY_GAP {
        keyboard_keys()
            .into_iter()
            .find(|(_, r)| inside(p, *r))
            .map_or(Spot::Keyboard, |(key, _)| Spot::Key(key))
    } else {
        Spot::List
    }
}

fn clear_search(menu: &mut FlightMenu) {
    menu.search.clear();
    menu.guide_scroll = 0.0;
}

/// Adds a character to the search, back at the top of the list (not past its longest).
fn push_search(menu: &mut FlightMenu, ch: char) {
    menu.focused = true;
    if menu.search.chars().count() < crate::key_guide::SEARCH_MAX {
        menu.search.push(ch);
        menu.guide_scroll = 0.0;
    }
}

/// Deletes the search's last character, back at the top of the list.
fn pop_search(menu: &mut FlightMenu) {
    if menu.search.pop().is_some() {
        menu.guide_scroll = 0.0;
    }
}

/// A VR keyboard key pressed.
fn press_key(menu: &mut FlightMenu, key: VKey) {
    match key {
        VKey::Char(ch) => push_search(menu, ch),
        VKey::Space => push_search(menu, ' '),
        VKey::Delete => pop_search(menu),
        VKey::Clear => clear_search(menu),
        VKey::Done => menu.keyboard = false,
    }
}

/// The keys typed in the key guide go into its search (Backspace deletes; the characters the
/// stroke font draws only, not with Alt or Ctrl, which make the menu's shortcuts, but with
/// AltGr, which types characters).
fn type_search(
    mut menu: ResMut<FlightMenu>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    logical: Res<ButtonInput<Key>>,
) {
    if !menu.open || !menu.guide || menu.bindings {
        events.clear();
        return;
    }
    let mods = [KeyCode::AltLeft, KeyCode::AltRight, KeyCode::ControlLeft, KeyCode::ControlRight];
    let shortcut = keys.any_pressed(mods) && !logical.pressed(Key::AltGraph);
    for e in events.read().filter(|e| e.state == ButtonState::Pressed) {
        if e.logical_key == Key::Backspace {
            pop_search(&mut menu);
        } else if !shortcut && let Some(t) = &e.text {
            for ch in t.chars().filter(|&ch| stroke_has(capital(ch))) {
                push_search(&mut menu, ch);
            }
        }
    }
}

/// The key guide scrolls: arrow keys (held), PgUp / PgDn, the mouse wheel, the gamepad d-pad,
/// a VR thumbstick.
fn scroll_guide(
    mut menu: ResMut<FlightMenu>,
    keys: Res<ButtonInput<KeyCode>>,
    wheel: Res<AccumulatedMouseScroll>,
    gamepads: Query<&Gamepad>,
    hands: Res<Hands>,
    time: Res<Time<Real>>,
) {
    if !menu.open || !menu.guide {
        return;
    }
    let held =
        |k: KeyCode, b: GamepadButton| keys.pressed(k) || gamepads.iter().any(|g| g.pressed(b));
    let mut speed = held(KeyCode::ArrowDown, GamepadButton::DPadDown) as i8 as f32
        - held(KeyCode::ArrowUp, GamepadButton::DPadUp) as i8 as f32;
    for hand in hands.hands.iter().filter(|h| h.active && h.thumb.y.abs() > 0.15) {
        speed -= hand.thumb.y;
    }
    let area = guide_area(menu.keyboard);
    let page = area[3] - area[1] - 40.0;
    let mut d = speed * crate::key_guide::SCROLL_SPEED * time.delta_secs()
        - crate::key_guide::wheel_notches(&wheel) * crate::key_guide::WHEEL_STEP;
    if keys.just_pressed(KeyCode::PageDown) {
        d += page;
    }
    if keys.just_pressed(KeyCode::PageUp) {
        d -= page;
    }
    if d != 0.0 {
        // Kept within the list when it is drawn.
        menu.guide_scroll = (menu.guide_scroll + d).max(0.0);
    }
}

/// Keyboard, gamepad, VR thumbsticks and buttons, VR fingertips and the mouse.
#[allow(clippy::too_many_arguments)]
fn navigate(
    mut menu: ResMut<FlightMenu>,
    mut settings: ResMut<Settings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    mut hands: ResMut<Hands>,
    windows: Query<&Window>,
    cams: Query<(&Camera, &GlobalTransform), With<EyeCamera>>,
    screen: Query<(&GlobalTransform, &Transform), With<MenuScreen>>,
    overlay: Query<&RelativeCursorPosition, With<MenuScreen>>,
    mut time: ResMut<Time<Virtual>>,
    mut block: ResMut<InputBlock>,
    mut actions: MessageWriter<ActionEvent>,
    sinks: Query<&AudioSink>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    mut dialog: ResMut<crate::input_dialog::InputDialog>,
) {
    if !menu.open || menu.exiting || menu.bindings {
        return;
    }
    let ops = campaign.as_deref().is_some_and(|c| c.screen_open());
    let pause = campaign.is_none();
    if !mouse.pressed(MouseButton::Left) {
        menu.mouse_armed = true;
    }
    let click = menu.mouse_armed && mouse.just_pressed(MouseButton::Left);
    let items = menu.page.items();
    let n = items.len();
    let mut step = 0i32;
    let mut adjust = 0i32;
    let mut select = false;
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    if keys.just_pressed(KeyCode::ArrowUp) || pad(GamepadButton::DPadUp) {
        step -= 1;
    }
    if keys.just_pressed(KeyCode::ArrowDown) || pad(GamepadButton::DPadDown) {
        step += 1;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || pad(GamepadButton::DPadLeft) {
        adjust -= 1;
    }
    if keys.just_pressed(KeyCode::ArrowRight) || pad(GamepadButton::DPadRight) {
        adjust += 1;
    }
    if (keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space])
        && !crate::fullscreen::alt_enter(&keys))
        || pad(GamepadButton::South)
    {
        select = true;
    }
    let back = pad(GamepadButton::East);
    for hand in hands.hands.iter().filter(|h| h.active) {
        if hand.just_pressed(BUTTON_STICK_UP) {
            step -= 1;
        }
        if hand.just_pressed(BUTTON_STICK_DOWN) {
            step += 1;
        }
        if hand.just_pressed(BUTTON_STICK_LEFT) {
            adjust -= 1;
        }
        if hand.just_pressed(BUTTON_STICK_RIGHT) {
            adjust += 1;
        }
        if hand.just_pressed(BUTTON_PRIMARY) {
            select = true;
        }
    }
    // Desktop overlay: the canvas pixel under the cursor (normalized: -0.5 .. 0.5 from its
    // centre).
    let pointer = overlay
        .single()
        .ok()
        .filter(|rel| rel.cursor_over())
        .and_then(|rel| rel.normalized)
        .map(|n| Vec2::new((n.x + 0.5) * W as f32, (n.y + 0.5) * H as f32));
    // The key guide: anything that presses an entry goes back to the entries, except on the
    // search: the keyboard types into it (Enter and Space stay there), a click or a fingertip
    // and the trigger on its X clear it, on the field open the VR keyboard, on a key of that
    // press it.
    if menu.guide {
        let (empty, keyboard) = (menu.search.is_empty(), menu.keyboard);
        let mut hover = pointer.map(|p| spot_at(p, empty, keyboard));
        // A click off the overlay goes back too.
        let mut pressed = click.then(|| hover.unwrap_or(Spot::List));
        let mut trigger_away = false;
        if let Ok((_, screen_tf)) = screen.single() {
            // Fingertips are in the cockpit frame, the screen's parent.
            let local_from_cockpit = screen_tf.compute_affine().inverse();
            for hand in hands.hands.iter_mut().filter(|h| h.active) {
                let l = local_from_cockpit.transform_point3(hand.finger);
                let spot = (l.z < TOUCH_FRONT && l.z > -TOUCH_BEHIND)
                    .then(|| to_canvas(l))
                    .flatten()
                    .map(|p| spot_at(p, empty, keyboard));
                if let Some(spot) = spot {
                    let control = matches!(spot, Spot::Clear | Spot::Field | Spot::Key(_));
                    if control && menu.hover != Some(spot) {
                        hand.haptic = hand.haptic.max(0.1);
                    }
                    hover = hover.or(Some(spot));
                }
                if hand.trigger_pulled() {
                    match spot {
                        Some(spot) => {
                            hand.haptic = hand.haptic.max(0.6);
                            pressed = Some(spot);
                        }
                        None => trigger_away = true,
                    }
                }
            }
        }
        menu.hover = hover;
        let primary = pad(GamepadButton::South)
            || hands.hands.iter().any(|h| h.active && h.just_pressed(BUTTON_PRIMARY));
        match pressed {
            Some(Spot::Clear) => clear_search(&mut menu),
            Some(Spot::Field) => {
                menu.focused = true;
                menu.keyboard |= settings.vr;
            }
            Some(Spot::Key(key)) => press_key(&mut menu, key),
            Some(Spot::Keyboard) => {}
            Some(Spot::List) => menu.guide = false,
            None if primary || back || trigger_away => menu.guide = false,
            None => {}
        }
        return;
    }
    if step != 0 {
        menu.cursor = (menu.cursor as i32 + step).rem_euclid(n as i32) as usize;
    }

    // Pointing: the mouse hovers and clicks, a fingertip touches and the trigger presses.
    let mut clicked: Option<(usize, bool)> = None;
    if let Some(hit) = pointer.and_then(|p| item_at(p, menu.page)) {
        menu.cursor = hit.0;
        if click {
            clicked = Some(hit);
        }
    }
    if let Ok((screen_gtf, screen_tf)) = screen.single() {
        let inv = screen_gtf.affine().inverse();
        if !settings.vr
            && let (Ok(window), Ok((camera, cam_tf))) = (windows.single(), cams.single())
            && let Some(cursor) = window.cursor_position()
            && let Ok(ray) = camera.viewport_to_world(cam_tf, cursor)
        {
            let o = inv.transform_point3(ray.origin);
            let d = inv.transform_vector3(*ray.direction);
            if d.z.abs() > 1e-6 {
                let t = -o.z / d.z;
                if let Some(hit) = (t > 0.0)
                    .then(|| to_canvas(o + d * t))
                    .flatten()
                    .and_then(|p| item_at(p, menu.page))
                {
                    menu.cursor = hit.0;
                    if click {
                        clicked = Some(hit);
                    }
                }
            }
        }
        // Fingertips are in the cockpit frame, the screen's parent.
        let local_from_cockpit = screen_tf.compute_affine().inverse();
        for hand in hands.hands.iter_mut().filter(|h| h.active) {
            let l = local_from_cockpit.transform_point3(hand.finger);
            let hit = (l.z < TOUCH_FRONT && l.z > -TOUCH_BEHIND)
                .then(|| to_canvas(l))
                .flatten()
                .and_then(|p| item_at(p, menu.page));
            if let Some(hit) = hit {
                if menu.cursor != hit.0 {
                    hand.haptic = hand.haptic.max(0.1);
                }
                menu.cursor = hit.0;
            }
            // Away from the screen, not while the finger is on a cockpit control (the MENU
            // button that opens and closes the menu).
            if hand.trigger_pulled() && (hit.is_some() || !hand.touching) {
                hand.haptic = hand.haptic.max(0.6);
                match hit {
                    Some(hit) => clicked = Some(hit),
                    None => select = true,
                }
            }
        }
    }

    let (index, right) = match clicked {
        Some(hit) => (hit.0, Some(hit.1)),
        None => (menu.cursor, None),
    };
    let item = items[index.min(n - 1)];
    if !item.enabled(&settings, ops, menu.can_save) || !(select || clicked.is_some() || adjust != 0)
    {
        // B / the gamepad's East button: back a page, or close the menu.
        if back {
            match menu.page.back(menu.options_only) {
                Some(parent) => go_to(&mut menu, parent),
                None => set_open(&mut menu, false, pause, &settings, &mut time, &mut block, &sinks),
            }
        }
        return;
    }
    // Left / right only change values; they don't press the other entries.
    if adjust != 0 && clicked.is_none() && !select && !item.adjustable() {
        return;
    }
    // Values step left or right: the arrows, the half of the row clicked, else up.
    let dir = if adjust != 0 {
        adjust.signum()
    } else if right == Some(false) {
        -1
    } else {
        1
    } as f32;
    let mut close = false;
    let s = &mut *settings;
    match item {
        Item::Shaders => s.modern_shading = !s.modern_shading,
        Item::Msaa => s.msaa = !s.msaa,
        Item::Shadows => s.shadows = !s.shadows,
        Item::Lighting => s.local_lights = !s.local_lights,
        Item::AutoDetail => s.vr_auto_detail = !s.vr_auto_detail,
        Item::TimeOfDay => s.continuous_time = !s.continuous_time,
        Item::SunMoon => s.sun_moon = !s.sun_moon,
        Item::Cockpit => s.classic_cockpit = !s.classic_cockpit,
        Item::Hud => s.classic_hud = !s.classic_hud,
        Item::Avionics => s.classic_avionics = !s.classic_avionics,
        Item::AutoTrim => s.auto_trim = !s.auto_trim,
        Item::Tooltips => s.tooltips = !s.tooltips,
        Item::Atc => s.atc = !s.atc,
        Item::PlayerLabels => s.player_labels = !s.player_labels,
        Item::Effect(e) => e.toggle(&mut s.effects),
        Item::LowPass => s.engine_low_pass = !s.engine_low_pass,
        Item::RwrTones => s.rwr_tones = s.rwr_tones.step(dir as i32),
        Item::HeadTracking => s.head_tracking = !s.head_tracking,
        Item::TrackingMouse => s.tracking_mouse_look = !s.tracking_mouse_look,
        Item::Bindings => {
            menu.bindings = true;
            dialog.open = true;
        }
        Item::CockpitSize => {
            s.vr_cockpit_scale = (s.vr_cockpit_scale + 0.05 * dir)
                .clamp(*VR_COCKPIT_SCALE_RANGE.start(), *VR_COCKPIT_SCALE_RANGE.end());
        }
        Item::PixelSize => {
            let blocks = crate::effects::PIXEL_BLOCKS;
            s.effects.pixel_block = s
                .effects
                .pixel_block
                .saturating_add_signed(dir as i32)
                .clamp(*blocks.start(), *blocks.end());
        }
        Item::LabelSize => {
            s.player_label_size = (s.player_label_size + 0.1 * dir)
                .clamp(*PLAYER_LABEL_SIZE_RANGE.start(), *PLAYER_LABEL_SIZE_RANGE.end());
        }
        Item::Volume(v) => {
            let v = v.get(s);
            *v = ((*v * 10.0).round() + dir).clamp(0.0, 10.0) / 10.0;
        }
        Item::Open(page) => go_to(&mut menu, page),
        Item::Back => match menu.page.back(menu.options_only) {
            Some(parent) => go_to(&mut menu, parent),
            None => close = true,
        },
        Item::KeyGuide => {
            menu.guide = true;
            menu.focused = false;
            menu.keyboard = false;
            clear_search(&mut menu);
        }
        Item::Resume => close = true,
        Item::SaveMission => menu.save_request = true,
        // A campaign sortie ends; the flight goes on with the campaign's board.
        Item::EndMission if !pause => {
            actions.write(ActionEvent(Action::Quit));
            close = true;
        }
        Item::EndMission => {
            // Stays open (paused and silent) showing EXITING... until the flight ends.
            menu.exiting = true;
            save_options(&mut menu, s);
            actions.write(ActionEvent(Action::Quit));
        }
    }
    if close {
        set_open(&mut menu, false, pause, &settings, &mut time, &mut block, &sinks);
    }
}

/// The camera (desktop) or the VR panel carrying the bindings dialog's egui context.
#[derive(Component, Clone)]
struct BindingsOverlay;

/// Puts up the bindings dialog's egui context while it is open (as the campaign's Ops screen
/// does, `campaign::ops::overlay`); its DONE goes back to the menu.
#[allow(clippy::too_many_arguments)]
fn bindings_overlay(
    mut commands: Commands,
    mut menu: ResMut<FlightMenu>,
    dialog: Res<crate::input_dialog::InputDialog>,
    tagged: Query<Entity, With<BindingsOverlay>>,
    mut cams: Query<&mut Msaa, With<BindingsOverlay>>,
    eye: Query<&Msaa, (With<EyeCamera>, Without<BindingsOverlay>)>,
    settings: Res<Settings>,
    root: Option<Res<crate::camera::XrRoot>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if menu.bindings && !dialog.open {
        menu.bindings = false;
    }
    let want = menu.open && menu.bindings;
    // The overlay draws into the 3D view's picture only with the same anti-aliasing.
    let msaa = eye.iter().next().copied().unwrap_or(Msaa::Off);
    for mut m in &mut cams {
        if *m != msaa {
            *m = msaa;
        }
    }
    if want == !tagged.is_empty() {
        return;
    }
    if !want {
        for e in &tagged {
            commands.entity(e).try_despawn();
        }
        #[cfg(feature = "xr")]
        commands.remove_resource::<crate::vr_menu::WorldPanel>();
        return;
    }
    #[cfg(feature = "xr")]
    if settings.vr {
        let root = root.and_then(|r| r.0);
        crate::vr_menu::spawn(
            &mut commands,
            &mut images,
            &mut meshes,
            &mut materials,
            root,
            BindingsOverlay,
            false,
        );
        commands.insert_resource(crate::vr_menu::WorldPanel);
        return;
    }
    let _ = (&settings, &root, &mut images, &mut meshes, &mut materials);
    commands.spawn((
        Camera2d,
        Camera { order: 50, clear_color: ClearColorConfig::None, ..default() },
        msaa,
        BindingsOverlay,
        bevy_egui::PrimaryEguiContext,
    ));
}

/// The bindings dialog in flight.
fn bindings_ui(
    mut contexts: bevy_egui::EguiContexts,
    menu: Res<FlightMenu>,
    mut dialog: ResMut<crate::input_dialog::InputDialog>,
    mut config: ResMut<crate::hotas::InputConfig>,
    hotas: Res<crate::hotas::Hotas>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Settings>,
) -> Result {
    if !menu.bindings {
        return Ok(());
    }
    // The overlay's context comes up a frame after the dialog opens.
    let Ok(ctx) = contexts.ctx_mut() else { return Ok(()) };
    crate::menu::apply_ui_zoom(ctx);
    crate::menu::add_icon_font(ctx);
    if ctx.global_style().visuals.window_fill != crate::menu::BG {
        ctx.all_styles_mut(crate::menu::apply_terminal_style);
    }
    let config = config.bypass_change_detection();
    crate::input_dialog::show(ctx, &mut dialog, config, &hotas, &keys, settings.vr);
    Ok(())
}

fn draw(
    mut menu: ResMut<FlightMenu>,
    settings: Res<Settings>,
    mut q: Query<(&mut CanvasTexture, &mut Visibility), With<MenuScreen>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    config: Res<crate::hotas::InputConfig>,
    time: Res<Time<Real>>,
) {
    let Ok((mut tex, mut vis)) = q.single_mut() else { return };
    let shown = menu.open && !menu.bindings;
    vis.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
    if !shown {
        return;
    }
    let ops = campaign.as_deref().is_some_and(|c| c.screen_open());
    let c = &mut tex.canvas;
    c.clear(TRANSPARENT);
    // The key on the footer's BACK (in VR an arrow).
    let key = if settings.vr { "<" } else { "ESC" };
    let main_title = if campaign.is_some() { "MENU" } else { "PAUSE" };
    if menu.exiting {
        draw_frame(c, "", main_title, "", None);
        let mid = (TOP + BOTTOM) / 2.0;
        stroke_text_at(c, W as f32 / 2.0, mid, "EXITING...", 36.0, 4.0, ink::TEXT, Align::Centre);
        tex.dirty = true;
        return;
    }
    if menu.guide {
        let hint = match (settings.vr, menu.keyboard) {
            (true, false) => "STICK: SCROLL   FIELD + TRIGGER: KEYBOARD",
            (true, true) => "KEY + TRIGGER: TYPE   STICK: SCROLL",
            (false, _) => "WHEEL, ARROWS: SCROLL",
        };
        draw_frame(c, "", "KEY GUIDE", hint, Some((key, false)));
        let area = guide_area(menu.keyboard);
        let caret = crate::key_guide::caret_shown(time.elapsed_secs_f64());
        let focused = menu.focused || menu.keyboard;
        draw_search(c, &menu.search, focused, caret, menu.hover == Some(Spot::Clear));
        if menu.keyboard {
            draw_keyboard(c, menu.hover);
        }
        let lines =
            crate::key_guide::filter(crate::key_guide::lines(&config, settings.vr), &menu.search);
        if lines.is_empty() {
            let mid = (area[1] + area[3]) / 2.0;
            stroke_text_at(c, W as f32 / 2.0, mid, "NO MATCH", 18.0, 2.4, ink::OFF, Align::Centre);
        }
        let style = crate::key_guide::Style {
            size: 0.72,
            heading: ink::LIT_TEXT,
            bar: ink::TAB,
            keys: ink::TEXT,
            text: ink::shade(74),
            dim: ink::OFF,
        };
        let mut scroll = menu.guide_scroll;
        crate::key_guide::draw(c, &lines, area, &mut scroll, &style);
        menu.guide_scroll = scroll;
        tex.dirty = true;
        return;
    }
    let (crumb, title) = match menu.page {
        Page::Main => ("", main_title),
        Page::Options => ("", "OPTIONS"),
        page => ("OPTIONS >", page.title()),
    };
    // What only the main menu's OPTIONS change (not in the middle of a flight).
    let note = match menu.page {
        Page::Main if campaign.is_some() => "GAME IN PROGRESS",
        Page::Game => "FLIGHT MODEL, TACVIEW: MAIN MENU OPTIONS",
        _ => "",
    };
    let items = menu.page.items();
    let back_lit = items.get(menu.cursor) == Some(&Item::Back);
    draw_frame(c, crumb, title, note, Some((key, back_lit)));
    // The main page's entries and the pages of OPTIONS have an icon; the ones that open
    // another screen (a page, the key guide, the bindings dialog) also have a chevron.
    let icons = matches!(menu.page, Page::Main | Page::Options);
    for (i, (item, r)) in items.iter().zip(item_rects(menu.page)).enumerate() {
        if *item == Item::Back {
            continue;
        }
        let [x0, y0, x1, y1] = r;
        let (h, mid) = (y1 - y0, (y0 + y1) / 2.0);
        let lit = i == menu.cursor;
        draw_row(c, r, lit);
        let col = if !item.enabled(&settings, ops, menu.can_save) {
            ink::OFF
        } else if lit {
            ink::LIT_TEXT
        } else {
            ink::TEXT
        };
        let mut x = x0 + 20.0;
        if icons {
            let cell = (h * 1.3).round();
            let line = if lit { ink::LIT_GLOW } else { ink::FRAME };
            c.stroke(Vec2::new(x0 + cell, y0 + 1.0), Vec2::new(x0 + cell, y1 - 1.0), 1.2, line);
            draw_icon(c, *item, Vec2::new(x0 + cell / 2.0, mid), h * 0.3, col);
            x = x0 + cell + 26.0;
        }
        let size = (h * 0.6).min(20.0);
        let pen = size * 0.14;
        let label = match item {
            Item::EndMission if campaign.is_some() => "END SORTIE",
            _ => item.label(),
        };
        stroke_text_at(c, x, mid, label, size, pen, col, Align::Left);
        let mut right = x1 - 18.0;
        if matches!(item, Item::Open(_) | Item::KeyGuide | Item::Bindings) {
            let (tip, s) = (Vec2::new(x1 - 24.0, mid), (h * 0.17).min(10.0));
            let arms = [tip + Vec2::new(-0.65 * s, -s), tip, tip + Vec2::new(-0.65 * s, s)];
            c.polyline(&arms, pen.max(1.8), col);
            right = x1 - 48.0;
        }
        let v = match item {
            Item::SaveMission => menu.save_note.unwrap_or_default().to_string(),
            _ => item.value(&settings),
        };
        if !v.is_empty() {
            stroke_text_at(c, right, mid, &v, size, pen, col, Align::Right);
        }
    }
    tex.dirty = true;
}

/// The panel's outline `inset` units inside its edge, its corners cut.
fn outline(inset: f32) -> [Vec2; 8] {
    let (l, t) = (EDGE + inset, EDGE + inset);
    let (r, b) = (W as f32 - EDGE - inset, H as f32 - EDGE - inset);
    let k = CUT - inset * 0.41;
    [(l + k, t), (r - k, t), (r, t + k), (r, b - k), (r - k, b), (l + k, b), (l, b - k), (l, t + k)]
        .map(|(x, y)| Vec2::new(x, y))
}

/// A convex polygon, filled.
fn fill_poly(c: &mut Canvas, pts: &[Vec2], col: u8) {
    for i in 1..pts.len().saturating_sub(1) {
        c.triangle([pts[0], pts[i], pts[i + 1]], col);
    }
}

/// A slanted bar from its bottom-left corner: `w` wide, `h` tall, its top `lean` to the right
/// (to the left if negative).
fn slash(c: &mut Canvas, at: Vec2, w: f32, h: f32, lean: f32, col: u8) {
    let quad = [Vec2::ZERO, Vec2::new(w, 0.0), Vec2::new(w + lean, -h), Vec2::new(lean, -h)];
    fill_poly(c, &quad.map(|p| at + p), col);
}

/// A rectangle filled from `top` at its top to `bottom` at its bottom (palette ramp indices).
fn fill_gradient(c: &mut Canvas, [x0, y0, x1, y1]: [f32; 4], top: u8, bottom: u8) {
    let (ya, yb) = (y0.round() as i32, y1.round() as i32);
    let (xa, w) = (x0.round() as i32, (x1 - x0).round() as i32);
    for y in ya..yb {
        let t = (y - ya) as f32 / (yb - ya - 1).max(1) as f32;
        c.fill_rect(xa, y, w, 1, (top as f32 + (bottom as f32 - top as f32) * t).round() as u8);
    }
}

/// The panel: its frame and glow, the title in the tab at the top-left (after `crumb`, small),
/// the game's name at the top-right, `note` and the BACK button (its key, whether it is
/// highlighted) in the footer.
fn draw_frame(c: &mut Canvas, crumb: &str, title: &str, note: &str, back: Option<(&str, bool)>) {
    const TITLE: f32 = 28.0;
    const CRUMB: f32 = 14.0;
    let panel = outline(0.0);
    fill_poly(c, &panel, ink::PANEL);
    let tx = 76.0;
    let crumb_w = if crumb.is_empty() { 0.0 } else { stroke_width(crumb, CRUMB) + 16.0 };
    let tab = tx + crumb_w + stroke_width(title, TITLE) + 34.0 + SLANT;
    let tab_shape = [
        Vec2::new(EDGE + CUT, EDGE),
        Vec2::new(tab, EDGE),
        Vec2::new(tab - SLANT, HEADER_Y),
        Vec2::new(EDGE, HEADER_Y),
        Vec2::new(EDGE, EDGE + CUT),
    ];
    fill_poly(c, &tab_shape, ink::TAB);
    c.stroke(tab_shape[1], tab_shape[2], 1.6, ink::LINE);
    let step = W as f32 - 200.0;
    c.polyline(
        &[
            Vec2::new(EDGE, HEADER_Y),
            Vec2::new(step, HEADER_Y),
            Vec2::new(step + STEP, HEADER_Y - STEP),
            Vec2::new(W as f32 - EDGE, HEADER_Y - STEP),
        ],
        1.6,
        ink::LINE,
    );
    for i in 0..2 {
        slash(c, Vec2::new(24.0 + 16.0 * i as f32, 50.0), 9.0, 28.0, 12.0, ink::TEXT);
    }
    let mid = (EDGE + HEADER_Y) / 2.0 + 1.0;
    if !crumb.is_empty() {
        let y = mid + (TITLE - CRUMB) / 2.0;
        stroke_text_at(c, tx, y, crumb, CRUMB, 1.8, ink::NOTE, Align::Left);
    }
    stroke_text_at(c, tx + crumb_w, mid, title, TITLE, 3.4, ink::LIT_TEXT, Align::Left);
    stroke_text_at(
        c,
        W as f32 - 72.0,
        26.0,
        "STEALTH MISSION 89",
        11.0,
        1.6,
        ink::TEXT,
        Align::Right,
    );
    for i in 0..2 {
        slash(c, Vec2::new(W as f32 - 48.0 + 14.0 * i as f32, 36.0), 9.0, 22.0, -12.0, ink::TEXT);
    }

    let step = W as f32 - 210.0;
    c.polyline(
        &[
            Vec2::new(22.0, FOOT_Y),
            Vec2::new(step, FOOT_Y),
            Vec2::new(step + STEP, FOOT_Y - STEP),
            Vec2::new(W as f32 - 22.0, FOOT_Y - STEP),
        ],
        1.6,
        ink::LINE,
    );
    for i in 0..3 {
        slash(c, Vec2::new(24.0 + 14.0 * i as f32, 492.0), 8.0, 18.0, 10.0, ink::LINE);
    }
    stroke_text_at(c, 90.0, 483.0, note, 11.0, 1.6, ink::NOTE, Align::Left);
    if let Some((key, lit)) = back {
        let [x0, y0, _, y1] = BACK;
        if lit {
            draw_row(c, BACK, true);
        }
        let col = if lit { ink::LIT_TEXT } else { ink::TEXT };
        let mid = (y0 + y1) / 2.0;
        let (k0, k1) = (x0 + 6.0, x0 + 56.0);
        c.frame(Vec2::new(k0, y0 + 4.0), Vec2::new(k1, y1 - 4.0), 1.6, col);
        stroke_text_at(c, (k0 + k1) / 2.0, mid, key, 13.0, 1.8, col, Align::Centre);
        stroke_text_at(c, k1 + 14.0, mid, "BACK", 17.0, 2.3, col, Align::Left);
    }

    let mut edge = panel.to_vec();
    edge.push(panel[0]);
    c.polyline(&edge, 6.0, ink::GLOW);
    c.polyline(&edge, 2.4, ink::EDGE);
}

/// An entry's framed row, lit up (a gradient, a glowing frame) while highlighted.
fn draw_row(c: &mut Canvas, r: [f32; 4], lit: bool) {
    let [x0, y0, x1, y1] = r;
    let (a, b) = (Vec2::new(x0, y0), Vec2::new(x1, y1));
    if lit {
        fill_gradient(c, r, ink::LIT_TOP, ink::LIT_BOTTOM);
        c.frame(a, b, 5.0, ink::shade(24));
        c.frame(a, b, 3.0, ink::LIT_GLOW);
        c.frame(a, b, 1.6, ink::EDGE);
    } else {
        fill_gradient(c, r, ink::ROW, ink::ROW);
        c.frame(a, b, 1.2, ink::FRAME);
    }
}

/// The menus' icons: Lucide's font (`assets/fonts/lucide.ttf`, ISC licence, beside it), also
/// the egui menus' (`crate::menu::add_icon_font`).
pub(crate) static ICON_FONT: std::sync::LazyLock<Option<&'static [u8]>> =
    std::sync::LazyLock::new(|| {
        let bytes = crate::asset_path::read("fonts/lucide.ttf")
            .map_err(|e| warn!("menu icons: {e:#}"))
            .ok()?;
        Some(Box::leak(bytes.into_boxed_slice()))
    });

/// Its glyph outlines, for this menu's canvas.
static ICONS: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
    std::sync::LazyLock::new(|| ttf_parser::Face::parse((*ICON_FONT)?, 0).ok());

/// The Lucide icon (its code point) of an entry of the main page or of OPTIONS, those of the
/// menus' buttons of the same name (`crate::menu::icon`).
fn icon(item: Item) -> Option<char> {
    use crate::menu::icon;
    Some(match item {
        Item::Resume => '\u{e13c}', // play
        Item::SaveMission => icon::SAVE,
        Item::Open(Page::Options) => icon::OPTIONS,
        Item::KeyGuide => '\u{e05f}', // book-open
        Item::EndMission => icon::END_SORTIE,
        Item::Open(Page::Game) => '\u{e1de}',     // plane
        Item::Open(Page::Graphics) => '\u{e11d}', // monitor
        Item::Open(Page::Effects) => '\u{e412}',  // sparkles
        Item::Open(Page::Sound) => '\u{e1ab}',    // volume-2
        Item::Open(Page::Controls) => icon::KEYS,
        _ => return None,
    })
}

/// A glyph's outline as closed polylines (its curves in short segments), in font units.
#[derive(Default)]
struct Outline(Vec<Vec<Vec2>>);

impl Outline {
    /// Segments per curve.
    const STEPS: usize = 8;

    fn last(&self) -> Vec2 {
        self.0.last().and_then(|c| c.last()).copied().unwrap_or_default()
    }

    fn push(&mut self, p: Vec2) {
        if let Some(c) = self.0.last_mut() {
            c.push(p);
        }
    }
}

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(vec![Vec2::new(x, y)]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push(Vec2::new(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (p0, p1, p2) = (self.last(), Vec2::new(x1, y1), Vec2::new(x, y));
        for i in 1..=Self::STEPS {
            let t = i as f32 / Self::STEPS as f32;
            self.push(p0.lerp(p1, t).lerp(p1.lerp(p2, t), t));
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (p0, p1, p2, p3) = (self.last(), Vec2::new(x1, y1), Vec2::new(x2, y2), Vec2::new(x, y));
        for i in 1..=Self::STEPS {
            let t = i as f32 / Self::STEPS as f32;
            let (a, b, c) = (p0.lerp(p1, t), p1.lerp(p2, t), p2.lerp(p3, t));
            self.push(a.lerp(b, t).lerp(b.lerp(c, t), t));
        }
    }

    fn close(&mut self) {}
}

/// The icon of an entry of the main page or of OPTIONS centred on `o`, the font's em `2 r`
/// wide (the same stroke for every icon), its edges blended into the colour under it (the
/// menu's colours are a ramp).
fn draw_icon(c: &mut Canvas, item: Item, o: Vec2, r: f32, col: u8) {
    let Some(face) = ICONS.as_ref() else { return };
    let Some(id) = icon(item).and_then(|ch| face.glyph_index(ch)) else { return };
    let mut outline = Outline::default();
    let Some(b) = face.outline_glyph(id, &mut outline) else { return };
    let k = 2.0 * r / face.units_per_em() as f32;
    let mid = Vec2::new((b.x_min + b.x_max) as f32, (b.y_min + b.y_max) as f32) / 2.0;
    // Font units point up.
    let contours: Vec<Vec<Vec2>> = outline
        .0
        .into_iter()
        .map(|c| c.into_iter().map(|p| o + (p - mid) * Vec2::new(k, -k)).collect())
        .collect();
    c.fill_contours(&contours, |under, share| {
        (under as f32 + (col as f32 - under as f32) * share).round() as u8
    });
}

/// The search's VR keyboard, the key under the fingertip lit up.
fn draw_keyboard(c: &mut Canvas, hover: Option<Spot>) {
    for (key, [x0, y0, x1, y1]) in keyboard_keys() {
        let (x, y, w, h) = (x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32);
        let lit = hover == Some(Spot::Key(key));
        c.fill_rect(x, y, w, h, if lit { ink::LIT_BOTTOM } else { ink::ROW });
        c.rect(x, y, w, h, if key == VKey::Done || lit { ink::EDGE } else { ink::FRAME });
        let centre = Vec2::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let col = if lit { ink::LIT_TEXT } else { ink::TEXT };
        stroke_text_at(c, centre.x, centre.y, &key.label(), 14.0, 2.0, col, Align::Centre);
    }
}

/// The key guide's search field: a magnifier, the text and its `_` caret while `caret` (a hint
/// while empty), the clear button's X while there is text (lit up under the mouse or a
/// fingertip).
fn draw_search(c: &mut Canvas, text: &str, focused: bool, caret: bool, clear_hover: bool) {
    let [x0, y0, x1, y1] = SEARCH;
    let mid = (y0 + y1) / 2.0;
    let col = if text.is_empty() { ink::NOTE } else { ink::TEXT };
    let (x, y, w, h) = (x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32);
    c.fill_rect(x, y, w, h, ink::ROW);
    c.rect(x, y, w, h, if text.is_empty() { ink::FRAME } else { ink::EDGE });
    let lens = Vec2::new(x0 + 15.0, mid - 2.0);
    c.ring(lens, 6.0, 2.0, col);
    c.stroke(lens + Vec2::splat(4.5), lens + Vec2::splat(9.5), 2.4, col);
    let tx = x0 + 34.0;
    // Once clicked or typed into, the hint makes way for the caret.
    if text.is_empty() {
        if !focused {
            stroke_text_at(c, tx, mid, "TYPE TO SEARCH", 14.0, 2.0, ink::NOTE, Align::Left);
        } else if caret {
            stroke_text_at(c, tx, mid, "_", 14.0, 2.0, ink::TEXT, Align::Left);
        }
        return;
    }
    let end = stroke_text_at(c, tx, mid, text, 14.0, 2.0, ink::TEXT, Align::Left);
    if caret {
        stroke_text_at(c, end, mid, "_", 14.0, 2.0, ink::TEXT, Align::Left);
    }
    let [bx0, by0, bx1, by1] = CLEAR;
    if clear_hover {
        c.fill_rect(
            bx0 as i32 + 3,
            by0 as i32 + 3,
            (bx1 - bx0) as i32 - 6,
            (by1 - by0) as i32 - 6,
            ink::LIT_BOTTOM,
        );
    }
    let (cx, r) = ((bx0 + bx1) / 2.0, 6.0);
    c.stroke(Vec2::new(cx - r, mid - r), Vec2::new(cx + r, mid + r), 2.4, ink::LIT_TEXT);
    c.stroke(Vec2::new(cx - r, mid + r), Vec2::new(cx + r, mid - r), 2.4, ink::LIT_TEXT);
}

/// VR: the end-of-mission overlay on a screen in front of the head (a child of the XR tracking
/// root, so it stays in view in every camera view). Not after END MISSION, which goes straight
/// to the debrief.
#[allow(clippy::too_many_arguments)]
fn vr_end_screen(
    mut commands: Commands,
    settings: Res<Settings>,
    menu: Res<FlightMenu>,
    combat: Option<Res<crate::combat::Combat>>,
    roots: Query<Entity, With<EyeCamera>>,
    cams: Query<(&ChildOf, &Transform), With<Camera>>,
    screens: Query<(), With<EndScreen>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::cockpit3d::tooltips::TipScreenMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !settings.vr || menu.exiting || !screens.is_empty() {
        return;
    }
    let Some(end) = combat.as_ref().and_then(|c| c.end.as_ref()) else { return };
    let Ok(root) = roots.single() else { return };
    // The head pose in the tracking space: the headset cameras' (the root's children).
    let eyes: Vec<&Transform> =
        cams.iter().filter(|(p, _)| p.parent() == root).map(|(_, t)| t).collect();
    let head = eyes.iter().map(|t| t.translation).sum::<Vec3>() / eyes.len().max(1) as f32;
    let fwd = eyes.first().map_or(Vec3::NEG_Z, |t| t.forward().as_vec3());
    let flat = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or(Vec3::NEG_Z);
    let centre = head + flat * END_DISTANCE - Vec3::Y * 0.05;

    let mut canvas = Canvas::new(W, H);
    canvas.clear(BG);
    canvas.frame(Vec2::new(6.0, 6.0), Vec2::new(W as f32 - 7.0, H as f32 - 7.0), 3.0, DIM);
    // Each line as large as fits the width.
    let fit = |s: &str, h: f32| h.min((W as f32 - 64.0) / s.chars().count().max(1) as f32);
    let title = end.title.to_uppercase();
    stroke_text_at(
        &mut canvas,
        W as f32 / 2.0,
        64.0,
        &title,
        fit(&title, 32.0),
        3.6,
        BRIGHT,
        Align::Centre,
    );
    for (i, l) in end.lines.iter().enumerate() {
        let y = 140.0 + 42.0 * i as f32;
        stroke_text_at(&mut canvas, W as f32 / 2.0, y, l, fit(l, 20.0), 2.6, BRIGHT, Align::Centre);
    }
    let hint = "PULL THE TRIGGER";
    stroke_text_at(
        &mut canvas,
        W as f32 / 2.0,
        H as f32 - 50.0,
        hint,
        20.0,
        2.6,
        DIM,
        Align::Centre,
    );
    let tex = CanvasTexture::new(canvas, &mut images, false);
    if let Some(mut img) = images.get_mut(&tex.image) {
        img.sampler = bevy::image::ImageSampler::linear();
    }
    // Drawn over everything, the cockpit in front of it included (as a VR tooltip): last of
    // the transparent things, without the depth test.
    let material = materials.add(bevy::pbr::ExtendedMaterial {
        base: StandardMaterial {
            base_color_texture: Some(tex.image.clone()),
            alpha_mode: AlphaMode::Blend,
            depth_bias: 1000.0,
            unlit: true,
            cull_mode: None,
            fog_enabled: false,
            ..default()
        },
        extension: crate::cockpit3d::tooltips::OnTop {},
    });
    let screen = commands
        .spawn((
            EndScreen,
            tex,
            Mesh3d(meshes.add(Rectangle::new(END_SIZE.x, END_SIZE.y))),
            MeshMaterial3d(material),
            Transform::from_translation(centre).looking_to(flat, Vec3::Y),
            NotShadowCaster,
            DespawnOnExit(AppState::Flight),
        ))
        .id();
    commands.entity(root).add_child(screen);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The VR keyboard fits under the search's list, its keys apart, each found at its middle;
    /// the field's X only counts while there is text.
    #[test]
    fn keyboard_layout() {
        let keys = keyboard_keys();
        assert_eq!(keys.len(), 44);
        for (i, &(key, r)) in keys.iter().enumerate() {
            assert!(r[0] >= 8.0 && r[2] <= W as f32 - 8.0, "{key:?}");
            assert!(r[1] >= guide_area(true)[3] && r[3] <= GUIDE[3], "{key:?}");
            assert!(
                keys[i + 1..].iter().all(|&(_, o)| {
                    o[0] >= r[2] || o[2] <= r[0] || o[1] >= r[3] || o[3] <= r[1]
                })
            );
            let middle = Vec2::new((r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0);
            assert_eq!(spot_at(middle, true, true), Spot::Key(key));
            assert_eq!(spot_at(middle, true, false), Spot::List);
        }
        let x = Vec2::new((CLEAR[0] + CLEAR[2]) / 2.0, (CLEAR[1] + CLEAR[3]) / 2.0);
        assert_eq!(spot_at(x, false, false), Spot::Clear);
        assert_eq!(spot_at(x, true, false), Spot::Field);
        let gap = Vec2::new(keys[0].1[2] + KEY_GAP / 2.0, keys[0].1[1] + 1.0);
        assert_eq!(spot_at(gap, true, true), Spot::Keyboard);
    }

    /// Every icon the menus write (`'\u{eXXX}'` here and in `crate::menu`) is in the cut-down
    /// Lucide font (`tools/subset_lucide.py`).
    #[test]
    fn icons_in_font() {
        let face = ICONS.as_ref().expect("assets/fonts/lucide.ttf");
        let mut icons = 0;
        for source in [include_str!("menu.rs"), include_str!("flight_menu.rs")] {
            for (i, _) in source.match_indices("\\u{") {
                let hex = &source[i + 3..];
                let Some(end) = hex.find('}') else { continue };
                let Ok(code) = u32::from_str_radix(&hex[..end], 16) else { continue };
                if (0xe000..=0xf8ff).contains(&code) {
                    let c = char::from_u32(code).unwrap();
                    assert!(face.glyph_index(c).is_some(), "U+{code:04X} not in lucide.ttf");
                    icons += 1;
                }
            }
        }
        assert!(icons > 40);
    }
}
