//! Main menu: theater selector and mission generation.

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui,
};

use stealth_mission::{Mission, MissionKind, Sky};

use crate::aircraft::AircraftKind;
use crate::mission::{ActiveMission, MissionData, clean_text};
use crate::settings::{AppState, MissionSetup};
use stealth_i18n::{t, t_lines};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // The menu camera owns the egui context (the flight has several cameras, e.g. the
        // tracking camera, which must not get one).
        app.add_plugins(EguiPlugin::default())
            .insert_resource(EguiGlobalSettings { auto_create_primary_context: false, ..default() })
            .init_resource::<MenuState>()
            .init_resource::<crate::input_dialog::InputDialog>()
            .insert_resource(crate::options::OptionsWindow::from_args())
            // `--open-loadout`: start in the loadout editor (screenshots).
            .insert_resource(crate::loadout::LoadoutWindow::opened(
                std::env::args().any(|a| a == "--open-loadout"),
            ))
            .insert_resource(VrAvailable(vr_available()))
            .add_systems(OnEnter(AppState::Menu), spawn_menu_camera)
            .add_systems(OnExit(AppState::Menu), despawn_menu_camera)
            .add_systems(OnEnter(AppState::Debrief), spawn_menu_camera)
            .add_systems(OnExit(AppState::Debrief), despawn_menu_camera)
            // `--open-planner`: start in the mission planner (screenshots).
            .insert_resource(crate::planner::Planner::opened(
                std::env::args().any(|a| a == "--open-planner"),
            ))
            // `--open-career`: start in the career window (screenshots).
            .insert_resource(crate::career::CareerWindow::opened(
                std::env::args().any(|a| a == "--open-career"),
            ))
            .add_systems(
                Update,
                crate::planner::prepare_map
                    .run_if(in_state(AppState::Menu).or_else(in_state(AppState::Debrief))),
            )
            .add_systems(EguiPrimaryContextPass, menu_ui.run_if(in_state(AppState::Menu)));
        // `--export-maps <dir>`: write every theater's map picture (the master server's) and exit.
        if let Some(dir) = std::env::args().skip_while(|a| a != "--export-maps").nth(1) {
            app.insert_resource(crate::planner::MapExport::new(dir))
                .add_systems(Update, crate::planner::export_maps);
        }
    }
}

/// The classic theaters, then the newer ones (file prefix, display name).
pub use stealth_host::theater::THEATERS;

#[derive(Resource)]
pub struct MenuState {
    pub theater: usize,
    /// 0 Cold War, 1 Limited War, 2 Conventional War.
    pub roe: u8,
    pub kind: MissionKind,
    pub opponents: u8,
    pub lockheed: bool,
    /// A hosted coop campaign's friendly fire.
    pub friendly_fire: bool,
    /// None: rolled from the seed.
    pub weather: Option<Sky>,
    /// Coverage of the main cloud layer; None: rolled.
    pub cloud_cover: Option<u8>,
    pub seed: u16,
    pub mission: Option<Result<Mission, String>>,
    /// The route to fly (mission planner); reset with every new mission.
    pub route: Option<crate::mission::PlannedRoute>,
    /// The stores to carry (loadout editor); reset with every new mission and aircraft.
    pub loadout: Option<crate::loadout::PlannedLoadout>,
    /// Career mode: the pilot (file key) whose tour sortie this is.
    pub career: Option<String>,
    /// Takeoff hour (0-23); None: rolled with the mission.
    pub start_hour: Option<u8>,
    /// Moon phase in eighths of its cycle (0 new, 4 full); None: the briefing date's.
    pub moon: Option<u8>,
    /// The mission screen (single mission or career sortie) instead of the main menu.
    pub mission_screen: bool,
    /// The browser build: the first screen (play here, or download the desktop game) is up.
    web_gateway: bool,
    /// Multiplayer: the player's name, the last server joined ("host[:port]"), and how this
    /// game serves (name, empty: "<player>'s campaign"; port, players).
    pub player_name: String,
    pub server_addr: String,
    pub server_name: String,
    pub host_port: u16,
    pub max_players: u8,
    /// A hosted game is on the public server list.
    pub list_public: bool,
    dirty: bool,
}

impl MenuState {
    /// Career mode: the next sortie of the pilot's tour.
    pub fn start_career(&mut self, p: &crate::career::Pilot, t: &crate::career::Tour) {
        self.career = Some(p.key());
        (self.theater, self.roe, self.opponents, self.lockheed) =
            (t.theater, t.roe, t.opponents, t.lockheed);
        self.seed = rand::random();
        self.dirty = true;
    }

    /// Realistic weapons: only the F-117A has the real aircraft's limits.
    pub fn realistic(&self, aircraft: AircraftKind) -> bool {
        self.lockheed && aircraft == AircraftKind::F117
    }

    /// Generate a new mission next frame.
    pub fn regenerate(&mut self) {
        self.dirty = true;
    }

