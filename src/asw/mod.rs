//! The MH-60R Seahawk's anti-submarine warfare on the client (`stealth_sim::asw` runs the
//! hunt): the sensor operator's cabin console, its main display's pages (SONAR: the sensors'
//! waterfalls, WPN: the torpedo's program and release, MAP: the tactical picture with the
//! route, the buoys, the operator's marks, the tracker's fixes and tracks, the torpedoes), its status
//! screen and keyset; the route the operator edits becoming the pilot's waypoints.
//!
//! Every command goes to the simulation as an [`AswCmd`]: in a single mission straight to the
//! local simulation, in a campaign to its host. The operator is the owner himself at his own
//! console (the AI flying, `gunner::orbit`) or another player in his aircraft.
//!
//! Test aids: `--asw-page sonar|wpn|map`, and `--asw-script "t:cmd,..."` (seconds after the
//! seat is taken): `buoy=difar|dicass|bt[@east,north]` (km from the aircraft), `deep=0|1`,
//! `active=<n>` (the n-th buoy, from 1, pings), `hover=0|1` (the dip hover asked for), `dip=0|1`
//! (at his own console `dip=1` asks for the hover and lowers once there), `depth=<m>`, `dipping=0|1` (its
//! pings), `mad=0|1`, `mark`, `wp=<east>,<north>` (km), `dropwp=<kind>@e,n` (a drop waypoint), `movewp=<i>@e,n`,
//! `delwp=<i>` (waypoint i from 1, the route's then the mission's), `torp=<hdg|auto>/<delay>/<pattern>`,
//! `arm`, `release`, `page=sonar|wpn|map`, `audio`, `sensor=<n>` (the n-th buoy), `smark` (MARK on the SONAR page), `tbuoy=<kind>@e,n` (a buoy put there at once), `markid=<A..H>`, `cursor=<bearing>` (the selected sensor's
//! listening cursor, in the headset).

pub mod audio;
pub mod effects;
pub mod mad;
pub mod map;
pub mod marks;
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

/// The hover the AI brings the aircraft to for a dip (world units over the sea, 40 ft), and how
/// fast it comes down to it once hovering (world units a second, some 2.5 m/s).
pub const DIP_HOVER_WU: f32 = 25.0;
const DIP_DESCENT_WU: f32 = 5.0;

/// Metres per world unit.
pub const M_PER_WU: f64 = stealth_sim::aaa::M_PER_MAP / 32.0;

pub struct AswPlugin;

impl Plugin for AswPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AswUi>()
            .init_resource::<mad::MadTrace>()
            .add_plugins(audio::AswAudioPlugin)
            .add_systems(
            Update,
            (
                (script, input, mad::osb, send)
                    .chain()
                    .after(crate::input::InputSystemsSet)
                    .after(crate::script::ScriptSet)
                    .before(crate::flight::FlightSystems),
                route_sync.after(crate::combat::CombatSystems),
                sub_near.after(crate::combat::CombatSystems),
                dip_pilot.after(send).before(crate::flight::FlightSystems),
                mad::sample.after(crate::combat::CombatSystems),
                effects::splashes.after(crate::combat::CombatSystems),
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
    /// The selected waypoint: the route's points first, then the mission's.
    pub sel_wp: Option<usize>,
    pub drag: Option<Drag>,
    /// The route as edited here (world units east, north), sent with `route_rev`, and the buoy
    /// dropped at each of its points.
    pub route: Vec<(i32, i32)>,
    pub route_rev: u16,
    pub drops: Vec<Option<(BuoyKind, bool)>>,
    /// The mission's waypoints after the route as edited here (the last, the landing, stays),
    /// and when they were last sent (the crew's taken again after a while if it never came).
    pub mission: Vec<(i32, i32)>,
    pub mission_sent: Option<f32>,
    /// When the route was last sent.
    pub route_sent: Option<f32>,
    /// The torpedo station the release takes (1 left inner, 2 right inner).
    pub station: u8,
    /// The sensors' histories.
    pub falls: HashMap<u16, sonar::Waterfall>,
    /// The sensor in the headset.
    pub audio: Option<u16>,
    /// The listening cursor on a waterfall (the sensor, the bearing heard).
    pub cursor: HashMap<u16, f32>,
    /// The marks on the waterfalls, the letter the next one takes, the one selected on the map.
    pub marks: Vec<marks::Mark>,
    pub mark_id: u8,
    pub sel_mark: Option<usize>,
    /// A line shown on the status screen for a while.
    pub notice: Option<(String, f32)>,
    pub outbox: Vec<AswCmd>,
    /// At his own console: the AI to hold a hover (the dip hover asked for, or the sonar in
    /// the water), the height (world units over the sea) it comes down to slowly, and whether it
    /// is there. Test aid (`--asw-script dip=1`): the sonar lowered once there.
    pub low_hover: bool,
    pub hover_height: Option<f32>,
    pub in_hover: bool,
    pub dip_pending: bool,
    /// At the console now (its choices are forgotten when he leaves it).
    pub at_console: bool,
    /// Test aid (`tbuoy`): buoys to put in the water at once.
    pub test_buoys: Vec<(BuoyKind, [f64; 2])>,
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
            drops: Vec::new(),
            mission: Vec::new(),
            mission_sent: None,
            route_sent: None,
            station: 1,
            falls: HashMap::new(),
            audio: None,
            cursor: HashMap::new(),
            marks: Vec::new(),
            mark_id: 0,
            sel_mark: None,
            notice: None,
            outbox: Vec::new(),
            dip_pending: false,
            low_hover: false,
            hover_height: None,
            in_hover: false,
            at_console: false,
            test_buoys: Vec::new(),
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
        // Up from the console: its choices forgotten.
        if ui.at_console {
            *ui = AswUi::default();
        }
        return;
    };
    ui.at_console = true;
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
    follow_crew(&mut ui, &crew);
    for ConsoleButton(k) in keys.read() {
        key(&mut ui, *k, &crew, sim, owner);
    }
    for d in drags.read().filter(|d| d.screen == ConsoleScreen::Asw) {
        let p = d.uv * CANVAS;
        touch(&mut ui, p, d.phase, &crew, sim, owner);
    }
}

