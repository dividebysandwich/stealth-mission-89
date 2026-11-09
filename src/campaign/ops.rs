//! The campaign's Ops screen over the flight: mission board, war map, debrief and log, and the
//! prompt after landing at a friendly base. On the desktop an egui overlay over the 3D view,
//! in the headset the menu panel in front of the head (`vr_menu.rs`).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, PrimaryEguiContext, egui};
use stealth_campaign::weather::Now;
use stealth_campaign::{
    Aircraft as Airframe, Due, InsertionState, MissionKind, MissionOffer, Objective, PlayerId,
    Side, SortieResult, tuning,
};
use stealth_host::glue::sortie_mission;
use stealth_mission::Sky;
use stealth_net::protocol::PlayerStatus;
use stealth_sim::station::StationCmd;

use super::{Action, ActiveCampaign, Phase, Role, describe, kind_of, preview};
use crate::layout::{Flow, Span, Split, screen_window};
use crate::map_draw::{
    self, FRIENDLY_BLUE, Layer, MISSILE_RED, MapLayers, MapTips, MapView, MapWeather,
};
use crate::menu::{
    AMBER, BG, BRIGHT, DIM, GREEN, Panel, apply_terminal_style, big_button, footer_rule, icon,
    section,
};
use crate::settings::AppState;
use stealth_i18n::{Text, t};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Board,
    Map,
    AirDefence,
    Logistics,
    Players,
    Debrief,
    Log,
    Chat,
}

