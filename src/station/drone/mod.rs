//! The drone command center's console (`cockpit_drone.glb`): the player flies up to five drones
//! (`stealth_sim::uav`) from the bunker.
//!
//! * The ground control station (`map`, the wide screen on the desk): the terrain, the friendly
//!   units and the known contacts (those the drones found among them), the drones with the
//!   selected one's route, a list of the drones beside it, and the on-screen buttons. Drag to
//!   pan (or the left VR stick), the zoom buttons (or the right VR stick up and down); click a
//!   drone or its row to select it, drag its waypoints (or move the selected one with the left
//!   VR stick), give a waypoint another type, make it the next one. NEW DRONE plans a launch:
//!   the type, the waypoints (a click on a contact puts the route's end on it: an attack
//!   drone's terminate point, a surveillance drone's loiter point), then LAUNCH. ALT+ and ALT-
//!   set the selected waypoint's height. The map shows the aircraft and drones in the air with
//!   their heights, to put an interceptor where it will meet one.
//! * The selected drone's camera (`feed`): in the map's picture in picture, on the main screen
//!   with MAIN (one camera, `trackcam`: one place at a time). Its TGP: the stick slews it over
//!   the ground (TRACK holds it on a point, SLAVE puts it back on the next waypoint), brackets
//!   on the enemy air defences the drone sees. Flown by hand (MAN, the map's MANUAL): the main
//!   screen shows the view ahead with a flight display, the stick and throttle fly it. An
//!   interceptor has only the view ahead: it marks the aircraft in front and the one it locked
//!   on to (the drone then steers itself, the throttle closes in).
//! * The drone list (`list`, the screen top right): each drone's battery.

mod feed;
mod list;
mod map;

use bevy::math::DVec2;
use bevy::prelude::*;
use stealth_sim::aaa::M_PER_MAP;
use stealth_sim::station::StationCmd;
use stealth_sim::uav::{MAX_WAYPOINTS, Uav, UavKind, UavMode, UavWp, WpKind, route_refusal};

use super::{Manning, PipShown};
use crate::campaign::{Action, ActiveCampaign};
use crate::cockpit3d::{
    ConsoleButton, ConsoleKey, ConsolePanel, ConsoleScreen, DragPhase, ScreenDrag,
};
use crate::combat::Combat;
use crate::flight::FlightSystems;
use crate::settings::AppState;

pub(super) fn build(app: &mut App) {
    app.init_resource::<DroneUi>().init_resource::<PipShown>().add_systems(
        Update,
        (
            reset.after(super::sync_manning),
            map::detail,
            stick.after(super::hand_controller).before(FlightSystems),
            (input, test_aid, feed::aim, map::draw, feed::draw, list::draw, panel)
                .chain()
                .after(reset),
        )
            .run_if(in_state(AppState::Flight)),
    );
}

/// The map's zoom steps: kilometres across its picture.
pub const ZOOMS_KM: [f32; 8] = [4.0, 8.0, 15.0, 30.0, 60.0, 120.0, 240.0, 480.0];
/// A press that moves less than this (canvas pixels) is a click.
const CLICK: f32 = 4.0;
/// The left VR stick pans this fast (map pixels a second), the camera stick slews this much of
/// the picture a second.
const STICK_PAN: f32 = 160.0;
const SLEW: f32 = 0.6;
/// The right VR stick steps the zoom this often while held.
const ZOOM_REPEAT: f32 = 0.35;
/// Seconds between the stick's commands to a drone flown by hand.
const FLY_PERIOD: f32 = 0.06;
/// The keyboard's throttle keys move it this fast (of full, a second).
const THROTTLE_KEYS: f32 = 0.5;
/// The stick a drone is flown by eases to where it is held this fast (of full, a second): the
/// arrow keys roll and pitch it in and out smoothly.
const STICK_EASE: f32 = 6.0;
/// ALT+ and ALT- move a waypoint this much (m), within these heights.
const ALT_STEP: f32 = 100.0;
const ALT_RANGE: (f32, f32) = (100.0, 5000.0);

/// A drone being planned: its type and route.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub kind: UavKind,
    pub route: Vec<UavWp>,
}

/// A drone's camera: the ground point it holds (None: slaved to the next waypoint), the point
/// it looks at now (map units), its zoom step; looking ahead (the forward FLIR, to fly by)
/// instead of a TGP.
#[derive(Debug, Clone, Copy)]
pub struct Cam {
    pub track: Option<DVec2>,
    pub look: Option<DVec2>,
    pub zoom: u8,
    pub forward: bool,
}

