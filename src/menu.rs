//! Main menu: theater selector and mission generation.

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, EguiTextureHandle,
    PrimaryEguiContext, egui,
};

use stealth_mission::{Mission, MissionKind, Sky};

use crate::aircraft::AircraftKind;
use crate::mission::{ActiveMission, MissionData, clean_text};
use crate::settings::{AppState, MissionSetup};

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

/// Flyable aircraft and their notes.
const AIRCRAFT: [(AircraftKind, &[&str]); 10] = [
    (
        AircraftKind::F117,
        &["Stealth attack aircraft", "Subsonic, 7 g", "Modern and classic cockpit"],
    ),
    (
        AircraftKind::F16,
        &["Multirole fighter", "Mach 2 class, 9 g, quick roll", "F-16 cockpit with RWR"],
    ),
    (
        AircraftKind::Tornado,
        &[
            "Low-level strike aircraft",
            "Swing wings sweep with speed",
            "Modern cockpit, three MFDs",
        ],
    ),
    (
        AircraftKind::A10,
        &["Close air support aircraft", "Slow and rugged, 30 mm cannon", "A-10C cockpit, two MFDs"],
    ),
    (
        AircraftKind::AV8B,
        &[
            "V/STOL attack aircraft",
            "Vectored thrust: hover, land anywhere",
            "Nozzle lever: R / F, Shift for the stops",
        ],
    ),
    (
        AircraftKind::UH60,
        &["Utility helicopter", "Inserts troops: land near the target", "Cyclic and collective"],
    ),
    (
        AircraftKind::AH1Z,
        &["Attack helicopter", "Turret gun, Hellfires, AIM-9", "Gun and Hellfires follow the pod"],
    ),
    (
        AircraftKind::RAH66,
        &[
            "Stealth scout helicopter",
            "Bays: Hellfires or Stingers",
            "Hard to see on radar, slow or hovering",
        ],
    ),
    (
        AircraftKind::OH58,
        &[
            "Armed scout helicopter, nimble",
            "Pylons: Hellfires, Stingers or .50 cal pods",
            "Mast-mounted sight, helmet display",
        ],
    ),
    (
        AircraftKind::CH47,
        &[
            "Tandem-rotor cargo helicopter",
            "Slings resupply and logistics loads",
            "No torque to hold: the rotors cancel",
        ],
    ),
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
        (!settings.realistic_terrain, "--classic-terrain"),
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

// 80s military terminal: green phosphor on black, the in-flight menu's greens
// (`flight_menu::colours`).
pub(crate) const BG: egui::Color32 = egui::Color32::from_rgb(5, 18, 13);
pub(crate) const FIELD: egui::Color32 = egui::Color32::from_rgb(9, 29, 21);
const HOVER: egui::Color32 = egui::Color32::from_rgb(16, 69, 47);
/// Frames of fields, buttons and groups.
pub(crate) const FRAME: egui::Color32 = egui::Color32::from_rgb(19, 84, 56);
pub(crate) const GREEN: egui::Color32 = egui::Color32::from_rgb(108, 251, 174);
pub(crate) const DIM: egui::Color32 = egui::Color32::from_rgb(27, 136, 82);
pub(crate) const BRIGHT: egui::Color32 = egui::Color32::from_rgb(183, 255, 216);
pub(crate) const AMBER: egui::Color32 = egui::Color32::from_rgb(255, 176, 0);
/// The panels' glowing edge; a lit button's fill from top to bottom.
const EDGE: egui::Color32 = egui::Color32::from_rgb(61, 231, 140);
const LIT_TOP: egui::Color32 = egui::Color32::from_rgb(31, 156, 91);
const LIT_BOTTOM: egui::Color32 = egui::Color32::from_rgb(23, 111, 70);

/// Width of the VEHICLE column (the aircraft preview above the choices).
pub(crate) const VEHICLE_WIDTH: f32 = 260.0;

/// Hover facts per theater ([`THEATERS`] order).
const THEATER_FACTS: [&[&str]; 10] = [
    &[
        "Gulf of Sidra and the Libyan coast",
        "Era of Operation El Dorado Canyon",
        "Clear desert skies most days",
    ],
    &[
        "Iran-Iraq Tanker War era",
        "Over water and coastal desert",
        "Clearest weather of all theaters",
    ],
    &[
        "Arctic Norway and the Kola Peninsula",
        "Home of the Soviet Northern Fleet",
        "Frequent low overcast",
    ],
    &["NATO vs Warsaw Pact over Germany", "Clear skies only 15% of the time"],
    &[
        "Hanoi and Haiphong region",
        "Jungle and mountain terrain",
        "Thunderstorms in 30% of missions",
    ],
    &["Eastern Mediterranean and the Levant", "Mostly clear skies"],
    &["Kuwait and Iraq, 1991", "The F-117A's real combat debut", "Mostly clear desert skies"],
    &["Caribbean island theater", "Sea and land targets", "Tropical: storms in 15% of missions"],
    &[
        "Korean peninsula north of the DMZ",
        "Mountainous terrain",
        "Changeable weather, any sky is likely",
    ],
    &[
        "Eastern Ukraine, Crimea and southern Russia",
        "Front line from Kharkiv to Kherson burns",
        "Open farmland, ruined towns at the front",
    ],
];
const ROE: [(&str, &[&str]); 3] = [
    (
        "Cold War",
        &[
            "Covert peacetime operation",
            "Staying undetected matters most",
            "Fewer enemy aircraft on patrol",
            "Exact rules of engagement are in the briefing",
        ],
    ),
    (
        "Limited War",
        &[
            "Regional shooting conflict",
            "Enemy air defenses on alert",
            "More enemy aircraft than in Cold War",
        ],
    ),
    (
        "Conventional War",
        &["Full-scale war", "All enemy forces active", "More enemy aircraft than in Cold War"],
    ),
];
const KINDS: [(MissionKind, &str, &[&str]); 5] = [
    (
        MissionKind::Strike,
        "Strike",
        &[
            "Destroy, photograph or reach a ground target",
            "Primary and secondary objective",
            "Air-to-ground loadout",
        ],
    ),
    (
        MissionKind::Air,
        "Air to air",
        &[
            "Intercept an enemy aircraft",
            "Transports, bombers or fighter patrols",
            "Air-to-air missile loadout",
        ],
    ),
    (
        MissionKind::StrikeTraining,
        "Strike training",
        &["Practice strike mission", "Learn weapons and navigation"],
    ),
    (
        MissionKind::AirTraining,
        "Air training",
        &["Practice air-to-air mission", "Learn intercepts and missiles"],
    ),
    (
        MissionKind::DroneDefense,
        "Drone defense",
        &[
            "Ukraine only",
            "Stop a wave of Geran attack drones before it hits a city or a nuclear plant",
            "Then bomb their launch site",
        ],
    ),
];

/// Theaters with drone launch sites (drone defense missions).
const DRONE_THEATERS: [&str; 1] = ["ukraine"];
const OPPONENTS: [(&str, &[&str]); 4] = [
    ("Green", &["Inexperienced enemy crews", "Easiest setting"]),
    ("Regular", &["Average enemy crews", "Default setting"]),
    ("Veteran", &["Skilled enemy crews", "More ships and patrols"]),
    ("Elite", &["Best enemy pilots and SAM crews", "Hardest setting"]),
];
const RANDOM_SKY_FACTS: &[&str] = &[
    "Rolled from the theater's climate",
    "Changes with each new mission",
    "Clouds block line of sight for both sides",
];

fn sky_facts(sky: Sky) -> &'static [&'static str] {
    match sky {
        Sky::Clear => &["No clouds", "Visibility 40 km", "The classic DOS era sky"],
        Sky::Scattered => &[
            "Cumulus 20-45%, base 2500-6000 ft",
            "Sometimes a thin altostratus high above",
            "Visibility 30 km",
        ],
        Sky::Broken => &[
            "Cumulus 45-70%, base 2000-5000 ft",
            "Often a stratocumulus layer above",
            "Visibility 25 km",
        ],
        Sky::Overcast => &[
            "Stratus 80-100%, base 1500-5000 ft",
            "A few cumulus fragments below",
            "Visibility 20 km",
        ],
        Sky::Storm => &[
            "Cumulonimbus towers up to 40000 ft",
            "Low stratus and cumulus from 1000 ft",
            "Visibility 8 km",
        ],
    }
}

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
    v.window_stroke = Stroke::new(2.0, EDGE);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.selection.bg_fill = LIT_BOTTOM;
    v.selection.stroke = Stroke::new(1.0, BRIGHT);
    for (w, bg, fg, edge) in [
        (&mut v.widgets.noninteractive, BG, GREEN, FRAME),
        (&mut v.widgets.inactive, FIELD, GREEN, FRAME),
        (&mut v.widgets.hovered, HOVER, BRIGHT, EDGE),
        (&mut v.widgets.active, LIT_BOTTOM, BRIGHT, EDGE),
        (&mut v.widgets.open, FIELD, GREEN, FRAME),
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

pub(crate) fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(format!("[ {} ]", title.to_ascii_uppercase())).color(AMBER).size(14.0),
    );
}