/// The Ops screen's own state.
#[derive(Default)]
pub struct OpsUi {
    pub tab: Tab,
    selected: Option<u32>,
    /// Base chosen for the selected mission (None: the offer's).
    base: Option<u16>,
    pub error: Option<Text>,
    pub(super) centre: egui::Vec2,
    pub(super) scale: f32,
    fitted: bool,
    /// The war map's side column is open over the map (on a window too narrow for it beside).
    side_open: bool,
    /// The air defence site selected on the AIR DEFENSE tab (AAA unit), and the list scrolls
    /// to it (picked on the map).
    pub(super) defence: Option<usize>,
    pub(super) defence_scroll: bool,
    /// A new site is being designated on the map (a Patriot battery, a Gepard, a drone command
    /// center), and `--campaign-designate` was done.
    pub(super) designate: Option<stealth_campaign::DefenceKind>,
    pub(super) test_designated: bool,
    /// `--campaign-designate <kind>+built` fast-tracked it.
    pub(super) test_fast_tracked: bool,
    /// The airbase (or the forces) picked on the LOGISTICS tab.
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
    /// The accepted mission's briefing (map and loadout) before take-off.
    briefing: Option<Briefing>,
    /// SAVE & EXIT asks first.
    pub(super) confirm_exit: bool,
    /// The chat message being typed on the CHAT tab.
    chat: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BriefPage {
    Map,
    Loadout,
}

/// A mission accepted from the board, briefed before take-off.
struct Briefing {
    offer: u32,
    base: u16,
    /// The aircraft to fly it in.
    aircraft: Airframe,
    loadout: crate::loadout::PlannedLoadout,
    window: crate::loadout::LoadoutWindow,
    page: BriefPage,
    /// The route to fly (None until the editor first shows it). In a coop game the plan's,
    /// with this player's changes not sent yet.
    route: Option<crate::mission::PlannedRoute>,
    /// Changed here, not sent to the plan yet.
    dirty: bool,
    /// When the last change went (egui time), and until when the plan's route is not taken
    /// over (its echo is on the way).
    sent_at: f64,
    hold_until: f64,
}

impl Briefing {
    fn new(
        offer: u32,
        base: u16,
        aircraft: Airframe,
        loadout: crate::loadout::PlannedLoadout,
        page: BriefPage,
    ) -> Self {
        Self {
            offer,
            base,
            aircraft,
            loadout,
            window: default(),
            page,
            route: None,
            dirty: false,
            sent_at: 0.0,
            hold_until: 0.0,
        }
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

/// Test aids: `--campaign-tab board|map|airdefense|logistics|players|debrief|log|chat` opens a tab, `--campaign-take <n>`
/// takes the n-th mission of the board (0: the first) once the campaign is up.
pub fn test_tab() -> Option<Tab> {
    let args: Vec<String> = std::env::args().collect();
    let t = args.iter().position(|a| a == "--campaign-tab").and_then(|i| args.get(i + 1))?;
    Some(match t.as_str() {
        "map" => Tab::Map,
        "airdefense" => Tab::AirDefence,
        "logistics" | "forces" => Tab::Logistics,
        "debrief" => Tab::Debrief,
        "log" => Tab::Log,
        "players" => Tab::Players,
        "chat" => Tab::Chat,
        _ => Tab::Board,
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

fn test_accept() -> Option<BriefPage> {
    let args: Vec<String> = std::env::args().collect();
    let p = args.iter().position(|a| a == "--campaign-accept").and_then(|i| args.get(i + 1))?;
    Some(if p == "loadout" { BriefPage::Loadout } else { BriefPage::Map })
}

/// The Ops window's size, frame included: as high as the screen lets it be up to this.
const OPS_SIZE: egui::Vec2 = egui::vec2(1232.0, 900.0);
/// The least height of a tab: a lower window scrolls it.
const TAB_LEAST: f32 = 300.0;
/// The height of the BRIEFED button's row under the loadout editor.
const LOADOUT_BUTTONS: f32 = 60.0;
/// The board's width from which the missions and their details are side by side, and the least
/// width of the details.
const BOARD_SPLIT: f32 = 900.0;
/// The war map's width from which its side column is beside it and not a drawer over it.
const SIDE_FROM: f32 = 800.0;
const DETAIL_LEAST: f32 = 320.0;
/// Background of an unselected mission row.
const ROW: egui::Color32 = egui::Color32::from_rgb(9, 29, 21);

/// Map units to km.
fn km(d: i32) -> f32 {
    d as f32 * 15.625 / 1000.0
}

fn dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    stealth_campaign::dist(ax - bx, ay - by)
}

fn kind_colour(k: MissionKind) -> egui::Color32 {
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
    let theater = c.mission.theater_name.clone();
    // Test aid: `--campaign-accept map|loadout` opens the first mission's briefing.
    if let Some(page) = test_accept()
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
        briefing(ctx, c, &combat.sim, &mut planner, line, &map_wx);
        return Ok(());
    }
    super::designate::test_aid(c, &combat.sim, scenery.as_deref());
    let mut leave = false;
    // The big buttons the tab asks the footer for, and the one pressed.
    let mut deck: Vec<Deed> = Vec::new();
    let mut pressed: Option<usize> = None;
    let window = screen_window(ctx, "Ops", OPS_SIZE);
    let server = match (&c.server, c.role) {
        (Some(server), _) => Some(t!("ops-on-server", server = server.to_uppercase())),
        (None, Role::Host) => Some(t!("ops-serving").to_string()),
        _ => None,
    };
    let mut panel = Panel::new(t!("menu-campaign")).note(theater.to_uppercase());
    if let Some(server) = server {
        panel = panel.after(egui::RichText::new(format!("  {server}")).color(FRIENDLY_BLUE));
    }
    panel.window(window, ctx, |ui| {
        // The right side first, the tabs in what is left (on two rows if they don't fit), the
        // weather under them.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            // Top right: the clock and the score, the way out under them.
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(t!("ops-blue", count = c.state().held(Side::Blue)))
                            .color(FRIENDLY_BLUE),
                    );
                    ui.label(
                        egui::RichText::new(t!("ops-red", count = c.state().held(Side::Red)))
                            .color(MISSILE_RED),
                    );
                    ui.label(egui::RichText::new(clock_text(c)).color(BRIGHT));
                });
                // The war simulation's state, under the clock.
                ui.horizontal(|ui| {
                    if c.state().outcome.is_some() {
                        ui.label(egui::RichText::new(t!("ops-war-over")).color(DIM));
                    } else {
                        // The rows fill from the right here: the label, then the spinner to
                        // its left.
                        ui.label(egui::RichText::new(t!("ops-running")).color(DIM));
                        ui.add(egui::Spinner::new().color(GREEN));
                    }
                });
            });
            ui.add_space(16.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                    |ui| {
                        for (tab, label) in [
                            (Tab::Board, t!("ops-tab-missions")),
                            (Tab::Map, t!("ops-tab-map")),
                            (Tab::AirDefence, t!("ops-tab-air-defence")),
                            (Tab::Logistics, t!("ops-tab-logistics")),
                            (Tab::Players, t!("ops-tab-players")),
                            (Tab::Debrief, t!("ops-tab-debrief")),
                            (Tab::Log, t!("ops-tab-log")),
                            (Tab::Chat, t!("ops-tab-chat")),
                        ] {
                            if tab == Tab::Chat && c.role == Role::Local {
                                continue;
                            }
                            if ui.selectable_label(c.ui.tab == tab, label).clicked() {
                                c.ui.tab = tab;
                            }
                        }
                    },
                );
                if let Some((short, _)) = &wx {
                    ui.label(egui::RichText::new(short).color(DIM));
                }
                // The supplies and the front under them.
                ui.horizontal_wrapped(|ui| supply_line(ui, c));
            });
        });
        ui.separator();
        let split = Split::new(ui, "ops");
        split.body_sized(ui, |ui, room| {
            // The tab fills the body: the map, the lists and the briefing take its size.
            let size = egui::vec2(ui.available_width(), room.max(TAB_LEAST));
            ui.allocate_ui(size, |ui| {
                ui.set_min_height(size.y);
                if c.state().outcome.is_some() {
                    end_screen(ui, c);
                    return;
                }
                match c.ui.tab {
                    Tab::Board => board(ui, c, size.x, &mut deck),
                    Tab::Map => war_map(
                        ui,
                        c,
                        &combat.sim,
                        &mut unit_view,
                        &mut planner,
                        &map_wx,
                        scenery.as_deref(),
                        size,
                    ),
                    Tab::AirDefence => super::air_defence::tab(
                        ui,
                        c,
                        &combat.sim,
                        &mut planner,
                        scenery.as_deref(),
                        size,
                    ),
                    Tab::Logistics => super::logistics::tab(ui, c, size),
                    Tab::Players => players(ui, c, &mut voice),
                    Tab::Debrief => debrief(ui, c),
                    Tab::Log => log(ui, c),
                    Tab::Chat => chat(ui, c, &mut notices, settings.vr),
                }
            });
        });
        split.footer(ui, |ui| {
            footer_rule(ui);
            if let Some(e) = &c.ui.error {
                ui.colored_label(AMBER, e.tr());
            }
            ui.add_space(6.0);
            if c.ui.tab == Tab::Map && c.state().outcome.is_none() {
                deck = map_deeds(c, &combat.sim, &unit_view);
            }
            // All in one row where it fits: the way out and OPTIONS to the left, what the tab
            // offers to the right (its main button last). Else two rows.
            let one_row = ui.available_width() >= (2 + deck.len()) as f32 * DEED_WIDTH;
            if one_row {
                ui.horizontal(|ui| {
                    way_out(ui, c, &mut menu);
                    pressed = deed_row(ui, &deck);
                });
            } else {
                if !deck.is_empty() {
                    ui.horizontal(|ui| pressed = deed_row(ui, &deck));
                }
                ui.horizontal(|ui| way_out(ui, c, &mut menu));
            }
        });
    });
    if let Some(i) = pressed {
        do_deed(c, &mut unit_view, deck.swap_remove(i).todo);
    }
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