impl Default for Cam {
    fn default() -> Self {
        Self { track: None, look: None, zoom: crate::mfd::CAM_ZOOM_DEFAULT, forward: false }
    }
}

/// A press held on the map: panning it, dragging a waypoint, or not moved yet.
#[derive(Debug, Clone, Copy)]
enum Drag {
    Press { at: Vec2 },
    Pan { at: Vec2, centre: DVec2 },
    Wp { index: usize },
}

/// What the console shows and the player is doing (the drones themselves are the simulation's).
#[derive(Resource)]
pub struct DroneUi {
    /// The selected drone (its id at the center).
    pub selected: Option<u16>,
    pub plan: Option<Plan>,
    /// The selected waypoint (of the plan, else of the selected drone).
    pub wp: Option<usize>,
    /// The map's centre (map units, x east, y south) and zoom step (`ZOOMS_KM`).
    pub centre: DVec2,
    pub zoom: usize,
    /// The camera is on the main screen (else in the map's picture in picture).
    pub on_main: bool,
    drag: Option<Drag>,
    /// A drone's route as edited here and not yet sent.
    pub edit: Option<(u16, Vec<UavWp>)>,
    pub cams: Vec<(u16, Cam)>,
    /// The throttle (0..1) for a drone flown by hand.
    pub throttle: f32,
    fly: (f32, [f32; 3]),
    /// The stick (roll, pitch) as eased to where it is held.
    eased: [f32; 2],
    /// A message (a refused command) and how long it shows yet (s).
    pub note: Option<(String, f32)>,
    /// The map was put on the center, the right stick's zoom repeat, a waypoint moved by the
    /// left stick waits to be sent.
    placed: bool,
    zoom_wait: f32,
    stick_moved: bool,
    /// `--drone-launch`: drones launched, and those selected.
    tested: u8,
    test_picked: Vec<u16>,
}

impl Default for DroneUi {
    fn default() -> Self {
        Self {
            selected: None,
            plan: None,
            wp: None,
            centre: DVec2::splat(0x4000 as f64),
            zoom: 3,
            on_main: false,
            drag: None,
            edit: None,
            cams: Vec::new(),
            throttle: 0.5,
            fly: (0.0, [0.0; 3]),
            eased: [0.0; 2],
            note: None,
            placed: false,
            zoom_wait: 0.0,
            stick_moved: false,
            tested: 0,
            test_picked: Vec::new(),
        }
    }
}

impl DroneUi {
    /// Map units a canvas pixel of the map spans.
    pub fn scale(&self) -> f64 {
        ZOOMS_KM[self.zoom] as f64 * 1000.0 / M_PER_MAP / map::MAP_W as f64
    }

    pub fn cam(&self, id: u16) -> Cam {
        self.cams.iter().find(|c| c.0 == id).map_or_else(Cam::default, |c| c.1)
    }

    pub fn cam_mut(&mut self, id: u16) -> &mut Cam {
        if let Some(i) = self.cams.iter().position(|c| c.0 == id) {
            return &mut self.cams[i].1;
        }
        self.cams.push((id, Cam::default()));
        &mut self.cams.last_mut().expect("pushed").1
    }

    /// The route shown and edited: the plan's, else the selected drone's (as edited here).
    pub fn route<'a>(&'a self, uavs: &'a [Uav]) -> Option<&'a [UavWp]> {
        if let Some(p) = &self.plan {
            return Some(&p.route);
        }
        let id = self.selected?;
        if let Some((e, r)) = &self.edit
            && *e == id
        {
            return Some(r);
        }
        uavs.iter().find(|u| u.id == id).map(|u| u.route.as_slice())
    }

    fn say(&mut self, text: impl Into<String>) {
        self.note = Some((text.into(), 4.0));
    }

    fn zoom_by(&mut self, d: i32) {
        self.zoom = (self.zoom as i32 + d).clamp(0, ZOOMS_KM.len() as i32 - 1) as usize;
    }
}

/// The drones of the manned center in the air (at most five).
pub fn center_uavs(combat: &Combat, m: &Manning) -> Vec<Uav> {
    combat.sim.uavs_of(m.unit).cloned().collect()
}

/// A fresh console for each manning of a drone command center.
fn reset(
    manning: Option<Res<Manning>>,
    mut ui: ResMut<DroneUi>,
    mut pip: ResMut<PipShown>,
    mut was: Local<Option<usize>>,
) {
    let now = manning.as_deref().filter(|m| m.drones()).map(|m| m.unit);
    if now != *was {
        *ui = DroneUi::default();
        *was = now;
    }
    if now.is_none() && pip.0 {
        pip.0 = false;
    }
}