    /// A saved mission (`savegame.rs`) on the single mission screen: its options and number,
    /// its route and stores, as it was generated (not generated again).
    pub fn show_saved(
        &mut self,
        m: &Mission,
        seed: u16,
        route: crate::mission::PlannedRoute,
        loadout: crate::loadout::PlannedLoadout,
    ) {
        let s = &m.settings;
        (self.theater, self.roe, self.kind, self.opponents) =
            (s.theater, s.roe, s.kind, s.opponents);
        self.lockheed = s.plane == stealth_mission::Plane::Lockheed;
        (self.weather, self.cloud_cover) = (s.weather, s.cloud_cover);
        (self.start_hour, self.moon) = (s.start_hour, s.moon);
        self.seed = seed;
        self.career = None;
        self.mission = Some(Ok(m.clone()));
        self.route = Some(route);
        self.loadout = Some(loadout);
        self.dirty = false;
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            theater: 3,
            roe: 1,
            kind: MissionKind::Strike,
            opponents: 1,
            lockheed: false,
            friendly_fire: false,
            weather: None,
            cloud_cover: None,
            seed: rand::random(),
            mission: None,
            route: None,
            loadout: None,
            career: None,
            start_hour: Some(8),
            moon: None,
            // `--open-mission`: start in the mission screen (screenshots), also where
            // `--open-planner`, `--open-loadout` and `--open-saves` return to.
            mission_screen: std::env::args().any(|a| {
                ["--open-mission", "--open-planner", "--open-loadout", "--open-saves"]
                    .contains(&a.as_str())
            }),
            web_gateway: cfg!(target_arch = "wasm32"),
            player_name: String::new(),
            server_addr: String::new(),
            server_name: String::new(),
            host_port: stealth_net::DEFAULT_PORT,
            max_players: 8,
            list_public: true,
            dirty: true,
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct MenuCamera;

/// An OpenXR runtime is there, so "Fly in VR" can relaunch the game with `--vr`.
#[derive(Resource)]
pub struct VrAvailable(pub bool);

fn vr_available() -> bool {
    #[cfg(feature = "xr")]
    {
        let ok = crate::xr::runtime_available();
        info!("OpenXR runtime available: {ok}");
        ok
    }
    #[cfg(not(feature = "xr"))]
    {
        false
    }
}

/// Flyable aircraft.
const AIRCRAFT: [AircraftKind; 8] = [
    AircraftKind::F117,
    AircraftKind::F16,
    AircraftKind::Tornado,
    AircraftKind::A10,
    AircraftKind::AV8B,
    AircraftKind::UH60,
    AircraftKind::AH1Z,
    AircraftKind::RAH66,
];

/// The flag a relaunch into VR takes: the headset drawn as this game draws it (`--gl-vr`, glvr's
/// OpenGL renderer, or `--vr`, Bevy's). The Steam Frame (SteamOS on aarch64) takes `--gl-vr`
/// unless started with `--vr`: only glvr has its eye-tracked foveation.
fn vr_flag() -> &'static str {
    let has = |flag: &str| std::env::args().any(|a| a == flag);
    let frame = cfg!(all(feature = "glvr", target_os = "linux", target_arch = "aarch64"))
        && std::fs::read_to_string("/etc/os-release")
            .is_ok_and(|s| s.lines().any(|l| l == "ID=steamos"));
    if has("--gl-vr") || (frame && !has("--vr")) { "--gl-vr" } else { "--vr" }
}

/// Starts the game again in VR, straight into the menu's mission (OpenXR has to replace the
/// renderer at startup, so this cannot happen in the running process).
fn relaunch_in_vr(state: &MenuState, settings: &crate::settings::Settings) -> std::io::Result<()> {
    // From an AppImage, the AppImage itself: the mounted copy goes away when this process exits.
    let exe = match std::env::var_os("APPIMAGE") {
        Some(appimage) => appimage.into(),
        None => std::env::current_exe()?,
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.args([
        vr_flag(),
        "--mission",
        &state.seed.to_string(),
        "--theater-index",
        &state.theater.to_string(),
    ])
    .args(["--roe", &state.roe.to_string(), "--kind", &state.kind.id().to_string()])
    .args(["--opponents", &state.opponents.to_string()])
    .args(["--weather", state.weather.map_or("random", Sky::name)]);
    if let Some(c) = state.cloud_cover {
        cmd.args(["--cloud-cover", &c.to_string()]);
    }
    if let Some(m) = state.moon {
        cmd.args(["--moon", &m.to_string()]);
    }
    // Career sorties start at the rolled time.
    if let Some(h) = state.start_hour.filter(|_| state.career.is_none()) {
        cmd.args(["--start-hour", &h.to_string()]);
    }
    if state.realistic(settings.aircraft) {
        cmd.arg("--lockheed");
    }
    settings_args(&mut cmd, settings);
    if let Some(key) = &state.career {
        cmd.args(["--career", key]);
    }
    if let Some(lo) = &state.loadout {
        match crate::prefs::save_json(crate::loadout::LOADOUT_FILE, lo) {
            Ok(()) => {
                if let Some(path) = crate::prefs::config_path(crate::loadout::LOADOUT_FILE) {
                    cmd.arg("--loadout").arg(path);
                }
            }
            Err(e) => warn!("saving the loadout: {e}"),
        }
    }
    if let Some(route) = &state.route {
        match crate::prefs::save_json(crate::mission::ROUTE_FILE, route) {
            Ok(()) => {
                if let Some(path) = crate::prefs::config_path(crate::mission::ROUTE_FILE) {
                    cmd.arg("--route").arg(path);
                }
            }
            Err(e) => warn!("saving the route: {e}"),
        }
    }
    cmd.spawn().map(|_| ())
}

/// Starts the game again in VR, straight into a saved campaign.
pub fn relaunch_campaign_in_vr(
    name: &str,
    settings: &crate::settings::Settings,
) -> std::io::Result<()> {
    let exe = match std::env::var_os("APPIMAGE") {
        Some(appimage) => appimage.into(),
        None => std::env::current_exe()?,
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.args([vr_flag(), "--campaign", name]);
    settings_args(&mut cmd, settings);
    cmd.spawn().map(|_| ())
}

/// The settings a relaunched game takes over.
fn settings_args(cmd: &mut std::process::Command, settings: &crate::settings::Settings) {
    for (on, flag) in [
        (settings.modern_shading, "--modern"),
        (!settings.msaa, "--no-msaa"),
        (settings.shadows, "--shadows"),
        (settings.local_lights, "--local-lights"),
        (!settings.vr_auto_detail, "--no-auto-detail"),
        (settings.continuous_time, "--continuous-time"),
        (settings.sun_moon, "--sun-moon"),
        (settings.classic_cockpit, "--classic-cockpit"),
        (settings.classic_hud, "--classic-hud"),
        (settings.classic_avionics, "--classic-avionics"),
        (settings.classic_flight, "--classic-flight"),
        (settings.no_crashes, "--no-crashes"),
        (!settings.auto_trim, "--no-auto-trim"),
        (settings.realistic_damage, "--realistic-damage"),
        (!settings.realistic_guidance, "--classic-guidance"),
        (settings.tacview, "--tacview"),
        (settings.tacview_real_time, "--tacview-real-time"),
        (settings.air_defence, "--air-defence"),
        (settings.head_tracking, "--head-tracking"),
    ] {
        if on {
            cmd.arg(flag);
        }
    }
    if settings.tacview_address != crate::tacview_real_time::DEFAULT_ADDRESS {
        cmd.args(["--tacview-address", &settings.tacview_address.to_string()]);
    }
    if let Some(p) = &settings.tacview_password {
        cmd.args(["--tacview-password", p]);
    }
    cmd.args(["--vr-cockpit-scale", &format!("{:.0}", settings.vr_cockpit_scale * 100.0)]);
    cmd.args(["--aircraft", settings.aircraft.id()]);
}

fn spawn_menu_camera(
    mut commands: Commands,
    state: Res<State<AppState>>,
    settings: Res<crate::settings::Settings>,
    root: Option<Res<crate::camera::XrRoot>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // In the headset: the menu on a panel in front of the player.
    #[cfg(feature = "xr")]
    if settings.vr {
        let root = root.and_then(|r| r.0);
        crate::vr_menu::spawn(
            &mut commands,
            &mut images,
            &mut meshes,
            &mut materials,
            root,
            MenuCamera,
            *state.get() == AppState::Menu,
        );
        return;
    }
    let _ = (&settings, &root, &mut images, &mut meshes, &mut materials);
    // Over the scene behind the main menu (menu_scene.rs), else on black.
    let clear_color = if *state.get() == AppState::Menu {
        ClearColorConfig::None
    } else {
        ClearColorConfig::Custom(Color::BLACK)
    };
    commands.spawn((
        Camera2d,
        Camera { order: 1, clear_color, ..default() },
        MenuCamera,
        PrimaryEguiContext,
    ));
}

fn despawn_menu_camera(mut commands: Commands, q: Query<Entity, With<MenuCamera>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// 80s military terminal: green phosphor on black.
pub(crate) const BG: egui::Color32 = egui::Color32::from_rgb(4, 12, 6);
const FIELD: egui::Color32 = egui::Color32::from_rgb(8, 28, 13);
const HOVER: egui::Color32 = egui::Color32::from_rgb(18, 70, 32);
pub(crate) const GREEN: egui::Color32 = egui::Color32::from_rgb(64, 255, 110);
pub(crate) const DIM: egui::Color32 = egui::Color32::from_rgb(30, 130, 60);
pub(crate) const BRIGHT: egui::Color32 = egui::Color32::from_rgb(190, 255, 205);
pub(crate) const AMBER: egui::Color32 = egui::Color32::from_rgb(255, 176, 0);

const KINDS: [MissionKind; 5] = [
    MissionKind::Strike,
    MissionKind::Air,
    MissionKind::StrikeTraining,
    MissionKind::AirTraining,
    MissionKind::DroneDefense,
];

/// Theaters with drone launch sites (drone defense missions).
const DRONE_THEATERS: [&str; 1] = ["ukraine"];

/// The window size the menus are laid out for (logical pixels, the desktop window's).
const UI_REFERENCE: egui::Vec2 = egui::vec2(1280.0, 768.0);
/// The furthest the menus zoom out to fit a small window (a phone on its side).
const UI_ZOOM_MIN: f32 = 0.3;

/// The interface scale: smaller windows than `UI_REFERENCE` (small laptops, a phone or tablet
/// in the browser) zoom out to fit it, larger ones keep it; `--ui-zoom f` sets it instead.
pub(crate) fn apply_ui_zoom(ctx: &egui::Context) {
    let arg = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "--ui-zoom")
        .and_then(|w| w[1].parse::<f32>().ok());
    // The window's own size: the content rectangle is in zoomed points.
    let window = ctx.content_rect().size() * ctx.zoom_factor();
    let Some(z) = arg.or_else(|| fit_zoom(window)) else { return };
    if (ctx.zoom_factor() - z).abs() > 0.005 {
        ctx.set_zoom_factor(z);
    }
}

/// The zoom that fits `UI_REFERENCE` into a window this size (None before it has a size).
fn fit_zoom(window: egui::Vec2) -> Option<f32> {
    (window.min_elem() > 0.0)
        .then(|| (window.x / UI_REFERENCE.x).min(window.y / UI_REFERENCE.y).clamp(UI_ZOOM_MIN, 1.0))
}

pub(crate) fn apply_terminal_style(style: &mut egui::Style) {
    use egui::{CornerRadius, FontFamily::Monospace, FontId, Shadow, Stroke, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(26.0, Monospace)),
        (TextStyle::Body, FontId::new(15.0, Monospace)),
        (TextStyle::Button, FontId::new(15.0, Monospace)),
        (TextStyle::Small, FontId::new(12.0, Monospace)),
        (TextStyle::Monospace, FontId::new(15.0, Monospace)),
    ]
    .into();
    let v = &mut style.visuals;
    *v = egui::Visuals::dark();
    v.window_fill = BG;
    v.panel_fill = BG;
    v.extreme_bg_color = egui::Color32::BLACK;
    v.faint_bg_color = FIELD;
    v.window_stroke = Stroke::new(2.0, GREEN);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.selection.bg_fill = DIM;
    v.selection.stroke = Stroke::new(1.0, BRIGHT);
    for (w, bg, fg, edge) in [
        (&mut v.widgets.noninteractive, BG, GREEN, DIM),
        (&mut v.widgets.inactive, FIELD, GREEN, DIM),
        (&mut v.widgets.hovered, HOVER, BRIGHT, BRIGHT),
        (&mut v.widgets.active, DIM, BRIGHT, BRIGHT),
        (&mut v.widgets.open, FIELD, GREEN, DIM),
    ] {
        w.bg_fill = bg;
        w.weak_bg_fill = bg;
        w.fg_stroke = Stroke::new(1.5, fg);
        w.bg_stroke = Stroke::new(1.0, edge);
        w.corner_radius = CornerRadius::ZERO;
        w.expansion = 0.0;
    }
    style.spacing.item_spacing = egui::vec2(8.0, 5.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.interact_size.y = 24.0;
    style.spacing.icon_width = 18.0;
    style.spacing.icon_width_inner = 10.0;
}

/// Hover text: a short list of facts.
pub(crate) fn facts(resp: egui::Response, facts: &[&str]) -> egui::Response {
    resp.on_hover_ui(|ui| {
        ui.set_max_width(380.0);
        for f in facts {
            ui.label(egui::RichText::new(format!("> {f}")).size(14.0));
        }
    })
}

/// `text` in monospace, smaller if it is wider than `width` points (a translation longer than
/// the English), down to 70 % of the size: a label's truncation takes what still does not fit.
pub(crate) fn fit_monospace(ui: &egui::Ui, text: egui::RichText, width: f32) -> egui::RichText {
    let size = egui::TextStyle::Monospace.resolve(ui.style()).size;
    let font = egui::FontId::monospace(size);
    let w = ui.painter().layout_no_wrap(text.text().to_string(), font, GREEN).size().x;
    let text = text.monospace();
    if w <= width { text } else { text.size((size * width / w).max(size * 0.7)) }
}

pub(crate) fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(format!("[ {} ]", title.to_uppercase())).color(AMBER).size(14.0));
}

/// A framed group of sections under a bright title, filling the column.
pub(crate) fn group<R>(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let r = egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, DIM))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(title.to_uppercase()).size(16.0).color(BRIGHT));
            add(ui)
        })
        .inner;
    ui.add_space(6.0);
    r
}