/// The mission board: the offers, the selected one's details, the insertions under way.
fn board(ui: &mut egui::Ui, c: &mut ActiveCampaign, width: f32, deck: &mut Vec<Deed>) {
    let from = |c: &ActiveCampaign, o: &MissionOffer| {
        c.state().site(o.base).map(|b| (b.x, b.y)).unwrap_or((o.x, o.y))
    };
    if c.ui.selected.is_none_or(|id| !c.state().board.iter().any(|o| o.id == id)) {
        c.ui.selected = c.state().board.first().map(|o| o.id);
        c.ui.base = None;
    }
    // Side by side where there is room, else the details under the missions.
    let room = height_of(ui);
    let left = if width < BOARD_SPLIT { width } else { width * 0.52 };
    let mut flow = Flow::new(ui);
    flow.card(ui, Span::Fixed(left), |ui| {
        section(ui, t!("ops-section-missions"));
        let front = c.state().front(Side::Blue);
        egui::ScrollArea::vertical().id_salt("offers").max_height((room - 40.0).max(120.0)).show(
            ui,
            |ui| {
                for o in c.state().board.clone() {
                    let (bx, by) = from(c, &o);
                    let selected = c.ui.selected == Some(o.id);
                    // Each mission is a framed button row: a marker and the text.
                    // How long an urgent one has left.
                    let due = match c.state().due(&o.objective) {
                        Some(Due::At(t)) => {
                            let mins = ((t - c.state().clock) / 60.0).max(0.0).ceil() as i64;
                            t!("ops-due", mins = mins).to_string()
                        }
                        Some(Due::Now) => t!("ops-due-now").to_string(),
                        None => String::new(),
                    };
                    let main = format!(
                        "{} {:<10} {:<32} {:>4.0} km  {:<9} {}",
                        if selected { '\u{25B6}' } else { ' ' },
                        kind_name(o.kind),
                        o.title.tr(),
                        km(dist(bx, by, o.x, o.y)),
                        if o.threat > 0 {
                            t!("ops-threat", threat = o.threat)
                        } else {
                            String::new()
                        },
                        due
                    );
                    // The box: the mission, who plans and flies it (the AI's flight too), and
                    // the steps of the capture it prepares.
                    let ink = |col| if selected { BG } else { col };
                    let frame = egui::Frame::new()
                        .fill(if selected { kind_colour(o.kind) } else { ROW })
                        .stroke(egui::Stroke::new(1.0, if selected { BRIGHT } else { DIM }))
                        .corner_radius(2.0)
                        .inner_margin(egui::Margin::symmetric(8, 5));
                    let inner = frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 3.0;
                        let text = egui::RichText::new(main.trim_end()).monospace();
                        ui.add(
                            egui::Label::new(text.color(ink(kind_colour(o.kind))))
                                .selectable(false),
                        );
                        let line = crew_line(c, &o);
                        if !line.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                for (part, col) in line {
                                    let text = egui::RichText::new(part).small().color(ink(col));
                                    ui.add(egui::Label::new(text).selectable(false));
                                }
                            });
                        }
                        if let Some((place, steps)) = operation_steps(c, &o, &front) {
                            ui.horizontal(|ui| {
                                let text = t!("ops-operation", place = place);
                                let text = egui::RichText::new(text).small().color(ink(GREEN));
                                ui.add(egui::Label::new(text).selectable(false));
                                task_sequence(ui, &steps, selected.then(|| kind_colour(o.kind)));
                            });
                        }
                    });
                    let resp = ui
                        .interact(
                            inner.response.rect,
                            egui::Id::new(("offer", o.id)),
                            egui::Sense::click(),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.hovered() && !selected {
                        ui.painter().rect_stroke(
                            resp.rect,
                            2.0,
                            egui::Stroke::new(2.0, kind_colour(o.kind)),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if resp.clicked() {
                        c.ui.selected = Some(o.id);
                        c.ui.base = None;
                        c.ui.error = None;
                    }
                    ui.add_space(2.0);
                }
                if c.state().board.is_empty() {
                    ui.label(egui::RichText::new(t!("ops-no-missions")).color(DIM));
                }
            },
        );
    });
    flow.card(ui, Span::Fill(DETAIL_LEAST), |ui| {
        let Some(o) =
            c.ui.selected.and_then(|id| c.state().board.iter().find(|o| o.id == id)).cloned()
        else {
            return;
        };
        section(ui, kind_name(o.kind));
        ui.label(egui::RichText::new(o.title.tr()).size(20.0).color(BRIGHT));
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
            let sams = p
                .defences
                .iter()
                .filter(|&&d| c.state().site(d).is_some_and(|s| s.alive()))
                .count();
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
            _ => {
                c.ui.base.filter(|b| bases.iter().any(|x| x.0 == *b)).or(planned).unwrap_or(o.base)
            }
        };
        let label = bases
            .iter()
            .find(|b| b.0 == chosen)
            .map_or(String::new(), |b| format!("{} ({:.0} km)", b.1, km(b.2)));
        ui.horizontal(|ui| {
            ui.label(t!("ops-from"));
            ui.add_enabled_ui(!taken, |ui| {
                egui::ComboBox::from_id_salt("base").selected_text(label).width(260.0).show_ui(
                    ui,
                    |ui| {
                        for (b, name, d) in &bases {
                            if ui
                                .selectable_label(
                                    *b == chosen,
                                    format!("{name} ({:.0} km)", km(*d)),
                                )
                                .clicked()
                            {
                                c.ui.base = Some(*b);
                            }
                        }
                    },
                );
            });
            if taken {
                ui.label(egui::RichText::new(t!("ops-planned-by-group")).small().color(DIM));
            }
        });
        let sortie = preview(&o, chosen, aircraft);
        let m = sortie_mission(&c.mission, c.state(), &sortie);
        let loadout = crate::loadout::PlannedLoadout::of(&m, kind_of(aircraft));
        let text =
            t!("ops-stores", aircraft = kind_of(aircraft).name(), stores = loadout.summary());
        ui.label(egui::RichText::new(text).color(DIM));
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
    });
    flow.finish(ui);
}

