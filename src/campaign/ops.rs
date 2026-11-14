//! The campaign's Ops screen over the flight: mission board, war map, debrief and log, and the
//! prompt after landing at a friendly base. On the desktop an egui overlay over the 3D view,
//! in the headset the menu panel in front of the head (`vr_menu.rs`).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, PrimaryEguiContext, egui};
use stealth_campaign::weather::Now;
use stealth_campaign::{
    Aircraft as Airframe, Due, MissionKind, MissionOffer, Objective, PlayerId, Side, SortieResult,
    tuning,
};
use stealth_host::glue::sortie_mission;
use stealth_mission::Sky;
use stealth_net::protocol::PlayerStatus;
use stealth_sim::crew::{CrewCmd, Seat};
use stealth_sim::station::StationCmd;

mod briefing;
mod flights;

use briefing::{Briefing, Step};

use super::shell;
use super::{Action, ActiveCampaign, Phase, Role, describe, kind_of, preview};
use crate::map_draw::{
    self, FRIENDLY_BLUE, Layer, MISSILE_RED, MapLayers, MapTips, MapView, MapWeather,
};
use crate::menu::{
    AMBER, BG, BRIGHT, DIM, GREEN, Panel, apply_terminal_style, big_button, icon, section,
};
use crate::settings::AppState;
use stealth_i18n::{Text, t};

/// A list that slides in over the map from the rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Missions,
    Flights,
    AirDefence,
    Logistics,
    Players,
    Debrief,
    Log,
    Chat,
}

impl Pane {
    /// The rail's buttons in order (CHAT only in a coop game).
    pub const ALL: [Pane; 8] = [
        Pane::Missions,
        Pane::Flights,
        Pane::AirDefence,
        Pane::Logistics,
        Pane::Players,
        Pane::Debrief,
        Pane::Log,
        Pane::Chat,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Pane::Missions => t!("ops-tab-missions"),
            Pane::Flights => t!("ops-tab-flights"),
            Pane::AirDefence => t!("ops-tab-air-defence"),
            Pane::Logistics => t!("ops-tab-logistics"),
            Pane::Players => t!("ops-tab-players"),
            Pane::Debrief => t!("ops-tab-debrief"),
            Pane::Log => t!("ops-tab-log"),
            Pane::Chat => t!("ops-tab-chat"),
        }
    }

    pub fn icon(self) -> char {
        match self {
            Pane::Missions => icon::MISSIONS,
            Pane::Flights => icon::FLIGHTS,
            Pane::AirDefence => icon::AIR_DEFENCE,
            Pane::Logistics => icon::SUPPLY,
            Pane::Players => icon::PLAYERS,
            Pane::Debrief => icon::DEBRIEF,
            Pane::Log => icon::LOG,
            Pane::Chat => icon::CHAT,
        }
    }

    /// A pane of tables and long text, wider than the lists.
    fn wide(self) -> bool {
        matches!(self, Pane::Logistics | Pane::Players | Pane::Debrief)
    }
}

/// The MISSIONS pane's filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Filter {
    #[default]
    All,
    Attack,
    Recon,
    Support,
}

impl Filter {
    fn keeps(self, k: MissionKind) -> bool {
        match self {
            Filter::All => true,
            Filter::Recon => k == MissionKind::Recon,
            Filter::Attack => k != MissionKind::Recon && kind_colour(k) == AMBER,
            Filter::Support => k != MissionKind::Recon && kind_colour(k) != AMBER,
        }
    }
}

/// The Ops screen's own state.
#[derive(Default)]
pub struct OpsUi {
    /// The pane open over the map, and when it opened (egui time; None: from the next frame).
    pub pane: Option<Pane>,
    pane_since: Option<f64>,
    /// The map's view, and the one it had before a pane switched it (AIR DEFENSE).
    pub(super) preset: super::war_map::MapPreset,
    pub(super) preset_before: Option<super::war_map::MapPreset>,
    /// A place to bring into view (picked in a list), a zoom asked by the map's buttons.
    pub(super) focus: Option<egui::Vec2>,
    pub(super) zoom: Option<f32>,
    filter: Filter,
    /// Chat messages read, the debrief seen (its sortie's id).
    chat_seen: usize,
    debrief_seen: Option<u32>,
    pub(super) selected: Option<u32>,
    /// Base chosen for the selected mission (None: the offer's).
    pub(super) base: Option<u16>,
    pub error: Option<Text>,
    pub(super) centre: egui::Vec2,
    pub(super) scale: f32,
    pub(super) fitted: bool,
    /// The air defence site selected on the AIR DEFENSE pane (AAA unit), and the list scrolls
    /// to it (picked on the map).
    pub(super) defence: Option<usize>,
    pub(super) defence_scroll: bool,
    /// A new site is being designated on the map (a Patriot battery, a Gepard, a drone command
    /// center), and `--campaign-designate` was done.
    pub(super) designate: Option<stealth_campaign::DefenceKind>,
    pub(super) test_designated: bool,
    /// `--campaign-designate <kind>+built` fast-tracked it.
    pub(super) test_fast_tracked: bool,
    /// The airbase (or the forces) picked on the LOGISTICS pane.
    pub(super) logistics: Option<super::logistics::Pick>,
    /// The new site whose FAST-TRACK was clicked, awaiting confirmation.
    pub(super) fast_track: Option<u32>,
    /// `--campaign-take` was done, `--campaign-man`, `--campaign-geran`.
    pub test_taken: bool,
    pub test_manned: bool,
    pub test_strike: bool,
    pub test_geran: bool,
    /// `--campaign-take-control` was done, `--campaign-leave`.
    pub test_controlled: bool,
    pub test_left: bool,
    /// The flight selected on the FLIGHTS pane.
    flight: Option<flights::FlightKey>,
    /// `--campaign-crew` was done.
    pub test_crewed: bool,
    /// The accepted mission's briefing (map and loadout) before take-off.
    briefing: Option<Briefing>,
    /// SAVE & EXIT asks first.
    pub(super) confirm_exit: bool,
    /// The chat message being typed on the CHAT pane.
    chat: String,
}

impl OpsUi {
    /// Opens a pane (None: the map alone), or closes it if it is the one open. Opening AIR
    /// DEFENSE turns the map to its view; closing it turns the map back.
    pub fn open(&mut self, pane: Option<Pane>) {
        use super::war_map::MapPreset;
        if pane == Some(Pane::AirDefence) && self.preset != MapPreset::AirDefence {
            self.preset_before = Some(self.preset);
            self.preset = MapPreset::AirDefence;
        } else if self.pane == Some(Pane::AirDefence)
            && pane != Some(Pane::AirDefence)
            && let Some(p) = self.preset_before.take()
        {
            self.preset = p;
        }
        if self.pane != pane {
            self.pane_since = None;
        }
        self.pane = pane;
    }

    /// The rail's button: its pane, or closed again.
    fn toggle(&mut self, pane: Pane) {
        self.open(if self.pane == Some(pane) { None } else { Some(pane) });
    }
}

/// The camera that carries the Ops screen's egui context.
#[derive(Component, Clone)]
pub struct OpsCamera;

/// Shows and hides the Ops screen's egui context with the screen.
#[allow(clippy::too_many_arguments)]
pub fn overlay(
    mut commands: Commands,
    c: Option<Res<ActiveCampaign>>,
    tagged: Query<Entity, With<OpsCamera>>,
    mut cams: Query<&mut Msaa, With<OpsCamera>>,
    eye: Query<&Msaa, (With<crate::camera::EyeCamera>, Without<OpsCamera>)>,
    settings: Res<crate::settings::Settings>,
    root: Option<Res<crate::camera::XrRoot>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let want = c.is_some_and(|c| c.screen_open() && !c.exit);
    // The overlay draws into the 3D view's picture only with the same anti-aliasing (else it
    // gets a picture of its own that nothing clears: the world gone behind the screen and
    // moved windows leaving trails). The 3D view's MSAA follows the MSAA option.
    let msaa = eye.iter().next().copied().unwrap_or(Msaa::Off);
    for mut m in &mut cams {
        if *m != msaa {
            *m = msaa;
        }
    }
    // Everything spawned with the screen carries the tag (in VR also the panel, the laser and
    // the desktop mirror), not only the cameras.
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
            OpsCamera,
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
        OpsCamera,
        PrimaryEguiContext,
    ));
}

/// The flight's controls are off while the screen is up.
pub fn block_input(
    c: Res<ActiveCampaign>,
    menu: Res<crate::flight_menu::FlightMenu>,
    mut block: ResMut<crate::input::InputBlock>,
    settings: Res<crate::settings::Settings>,
    mut hide: ResMut<crate::camera::HideOwnAircraft>,
    mut console: ResMut<crate::input::ConsoleStick>,
) {
    // At an air defence console the flight's controls are off too (its buttons are clicked),
    // but for the stick and the triggers (its gun's).
    let station = matches!(c.phase, super::Phase::Station { .. });
    let want = c.screen_open() || menu.open || station;
    if block.0 != want {
        block.0 = want;
    }
    let stick = station && !c.screen_open() && !menu.open;
    if console.0 != stick {
        console.0 = stick;
    }
    // VR: the parked aircraft and its cockpit are not drawn around the Ops screen (the
    // in-flight menu's screen is in the cockpit: then it is drawn).
    let hidden = settings.vr && c.screen_open() && !menu.open;
    if hide.0 != hidden {
        hide.0 = hidden;
    }
}

/// Test aids: `--campaign-tab board|map|flights|airdefense|logistics|players|debrief|log|chat`
/// opens a pane (`map`: none), `--campaign-take <n>` takes the n-th mission of the board (0: the
/// first) once the campaign is up.
pub fn test_pane() -> Option<Option<Pane>> {
    let args: Vec<String> = std::env::args().collect();
    let t = args.iter().position(|a| a == "--campaign-tab").and_then(|i| args.get(i + 1))?;
    Some(match t.as_str() {
        "map" => None,
        "flights" => Some(Pane::Flights),
        "airdefense" => Some(Pane::AirDefence),
        "logistics" | "forces" => Some(Pane::Logistics),
        "debrief" => Some(Pane::Debrief),
        "log" => Some(Pane::Log),
        "players" => Some(Pane::Players),
        "chat" => Some(Pane::Chat),
        _ => Some(Pane::Missions),
    })
}

/// Test aid: `--campaign-man <n>` mans the n-th friendly air defence console once the
/// campaign is up (Patriot batteries first, then NASAMS, then the Gepards); `drone` is
/// [`TEST_MAN_DRONES`], `nasams` [`TEST_MAN_NASAMS`].
pub fn test_man() -> Option<usize> {
    let args: Vec<String> = std::env::args().collect();
    let a = args.iter().position(|a| a == "--campaign-man").and_then(|i| args.get(i + 1))?;
    match a.as_str() {
        "drone" => Some(TEST_MAN_DRONES),
        "nasams" => Some(TEST_MAN_NASAMS),
        _ => a.parse().ok(),
    }
}

pub const TEST_MAN_DRONES: usize = usize::MAX;
pub const TEST_MAN_NASAMS: usize = usize::MAX - 1;

/// `--campaign-take <n>`: the n-th mission of the board, or the first of a kind (`logistics`).
pub enum TestTake {
    Index(usize),
    Kind(String),
}