/// A framed group of sections under a bright title, filling the column.
pub(crate) fn group<R>(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let r = egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, FRAME))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(title.to_ascii_uppercase()).size(16.0).color(BRIGHT));
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
    two_way_switch(ui, label, modern, ("CLASSIC", "MODERN"))
}

/// A labelled slide switch between two settings (`sides`: left, right); `right` is the right
/// position.
fn two_way_switch(
    ui: &mut egui::Ui,
    label: &str,
    modern: &mut bool,
    sides: (&str, &str),
) -> egui::Response {
    let width = ui.available_width().min(200.0);
    let (rect, mut resp) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::click());
    if resp.clicked() {
        *modern = !*modern;
        resp.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool_responsive(resp.id, *modern);
        let hovered = resp.hovered();
        let p = ui.painter();
        let font = egui::FontId::monospace(12.0);
        p.text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            label.to_ascii_uppercase(),
            font.clone(),
            if hovered { BRIGHT } else { GREEN },
        );
        // The track: two halves, the knob slides over the selected one.
        let track = egui::Rect::from_min_max(
            egui::pos2(rect.right() - 108.0, rect.top()),
            rect.right_bottom(),
        );
        p.rect_filled(track, 0.0, FIELD);
        p.rect_stroke(
            track,
            0.0,
            egui::Stroke::new(1.0, if hovered { BRIGHT } else { DIM }),
            egui::StrokeKind::Inside,
        );
        let half = track.width() / 2.0;
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
                egui::FontId::monospace(11.0),
                if on { BRIGHT } else { DIM },
            );
        }
    }
    resp
}

pub(crate) fn big_button(text: &str) -> BigButton {
    BigButton {
        text: text.to_string(),
        min_size: egui::vec2(170.0, 50.0),
        fill: None,
        primary: false,
        icon: None,
        left: false,
    }
}

/// The icon font's family in egui (`add_icon_font`).
static ICON_FAMILY: std::sync::LazyLock<egui::FontFamily> =
    std::sync::LazyLock::new(|| egui::FontFamily::Name("lucide".into()));

/// Whether `ctx` has the icon font yet (`add_icon_font`): painting with it before would panic.
fn has_icon_font(ctx: &egui::Context) -> bool {
    ctx.fonts(|f| f.definitions().families.contains_key(&*ICON_FAMILY))
}

/// Gives `ctx` the in-flight menu's icon font, from its next pass on (`BigButton::icon`,
/// `icon_text`).
pub(crate) fn add_icon_font(ctx: &egui::Context) {
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
    if has_icon_font(ctx) {
        return;
    }
    let Some(bytes) = *crate::flight_menu::ICON_FONT else { return };
    ctx.add_font(FontInsert::new(
        "lucide",
        egui::FontData::from_static(bytes),
        vec![InsertFontFamily { family: ICON_FAMILY.clone(), priority: FontPriority::Highest }],
    ));
}

/// A small button's text after its icon (as `BigButton::icon`), in the text's size and colour;
/// the text alone before the icon font is added.
pub(crate) fn icon_text(
    ui: &egui::Ui,
    icon: char,
    text: impl Into<egui::RichText>,
) -> egui::WidgetText {
    let text = text.into();
    if !has_icon_font(ui.ctx()) {
        return text.into();
    }
    let mut job = egui::text::LayoutJob::default();
    job.append(&icon.to_string(), 0.0, egui::TextFormat::default());
    let font = egui::FontSelection::Style(egui::TextStyle::Button);
    text.append_to(&mut job, ui.style(), font, egui::Align::Center);
    let Some(format) = job.sections.get(1).map(|s| s.format.clone()) else { return job.into() };
    job.sections[1].leading_space = 6.0;
    job.sections[0].format = egui::TextFormat {
        font_id: egui::FontId::new(format.font_id.size, ICON_FAMILY.clone()),
        ..format
    };
    job.into()
}

