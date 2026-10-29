//! In-flight settings menu (Esc, Pause, gamepad Start or the MENU button of the left side panel):
//! a screen in front of the pilot, in the cockpit, so it works on the desktop and in VR. The
//! simulation pauses while it is open.
//!
//! The main page: RESUME, SAVE MISSION, OPTIONS, KEY GUIDE, END MISSION (set apart below a line);
//! RESUME and every page's BACK are a button with an arrow in the top-left corner. SAVE MISSION
//! saves a single mission's flight as it is (`savegame.rs`) and says SAVED beside it; a loaded
//! save starts with the menu open. OPTIONS has the pages of the main menu's OPTIONS window (`options.rs`) that
//! make sense in flight: GAME (cockpit, HUD and avionics style, auto trim, tooltips, the VR cockpit
//! size), GRAPHICS (shaders, MSAA, shadows, point lights, time of day), SOUND (volumes, the engine
//! low-pass) and CONTROLS (head tracking, the key and joystick bindings, the key guide). The
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
use crate::canvas::{Canvas, CanvasTexture};
use crate::cockpit3d::{
    BUTTON_PRIMARY, BUTTON_STICK_DOWN, BUTTON_STICK_LEFT, BUTTON_STICK_RIGHT, BUTTON_STICK_UP,
    CockpitRoot, Hands,
};
use crate::font::{Align, capital, stroke_has, stroke_text_at};
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
    /// The key guide's search (as typed), whether its VR keyboard is open, what of it the mouse
    /// or a fingertip is over.
    search: String,
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
    /// The entries, the first one (RESUME, BACK) in the top-left corner (`item_rects`).
    fn items(self) -> &'static [Item] {
        use Item::*;
        match self {
            Page::Main => &[Resume, SaveMission, Open(Page::Options), KeyGuide, EndMission],
            Page::Options => &[
                Back,
                Open(Page::Game),
                Open(Page::Graphics),
                Open(Page::Effects),
                Open(Page::Sound),
                Open(Page::Controls),
            ],
            Page::Game => &[
                Back,
                Cockpit,
                Hud,
                Avionics,
                AutoTrim,
                Tooltips,
                PlayerLabels,
                LabelSize,
                CockpitSize,
            ],
            Page::Graphics => {
                use crate::effects::Effect as E;
                &[
                    Back,
                    Shaders,
                    Msaa,
                    Shadows,
                    Lighting,
                    TimeOfDay,
                    SunMoon,
                    AutoDetail,
                    Effect(E::Vga),
                    Effect(E::Scanlines),
                ]
            }
            Page::Effects => {
                use crate::effects::Effect as E;
                &[
                    Back,
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
                ]
            }
            Page::Sound => &[
                Back,
                Volume(Vol::Master),
                Volume(Vol::Engine),
                Volume(Vol::Effects),
                Volume(Vol::Warnings),
                Volume(Vol::Rwr),
                RwrTones,
                LowPass,
            ],
            Page::Controls => &[Back, HeadTracking, Bindings, KeyGuide],
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
}

impl Vol {
    fn read(self, s: &Settings) -> f32 {
        match self {
            Vol::Master => s.master_volume,
            Vol::Engine => s.engine_volume,
            Vol::Effects => s.effects_volume,
            Vol::Warnings => s.warning_volume,
            Vol::Rwr => s.rwr_volume,
        }
    }

    fn get(self, s: &mut Settings) -> &mut f32 {
        match self {
            Vol::Master => &mut s.master_volume,
            Vol::Engine => &mut s.engine_volume,
            Vol::Effects => &mut s.effects_volume,
            Vol::Warnings => &mut s.warning_volume,
            Vol::Rwr => &mut s.rwr_volume,
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
    PlayerLabels,
    LabelSize,
    CockpitSize,
    /// An atmosphere effect's switch.
    Effect(crate::effects::Effect),
    Volume(Vol),
    /// Which radar warning tones sound.
    RwrTones,
    LowPass,
    HeadTracking,
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
            Item::PlayerLabels => "PLAYER LABELS",
            Item::LabelSize => "LABEL SIZE",
            Item::CockpitSize => "COCKPIT SIZE",
            Item::Effect(e) => e.menu_label(),
            Item::Volume(Vol::Master) => "MASTER VOLUME",
            Item::Volume(Vol::Engine) => "ENGINE",
            Item::Volume(Vol::Effects) => "EFFECTS",
            Item::Volume(Vol::Warnings) => "WARNINGS",
            Item::Volume(Vol::Rwr) => "RWR VOLUME",
            Item::RwrTones => "RWR TONES",
            Item::LowPass => "ENGINE LOW-PASS",
            Item::HeadTracking => "HEAD TRACKING",
            Item::Bindings => "KEYS AND JOYSTICK",
        }
    }

