//! The MH-60R Seahawk's anti-submarine warfare on the client (`stealth_sim::asw` runs the
//! hunt): the sensor operator's cabin console, its main display's pages (SONAR: the sensors'
//! waterfalls, WPN: the torpedo's program and release, MAP: the tactical picture with the
//! route, the buoys' bearings, the tracker's fixes and tracks, the torpedoes), its status
//! screen and keyset; the route the operator edits becoming the pilot's waypoints.
//!
//! Every command goes to the simulation as an [`AswCmd`]: in a single mission straight to the
//! local simulation, in a campaign to its host. The operator is the owner himself at his own
//! console (the AI flying, `gunner::orbit`) or another player in his aircraft.
//!
//! Test aids: `--asw-page sonar|wpn|map`, and `--asw-script "t:cmd,..."` (seconds after the
//! seat is taken): `buoy=difar|dicass|bt[@east,north]` (km from the aircraft), `deep=0|1`,
//! `active=<n>` (the n-th buoy, from 1, pings), `dip=0|1`, `depth=<m>`, `dipping=0|1` (its
//! pings), `mad=0|1`, `mark`, `wp=<east>,<north>` (km), `torp=<hdg|auto>/<delay>/<pattern>`,
//! `arm`, `release`, `page=sonar|wpn|map`, `audio`.

pub mod audio;
pub mod map;
pub mod sonar;
pub mod wpn;

use std::collections::HashMap;

use bevy::prelude::*;
use stealth_sim::asw::{AswCmd, AswCrew, BuoyKind, DipState, Pattern, TorpProgram};

use crate::cockpit3d::{ConsoleButton, ConsoleCanvas, ConsolePanel, ConsoleScreen, DragPhase, ScreenDrag};
use crate::cockpit3d::layout::ConsoleKey;
use crate::combat::Combat;
use crate::font::{Align, stroke_text_at};
use crate::station::Manning;
use crate::settings::AppState;

/// The main display's canvas (`cockpit3d::displays::ASW_CANVAS`).
const CANVAS: Vec2 = Vec2::new(640.0, 480.0);

/// The RELEASE key's light: palette dark red (safe, and the armed flash's off phase), bright red.
const RELEASE_DIM: u8 = 222;
const RELEASE_BRIGHT: u8 = 147;

/// Metres per world unit.
pub const M_PER_WU: f64 = stealth_sim::aaa::M_PER_MAP / 32.0;

pub struct AswPlugin;

impl Plugin for AswPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AswUi>().add_plugins(audio::AswAudioPlugin).add_systems(
            Update,
            (
                (script, input, send)
                    .chain()
                    .after(crate::input::InputSystemsSet)
                    .after(crate::script::ScriptSet)
                    .before(crate::flight::FlightSystems),
                route_sync.after(crate::combat::CombatSystems),
                sub_near.after(crate::combat::CombatSystems),
                (sonar::record, draw_main, draw_info, panel).chain().after(crate::combat::CombatSystems),
            )
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// The main display's pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Sonar,
    Wpn,
    Map,
}

impl Page {
    fn of(n: u8) -> Self {
        match n {
            1 => Page::Wpn,
            2 => Page::Map,
            _ => Page::Sonar,
        }
    }
}

/// A press held on the map: a waypoint moved, or the map panned (from where it started, the
/// centre then).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Drag {
    Wp(usize),
    Pan { from: Vec2, centre: [f64; 2], moved: bool },
}