/// Lucide icons (code points) of the menus' buttons, one per button name: the same everywhere
/// (the in-flight menu's too).
pub(crate) mod icon {
    pub(crate) const SINGLE_MISSION: char = '\u{e0ac}'; // crosshair
    pub(crate) const CAREER: char = '\u{e36f}'; // medal
    pub(crate) const CAMPAIGN: char = '\u{e110}'; // map
    pub(crate) const MULTIPLAYER: char = '\u{e1a4}'; // users
    pub(crate) const OPTIONS: char = '\u{e29a}'; // sliders-horizontal
    pub(crate) const QUIT: char = '\u{e140}'; // power
    #[cfg(target_arch = "wasm32")]
    pub(crate) const BROWSER: char = '\u{e0e8}'; // globe
    #[cfg(target_arch = "wasm32")]
    pub(crate) const DOWNLOAD: char = '\u{e0b2}'; // download
    #[cfg(target_arch = "wasm32")]
    pub(crate) const DISCORD: char = '\u{e116}'; // message-circle
    #[cfg(target_arch = "wasm32")]
    pub(crate) const GITHUB: char = '\u{e28d}'; // git-fork
    pub(crate) const BACK: char = '\u{e048}'; // arrow-left
    /// NEW MISSION, NEW SORTIE.
    pub(crate) const NEW_MISSION: char = '\u{e2c5}'; // dices
    pub(crate) const LOAD: char = '\u{e247}'; // folder-open
    /// FLY, FLY NEXT SORTIE, TAKE OFF.
    pub(crate) const FLY: char = '\u{e3ce}'; // plane-takeoff
    /// FLY IN VR, IN VR.
    pub(crate) const VR: char = '\u{e656}'; // rectangle-goggles
    pub(crate) const DELETE: char = '\u{e18e}'; // trash-2
    pub(crate) const CANCEL: char = '\u{e1b2}'; // x
    /// START, START TOUR, PLAY.
    pub(crate) const START: char = '\u{e13c}'; // play
    pub(crate) const PAUSE: char = '\u{e12e}'; // pause
    pub(crate) const CONTINUE: char = '\u{e073}'; // chevrons-right
    pub(crate) const REARM: char = '\u{e2af}'; // fuel
    /// END SORTIE, END MISSION.
    pub(crate) const END_SORTIE: char = '\u{e10e}'; // log-out
    /// SAVE & EXIT, SAVE MISSION.
    pub(crate) const SAVE: char = '\u{e14d}'; // save
    pub(crate) const DISCONNECT: char = '\u{e45d}'; // unplug
    /// ORDER INSERTION, FLY THE UH-60.
    pub(crate) const HELICOPTER: char = '\u{e69d}'; // helicopter
    pub(crate) const ACCEPT: char = '\u{e07c}'; // circle-check
    /// CONTROL, ENTER: man an air defence station.
    pub(crate) const CONTROL: char = '\u{e497}'; // radar
    pub(crate) const SEND: char = '\u{e152}'; // send
    pub(crate) const BRIEFED: char = '\u{e086}'; // clipboard-list
    pub(crate) const RESULTS: char = '\u{e106}'; // list
    pub(crate) const FLIGHT_MAP: char = CAMPAIGN;
    /// CLEAR, CLEAR ALL.
    pub(crate) const CLEAR: char = '\u{e28f}'; // eraser
    pub(crate) const DONE: char = '\u{e06c}'; // check
    /// RESET, DEFAULTS, DEFAULT KEYS.
    pub(crate) const RESET: char = '\u{e148}'; // rotate-ccw
    /// HOST, HOST NEW, HOST A GAME.
    pub(crate) const HOST: char = '\u{e153}'; // server
    /// JOIN, CONNECT.
    pub(crate) const JOIN: char = '\u{e10d}'; // log-in
    pub(crate) const REFRESH: char = '\u{e145}'; // refresh-cw
    pub(crate) const ENLIST: char = '\u{e1a2}'; // user-plus
    /// KEYS AND JOYSTICK, the in-flight menu's CONTROLS.
    pub(crate) const KEYS: char = '\u{e355}'; // joystick
    pub(crate) const PLAN_ROUTE: char = '\u{e53e}'; // route
    pub(crate) const LOADOUT: char = '\u{e2ff}'; // bomb
}

/// A framed button that lights up (a green gradient, a glowing frame) under the mouse, as the
/// in-flight menu's highlighted row; a primary one (FLY, CONTINUE) is always lit.
pub(crate) struct BigButton {
    text: String,
    min_size: egui::Vec2,
    /// A flat colour of its own instead (a red DELETE).
    fill: Option<egui::Color32>,
    primary: bool,
    /// A Lucide icon before the label.
    icon: Option<char>,
    /// The icon and the label at the left instead of centred.
    left: bool,
}

/// A laid out text.
type Galley = std::sync::Arc<egui::Galley>;

/// A big button's label size, the space between its icon and its label, and (left-aligned)
/// between its left edge and its icon.
const BIG_SIZE: f32 = 22.0;
const BIG_GAP: f32 = 12.0;
const BIG_INDENT: f32 = 20.0;

impl BigButton {
    pub(crate) fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    pub(crate) fn min_size(mut self, size: egui::Vec2) -> Self {
        self.min_size = size;
        self
    }

    pub(crate) fn fill(mut self, fill: egui::Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub(crate) fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    pub(crate) fn left(mut self) -> Self {
        self.left = true;
        self
    }

    /// The label and the icon laid out, and how far the label is after the icon's start.
    fn layout(&self, ctx: &egui::Context) -> (Galley, Option<Galley>, f32) {
        let layout = |text: String, font| {
            ctx.fonts_mut(|f| f.layout(text, font, egui::Color32::PLACEHOLDER, f32::INFINITY))
        };
        let galley = layout(self.text.clone(), egui::FontId::monospace(BIG_SIZE));
        // Before its font is added (the first pass) there is no icon.
        let icon = self
            .icon
            .filter(|_| has_icon_font(ctx))
            .map(|icon| layout(icon.to_string(), egui::FontId::new(BIG_SIZE, ICON_FAMILY.clone())));
        let lead = icon.as_ref().map_or(0.0, |g| g.size().x + BIG_GAP);
        (galley, icon, lead)
    }

    /// The width its icon and label need (at least `min_size`'s).
    pub(crate) fn width(&self, ctx: &egui::Context) -> f32 {
        let (galley, _, lead) = self.layout(ctx);
        let padding = ctx.global_style().spacing.button_padding.x;
        let indent = if self.left { BIG_INDENT } else { padding };
        (galley.size().x + lead + 2.0 * indent).max(self.min_size.x)
    }
}

impl egui::Widget for BigButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (galley, icon, lead) = self.layout(ui.ctx());
        let padding = ui.spacing().button_padding;
        let size =
            egui::vec2(self.width(ui.ctx()), galley.size().y + 2.0 * padding.y).max(self.min_size);
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        let enabled = ui.is_enabled();
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &self.text)
        });
        if !ui.is_rect_visible(rect) {
            return resp;
        }
        let p = ui.painter();
        let hovered = enabled && resp.hovered();
        let pressed = hovered && resp.is_pointer_button_down_on();
        let text = match self.fill {
            _ if !enabled => DIM,
            Some(fill) => {
                let fill =
                    if hovered { fill.lerp_to_gamma(egui::Color32::WHITE, 0.15) } else { fill };
                p.rect_filled(rect, 0.0, fill);
                let edge = if hovered { BRIGHT } else { FRAME };
                p.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, edge), egui::StrokeKind::Inside);
                BRIGHT
            }
            None if self.primary || hovered => {
                let (top, bottom) =
                    if pressed { (LIT_BOTTOM, LIT_TOP) } else { (LIT_TOP, LIT_BOTTOM) };
                let mut mesh = egui::Mesh::default();
                for (corner, colour) in [
                    (rect.left_top(), top),
                    (rect.right_top(), top),
                    (rect.right_bottom(), bottom),
                    (rect.left_bottom(), bottom),
                ] {
                    mesh.colored_vertex(corner, colour);
                }
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                p.add(mesh);
                let glow = EDGE.gamma_multiply(if hovered { 0.45 } else { 0.25 });
                p.rect_stroke(rect, 0.0, egui::Stroke::new(4.0, glow), egui::StrokeKind::Outside);
                p.rect_stroke(rect, 0.0, egui::Stroke::new(1.5, EDGE), egui::StrokeKind::Inside);
                BRIGHT
            }
            None => {
                p.rect_filled(rect, 0.0, FIELD);
                p.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, FRAME), egui::StrokeKind::Inside);
                GREEN
            }
        };
        // The icon then the label, centred together (or at the left).
        let left = if self.left {
            rect.left() + BIG_INDENT
        } else {
            rect.center().x - (lead + galley.size().x) / 2.0
        };
        if let Some(icon) = icon {
            let at = egui::pos2(left, rect.center().y - icon.size().y / 2.0);
            p.galley(at, icon, text);
        }
        let at = egui::pos2(left + lead, rect.center().y - galley.size().y / 2.0);
        p.galley(at, galley, text);
        resp
    }
}