    fn value(self, s: &Settings) -> String {
        let style = |classic: bool| if classic { "CLASSIC" } else { "MODERN" }.to_string();
        let on = |b: bool| if b { "ON" } else { "OFF" }.to_string();
        match self {
            Item::Shaders => style(!s.modern_shading),
            Item::Msaa => on(s.msaa),
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
            Item::PlayerLabels => on(s.player_labels),
            Item::LabelSize => format!("< {:.0}% >", s.player_label_size * 100.0),
            Item::Effect(e) => on(e.get(&s.effects)),
            Item::CockpitSize => format!("< {:.0}% >", s.vr_cockpit_scale * 100.0),
            Item::Volume(v) => format!("< {:.0}% >", v.read(s) * 100.0),
            Item::RwrTones => format!("< {} >", s.rwr_tones.name()),
            Item::LowPass => on(s.engine_low_pass),
            Item::HeadTracking => on(s.head_tracking),
            Item::Open(_) | Item::KeyGuide | Item::Bindings => ">".to_string(),
            Item::Resume | Item::SaveMission | Item::EndMission | Item::Back => String::new(),
        }
    }

    /// The cockpit size and the automatic scenery detail only matter in the headset, the label
    /// size only with the player labels on, the cockpit style only for aircraft with
    /// two cockpits, the sun and moon switch only with the classic time of day, head tracking
    /// only on the desktop, MSAA not on WebGL2; the bindings not over the campaign's Ops screen
    /// (its egui context is the one up then); SAVE MISSION while the flight can be saved.
    fn enabled(self, s: &Settings, ops: bool, can_save: bool) -> bool {
        match self {
            Item::Bindings => !ops,
            Item::SaveMission => can_save,
            Item::CockpitSize | Item::AutoDetail => s.vr,
            Item::LabelSize => s.player_labels,
            Item::Cockpit => s.aircraft.spec().cockpits.len() > 1,
            Item::SunMoon => !s.continuous_time,
            Item::HeadTracking => !s.vr && !cfg!(target_arch = "wasm32"),
            Item::Msaa => !crate::WEBGL2,
            Item::Effect(crate::effects::Effect::Scanlines) => !s.vr,
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
/// Canvas pixels.
const W: usize = 768;
const H: usize = 512;
/// Rows of the entries, under the title.
const TOP: f32 = 116.0;
const ROW: f32 = 33.0;
const TITLE_Y: f32 = 80.0;
/// BACK and RESUME in the top-left corner (x0, y0, x1, y1).
const CORNER: [f32; 4] = [16.0, 14.0, 216.0, 48.0];
/// The space above END MISSION, a line in its middle.
const END_GAP: f32 = 22.0;
/// The key guide's area, below its search field.
const GUIDE: [f32; 4] = [24.0, 100.0, W as f32 - 24.0, H as f32 - 40.0];
/// The key guide's search field (x0, y0, x1, y1) and its clear button at the right end.
const SEARCH: [f32; 4] = [24.0, 56.0, W as f32 - 24.0, 86.0];
const CLEAR: [f32; 4] = [SEARCH[2] - 32.0, SEARCH[1], SEARCH[2], SEARCH[3]];
/// The search's VR keyboard: its keys' size and spacing, the top of its five rows (at the
/// bottom of the guide, above the hint).
const KEY_W: f32 = 64.0;
const KEY_H: f32 = 34.0;
const KEY_GAP: f32 = 6.0;
const KEYBOARD_TOP: f32 = GUIDE[3] - 5.0 * (KEY_H + KEY_GAP);
/// Palette colours: background, bright and dim green (the panel's lamp greens), dark row.
const BG: u8 = 5;
const BRIGHT: u8 = 47;
const DIM: u8 = 45;
const ROW_BG: u8 = 44;
/// A VR fingertip touches the screen from this far in front of it to this far behind it
/// (metres; a finger easily passes through the plane).
const TOUCH_FRONT: f32 = 0.05;
const TOUCH_BEHIND: f32 = 0.10;

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
    } else if pressed
        && menu.open
        && !menu.exiting
        && let Some(parent) = menu.page.back(menu.options_only)
    {
        go_to(&mut menu, parent);
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
    let tex = CanvasTexture::new(Canvas::new(W, H), &mut images, false);
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
        unlit: true,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    });
    let screen = commands
        .spawn((
            MenuScreen,
            tex,
            Mesh3d(meshes.add(Rectangle::new(SIZE.x, SIZE.y))),
            MeshMaterial3d(material),
            Transform::from_translation(POSITION),
            Visibility::Hidden,
            NotShadowCaster,
        ))
        .id();
    commands.entity(root).add_child(screen);
}