/// Drone `u`'s camera looks ahead (an interceptor's always does).
pub fn forward(ui: &DroneUi, u: &Uav) -> bool {
    ui.cam(u.id).forward || !u.kind.tgp()
}

/// The selected drone (if it is still in the air).
fn selected<'a>(ui: &DroneUi, uavs: &'a [Uav]) -> Option<&'a Uav> {
    let id = ui.selected?;
    uavs.iter().find(|u| u.id == id)
}

/// The stick and the throttle: a drone flown by hand follows them, else the stick slews the
/// selected drone's camera over the ground.
fn stick(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut c: Option<ResMut<ActiveCampaign>>,
    controls: Res<crate::input::FlightControls>,
    mut ui: ResMut<DroneUi>,
    time: Res<Time<Real>>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c.as_deref_mut()) else { return };
    if !m.drones() {
        return;
    }
    let dt = time.delta_secs();
    if let Some(t) = controls.throttle {
        ui.throttle = t.clamp(0.0, 1.0);
    }
    ui.throttle = (ui.throttle + controls.console_throttle * THROTTLE_KEYS * dt).clamp(0.0, 1.0);
    let uavs = center_uavs(&combat, &m);
    let Some(u) = selected(&ui, &uavs) else { return };
    let dead = |v: f32| if v.abs() < 0.04 { 0.0 } else { v };
    let (roll, pitch) = (dead(controls.roll), dead(controls.pitch));
    let forward = forward(&ui, u);
    if u.mode == UavMode::Manual {
        let step = STICK_EASE * dt;
        for (e, v) in ui.eased.iter_mut().zip([roll, pitch]) {
            *e += (v - *e).clamp(-step, step);
        }
        let [roll, pitch] = ui.eased;
        ui.fly.0 -= dt;
        let want = [roll, pitch, ui.throttle];
        // Let go, exactly 0 goes out (a sliver of a rate left would creep on).
        let changed =
            want.iter().zip(ui.fly.1).any(|(&a, b)| (a - b).abs() > 0.01 || (a == 0.0 && b != 0.0));
        if changed && ui.fly.0 <= 0.0 {
            ui.fly = (FLY_PERIOD, want);
            c.actions.push(Action::Station(StationCmd::Fly {
                uav: u.id,
                roll,
                pitch,
                throttle: ui.throttle,
            }));
        }
        return;
    }
    ui.eased = [0.0; 2];
    // The TGP: the stick moves the point it holds across the picture (finer zoomed in).
    if (roll == 0.0 && pitch == 0.0) || forward {
        return;
    }
    let id = u.id;
    let cam = ui.cam(id);
    let Some(look) = cam.track.or(cam.look) else { return };
    let (x, y) = u.map();
    let eye = DVec2::new(x as f64, y as f64);
    let ahead = (look - eye).try_normalize().unwrap_or(DVec2::new(0.0, -1.0));
    let right = DVec2::new(-ahead.y, ahead.x);
    let range = (look - eye).length().max(20.0) + u.pos[2] / M_PER_MAP;
    let k = range * SLEW as f64 / crate::mfd::cam_magnification(cam.zoom) as f64 * dt as f64;
    let moved = look + (right * roll as f64 + ahead * pitch as f64) * k;
    ui.cam_mut(id).track = Some(moved);
}