pub fn test_take() -> Option<TestTake> {
    let args: Vec<String> = std::env::args().collect();
    let a = args.iter().position(|a| a == "--campaign-take").and_then(|i| args.get(i + 1))?;
    Some(a.parse().map_or_else(|_| TestTake::Kind(a.clone()), TestTake::Index))
}

/// Background of an unselected mission row.
const ROW: egui::Color32 = egui::Color32::from_rgb(9, 29, 21);

/// Map units to km.
fn km(d: i32) -> f32 {
    d as f32 * 15.625 / 1000.0
}

fn dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    stealth_campaign::dist(ax - bx, ay - by)
}

pub(super) fn kind_colour(k: MissionKind) -> egui::Color32 {
    match k {
        MissionKind::Recon => DIM,
        MissionKind::Insertion
        | MissionKind::Escort
        | MissionKind::Resupply
        | MissionKind::Logistics
        | MissionKind::Airlift => FRIENDLY_BLUE,
        MissionKind::Cap => GREEN,
        MissionKind::Intercept | MissionKind::Defend | MissionKind::Csar | MissionKind::Awacs => {
            MISSILE_RED
        }
        _ => AMBER,
    }
}

/// The weather and the next change: a short form for the header and the briefing's line.
fn weather_text(c: &ActiveCampaign, now: &Now) -> (String, String) {
    let name = |s: Sky| match s {
        Sky::Storm => t!("ops-thunderstorms").to_string(),
        s => crate::i18n::sky_name(s).to_uppercase(),
    };
    let setup = &c.state().setup;
    let h = setup.start_hour as f64 + now.next.1 * setup.sky_ratio as f64 / 3600.0;
    let mins = (h.rem_euclid(24.0) * 60.0).floor() as i64;
    let at = format!("{:02}:{:02}", mins / 60, mins % 60);
    let short =
        t!("ops-wx-short", now = name(now.weather.sky), next = name(now.next.0), at = at.as_str());
    let next = match now.next.0 {
        Sky::Storm => t!("ops-thunderstorms").to_lowercase(),
        s => crate::i18n::sky_name(s).to_lowercase(),
    };
    let line = t!("ops-wx-line", weather = now.weather.describe(), next = next, at = at);
    (short, line)
}

/// The campaign clock as the sky shows it: day and time.
fn clock_text(c: &ActiveCampaign) -> String {
    let h = c.state().sky_hours();
    let day = (h / 24.0).floor() as i64 + 1;
    let hh = h.rem_euclid(24.0);
    let secs = (hh * 3600.0).floor() as i64;
    let time = format!("{:02}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60);
    t!("ops-clock", day = day, time = time)
}

#[allow(clippy::too_many_arguments)]
pub fn ops_ui(
    mut contexts: EguiContexts,
    mut c: ResMut<ActiveCampaign>,
    combat: Res<crate::combat::Combat>,
    mut planner: ResMut<crate::planner::Planner>,
    loadout: Option<Res<crate::loadout::PlannedLoadout>>,
    mut next: ResMut<NextState<AppState>>,
    mut unit_view: ResMut<super::UnitView>,
    weather: Res<crate::weather::WeatherState>,
    sky: Res<crate::sky::Sky>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
    settings: Res<crate::settings::Settings>,
    mut menu: ResMut<crate::flight_menu::FlightMenu>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    mut voice: ResMut<crate::voice::Voice>,
) -> Result {
    // On the desktop the in-flight menu (OPTIONS) is drawn in the view the screen covers.
    if !c.screen_open() || c.exit || (menu.open && !settings.vr) {
        return Ok(());
    }
    let wx = weather.now.map(|n| weather_text(&c, &n));
    // The maps' weather: the forecast's now (a clear sky without changing weather).
    let map_wx =
        MapWeather::new(weather.now.map(|n| n.weather).unwrap_or_default(), sky.freezing_level());
    let Ok(ctx) = contexts.ctx_mut() else { return Ok(()) };
    crate::menu::apply_reflow_zoom(ctx);
    crate::menu::add_icon_font(ctx);
    if ctx.global_style().visuals.window_fill != BG {
        ctx.all_styles_mut(apply_terminal_style);
    }
    let c = &mut *c;
    // Landed at a friendly base during a sortie: rearm, or end it.
    if c.phase == Phase::Sortie
        && let Some(base) = c.landed
    {
        let name = combat.sim.place_name(base);
        let window = egui::Window::new("Landed")
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -40.0))
            .collapsible(false)
            .resizable(false)
            .title_bar(false);
        let title = t!("ops-landed-at", base = name.to_uppercase());
        Panel::new(&title).window(window, ctx, |ui| {
            ui.label(egui::RichText::new(t!("ops-ground-crew")).color(DIM));
            ui.horizontal(|ui| {
                if ui.add(big_button(t!("ops-rearm")).icon(icon::REARM)).clicked() {
                    if let Some(loadout) = loadout.as_deref().cloned() {
                        c.actions.push(Action::Rearm { loadout });
                    }
                }
                if ui
                    .add(big_button(t!("ops-end-sortie")).icon(icon::END_SORTIE).primary())
                    .clicked()
                {
                    c.actions.push(Action::EndSortie { outcome: 0 });
                    c.landed = None;
                }
            });
        });
        return Ok(());
    }
    // Test aid: `--campaign-accept map|loadout` opens the first mission's briefing.
    if let Some(page) = briefing::test_accept()
        && !std::mem::replace(&mut c.ui.test_taken, true)
        && let Some(o) = c.state().board.iter().find(|o| o.kind.flyable_by(c.aircraft)).cloned()
    {
        let sortie = preview(&o, o.base, c.aircraft);
        let m = sortie_mission(&c.mission, c.state(), &sortie);
        let loadout = crate::loadout::PlannedLoadout::of(&m, kind_of(c.aircraft));
        c.ui.briefing = Some(Briefing::new(o.id, o.base, c.aircraft, loadout, page));
        if c.role != Role::Local {
            c.actions.push(Action::SelectMission { offer: Some(o.id), base: o.base });
        }
    }
    if c.ui.briefing.is_some() {
        let line = wx.as_ref().map(|w| w.1.as_str());
        briefing::briefing(ctx, c, &combat.sim, &mut planner, line, &map_wx);
        return Ok(());
    }
    super::designate::test_aid(c, &combat.sim, scenery.as_deref());
    let screen = ctx.content_rect();
    let room = shell::Room::of(screen.width());
    let now = ctx.input(|i| i.time);
    // The deeds the open pane (or the map's selection) offers, the one pressed, and what the
    // action bar says of it.
    let mut deck: Vec<Deed> = Vec::new();
    let mut bar: Option<(String, String)> = None;
    let sites = super::air_defence::sites(c, &combat.sim);
    let pane_w = c.ui.pane.map_or(0.0, |p| shell::pane_width(screen.width(), p.wide()));
    // The screen: the top bar, the rail down the left, the map in the rest; one area under all
    // that floats over the map.
    let top = egui::Rect::from_min_size(screen.min, egui::vec2(screen.width(), shell::TOP_H));
    let rail_rect = egui::Rect::from_min_max(
        egui::pos2(screen.left(), top.bottom()),
        egui::pos2(screen.left() + shell::RAIL_W, screen.bottom()),
    );
    let map_rect =
        egui::Rect::from_min_max(egui::pos2(rail_rect.right(), top.bottom()), screen.max);
    egui::Area::new(egui::Id::new("ops_screen"))
        .fixed_pos(screen.min)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            let edge = egui::Stroke::new(2.0, crate::menu::FRAME);
            ui.scope_builder(egui::UiBuilder::new().max_rect(map_rect), |ui| {
                ui.set_clip_rect(map_rect);
                super::war_map::map(
                    ui,
                    c,
                    &combat.sim,
                    &mut unit_view,
                    &mut planner,
                    &map_wx,
                    scenery.as_deref(),
                    &sites,
                    pane_w,
                );
            });
            ui.painter().rect_filled(top, 0.0, BG);
            ui.painter().hline(top.x_range(), top.bottom() - 1.0, edge);
            ui.scope_builder(egui::UiBuilder::new().max_rect(top), |ui| {
                top_bar(ui, c, wx.as_ref().map(|w| w.0.as_str()), room, &mut menu);
            });
            ui.painter().rect_filled(rail_rect, 0.0, BG);
            ui.painter().vline(rail_rect.right() - 1.0, rail_rect.y_range(), edge);
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(rail_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| rail(ui, c, &notices),
            );
        });
    // The pane over the map's left, sliding in.
    if let Some(pane) = c.ui.pane {
        let since = *c.ui.pane_since.get_or_insert(now);
        let shown = shell::slide(ctx, since);
        let at = map_rect.left_top() - egui::vec2((1.0 - shown) * pane_w, 0.0);
        let size = egui::vec2(pane_w, map_rect.height());
        egui::Area::new(egui::Id::new("ops_pane")).fixed_pos(at).order(egui::Order::Middle).show(
            ctx,
            |ui| {
                let frame = egui::Frame::new()
                    .fill(BG.gamma_multiply(0.97))
                    .stroke(egui::Stroke::new(2.0, crate::menu::EDGE))
                    .inner_margin(egui::Margin::symmetric(14, 8));
                frame.show(ui, |ui| {
                    let inner = size - egui::vec2(28.0, 16.0);
                    ui.set_min_size(inner);
                    ui.set_max_size(inner);
                    if shell::heading(ui, Some(pane.icon()), pane.label(), true) {
                        c.ui.open(None);
                    }
                    let room = ui.available_size();
                    ui.allocate_ui(room, |ui| {
                        ui.set_min_size(room);
                        pane_body(
                            ui,
                            c,
                            pane,
                            &combat.sim,
                            &sites,
                            &mut notices,
                            &mut voice,
                            &mut deck,
                            &mut bar,
                            settings.vr,
                        );
                    });
                });
            },
        );
    }
    if c.ui.pane == Some(Pane::Chat) {
        c.ui.chat_seen = notices.log().len();
    }
    if c.ui.pane == Some(Pane::Debrief) {
        c.ui.debrief_seen = c.debrief.as_ref().map(|d| d.record.id);
    }
    // Nothing offered by a pane: the map's watched unit, else the way to the missions.
    if deck.is_empty() && bar.is_none() && c.state().outcome.is_none() {
        deck = map_deeds(c, &combat.sim, &unit_view);
        if unit_view.target.is_some() {
            bar = Some((unit_view.title.clone(), String::new()));
        } else if c.ui.pane.is_none() {
            let n = c.state().board.len();
            bar = Some((
                t!("ops-bar-missions", count = n).to_string(),
                t!("ops-bar-hint").to_string(),
            ));
            deck.push(
                Deed::new(Todo::Open(Pane::Missions), t!("ops-tab-missions"), icon::MISSIONS, "")
                    .primary(),
            );
        }
    }
    let pressed = action_bar(ctx, c, map_rect, pane_w, &deck, bar.as_ref());
    if c.ui.pane.is_none() && screen.width() >= 1400.0 {
        ticker(ctx, c, map_rect);
    }
    if let Some(i) = pressed {
        do_deed(c, &mut unit_view, deck.swap_remove(i).todo);
    }
    if c.state().outcome.is_some() {
        let window = egui::Window::new("War over")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(pane_w / 2.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .fixed_size(egui::vec2(820.0_f32.min(screen.width() - pane_w - 140.0), 0.0));
        Panel::new(t!("menu-campaign")).window(window, ctx, |ui| end_screen(ui, c));
    }
    let mut leave = false;
    super::designate::confirm(ctx, c);
    if c.ui.confirm_exit {
        let window = egui::Window::new("Leave the campaign")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .order(egui::Order::Foreground);
        let id = match c.role {
            Role::Local => "ops-leave-local",
            Role::Host => "ops-leave-host",
            Role::Client => "ops-leave-client",
        };
        let (title, text) = (t!(id), t!(&format!("{id}.text")));
        let button = exit_label(c.role);
        Panel::new(title).window(window, ctx, |ui| {
            ui.label(egui::RichText::new(text).color(DIM));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(big_button(button).icon(exit_icon(c.role)).fill(EXIT_RED)).clicked() {
                    leave = true;
                }
                if ui.add(big_button(t!("cancel")).icon(icon::CANCEL)).clicked() {
                    c.ui.confirm_exit = false;
                }
            });
        });
    }
    if leave {
        c.ui.confirm_exit = false;
        c.save();
        c.exit = true;
        next.set(AppState::Menu);
    }
    Ok(())
}