/// A dropdown over `options` (value, name, hover facts), the selected one's facts on the button.
/// The mouse wheel over the button steps to the previous (up) or next (down) option, as in
/// Windows.
pub(crate) fn dropdown<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut T,
    options: impl IntoIterator<Item = (T, &'static str, &'static [&'static str])> + Clone,
) {
    let (name, f) =
        options.clone().into_iter().find(|o| o.0 == *value).map_or(("", &[][..]), |o| (o.1, o.2));
    let resp = egui::ComboBox::from_id_salt(id)
        .selected_text(name)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for (v, name, f) in options.clone() {
                facts(ui.selectable_value(value, v, name), f);
            }
        })
        .response;
    let notches = wheel_notches(ui, &resp);
    if notches != 0 {
        let values: Vec<T> = options.into_iter().map(|o| o.0).collect();
        if let Some(i) = values.iter().position(|v| v == value) {
            let to = i.saturating_add_signed(-notches as isize).min(values.len() - 1);
            if to != i {
                *value = values[to];
                ui.ctx().request_repaint();
            }
        }
    }
    facts(resp, f);
}

/// The wheel's rest (s) that ends a turn, the next one going to whatever is under the pointer.
const WHEEL_REST: f64 = 0.5;

/// The current wheel turn, shared by all dropdowns: it belongs to the dropdown under the pointer
/// when it started, or to none, so scrolling a panel past a dropdown does not change it.
#[derive(Clone, Copy, Default)]
struct WheelTurn {
    owner: Option<egui::Id>,
    /// When the turn started and when the wheel last moved (egui time, s).
    start: f64,
    last: f64,
    /// The owner's scroll not yet a whole notch (a touchpad's fine scroll).
    lines: f32,
}