/// Width a big button of the footer takes, spacing included, to tell whether they fit a row.
const DEED_WIDTH: f32 = 200.0;

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
    /// Man an air defence unit's console.
    Man {
        unit: u16,
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

/// The footer's left side: SAVE & EXIT (DISCONNECT from a server) and OPTIONS.
fn way_out(ui: &mut egui::Ui, c: &mut ActiveCampaign, menu: &mut crate::flight_menu::FlightMenu) {
    if ui.add(big_button(exit_label(c.role)).icon(exit_icon(c.role)).fill(EXIT_RED)).clicked() {
        c.ui.confirm_exit = true;
    }
    if ui
        .add(big_button(t!("menu-options")).icon(icon::OPTIONS))
        .on_hover_text(t!("menu-options.short"))
        .clicked()
    {
        menu.open_options = true;
    }
}

/// The footer's right side, from the right: the deeds in reverse, so the last (the main one) is
/// the rightmost. The index of the one pressed.
fn deed_row(ui: &mut egui::Ui, deck: &[Deed]) -> Option<usize> {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let mut pressed = None;
        for (i, d) in deck.iter().enumerate().rev() {
            let mut button = big_button(&d.label).icon(d.icon);
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
        pressed
    })
    .inner
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
            c.ui.briefing = Some(Briefing::new(offer, base, aircraft, loadout, BriefPage::Map));
        }
        Todo::TakeControl { tag, watched } => {
            let loadout = super::control_loadout(c, tag);
            c.actions.push(Action::TakeControl { tag, loadout });
            if watched {
                unit_view.close();
            }
        }
        Todo::Man { unit } => {
            c.actions.push(Action::Station(StationCmd::Man { unit }));
            unit_view.close();
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
fn kind_name(k: MissionKind) -> &'static str {
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
fn supply_line(ui: &mut egui::Ui, c: &ActiveCampaign) {
    let st = c.state();
    ui.label(egui::RichText::new(t!("ops-materiel")).color(DIM));
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

/// The colour of the selection's outline now: black to white and back, once a second.
fn blink(ctx: &egui::Context) -> egui::Color32 {
    ctx.request_repaint();
    let t = ctx.input(|i| i.time) as f32;
    let white = 0.5 - 0.5 * (t * std::f32::consts::TAU).cos();
    egui::Color32::from_gray((white * 255.0).round() as u8)
}

/// The war map: who holds which place, the insertions, the aircraft in the air.
#[allow(clippy::too_many_arguments)]
fn war_map(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    sim: &stealth_sim::Sim,
    unit_view: &mut super::UnitView,
    planner: &mut crate::planner::Planner,
    weather: &MapWeather,
    scenery: Option<&crate::scenery::Scenery>,
    size: egui::Vec2,
) {
    let legend_w = 190.0;
    // On a narrow window the side column is a drawer over the map, not a column beside it.
    let drawer = size.x < SIDE_FROM;
    let map_w = if drawer { size.x } else { size.x - legend_w };
    let off =
        [&[Layer::MissileRange, Layer::EngagementRange][..], &map_draw::WEATHER_LAYERS].concat();
    let mut layers = MapLayers::of(ui.ctx(), "war", &off);
    ui.horizontal_top(|ui| {
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(map_w, size.y), egui::Sense::click_and_drag());
        fit_map(c, rect);
        let view = MapView { rect, centre: c.ui.centre, scale: c.ui.scale };
        let (mut centre, mut scale) = (c.ui.centre, c.ui.scale);
        view.pan_zoom(ui, &resp, &mut centre, &mut scale);
        (c.ui.centre, c.ui.scale) = (centre, scale);
        let painter = ui.painter_at(rect);
        planner.draw_map(&painter, &view, &c.mission.theater_code.to_ascii_lowercase());
        map_draw::weather(&painter, &view, "war", weather, &layers);
        let (parked, selected) = (c.parked, c.ui.selected);
        let test_taken = &mut c.ui.test_taken;
        let st = c.host.as_ref().map(|h| &h.state).or(c.copy.as_ref()).expect("campaign");
        let at = |x: i32, y: i32| view.to_screen(x as f32, y as f32);
        // The war map's symbols twice the size of the other maps'.
        let k = 2.0 * map_draw::BIG;
        let mut tips = MapTips::grown(k);
        // Insertion routes.
        for ins in st.insertions.iter().filter(|_| layers.shows(Layer::Insertions)) {
            if !matches!(ins.state, InsertionState::Pending { .. } | InsertionState::Enroute) {
                continue;
            }
            if let (Some(a), Some(b)) = (st.site(ins.from), st.site(ins.target)) {
                let pts = [at(a.x, a.y), at(b.x, b.y)];
                painter.extend(egui::Shape::dashed_line(
                    &pts,
                    egui::Stroke::new(1.5, FRIENDLY_BLUE),
                    6.0,
                    4.0,
                ));
            }
        }
        // The places.
        for p in st.places.iter().filter(|p| shows_place(&layers, p)) {
            let col = match p.owner {
                Side::Blue => FRIENDLY_BLUE,
                Side::Red => MISSILE_RED,
                Side::Neutral => DIM,
            };
            let c0 = at(p.x, p.y);
            let (owner, airfield) = (affiliation(p.owner), p.airfield.is_some());
            map_draw::scaled(&painter, c0, k, |pa, c| {
                map_draw::place_symbol(pa, c, owner, airfield, true);
            });
            if p.intel(st.clock) {
                let r = map_draw::halo(10.0) * k;
                painter.circle_stroke(c0, r, egui::Stroke::new(1.0, BRIGHT));
            }
            let production = super::logistics::production_of(st, p);
            if let Some(kind) = production {
                super::logistics::production_mark(&painter, c0, k, kind);
            }
            place_tip(&mut tips, c0, p, st.clock, true, production);
            if 1.0 / view.scale > 0.012 || p.owner != Side::Neutral {
                painter.text(
                    c0 + egui::vec2(7.0, -7.0) * k,
                    egui::Align2::LEFT_BOTTOM,
                    &p.name,
                    egui::FontId::monospace(11.0),
                    col,
                );
            }
        }
        // What the friendly side sees: its aircraft, enemy fighters on radar, known defences.
        let pic = super::picture::picture(st, &sim);
        // The watched unit's outline blinks from black to white.
        let watched = unit_view.target.map(|unit| (unit, blink(ui.ctx())));
        super::picture::draw(&painter, &view, &pic, &layers, &mut tips, k, watched);
        // Test aid: `--campaign-watch` watches the first friendly aircraft, `--campaign-watch
        // armour` the first tank of a column on the move.
        let watch = std::env::args().skip_while(|a| a != "--campaign-watch").nth(1);
        if watch.as_deref() == Some("fp2") {
            // The first attack drone in the air.
            if unit_view.target.is_none()
                && let Some(a) = pic.friendly.iter().find(|a| a.spec == stealth_sim::uav::FP2_SPEC)
                && !std::mem::replace(test_taken, true)
            {
                let unit = super::unit_view::UnitRef::Flight { index: a.unit, spec: a.spec };
                unit_view.watch(unit, sim.aircraft_name(a.spec));
                unit_view.test_orbit();
            }
        } else if watch.as_deref() == Some("zsu") {
            // The Shilka of an enemy column on the move.
            let zsu = (0..sim.units().len()).find(|&k| {
                sim.units()[k].kind == stealth_sim::aaa::GroundKind::Zsu
                    && !sim.units()[k].destroyed
                    && !sim.aaa_blue(k)
                    && sim.unit_moving(k)
            });
            if unit_view.target.is_none()
                && let Some(k) = zsu
                && !std::mem::replace(test_taken, true)
            {
                unit_view.watch(super::unit_view::UnitRef::Aaa(k), "ZSU-23-4".into());
                unit_view.test_orbit();
            }
        } else if watch.as_deref() == Some("armour") {
            if unit_view.target.is_none()
                && let Some(v) = pic.armour.iter().find(|v| v.heading.is_some())
                && !std::mem::replace(test_taken, true)
            {
                unit_view.watch(super::unit_view::UnitRef::Aaa(v.index), v.kind.name().into());
            }
        } else if watch.as_deref() == Some("nasams") {
            // A friendly NASAMS launcher (a site the players built).
            let ln = (0..sim.units().len()).find(|&k| {
                sim.units()[k].kind == stealth_sim::aaa::GroundKind::NasamsLauncher
                    && !sim.units()[k].destroyed
                    && sim.aaa_blue(k)
            });
            if unit_view.target.is_none()
                && let Some(k) = ln
                && !std::mem::replace(test_taken, true)
            {
                unit_view.watch(super::unit_view::UnitRef::Aaa(k), "NASAMS".into());
                unit_view.test_orbit();
            }
        } else if std::env::args().any(|a| a == "--campaign-watch")
            && unit_view.target.is_none()
            && let Some(a) = pic.friendly.first()
            && !std::mem::replace(test_taken, true)
        {
            let unit = super::unit_view::UnitRef::Flight { index: a.unit, spec: a.spec };
            unit_view.watch(unit, sim.aircraft_name(a.spec));
        }
        super::designate::waiting(&painter, &view, st);
        let designating = c.ui.designate.is_some();
        if pic.player.is_none()
            && let Some(b) = parked.and_then(|b| st.site(b as u16))
        {
            painter.circle_stroke(at(b.x, b.y), 8.0 * k, egui::Stroke::new(1.5, GREEN));
            tips.add(at(b.x, b.y), t!("mt-parked"), "", GREEN).symbol(|p, c| {
                p.circle_stroke(c, 8.0, egui::Stroke::new(1.5, GREEN));
            });
        }
        // The selected mission's target.
        if let Some(o) = selected.and_then(|id| st.board.iter().find(|o| o.id == id)) {
            let r = 14.0 * k;
            painter.circle_stroke(at(o.x, o.y), r, egui::Stroke::new(2.0, kind_colour(o.kind)));
            let title = crate::mission::clean_text(&o.title.tr());
            let colour = kind_colour(o.kind);
            tips.add(at(o.x, o.y), t!("mt-selected-mission"), title, colour)
                .reach(16.0 * k)
                .line(kind_name(o.kind))
                .symbol(move |p, c| {
                    p.circle_stroke(c, 14.0, egui::Stroke::new(2.0, colour));
                });
        }
        // The layers' checkboxes in a panel over the map, opened by its LAYERS button.
        let panel = crate::planner::layers_panel(ui, rect, &mut layers, |ui, layers| {
            place_legend(ui, layers);
            layers.legend(ui, Layer::Insertions, t!("ops-legend-insertion"), |p, c0| {
                p.extend(egui::Shape::dashed_line(
                    &[c0 - egui::vec2(8.0, 0.0), c0 + egui::vec2(8.0, 0.0)],
                    egui::Stroke::new(1.5, FRIENDLY_BLUE),
                    6.0,
                    4.0,
                ));
            });
            super::picture::legend(ui, layers, Some(&pic));
            map_draw::weather_legend(ui, layers, weather.snow);
        });
        // What is under the pointer (not the LAYERS panel): its tooltip; a click on a unit
        // watches it live.
        let drawer_rect =
            if drawer { side_drawer(ui, rect, size.y, c) } else { egui::Rect::NOTHING };
        if !designating
            && let Some(pos) =
                resp.hover_pos().filter(|&p| !panel.contains(p) && !drawer_rect.contains(p))
        {
            let picked = super::picture::pick(&view, &pic, pos, k);
            tips.show(&resp, picked.as_ref().map(|_| t!("mt-click-watch")));
            if let Some((unit, name)) = picked {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    unit_view.watch(unit, name);
                }
            }
        }
        super::designate::pointer(ui, &painter, &view, &resp, c, sim, scenery);
        // The side column scrolls rather than make the window taller than the map.
        if !drawer {
            ui.vertical(|ui| {
                ui.set_width(legend_w - 12.0);
                egui::ScrollArea::vertical()
                    .id_salt("war_side")
                    .max_height(size.y)
                    .show(ui, |ui| side_column(ui, c));
            });
        }
    });
    layers.keep(ui.ctx(), "war");
}