/// The top bar: the campaign and its theater, the clock, the places held, the weather, the
/// materiel and the front, the war simulation's state; OPTIONS and the way out at the right.
fn top_bar(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    wx: Option<&str>,
    room: shell::Room,
    menu: &mut crate::flight_menu::FlightMenu,
) {
    use shell::Room;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        // The brand over the rail.
        let (brand, _) =
            ui.allocate_exact_size(egui::vec2(shell::RAIL_W, shell::TOP_H), egui::Sense::hover());
        let p = ui.painter();
        p.text(
            brand.left_center() + egui::vec2(16.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "//",
            egui::FontId::monospace(22.0),
            crate::menu::EDGE,
        );
        if room > Room::Small {
            let name = c.mission.theater_name.to_uppercase();
            let title = t!("ops-top-campaign");
            let r = ui.label(egui::RichText::new(title).size(20.0).color(BRIGHT));
            if room == Room::Full {
                ui.label(egui::RichText::new(name).small().color(DIM));
            } else {
                r.on_hover_text(name);
            }
        }
        let chip = |ui: &mut egui::Ui, glyph: Option<char>, add: &mut dyn FnMut(&mut egui::Ui)| {
            egui::Frame::new()
                .fill(crate::menu::FIELD)
                .stroke(egui::Stroke::new(1.0, crate::menu::FRAME))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(g) = glyph {
                            let (r, _) = ui
                                .allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                            shell::paint_icon(ui.painter(), r.center(), g, 17.0, DIM);
                        }
                        add(ui);
                    });
                })
                .response
        };
        chip(ui, Some(icon::CLOCK), &mut |ui| {
            ui.label(egui::RichText::new(clock_text(c)).color(BRIGHT));
        });
        chip(ui, None, &mut |ui| {
            ui.label(
                egui::RichText::new(t!("ops-blue", count = c.state().held(Side::Blue)))
                    .color(FRIENDLY_BLUE),
            );
            ui.label(
                egui::RichText::new(t!("ops-red", count = c.state().held(Side::Red)))
                    .color(MISSILE_RED),
            );
        });
        if let Some(wx) = wx {
            let r = chip(ui, Some(icon::WEATHER), &mut |ui| {
                if room > Room::Small {
                    ui.label(egui::RichText::new(wx).color(BRIGHT));
                }
            });
            if room == Room::Small {
                r.on_hover_text(wx);
            }
        }
        chip(ui, Some(icon::MATERIEL), &mut |ui| supply_line(ui, c, room == Room::Full));
        if let Some(server) = &c.server {
            ui.label(
                egui::RichText::new(t!("ops-on-server", server = server.to_uppercase()))
                    .small()
                    .color(FRIENDLY_BLUE),
            );
        } else if c.role == Role::Host {
            ui.label(egui::RichText::new(t!("ops-serving")).small().color(FRIENDLY_BLUE));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            let exit = if room == Room::Small {
                shell::square_button(ui, exit_icon(c.role), exit_label(c.role))
            } else {
                ui.add(
                    big_button(exit_label(c.role))
                        .icon(exit_icon(c.role))
                        .fill(EXIT_RED)
                        .min_size(egui::vec2(120.0, 44.0)),
                )
            };
            if exit.clicked() {
                c.ui.confirm_exit = true;
            }
            if shell::square_button(ui, icon::OPTIONS, t!("menu-options.short")).clicked() {
                menu.open_options = true;
            }
            if c.state().outcome.is_some() {
                ui.label(egui::RichText::new(t!("ops-war-over")).color(DIM));
            } else {
                ui.add(egui::Spinner::new().color(GREEN)).on_hover_text(t!("ops-running"));
            }
        });
    });
}

/// The rail: a button per pane, with what is new on it.
fn rail(ui: &mut egui::Ui, c: &mut ActiveCampaign, notices: &crate::player_notices::PlayerNotices) {
    ui.add_space(6.0);
    ui.spacing_mut().item_spacing.y = 0.0;
    for pane in Pane::ALL {
        if pane == Pane::Chat && c.role == Role::Local {
            continue;
        }
        if pane == Pane::Players {
            ui.add_space(6.0);
            let r = ui.available_rect_before_wrap();
            ui.painter().hline(
                r.left() + 12.0..=r.right() - 12.0,
                r.top(),
                egui::Stroke::new(1.0, crate::menu::FRAME),
            );
            ui.add_space(6.0);
        }
        let badge = match pane {
            Pane::Missions if c.ui.pane != Some(Pane::Missions) => {
                shell::Badge::Count(c.state().board.len()).filter_zero()
            }
            Pane::Chat if c.ui.pane != Some(Pane::Chat) => {
                shell::Badge::Count(notices.log().len().saturating_sub(c.ui.chat_seen))
                    .filter_zero()
            }
            Pane::Debrief
                if c.ui.pane != Some(Pane::Debrief)
                    && c.debrief
                        .as_ref()
                        .is_some_and(|d| Some(d.record.id) != c.ui.debrief_seen) =>
            {
                shell::Badge::Dot
            }
            _ => shell::Badge::None,
        };
        if shell::rail_button(ui, pane.icon(), pane.label(), c.ui.pane == Some(pane), badge)
            .clicked()
        {
            c.ui.toggle(pane);
        }
    }
}

/// What a pane holds.
#[allow(clippy::too_many_arguments)]
fn pane_body(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    pane: Pane,
    sim: &stealth_sim::Sim,
    sites: &[super::air_defence::Site],
    notices: &mut crate::player_notices::PlayerNotices,
    voice: &mut crate::voice::Voice,
    deck: &mut Vec<Deed>,
    bar: &mut Option<(String, String)>,
    vr: bool,
) {
    let size = ui.available_size();
    match pane {
        Pane::Missions => missions(ui, c, deck, bar),
        Pane::Flights => flights::tab(ui, c, sim, size.x, deck, bar),
        Pane::AirDefence => {
            super::air_defence::pane(ui, c, sim, sites, size.y);
            if let Some(unit) = c.ui.defence
                && let Some((title, sub)) = super::air_defence::describe(c, sites, unit)
            {
                *bar = Some((title.clone(), sub));
                let mut watch =
                    Deed::new(Todo::Watch { unit, title }, t!("ops-watch"), icon::WATCH, "");
                watch.why = None;
                deck.push(watch);
                if sim.units().get(unit).is_some_and(|u| stealth_sim::station::mannable(u.kind)) {
                    let mut man = Deed::new(
                        Todo::Man { unit: unit as u16 },
                        t!("ops-control"),
                        icon::CONTROL,
                        t!("ops-control.facts"),
                    )
                    .primary();
                    man.why = man_refusal(c, sim, unit);
                    deck.push(man);
                }
            }
        }
        Pane::Logistics => super::logistics::tab(ui, c, size),
        Pane::Players => {
            egui::ScrollArea::vertical()
                .id_salt("players_pane")
                .show(ui, |ui| players(ui, c, voice))
                .inner
        }
        Pane::Debrief => {
            egui::ScrollArea::vertical().id_salt("debrief_pane").show(ui, |ui| debrief(ui, c)).inner
        }
        Pane::Log => log(ui, c),
        Pane::Chat => chat(ui, c, notices, vr),
    }
}

/// The action bar at the map's bottom right: what is selected, and its buttons, the main one
/// last; the error of the last thing tried. The index of the deed pressed.
fn action_bar(
    ctx: &egui::Context,
    c: &mut ActiveCampaign,
    map: egui::Rect,
    pane: f32,
    deck: &[Deed],
    bar: Option<&(String, String)>,
) -> Option<usize> {
    if deck.is_empty() && bar.is_none() && c.ui.error.is_none() {
        return None;
    }
    let room = map.width() - pane - 2.0 * shell::GAP;
    let mut pressed = None;
    egui::Area::new(egui::Id::new("ops_action_bar"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-shell::GAP, -shell::GAP))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            shell::overlay_frame().inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                ui.set_max_width(room - 24.0);
                ui.horizontal(|ui| {
                    if bar.is_some() || c.ui.error.is_some() {
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            // As wide as the buttons leave (cut short, the rest on hover).
                            let text_w = (room - 24.0 - deck.len() as f32 * 240.0).max(160.0);
                            ui.set_max_width(text_w);
                            if let Some((title, sub)) = bar {
                                let t = egui::Label::new(
                                    egui::RichText::new(title).size(17.0).color(BRIGHT),
                                )
                                .truncate();
                                ui.add(t);
                                if !sub.is_empty() {
                                    let l = egui::Label::new(
                                        egui::RichText::new(sub).small().color(DIM),
                                    )
                                    .truncate();
                                    ui.add(l).on_hover_text(sub);
                                }
                            }
                            if let Some(e) = &c.ui.error {
                                ui.label(egui::RichText::new(e.tr()).color(AMBER));
                            }
                        });
                        ui.add_space(12.0);
                    }
                    for (i, d) in deck.iter().enumerate() {
                        let mut button =
                            big_button(&d.label).icon(d.icon).min_size(egui::vec2(150.0, 56.0));
                        if d.primary {
                            button = button.primary();
                        }
                        let mut resp = ui.add_enabled(d.why.is_none(), button);
                        if !d.facts.is_empty() {
                            resp = resp.on_hover_text(&d.facts);
                        }
                        if let Some(why) = &d.why {
                            resp = resp.on_disabled_hover_text(why);
                        }
                        if resp.clicked() {
                            pressed = Some(i);
                        }
                    }
                });
                // Why a button is off, written out (no hover text in the headset).
                if let Some(why) = deck.iter().find_map(|d| d.why.as_ref()) {
                    ui.label(egui::RichText::new(why).small().color(AMBER));
                }
            });
        });
    pressed
}

/// The war's last two events at the map's bottom left; a click opens the log.
fn ticker(ctx: &egui::Context, c: &mut ActiveCampaign, map: egui::Rect) {
    let lines: Vec<(f64, String)> =
        c.state().log.iter().rev().take(2).map(|e| (e.time, e.text.tr())).collect();
    if lines.is_empty() {
        return;
    }
    let mut open = false;
    egui::Area::new(egui::Id::new("ops_ticker"))
        .fixed_pos(map.left_bottom() + egui::vec2(shell::GAP, -shell::GAP - 2.0 * 40.0))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.set_max_width(470.0);
            ui.spacing_mut().item_spacing.y = 6.0;
            for (time, text) in lines.into_iter().rev() {
                let m = (time / 60.0).floor() as i64;
                let r = shell::chip_frame()
                    .show(ui, |ui| {
                        ui.set_width(450.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(format!("T+{:02}:{:02}", m / 60, m % 60))
                                    .small()
                                    .color(DIM),
                            );
                            ui.label(egui::RichText::new(text).small().color(BRIGHT));
                        });
                    })
                    .response;
                let r = ui.interact(r.rect, r.id.with("ticker"), egui::Sense::click());
                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    open = true;
                }
            }
        });
    if open {
        c.ui.open(Some(Pane::Log));
    }
}