/// The whole wheel notches turned over `resp` this frame (positive: up), the plain wheel only
/// (Ctrl zooms, Shift scrolls sideways). The owner swallows the scroll so the panel under it stays
/// put.
fn wheel_notches(ui: &egui::Ui, resp: &egui::Response) -> i32 {
    let ctx = ui.ctx();
    let line = ctx.options(|o| o.input_options.line_scroll_speed).max(1.0);
    let (lines, now) = ctx.input(|i| {
        let lines: f32 = i
            .events
            .iter()
            .map(|e| match e {
                egui::Event::MouseWheel { unit, delta, modifiers, .. } if modifiers.is_none() => {
                    match unit {
                        egui::MouseWheelUnit::Point => delta.y / line,
                        _ => delta.y,
                    }
                }
                _ => 0.0,
            })
            .sum();
        (lines, i.time)
    });
    let hovered = resp.hovered();
    if lines == 0.0 && !hovered {
        return 0;
    }
    let notches = ctx.data_mut(|d| {
        let turn = d.get_temp_mut_or_default::<WheelTurn>(egui::Id::new("dropdown_wheel_turn"));
        // The first dropdown to see this frame's scroll updates the turn for all of them.
        if lines != 0.0 && turn.last != now {
            if now - turn.last > WHEEL_REST {
                *turn = WheelTurn { start: now, ..Default::default() };
            }
            turn.last = now;
        }
        if !hovered {
            return None;
        }
        if turn.start == now {
            turn.owner = Some(resp.id);
        }
        (turn.owner == Some(resp.id)).then(|| {
            turn.lines += lines;
            let n = turn.lines.trunc();
            turn.lines -= n;
            n as i32
        })
    });
    match notches {
        Some(n) => {
            ctx.input_mut(|i| i.smooth_scroll_delta.y = 0.0);
            n
        }
        None => 0,
    }
}

/// A labelled CLASSIC / MODERN slide switch; `modern` is the right position.
pub(crate) fn slide_switch(ui: &mut egui::Ui, label: &str, modern: &mut bool) -> egui::Response {
    two_way_switch(ui, label, modern, (t!("classic"), t!("modern")))
}

/// A labelled slide switch between two settings (`sides`: left, right); `right` is the right
/// position. The track is as wide as its words; a label that does not fit beside it goes above.
fn two_way_switch(
    ui: &mut egui::Ui,
    label: &str,
    modern: &mut bool,
    sides: (&str, &str),
) -> egui::Response {
    let font = egui::FontId::monospace(12.0);
    let side_font = egui::FontId::monospace(11.0);
    let label = label.to_uppercase();
    let text_width = |t: &str, f: &egui::FontId| {
        ui.painter().layout_no_wrap(t.to_string(), f.clone(), GREEN).size().x
    };
    let half =
        (text_width(sides.0, &side_font).max(text_width(sides.1, &side_font)) + 8.0).max(54.0);
    let track_width = 2.0 * half;
    let beside = text_width(&label, &font) + 8.0 + track_width;
    // One width for every switch of a column, so that their tracks line up.
    let width = ui.available_width().min(260.0);
    let height = if beside > width { 40.0 } else { 22.0 };
    let (rect, mut resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    if resp.clicked() {
        *modern = !*modern;
        resp.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool_responsive(resp.id, *modern);
        let hovered = resp.hovered();
        let p = ui.painter();
        p.text(
            egui::pos2(rect.left(), rect.top() + 11.0),
            egui::Align2::LEFT_CENTER,
            label,
            font,
            if hovered { BRIGHT } else { GREEN },
        );
        // The track: two halves, the knob slides over the selected one.
        let track = egui::Rect::from_min_max(
            egui::pos2(rect.right() - track_width, rect.bottom() - 22.0),
            rect.right_bottom(),
        );
        p.rect_filled(track, 0.0, FIELD);
        p.rect_stroke(
            track,
            0.0,
            egui::Stroke::new(1.0, if hovered { BRIGHT } else { DIM }),
            egui::StrokeKind::Inside,
        );
        let knob = egui::Rect::from_min_size(
            egui::pos2(track.left() + t * half, track.top()),
            egui::vec2(half, track.height()),
        )
        .shrink(2.0);
        p.rect_filled(knob, 0.0, if *modern { DIM } else { AMBER.gamma_multiply(0.55) });
        for (i, (text, on)) in [(sides.0, !*modern), (sides.1, *modern)].into_iter().enumerate() {
            let center = egui::pos2(track.left() + half * (i as f32 + 0.5), track.center().y);
            p.text(
                center,
                egui::Align2::CENTER_CENTER,
                text,
                side_font.clone(),
                if on { BRIGHT } else { DIM },
            );
        }
    }
    resp
}

pub(crate) fn big_button(text: &str) -> egui::Button<'static> {
    // The grow atoms centre the label; otherwise it follows the parent layout's alignment.
    let label = (egui::Atom::grow(), egui::RichText::new(text).size(22.0), egui::Atom::grow());
    egui::Button::new(label).min_size(egui::vec2(170.0, 50.0))
}

type SettingsMut<'a> = ResMut<'a, crate::settings::Settings>;

pub(crate) fn theater_options(ui: &mut egui::Ui, theater: &mut usize) {
    section(ui, t!("section-theater"));
    for i in 0..THEATERS.len() {
        facts(
            ui.radio_value(theater, i, crate::i18n::theater_name(i)),
            crate::i18n::theater_facts(i),
        );
    }
}

pub(crate) fn opponent_options(ui: &mut egui::Ui, opponents: &mut u8) {
    section(ui, t!("section-opponents"));
    let options =
        (0..4).map(|o| (o, crate::i18n::opponents_name(o), crate::i18n::opponents_facts(o)));
    dropdown(ui, "opponents", opponents, options);
}

pub(crate) fn situation_options(ui: &mut egui::Ui, roe: &mut u8) {
    section(ui, t!("section-situation"));
    let options = (0..3).map(|r| (r, crate::i18n::roe_name(r), crate::i18n::roe_facts(r)));
    dropdown(ui, "situation", roe, options);
}