/// A window drawn as a terminal panel, as the in-flight menu's screen: cut corners and a
/// glowing edge, the title in a slanted tab at the top-left (after two slanted marks), a dim
/// note at the top-right, a stepped rule under them.
pub(crate) struct Panel<'a> {
    title: &'a str,
    /// After the title, in the tab (a career sortie's tour).
    after: Option<egui::RichText>,
    note: String,
}

/// Space between a panel's edge and its contents, how far its corners are cut, how far in its
/// edge is (room for the glow, inside the window).
const PANEL_MARGIN: f32 = 16.0;
const PANEL_CUT: f32 = 16.0;
const PANEL_INSET: f32 = 3.0;
/// The logo's width before the title; how far the title tab reaches past the title and how far
/// its right side slants.
const PANEL_LOGO: f32 = 30.0;
const PANEL_TAB: f32 = 24.0;
const PANEL_SLANT: f32 = 18.0;

/// Where a panel's header put the logo, the right end of the title and the header's row.
type Head = (egui::Rect, f32, egui::Rect);

impl<'a> Panel<'a> {
    pub(crate) fn new(title: &'a str) -> Self {
        Self { title, after: None, note: String::new() }
    }

    pub(crate) fn after(mut self, text: egui::RichText) -> Self {
        self.after = Some(text);
        self
    }

    pub(crate) fn note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    /// The narrowest window whose title tab still ends before its cut corner (the title alone,
    /// without `after` or the note).
    pub(crate) fn min_width(&self, ctx: &egui::Context) -> f32 {
        let style = ctx.global_style();
        let font = egui::TextStyle::Heading.resolve(&style);
        let title = ctx.fonts_mut(|f| {
            f.layout(self.title.to_string(), font, egui::Color32::PLACEHOLDER, f32::INFINITY)
        });
        let end = PANEL_MARGIN + PANEL_LOGO + style.spacing.item_spacing.x + title.size().x;
        end + PANEL_TAB + PANEL_SLANT + PANEL_CUT + PANEL_INSET + 8.0
    }