/// The buttons, the screens' presses and drags, the VR sticks on the map.
#[allow(clippy::too_many_arguments)]
fn input(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut c: Option<ResMut<ActiveCampaign>>,
    mut ui: ResMut<DroneUi>,
    mut keys: MessageReader<ConsoleButton>,
    mut drags: MessageReader<ScreenDrag>,
    hands: Res<crate::cockpit3d::Hands>,
    settings: Res<crate::settings::Settings>,
    time: Res<Time<Real>>,
    mut mfd: ResMut<crate::mfd::MfdState>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c.as_deref_mut()) else {
        keys.clear();
        drags.clear();
        return;
    };
    if !m.drones() {
        keys.clear();
        drags.clear();
        return;
    }
    let sim = &combat.sim;
    let uavs = center_uavs(&combat, &m);
    let contacts = map::contacts(sim, Some(c));
    let mut send = |cmd: StationCmd| c.actions.push(Action::Station(cmd));
    if !ui.placed
        && let Some(u) = sim.aaa_units().get(m.unit)
    {
        ui.placed = true;
        ui.centre = DVec2::new(u.x as f64, u.y as f64);
    }
    // A drone gone: no longer selected, its camera and edits forgotten.
    if ui.selected.is_some() && selected(&ui, &uavs).is_none() {
        ui.selected = None;
        ui.on_main = false;
        if ui.plan.is_none() {
            ui.wp = None;
        }
    }
    let ids: Vec<u16> = uavs.iter().map(|u| u.id).collect();
    ui.cams.retain(|(id, _)| ids.contains(id));
    if ui.edit.as_ref().is_some_and(|(id, _)| !ids.contains(id)) {
        ui.edit = None;
    }
    for ConsoleButton(k) in keys.read() {
        match *k {
            ConsoleKey::Auto | ConsoleKey::Manual => {
                let on = *k == ConsoleKey::Manual;
                match selected(&ui, &uavs) {
                    Some(u) => {
                        send(StationCmd::Manual { uav: u.id, on });
                        if on {
                            ui.on_main = true;
                            ui.cam_mut(u.id).forward = true;
                        }
                    }
                    None => ui.say("Select a drone first"),
                }
            }
            ConsoleKey::View => toggle_view(&mut ui, &uavs, &mut mfd),
            ConsoleKey::Track => {
                if let Some(u) = selected(&ui, &uavs) {
                    let cam = ui.cam(u.id);
                    ui.cam_mut(u.id).track = cam.look;
                }
            }
            ConsoleKey::Slave => {
                if let Some(id) = ui.selected {
                    ui.cam_mut(id).track = None;
                }
            }
            ConsoleKey::ZoomIn | ConsoleKey::ZoomOut => {
                if let Some(id) = ui.selected {
                    let d = if *k == ConsoleKey::ZoomIn { 1 } else { -1 };
                    let cam = ui.cam_mut(id);
                    cam.zoom =
                        (cam.zoom as i32 + d).clamp(0, crate::mfd::CAM_ZOOM_MAX as i32) as u8;
                }
            }
            _ => {}
        }
    }
    for d in drags.read() {
        let at = d.uv * Vec2::new(map::CANVAS.0 as f32, map::CANVAS.1 as f32);
        match (d.screen, d.phase) {
            (ConsoleScreen::Map, DragPhase::Start) => {
                ui.drag = None;
                if let Some(b) = map::button_at(&ui, &uavs, at) {
                    if b == map::Button::View {
                        toggle_view(&mut ui, &uavs, &mut mfd);
                    }
                    press(&mut ui, sim, &m, &uavs, b, &mut send);
                } else if let Some(row) = map::row_at(at) {
                    if ui.plan.is_none()
                        && let Some(u) = uavs.get(row)
                    {
                        select(&mut ui, &uavs, u.id, &mut send);
                    }
                } else if map::in_pip(at) && ui.selected.is_some() && !ui.on_main {
                    ui.on_main = true;
                } else if map::in_map(at) {
                    let route = ui.route(&uavs).map(<[UavWp]>::to_vec).unwrap_or_default();
                    match map::waypoint_at(&ui, &route, at) {
                        Some(i) => {
                            ui.wp = Some(i);
                            ui.drag = Some(Drag::Wp { index: i });
                        }
                        None => ui.drag = Some(Drag::Press { at }),
                    }
                }
            }
            (ConsoleScreen::Map, DragPhase::Move) => match ui.drag {
                Some(Drag::Press { at: from }) if from.distance(at) > CLICK => {
                    ui.drag = Some(Drag::Pan { at: from, centre: ui.centre });
                    pan_to(&mut ui, from, at);
                }
                Some(Drag::Pan { at: from, centre }) => {
                    ui.centre = centre;
                    pan_to(&mut ui, from, at);
                }
                Some(Drag::Wp { index }) => {
                    let p = map::to_map(&ui, at);
                    move_wp(&mut ui, &uavs, index, p);
                }
                _ => {}
            },
            (ConsoleScreen::Map, DragPhase::End) => match ui.drag.take() {
                Some(Drag::Press { .. }) => {
                    click_map(&mut ui, sim, &contacts, &uavs, at, &mut send)
                }
                Some(Drag::Wp { .. }) => commit_edit(&mut ui, &uavs, &mut send),
                _ => {}
            },
            _ => {}
        }
    }
    // VR: the left stick pans the map (or moves the selected waypoint), the right one zooms.
    let dt = time.delta_secs();
    if settings.vr {
        let [left, right] = &hands.hands;
        let thumb = if left.active { left.thumb } else { Vec2::ZERO };
        if thumb.length() > 0.2 {
            let step = thumb * STICK_PAN * dt;
            let route = ui.route(&uavs).map(<[UavWp]>::to_vec);
            match (ui.wp, route) {
                (Some(i), Some(r)) if i < r.len() => {
                    let s = map::to_screen(&ui, r[i].x as f64, r[i].y as f64);
                    let p = map::to_map(&ui, s + Vec2::new(step.x, -step.y));
                    move_wp(&mut ui, &uavs, i, p);
                    ui.stick_moved = true;
                }
                _ => {
                    let k = ui.scale();
                    ui.centre += DVec2::new(step.x as f64, -step.y as f64) * k;
                }
            }
        } else if std::mem::take(&mut ui.stick_moved) {
            commit_edit(&mut ui, &uavs, &mut send);
        }
        let zoom = if right.active { right.thumb.y } else { 0.0 };
        ui.zoom_wait -= dt;
        if zoom.abs() > 0.6 && ui.zoom_wait <= 0.0 {
            ui.zoom_wait = ZOOM_REPEAT;
            ui.zoom_by(if zoom > 0.0 { -1 } else { 1 });
        } else if zoom.abs() <= 0.6 {
            ui.zoom_wait = 0.0;
        }
    }
    // A refused command (the host's answer; a client's comes back from the server).
    if let Some(e) = c.ui.error.take() {
        ui.say(e);
    }
    if let Some((_, t)) = ui.note.as_mut() {
        *t -= dt;
        if *t <= 0.0 {
            ui.note = None;
        }
    }
}