fn mission_options(ui: &mut egui::Ui, state: &mut MenuState) {
    section(ui, t!("section-mission"));
    let drones = DRONE_THEATERS.contains(&THEATERS[state.theater].0);
    if !drones && state.kind == MissionKind::DroneDefense {
        state.kind = MissionKind::Air;
    }
    let options = KINDS
        .into_iter()
        .filter(|k| *k != MissionKind::DroneDefense || drones)
        .map(|k| (k, crate::i18n::kind_name(k), crate::i18n::kind_facts(k)));
    dropdown(ui, "mission", &mut state.kind, options);
}

/// The moon's phase (with the time of day switch on).
pub(crate) fn moon_options(ui: &mut egui::Ui, moon: &mut Option<u8>) {
    section(ui, t!("section-moon"));
    let options =
        std::iter::once((None, t!("moon-date"), t_lines!("moon-date.facts")))
            .chain((0..8u8).map(|m| {
                (Some(m), t!(&format!("moon-{m}")), t_lines!(&format!("moon-{m}.facts")))
            }));
    dropdown(ui, "moon", moon, options);
}

fn start_time_options(ui: &mut egui::Ui, state: &mut MenuState) {
    section(ui, t!("section-start-time"));
    let mut random = state.start_hour.is_none();
    let r = facts(
        ui.checkbox(&mut random, t!("menu-start-random")),
        t_lines!("menu-start-random.facts"),
    );
    if r.changed() {
        state.start_hour = if random { None } else { Some(8) };
    }
    if let Some(h) = state.start_hour.as_mut() {
        let slider = egui::Slider::new(h, 0..=23).custom_formatter(|v, _| format!("{v:02.0}:00"));
        facts(ui.add(slider), t_lines!("menu-start-slider.facts"));
    }
}

/// The aircraft, with its weapons switch; `lockheed` shows "Realistic weapons".
pub(crate) fn vehicle_options(
    ui: &mut egui::Ui,
    settings: &mut SettingsMut,
    lockheed: Option<&mut bool>,
) {
    let s = settings.bypass_change_detection();
    let mut picked = false;
    for kind in AIRCRAFT {
        picked |= facts(
            ui.radio_value(&mut s.aircraft, kind, kind.name()),
            crate::i18n::aircraft_facts(kind),
        )
        .changed();
    }
    if picked {
        settings.set_changed();
    }
    // The F-16 carries AIM-9s and AIM-120s on its pylons anyway.
    if settings.aircraft == AircraftKind::F117
        && let Some(lockheed) = lockheed
    {
        facts(
            ui.checkbox(lockheed, t!("menu-realistic-weapons")),
            t_lines!("menu-realistic-weapons.facts"),
        );
    }
}

/// The realism switches with crash protection: the settings each game is set up with (the
/// player's own settings are in OPTIONS, `options.rs`).
pub(crate) fn simulation_options(
    ui: &mut egui::Ui,
    settings: &mut SettingsMut,
    note: Option<&str>,
) {
    group(ui, t!("group-simulation"), |ui| {
        realism_options(ui, settings);
        crash_option(ui, settings);
        if let Some(note) = note {
            ui.label(egui::RichText::new(note).small().color(DIM));
        }
    });
}

fn crash_option(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    let s = settings.bypass_change_detection();
    if facts(
        ui.checkbox(&mut s.no_crashes, t!("menu-crash-protection")),
        t_lines!("menu-crash-protection.facts"),
    )
    .changed()
    {
        settings.set_changed();
    }
}

pub(crate) fn tacview_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    // The browser build writes no files and cannot listen for Tacview.
    if cfg!(target_arch = "wasm32") {
        return;
    }
    section(ui, t!("section-tacview"));
    let s = settings.bypass_change_detection();
    if facts(
        ui.checkbox(&mut s.tacview, t!("menu-tacview-export")),
        t_lines!("menu-tacview-export.facts"),
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        ui.checkbox(&mut s.tacview_real_time, t!("menu-tacview-real-time")),
        t_lines!("menu-tacview-real-time.facts"),
    )
    .changed()
    {
        settings.set_changed();
    }
}

pub(crate) fn realism_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, t!("section-realism"));
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(
            ui,
            t!("menu-damage"),
            &mut s.realistic_damage,
            (t!("switch-game"), t!("switch-real")),
        ),
        t_lines!("menu-damage.facts"),
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(
            ui,
            t!("menu-guidance"),
            &mut s.realistic_guidance,
            (t!("classic"), t!("switch-real")),
        ),
        t_lines!("menu-guidance.facts"),
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(
            ui,
            t!("menu-air-defence"),
            &mut s.air_defence,
            (t!("classic"), t!("switch-real")),
        ),
        t_lines!("menu-air-defence.facts"),
    )
    .changed()
    {
        settings.set_changed();
    }
}

pub(crate) fn weather_options(
    ui: &mut egui::Ui,
    weather: &mut Option<Sky>,
    cloud_cover: &mut Option<u8>,
) {
    section(ui, t!("section-weather"));
    let options = std::iter::once((None, t!("sky-random"), t_lines!("sky-random.facts"))).chain(
        Sky::ALL
            .into_iter()
            .map(|sky| (Some(sky), crate::i18n::sky_name(sky), crate::i18n::sky_facts(sky))),
    );
    dropdown(ui, "weather", weather, options);
    // Coverage of the main layer (cumulus, or stratus for overcast and storms).
    let mut auto = cloud_cover.is_none();
    let r =
        facts(ui.checkbox(&mut auto, t!("menu-random-cover")), t_lines!("menu-random-cover.facts"));
    if r.changed() {
        *cloud_cover = if auto { None } else { Some(50) };
    }
    if let Some(c) = cloud_cover.as_mut() {
        facts(
            ui.add(egui::Slider::new(c, 0..=100).suffix("%")),
            t_lines!("menu-cover-slider.facts"),
        );
    }
}

/// The download page beside the game's page, as an absolute address (egui opens no relative one).
#[cfg(target_arch = "wasm32")]
fn download_url() -> String {
    let location = web_sys::window().map(|w| w.location());
    let origin = location.as_ref().and_then(|l| l.origin().ok()).unwrap_or_default();
    let path = location.as_ref().and_then(|l| l.pathname().ok()).unwrap_or_default();
    format!("{origin}{}download.html", &path[..path.rfind('/').map_or(0, |i| i + 1)])
}

/// The game's Discord server (also on the browser build's loading screen, web/index.html).
#[cfg(target_arch = "wasm32")]
const DISCORD_URL: &str = "https://discord.gg/zjjGeKKpC";

/// A full-screen-width menu window in the terminal style.
pub(crate) fn menu_window(ctx: &egui::Context, id: &str, width: f32) -> egui::Window<'static> {
    egui::Window::new(id.to_string())
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .min_width(width)
        .max_width(width)
        .frame(egui::Frame::window(&ctx.global_style()).inner_margin(16.0))
}

/// Width of the menu's wide windows.
pub(crate) fn wide_width(ctx: &egui::Context) -> f32 {
    (ctx.content_rect().width() - 32.0).clamp(600.0, 1400.0)
}