/// What the war map's side column holds: the symbols, how to use the map, the designations
/// (CONTROL of the watched unit is in the footer).
fn side_column(ui: &mut egui::Ui, c: &mut ActiveCampaign) {
    section(ui, t!("debrief-section-map"));
    map_draw::symbology_switch(ui);
    ui.label(egui::RichText::new(t!("ops-layers-hint")).small().color(DIM));
    ui.add_space(8.0);
    ui.label(egui::RichText::new(t!("ops-watch-hint")).small().color(BRIGHT));
    ui.label(egui::RichText::new(t!("ops-map-hint")).small().color(DIM));
    ui.add_space(10.0);
    super::designate::buttons(ui, c);
}

/// The side column as a drawer over the map's top left, opened by its MAP button: the rect it
/// takes, to keep the map's tooltips from it.
fn side_drawer(
    ui: &mut egui::Ui,
    map: egui::Rect,
    height: f32,
    c: &mut ActiveCampaign,
) -> egui::Rect {
    const WIDTH: f32 = 230.0;
    let area = egui::Rect::from_min_size(
        map.left_top() + egui::vec2(8.0, 8.0),
        egui::vec2(WIDTH, height - 16.0),
    );
    // A child that takes no room in the parent: over the map.
    let ui = &mut ui.new_child(egui::UiBuilder::new().max_rect(area));
    let text = format!("{}  {}", t!("ops-map-tab"), if c.ui.side_open { "^" } else { "v" });
    if ui
        .add(egui::Button::new(egui::RichText::new(text).small()).selected(c.ui.side_open))
        .clicked()
    {
        c.ui.side_open = !c.ui.side_open;
    }
    if c.ui.side_open {
        egui::Frame::new()
            .fill(egui::Color32::from_black_alpha(220))
            .stroke(egui::Stroke::new(1.0, DIM))
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.set_width(WIDTH - 14.0);
                egui::ScrollArea::vertical()
                    .id_salt("war_drawer")
                    .max_height(height - 60.0)
                    .show(ui, |ui| side_column(ui, c));
            });
    }
    ui.min_rect()
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
fn shows_place(layers: &MapLayers, p: &stealth_campaign::Place) -> bool {
    layers.shows(if p.airfield.is_some() { Layer::Airfields } else { Layer::Cities })
}