/// The MISSIONS pane: the board's offers by objective, the chosen one opened under its row;
/// what it offers goes to the action bar.
fn missions(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    deck: &mut Vec<Deed>,
    bar: &mut Option<(String, String)>,
) {
    let from = |c: &ActiveCampaign, o: &MissionOffer| {
        c.state().site(o.base).map(|b| (b.x, b.y)).unwrap_or((o.x, o.y))
    };
    if c.ui.selected.is_none_or(|id| !c.state().board.iter().any(|o| o.id == id)) {
        c.ui.selected = c.state().board.first().map(|o| o.id);
        c.ui.base = None;
    }
    let count = |f: Filter| c.state().board.iter().filter(|o| f.keeps(o.kind)).count();
    let all = format!("{} {}", t!("ops-filter-all"), count(Filter::All));
    let attack = format!("{} {}", t!("ops-filter-attack"), count(Filter::Attack));
    let recon = format!("{} {}", kind_name(MissionKind::Recon), count(Filter::Recon));
    let support = format!("{} {}", t!("ops-filter-support"), count(Filter::Support));
    let options = [
        (Filter::All, None, all.as_str()),
        (Filter::Attack, None, attack.as_str()),
        (Filter::Recon, None, recon.as_str()),
        (Filter::Support, None, support.as_str()),
    ];
    if let Some(f) = shell::segmented(ui, c.ui.filter, &options, 44.0) {
        c.ui.filter = f;
    }
    ui.add_space(8.0);
    let front = c.state().front(Side::Blue);
    // The offers by objective, in the board's order.
    let mut groups: Vec<(Option<u16>, Vec<MissionOffer>)> = Vec::new();
    for o in c.state().board.iter().filter(|o| c.ui.filter.keeps(o.kind)) {
        let place = place_of(&o.objective);
        match groups.iter_mut().find(|g| place.is_some() && g.0 == place) {
            Some(g) => g.1.push(o.clone()),
            None => groups.push((place, vec![o.clone()])),
        }
    }
    egui::ScrollArea::vertical().id_salt("offers").auto_shrink([false, false]).show(ui, |ui| {
        if groups.is_empty() {
            ui.label(egui::RichText::new(t!("ops-no-missions")).color(DIM));
        }
        for (_, offers) in &groups {
            // The capture it prepares: its steps once over the group.
            if let Some((place, steps)) = operation_steps(c, &offers[0], &front) {
                ui.horizontal(|ui| {
                    let (r, _) =
                        ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                    shell::paint_icon(ui.painter(), r.center(), icon::OBJECTIVE, 15.0, BRIGHT);
                    let text = t!("ops-operation", place = place);
                    ui.label(
                        egui::RichText::new(text.trim_end_matches(':').to_uppercase())
                            .color(BRIGHT),
                    );
                });
                ui.horizontal(|ui| task_sequence(ui, &steps, None));
                ui.add_space(4.0);
            }
            for o in offers {
                let (bx, by) = from(c, o);
                let selected = c.ui.selected == Some(o.id);
                if offer_row(ui, c, o, km(dist(bx, by, o.x, o.y)), selected).clicked() && !selected
                {
                    c.ui.selected = Some(o.id);
                    c.ui.base = None;
                    c.ui.error = None;
                    c.ui.focus = Some(egui::vec2(o.x as f32, o.y as f32));
                }
                if selected {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(8, 26, 18))
                        .stroke(egui::Stroke::new(2.0, crate::menu::EDGE))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            offer_detail(ui, c, o, deck, bar);
                        });
                }
                ui.add_space(6.0);
            }
            ui.add_space(8.0);
        }
    });
}

/// An offer's row: its kind, title, who is on it and when it is due; its distance and threat.
fn offer_row(
    ui: &mut egui::Ui,
    c: &ActiveCampaign,
    o: &MissionOffer,
    km: f32,
    selected: bool,
) -> egui::Response {
    let due = match c.state().due(&o.objective) {
        Some(Due::At(t)) => {
            let mins = ((t - c.state().clock) / 60.0).max(0.0).ceil() as i64;
            t!("ops-due", mins = mins).to_string()
        }
        Some(Due::Now) => t!("ops-due-now").to_string(),
        None => String::new(),
    };
    let colour = kind_colour(o.kind);
    let frame = egui::Frame::new()
        .fill(if selected { egui::Color32::from_rgb(12, 42, 29) } else { ROW })
        .stroke(egui::Stroke::new(
            if selected { 2.0 } else { 1.0 },
            if selected { crate::menu::EDGE } else { crate::menu::FRAME },
        ))
        .inner_margin(egui::Margin::symmetric(10, 8));
    let inner = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(40.0);
        ui.horizontal(|ui| {
            // The kind in its colour, framed.
            let g = ui.painter().layout_no_wrap(
                kind_name(o.kind).to_string(),
                egui::FontId::monospace(12.0),
                colour,
            );
            let (r, _) = ui.allocate_exact_size(egui::vec2(96.0, 26.0), egui::Sense::hover());
            ui.painter().rect_stroke(
                r,
                0.0,
                egui::Stroke::new(1.0, colour),
                egui::StrokeKind::Inside,
            );
            let g = if g.size().x > 90.0 {
                ui.painter().layout_no_wrap(
                    kind_name(o.kind).to_string(),
                    egui::FontId::monospace(10.0),
                    colour,
                )
            } else {
                g
            };
            ui.painter().galley(r.center() - g.size() / 2.0, g, colour);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                let right = 92.0;
                ui.set_max_width(ui.available_width() - right);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(crate::mission::clean_text(&o.title.tr()))
                            .size(16.0)
                            .color(BRIGHT),
                    )
                    .selectable(false),
                );
                let mut line = crew_line(c, o);
                if !due.is_empty() {
                    line.insert(0, (due.clone(), AMBER));
                }
                if !line.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for (part, col) in line {
                            ui.add(
                                egui::Label::new(egui::RichText::new(part).small().color(col))
                                    .selectable(false),
                            );
                        }
                    });
                }
            });
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.add(
                    egui::Label::new(egui::RichText::new(format!("{km:.0} km")).color(BRIGHT))
                        .selectable(false),
                );
                if o.threat > 0 {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(t!("ops-threat", threat = o.threat))
                                .small()
                                .color(AMBER),
                        )
                        .selectable(false),
                    );
                }
            });
        });
    });
    ui.interact(inner.response.rect, egui::Id::new(("offer", o.id)), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The chosen offer's details under its row: what it asks, who is on it, the defences, the
/// package, the base to fly from and the stores; its deeds and the action bar's text.
fn offer_detail(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    o: &MissionOffer,
    deck: &mut Vec<Deed>,
    bar: &mut Option<(String, String)>,
) {
    let o = o.clone();
    ui.label(egui::RichText::new(describe_offer(c, &o)).color(DIM));
    if let Objective::Intercept { insertion } | Objective::Escort { insertion } = o.objective
        && let Some(t) = c.state().insertion_status(insertion)
    {
        ui.label(egui::RichText::new(t.tr()).color(BRIGHT));
    }
    // Nobody takes it in time: an AI helicopter does.
    if let Objective::Resupply { request } = o.objective
        && let Some(r) = c.state().supply_request(request)
        && let stealth_campaign::SupplyState::Open { since } = r.state
    {
        let wait = stealth_campaign::tuning::SUPPLY_AI_WAIT;
        let left = (wait - (c.state().clock - since)).max(0.0) as i32;
        let text = t!("ops-ai-helicopter", time = format!("{}:{:02}", left / 60, left % 60));
        ui.label(egui::RichText::new(text).color(BRIGHT));
    }
    // The site's cargo so far, how long its helipad waits, the AI's turn.
    if let Objective::Logistics { site, .. } = o.objective
        && let Some(d) = c.state().deployment(site)
    {
        let now = c.state().clock;
        let left = (d.expires - now).max(0.0) as i32;
        let text = t!(
            "ops-cargo-in",
            delivered = d.delivered.len(),
            loads = d.loads,
            time = format!("{}:{:02}", left / 3600, left / 60 % 60)
        );
        ui.label(egui::RichText::new(text).color(BRIGHT));
        let wait = stealth_campaign::tuning::DEPLOY_AI_WAIT;
        let ai = (wait - (now - d.open_since)).max(0.0) as i32;
        let text = t!("ops-ai-load", time = format!("{}:{:02}", ai / 60, ai % 60));
        ui.label(egui::RichText::new(text).color(DIM));
    }
    group_label(ui, c, &o);
    ui.add_space(6.0);
    if let Some(p) = place_of(&o.objective).and_then(|p| c.state().place(p)) {
        let intel = p.intel(c.state().clock);
        let sams =
            p.defences.iter().filter(|&&d| c.state().site(d).is_some_and(|s| s.alive())).count();
        let line = t!("ops-defences", sams = sams, threat = o.threat);
        ui.label(if intel { format!("{line}  -  {}", t!("ops-recon-intel")) } else { line });
    }
    package_info(ui, c, &o);
    if o.kind.ai_flown()
        && let Objective::Insertion { place } = o.objective
    {
        deck.push(Deed::new(
            Todo::OrderInsertion { place },
            t!("ops-order-insertion"),
            icon::HELICOPTER,
            t!("ops-order-insertion.facts"),
        ));
    }
    // The aircraft that flies it: the one last flown if it is made for it, else the best
    // one for it.
    let aircraft = o.kind.pick(c.aircraft);
    // From which base.
    let mut bases: Vec<(u16, String, i32)> = c
        .state()
        .player_bases()
        .into_iter()
        .filter_map(|b| {
            let site = c.state().site(b)?;
            Some((b, site.name.clone(), dist(site.x, site.y, o.x, o.y)))
        })
        .collect();
    bases.sort_by_key(|b| b.2);
    // Another player's plan sets the base (a plan nobody is on: its base unless the
    // player picks another).
    let plan = c.state().plan(o.id).filter(|_| c.role != Role::Local);
    let taken = plan.is_some_and(|p| !p.crew.is_empty() || !p.flying.is_empty());
    let planned = plan.map(|p| p.base).filter(|b| bases.iter().any(|x| x.0 == *b));
    let chosen = match planned {
        Some(b) if taken => b,
        _ => c.ui.base.filter(|b| bases.iter().any(|x| x.0 == *b)).or(planned).unwrap_or(o.base),
    };
    ui.add_space(4.0);
    ui.label(egui::RichText::new(t!("ops-from")).small().color(AMBER));
    ui.add_enabled_ui(!taken, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (b, name, d) in bases.iter().take(6) {
                let text = format!("{name} {:.0} km", km(*d));
                let glyph = (*b == chosen).then_some(icon::DONE);
                if shell::button(ui, glyph, &text, *b == chosen, 44.0, None).clicked() {
                    c.ui.base = Some(*b);
                    c.ui.focus = Some(egui::vec2(o.x as f32, o.y as f32));
                }
            }
        });
    });
    if taken {
        ui.label(egui::RichText::new(t!("ops-planned-by-group")).small().color(DIM));
    }
    let sortie = preview(&o, chosen, aircraft);
    let m = sortie_mission(&c.mission, c.state(), &sortie);
    let loadout = crate::loadout::PlannedLoadout::of(&m, kind_of(aircraft));
    let text = t!("ops-stores", aircraft = kind_of(aircraft).name(), stores = loadout.summary());
    ui.label(egui::RichText::new(&text).small().color(DIM));
    let from = bases
        .iter()
        .find(|b| b.0 == chosen)
        .map_or(String::new(), |b| format!("{} · {:.0} km", b.1, km(b.2)));
    *bar = Some((
        format!("{} - {}", kind_name(o.kind), crate::mission::clean_text(&o.title.tr())),
        format!("{from} · {text}"),
    ));
    support_button(ui, c, &o);
    // The AI's flight is up: fly along with it, or take over one of its aircraft.
    let ai = c.state().ai_mission_for(o.id).filter(|m| m.open()).and_then(|m| m.takeable());
    if let Some(tag) = ai {
        let mut deed = Deed::new(
            Todo::TakeControl { tag, watched: false },
            t!("ops-take-control"),
            icon::JOIN,
            t!("ops-take-control.facts"),
        );
        deed.why = take_over_refusal(c);
        deck.push(deed);
    }
    let (accept, mark) = if o.kind.ai_flown() {
        (t!("ops-fly-uh60"), icon::HELICOPTER)
    } else if ai.is_some() {
        (t!("ops-fly-with-ai"), icon::ACCEPT)
    } else {
        (t!("ops-accept"), icon::ACCEPT)
    };
    let todo = Todo::Accept { offer: o.id, base: chosen, aircraft, loadout };
    deck.push(Deed::new(todo, accept, mark, "").primary());
}