/// What the download has that the browser build has not (`web_gateway`), with the reason.
#[cfg(target_arch = "wasm32")]
const DOWNLOAD_GIVES: [&str; 7] = [
    "web-gives-vr",
    "web-gives-hotas",
    "web-gives-multiplayer",
    "web-gives-server",
    "web-gives-head-tracking",
    "web-gives-tacview",
    "web-gives-keep",
];

/// The browser build's first screen: play in the browser or download the desktop game, beside
/// what the download version adds.
#[cfg(target_arch = "wasm32")]
fn web_gateway(ctx: &egui::Context, state: &mut MenuState) {
    let wide = |text: &str| big_button(text).min_size(egui::vec2(300.0, 50.0));
    let width = (ctx.content_rect().width() - 32.0).clamp(600.0, 820.0);
    menu_window(ctx, "Stealth Mission 89", width).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(t!("menu-title")).heading().color(BRIGHT));
            ui.label(egui::RichText::new(t!("menu-subtitle")).color(DIM));
        });
        ui.add_space(14.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.0);
                if ui.add(wide(t!("web-play"))).clicked() {
                    state.web_gateway = false;
                }
                ui.label(egui::RichText::new(t!("web-play.facts")).color(DIM));
                ui.add_space(14.0);
                if ui.add(wide(t!("menu-download"))).clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(download_url()));
                }
                ui.label(egui::RichText::new(t!("web-download-platforms")).color(DIM));
            });
            ui.add_space(16.0);
            ui.vertical(|ui| {
                group(ui, t!("web-download-version"), |ui| {
                    for id in DOWNLOAD_GIVES {
                        ui.label(egui::RichText::new(format!("> {}", t!(id))).color(GREEN));
                        ui.label(
                            egui::RichText::new(t!(&format!("{id}.why"))).size(14.0).color(DIM),
                        );
                        ui.add_space(2.0);
                    }
                });
            });
        });
    });
}