/// The selected drone's camera between its TGP and the forward-looking FLIR (thermal), on the
/// main screen.
fn toggle_view(ui: &mut DroneUi, uavs: &[Uav], mfd: &mut crate::mfd::MfdState) {
    let Some(id) = ui.selected else {
        ui.say("Select a drone first");
        return;
    };
    if selected(ui, uavs).is_some_and(|u| !u.kind.tgp()) {
        ui.say("An interceptor only looks ahead");
        return;
    }
    let cam = ui.cam_mut(id);
    cam.forward = !cam.forward;
    if cam.forward {
        mfd.flir = true;
        ui.on_main = true;
    }
}

/// The map dragged from `from` to `to` (canvas pixels): its ground follows the pointer.
fn pan_to(ui: &mut DroneUi, from: Vec2, to: Vec2) {
    let d = (to - from).as_dvec2() * ui.scale();
    ui.centre -= d;
}

/// Drone `id` selected: its camera in the picture in picture (another flown by hand goes back
/// on its route).
fn select(ui: &mut DroneUi, uavs: &[Uav], id: u16, send: &mut impl FnMut(StationCmd)) {
    if ui.selected == Some(id) {
        return;
    }
    if let Some(u) = selected(ui, uavs)
        && u.mode == UavMode::Manual
    {
        send(StationCmd::Manual { uav: u.id, on: false });
    }
    ui.selected = Some(id);
    ui.wp = None;
    ui.on_main = false;
    ui.edit = None;
}

/// Waypoint `i` of the route shown moved to map point `p` (a flying drone's is sent when the
/// drag ends).
fn move_wp(ui: &mut DroneUi, uavs: &[Uav], i: usize, p: DVec2) {
    let (x, y) = (p.x.clamp(0.0, 0x7FFF as f64) as i32, p.y.clamp(0.0, 0x7FFF as f64) as i32);
    if let Some(plan) = ui.plan.as_mut() {
        if let Some(w) = plan.route.get_mut(i) {
            (w.x, w.y) = (x, y);
        }
        return;
    }
    let Some(u) = selected(ui, uavs) else { return };
    if u.mode == UavMode::Dive {
        return;
    }
    let id = u.id;
    let mut route = ui.route(uavs).map(<[UavWp]>::to_vec).unwrap_or_default();
    if let Some(w) = route.get_mut(i) {
        (w.x, w.y) = (x, y);
    }
    ui.edit = Some((id, route));
}

/// The selected drone's edited route goes to it.
fn commit_edit(ui: &mut DroneUi, uavs: &[Uav], send: &mut impl FnMut(StationCmd)) {
    if ui.plan.is_some() {
        return;
    }
    let Some((id, route)) = ui.edit.take() else { return };
    let Some(u) = uavs.iter().find(|u| u.id == id) else { return };
    if route != u.route {
        send(StationCmd::Route { uav: id, route: route.clone() });
        // Shown as sent until the simulation has it.
        ui.edit = Some((id, route));
    }
}