/// What a footer button does when pressed.
enum Todo {
    OrderInsertion {
        place: stealth_campaign::PlaceId,
    },
    Accept {
        offer: u32,
        base: u16,
        aircraft: Airframe,
        loadout: crate::loadout::PlannedLoadout,
    },
    /// TAKE CONTROL of an AI aircraft (campaign tag `tag`); `watched`: from the war map's view
    /// of it, which closes.
    TakeControl {
        tag: u32,
        watched: bool,
    },
    /// Sit at the gunner's console of an AI aircraft (`tag`; the AI flies on), or of another
    /// player's aircraft.
    GunnerAi {
        tag: u32,
    },
    GunnerWith {
        owner: u8,
    },
    /// Man an air defence unit's console.
    Man {
        unit: u16,
    },
    /// Open a pane.
    Open(Pane),
    /// Watch an air defence unit live.
    Watch {
        unit: usize,
        title: String,
    },
}

/// A big button a tab asks of the footer, where the other windows have theirs.
struct Deed {
    todo: Todo,
    label: String,
    icon: char,
    primary: bool,
    /// What it does, on hover.
    facts: String,
    /// Why it can't be pressed now (None: it can).
    why: Option<String>,
}

impl Deed {
    fn new(todo: Todo, label: impl AsRef<str>, icon: char, facts: impl AsRef<str>) -> Self {
        Self {
            todo,
            label: label.as_ref().to_string(),
            icon,
            primary: false,
            facts: facts.as_ref().to_string(),
            why: None,
        }
    }

    fn primary(mut self) -> Self {
        self.primary = true;
        self
    }
}

/// What a footer button does.
fn do_deed(c: &mut ActiveCampaign, unit_view: &mut super::UnitView, todo: Todo) {
    c.ui.error = None;
    match todo {
        Todo::OrderInsertion { place } => c.actions.push(Action::OrderInsertion { place }),
        Todo::Accept { offer, base, aircraft, loadout } => {
            if c.role != Role::Local {
                c.actions.push(Action::SelectMission { offer: Some(offer), base });
            }
            c.ui.briefing = Some(Briefing::new(offer, base, aircraft, loadout, Step::Aircraft));
        }
        Todo::TakeControl { tag, watched } => {
            let loadout = super::control_loadout(c, tag);
            c.actions.push(Action::TakeControl { tag, loadout, seat: Seat::Pilot });
            if watched {
                unit_view.close();
            }
        }
        Todo::GunnerAi { tag } => {
            let loadout = super::control_loadout(c, tag);
            c.actions.push(Action::TakeControl { tag, loadout, seat: Seat::Gunner });
            unit_view.close();
        }
        Todo::GunnerWith { owner } => {
            c.actions.push(Action::Crew(CrewCmd::Join { owner }));
            unit_view.close();
        }
        Todo::Man { unit } => {
            c.actions.push(Action::Station(StationCmd::Man { unit }));
            unit_view.close();
        }
        Todo::Open(pane) => c.ui.open(Some(pane)),
        Todo::Watch { unit, title } => {
            unit_view.watch(super::unit_view::UnitRef::Aaa(unit), title);
        }
    }
}

/// Why this player can't TAKE CONTROL of an AI aircraft now (None: they can).
fn take_over_refusal(c: &ActiveCampaign) -> Option<String> {
    (c.phase != Phase::Ops || c.state().sortie(c.me).is_some())
        .then(|| t!("ce-finish-sortie").to_string())
}

/// The war map's deeds: CONTROL of the watched aircraft, or air defence unit.
fn map_deeds(c: &ActiveCampaign, sim: &stealth_sim::Sim, unit_view: &super::UnitView) -> Vec<Deed> {
    match unit_view.target {
        // A friendly Patriot or NASAMS battery or Gepard: its console.
        Some(super::unit_view::UnitRef::Aaa(k)) => {
            let Some(u) = sim.units().get(k) else { return Vec::new() };
            if !sim.aaa_blue(k) || !stealth_sim::station::mannable(u.kind) {
                return Vec::new();
            }
            let mut deed = Deed::new(
                Todo::Man { unit: k as u16 },
                t!("ops-control"),
                icon::CONTROL,
                t!("ops-control.facts"),
            );
            deed.why = man_refusal(c, sim, k);
            vec![deed]
        }
        // An aircraft flying a mission for the friendly AI: fly it on.
        Some(super::unit_view::UnitRef::Flight { index, .. }) => {
            let Some(f) = sim.campaign_flight(index).filter(|f| f.blue) else {
                return Vec::new();
            };
            let Some(m) = c.state().ai_mission_of_tag(f.tag).filter(|m| m.open()) else {
                return Vec::new();
            };
            // Still taking off (or landing): not yet.
            if !m.flying().any(|t| t == f.tag) || !sim.takeable(f.tag) {
                return Vec::new();
            }
            let mut deed = Deed::new(
                Todo::TakeControl { tag: f.tag, watched: true },
                t!("ops-control"),
                icon::JOIN,
                t!("ops-take-control.facts"),
            );
            deed.why = take_over_refusal(c);
            vec![deed]
        }
        _ => Vec::new(),
    }
}

/// The players on an offer's plan (coop): planning it, flying it.
fn group(c: &ActiveCampaign, offer: u32) -> (Vec<String>, Vec<String>) {
    let st = c.state();
    let names = |ids: &[PlayerId]| -> Vec<String> {
        ids.iter().map(|&p| st.player_name(p).unwrap_or("?").to_string()).collect()
    };
    match st.plan(offer) {
        Some(p) if c.role != Role::Local => (names(&p.crew), names(&p.flying)),
        _ => default(),
    }
}

/// Light grey: the AI's flights on the board.
const AI_GREY: egui::Color32 = egui::Color32::from_gray(185);

/// Who is planning and flying an offer, and the AI's flight on it (its aircraft in the air,
/// when the next takes off): the parts of its line on the board, each in its colour.
fn crew_line(c: &ActiveCampaign, o: &MissionOffer) -> Vec<(String, egui::Color32)> {
    let st = c.state();
    let (planning, flying) = group(c, o.id);
    let mut line = Vec::new();
    if !planning.is_empty() {
        line.push((t!("ops-planning", pilots = planning.join(", ")).to_string(), FRIENDLY_BLUE));
    }
    if !flying.is_empty() {
        line.push((t!("ops-flying", pilots = flying.join(", ")).to_string(), GREEN));
    }
    if let Some(m) = st.ai_mission_for(o.id).filter(|m| m.open()) {
        let ai: Vec<String> =
            m.flying().map(|_| t!("ops-ai-aircraft", aircraft = m.airframe.name())).collect();
        if !ai.is_empty() {
            line.push((t!("ops-flying", pilots = ai.join(", ")).to_string(), AI_GREY));
        }
    }
    // (Due, but held back: not enough aircraft or materiel.)
    if let Some(at) = st.ai_launch_at(o).filter(|&t| t > st.clock) {
        let left = (at - st.clock).ceil() as i32;
        let time = format!("{}:{:02}", left / 60, left % 60);
        line.push((t!("ops-ai-takes-off", time = time).to_string(), AI_GREY));
    }
    line
}

/// The capture of an enemy place an offer prepares: the place, and its steps in order
/// (recon, SEAD, a strike on its airfield, then the insertion that takes it), each done or not.
fn operation_steps(
    c: &ActiveCampaign,
    o: &MissionOffer,
    front: &[u16],
) -> Option<(String, Vec<(MissionKind, bool)>)> {
    let st = c.state();
    let op = place_of(&o.objective).and_then(|p| st.operation_on(p, front))?;
    let place = st.place(op.place).map_or(String::new(), |p| p.name.clone());
    let mut steps = op.steps;
    steps.push((MissionKind::Insertion, false));
    Some((place, steps))
}

/// A sequence of steps as boxes joined by lines: the ones done darkened, the next one to do
/// inverted, the others outlined in green (`on`: drawn over a selected row of that colour).
fn task_sequence(ui: &mut egui::Ui, steps: &[(MissionKind, bool)], on: Option<egui::Color32>) {
    const JOIN: f32 = 10.0;
    let (ink, back) = match on {
        Some(fill) => (BG, fill),
        None => (GREEN, ROW),
    };
    let next = steps.iter().position(|s| !s.1);
    let font = egui::TextStyle::Small.resolve(ui.style());
    ui.spacing_mut().item_spacing.x = 0.0;
    for (k, &(kind, done)) in steps.iter().enumerate() {
        if k > 0 {
            let (r, _) = ui.allocate_exact_size(egui::vec2(JOIN, 14.0), egui::Sense::hover());
            ui.painter()
                .line_segment([r.left_center(), r.right_center()], egui::Stroke::new(1.0, ink));
        }
        let (fill, text, stroke) = if done {
            (ink.gamma_multiply(0.35), back.lerp_to_gamma(ink, 0.45), ink.gamma_multiply(0.35))
        } else if Some(k) == next {
            (ink, back, ink)
        } else {
            (egui::Color32::TRANSPARENT, ink, ink)
        };
        let galley = ui.painter().layout_no_wrap(kind_name(kind).to_string(), font.clone(), text);
        let size = galley.size() + egui::vec2(10.0, 4.0);
        let (r, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        ui.painter().rect(r, 1.0, fill, egui::Stroke::new(1.0, stroke), egui::StrokeKind::Inside);
        ui.painter().galley(r.center() - galley.size() / 2.0, galley, text);
    }
}

/// [`crew_line`] under an offer (its details, the briefing).
fn group_label(ui: &mut egui::Ui, c: &ActiveCampaign, o: &MissionOffer) {
    let line = crew_line(c, o);
    if line.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for (text, col) in line {
            ui.label(egui::RichText::new(text).small().color(col));
        }
    });
}