/// The console's own choices (the simulation has the rest).
#[derive(Resource)]
pub struct AswUi {
    pub page: Page,
    /// The selected sensor: a buoy's id, 0 the dipping sonar.
    pub sensor: Option<u16>,
    /// New buoys' hydrophones deep.
    pub deep: bool,
    /// A buoy kind waiting for its point on the map.
    pub drop: Option<BuoyKind>,
    /// The map's centre (m east, north; None: on the aircraft) and range (km, the half height).
    pub centre: Option<[f64; 2]>,
    pub range_km: f64,
    pub sel_wp: Option<usize>,
    pub drag: Option<Drag>,
    /// The route as edited here (world units east, north), sent with `route_rev`.
    pub route: Vec<(i32, i32)>,
    pub route_rev: u16,
    /// The torpedo station the release takes (1 left inner, 2 right inner).
    pub station: u8,
    /// The sensors' histories.
    pub falls: HashMap<u16, sonar::Waterfall>,
    /// The sensor in the headset.
    pub audio: Option<u16>,
    /// A line shown on the status screen for a while.
    pub notice: Option<(String, f32)>,
    pub outbox: Vec<AswCmd>,
    /// Test aid: seconds at the console, the script's next step.
    pub t: f32,
    pub step: usize,
}

impl Default for AswUi {
    fn default() -> Self {
        AswUi {
            page: Page::Sonar,
            sensor: None,
            deep: true,
            drop: None,
            centre: None,
            range_km: 8.0,
            sel_wp: None,
            drag: None,
            route: Vec::new(),
            route_rev: 0,
            station: 1,
            falls: HashMap::new(),
            audio: None,
            notice: None,
            outbox: Vec::new(),
            t: 0.0,
            step: 0,
        }
    }
}

impl AswUi {
    pub fn note(&mut self, s: impl Into<String>) {
        self.notice = Some((s.into(), 5.0));
    }
}

/// The Seahawk crew the player belongs to: the owner, and whether he sits at the console.
pub fn crew_owner(
    manning: Option<&Manning>,
    combat: &Combat,
    settings: &crate::settings::Settings,
) -> Option<(u8, bool)> {
    match manning {
        Some(m) if m.asw() => Some((m.unit as u8, true)),
        _ if settings.aircraft.seahawk() => Some((combat.sim.active_pilot(), false)),
        _ => None,
    }
}

/// The owner's aircraft as the simulation sees it.
pub fn owner_view(sim: &stealth_sim::Sim, owner: u8) -> Option<stealth_sim::PlayerView> {
    if sim.active_pilot() == owner {
        Some(sim.player())
    } else {
        sim.remote_pilots().into_iter().find(|p| p.id == owner).map(|p| p.view)
    }
}

/// The owner's position (m east, north) and heading (rad).
pub fn owner_at(sim: &stealth_sim::Sim, owner: u8) -> Option<([f64; 2], f64)> {
    owner_view(sim, owner).map(|v| {
        (
            [v.wx as f64 * M_PER_WU, v.wy as f64 * M_PER_WU],
            (v.heading as u16) as f64 / 65536.0 * std::f64::consts::TAU,
        )
    })
}

/// The owner's stores (weapon type, rounds) by station: his own, or as his crew is told.
pub fn owner_stores(sim: &stealth_sim::Sim, owner: u8) -> Vec<(i32, i32)> {
    if sim.active_pilot() == owner {
        return sim.bays().0.to_vec();
    }
    sim.crew_view(sim.active_pilot())
        .map(|v| v.stores.iter().map(|&(w, n)| (w as i32, n as i32)).collect())
        .unwrap_or_default()
}

/// The keyset and the main display's touches.
#[allow(clippy::too_many_arguments)]
fn input(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut ui: ResMut<AswUi>,
    mut keys: MessageReader<ConsoleButton>,
    mut drags: MessageReader<ScreenDrag>,
    time: Res<Time>,
) {
    let (Some(combat), Some(m)) = (combat, manning.as_deref().filter(|m| m.asw())) else {
        keys.clear();
        drags.clear();
        if ui.t != 0.0 {
            *ui = AswUi::default();
        }
        return;
    };
    let owner = m.unit as u8;
    let sim = &combat.sim;
    let crew = sim.asw().crew(owner).cloned().unwrap_or_default();
    ui.t += time.delta_secs();
    if let Some((_, t)) = ui.notice.as_mut() {
        *t -= time.delta_secs();
        if *t <= 0.0 {
            ui.notice = None;
        }
    }
    // The route as the simulation has it, unless edited here since.
    if crew.route_rev != ui.route_rev && ui.drag.is_none() {
        ui.route = crew.route.clone();
        ui.route_rev = crew.route_rev;
    }
    for ConsoleButton(k) in keys.read() {
        key(&mut ui, *k, &crew, sim, owner);
    }
    for d in drags.read().filter(|d| d.screen == ConsoleScreen::Asw) {
        let p = d.uv * CANVAS;
        touch(&mut ui, p, d.phase, &crew, sim, owner);
    }
}