    /// Shows `window` (its frame replaced) with the header and `add` on the panel, painted
    /// behind them at the window's size.
    pub(crate) fn window<R>(
        self,
        window: egui::Window<'_>,
        ctx: &egui::Context,
        add: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<egui::InnerResponse<Option<R>>> {
        let mut back = None;
        let frame = egui::Frame::new().inner_margin(PANEL_MARGIN);
        let (title, after) = (self.title, self.after);
        let r = window.frame(frame).show(ctx, |ui| {
            let shape = ui.painter().add(egui::Shape::Noop);
            back = Some((shape, header(ui, title, after)));
            add(ui)
        })?;
        // The note and the marks at the right of the window as it came out (contents wider
        // than the window's width widen it).
        if let Some((shape, head)) = back {
            let painter = ctx.layer_painter(r.response.layer_id);
            let note = painter.layout_no_wrap(self.note, egui::FontId::monospace(15.0), DIM);
            painter.set(shape, panel_shapes(r.response.rect, head, note));
        }
        Some(r)
    }
}

/// A panel's header row: the room for the logo, the title, `after` it.
fn header(ui: &mut egui::Ui, title: &str, after: Option<egui::RichText>) -> Head {
    let row = ui.horizontal(|ui| {
        let (logo, _) = ui.allocate_exact_size(egui::vec2(PANEL_LOGO, 24.0), egui::Sense::hover());
        let mut end = ui.label(egui::RichText::new(title).heading().color(BRIGHT)).rect.right();
        if let Some(after) = after {
            end = ui.label(after).rect.right();
        }
        (logo, end)
    });
    ui.add_space(12.0);
    let (logo, end) = row.inner;
    (logo, end, row.response.rect)
}

/// Two slanted bars in `rect`, leaning right (`lean` > 0) or left.
fn slashes(rect: egui::Rect, lean: f32, colour: egui::Color32) -> [egui::Shape; 2] {
    let w = rect.width() * 0.3;
    std::array::from_fn(|i| {
        let x = if lean > 0.0 { rect.left() } else { rect.left() - lean } + i as f32 * w * 1.6;
        let (b, t) = (rect.bottom(), rect.top());
        egui::Shape::convex_polygon(
            vec![
                egui::pos2(x, b),
                egui::pos2(x + w, b),
                egui::pos2(x + w + lean, t),
                egui::pos2(x + lean, t),
            ],
            colour,
            egui::Stroke::NONE,
        )
    })
}

/// A panel's background, tab, note, marks, rule and edge in the window's `rect`.
fn panel_shapes(
    rect: egui::Rect,
    (logo, end, row): Head,
    note: std::sync::Arc<egui::Galley>,
) -> egui::Shape {
    use egui::{Shape, Stroke, pos2};
    let rule = row.bottom() + 6.0;
    let marks = egui::Rect::from_min_max(
        pos2(rect.right() - PANEL_MARGIN - 30.0, row.center().y - 9.0),
        pos2(rect.right() - PANEL_MARGIN, row.center().y + 9.0),
    );
    let r = rect.shrink(PANEL_INSET);
    let k = PANEL_CUT;
    let outline = vec![
        pos2(r.left() + k, r.top()),
        pos2(r.right() - k, r.top()),
        pos2(r.right(), r.top() + k),
        pos2(r.right(), r.bottom() - k),
        pos2(r.right() - k, r.bottom()),
        pos2(r.left() + k, r.bottom()),
        pos2(r.left(), r.bottom() - k),
        pos2(r.left(), r.top() + k),
    ];
    let slant = PANEL_SLANT;
    let tab = end + PANEL_TAB + slant;
    let line = Stroke::new(1.5, DIM);
    // The rule steps up past the tab, where there is room.
    let step = (r.right() - 190.0).max(tab + 30.0);
    let rule_line = if step + 40.0 < r.right() {
        vec![
            pos2(r.left(), rule),
            pos2(step, rule),
            pos2(step + 10.0, rule - 10.0),
            pos2(r.right(), rule - 10.0),
        ]
    } else {
        vec![pos2(r.left(), rule), pos2(r.right(), rule)]
    };
    let mut shapes = vec![
        Shape::convex_polygon(outline.clone(), BG, Stroke::NONE),
        Shape::convex_polygon(
            vec![
                pos2(r.left() + k, r.top()),
                pos2(tab, r.top()),
                pos2(tab - slant, rule),
                pos2(r.left(), rule),
                pos2(r.left(), r.top() + k),
            ],
            FIELD,
            Stroke::NONE,
        ),
        Shape::line_segment([pos2(tab, r.top()), pos2(tab - slant, rule)], line),
        Shape::line(rule_line, line),
    ];
    shapes.extend(slashes(logo.shrink2(egui::vec2(0.0, 2.0)), 10.0, GREEN));
    // Each where there is room right of the tab.
    if tab + 10.0 < marks.left() {
        shapes.extend(slashes(marks, -9.0, GREEN));
    }
    let at = pos2(marks.left() - 10.0 - note.size().x, row.center().y - note.size().y / 2.0);
    if tab + 16.0 < at.x {
        shapes.push(Shape::galley(at, note, DIM));
    }
    shapes.push(Shape::closed_line(outline.clone(), Stroke::new(6.0, EDGE.gamma_multiply(0.22))));
    shapes.push(Shape::closed_line(outline, Stroke::new(2.0, EDGE)));
    Shape::Vec(shapes)
}

/// The rule above a panel's buttons, stepping up towards the right as the in-flight menu's
/// footer.
pub(crate) fn footer_rule(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 14.0), egui::Sense::hover());
    let (y, step) = (rect.bottom() - 2.0, rect.right() - 190.0);
    ui.painter().line(
        vec![
            egui::pos2(rect.left(), y),
            egui::pos2(step, y),
            egui::pos2(step + 10.0, y - 10.0),
            egui::pos2(rect.right(), y - 10.0),
        ],
        egui::Stroke::new(1.5, DIM),
    );
}

type SettingsMut<'a> = ResMut<'a, crate::settings::Settings>;

pub(crate) fn theater_options(ui: &mut egui::Ui, theater: &mut usize) {
    section(ui, "Theater");
    for (i, (_, name)) in THEATERS.iter().enumerate() {
        facts(ui.radio_value(theater, i, *name), THEATER_FACTS[i]);
    }
}

pub(crate) fn opponent_options(ui: &mut egui::Ui, opponents: &mut u8) {
    section(ui, "Opponents");
    let options = OPPONENTS.iter().enumerate().map(|(i, (name, f))| (i as u8, *name, *f));
    dropdown(ui, "opponents", opponents, options);
}

pub(crate) fn situation_options(ui: &mut egui::Ui, roe: &mut u8) {
    section(ui, "Situation");
    let options = ROE.iter().enumerate().map(|(i, (name, f))| (i as u8, *name, *f));
    dropdown(ui, "situation", roe, options);
}

fn mission_options(ui: &mut egui::Ui, state: &mut MenuState) {
    section(ui, "Mission");
    let drones = DRONE_THEATERS.contains(&THEATERS[state.theater].0);
    if !drones && state.kind == MissionKind::DroneDefense {
        state.kind = MissionKind::Air;
    }
    let options = KINDS.into_iter().filter(|k| k.0 != MissionKind::DroneDefense || drones);
    dropdown(ui, "mission", &mut state.kind, options);
}

/// The moon phases to choose from, in eighths of the cycle.
const MOON_PHASES: [(u8, &str, &[&str]); 8] = [
    (
        0,
        "New moon",
        &["No moonlight: a black night, lights and stars only", "Enemy pilots see you at 2 km"],
    ),
    (1, "Waxing crescent", &["A thin moon in the evening sky, set before midnight"]),
    (2, "First quarter", &["Half lit, up in the evening, sets around midnight"]),
    (3, "Waxing gibbous", &["Bright, up from the afternoon until after midnight"]),
    (4, "Full moon", &["Up all night, high at midnight", "Enemy pilots see you at 4 km"]),
    (5, "Waning gibbous", &["Bright, rises in the evening, up until the morning"]),
    (6, "Last quarter", &["Half lit, rises around midnight"]),
    (7, "Waning crescent", &["A thin moon rising before dawn"]),
];

/// The moon's phase (with the time of day switch on).
pub(crate) fn moon_options(ui: &mut egui::Ui, moon: &mut Option<u8>) {
    section(ui, "Moon");
    let date: &[&str] =
        &["The real moon of the briefing date", "Needs the time of day switch (modern)"];
    let options = std::iter::once((None, "Mission date", date))
        .chain(MOON_PHASES.into_iter().map(|(m, name, f)| (Some(m), name, f)));
    dropdown(ui, "moon", moon, options);
}