/// Height left in the tab.
fn height_of(ui: &egui::Ui) -> f32 {
    ui.available_height().max(120.0)
}

fn place_of(o: &Objective) -> Option<u16> {
    match *o {
        Objective::Recon { place, .. }
        | Objective::Sead { place, .. }
        | Objective::Cas { place, .. }
        | Objective::Strike { place, .. }
        | Objective::Interdict { place, .. }
        | Objective::Support { place, .. }
        | Objective::Insertion { place } => Some(place),
        _ => None,
    }
}

/// What an offer asks for (a drone wave is no air raid).
fn describe_offer(c: &ActiveCampaign, o: &MissionOffer) -> &'static str {
    match o.objective {
        Objective::Defend { raid } if c.state().raid(raid).is_some_and(|r| r.drones) => {
            t!("brief-task-drones")
        }
        Objective::Defend { raid } if c.state().raid(raid).is_some_and(|r| r.sam.is_some()) => {
            t!(stealth_host::glue::SEAD_DEFENCE_TASK)
        }
        Objective::Battery { .. } => t!(stealth_host::glue::BATTERY_TASK),
        Objective::Armour { .. } => t!(stealth_host::glue::ARMOUR_TASK),
        Objective::Support { .. } => t!(stealth_host::glue::SUPPORT_TASK),
        _ => describe(o.kind),
    }
}

/// The other offers at an offer's place: flown together, they are a package.
fn package_info(ui: &mut egui::Ui, c: &ActiveCampaign, o: &MissionOffer) {
    use stealth_campaign::tuning::{PACKAGE_BONUS, PACKAGE_WINDOW};
    let st = c.state();
    let Some(place) = st.objective_place(&o.objective) else { return };
    let mut kinds: Vec<&str> = st
        .board
        .iter()
        .filter(|x| x.id != o.id && x.kind != MissionKind::Insertion && x.kind != o.kind)
        .filter(|x| st.objective_place(&x.objective) == Some(place))
        .map(|x| kind_name(x.kind))
        .collect();
    kinds.dedup();
    if kinds.is_empty() {
        return;
    }
    ui.label(
        egui::RichText::new(t!(
            "ops-package",
            kinds = kinds.join(", "),
            mins = (PACKAGE_WINDOW / 60.0).round() as i64,
            bonus = PACKAGE_BONUS as i64
        ))
        .small()
        .color(GREEN),
    );
}

/// Calling in AI SEAD for an offer's place (the friendly side's materiel and fighters).
fn support_button(ui: &mut egui::Ui, c: &mut ActiveCampaign, o: &MissionOffer) {
    use stealth_campaign::tuning::{
        AI_FLIGHT_AIRCRAFT, AI_MISSION_COST, RAID_AIRCRAFT, SUPPORT_COST,
    };
    let st = c.state();
    let Some(place) = st.objective_place(&o.objective) else { return };
    let Some(p) = st.place(place).filter(|p| p.owner == Side::Red) else { return };
    let sams = p.defences.iter().filter(|&&d| st.site(d).is_some_and(|s| s.alive())).count();
    if sams == 0 {
        return;
    }
    // A SEAD offer the AI flies: its own flight, at once (none while one is due or up).
    let ai = o.kind == MissionKind::Sead && st.ai_missions_on();
    if ai && st.ai_scheduled(o) {
        return;
    }
    let busy = st.raids.iter().any(|r| r.side == Side::Blue && r.open() && r.place == place);
    if busy && !ai {
        ui.label(egui::RichText::new(t!("ops-sead-on-way")).small().color(FRIENDLY_BLUE));
        return;
    }
    let (cost, aircraft) =
        if ai { (AI_MISSION_COST, AI_FLIGHT_AIRCRAFT) } else { (SUPPORT_COST, RAID_AIRCRAFT) };
    let can = st.blue.materiel >= cost && st.blue.fighters as usize >= aircraft;
    let label = t!("ops-request-sead", cost = cost.round() as i64, aircraft = aircraft);
    let offer = o.id;
    if ui
        .add_enabled(can, egui::Button::new(egui::RichText::new(label).color(FRIENDLY_BLUE)))
        .on_hover_text(t!("ops-request-sead.facts"))
        .on_disabled_hover_text(t!("ops-request-sead.disabled"))
        .clicked()
    {
        c.ui.error = None;
        c.actions.push(Action::RequestSupport { offer });
    }
}

/// A rank for campaign points (the career's ranks).
fn rank_of(points: i32) -> &'static str {
    let k = crate::career::RANKS.iter().rposition(|r| points >= r.2).unwrap_or(0);
    crate::career::rank_abbreviation(k)
}

/// The medal a sortie's points earn (the career's), if any.
fn medal_of(points: i32) -> Option<&'static str> {
    crate::career::MEDALS
        .iter()
        .rev()
        .find(|m| points >= m.1)
        .map(|m| crate::career::medal_name(m.0))
}

/// A kind of mission's name on the board ("STRIKE").
pub(super) fn kind_name(k: MissionKind) -> &'static str {
    t!(&format!("ck-{}", k.name().to_ascii_lowercase()))
}

/// SAVE & EXIT, or DISCONNECT from a server.
fn exit_label(role: Role) -> &'static str {
    t!(exit_id(role))
}

/// SAVE & EXIT's message id, or DISCONNECT's.
pub(super) fn exit_id(role: Role) -> &'static str {
    if role == Role::Client { "ops-disconnect" } else { "ops-save-exit" }
}

/// The campaign's rules in words.
fn rules_text(r: &stealth_host::Rules) -> String {
    let on: Vec<&str> = [
        (r.realistic_damage, "ops-rule-damage"),
        (r.realistic_guidance, "ops-rule-guidance"),
        (r.no_crashes, "ops-rule-no-crashes"),
        (r.continuous_time, "ops-rule-time"),
        (r.realistic_weapons, "ops-rule-weapons"),
        (r.friendly_fire, "ops-rule-friendly-fire"),
        (r.classic_terrain, "ops-rule-classic-terrain"),
        (r.classic_ai_flight, "ops-rule-classic-ai-flight"),
        (r.airframe_losses, "ops-rule-airframe-losses"),
        (r.no_ai_missions, "ops-rule-no-ai-missions"),
    ]
    .iter()
    .filter(|x| x.0)
    .map(|x| t!(x.1))
    .collect();
    if on.is_empty() { t!("ops-rule-classic").into() } else { on.join(", ") }
}

/// Both sides' materiel and income at a glance (the footer).
fn supply_line(ui: &mut egui::Ui, c: &ActiveCampaign, full: bool) {
    let st = c.state();
    if full {
        ui.label(egui::RichText::new(t!("ops-materiel")).color(DIM));
    }
    for (side, id, col) in [
        (Side::Blue, "ops-materiel-blue", FRIENDLY_BLUE),
        (Side::Red, "ops-materiel-red", MISSILE_RED),
    ] {
        let (have, income) = (st.forces(side).materiel.floor(), st.income(side));
        let text = t!(id, materiel = format!("{have:.0}"), income = format!("{income:.0}"));
        ui.label(egui::RichText::new(text).color(col));
    }
    // Which way the front is moving.
    let m = st.momentum();
    let (id, col) = if m >= tuning::MOMENTUM_SHOWN {
        ("ops-front-blue", FRIENDLY_BLUE)
    } else if m <= -tuning::MOMENTUM_SHOWN {
        ("ops-front-red", MISSILE_RED)
    } else {
        ("ops-front-even", DIM)
    };
    ui.label(egui::RichText::new(t!(id)).color(col));
    // More players, a faster enemy (coop scaling).
    let pace = st.pace();
    if pace > 1.05 {
        let text = t!("ops-crew-tempo", pace = format!("{pace:.1}"));
        ui.label(egui::RichText::new(text).color(MISSILE_RED));
    }
}

/// The two sides' means of war side by side: materiel, income and where it comes from, the
/// AI's aircraft, where the materiel went, and what everything costs.
pub(super) fn forces(ui: &mut egui::Ui, c: &ActiveCampaign) {
    use stealth_campaign::tuning::*;
    let st = c.state();
    let sides = [
        (Side::Blue, t!("ops-friendly"), FRIENDLY_BLUE),
        (Side::Red, t!("ops-enemy"), MISSILE_RED),
    ];
    egui::ScrollArea::vertical().id_salt("forces").show(ui, |ui| {
        section(ui, t!("ops-section-forces"));
        egui::Grid::new("forces").num_columns(3).spacing(egui::vec2(40.0, 6.0)).striped(true).show(
            ui,
            |ui| {
                let row = |ui: &mut egui::Ui, label: &str, f: &dyn Fn(Side) -> String| {
                    ui.label(egui::RichText::new(label).color(DIM));
                    for (side, _, col) in sides {
                        ui.label(egui::RichText::new(f(side)).monospace().color(col));
                    }
                    ui.end_row();
                };
                ui.label("");
                for (_, name, col) in sides {
                    ui.label(egui::RichText::new(name).strong().color(col));
                }
                ui.end_row();
                row(ui, t!("ops-places-held"), &|s| st.held(s).to_string());
                row(ui, t!("ops-materiel-row"), &|s| {
                    format!("{:.0} / {MATERIEL_MAX:.0}", st.forces(s).materiel.floor())
                });
                row(ui, t!("ops-income"), &|s| format!("+{:.0}", st.income(s)));
                row(ui, &format!("  {}", t!("ops-income-from")), &|s| {
                    let (places, cities) = st.producers(s);
                    t!(
                        "ops-income-sources",
                        base = INCOME_BASE.round() as i64,
                        places = places,
                        per_place = INCOME_PLACE.round() as i64,
                        cities = cities,
                        per_city = INCOME_CITY.round() as i64
                    )
                });
                row(ui, t!("ops-fighters-ground"), &|s| {
                    format!("{} / {}", st.forces(s).fighters, st.aircraft_limits(s).0)
                });
                row(ui, t!("ops-helicopters-ground"), &|s| {
                    format!("{} / {}", st.forces(s).helicopters, st.aircraft_limits(s).1)
                });
                row(ui, t!("ops-in-air"), &|s| {
                    let (f, strike, h) = st.airborne(s);
                    t!("ops-in-air-value", fighters = f, strike = strike, helicopters = h)
                });
                ui.label(egui::RichText::new(t!("ops-since-start")).color(AMBER));
                ui.end_row();
                let l = |s: Side| st.forces(s).ledger.clone();
                let out = |v: f64| if v < 0.5 { "0".to_string() } else { format!("-{v:.0}") };
                let sub = |id: &str| format!("  {}", t!(id));
                row(ui, &sub("ops-earned"), &|s| format!("+{:.0}", l(s).earned));
                row(ui, &sub("ops-repairs"), &|s| out(l(s).repairs));
                row(ui, &sub("ops-operations"), &|s| out(l(s).operations));
                row(ui, &sub("ops-new-aircraft"), &|s| out(l(s).aircraft));
                row(ui, &sub("ops-lost"), &|s| out(l(s).lost));
            },
        );
        ui.add_space(10.0);
        section(ui, t!("ops-section-how"));
        for line in stealth_i18n::t_lines!("ops-how") {
            ui.label(egui::RichText::new(*line).color(BRIGHT));
            ui.add_space(2.0);
        }
        ui.label(egui::RichText::new(t!("ops-how-repairs")).color(BRIGHT));
        ui.add_space(2.0);
        // The rule that makes the players' losses count.
        if c.rules.airframe_losses {
            ui.label(egui::RichText::new(t!("ops-how-airframes")).color(AMBER));
            ui.add_space(2.0);
        }
        ui.add_space(8.0);
        section(ui, t!("ops-section-prices"));
        egui::Grid::new("prices").num_columns(4).spacing(egui::vec2(28.0, 4.0)).show(ui, |ui| {
            let prices = [
                ("ops-price-airfield", REPAIR_COST_AIRFIELD),
                ("ops-price-sam", REPAIR_COST_SAM),
                ("ops-price-city", REPAIR_COST_CITY),
                ("ops-price-ship", REPAIR_COST_SHIP),
                ("ops-price-gun", GUN_COST),
                ("ops-price-sortie", SORTIE_COST),
                ("ops-price-raid", RAID_COST),
                ("ops-price-insertion", INSERTION_COST),
                ("ops-price-fighter", FIGHTER_COST),
                ("ops-price-helicopter", HELICOPTER_COST),
                ("ops-price-city-lost", CITY_LOSS),
                ("ops-price-captured", CAPTURE_COST),
            ];
            for pair in prices.chunks(2) {
                for (name, cost) in pair {
                    ui.label(egui::RichText::new(t!(name)).color(DIM));
                    ui.label(egui::RichText::new(format!("{cost:.0}")).monospace().color(BRIGHT));
                }
                ui.end_row();
            }
        });
    });
}