/// Canvas pixel of a point in the screen's local frame (None off the screen).
fn to_canvas(local: Vec3) -> Option<Vec2> {
    let u = local.x / SIZE.x + 0.5;
    let v = 0.5 - local.y / SIZE.y;
    ((0.0..1.0).contains(&u) && (0.0..1.0).contains(&v))
        .then(|| Vec2::new(u * W as f32, v * H as f32))
}

/// Where the entries of a page are (x0, y0, x1, y1): BACK and RESUME (which goes back to the
/// flight) in the top-left corner, the others in rows, END MISSION set apart below a line.
fn item_rects(page: Page) -> Vec<[f32; 4]> {
    let mut y = TOP;
    page.items()
        .iter()
        .map(|item| match item {
            Item::Back | Item::Resume => CORNER,
            _ => {
                if *item == Item::EndMission {
                    y += END_GAP;
                }
                y += ROW;
                [24.0, y - ROW, W as f32 - 24.0, y]
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
            Some(Spot::Field) => menu.keyboard |= settings.vr,
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
        Item::PlayerLabels => s.player_labels = !s.player_labels,
        Item::Effect(e) => e.toggle(&mut s.effects),
        Item::LowPass => s.engine_low_pass = !s.engine_low_pass,
        Item::RwrTones => s.rwr_tones = s.rwr_tones.step(dir as i32),
        Item::HeadTracking => s.head_tracking = !s.head_tracking,
        Item::Bindings => {
            menu.bindings = true;
            dialog.open = true;
        }
        Item::CockpitSize => {
            s.vr_cockpit_scale = (s.vr_cockpit_scale + 0.05 * dir)
                .clamp(*VR_COCKPIT_SCALE_RANGE.start(), *VR_COCKPIT_SCALE_RANGE.end());
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
    c.clear(BG);
    c.frame(Vec2::new(6.0, 6.0), Vec2::new(W as f32 - 7.0, H as f32 - 7.0), 3.0, DIM);
    if menu.exiting {
        stroke_text_at(
            c,
            W as f32 / 2.0,
            H as f32 / 2.0,
            "EXITING...",
            40.0,
            4.0,
            BRIGHT,
            Align::Centre,
        );
        tex.dirty = true;
        return;
    }
    if menu.guide {
        stroke_text_at(c, W as f32 / 2.0, 32.0, "KEY GUIDE", 22.0, 2.8, BRIGHT, Align::Centre);
        let area = guide_area(menu.keyboard);
        let caret = crate::key_guide::caret_shown(time.elapsed_secs_f64());
        draw_search(c, &menu.search, caret, menu.hover == Some(Spot::Clear));
        if menu.keyboard {
            draw_keyboard(c, menu.hover);
        }
        let lines =
            crate::key_guide::filter(crate::key_guide::lines(&config, settings.vr), &menu.search);
        if lines.is_empty() {
            let mid = (area[1] + area[3]) / 2.0;
            stroke_text_at(c, W as f32 / 2.0, mid, "NO MATCH", 18.0, 2.4, DIM, Align::Centre);
        }
        let style = crate::key_guide::Style {
            size: 0.72,
            heading: BRIGHT,
            bar: ROW_BG,
            keys: BRIGHT,
            text: 46,
            dim: DIM,
        };
        let mut scroll = menu.guide_scroll;
        crate::key_guide::draw(c, &lines, area, &mut scroll, &style);
        menu.guide_scroll = scroll;
        let hint = match (settings.vr, menu.keyboard) {
            (true, false) => "STICK: SCROLL    FIELD + TRIGGER: KEYBOARD    TRIGGER: BACK",
            (true, true) => "KEY + TRIGGER: TYPE    STICK: SCROLL",
            (false, _) => "WHEEL, ARROWS: SCROLL    ESC, CLICK: BACK",
        };
        stroke_text_at(c, W as f32 / 2.0, H as f32 - 22.0, hint, 12.0, 1.8, DIM, Align::Centre);
        tex.dirty = true;
        return;
    }
    let title = match menu.page {
        Page::Main if campaign.is_some() => "MENU  -  CAMPAIGN RUNS ON".to_string(),
        Page::Main => "MENU  -  PAUSED".to_string(),
        Page::Options => "OPTIONS".to_string(),
        page => format!("OPTIONS  -  {}", page.title()),
    };
    stroke_text_at(c, W as f32 / 2.0, TITLE_Y, &title, 26.0, 3.2, BRIGHT, Align::Centre);
    // What only the main menu's OPTIONS change (not in the middle of a flight).
    let note = match menu.page {
        Page::Game => "FLIGHT MODEL, TACVIEW: MAIN MENU OPTIONS",
        _ => "",
    };
    stroke_text_at(c, W as f32 / 2.0, H as f32 - 22.0, note, 12.0, 1.8, DIM, Align::Centre);
    for (i, (item, [x0, y0, x1, y1])) in
        menu.page.items().iter().zip(item_rects(menu.page)).enumerate()
    {
        let mid = (y0 + y1) / 2.0;
        let enabled = item.enabled(&settings, ops, menu.can_save);
        let col = if enabled { BRIGHT } else { DIM };
        let (x, w) = (x0 as i32, (x1 - x0) as i32);
        if i == menu.cursor {
            c.fill_rect(x, y0 as i32 + 3, w, (y1 - y0) as i32 - 6, ROW_BG);
        }
        if matches!(item, Item::Back | Item::Resume) {
            // A button with an arrow pointing back.
            c.rect(x, y0 as i32 + 3, w, (y1 - y0) as i32 - 6, DIM);
            let (tip, tail) = (Vec2::new(x0 + 14.0, mid), Vec2::new(x0 + 42.0, mid));
            c.stroke(tip, tail, 3.0, col);
            c.stroke(tip, tip + Vec2::new(9.0, -9.0), 3.0, col);
            c.stroke(tip, tip + Vec2::new(9.0, 9.0), 3.0, col);
            stroke_text_at(c, x0 + 56.0, mid, item.label(), 20.0, 2.6, col, Align::Left);
            continue;
        }
        if *item == Item::EndMission {
            c.hline(x + 24, x + w - 24, (y0 - END_GAP / 2.0) as i32, DIM);
        }
        let label = match item {
            Item::EndMission if campaign.is_some() => "END SORTIE",
            _ => item.label(),
        };
        stroke_text_at(c, 48.0, mid, label, 22.0, 2.8, col, Align::Left);
        let v = match item {
            Item::SaveMission => menu.save_note.unwrap_or_default().to_string(),
            _ => item.value(&settings),
        };
        if !v.is_empty() {
            stroke_text_at(c, W as f32 - 48.0, mid, &v, 22.0, 2.8, col, Align::Right);
        }
    }
    tex.dirty = true;
}

/// The search's VR keyboard, the key under the fingertip on a dark row.
fn draw_keyboard(c: &mut Canvas, hover: Option<Spot>) {
    for (key, [x0, y0, x1, y1]) in keyboard_keys() {
        let (x, y, w, h) = (x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32);
        if hover == Some(Spot::Key(key)) {
            c.fill_rect(x, y, w, h, ROW_BG);
        }
        c.rect(x, y, w, h, if key == VKey::Done { BRIGHT } else { DIM });
        let centre = Vec2::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        stroke_text_at(c, centre.x, centre.y, &key.label(), 14.0, 2.0, BRIGHT, Align::Centre);
    }
}

/// The key guide's search field: a magnifier, the text and its `_` caret while `caret` (a hint
/// while empty), the clear button's X while there is text (on a dark square under the mouse or
/// a fingertip).
fn draw_search(c: &mut Canvas, text: &str, caret: bool, clear_hover: bool) {
    let [x0, y0, x1, y1] = SEARCH;
    let mid = (y0 + y1) / 2.0;
    let col = if text.is_empty() { DIM } else { BRIGHT };
    c.rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, col);
    let lens = Vec2::new(x0 + 15.0, mid - 2.0);
    c.ring(lens, 6.0, 2.0, col);
    c.stroke(lens + Vec2::splat(4.5), lens + Vec2::splat(9.5), 2.4, col);
    let tx = x0 + 34.0;
    if text.is_empty() {
        stroke_text_at(c, tx, mid, "TYPE TO SEARCH", 14.0, 2.0, DIM, Align::Left);
        return;
    }
    let end = stroke_text_at(c, tx, mid, text, 14.0, 2.0, BRIGHT, Align::Left);
    if caret {
        stroke_text_at(c, end, mid, "_", 14.0, 2.0, BRIGHT, Align::Left);
    }
    let [bx0, by0, bx1, by1] = CLEAR;
    if clear_hover {
        c.fill_rect(
            bx0 as i32 + 3,
            by0 as i32 + 3,
            (bx1 - bx0) as i32 - 6,
            (by1 - by0) as i32 - 6,
            ROW_BG,
        );
    }
    let (cx, r) = ((bx0 + bx1) / 2.0, 6.0);
    c.stroke(Vec2::new(cx - r, mid - r), Vec2::new(cx + r, mid + r), 2.4, BRIGHT);
    c.stroke(Vec2::new(cx - r, mid + r), Vec2::new(cx + r, mid - r), 2.4, BRIGHT);
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
}