/// A key of the keyset.
pub fn key(ui: &mut AswUi, k: ConsoleKey, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    match k {
        ConsoleKey::AswPage(n) => {
            ui.page = Page::of(n);
            ui.drop = None;
        }
        ConsoleKey::Buoy(n) => {
            let Some(kind) = BuoyKind::from_key(n) else { return };
            if ui.page == Page::Map {
                ui.drop = if ui.drop == Some(kind) { None } else { Some(kind) };
            } else {
                ui.outbox.push(AswCmd::DropBuoy { kind, deep: ui.deep, at: None });
            }
        }
        ConsoleKey::Ping => ping(ui, crew, sim, owner),
        ConsoleKey::Dip => {
            let down = matches!(crew.dip.state, DipState::Lowering | DipState::Down);
            ui.outbox.push(AswCmd::Dip(!down));
            if !down {
                ui.sensor = Some(0);
            }
        }
        ConsoleKey::Depth(n) => {
            let set = if crew.dip.set > 0.0 { crew.dip.set } else { 60.0 };
            let step = if n == 1 { -10.0 } else { 10.0 };
            ui.outbox.push(AswCmd::DipDepth((set + step).clamp(10.0, stealth_sim::asw::DIP_MAX) as u16));
        }
        ConsoleKey::Audio => ui.audio = if ui.audio.is_some() { None } else { ui.sensor },
        ConsoleKey::Mad => ui.outbox.push(AswCmd::Mad(!crew.mad_down)),
        ConsoleKey::Mark => ui.outbox.push(AswCmd::Mark),
        ConsoleKey::RangeIn => ui.range_km = (ui.range_km / 2.0).max(1.0),
        ConsoleKey::RangeOut => ui.range_km = (ui.range_km * 2.0).min(64.0),
        ConsoleKey::Arm => ui.outbox.push(AswCmd::Arm(!crew.armed)),
        ConsoleKey::Release => release(ui, sim, owner),
        _ => {}
    }
}

/// PING: the selected DICASS or the dipping sonar between pinging and listening.
fn ping(ui: &mut AswUi, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    match ui.sensor {
        Some(0) => ui.outbox.push(AswCmd::DipMode { active: !crew.dip.active }),
        Some(id) => match sim.asw().buoys_of(owner).find(|b| b.id == id) {
            Some(b) if b.kind == BuoyKind::Dicass => {
                ui.outbox.push(AswCmd::BuoyMode { id, active: !b.active });
            }
            _ => ui.note("SELECT A DICASS OR THE DIP"),
        },
        None => ui.note("SELECT A DICASS OR THE DIP"),
    }
}

/// RELEASE: the torpedo on the chosen station (else the other one).
fn release(ui: &mut AswUi, sim: &stealth_sim::Sim, owner: u8) {
    let stores = owner_stores(sim, owner);
    let has = |s: u8| stores.get(s as usize).is_some_and(|&(w, n)| w == stealth_sim::asw::MK54_WEAPON && n > 0);
    let station = [ui.station, 3 - ui.station].into_iter().find(|&s| has(s));
    match station {
        Some(s) => ui.outbox.push(AswCmd::Release { station: s }),
        None => ui.note("NO TORPEDO"),
    }
}