/// A place's tooltip: an airfield or a city, its name, its side and the recon on it; its
/// symbol `filled` as on the war map, else as on the briefing's.
pub(super) fn place_tip(
    tips: &mut MapTips,
    c0: egui::Pos2,
    p: &stealth_campaign::Place,
    now: f64,
    filled: bool,
    production: Option<super::logistics::Production>,
) {
    let (owner, airfield) = (affiliation(p.owner), p.airfield.is_some());
    let (side, colour) = map_draw::side(owner);
    let kind = if airfield { t!("mt-airfield") } else { t!("mt-city") };
    let tip = tips.add(c0, kind, crate::mission::clean_text(&p.name), colour).line(side);
    tip.symbol(move |p, c| map_draw::place_symbol(p, c, owner, airfield, filled));
    if p.intel(now) {
        tip.line(t!("pic-reconned"));
    }
    if let Some(kind) = production {
        tip.line(kind.tip());
    }
}

/// A place on the map is shown: its layer (airfields, cities) is.
pub(super) fn shows_place(layers: &MapLayers, p: &stealth_campaign::Place) -> bool {
    layers.shows(if p.airfield.is_some() { Layer::Airfields } else { Layer::Cities })
}

/// The places' legend lines, with their layers' checkboxes.
pub(super) fn place_legend(ui: &mut egui::Ui, layers: &mut MapLayers) {
    layers.legend(ui, Layer::Airfields, t!("mt-production-base"), |p, c0| {
        let base = super::logistics::Production::Base;
        super::logistics::production_mark(p, c0 + egui::vec2(11.0, 7.0), 1.0, base);
    });
    if map_draw::app6() {
        use crate::app6::Affiliation::{Friend, Hostile};
        for (layer, text, airfield) in [
            (Layer::Airfields, t!("ops-legend-airfield"), true),
            (Layer::Cities, t!("ops-legend-city"), false),
        ] {
            layers.legend(ui, layer, text, |p, c0| {
                map_draw::app6_place(p, c0 + egui::vec2(-7.5, 1.0), Friend, airfield, 0.65);
                map_draw::app6_place(p, c0 + egui::vec2(7.5, 1.0), Hostile, airfield, 0.65);
            });
        }
        return;
    }
    layers.legend(ui, Layer::Airfields, t!("ops-legend-airfield"), |p, c0| {
        for (dx, col) in [(-4.5, FRIENDLY_BLUE), (5.0, MISSILE_RED)] {
            p.rect_stroke(
                egui::Rect::from_center_size(c0 + egui::vec2(dx, 0.0), egui::vec2(7.0, 7.0)),
                0.0,
                egui::Stroke::new(1.5, col),
                egui::StrokeKind::Middle,
            );
        }
    });
    layers.legend(ui, Layer::Cities, t!("ops-legend-city"), |p, c0| {
        p.circle_stroke(c0 - egui::vec2(4.0, 0.0), 3.5, egui::Stroke::new(1.5, FRIENDLY_BLUE));
        p.circle_stroke(c0 + egui::vec2(5.0, 0.0), 3.5, egui::Stroke::new(1.5, MISSILE_RED));
    });
}

/// A side as an APP-6 affiliation.
pub(super) fn affiliation(side: Side) -> crate::app6::Affiliation {
    match side {
        Side::Blue => crate::app6::Affiliation::Friend,
        Side::Red => crate::app6::Affiliation::Hostile,
        Side::Neutral => crate::app6::Affiliation::Neutral,
    }
}

/// The war map's view the first time: all the places in `rect`.
pub(super) fn fit_map(c: &mut ActiveCampaign, rect: egui::Rect) {
    // (Not before the window has its size: egui's first screen is 10000 points wide.)
    if c.ui.fitted || rect.width() >= 8000.0 {
        return;
    }
    // The war that matters: the friendly places and the enemy's within reach of them (all of
    // them if that leaves nothing).
    let st = c.state();
    let reach = 150 * 64;
    let near = |p: &stealth_campaign::Place| {
        p.owner == Side::Blue
            || st.places.iter().any(|q| {
                q.owner == Side::Blue && stealth_campaign::dist(q.x - p.x, q.y - p.y) <= reach
            })
    };
    let mut active: Vec<&stealth_campaign::Place> = st.places.iter().filter(|p| near(p)).collect();
    if active.len() < 2 {
        active = st.places.iter().collect();
    }
    let (lo, hi) = active.iter().fold(
        (egui::vec2(f32::MAX, f32::MAX), egui::vec2(f32::MIN, f32::MIN)),
        |(lo, hi), p| {
            let v = egui::vec2(p.x as f32, p.y as f32);
            (lo.min(v), hi.max(v))
        },
    );
    (c.ui.centre, c.ui.scale) = MapView::fit(rect, lo, hi);
    // Test aid: `--map-view <x>,<y>,<map units per point>` frames the map there.
    if let Some(v) = std::env::args().skip_while(|a| a != "--map-view").nth(1) {
        let n: Vec<f32> = v.split(',').filter_map(|s| s.parse().ok()).collect();
        if let [x, y, scale] = n[..] {
            (c.ui.centre, c.ui.scale) = (egui::vec2(x, y), scale);
        }
    }
    c.ui.fitted = true;
}

/// Why this player can't man air defence unit `k`'s console now (None: they can).
pub(super) fn man_refusal(c: &ActiveCampaign, sim: &stealth_sim::Sim, k: usize) -> Option<String> {
    if sim.units().get(k).is_none_or(|u| u.destroyed) {
        return Some(t!("st-destroyed").to_string());
    }
    if c.phase == Phase::Sortie {
        return Some(t!("ce-finish-sortie").to_string());
    }
    sim.manned_by(k).filter(|&p| p != c.me.0).map(|p| match c.state().player_name(PlayerId(p)) {
        Some(name) => t!("ops-manned-by", pilot = name),
        None => t!("ops-manned-by-other").to_string(),
    })
}

fn debrief(ui: &mut egui::Ui, c: &ActiveCampaign) {
    let Some(d) = &c.debrief else {
        ui.label(egui::RichText::new(t!("ops-no-sortie")).color(DIM));
        return;
    };
    let r = &d.record;
    section(ui, t!("ops-section-sortie"));
    ui.label(egui::RichText::new(r.title.tr()).size(22.0).color(BRIGHT));
    ui.label(egui::RichText::new(d.title.tr().to_uppercase()).color(AMBER));
    let (text, col) = match r.result {
        SortieResult::Success => (t!("ops-achieved"), GREEN),
        SortieResult::Failed => (t!("ops-not-achieved"), AMBER),
        SortieResult::Pending if r.raid.is_some() => (t!("ops-waiting-raid"), FRIENDLY_BLUE),
        SortieResult::Pending => (t!("ops-waiting-insertion"), FRIENDLY_BLUE),
    };
    // An escort settled after the sortie shows its final result.
    let settled = c.state().history.iter().find(|h| h.id == r.id).map_or(r.result, |h| h.result);
    let (text, col) = if settled != r.result {
        match settled {
            SortieResult::Success => (t!("ops-achieved"), GREEN),
            _ => (t!("ops-not-achieved"), AMBER),
        }
    } else {
        (text, col)
    };
    ui.label(egui::RichText::new(text).size(20.0).color(col));
    ui.add_space(6.0);
    let mins = ((r.ended - r.started) / 60.0).round();
    ui.label(t!("ops-flight-time", mins = mins as i64));
    ui.label(t!("ops-kills", air = r.air_kills, ground = r.ground_kills));
    if !r.crew.is_empty() {
        ui.label(t!("ops-crew", names = r.crew.join(", ")));
    }
    let points = c.state().history.iter().find(|h| h.id == r.id).map_or(r.points, |h| h.points);
    let bonus = c.state().history.iter().find(|h| h.id == r.id).map_or(r.bonus, |h| h.bonus);
    ui.label(
        egui::RichText::new(if bonus > 0 {
            t!("ops-points-bonus", points = points, bonus = bonus)
        } else {
            t!("ops-points", points = points)
        })
        .color(BRIGHT),
    );
    if let Some(m) = medal_of(points) {
        ui.label(egui::RichText::new(t!("career-awarded", medal = m)).color(AMBER));
    }
    let me = c.state().pilot_record(c.me);
    ui.label(
        egui::RichText::new(t!(
            "ops-campaign-points",
            rank = rank_of(me.points),
            points = me.points
        ))
        .color(DIM),
    );
    let done = c.state().history.iter().filter(|h| h.player == c.me).count();
    let won = c
        .state()
        .history
        .iter()
        .filter(|h| h.player == c.me && h.result == SortieResult::Success)
        .count();
    ui.label(egui::RichText::new(t!("ops-campaign-record", sorties = done, won = won)).color(DIM));
    // What the sortie changed in the war.
    if !r.effects.is_empty() {
        ui.add_space(8.0);
        section(ui, t!("ops-section-effects"));
        for e in &r.effects {
            ui.label(egui::RichText::new(e.tr()).color(GREEN));
        }
    }
    ui.add_space(8.0);
    section(ui, t!("ops-section-meanwhile"));
    egui::ScrollArea::vertical().id_salt("news").max_height(300.0).show(ui, |ui| {
        for n in &d.news {
            ui.label(egui::RichText::new(n.tr()).color(BRIGHT));
        }
    });
}