/// Waiting this long (s) for an edit to come back from the simulation (a campaign's host), the
/// crew's own taken again (it was refused).
const EDIT_WAIT: f32 = 3.0;

/// The route and the mission's waypoints as the crew has them, unless edited here since (sent
/// and not back yet, or being dragged).
fn follow_crew(ui: &mut AswUi, crew: &AswCrew) {
    if ui.drag.is_some() {
        return;
    }
    let waiting = |sent: Option<f32>| sent.is_some_and(|t| ui.t - t < EDIT_WAIT);
    // Ahead of the crew's revision: sent, not back yet.
    let ahead = (ui.route_rev.wrapping_sub(crew.route_rev) as i16) > 0;
    if !(ahead && waiting(ui.route_sent)) {
        ui.route_sent = None;
        ui.route = crew.route.clone();
        ui.route_rev = crew.route_rev;
        ui.drops = drops_of(crew);
    }
    if crew.mission == ui.mission || !waiting(ui.mission_sent) {
        ui.mission_sent = None;
        ui.mission = crew.mission.clone();
    }
    let n = ui.route.len() + ui.mission.len();
    if ui.sel_wp.is_some_and(|i| i >= n) {
        ui.sel_wp = None;
    }
}

/// The buoy waiting at each of the crew's route points.
pub fn drops_of(crew: &AswCrew) -> Vec<Option<(BuoyKind, bool)>> {
    crew.route
        .iter()
        .map(|&p| {
            let at = stealth_sim::asw::wu_metres(p);
            crew.pending
                .iter()
                .find(|d| (d.2[0] - at[0]).abs() < 1.0 && (d.2[1] - at[1]).abs() < 1.0)
                .map(|d| (d.0, d.1))
        })
        .collect()
}

/// The route's drops as the command has them: the point's index, kind, deep.
pub fn route_drops(drops: &[Option<(BuoyKind, bool)>]) -> Vec<(u8, BuoyKind, bool)> {
    drops.iter().enumerate().filter_map(|(i, d)| d.map(|(k, deep)| (i as u8, k, deep))).collect()
}

/// The route as edited here to the simulation.
pub fn send_route(ui: &mut AswUi) {
    ui.drops.resize(ui.route.len(), None);
    ui.route_rev = ui.route_rev.wrapping_add(1);
    let (rev, points, drops) = (ui.route_rev, ui.route.clone(), route_drops(&ui.drops));
    ui.route_sent = Some(ui.t);
    ui.outbox.push(AswCmd::Route { rev, points, drops });
}