/// A touch on the main display (canvas units).
fn touch(ui: &mut AswUi, p: Vec2, phase: DragPhase, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    match ui.page {
        Page::Sonar => {
            if phase == DragPhase::Start {
                sonar::click(ui, p, crew, sim, owner);
            }
        }
        Page::Wpn => {
            if phase == DragPhase::Start {
                wpn::click(ui, p, crew, sim, owner);
            }
        }
        Page::Map => map::touch(ui, p, phase, crew, sim, owner),
    }
}

/// The commands to the simulation: a single mission's own, a campaign's host's.
fn send(
    mut ui: ResMut<AswUi>,
    combat: Option<ResMut<Combat>>,
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    mut msgs: ResMut<crate::hud::HudMessages>,
) {
    if ui.outbox.is_empty() {
        return;
    }
    let Some(mut combat) = combat else {
        ui.outbox.clear();
        return;
    };
    let me = combat.sim.active_pilot();
    for cmd in std::mem::take(&mut ui.outbox) {
        match campaign.as_deref_mut() {
            Some(c) => c.actions.push(crate::campaign::Action::Asw(cmd)),
            None => {
                if let Err(t) = combat.sim.asw_command(me, cmd) {
                    let s = t.tr_flight();
                    msgs.show(s.clone());
                    ui.note(s.to_ascii_uppercase());
                }
            }
        }
    }
}

/// The pilot's waypoints follow the crew's route: its points first, then the mission's.
fn route_sync(
    combat: Option<Res<Combat>>,
    settings: Res<crate::settings::Settings>,
    mut wps: Option<ResMut<crate::hud::Waypoints>>,
    mut seen: Local<Option<(u16, Vec<crate::hud::Waypoint>, Vec<usize>)>>,
) {
    let (Some(combat), Some(wps)) = (combat, wps.as_deref_mut()) else { return };
    if !settings.aircraft.seahawk() {
        return;
    }
    let me = combat.sim.active_pilot();
    let Some(crew) = combat.sim.asw().crew(me) else { return };
    let (rev, base, anchors) = seen.get_or_insert_with(|| (0, wps.points.clone(), wps.anchors.clone()));
    if crew.route_rev == *rev {
        return;
    }
    *rev = crew.route_rev;
    let n = crew.route.len();
    let mut points: Vec<crate::hud::Waypoint> = crew
        .route
        .iter()
        .map(|&(e, n)| crate::hud::Waypoint { east: e as f64, north: n as f64 })
        .collect();
    points.extend(base.iter().copied());
    let anchors: Vec<usize> = if anchors.is_empty() {
        (0..base.len()).map(|k| k + n).collect()
    } else {
        anchors.iter().map(|k| k + n).collect()
    };
    let current = if n > 0 { 0 } else { wps.current.min(points.len().saturating_sub(1)) };
    let editing = wps.editing;
    *wps = crate::hud::Waypoints::planned(points, anchors);
    wps.current = current;
    wps.editing = editing;
}

/// The main display.
#[allow(clippy::too_many_arguments)]
fn draw_main(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    mut ui: ResMut<AswUi>,
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTextureRef)>,
) {
    let (Some(combat), Some(m)) = (combat, manning.as_deref().filter(|m| m.asw())) else { return };
    *since += time.delta_secs();
    if *since < 0.1 {
        return;
    }
    *since = 0.0;
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Asw) else { return };
    let owner = m.unit as u8;
    let sim = &combat.sim;
    let crew = sim.asw().crew(owner).cloned().unwrap_or_default();
    tex.dirty = true;
    // Opaque: the waterfalls' picture shows where the canvas is index 0.
    tex.transparent = false;
    let hunt = mission.as_deref().and_then(|m| m.0.hunt.clone());
    match ui.page {
        Page::Sonar => {
            let prev = tex.backdrop.take();
            let pic = sonar::draw(&mut tex.canvas, &mut ui, &crew, sim, owner, prev);
            tex.backdrop = pic;
        }
        Page::Wpn => {
            tex.backdrop = None;
            wpn::draw(&mut tex.canvas, &ui, &crew, sim, owner);
        }
        Page::Map => {
            tex.backdrop = None;
            map::draw(&mut tex.canvas, &ui, &crew, sim, owner, hunt.as_ref());
        }
    }
    // The page keys' row along the bottom.
    let cv = &mut tex.canvas;
    let names = [(Page::Sonar, "SONAR"), (Page::Wpn, "WPN"), (Page::Map, "MAP")];
    for (k, (p, name)) in names.into_iter().enumerate() {
        let x = 160.0 + k as f32 * 160.0;
        let col = if ui.page == p { scope::YELLOW } else { scope::GREY };
        stroke_text_at(cv, x, 472.0, name, 9.0, 1.2, col, Align::Centre);
    }
}