/// The top level: what to play.
fn main_screen(
    ctx: &egui::Context,
    state: &mut MenuState,
    options: &mut crate::options::OptionsWindow,
    career: &mut crate::career::CareerWindow,
    campaign: &mut crate::campaign::CampaignWindow,
    multiplayer: &mut crate::multiplayer_menu::MultiplayerWindow,
    exit: &mut MessageWriter<AppExit>,
) {
    let wide = |text: &str| big_button(text).min_size(egui::vec2(360.0, 50.0));
    menu_window(ctx, "Stealth Mission 89", 420.0).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(t!("menu-title")).heading().color(BRIGHT));
            ui.label(egui::RichText::new(t!("menu-subtitle")).color(DIM));
            ui.add_space(10.0);
            if facts(ui.add(wide(t!("menu-single-mission"))), t_lines!("menu-single-mission.facts"))
                .clicked()
            {
                // Leaves career mode; the same mission again if the options are the same.
                state.career = None;
                state.dirty = true;
                state.mission_screen = true;
            }
            if facts(ui.add(wide(t!("menu-career"))), t_lines!("menu-career.facts")).clicked() {
                career.show();
            }
            if facts(ui.add(wide(t!("menu-campaign"))), t_lines!("menu-campaign.facts")).clicked() {
                campaign.show();
            }
            // Browsers only join, through the master server's relay (docs/web.md).
            let how = if cfg!(target_arch = "wasm32") {
                t!("menu-multiplayer.join")
            } else {
                t!("menu-multiplayer.host")
            };
            if facts(ui.add(wide(t!("menu-multiplayer"))), &[t!("menu-multiplayer.facts"), how])
                .clicked()
            {
                multiplayer.show();
            }
            if facts(ui.add(wide(t!("menu-options"))), t_lines!("menu-options.facts")).clicked() {
                options.open = true;
            }
            // The browser build: the desktop game is a download (web/download.html).
            #[cfg(target_arch = "wasm32")]
            if facts(ui.add(wide(t!("menu-download"))), t_lines!("menu-download.facts")).clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(download_url()));
            }
            #[cfg(target_arch = "wasm32")]
            if facts(ui.add(wide(t!("menu-discord"))), t_lines!("menu-discord.facts")).clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(DISCORD_URL));
            }
            ui.add_space(10.0);
            // In the browser the tab is closed instead.
            if !cfg!(target_arch = "wasm32")
                && facts(ui.add(wide(t!("menu-quit"))), t_lines!("menu-quit.facts")).clicked()
            {
                exit.write(AppExit::Success);
            }
            ui.add_space(6.0);
            // In the browser: which build runs (docs/web.md).
            let renderer = match (cfg!(target_arch = "wasm32"), crate::WEBGL2) {
                (false, _) => "",
                (true, false) => " · WEBGPU",
                (true, true) => " · WEBGL2",
            };
            ui.label(
                egui::RichText::new(format!(
                    "{}{renderer}",
                    t!("menu-version", version = env!("CARGO_PKG_VERSION"))
                ))
                .size(11.0)
                .color(DIM),
            );
        });
    });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn menu_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut state: ResMut<MenuState>,
    data: Option<Res<MissionData>>,
    tables: Res<crate::flight::SimTables>,
    mut setup: ResMut<MissionSetup>,
    mut settings: ResMut<crate::settings::Settings>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    (vr, head): (Res<VrAvailable>, Res<crate::headtrack::HeadTrack>),
    mut dialog: ResMut<crate::input_dialog::InputDialog>,
    mut input_config: ResMut<crate::hotas::InputConfig>,
    hotas: Res<crate::hotas::Hotas>,
    keys: Res<ButtonInput<KeyCode>>,
    (mut saved, mut worded_in): (
        Local<Option<crate::prefs::Prefs>>,
        Local<Option<stealth_i18n::Language>>,
    ),
    windows: (
        ResMut<crate::planner::Planner>,
        ResMut<crate::career::CareerWindow>,
        ResMut<crate::loadout::LoadoutWindow>,
        ResMut<crate::campaign::CampaignWindow>,
        ResMut<crate::multiplayer_menu::MultiplayerWindow>,
        ResMut<crate::options::OptionsWindow>,
        ResMut<crate::savegame::SavesWindow>,
    ),
) -> Result {
    let (
        mut planner,
        mut career,
        mut loadout,
        mut campaign,
        mut multiplayer,
        mut options,
        mut saves,
    ) = windows;
    // Generator data is loaded on first use.
    let Some(data) = data else {
        match MissionData::load() {
            Ok(d) => commands.insert_resource(d),
            Err(e) => state.mission = Some(Err(e.to_string())),
        }
        return Ok(());
    };
    // The briefing is worded when the mission is generated: again (the same mission) in
    // another language.
    if worded_in.replace(settings.language).is_some_and(|l| l != settings.language) {
        state.dirty = true;
    }
    if state.dirty {
        state.dirty = false;
        let pilot = state.career.as_deref().and_then(crate::career::Pilot::load);
        if let Some(p) = pilot {
            // Career: the tour's next sortie, on targets that still stand.
            match crate::career::next_sortie(&data, &p, &state, settings.aircraft) {
                Some((seed, m)) => {
                    state.seed = seed;
                    state.kind = m.settings.kind;
                    state.mission = Some(Ok(m));
                }
                None => state.mission = Some(Err(t!("menu-no-targets").into())),
            }
        } else {
            state.career = None;
            let mut mission_settings = crate::mission::settings(
                state.theater,
                state.roe,
                state.kind,
                state.opponents,
                state.realistic(settings.aircraft),
                state.weather,
                state.cloud_cover,
            );
            mission_settings.start_hour = state.start_hour;
            mission_settings.moon = state.moon;
            mission_settings.helicopter = settings.aircraft.spec().helicopter;
            let transport = settings.aircraft.spec().transport();
            state.mission = Some(
                crate::mission::generate(&data, &mission_settings, state.seed)
                    .map(|mut m| {
                        if transport {
                            stealth_mission::insertion::for_helicopter(&mut m);
                        }
                        m
                    })
                    .map_err(|e| format!("{e:#}")),
            );
        }
        state.route = state
            .mission
            .as_ref()
            .and_then(|m| m.as_ref().ok())
            .map(crate::mission::PlannedRoute::of);
    }
    // The briefed stores, on the aircraft flown.
    if let Some(Ok(m)) = &state.mission
        && !state.loadout.as_ref().is_some_and(|l| l.fits(m, settings.aircraft))
    {
        state.loadout = Some(crate::loadout::PlannedLoadout::of(m, settings.aircraft));
    }

    let ctx = contexts.ctx_mut()?;
    apply_ui_zoom(ctx);
    // Both themes, so the system light/dark preference does not matter.
    if ctx.global_style().visuals.window_fill != BG {
        ctx.all_styles_mut(apply_terminal_style);
    }
    let before = (
        state.theater,
        state.roe,
        state.kind,
        state.opponents,
        state.lockheed,
        state.weather,
        state.cloud_cover,
        state.start_hour,
        state.moon,
    );
    let realistic_before = state.realistic(settings.aircraft);
    // One window at a time: the open dialog replaces the screen it was opened from.
    let dialogs = dialog.open || options.open || career.open || campaign.open || multiplayer.open;
    // The browser build first asks whether to play here or download (never up on the desktop).
    let gateway = state.web_gateway && !state.mission_screen && !dialogs;
    #[cfg(target_arch = "wasm32")]
    if gateway {
        web_gateway(ctx, &mut state);
    }
    if !state.mission_screen && !dialogs && !gateway {
        main_screen(
            ctx,
            &mut state,
            &mut options,
            &mut career,
            &mut campaign,
            &mut multiplayer,
            &mut exit,
        );
    }
    if state.mission_screen && !dialogs && !planner.open && !loadout.open && !saves.open {
        let width = wide_width(ctx);
        // Career sorties take their options from the career window.
        let career_mode = state.career.is_some();
        let mut back = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        menu_window(ctx, "Single mission", width).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if career_mode {
                        t!("menu-career-sortie")
                    } else {
                        t!("menu-single-mission")
                    })
                    .heading()
                    .color(BRIGHT),
                );
                if let Some(p) = state.career.as_deref().and_then(crate::career::Pilot::load)
                    && let Some(t) = &p.tour
                {
                    let line = t!(
                        "menu-tour-sortie",
                        pilot = p.title().to_uppercase(),
                        sortie = t.sortie + 1,
                        length = t.length
                    );
                    ui.label(egui::RichText::new(format!("  {line}")).color(AMBER));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(t!("menu-msn", seed = format!("{:05}", state.seed)))
                            .color(DIM),
                    );
                });
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                if !career_mode {
                    // Options: three narrow columns.
                    ui.vertical(|ui| {
                        ui.set_width(210.0);
                        group(ui, t!("group-mission"), |ui| {
                            theater_options(ui, &mut state.theater);
                            situation_options(ui, &mut state.roe);
                            opponent_options(ui, &mut state.opponents);
                            mission_options(ui, &mut state);
                        });
                    });
                    ui.vertical(|ui| {
                        ui.set_width(210.0);
                        group(ui, t!("group-conditions"), |ui| {
                            let state = &mut *state;
                            weather_options(ui, &mut state.weather, &mut state.cloud_cover);
                            start_time_options(ui, state);
                            moon_options(ui, &mut state.moon);
                        });
                        group(ui, t!("group-vehicle"), |ui| {
                            let was = settings.aircraft;
                            vehicle_options(ui, &mut settings, Some(&mut state.lockheed));
                            // A troop helicopter flies the same seed's targets as insertions;
                            // helicopters fly short missions.
                            let (now, was) = (settings.aircraft.spec(), was.spec());
                            if now.transport() != was.transport()
                                || now.helicopter != was.helicopter
                            {
                                state.regenerate();
                            }
                        });
                    });
                    ui.vertical(|ui| {
                        ui.set_width(220.0);
                        simulation_options(ui, &mut settings, None);
                    });
                }
                // Mission summary in big type, the full briefing below in small type.
                // What is left beside the option columns, less the frame margins and border.
                let summary_width = (ui.available_width() - 22.0).max(300.0);
                egui::Frame::new().stroke(egui::Stroke::new(1.0, DIM)).inner_margin(10.0).show(
                    ui,
                    |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(summary_width);
                            match &state.mission {
                                Some(Ok(m)) => {
                                    mission_summary(ui, &state, m);
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        let plan = ui.button(
                                            egui::RichText::new(t!("menu-plan-route")).size(16.0),
                                        );
                                        if facts(plan, t_lines!("menu-plan-route.facts")).clicked()
                                        {
                                            planner.open = true;
                                        }
                                        let arm = ui.button(
                                            egui::RichText::new(t!("menu-loadout")).size(16.0),
                                        );
                                        if facts(arm, t_lines!("menu-loadout.facts")).clicked() {
                                            loadout.open = true;
                                        }
                                    });
                                }
                                Some(Err(e)) => {
                                    ui.colored_label(AMBER, e);
                                }
                                None => {
                                    ui.label(t!("menu-generating"));
                                }
                            }
                        });
                    },
                );
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let to = if career_mode { t!("menu-back-records") } else { t!("menu-back-main") };
                if facts(ui.add(big_button(t!("back"))), &[to]).clicked() {
                    back = true;
                }
                let new = if career_mode { t!("menu-new-sortie") } else { t!("menu-new-mission") };
                if facts(ui.add(big_button(new)), t_lines!("menu-new-mission.facts")).clicked() {
                    state.seed = rand::random();
                    state.dirty = true;
                }
                if !career_mode
                    && facts(ui.add(big_button(t!("menu-load"))), t_lines!("menu-load.facts"))
                        .clicked()
                {
                    saves.show();
                }
                if facts(ui.add(big_button(t!("menu-options"))), &[t!("menu-options.short")])
                    .clicked()
                {
                    options.open = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_fly = matches!(state.mission, Some(Ok(_)));
                    let fly = ui.add_enabled(can_fly, big_button(t!("menu-fly")).fill(DIM));
                    if facts(fly, t_lines!("menu-fly.facts")).clicked()
                        && let Some(Ok(m)) = &state.mission
                    {
                        *setup = crate::mission::setup_from_mission(m, &tables.start_dir);
                        commands.insert_resource(ActiveMission(m.clone()));
                        commands.insert_resource(
                            state
                                .route
                                .clone()
                                .unwrap_or_else(|| crate::mission::PlannedRoute::of(m)),
                        );
                        commands.insert_resource(state.loadout.clone().unwrap_or_else(|| {
                            crate::loadout::PlannedLoadout::of(m, settings.aircraft)
                        }));
                        match &state.career {
                            Some(key) => {
                                commands.insert_resource(crate::career::CareerFlight(key.clone()))
                            }
                            None => commands.remove_resource::<crate::career::CareerFlight>(),
                        }
                        next.set(AppState::Flight);
                    }
                    // Already in VR: "Fly" flies in the headset. No VR in the browser.
                    if !settings.vr && !cfg!(target_arch = "wasm32") {
                        let vr_button = ui
                            .add_enabled(can_fly && vr.0, big_button(t!("menu-fly-vr")))
                            .on_disabled_hover_text(if vr.0 {
                                t!("menu-fly-vr.no-mission")
                            } else {
                                t!("menu-fly-vr.no-runtime")
                            });
                        if facts(vr_button, t_lines!("menu-fly-vr.facts")).clicked() {
                            match relaunch_in_vr(&state, &settings) {
                                Ok(()) => {
                                    exit.write(AppExit::Success);
                                }
                                Err(e) => {
                                    state.mission =
                                        Some(Err(t!("menu-vr-failed", error = e.to_string())))
                                }
                            }
                        }
                    }
                });
            });
        });
        if back {
            state.mission_screen = false;
            if career_mode {
                career.show();
            }
        }
    }
    crate::input_dialog::show(
        ctx,
        &mut dialog,
        input_config.bypass_change_detection(),
        &hotas,
        &keys,
        settings.vr,
    );
    crate::planner::planner_ui(ctx, &mut planner, &mut state);
    crate::loadout::loadout_ui(ctx, &mut loadout, &mut state, settings.aircraft);
    let loaded = crate::savegame::saves_ui(ctx, &mut saves);
    if before
        != (
            state.theater,
            state.roe,
            state.kind,
            state.opponents,
            state.lockheed,
            state.weather,
            state.cloud_cover,
            state.start_hour,
            state.moon,
        )
    {
        // Changing the mission's options leaves career mode.
        state.dirty = true;
        state.career = None;
    } else if realistic_before != state.realistic(settings.aircraft) {
        // Another aircraft: the same mission with or without the F-117A's limits.
        state.dirty = true;
    }
    // A save loaded: its mission and choices, not a new mission from them.
    if let Some(save) = loaded {
        crate::savegame::fly(
            &mut commands,
            save,
            &mut state,
            &mut settings,
            &mut setup,
            &tables.start_dir,
        );
        next.set(AppState::Flight);
    }
    crate::career::career_ui(ctx, &mut career, &mut state, &mut settings, &mut options);
    crate::options::options_ui(ctx, &mut options, &mut settings, head.source, &mut dialog);
    crate::prefs::Prefs::save_if_changed(&state, &settings, &mut saved);
    Ok(())
}