/// A click on the map (not on a waypoint): in a plan a new waypoint there (on a contact: the
/// route's end on it), else the drone there is selected.
fn click_map(
    ui: &mut DroneUi,
    sim: &stealth_sim::Sim,
    contacts: &[map::Contact],
    uavs: &[Uav],
    at: Vec2,
    send: &mut impl FnMut(StationCmd),
) {
    let p = map::to_map(ui, at);
    let contact = map::contact_at(ui, contacts, at);
    if let Some(plan) = ui.plan.as_mut() {
        if plan.route.last().is_some_and(|w| w.kind.last()) {
            ui.say("The route ends there: move or delete its last point");
            return;
        }
        if plan.route.len() >= MAX_WAYPOINTS {
            ui.say("No more waypoints");
            return;
        }
        let (x, y) = contact.unwrap_or((p.x as i32, p.y as i32));
        let kind = match (contact, plan.kind) {
            (Some(_), UavKind::Fp2) => WpKind::Terminate,
            (Some(_), _) => WpKind::Loiter,
            (None, _) => WpKind::Nav,
        };
        // As high as the last one (the type's height for the first).
        let alt = plan.route.last().map_or(plan.kind.perf().altitude as f32, |w| w.alt);
        plan.route.push(UavWp { x, y, alt, kind });
        ui.wp = Some(plan.route.len() - 1);
        return;
    }
    match map::drone_at(ui, sim, uavs, at) {
        Some(id) => select(ui, uavs, id, send),
        None => ui.wp = None,
    }
}

/// The map's on-screen buttons.
fn press(
    ui: &mut DroneUi,
    sim: &stealth_sim::Sim,
    m: &Manning,
    uavs: &[Uav],
    b: map::Button,
    send: &mut impl FnMut(StationCmd),
) {
    use map::Button as B;
    let pan = |ui: &mut DroneUi, dx: f64, dy: f64| {
        let k = ui.scale() * map::MAP_W as f64 / 4.0;
        ui.centre += DVec2::new(dx, dy) * k;
    };
    match b {
        B::ZoomIn => ui.zoom_by(-1),
        B::ZoomOut => ui.zoom_by(1),
        B::Left => pan(ui, -1.0, 0.0),
        B::Right => pan(ui, 1.0, 0.0),
        B::Up => pan(ui, 0.0, -1.0),
        B::Down => pan(ui, 0.0, 1.0),
        B::Centre => {
            let at = selected(ui, uavs)
                .map(|u| u.map())
                .or_else(|| sim.aaa_units().get(m.unit).map(|u| (u.x, u.y)));
            if let Some((x, y)) = at {
                ui.centre = DVec2::new(x as f64, y as f64);
            }
        }
        B::Main => {
            if ui.selected.is_some() {
                ui.on_main = !ui.on_main;
            } else {
                ui.say("Select a drone first");
            }
        }
        B::Manual => match selected(ui, uavs) {
            Some(u) => {
                let on = u.mode != UavMode::Manual;
                send(StationCmd::Manual { uav: u.id, on });
                if on {
                    ui.on_main = true;
                    ui.cam_mut(u.id).forward = true;
                }
            }
            None => ui.say("Select a drone first"),
        },
        // `toggle_view` (it needs the camera's FLIR).
        B::View => {}
        B::Type => wp_type(ui, uavs, send),
        B::Next => {
            let (Some(u), Some(i)) = (selected(ui, uavs), ui.wp) else { return };
            if ui.plan.is_none() {
                send(StationCmd::Next { uav: u.id, wp: i as u16 });
            }
        }
        B::Delete => wp_delete(ui, uavs, send),
        B::AltUp | B::AltDown => {
            let d = if b == B::AltUp { ALT_STEP } else { -ALT_STEP };
            wp_alt(ui, uavs, d, send);
        }
        B::NewDrone => {
            let stock = sim.uav_stock(m.unit);
            let kind = [UavKind::Stork, UavKind::Fp2, UavKind::Interceptor]
                .into_iter()
                .find(|k| stock[k.stock_index()] > 0)
                .unwrap_or(UavKind::Stork);
            ui.plan = Some(Plan { kind, route: Vec::new() });
            ui.wp = None;
            ui.edit = None;
        }
        B::Kind(kind) => {
            if let Some(p) = ui.plan.as_mut() {
                p.kind = kind;
                // A surveillance drone or an interceptor loiters where an attack drone would
                // terminate.
                for w in &mut p.route {
                    if kind != UavKind::Fp2 && w.kind == WpKind::Terminate {
                        w.kind = WpKind::Loiter;
                    }
                    w.alt = kind.perf().altitude as f32;
                }
            }
        }
        B::Launch => {
            let Some(p) = ui.plan.clone() else { return };
            if let Some(why) = route_refusal(p.kind, &p.route) {
                ui.say(why);
                return;
            }
            send(StationCmd::Launch { kind: p.kind, route: p.route });
            ui.plan = None;
            ui.wp = None;
        }
        B::Cancel => {
            ui.plan = None;
            ui.wp = None;
        }
    }
}