type CanvasTextureRef = crate::canvas::CanvasTexture;

use crate::station::scope;

/// The status screen: who flies, the aircraft's state, the sensors and stores, the notice.
fn draw_info(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<AswUi>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTextureRef)>,
) {
    let (Some(combat), Some(m)) = (combat, manning.as_deref().filter(|m| m.asw())) else { return };
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Info) else { return };
    use scope::{BLACK, CYAN, GREEN, RED, YELLOW};
    let owner = m.unit as u8;
    let sim = &combat.sim;
    let crew = sim.asw().crew(owner).cloned().unwrap_or_default();
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let (w, h) = (cv.w as i32, cv.h as i32);
    cv.rect(1, 1, w - 2, h - 2, CYAN);
    let line = |cv: &mut crate::canvas::Canvas, row: i32, s: &str, col: u8| {
        stroke_text_at(cv, 10.0, 16.0 + row as f32 * 19.0, s, 8.0, 1.2, col, Align::Left);
    };
    line(cv, 0, "MH-60R SENSOR OPERATOR", YELLOW);
    let pilot = if m.remote_gunner() {
        campaign
            .as_deref()
            .and_then(|c| c.state().player_name(stealth_campaign::PlayerId(owner)))
            .unwrap_or("PILOT")
            .to_ascii_uppercase()
    } else if matches!(crew.dip.state, DipState::Lowering | DipState::Down) {
        "AI (DIP HOVER)".into()
    } else {
        "AI (ROUTE)".into()
    };
    line(cv, 1, &format!("PILOT {pilot}"), GREEN);
    if let Some(v) = owner_view(sim, owner) {
        line(cv, 2, &format!("ALT {:>5.0} FT  SPD {:>3} KT", v.height as f64 * M_PER_WU / 0.3048, v.knots), GREEN);
    }
    let dip = match crew.dip.state {
        DipState::Up => "UP".to_string(),
        DipState::Broken => "LOST".to_string(),
        s => format!(
            "{} {:.0}/{:.0} M {}",
            if s == DipState::Lowering { "LWR" } else if s == DipState::Raising { "RAISE" } else { "DOWN" },
            crew.dip.depth,
            crew.dip.set.max(10.0),
            if crew.dip.active { "ACT" } else { "PAS" }
        ),
    };
    line(cv, 3, &format!("DIP {dip}"), if crew.dip.state == DipState::Broken { RED } else { GREEN });
    line(
        cv,
        4,
        &format!("MAD {} {:>4.1} NT", if crew.mad_down { "OUT" } else { "IN " }, crew.mad as f32 / 10.0),
        if crew.mad > 30 { YELLOW } else { GREEN },
    );
    let live = sim.asw().buoys_of(owner).count();
    line(
        cv,
        5,
        &format!(
            "BUOYS {live}/4  D{} C{} B{}",
            crew.buoys_left[0], crew.buoys_left[1], crew.buoys_left[2]
        ),
        GREEN,
    );
    let stores = owner_stores(sim, owner);
    let torps: i32 = stores.iter().filter(|s| s.0 == stealth_sim::asw::MK54_WEAPON).map(|s| s.1).sum();
    line(
        cv,
        6,
        &format!("MK54 {torps}  {}", if crew.armed { "ARMED" } else { "SAFE" }),
        if crew.armed { RED } else { GREEN },
    );
    line(
        cv,
        7,
        &match crew.layer {
            Some(l) => format!("LAYER {l} M"),
            None => "LAYER ?".to_string(),
        },
        CYAN,
    );
    if let Some((n, _)) = &ui.notice {
        line(cv, 9, n, YELLOW);
    }
    let _ = h;
}