/// The mission's waypoints as edited here to the simulation.
pub fn send_mission(ui: &mut AswUi) {
    ui.mission_sent = Some(ui.t);
    let points = ui.mission.clone();
    ui.outbox.push(AswCmd::Mission { points });
}

/// A key of the keyset.
pub fn key(ui: &mut AswUi, k: ConsoleKey, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    match k {
        ConsoleKey::AswPage(n) => {
            ui.page = Page::of(n);
            ui.drop = None;
        }
        // The keyset drops at once, where the aircraft is (a point on the map: the MAP page's
        // own drop buttons, then a touch on the map).
        ConsoleKey::Buoy(n) => {
            let Some(kind) = BuoyKind::from_key(n) else { return };
            ui.drop = None;
            ui.outbox.push(AswCmd::DropBuoy { kind, deep: ui.deep, at: None });
        }
        ConsoleKey::Ping => ping(ui, crew, sim, owner),
        // The sonar down or up: from the dip hover (HOVER asks the pilot for it).
        ConsoleKey::Dip => {
            let down = matches!(crew.dip.state, DipState::Lowering | DipState::Down);
            ui.dip_pending = false;
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
        // On the SONAR page: mark the trace under the cursor (or unmark it); elsewhere a fix
        // where the aircraft is.
        ConsoleKey::Mark if ui.page == Page::Sonar => mark_signal(ui, sim, crew, owner),
        ConsoleKey::Mark => ui.outbox.push(AswCmd::Mark),
        ConsoleKey::RangeIn => ui.range_km = (ui.range_km / 2.0).max(1.0),
        ConsoleKey::RangeOut => ui.range_km = (ui.range_km * 2.0).min(64.0),
        ConsoleKey::Arm => ui.outbox.push(AswCmd::Arm(!crew.armed)),
        ConsoleKey::Release => release(ui, sim, owner),
        _ => {}
    }
}

/// MARK on the SONAR page: the trace under the cursor marked with the letter chosen (replacing
/// that letter's mark on this sensor); a mark there already is deleted.
pub fn mark_signal(ui: &mut AswUi, sim: &stealth_sim::Sim, crew: &AswCrew, owner: u8) {
    let Some((&sensor, &brg)) = ui.cursor.iter().next() else {
        ui.note("PUT THE CURSOR ON A TRACE");
        return;
    };
    if let Some(i) = ui
        .marks
        .iter()
        .position(|m| m.sensor == sensor && sonar::bearing_off(m.bearing, brg).abs() <= marks::FOLLOW)
    {
        let m = ui.marks.remove(i);
        ui.sel_mark = None;
        ui.note(format!("MARK {} DELETED", m.letter()));
        return;
    }
    let sample = match sensor {
        0 => crew.dip.sample.clone(),
        id => match sim.asw().buoys_of(owner).find(|b| b.id == id) {
            Some(b) => b.sample.clone(),
            None => return,
        },
    };
    // The trace under the cursor, else an echo heard there lately (it went by in a moment).
    let echo = || {
        let w = ui.falls.get(&sensor)?;
        w.echoes
            .iter()
            .rev()
            .map(|&(_, b, snr)| (b as f32, snr))
            .find(|&(b, _)| sonar::bearing_off(b, brg).abs() <= marks::FOLLOW)
    };
    let Some((bearing, snr)) = marks::trace_near(&sample, brg).map(|l| (l.bearing.unwrap_or_default() as f32, l.snr)).or_else(echo)
    else {
        ui.note("NO SIGNAL AT THE CURSOR");
        return;
    };
    let (id, t) = (ui.mark_id, ui.t);
    ui.marks.retain(|m| !(m.sensor == sensor && m.id == id));
    ui.marks.push(marks::Mark { id, sensor, bearing, snr, seen: t });
    ui.note(format!("MARK {} ON {}", marks::letter(id), if sensor == 0 { "DIP".to_string() } else { format!("B{sensor}") }));
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
        Page::Sonar => sonar::click(ui, p, phase, crew, sim, owner),
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
    if ui.outbox.is_empty() && ui.test_buoys.is_empty() {
        return;
    }
    let Some(mut combat) = combat else {
        ui.outbox.clear();
        return;
    };
    let me = combat.sim.active_pilot();
    for (kind, at) in std::mem::take(&mut ui.test_buoys) {
        let deep = ui.deep;
        let _ = combat.sim.test_buoy_at(me, kind, deep, at);
    }
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

/// What the pilot's waypoints were last built from: the crew's route revision, its mission
/// points and drops, the mission's own waypoints and their anchors as the aircraft first had
/// them, and the mission point flown to (counting from the first after the route).
#[derive(Default)]
pub struct RouteSeen {
    built: Option<(u16, Vec<(i32, i32)>, Vec<Option<(BuoyKind, bool)>>)>,
    base: Option<(Vec<crate::hud::Waypoint>, Vec<usize>)>,
    mission_at: usize,
    /// The revision the passed route points were trimmed at.
    trimmed: Option<u16>,
}

/// The mission's waypoints as the crew starts with them: in world units, a point repeated
/// (the primary as the secondary, a hunt's datum) once, with where each was.
fn mission_points(base: &[crate::hud::Waypoint]) -> (Vec<(i32, i32)>, Vec<usize>) {
    let mut points: Vec<(i32, i32)> = Vec::new();
    let mut index = Vec::new();
    for w in base {
        let p = (w.east as i32, w.north as i32);
        if points.last() != Some(&p) {
            points.push(p);
        }
        index.push(points.len() - 1);
    }
    (points, index)
}

/// The pilot's waypoints follow the crew's route: its points first, then the mission's (as the
/// operator may have moved them; the landing stays last). The route's points are dropped from
/// it once passed, so an edit does not send the aircraft back to them.
fn route_sync(
    combat: Option<Res<Combat>>,
    settings: Res<crate::settings::Settings>,
    manning: Option<Res<Manning>>,
    mut wps: Option<ResMut<crate::hud::Waypoints>>,
    mut ui: ResMut<AswUi>,
    mut seen: Local<RouteSeen>,
) {
    let (Some(combat), Some(wps)) = (combat, wps.as_deref_mut()) else { return };
    // A remote operator's own waypoints are his parked aircraft's: not the crew's.
    if !settings.aircraft.seahawk() || manning.as_deref().is_some_and(|m| m.remote_gunner()) {
        return;
    }
    let (base, anchors) =
        seen.base.get_or_insert_with(|| (wps.points.clone(), wps.anchors.clone())).clone();
    let (start, index) = mission_points(&base);
    let me = combat.sim.active_pilot();
    let queued = ui.outbox.iter().any(|c| matches!(c, AswCmd::Mission { .. }));
    let Some(crew) = combat.sim.asw().crew(me) else {
        // The crew begins with the first command; the mission's route goes with it.
        if !start.is_empty() && ui.outbox.is_empty() {
            ui.outbox.push(AswCmd::Mission { points: start });
        }
        return;
    };
    if crew.mission.is_empty() && !start.is_empty() && !queued {
        ui.outbox.push(AswCmd::Mission { points: start.clone() });
    }
    let mission = if crew.mission.is_empty() { start.clone() } else { crew.mission.clone() };
    let drops = drops_of(crew);
    let key = (crew.route_rev, mission.clone(), drops.clone());
    if seen.built.as_ref() == Some(&key) {
        // The route's points passed: off the route (their drops are gone by then), once the
        // waypoints are built from the crew's route as it is (not one trimmed already).
        let tactical = wps.tactical.min(crew.route.len());
        let passed =
            (0..tactical).take_while(|&i| wps.passed.get(i).copied().unwrap_or(false)).count();
        if passed > 0 && seen.trimmed != Some(crew.route_rev) {
            seen.trimmed = Some(crew.route_rev);
            let mut drops = drops_of(crew);
            drops.drain(..passed);
            let rev = crew.route_rev.wrapping_add(1);
            let points = crew.route[passed..].to_vec();
            ui.outbox.push(AswCmd::Route { rev, points, drops: route_drops(&drops) });
        }
        return;
    }
    // The mission point flown to, kept through the rebuild.
    if seen.built.is_none() {
        seen.mission_at = index.get(wps.current).copied().unwrap_or(0);
    } else if wps.current >= wps.tactical {
        seen.mission_at = wps.current - wps.tactical;
    }
    seen.built = Some(key);
    let n = crew.route.len();
    let mut points: Vec<crate::hud::Waypoint> = crew
        .route
        .iter()
        .chain(&mission)
        .map(|&(e, n)| crate::hud::Waypoint { east: e as f64, north: n as f64 })
        .collect();
    if points.is_empty() {
        points = base.clone();
    }
    // The mission's anchors (midpoint, primary, secondary, landing) on its points as they are:
    // where they were, or within the points left once the operator moved them, the landing last.
    let own: Vec<usize> =
        if anchors.is_empty() { (0..base.len()).collect() } else { anchors.clone() };
    let last = mission.len().saturating_sub(1);
    let mut anchors: Vec<usize> =
        own.iter().map(|&k| index.get(k).copied().unwrap_or(k).min(last)).collect();
    if mission != start
        && let Some(land) = anchors.get_mut(3)
    {
        *land = last;
    }
    let anchors: Vec<usize> = anchors.into_iter().map(|k| k + n).collect();
    let current = if n > 0 { 0 } else { (n + seen.mission_at).min(points.len().saturating_sub(1)) };
    let editing = wps.editing;
    *wps = crate::hud::Waypoints::planned(points, anchors);
    wps.current = current;
    wps.mission_current = current;
    wps.editing = editing;
    wps.tactical = n;
    wps.drop_points =
        drops.iter().enumerate().filter(|(_, d)| d.is_some()).map(|(i, _)| i).collect();
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
    mut dumped: Local<f32>,
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
    let page_name = match ui.page {
        Page::Sonar => "asw_sonar",
        Page::Wpn => "asw_wpn",
        Page::Map => "asw_map",
    };
    let cv = &mut tex.canvas;
    let names = [(Page::Sonar, "SONAR"), (Page::Wpn, "WPN"), (Page::Map, "MAP")];
    for (k, (p, name)) in names.into_iter().enumerate() {
        let x = 160.0 + k as f32 * 160.0;
        let col = if ui.page == p { scope::YELLOW } else { scope::GREY };
        stroke_text_at(cv, x, 472.0, name, 9.0, 1.2, col, Align::Centre);
    }
    if std::env::var_os("SM89_ASW_PNG").is_some() {
        *dumped += 0.1;
        if *dumped >= 1.0 {
            *dumped = 0.0;
            dump_png(&tex.canvas, tex.backdrop.as_deref(), page_name);
        }
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
    } else if ui.in_hover {
        "AI (IN DIP HOVER)".into()
    } else if let Some(h) = ui.hover_height.filter(|_| ui.low_hover) {
        format!("AI (HOVER {:.0} FT)", h as f64 * M_PER_WU / 0.3048)
    } else {
        "AI (ROUTE)".into()
    };
    line(cv, 1, &format!("PILOT {pilot}"), GREEN);
    if crew.hover_req {
        let there = owner_view(sim, owner).is_some_and(|v| {
            (v.height as f64 * M_PER_WU) < 70.0 * 0.3048 && v.knots < 5
        });
        line(cv, 8, if there { "IN THE DIP HOVER" } else { "DIP HOVER REQUESTED" }, if there { GREEN } else { YELLOW });
    }
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

/// At his own console: the AI flying comes to a hover where it is when the operator asks for
/// the dip hover, then down slowly to 40 ft over the sea, and holds it while the sonar is in
/// the water. The operator lowers and raises the sonar.
fn dip_pilot(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    k: Res<crate::flight::FlightConstants>,
    time: Res<Time>,
    mut ui: ResMut<AswUi>,
    player: Query<&crate::flight::Aircraft, With<crate::flight::Player>>,
) {
    let own = manning.as_deref().is_some_and(|m| m.asw() && !m.remote_gunner());
    let (Some(combat), true) = (combat, own) else {
        ui.low_hover = false;
        ui.dip_pending = false;
        ui.hover_height = None;
        ui.in_hover = false;
        return;
    };
    let me = combat.sim.active_pilot();
    let crew = combat.sim.asw().crew(me).cloned().unwrap_or_default();
    let wet = !matches!(crew.dip.state, DipState::Up | DipState::Broken);
    ui.low_hover = crew.hover_req || wet || ui.dip_pending;
    let Ok(a) = player.single() else { return };
    if !ui.low_hover || k.heli.is_none() {
        ui.hover_height = None;
        ui.in_hover = false;
        return;
    }
    let agl = a.altitude - a.ground_altitude;
    let hovering = a.auto_hover() && a.knots() < 4.0;
    // At the height it was, until it hovers; then down slowly.
    let h = ui.hover_height.get_or_insert(agl.max(DIP_HOVER_WU));
    if a.auto_hover() {
        *h = (*h - DIP_DESCENT_WU * time.delta_secs()).max(DIP_HOVER_WU);
    }
    let there = hovering && (agl - DIP_HOVER_WU).abs() < 8.0;
    if there && !ui.in_hover && crew.hover_req && !wet {
        ui.note("IN THE DIP HOVER: LOWER THE SONAR");
    }
    ui.in_hover = there;
    if ui.dip_pending && there {
        ui.dip_pending = false;
        ui.outbox.push(AswCmd::Dip(true));
    }
}

/// Test aid `--asw-sub-near <km>[,ahead[,<s>]]`: the first submarine moved that far north-east
/// of the aircraft (or on its heading) once the flight is under way (2 s, or `s`).
fn sub_near(combat: Option<ResMut<Combat>>, time: Res<Time>, mut done: Local<bool>) {
    let Some(mut combat) = combat else { return };
    if *done {
        return;
    }
    let arg = std::env::args().skip_while(|a| a != "--asw-sub-near").nth(1);
    let Some((km, ahead, at)) = arg.as_deref().and_then(|a| {
        let mut f = a.split(',');
        let km = f.next()?.parse::<f64>().ok()?;
        let ahead = f.next() == Some("ahead");
        let at = f.next().and_then(|t| t.parse::<f32>().ok()).unwrap_or(2.0);
        Some((km, ahead, at))
    }) else {
        *done = true;
        return;
    };
    if time.elapsed_secs() < at {
        return;
    }
    *done = true;
    let me = combat.sim.active_pilot();
    if let Some((p, heading)) = owner_at(&combat.sim, me) {
        // North-east of the aircraft, or (`,ahead`) on its heading.
        let b = if ahead { heading } else { std::f64::consts::FRAC_PI_4 };
        let d = km * 1000.0;
        combat.sim.place_sub(0, [p[0] + b.sin() * d, p[1] + b.cos() * d]);
    }
}

/// `--asw-page`, `--asw-script` (see the module).
fn script(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    settings: Res<crate::settings::Settings>,
    time: Res<Time>,
    mut ui: ResMut<AswUi>,
    mut started: Local<bool>,
) {
    let Some(combat) = combat else { return };
    // At the console, or (for the outside views) the pilot of a Seahawk.
    let Some((owner, console)) = crew_owner(manning.as_deref(), &combat, &settings) else { return };
    if !console {
        ui.t += time.delta_secs();
    }
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
            // Test aid: a buoy put straight in the water at a point (a single mission's).
            "tbuoy" => {
                let (kind, at) = val.split_once('@').unwrap_or((val, ""));
                let kind = match kind {
                    "dicass" => BuoyKind::Dicass,
                    "bt" => BuoyKind::Bt,
                    _ => BuoyKind::Difar,
                };
                if let Some(p) = km(at) {
                    ui.test_buoys.push((kind, p));
                }
            }
            "deep" => ui.deep = on,
            "active" => {
                let n: usize = val.parse().unwrap_or(1);
                if let Some(b) = sim.asw().buoys_of(owner).nth(n.saturating_sub(1)) {
                    ui.outbox.push(AswCmd::BuoyMode { id: b.id, active: true });
                    ui.sensor = Some(b.id);
                }
            }
            // At his own console: the dip hover asked for, the sonar lowered once there.
            "dip" if on && sim.active_pilot() == owner => {
                ui.dip_pending = true;
                ui.outbox.push(AswCmd::Hover(true));
            }
            "dip" => ui.outbox.push(AswCmd::Dip(on)),
            "hover" => ui.outbox.push(AswCmd::Hover(on)),
            "depth" => ui.outbox.push(AswCmd::DipDepth(val.parse().unwrap_or(60))),
            "dipping" => ui.outbox.push(AswCmd::DipMode { active: on }),
            "mad" => ui.outbox.push(AswCmd::Mad(on)),
            "mark" => ui.outbox.push(AswCmd::Mark),
            "wp" => {
                if let Some(p) = km(val) {
                    ui.route.push(((p[0] / M_PER_WU) as i32, (p[1] / M_PER_WU) as i32));
                    send_route(&mut ui);
                }
            }
            // As the MAP page's D/C/B and a touch: a drop waypoint at the route's end.
            "dropwp" => {
                let (kind, at) = val.split_once('@').unwrap_or((val, ""));
                let kind = match kind {
                    "dicass" => BuoyKind::Dicass,
                    "bt" => BuoyKind::Bt,
                    _ => BuoyKind::Difar,
                };
                if let Some(p) = km(at) {
                    ui.route.push(stealth_sim::asw::wu_point(p));
                    let n = ui.route.len();
                    ui.drops.resize(n - 1, None);
                    let deep = ui.deep;
                    ui.drops.push(Some((kind, deep)));
                    send_route(&mut ui);
                }
            }
            // Waypoint i (from 1: the route's, then the mission's) moved to a point, or deleted.
            "movewp" | "delwp" => {
                let (i, at) = val.split_once('@').unwrap_or((val, ""));
                let i = i.parse::<usize>().unwrap_or(1).saturating_sub(1);
                let n = ui.route.len();
                let at = km(at).map(stealth_sim::asw::wu_point);
                if i < n {
                    match at {
                        Some(p) => ui.route[i] = p,
                        None => {
                            ui.route.remove(i);
                            ui.drops.resize(n, None);
                            ui.drops.remove(i);
                        }
                    }
                    send_route(&mut ui);
                } else if i - n + 1 < ui.mission.len() {
                    match at {
                        Some(p) => ui.mission[i - n] = p,
                        None => {
                            ui.mission.remove(i - n);
                        }
                    }
                    send_mission(&mut ui);
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
            "smark" => mark_signal(&mut ui, sim, &crew, owner),
            "range" => ui.range_km = val.parse().unwrap_or(8.0),
            "markid" => ui.mark_id = val.bytes().next().map_or(0, |c| c.to_ascii_uppercase().saturating_sub(b'A')) % marks::IDS as u8,
            "sensor" => {
                let n: usize = val.parse().unwrap_or(1);
                ui.sensor = sim.asw().buoys_of(owner).nth(n.saturating_sub(1)).map(|b| b.id);
            }
            "cursor" => {
                ui.sensor = ui.sensor.or_else(|| sim.asw().buoys_of(owner).next().map(|b| b.id));
                if let Some(id) = ui.sensor {
                    // `cursor=sub`: on the submarine's trace (a test's peek at the truth).
                    let sub = || {
                        let sample = match id {
                            0 => crew.dip.sample.clone(),
                            id => sim.asw().buoys_of(owner).find(|b| b.id == id)?.sample.clone(),
                        };
                        let l = sample.lines.into_iter().find(|l| !l.ship && !l.ping && !l.torpedo)?;
                        l.bearing.map(|b| b as f32)
                    };
                    let brg = if val == "sub" { sub().unwrap_or(0.0) } else { val.parse().unwrap_or(0.0) };
                    ui.cursor.clear();
                    ui.cursor.insert(id, brg);
                    ui.audio = Some(id);
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

/// Test aid `SM89_ASW_PNG=<dir>`: a console canvas (and the picture under it) written as a PNG,
/// in the console's colours (the manual's screenshots, sharp as the screen is drawn).
pub fn dump_png(c: &crate::canvas::Canvas, under: Option<&crate::canvas::Picture>, name: &str) {
    let Ok(dir) = std::env::var("SM89_ASW_PNG") else { return };
    let rgb = |i: u8| -> [u8; 3] {
        match i {
            scope::DARK => [60, 66, 60],
            scope::GREY => [150, 155, 150],
            scope::GREEN => [60, 230, 80],
            scope::CYAN => [70, 220, 235],
            scope::RED => [235, 50, 45],
            scope::WHITE => [240, 240, 240],
            scope::YELLOW => [240, 225, 60],
            146 | 147 => [235, 30, 30],
            _ => [0, 0, 0],
        }
    };
    let (w, h) = (c.pw(), c.ph());
    let mut px = Vec::with_capacity(w * h * 3);
    for (k, &i) in c.px.iter().enumerate() {
        match (i, under) {
            (0, Some(p)) if p.w == w => px.extend_from_slice(&p.rgba[k * 4..k * 4 + 3]),
            _ => px.extend_from_slice(&rgb(i)),
        }
    }
    if let Some(img) = image::RgbImage::from_raw(w as u32, h as u32, px) {
        let _ = img.save(format!("{dir}/{name}.png"));
    }
}