/// The selected waypoint's type steps on (NAV, LOITER, TERMINATE for an attack drone); the
/// route then ends at a loiter or terminate point.
fn wp_type(ui: &mut DroneUi, uavs: &[Uav], send: &mut impl FnMut(StationCmd)) {
    let Some(i) = ui.wp else {
        ui.say("Select a waypoint first");
        return;
    };
    let (kind, mut route) = match &ui.plan {
        Some(p) => (p.kind, p.route.clone()),
        None => {
            let Some(u) = selected(ui, uavs) else { return };
            (u.kind, ui.route(uavs).map(<[UavWp]>::to_vec).unwrap_or_default())
        }
    };
    let Some(w) = route.get_mut(i) else { return };
    w.kind = match (w.kind, kind) {
        (WpKind::Nav, _) => WpKind::Loiter,
        (WpKind::Loiter, UavKind::Fp2) => WpKind::Terminate,
        _ => WpKind::Nav,
    };
    if w.kind.last() && route.len() > i + 1 {
        route.truncate(i + 1);
        ui.say("The route ends there now");
    }
    set_route(ui, uavs, route, send);
}

/// The selected waypoint's height moved by `d` (m).
fn wp_alt(ui: &mut DroneUi, uavs: &[Uav], d: f32, send: &mut impl FnMut(StationCmd)) {
    let Some(i) = ui.wp else {
        ui.say("Select a waypoint first");
        return;
    };
    let mut route = ui.route(uavs).map(<[UavWp]>::to_vec).unwrap_or_default();
    let Some(w) = route.get_mut(i) else { return };
    w.alt = ((w.alt / ALT_STEP).round() * ALT_STEP + d).clamp(ALT_RANGE.0, ALT_RANGE.1);
    set_route(ui, uavs, route, send);
}

fn wp_delete(ui: &mut DroneUi, uavs: &[Uav], send: &mut impl FnMut(StationCmd)) {
    let Some(i) = ui.wp else {
        ui.say("Select a waypoint first");
        return;
    };
    let mut route = ui.route(uavs).map(<[UavWp]>::to_vec).unwrap_or_default();
    if i >= route.len() {
        return;
    }
    if route.len() == 1 && ui.plan.is_none() {
        ui.say("A drone keeps one waypoint");
        return;
    }
    route.remove(i);
    ui.wp = None;
    set_route(ui, uavs, route, send);
}

/// The plan's route, or the selected drone's (sent to it).
fn set_route(ui: &mut DroneUi, uavs: &[Uav], route: Vec<UavWp>, send: &mut impl FnMut(StationCmd)) {
    if let Some(p) = ui.plan.as_mut() {
        p.route = route;
        return;
    }
    let Some(id) = ui.selected else { return };
    ui.edit = Some((id, route));
    commit_edit(ui, uavs, send);
}

/// The buttons beside the camera: AUTO lit while the selected drone flies its route, MAN while
/// it is flown by hand, FLIR with the thermal picture, TRACK while its camera holds a point.
fn panel(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    ui: Res<DroneUi>,
    mfd: Res<crate::mfd::MfdState>,
    mut panel: ResMut<ConsolePanel>,
    mut pip: ResMut<PipShown>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    if !m.drones() {
        return;
    }
    use super::scope::{GREEN, YELLOW};
    let uavs = center_uavs(&combat, &m);
    let mut lit = Vec::new();
    let u = selected(&ui, &uavs);
    match u.map(|u| u.mode) {
        Some(UavMode::Manual) => lit.push((ConsoleKey::Manual, YELLOW)),
        Some(_) => lit.push((ConsoleKey::Auto, GREEN)),
        None => {}
    }
    if mfd.flir {
        lit.push((ConsoleKey::Flir, GREEN));
    }
    if u.is_some_and(|u| ui.cam(u.id).track.is_some()) {
        lit.push((ConsoleKey::Track, GREEN));
    }
    if u.is_some_and(|u| forward(&ui, u)) {
        lit.push((ConsoleKey::View, GREEN));
    }
    let want = ConsolePanel { hidden: Vec::new(), lit };
    if *panel != want {
        *panel = want;
    }
    let shown = u.is_some() && !ui.on_main;
    if pip.0 != shown {
        pip.0 = shown;
    }
}