/// The keyset's lights.
fn panel(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    ui: Res<AswUi>,
    time: Res<Time<Real>>,
    mut out: ResMut<ConsolePanel>,
) {
    let (Some(combat), Some(m)) = (combat, manning.as_deref().filter(|m| m.asw())) else { return };
    let crew = combat.sim.asw().crew(m.unit as u8).cloned().unwrap_or_default();
    use scope::{GREEN, RED, YELLOW};
    let mut lit = vec![(ConsoleKey::AswPage(ui.page as u8), GREEN)];
    if let Some(k) = ui.drop {
        lit.push((ConsoleKey::Buoy(k.index() as u8 + 1), YELLOW));
    }
    if crew.mad_down {
        lit.push((ConsoleKey::Mad, GREEN));
    }
    if crew.dip.wet() || crew.dip.state == DipState::Lowering {
        lit.push((ConsoleKey::Dip, GREEN));
    }
    if crew.dip.active || ui.sensor.is_some_and(|id| combat.sim.asw().buoys.iter().any(|b| b.id == id && b.active)) {
        lit.push((ConsoleKey::Ping, YELLOW));
    }
    if ui.audio.is_some() {
        lit.push((ConsoleKey::Audio, GREEN));
    }
    // RELEASE: a red key, dim while safe, flashing bright while armed.
    if crew.armed {
        lit.push((ConsoleKey::Arm, RED));
        let on = (time.elapsed_secs() * 2.0).fract() < 0.5;
        lit.push((ConsoleKey::Release, if on { RELEASE_BRIGHT } else { RELEASE_DIM }));
    } else {
        lit.push((ConsoleKey::Release, RELEASE_DIM));
    }
    let want = ConsolePanel { hidden: Vec::new(), lit };
    if *out != want {
        *out = want;
    }
}

/// Test aid `--asw-sub-near <km>`: the first submarine moved that far north-east of the
/// aircraft once the flight is under way (2 s).
fn sub_near(combat: Option<ResMut<Combat>>, time: Res<Time>, mut done: Local<bool>) {
    let Some(mut combat) = combat else { return };
    if *done || time.elapsed_secs() < 2.0 {
        return;
    }
    let Some(km) = std::env::args().skip_while(|a| a != "--asw-sub-near").nth(1).and_then(|s| s.parse::<f64>().ok())
    else {
        *done = true;
        return;
    };
    *done = true;
    let me = combat.sim.active_pilot();
    if let Some((p, _)) = owner_at(&combat.sim, me) {
        let d = km * 1000.0 / std::f64::consts::SQRT_2;
        combat.sim.place_sub(0, [p[0] + d, p[1] + d]);
    }
}