/// The places' legend lines, with their layers' checkboxes.
fn place_legend(ui: &mut egui::Ui, layers: &mut MapLayers) {
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
fn affiliation(side: Side) -> crate::app6::Affiliation {
    match side {
        Side::Blue => crate::app6::Affiliation::Friend,
        Side::Red => crate::app6::Affiliation::Hostile,
        Side::Neutral => crate::app6::Affiliation::Neutral,
    }
}

/// The war map's view the first time: all the places in `rect`.
pub(super) fn fit_map(c: &mut ActiveCampaign, rect: egui::Rect) {
    if c.ui.fitted {
        return;
    }
    let (lo, hi) = c.state().places.iter().fold(
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

/// The accepted mission's briefing: the route and what is known of the defences on a map, the
/// stores to carry; then take off (or back to the board).
fn briefing(
    ctx: &egui::Context,
    c: &mut ActiveCampaign,
    sim: &stealth_sim::Sim,
    planner: &mut crate::planner::Planner,
    weather: Option<&str>,
    map_wx: &MapWeather,
) {
    let Some(mut b) = c.ui.briefing.take() else { return };
    let Some(o) = c.state().board.iter().find(|o| o.id == b.offer).cloned() else {
        c.ui.error = Some(Text::new("ce-off-board"));
        return;
    };
    // A coop game: the group's plan sets the base.
    let coop = c.role != Role::Local;
    if coop && let Some(p) = c.state().plan(b.offer) {
        b.base = p.base;
    }
    let sortie = preview(&o, b.base, b.aircraft);
    let m = sortie_mission(&c.mission, c.state(), &sortie);
    // What the base has of the weapons that run out.
    let stock: Vec<(u8, u32)> = (0..crate::loadout::WEAPONS.len() as u8)
        .filter_map(|w| {
            let m = stealth_campaign::Munition::of_weapon(w as i32)?;
            let n = c.state().available(b.base, m);
            (n != u32::MAX).then_some((w, n))
        })
        .collect();
    let stock = |w: u8| stock.iter().find(|s| s.0 == w).map(|s| s.1);
    let mut back = false;
    let mut take_off = false;
    let mut edit = None;
    let window = screen_window(ctx, "Briefing", OPS_SIZE);
    Panel::new(t!("ops-briefing"))
        .after(egui::RichText::new(format!("  {}", kind_name(o.kind))).color(kind_colour(o.kind)))
        .note(clock_text(c))
        .window(window, ctx, |ui| {
            ui.label(egui::RichText::new(o.title.tr()).size(20.0).color(BRIGHT));
            ui.label(egui::RichText::new(describe_offer(c, &o)).color(DIM));
            let d = km(dist(m.takeoff.position.x, m.takeoff.position.y, o.x, o.y));
            ui.label(t!(
                "ops-briefing-line",
                base = m.takeoff.name.as_str(),
                km = d.round() as i64,
                aircraft = kind_of(b.aircraft).name(),
                stores = b.loadout.summary()
            ));
            if let Some(line) = weather {
                ui.label(egui::RichText::new(line).color(DIM));
            }
            if coop {
                group_label(ui, c, &o);
                let (_, flying) = group(c, o.id);
                if !flying.is_empty() {
                    let text = t!("ops-route-locked", pilots = flying.join(", "));
                    ui.label(egui::RichText::new(text).color(AMBER));
                }
            }
            ui.horizontal(|ui| {
                for (page, label) in [
                    (BriefPage::Map, t!("ops-route-threats")),
                    (BriefPage::Loadout, t!("menu-loadout")),
                ] {
                    if ui.selectable_label(b.page == page, label).clicked() {
                        b.page = page;
                    }
                }
            });
            ui.separator();
            let split = Split::new(ui, "briefing");
            // What the heading lines above and the buttons below leave.
            split.body_sized(ui, |ui, room| {
                let size = egui::vec2(ui.available_width(), room.max(TAB_LEAST));
                let height = size.y;
                ui.allocate_ui(size, |ui| {
                    ui.set_min_height(height);
                    match b.page {
                        BriefPage::Map => {
                            edit = route_editor(
                                ui,
                                c,
                                sim,
                                planner,
                                &m,
                                &mut b,
                                map_wx,
                                egui::vec2(size.x, height),
                            )
                        }
                        BriefPage::Loadout => {
                            // Wrapped: a row too wide for the window would widen the page.
                            let row = ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new(t!("debrief-aircraft")).color(DIM));
                                let flyable = [
                                    Airframe::F117,
                                    Airframe::F16,
                                    Airframe::F22,
                                    Airframe::Tornado,
                                    Airframe::A10,
                                    Airframe::AV8B,
                                    Airframe::C5,
                                    Airframe::UH60,
                                    Airframe::CH47,
                                    Airframe::AH1Z,
                                    Airframe::RAH66,
                                    Airframe::OH58,
                                ]
                                .into_iter()
                                .filter(|a| o.kind.flyable_by(*a));
                                for a in flyable {
                                    let label = kind_of(a).name();
                                    if ui.selectable_label(b.aircraft == a, label).clicked()
                                        && b.aircraft != a
                                    {
                                        b.aircraft = a;
                                        b.loadout =
                                            crate::loadout::PlannedLoadout::of(&m, kind_of(a));
                                    }
                                }
                            });
                            ui.add_space(4.0);
                            let below = row.response.rect.height() + 4.0 + LOADOUT_BUTTONS;
                            crate::loadout::editor(
                                ui,
                                &mut b.window,
                                &mut b.loadout,
                                &m,
                                (height - below).max(TAB_LEAST - LOADOUT_BUTTONS),
                                &stock,
                            );
                            ui.horizontal(|ui| {
                                if ui
                                    .add(big_button(t!("loadout-briefed")).icon(icon::BRIEFED))
                                    .clicked()
                                {
                                    b.loadout =
                                        crate::loadout::PlannedLoadout::of(&m, kind_of(b.aircraft));
                                }
                            });
                        }
                    }
                });
            });
            split.footer(ui, |ui| {
                footer_rule(ui);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    back = ui.add(big_button(t!("back")).icon(icon::BACK)).clicked();
                    if let Some(e) = &c.ui.error {
                        ui.colored_label(AMBER, e.tr());
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        take_off = ui
                            .add(big_button(t!("ops-take-off")).icon(icon::FLY).primary())
                            .clicked();
                        ui.add(egui::Spinner::new().color(GREEN));
                        ui.label(egui::RichText::new(t!("ops-running")).color(DIM));
                    });
                });
            });
        });
    // This player's changes to the group's route.
    if let Some(route) = edit {
        c.actions.push(Action::EditPlan { offer: o.id, base: b.base, route });
    }
    if take_off {
        if coop
            && std::mem::take(&mut b.dirty)
            && let Some(r) = &b.route
        {
            let route = r.points.clone();
            c.actions.push(Action::EditPlan { offer: o.id, base: b.base, route });
        }
        c.ui.error = None;
        c.aircraft = b.aircraft;
        c.actions.push(Action::TakeMission {
            offer: o.id,
            base: Some(b.base),
            aircraft: b.aircraft,
            loadout: b.loadout.clone(),
            route: b.route.clone(),
        });
        return;
    }
    if !back {
        c.ui.briefing = Some(b);
    } else if coop {
        c.actions.push(Action::SelectMission { offer: None, base: b.base });
    }
}