/// Test aids: `--drone-launch stork|fp2|int[,..]` launches drones in turn once the console is up
/// (each when the rail is ready): a surveillance drone to loiter 4.5 km short of the nearest enemy
/// air defence, an attack drone (once the drones found one) to terminate on the nearest found,
/// an interceptor to loiter at the nearest enemy aircraft's height 3 km towards it (else 3 km
/// north); each launched is selected. `--drone-main` puts its camera on the main screen,
/// `--drone-manual` flies it by hand (once it is off the rail).
fn test_aid(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut c: Option<ResMut<ActiveCampaign>>,
    mut ui: ResMut<DroneUi>,
    mut manual: Local<Vec<u16>>,
) {
    let arg = |name: &str| std::env::args().skip_while(|a| a != name).nth(1);
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c.as_deref_mut()) else { return };
    if !m.drones() {
        return;
    }
    let Some(list) = arg("--drone-launch") else { return };
    let sim = &combat.sim;
    let uavs = center_uavs(&combat, &m);
    let kinds: Vec<UavKind> = list
        .split(',')
        .map(|k| match k {
            "fp2" => UavKind::Fp2,
            "int" => UavKind::Interceptor,
            _ => UavKind::Stork,
        })
        .collect();
    let launched = ui.tested as usize;
    if let Some(&kind) = kinds.get(launched)
        && sim.uav_rail_wait(m.unit) == 0
        && uavs.len() == launched
    {
        let Some(home) = sim.aaa_units().get(m.unit).map(|u| (u.x, u.y)) else { return };
        let dist = |p: (i32, i32)| ((p.0 - home.0) as f64).hypot((p.1 - home.1) as f64);
        let air = || {
            sim.enemy_aircraft()
                .map(|j| sim.flights()[j])
                .min_by(|a, b| dist((a.x, a.y)).total_cmp(&dist((b.x, b.y))))
        };
        let target = match kind {
            UavKind::Interceptor => {
                Some(air().map_or((home.0, home.1 - (3000.0 / M_PER_MAP) as i32), |f| (f.x, f.y)))
            }
            UavKind::Fp2 => sim
                .uav_known()
                .iter()
                .filter_map(|&s| sim.seen_at(s))
                .min_by(|a, b| dist(*a).total_cmp(&dist(*b))),
            UavKind::Stork => sim
                .aaa_units()
                .iter()
                .enumerate()
                .filter(|(k, u)| !u.destroyed && !sim.aaa_blue(*k))
                .map(|(_, u)| (u.x, u.y))
                .min_by(|a, b| dist(*a).total_cmp(&dist(*b))),
        };
        let Some(t) = target else { return };
        let alt = match (kind, air()) {
            (UavKind::Interceptor, Some(f)) => (f.alt as f64 * crate::world::WU) as f32,
            _ => kind.perf().altitude as f32,
        };
        let route = match kind {
            UavKind::Fp2 => vec![UavWp { x: t.0, y: t.1, alt, kind: WpKind::Terminate }],
            UavKind::Stork | UavKind::Interceptor => {
                // How much of the way back from the target: 4.5 km of it, an interceptor's
                // all but 3 km from the center.
                let d = dist(t).max(1.0);
                let short = match kind {
                    UavKind::Stork => (4500.0 / M_PER_MAP / d).min(1.0),
                    _ => 1.0 - (3000.0 / M_PER_MAP / d).min(1.0),
                };
                let x = t.0 - ((t.0 - home.0) as f64 * short) as i32;
                let y = t.1 - ((t.1 - home.1) as f64 * short) as i32;
                vec![UavWp { x, y, alt, kind: WpKind::Loiter }]
            }
        };
        ui.tested += 1;
        c.actions.push(Action::Station(StationCmd::Launch { kind, route }));
    }
    if std::env::args().any(|a| a == "--drone-manual")
        && let Some(u) = selected(&ui, &uavs)
        && u.mode == UavMode::Auto
        && !manual.contains(&u.id)
    {
        manual.push(u.id);
        c.actions.push(Action::Station(StationCmd::Manual { uav: u.id, on: true }));
    }
    // The newest one launched is selected (once).
    if let Some(u) = uavs.last()
        && uavs.len() == ui.tested as usize
        && ui.selected != Some(u.id)
        && !ui.test_picked.contains(&u.id)
    {
        ui.test_picked.push(u.id);
        ui.selected = Some(u.id);
        let (x, y) = u.map();
        ui.centre = DVec2::new(x as f64, y as f64);
        ui.on_main = std::env::args().any(|a| a == "--drone-main");
        if std::env::args().any(|a| a == "--drone-manual") {
            ui.on_main = true;
            ui.cam_mut(u.id).forward = true;
        }
        if std::env::args().any(|a| a == "--drone-forward") {
            ui.on_main = true;
            ui.cam_mut(u.id).forward = true;
        }
    }
}