/// The war is won or lost: the result, every pilot's record, and (on a server) the time left
/// before the next campaign starts.
fn end_screen(ui: &mut egui::Ui, c: &ActiveCampaign) {
    let st = c.state();
    let Some(o) = st.outcome else { return };
    let won = o.winner == Side::Blue;
    ui.vertical_centered(|ui| {
        ui.add_space(16.0);
        let (title, col) =
            if won { (t!("ops-victory"), GREEN) } else { (t!("ops-defeat"), MISSILE_RED) };
        ui.label(egui::RichText::new(title).size(44.0).strong().color(col));
        ui.label(
            egui::RichText::new(t!(if won { "ops-victory.text" } else { "ops-defeat.text" }))
                .size(18.0)
                .color(BRIGHT),
        );
        let day = (st.sky_hours() / 24.0).floor() as i64 + 1;
        ui.label(
            egui::RichText::new(t!(
                "ops-end-day",
                day = day,
                friendly = st.held(Side::Blue),
                enemy = st.held(Side::Red)
            ))
            .color(DIM),
        );
        ui.add_space(10.0);
        if c.role == Role::Client {
            let left = (tuning::END_SCREEN_SECONDS - st.ended_for().unwrap_or(0.0)).max(0.0);
            let text = if left > 0.0 {
                let s = left.ceil() as i64;
                t!("ops-next-campaign", time = format!("{}:{:02}", s / 60, s % 60))
            } else {
                t!("ops-next-campaign-now").to_string()
            };
            ui.label(egui::RichText::new(text).size(20.0).color(AMBER));
        } else {
            ui.label(egui::RichText::new(t!("ops-next-from-menu")).color(DIM));
        }
        ui.add_space(12.0);
    });
    section(ui, t!("career-pilots"));
    egui::ScrollArea::vertical().id_salt("pilots").show(ui, |ui| leaderboard(ui, c, "end"));
}

fn log(ui: &mut egui::Ui, c: &ActiveCampaign) {
    section(ui, t!("ops-section-log"));
    egui::ScrollArea::vertical().id_salt("log").show(ui, |ui| {
        // The campaign's events and the coop chat, newest first.
        let s = c.state();
        let (mut log, mut chat) = (s.log.iter().rev().peekable(), s.chat.iter().rev().peekable());
        loop {
            let take_chat = match (log.peek(), chat.peek()) {
                (None, None) => break,
                (Some(l), Some(m)) => m.time >= l.time,
                (l, _) => l.is_none(),
            };
            let (time, from, text) = if take_chat {
                let m = chat.next().unwrap();
                (m.time, Some(&m.from), m.text.clone())
            } else {
                let e = log.next().unwrap();
                (e.time, None, e.text.tr())
            };
            let m = (time / 60.0).floor() as i64;
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!("T+{:02}:{:02}", m / 60, m % 60))
                        .monospace()
                        .color(DIM),
                );
                if let Some(from) = from {
                    ui.label(egui::RichText::new(format!("{from}:")).color(AMBER));
                }
                ui.label(egui::RichText::new(text).color(BRIGHT));
            });
        }
    });
}

/// A coop game's chat: every message so far, newest at the bottom, and a field to write one.
fn chat(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    notices: &mut crate::player_notices::PlayerNotices,
    vr: bool,
) {
    use crate::text_entry::{Charset, text_entry};
    use stealth_net::protocol::MAX_CHAT;
    section(ui, t!("ops-section-chat"));
    // The field's room under the list (in the headset with its keyboard).
    let below = if vr { 330.0 } else { 60.0 };
    let height = (ui.available_height() - below).max(120.0);
    egui::ScrollArea::vertical()
        .id_salt("chat")
        .max_height(height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if notices.log().is_empty() {
                ui.label(egui::RichText::new(t!("ops-no-messages")).color(DIM));
            }
            for m in notices.log() {
                ui.horizontal_wrapped(|ui| match &m.from {
                    Some(from) => {
                        let ink = if m.own { GREEN } else { AMBER };
                        ui.label(egui::RichText::new(format!("{from}:")).color(ink));
                        ui.label(egui::RichText::new(&m.text).color(BRIGHT));
                    }
                    None => {
                        ui.label(egui::RichText::new(&m.text).color(FRIENDLY_BLUE));
                    }
                });
            }
        });
    ui.add_space(8.0);
    let mut send = false;
    let mut field = |ui: &mut egui::Ui| {
        let resp = if vr {
            let hint = t!("ops-message");
            text_entry(ui, "ops_chat", &mut c.ui.chat, MAX_CHAT, Charset::Text, true, hint)
        } else {
            let field = egui::TextEdit::singleline(&mut c.ui.chat)
                .id(egui::Id::new("ops_chat"))
                .char_limit(MAX_CHAT)
                .hint_text(egui::RichText::new(t!("ops-message.hint")).color(DIM))
                .text_color(BRIGHT)
                .font(egui::FontId::monospace(18.0))
                .desired_width(ui.available_width() - 110.0);
            ui.add(field)
        };
        notices.field_focus |= resp.has_focus();
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            send = true;
            resp.request_focus();
        }
        if ui.add(big_button(t!("ops-send")).icon(icon::SEND)).clicked() {
            send = true;
        }
    };
    // In the headset the keyboard opens under the field.
    if vr {
        field(ui);
    } else {
        ui.horizontal(field);
    }
    if send && !stealth_net::protocol::chat_text(&c.ui.chat).is_empty() {
        notices.outbox.push(std::mem::take(&mut c.ui.chat));
    }
}

/// The red way out: SAVE & EXIT, or DISCONNECT from a server.
pub(crate) const EXIT_RED: egui::Color32 = egui::Color32::from_rgb(150, 24, 20);

/// SAVE & EXIT's icon, or DISCONNECT's.
pub(super) fn exit_icon(role: Role) -> char {
    if role == Role::Client { icon::DISCONNECT } else { icon::SAVE }
}

/// The players in the campaign: who flies what, who waits on the board.
fn players(ui: &mut egui::Ui, c: &ActiveCampaign, voice: &mut crate::voice::Voice) {
    section(ui, t!("ops-section-players"));
    let list = c.players();
    // Up to 64 of them: the list scrolls; the leaderboard under it.
    let height = (ui.available_height() * 0.45).max(120.0);
    egui::ScrollArea::vertical().max_height(height).auto_shrink([false, true]).show(ui, |ui| {
        egui::Grid::new("players")
            .num_columns(6)
            .spacing(egui::vec2(24.0, 8.0))
            .striped(true)
            .show(ui, |ui| {
                for h in [
                    "#",
                    "ops-col-name",
                    "ops-col-status",
                    "ops-col-sorties",
                    "ops-col-points",
                    "PING",
                ] {
                    let h = if h.starts_with("ops-") { t!(h) } else { h };
                    ui.label(egui::RichText::new(h).color(DIM));
                }
                ui.end_row();
                for p in &list {
                    ui.label(egui::RichText::new(format!("{}", p.id.0 + 1)).monospace().color(DIM));
                    let mut name = egui::RichText::new(&p.name).color(BRIGHT);
                    if p.id == c.me {
                        name = name.strong();
                    }
                    ui.horizontal(|ui| {
                        ui.label(name);
                        if p.id == c.me {
                            ui.label(egui::RichText::new(t!("ops-you")).small().color(DIM));
                        }
                        if p.host {
                            ui.label(
                                egui::RichText::new(t!("ops-host")).small().color(FRIENDLY_BLUE),
                            );
                        }
                        // The coop radio: talking now, and muting him.
                        if voice.talking.contains(&p.id.0) {
                            ui.label(egui::RichText::new(t!("ops-on-radio")).small().color(GREEN));
                        }
                        if p.id != c.me && c.role != Role::Local {
                            let muted = voice.muted.contains(&p.id.0);
                            let text = egui::RichText::new(if muted {
                                t!("ops-unmute")
                            } else {
                                t!("ops-mute")
                            })
                            .small()
                            .color(if muted { AMBER } else { DIM });
                            if ui.small_button(text).on_hover_text(t!("ops-mute.facts")).clicked() {
                                crate::voice::toggle_mute(voice, p.id.0);
                            }
                        }
                    });
                    let (status, col) = match &p.status {
                        PlayerStatus::Ops => {
                            let planning = c
                                .state()
                                .plans
                                .iter()
                                .find(|x| x.crew.contains(&p.id))
                                .and_then(|x| c.state().board.iter().find(|o| o.id == x.offer));
                            match planning {
                                Some(o) => {
                                    (t!("ops-status-planning", title = o.title.tr()), FRIENDLY_BLUE)
                                }
                                None => (t!("ops-status-board").to_string(), DIM),
                            }
                        }
                        PlayerStatus::Flying { title, aircraft } => {
                            let aircraft = kind_of(*aircraft).name();
                            (
                                t!("ops-status-flying", title = title.tr(), aircraft = aircraft),
                                GREEN,
                            )
                        }
                        PlayerStatus::Crew { title, aircraft, pilot } => {
                            let aircraft = kind_of(*aircraft).name();
                            (
                                t!(
                                    "ops-status-gunner",
                                    title = title.tr(),
                                    aircraft = aircraft,
                                    pilot = pilot.clone()
                                ),
                                GREEN,
                            )
                        }
                        PlayerStatus::Reconnecting { seconds } => {
                            (t!("ops-status-reconnecting", seconds = *seconds), AMBER)
                        }
                    };
                    ui.label(egui::RichText::new(status).color(col));
                    let flown = c.state().records(p.id).count();
                    let won = c
                        .state()
                        .records(p.id)
                        .filter(|r| r.result == SortieResult::Success)
                        .count();
                    ui.label(t!("ops-flown", flown = flown, won = won));
                    let points = c.state().pilot_record(p.id).points;
                    ui.label(format!("{points} {}", rank_of(points)));
                    ui.label(if p.ping > 0 { format!("{} ms", p.ping) } else { "-".into() });
                    ui.end_row();
                }
            });
    });
    ui.add_space(8.0);
    section(ui, t!("ops-section-leaderboard"));
    let height = (ui.available_height() - 70.0).max(100.0);
    egui::ScrollArea::vertical()
        .id_salt("leaders")
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| leaderboard(ui, c, "leaders"));
    if c.role == Role::Local {
        ui.add_space(10.0);
        ui.label(egui::RichText::new(t!("ops-alone")).color(DIM));
    }
    ui.add_space(10.0);
    ui.label(egui::RichText::new(t!("ops-rules", rules = rules_text(&c.rules))).color(DIM));
}

/// Every pilot of the campaign (who has left too), the most points first.
fn leaderboard(ui: &mut egui::Ui, c: &ActiveCampaign, id: &str) {
    let rows: Vec<_> = c.state().pilot_records().into_iter().filter(|p| p.sorties > 0).collect();
    egui::Grid::new(id).num_columns(7).spacing(egui::vec2(22.0, 6.0)).striped(true).show(
        ui,
        |ui| {
            for h in [
                "ops-col-pilot",
                "career-rank",
                "ops-col-points",
                "ops-col-sorties",
                "ops-col-objectives",
                "ops-col-kills",
                "ops-col-medal",
            ] {
                ui.label(egui::RichText::new(t!(h)).color(DIM));
            }
            ui.end_row();
            for p in &rows {
                let me =
                    c.state().player_name(c.me).is_some_and(|n| n.eq_ignore_ascii_case(&p.name));
                let mut name = egui::RichText::new(&p.name).color(BRIGHT);
                if me {
                    name = name.strong();
                }
                ui.label(name);
                ui.label(rank_of(p.points));
                ui.label(egui::RichText::new(p.points.to_string()).color(BRIGHT));
                ui.label(p.sorties.to_string());
                ui.label(p.achieved.to_string());
                ui.label(format!("{}/{}", p.air_kills, p.ground_kills));
                ui.label(egui::RichText::new(medal_of(p.best).unwrap_or("-")).color(AMBER));
                ui.end_row();
            }
        },
    );
    if rows.is_empty() {
        ui.label(egui::RichText::new(t!("ops-no-sorties")).color(DIM));
    }
}