/// Seconds between a dragged point's updates to the group.
const SEND_EVERY: f64 = 0.1;
/// Seconds the group's route is not taken over after a change of this player's went (until
/// its echo is back).
const HOLD: f64 = 1.0;

/// The route editor (the mission planner's): drag, add and remove waypoints over the radars
/// friendly aircraft have detected (and that still stand), with their detection and missile
/// ranges and the route's exposure; the campaign's places and air picture over the map. In a
/// coop game the route is the group's (locked while one of them flies it): what to send it.
#[allow(clippy::too_many_arguments)]
fn route_editor(
    ui: &mut egui::Ui,
    c: &ActiveCampaign,
    sim: &stealth_sim::Sim,
    planner: &mut crate::planner::Planner,
    m: &stealth_mission::Mission,
    b: &mut Briefing,
    weather: &MapWeather,
    size: egui::Vec2,
) -> Option<Vec<crate::mission::RoutePoint>> {
    let now = ui.input(|i| i.time);
    let plan = c.state().plan(b.offer).filter(|_| c.role != Role::Local);
    let locked = plan.is_some_and(|p| !p.flying.is_empty());
    let route = b.route.get_or_insert_with(|| {
        planner.refit();
        crate::mission::PlannedRoute::of(m)
    });
    // The group's route (the others' changes), unless this player's own are on the way.
    if let Some(p) = plan
        && (locked || (!planner.dragging() && !b.dirty && now >= b.hold_until))
    {
        b.dirty = false;
        if p.route.is_empty() {
            route.points = crate::mission::PlannedRoute::of(m).points;
        } else if route.points != p.route {
            route.points = p.route.clone();
        }
    }
    let before = route.points.clone();
    let st = c.state();
    let known = |i: usize| st.site(i as u16).is_some_and(|x| x.spotted && x.alive());
    // The SAM sites are the editor's own rings.
    let mut pic = super::picture::picture(st, &sim);
    pic.sams.clear();
    let draw = |painter: &egui::Painter, view: &MapView, layers: &MapLayers, tips: &mut MapTips| {
        for p in st.places.iter().filter(|p| shows_place(layers, p)) {
            let col = match p.owner {
                Side::Blue => FRIENDLY_BLUE,
                Side::Red => MISSILE_RED,
                Side::Neutral => DIM,
            };
            let c0 = view.to_screen(p.x as f32, p.y as f32);
            if map_draw::app6() {
                map_draw::app6_place(painter, c0, affiliation(p.owner), p.airfield.is_some(), 0.9);
            } else if p.airfield.is_some() {
                let r = egui::Rect::from_center_size(c0, egui::vec2(9.0, 9.0));
                painter.rect_stroke(r, 0.0, egui::Stroke::new(1.5, col), egui::StrokeKind::Middle);
            } else {
                painter.circle_stroke(c0, 4.5, egui::Stroke::new(1.5, col));
            }
            painter.text(
                c0 + egui::vec2(7.0, 7.0),
                egui::Align2::LEFT_TOP,
                &p.name,
                egui::FontId::monospace(11.0),
                col,
            );
            let production = super::logistics::production_of(st, p);
            if let Some(kind) = production {
                super::logistics::production_mark(painter, c0, 1.0, kind);
            }
            place_tip(tips, c0, p, st.clock, false, production);
        }
        super::picture::draw(painter, view, &pic, layers, tips, 1.0, None);
    };
    let legend = |ui: &mut egui::Ui, layers: &mut MapLayers| {
        place_legend(ui, layers);
        super::picture::legend(ui, layers, None);
    };
    planner.fly(super::kind_of(b.aircraft), true);
    crate::planner::route_panel(
        ui,
        planner,
        m,
        route,
        &known,
        Some(crate::planner::Overlay { draw: &draw, legend: &legend }),
        Some(*weather),
        size,
        !locked,
        |ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(t!("ops-radars-shown")).small().color(AMBER));
        },
    );
    plan?;
    b.dirty |= route.points != before;
    // A dragged point goes a few times a second, everything else at once.
    if b.dirty && (!planner.dragging() || now - b.sent_at >= SEND_EVERY) {
        b.dirty = false;
        b.sent_at = now;
        b.hold_until = now + HOLD;
        return Some(route.points.clone());
    }
    None
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