/// `--asw-page`, `--asw-script` (see the module).
fn script(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut ui: ResMut<AswUi>,
    mut started: Local<bool>,
) {
    let (Some(combat), Some(m)) = (combat, manning.as_deref().filter(|m| m.asw())) else { return };
    let arg = |name: &str| std::env::args().skip_while(|a| a != name).nth(1);
    if !*started {
        *started = true;
        if let Some(p) = arg("--asw-page") {
            ui.page = match p.as_str() {
                "wpn" => Page::Wpn,
                "map" => Page::Map,
                _ => Page::Sonar,
            };
        }
    }
    let Some(script) = arg("--asw-script") else { return };
    // Steps whose argument holds a comma (`wp=1,2`, `buoy=difar@1,2`) are joined back.
    let mut joined: Vec<(f32, String)> = Vec::new();
    for part in script.split(',') {
        match part.split_once(':').and_then(|(t, c)| Some((t.parse::<f32>().ok()?, c))) {
            Some((t, c)) => joined.push((t, c.to_string())),
            None => {
                if let Some(last) = joined.last_mut() {
                    last.1.push(',');
                    last.1.push_str(part);
                }
            }
        }
    }
    let owner = m.unit as u8;
    let sim = &combat.sim;
    let crew = sim.asw().crew(owner).cloned().unwrap_or_default();
    let here = owner_at(sim, owner).map(|p| p.0).unwrap_or_default();
    let km = |s: &str| -> Option<[f64; 2]> {
        let (e, n) = s.split_once(',')?;
        Some([here[0] + e.trim().parse::<f64>().ok()? * 1000.0, here[1] + n.trim().parse::<f64>().ok()? * 1000.0])
    };
    while let Some((t, c)) = joined.get(ui.step).cloned() {
        if ui.t < t {
            break;
        }
        ui.step += 1;
        let (name, val) = c.split_once('=').unwrap_or((c.as_str(), ""));
        let on = val != "0";
        match name {
            "buoy" => {
                let (kind, at) = val.split_once('@').unwrap_or((val, ""));
                let kind = match kind {
                    "dicass" => BuoyKind::Dicass,
                    "bt" => BuoyKind::Bt,
                    _ => BuoyKind::Difar,
                };
                let at = km(at).map(|p| (p[0] as i32, p[1] as i32));
                let deep = ui.deep;
                ui.outbox.push(AswCmd::DropBuoy { kind, deep, at });
            }
            "deep" => ui.deep = on,
            "active" => {
                let n: usize = val.parse().unwrap_or(1);
                if let Some(b) = sim.asw().buoys_of(owner).nth(n.saturating_sub(1)) {
                    ui.outbox.push(AswCmd::BuoyMode { id: b.id, active: true });
                    ui.sensor = Some(b.id);
                }
            }
            "dip" => ui.outbox.push(AswCmd::Dip(on)),
            "depth" => ui.outbox.push(AswCmd::DipDepth(val.parse().unwrap_or(60))),
            "dipping" => ui.outbox.push(AswCmd::DipMode { active: on }),
            "mad" => ui.outbox.push(AswCmd::Mad(on)),
            "mark" => ui.outbox.push(AswCmd::Mark),
            "wp" => {
                if let Some(p) = km(val) {
                    ui.route.push(((p[0] / M_PER_WU) as i32, (p[1] / M_PER_WU) as i32));
                    let rev = ui.route_rev.wrapping_add(1);
                    ui.route_rev = rev;
                    let points = ui.route.clone();
                    ui.outbox.push(AswCmd::Route { rev, points });
                }
            }
            "torp" => {
                let mut f = val.split('/');
                let heading = f.next().and_then(|h| h.parse::<u16>().ok());
                let delay = f.next().and_then(|d| d.parse().ok()).unwrap_or(crew.program.delay);
                let pattern = match f.next() {
                    Some("straight") => Pattern::Straight,
                    Some("snake") => Pattern::Snake,
                    Some("helix") => Pattern::Helix,
                    _ => Pattern::Circle,
                };
                ui.outbox.push(AswCmd::Program(TorpProgram { heading, delay, pattern, ..crew.program }));
            }
            "arm" => ui.outbox.push(AswCmd::Arm(true)),
            "release" => release(&mut ui, sim, owner),
            "page" => {
                ui.page = match val {
                    "wpn" => Page::Wpn,
                    "map" => Page::Map,
                    _ => Page::Sonar,
                }
            }
            "audio" => {
                ui.sensor = ui.sensor.or_else(|| sim.asw().buoys_of(owner).next().map(|b| b.id));
                ui.audio = ui.sensor;
            }
            _ => warn!("--asw-script: unknown step {c}"),
        }
    }
}