fn mission_summary(ui: &mut egui::Ui, state: &MenuState, m: &Mission) {
    let b = &m.briefing;
    section(ui, t!("section-mission-summary"));
    ui.label(egui::RichText::new(clean_text(&b.title)).size(20.0).color(BRIGHT));
    ui.add_space(4.0);
    let w = &m.weather;
    let rows = [
        (t!("summary-theater"), crate::i18n::theater_name(state.theater).to_string(), GREEN),
        (t!("summary-situation"), crate::i18n::roe_name(state.roe).to_string(), GREEN),
        (t!("summary-mission"), crate::i18n::kind_name(state.kind).to_string(), GREEN),
        (t!("summary-opponents"), crate::i18n::opponents_name(state.opponents).to_string(), GREEN),
        (
            t!("summary-time"),
            t!(
                "summary-time-value",
                daytime = crate::i18n::daytime(m.night),
                depart = m.time.depart.to_string()
            ),
            GREEN,
        ),
        (
            t!("summary-weather"),
            t!("summary-weather-value", sky = crate::i18n::sky_name(w.sky), vis = w.vis_km),
            if w.sky == Sky::Storm { AMBER } else { GREEN },
        ),
        (t!("summary-fuel"), t!("summary-fuel-value", fuel = m.fuel_lbs), GREEN),
    ];
    egui::Grid::new("summary").num_columns(2).spacing([18.0, 4.0]).show(ui, |ui| {
        for (key, value, color) in rows {
            ui.label(egui::RichText::new(key).size(14.0).color(DIM));
            ui.label(egui::RichText::new(value.to_uppercase()).size(20.0).color(color));
            ui.end_row();
        }
    });
    section(ui, t!("section-briefing"));
    egui::ScrollArea::vertical().max_height(230.0).auto_shrink([false, true]).show(ui, |ui| {
        let small = |ui: &mut egui::Ui, s: &str| {
            if !s.trim().is_empty() {
                ui.label(egui::RichText::new(clean_text(s)).small());
            }
        };
        for l in [
            &b.primary_objective,
            &b.secondary_objective,
            &b.depart_line,
            &b.land_line,
            &b.weather_line,
        ] {
            small(ui, l);
        }
        match &state.loadout {
            Some(lo) => small(ui, &t!("summary-armament", stores = lo.summary())),
            None => small(
                ui,
                &t!(
                    "summary-armament",
                    stores =
                        b.armament.iter().map(|a| clean_text(a)).collect::<Vec<_>>().join(", ")
                ),
            ),
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new(t!("summary-intelligence")).small().color(AMBER));
        for l in [&b.primary_summary, &b.secondary_summary, &b.roe_text] {
            small(ui, l);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The desktop window and larger keep the full size; a phone on its side zooms out to fit
    /// its height, never below the floor.
    #[test]
    fn zoom_fits_the_window() {
        assert_eq!(fit_zoom(egui::vec2(1920.0, 1080.0)), Some(1.0));
        assert_eq!(fit_zoom(UI_REFERENCE), Some(1.0));
        let phone = fit_zoom(egui::vec2(844.0, 390.0)).unwrap();
        assert!((phone - 390.0 / 768.0).abs() < 1e-4);
        assert_eq!(fit_zoom(egui::vec2(300.0, 200.0)), Some(UI_ZOOM_MIN));
        assert_eq!(fit_zoom(egui::Vec2::ZERO), None);
    }
}