fn start_time_options(ui: &mut egui::Ui, state: &mut MenuState) {
    section(ui, "Start time");
    let mut random = state.start_hour.is_none();
    let r = facts(
        ui.checkbox(&mut random, "Random"),
        &[
            "Rolled with each new mission, as in classic mode",
            "06:00-08:00 by day or 18:00-20:00 by night",
            "Uncheck to choose the hour",
        ],
    );
    if r.changed() {
        state.start_hour = if random { None } else { Some(8) };
    }
    if let Some(h) = state.start_hour.as_mut() {
        let slider = egui::Slider::new(h, 0..=23).custom_formatter(|v, _| format!("{v:02.0}:00"));
        facts(
            ui.add(slider),
            &["Takeoff time", "Night from 18:00 to 06:00", "Kept by NEW MISSION"],
        );
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
    for (kind, f) in AIRCRAFT {
        picked |= facts(ui.radio_value(&mut s.aircraft, kind, kind.name()), f).changed();
    }
    if picked {
        settings.set_changed();
    }
    // The F-16 carries AIM-9s and AIM-120s on its pylons anyway.
    if settings.aircraft == AircraftKind::F117
        && let Some(lockheed) = lockheed
    {
        facts(
            ui.checkbox(lockheed, "Realistic weapons"),
            &[
                "The real aircraft's limits",
                "2 weapon bays instead of 4",
                "No AIM-9 or AIM-120 missiles",
                "No carrier takeoffs or landings",
            ],
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
    group(ui, "Simulation", |ui| {
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
        ui.checkbox(&mut s.no_crashes, "Crash protection"),
        &[
            "Pulls up automatically near the ground (gear up)",
            "Ground contact and collisions are not fatal",
            "Off: the ground can always be hit",
        ],
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
    section(ui, "Tacview");
    let s = settings.bypass_change_detection();
    if facts(
        ui.checkbox(&mut s.tacview, "Export"),
        &[
            "Saves each flight as a Tacview recording (.acmi)",
            "In the game's folder for your files, under tacview",
        ],
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        ui.checkbox(&mut s.tacview_real_time, "Real-time telemetry"),
        &[
            "Sends the flight to Tacview while you fly",
            "In Tacview: Record > Real-Time Telemetry",
            "Address 127.0.0.1, port 42674 (--tacview-address)",
        ],
    )
    .changed()
    {
        settings.set_changed();
    }
}

pub(crate) fn realism_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, "Realism");
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(ui, "Damage", &mut s.realistic_damage, ("GAME", "REAL")),
        &[
            "Weapon damage", "Game: classic damage; hits add up but never bring the aircraft down",
            "Realistic: a missile hit can destroy the aircraft (eject!), damages more systems and often starts a fire, which burns out or blows the aircraft up",
            "Realistic: damaged control surfaces put the aircraft out of trim or make it oscillate",
        ],
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(ui, "Guidance", &mut s.realistic_guidance, ("CLASSIC", "REAL")),
        &[
            "Missile guidance", "Classic: missiles flying straight at their target",
            "Realistic: proportional navigation to the collision point; motors burn out and the missiles glide on, losing speed with every turn (smoke only while the motor burns)",
            "Realistic: SAMs loft against distant targets; the AIM-120 flies straight for 3 s (pull up to lob it)",
        ],
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(ui, "Terrain", &mut s.realistic_terrain, ("CLASSIC", "REAL")),
        &[
            "Terrain elevation", "Classic: the flat ground of the original game (mountains stay)",
            "Realistic: rolling hills and higher highlands; ground units, SAM sites and pilots can hide behind them",
        ],
    )
    .changed()
    {
        settings.set_changed();
    }
    let s = settings.bypass_change_detection();
    if facts(
        two_way_switch(ui, "Air defence", &mut s.air_defence, ("CLASSIC", "REAL")),
        &[
            "Low-level air defences", "Classic: only the SAM sites and fighters",
            "Realistic: radar-laid ZSU-23-4 guns (their radar sees you like a SAM's) and S-60 flak batteries guard the airfields and the targets; MANPADS teams in the towns fire SA-14s at what they see",
            "Flying low over defended places becomes dangerous; bombs and guns destroy the guns",
        ],
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
    section(ui, "Weather");
    let options = std::iter::once((None, "Random", RANDOM_SKY_FACTS))
        .chain(Sky::ALL.into_iter().map(|sky| (Some(sky), sky.name(), sky_facts(sky))));
    dropdown(ui, "weather", weather, options);
    // Coverage of the main layer (cumulus, or stratus for overcast and storms).
    let mut auto = cloud_cover.is_none();
    let r = facts(
        ui.checkbox(&mut auto, "Random cover"),
        &[
            "Main cloud layer cover rolled with the weather",
            "Uncheck to set it yourself",
            "Main layer: stratus for overcast and storm, cumulus otherwise",
        ],
    );
    if r.changed() {
        *cloud_cover = if auto { None } else { Some(50) };
    }
    if let Some(c) = cloud_cover.as_mut() {
        facts(
            ui.add(egui::Slider::new(c, 0..=100).suffix("%")),
            &["Share of the sky the main layer covers", "No effect on a clear sky"],
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

/// The game's source on GitHub.
#[cfg(target_arch = "wasm32")]
const GITHUB_URL: &str = "https://github.com/dividebysandwich/stealth-mission-89/";

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
const DOWNLOAD_GIVES: [(&str, &str); 7] = [
    ("VR", "Fly in VR on any OpenXR-compatible headset and the Steam Frame"),
    ("HOTAS", "Full support for gamepads, joysticks, throttles and pedals"),
    ("Better multiplayer", "Direct UDP connection to servers for less lag"),
    ("Host your own server", "Host a coop campaign, or run a dedicated server"),
    ("Head tracking", "TrackIR-style trackers through OpenTrack"),
    ("Tacview", "Flight recordings and real-time telemetry"),
    ("Yours to keep", "If the website goes down you still have your own copy"),
];

/// The browser build's first screen: play in the browser or download the desktop game, beside
/// what the download version adds.
#[cfg(target_arch = "wasm32")]
fn web_gateway(ctx: &egui::Context, state: &mut MenuState) {
    let wide = |text: &str| big_button(text).min_size(egui::vec2(300.0, 50.0));
    let width = (ctx.content_rect().width() - 32.0).clamp(600.0, 820.0);
    Panel::new("STEALTH MISSION 89").note("MISSION PLANNING SYSTEM").window(
        menu_window(ctx, "Stealth Mission 89", width),
        ctx,
        |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(300.0);
                    if ui.add(wide("PLAY IN BROWSER").icon(icon::BROWSER)).clicked() {
                        state.web_gateway = false;
                    }
                    ui.label(egui::RichText::new("Right here, nothing to install").color(DIM));
                    ui.add_space(14.0);
                    if ui.add(wide("DOWNLOAD").icon(icon::DOWNLOAD)).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(download_url()));
                    }
                    ui.label(
                        egui::RichText::new("Windows, Linux and macOS, Steam Deck and Steam Frame")
                            .color(DIM),
                    );
                    ui.add_space(14.0);
                    if ui.add(wide("GITHUB").icon(icon::GITHUB)).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(GITHUB_URL));
                    }
                    ui.label(egui::RichText::new("The source code").color(DIM));
                });
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    group(ui, "The download version", |ui| {
                        for (what, why) in DOWNLOAD_GIVES {
                            ui.label(egui::RichText::new(format!("> {what}")).color(GREEN));
                            ui.label(egui::RichText::new(why).size(14.0).color(DIM));
                            ui.add_space(2.0);
                        }
                    });
                });
            });
        },
    );
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
    // Every button as wide as the longest label needs, its icon and label at the left.
    let mut labels = vec![
        ("SINGLE MISSION", icon::SINGLE_MISSION),
        ("CAREER", icon::CAREER),
        ("DYNAMIC CAMPAIGN", icon::CAMPAIGN),
        ("MULTIPLAYER", icon::MULTIPLAYER),
        ("OPTIONS", icon::OPTIONS),
    ];
    #[cfg(target_arch = "wasm32")]
    labels.extend([("DOWNLOAD", icon::DOWNLOAD), ("DISCORD", icon::DISCORD)]);
    #[cfg(not(target_arch = "wasm32"))]
    labels.push(("QUIT", icon::QUIT));
    let width = labels
        .iter()
        .map(|&(text, i)| big_button(text).icon(i).left().width(ctx))
        .fold(0.0, f32::max);
    let wide = |text: &str| big_button(text).left().min_size(egui::vec2(width, 50.0));
    let panel = Panel::new("STEALTH MISSION 89");
    let window_width = (width + 60.0).max(panel.min_width(ctx));
    panel.window(menu_window(ctx, "Stealth Mission 89", window_width), ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("MISSION PLANNING SYSTEM").color(DIM));
            ui.add_space(10.0);
            if facts(
                ui.add(wide("SINGLE MISSION").icon(icon::SINGLE_MISSION)),
                &[
                    "One mission in the theater of your choice",
                    "Choose the options, plan the route and the loadout",
                ],
            )
            .clicked()
            {
                // Leaves career mode; the same mission again if the options are the same.
                state.career = None;
                state.dirty = true;
                state.mission_screen = true;
            }
            if facts(
                ui.add(wide("CAREER").icon(icon::CAREER)),
                &[
                    "Pilot records: rank, decorations, sorties",
                    "Tour of duty: 12 sorties in one theater",
                    "What you destroy stays destroyed",
                ],
            )
            .clicked()
            {
                career.show();
            }
            if facts(
                ui.add(wide("DYNAMIC CAMPAIGN").icon(icon::CAMPAIGN)),
                &[
                    "Dynamic campaign in real time",
                    "Take the enemy's cities and airfields",
                    "Missions from the live situation",
                ],
            )
            .clicked()
            {
                campaign.show();
            }
            // Browsers only join, through the master server's relay (docs/web.md).
            let how = if cfg!(target_arch = "wasm32") {
                "Join a server on the public list"
            } else {
                "Join a server, or host a campaign and fly in it"
            };
            if facts(
                ui.add(wide("MULTIPLAYER").icon(icon::MULTIPLAYER)),
                &["Coop in the dynamic campaign", how],
            )
            .clicked()
            {
                multiplayer.show();
            }
            if facts(
                ui.add(wide("OPTIONS").icon(icon::OPTIONS)),
                &[
                    "Game: cockpit, HUD and avionics style, auto trim",
                    "Graphics, sound volumes, key and joystick bindings",
                    "Saved for the next flights",
                ],
            )
            .clicked()
            {
                options.open = true;
            }
            // The browser build: the desktop game is a download (web/download.html).
            #[cfg(target_arch = "wasm32")]
            if facts(
                ui.add(wide("DOWNLOAD").icon(icon::DOWNLOAD)),
                &[
                    "The desktop game for Windows, Linux and macOS",
                    "With VR and multiplayer",
                    "Steam Deck and Steam Frame: the Linux AppImage",
                ],
            )
            .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(download_url()));
            }
            #[cfg(target_arch = "wasm32")]
            if facts(
                ui.add(wide("DISCORD").icon(icon::DISCORD)),
                &["Fly together, report bugs and share missions", "Follow development"],
            )
            .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(DISCORD_URL));
            }
            ui.add_space(10.0);
            // In the browser the tab is closed instead.
            if !cfg!(target_arch = "wasm32")
                && facts(ui.add(wide("QUIT").icon(icon::QUIT)), &["Exit to the desktop"]).clicked()
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
                egui::RichText::new(format!("VERSION {}{renderer}", env!("CARGO_PKG_VERSION")))
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
    mut saved: Local<Option<crate::prefs::Prefs>>,
    windows: (
        ResMut<crate::planner::Planner>,
        ResMut<crate::career::CareerWindow>,
        ResMut<crate::loadout::LoadoutWindow>,
        ResMut<crate::campaign::CampaignWindow>,
        ResMut<crate::multiplayer_menu::MultiplayerWindow>,
        ResMut<crate::options::OptionsWindow>,
        ResMut<crate::savegame::SavesWindow>,
        ResMut<crate::aircraft_preview::AircraftPreview>,
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
        mut preview,
    ) = windows;
    // Generator data is loaded on first use.
    let Some(data) = data else {
        match MissionData::load() {
            Ok(d) => commands.insert_resource(d),
            Err(e) => state.mission = Some(Err(e.to_string())),
        }
        return Ok(());
    };
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
                None => state.mission = Some(Err("No targets left standing for this tour".into())),
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

    // The aircraft preview's picture, once there is one.
    preview.texture =
        preview.image().map(|image| contexts.add_image(EguiTextureHandle::Strong(image)));
    let ctx = contexts.ctx_mut()?;
    apply_ui_zoom(ctx);
    add_icon_font(ctx);
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
        let mut panel = Panel::new(if career_mode { "CAREER SORTIE" } else { "SINGLE MISSION" })
            .note(format!("MISSION PLANNING SYSTEM  //  MSN {:05}", state.seed));
        if let Some(p) = state.career.as_deref().and_then(crate::career::Pilot::load)
            && let Some(t) = &p.tour
        {
            panel = panel.after(
                egui::RichText::new(format!(
                    "  {}  //  TOUR SORTIE {} OF {}",
                    p.title().to_ascii_uppercase(),
                    t.sortie + 1,
                    t.length
                ))
                .color(AMBER),
            );
        }
        panel.window(menu_window(ctx, "Single mission", width), ctx, |ui| {
            ui.horizontal_top(|ui| {
                if !career_mode {
                    // Options: three narrow columns.
                    ui.vertical(|ui| {
                        ui.set_width(210.0);
                        group(ui, "Mission", |ui| {
                            theater_options(ui, &mut state.theater);
                            situation_options(ui, &mut state.roe);
                            opponent_options(ui, &mut state.opponents);
                            mission_options(ui, &mut state);
                        });
                    });
                    ui.vertical(|ui| {
                        ui.set_width(220.0);
                        group(ui, "Conditions", |ui| {
                            let state = &mut *state;
                            weather_options(ui, &mut state.weather, &mut state.cloud_cover);
                            start_time_options(ui, state);
                            moon_options(ui, &mut state.moon);
                        });
                        simulation_options(ui, &mut settings, None);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(VEHICLE_WIDTH);
                        group(ui, "Vehicle", |ui| {
                            crate::aircraft_preview::show(ui, &mut preview);
                            let was = settings.aircraft;
                            vehicle_options(ui, &mut settings, Some(&mut state.lockheed));
                            // A troop helicopter flies the same seed's targets as insertions;
                            // helicopters fly short missions.
                            let (now, was) = (settings.aircraft.spec(), was.spec());
                            if now.transport() != was.transport() || now.helicopter != was.helicopter
                            {
                                state.regenerate();
                            }
                        });
                    });
                }
                // Mission summary in big type, the full briefing below in small type.
                // What is left beside the option columns, less the frame margins and border.
                let summary_width = (ui.available_width() - 22.0).max(300.0);
                egui::Frame::new().stroke(egui::Stroke::new(1.0, DIM)).inner_margin(10.0).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(summary_width);
                        match &state.mission {
                            Some(Ok(m)) => {
                                mission_summary(ui, &state, m);
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    let plan = ui.button(icon_text(ui, icon::PLAN_ROUTE, egui::RichText::new("PLAN ROUTE").size(16.0)));
                                    if facts(plan, &["The theater map with the enemy radar and SAM coverage", "Add, move and remove navigation points", "Targets and the landing base stay fixed"]).clicked() {
                                        planner.open = true;
                                    }
                                    let arm = ui.button(icon_text(ui, icon::LOADOUT, egui::RichText::new("LOADOUT").size(16.0)));
                                    if facts(arm, &["Choose the stores on each bay or pylon", "Starts with the briefed weapons"]).clicked() {
                                        loadout.open = true;
                                    }
                                });
                            }
                            Some(Err(e)) => {
                                ui.colored_label(AMBER, e);
                            }
                            None => {
                                ui.label("GENERATING...");
                            }
                        }
                    });
                });
            });
            ui.add_space(6.0);
            footer_rule(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let to = if career_mode { "Back to the pilot records" } else { "Back to the main menu" };
                if facts(ui.add(big_button("BACK").icon(icon::BACK)), &[to]).clicked() {
                    back = true;
                }
                let new = if career_mode { "NEW SORTIE" } else { "NEW MISSION" };
                if facts(ui.add(big_button(new).icon(icon::NEW_MISSION)), &["Rolls different objectives", "Keeps all the options and the start time"]).clicked() {
                    state.seed = rand::random();
                    state.dirty = true;
                }
                if !career_mode && facts(ui.add(big_button("LOAD").icon(icon::LOAD)), &["Missions saved in flight (menu: SAVE MISSION)", "Fly on from where they were saved"]).clicked() {
                    saves.show();
                }
                if facts(ui.add(big_button("OPTIONS").icon(icon::OPTIONS)), &["Game, graphics, sound and controls"]).clicked() {
                    options.open = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_fly = matches!(state.mission, Some(Ok(_)));
                    let fly = ui.add_enabled(can_fly, big_button("FLY").icon(icon::FLY).primary());
                    if facts(fly, &["Take off on the briefed mission"]).clicked()
                        && let Some(Ok(m)) = &state.mission
                    {
                        *setup = crate::mission::setup_from_mission(m, &tables.start_dir);
                        commands.insert_resource(ActiveMission(m.clone()));
                        commands.insert_resource(state.route.clone().unwrap_or_else(|| crate::mission::PlannedRoute::of(m)));
                        commands.insert_resource(state.loadout.clone().unwrap_or_else(|| crate::loadout::PlannedLoadout::of(m, settings.aircraft)));
                        match &state.career {
                            Some(key) => commands.insert_resource(crate::career::CareerFlight(key.clone())),
                            None => commands.remove_resource::<crate::career::CareerFlight>(),
                        }
                        next.set(AppState::Flight);
                    }
                    // Already in VR: "Fly" flies in the headset. No VR in the browser.
                    if !settings.vr && !cfg!(target_arch = "wasm32") {
                        let vr_button = ui
                            .add_enabled(can_fly && vr.0, big_button("FLY IN VR").icon(icon::VR))
                            .on_disabled_hover_text(if vr.0 { "Generate a mission first" } else { "No OpenXR runtime (Monado, WiVRn, SteamVR) is active" });
                        if facts(vr_button, &["Restarts the game in the VR headset", "Flies this exact mission"]).clicked() {
                            match relaunch_in_vr(&state, &settings) {
                                Ok(()) => {
                                    exit.write(AppExit::Success);
                                }
                                Err(e) => state.mission = Some(Err(format!("could not start in VR: {e}"))),
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
    crate::career::career_ui(
        ctx,
        &mut career,
        &mut state,
        &mut settings,
        &mut options,
        &mut preview,
    );
    crate::options::options_ui(ctx, &mut options, &mut settings, head.source, &mut dialog);
    crate::prefs::Prefs::save_if_changed(&state, &settings, &mut saved);
    Ok(())
}

fn mission_summary(ui: &mut egui::Ui, state: &MenuState, m: &Mission) {
    let b = &m.briefing;
    section(ui, "Mission summary");
    ui.label(egui::RichText::new(clean_text(&b.title)).size(20.0).color(BRIGHT));
    ui.add_space(4.0);
    let w = &m.weather;
    let rows = [
        ("THEATER", THEATERS[state.theater].1.to_string(), GREEN),
        ("SITUATION", ROE[state.roe as usize].0.to_string(), GREEN),
        (
            "MISSION",
            KINDS.iter().find(|k| k.0 == state.kind).map_or("", |k| k.1).to_string(),
            GREEN,
        ),
        ("OPPONENTS", OPPONENTS[state.opponents as usize].0.to_string(), GREEN),
        (
            "TIME",
            format!("{}  dep {}", if m.night { "Night" } else { "Day" }, m.time.depart),
            GREEN,
        ),
        (
            "WEATHER",
            format!("{}  vis {} km", w.sky.name(), w.vis_km),
            if w.sky == Sky::Storm { AMBER } else { GREEN },
        ),
        ("FUEL", format!("{} lbs", m.fuel_lbs), GREEN),
    ];
    egui::Grid::new("summary").num_columns(2).spacing([18.0, 4.0]).show(ui, |ui| {
        for (key, value, color) in rows {
            ui.label(egui::RichText::new(key).size(14.0).color(DIM));
            ui.label(egui::RichText::new(value.to_ascii_uppercase()).size(20.0).color(color));
            ui.end_row();
        }
    });
    section(ui, "Briefing");
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
            Some(lo) => small(ui, &format!("Armament: {}", lo.summary())),
            None => small(
                ui,
                &format!(
                    "Armament: {}",
                    b.armament.iter().map(|a| clean_text(a)).collect::<Vec<_>>().join(", ")
                ),
            ),
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new("INTELLIGENCE").small().color(AMBER));
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
