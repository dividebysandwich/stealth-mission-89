//! At the console of a friendly Patriot or NASAMS battery or Gepard (campaign, `Phase::Station`): the
//! player sits in the dark shelter of the battery's engagement control station
//! (`cockpit_console.glb`, a cockpit like the aircraft's: buttons, screens, VR fingertips).
//!
//! * The air picture (`scope`): the tracks around the site in APP-6 symbols with heading lines
//!   and history dots, refreshed twice a second (the radar's picture, not a live one).
//! * The board (`board`): the track list (a track clicked there or on the picture is selected
//!   on both), or the site's launchers, stocks and refills.
//! * The camera (`eo`): the locked track live (not at the picture's rate), in colour or FLIR;
//!   at a Gepard without a lock, where the gun points.
//! * The target data (`info`): at a Gepard the locked track's range, altitude, speed and the
//!   like; at a Patriot battery the radar's signal (`signal`): the A-scope, the lock's Doppler
//!   spectrum and angle error.
//! * The hand controller (the cockpit's stick, flight controls): at a Gepard without a lock it
//!   slews the turret; a press of its trigger (the gun trigger or fire button) fires a burst,
//!   with a lock at the lead, without one along the barrels.
//!
//! A drone command center's console (`cockpit_drone.glb`) is its own (`drone`).
//!
//! The console's commands go to the simulation through the campaign (`Action::Station`, the
//! server's on a client); what the console shows comes from the simulation's state.

mod board;
mod drone;
mod eo;
mod info;
pub(crate) mod scope;
mod signal;
mod sound;

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::aaa::{GroundKind, SamBattery};
use stealth_sim::station::{Guidance, StationCmd};

use crate::camera::{View, ViewState};
use crate::campaign::{Action, ActiveCampaign, Phase};
use crate::cockpit3d::{ConsoleButton, ConsoleKey, ConsolePanel, ConsoleScreen, ScreenClick};
use crate::combat::Combat;
use crate::flight::{Aircraft, FlightSystems, Player};
use crate::settings::AppState;
use crate::world::WorldPos;

pub use eo::ConsoleEo;
pub use scope::RADAR_REACH;

pub struct StationPlugin;

impl Plugin for StationPlugin {
    fn build(&self, app: &mut App) {
        drone::build(app);
        app.init_resource::<ConsoleUi>().add_systems(Startup, sound::load).add_systems(
            Update,
            (
                sync_manning,
                vr_console.after(sync_manning).before(buttons),
                hand_controller
                    .after(sync_manning)
                    .after(crate::script::ScriptSet)
                    .before(FlightSystems),
                seat_operator.after(FlightSystems),
                (
                    buttons,
                    panel,
                    scope::snapshot,
                    eo::aim,
                    scope::draw,
                    board::draw,
                    eo::draw,
                    info::draw,
                    signal::draw,
                    sound::play,
                )
                    .chain()
                    .after(sync_manning),
            )
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// The player mans this AAA unit's console (a Patriot radar, a Gepard, a drone command center).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Manning {
    pub unit: usize,
    pub kind: StationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationKind {
    /// A missile battery's radar: a Patriot's, a NASAMS's.
    Battery(SamBattery),
    Gepard,
    Drones,
    /// An aircraft's gunner's seat (`gunner`): `unit` is the pilot whose aircraft it is;
    /// `remote`: another's (the player's own parked away), else his own, the AI flying it;
    /// `cockpit`: a cockpit of its own (the AH-64's front seat), else a console (the AC-130's).
    Gunner {
        remote: bool,
        cockpit: bool,
    },
}

impl Manning {
    /// The console's title: PATRIOT, NASAMS, GEPARD.
    pub fn title(&self) -> &'static str {
        match self.kind {
            StationKind::Battery(b) => b.label(),
            StationKind::Gepard => "GEPARD",
            StationKind::Drones => "DCC",
            StationKind::Gunner { .. } => "GUNNER",
        }
    }

    /// The reach (m) of its weapon: the battery's missiles, the Gepard's rounds.
    pub fn reach(&self) -> f64 {
        match self.kind {
            StationKind::Battery(b) => b.max_range(),
            _ => stealth_sim::station::GEPARD_REACH,
        }
    }

    pub fn gepard(&self) -> bool {
        self.kind == StationKind::Gepard
    }

    pub fn drones(&self) -> bool {
        self.kind == StationKind::Drones
    }

    pub fn gunner(&self) -> bool {
        matches!(self.kind, StationKind::Gunner { .. })
    }

    /// At another's gunner's seat: the player's own aircraft is parked away.
    pub fn remote_gunner(&self) -> bool {
        matches!(self.kind, StationKind::Gunner { remote: true, .. })
    }

    /// The player's own aircraft waits parked meanwhile (all but his own gunner's seat, where
    /// it flies on).
    pub fn parks(&self) -> bool {
        !matches!(self.kind, StationKind::Gunner { remote: false, .. })
    }

    /// In a gunner's cockpit (the AH-64's front seat): the aircraft's cockpit, not a console.
    pub fn cockpit(&self) -> bool {
        matches!(self.kind, StationKind::Gunner { cockpit: true, .. })
    }

    /// At a console: an air defence unit's, the drone center's, the AC-130 gunner's.
    pub fn console(&self) -> bool {
        !self.cockpit()
    }
}

/// What the console shows (the player's own choices; the lock is the simulation's).
#[derive(Resource)]
pub struct ConsoleUi {
    /// The selected track (flight unit).
    pub selected: Option<usize>,
    pub tab: Tab,
    /// The air picture's range (km).
    pub range_km: f32,
    /// The launcher to fire from next (AAA unit; None: the readiest).
    pub launcher: Option<usize>,
    /// The camera's zoom step (`mfd::CAM_ZOOMS`; None: 4X, set from the range on a new lock).
    pub zoom: Option<u8>,
    /// The radar's picture: the latest and earlier ones, newest first.
    pub snapshots: std::collections::VecDeque<scope::Snapshot>,
    pub since_snapshot: f32,
    /// A message (a refused command) and how long it shows yet (s).
    pub note: Option<(String, f32)>,
    /// Where the aircraft waited before and its whole state (the seat moves it and its
    /// altitude), to put it back.
    parked: Option<(DVec3, Aircraft)>,
    /// The player's own aircraft type, while he sits in another's AH-64 (`settings.aircraft`
    /// is its then).
    parked_kind: Option<crate::aircraft::AircraftKind>,
    /// The hand controller's slew (rad/s: traverse, elevation) as last sent, its trigger held.
    pub slew: Vec2,
    trigger: bool,
    /// `--console-fire` fired, `--console-resupply` asked.
    test_fired: bool,
    test_resupply: bool,
    test_radar_off: bool,
}

impl Default for ConsoleUi {
    fn default() -> Self {
        Self {
            selected: None,
            tab: Tab::Tracks,
            range_km: 40.0,
            launcher: None,
            zoom: None,
            snapshots: Default::default(),
            since_snapshot: f32::MAX,
            note: None,
            parked: None,
            parked_kind: None,
            slew: Vec2::ZERO,
            trigger: false,
            test_fired: false,
            test_resupply: false,
            test_radar_off: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tracks,
    Site,
}

/// The air picture's ranges (km).
const RANGES: [f32; 6] = [5.0, 10.0, 20.0, 40.0, 80.0, 160.0];
/// The operator's seat: this far behind the radar (or Gepard) the shelter stands (m), the eye
/// this high.
const SHELTER_OFFSET: f64 = 25.0;
const EYE_HEIGHT: f64 = 2.6;

/// The console comes and goes with the campaign's phase.
#[allow(clippy::too_many_arguments)]
fn sync_manning(
    mut commands: Commands,
    c: Option<Res<ActiveCampaign>>,
    combat: Option<Res<Combat>>,
    manning: Option<Res<Manning>>,
    mut ui: ResMut<ConsoleUi>,
    mut player: Query<(&mut WorldPos, &mut Aircraft), With<Player>>,
    mut mfd: ResMut<crate::mfd::MfdState>,
    (seat, mut settings): (Res<crate::gunner::OwnSeat>, ResMut<crate::settings::Settings>),
) {
    let phase = c.as_deref().map(|c| c.phase);
    let own_gunner = seat.0 == stealth_sim::crew::Seat::Gunner
        && settings.aircraft.crew_seat()
        && matches!(phase, None | Some(Phase::Sortie));
    let me = combat.as_deref().map_or(0, |c| c.sim.active_pilot()) as usize;
    // Another's AH-64: its front cockpit (his aircraft as everyone sees it).
    let apache_of = |owner: u8| {
        combat
            .as_deref()
            .is_some_and(|c| c.sim.remote_pilots().iter().any(|p| p.id == owner && p.apache))
    };
    let want = match (phase, combat.as_deref()) {
        (Some(Phase::Crew { owner }), _) => Some(Manning {
            unit: owner as usize,
            kind: StationKind::Gunner { remote: true, cockpit: apache_of(owner) },
        }),
        _ if own_gunner => Some(Manning {
            unit: me,
            kind: StationKind::Gunner { remote: false, cockpit: settings.aircraft.apache() },
        }),
        (Some(Phase::Station { unit }), Some(combat)) => {
            let kind = match combat.sim.units().get(unit).map(|u| u.kind) {
                Some(GroundKind::Gepard) => StationKind::Gepard,
                Some(GroundKind::DroneCenter) => StationKind::Drones,
                kind => StationKind::Battery(
                    kind.and_then(GroundKind::battery_radar).unwrap_or(SamBattery::Patriot),
                ),
            };
            Some(Manning { unit, kind })
        }
        _ => None,
    };
    // The gunner's cockpit (`Settings::cockpit`): its own cockpit model and eye.
    let cockpit = want.is_some_and(|m| m.cockpit());
    if settings.gunner_cockpit != cockpit {
        settings.gunner_cockpit = cockpit;
    }
    if want == manning.as_deref().copied() {
        return;
    }
    match want {
        Some(m) => {
            // Off to a console: the aircraft parked (not at its own gunner's console, where it
            // flies on; a sortie begun there has its aircraft where the sortie put it).
            if m.parks()
                && manning.as_deref().is_none_or(|was| !was.parks())
                && let Ok((pos, a)) = player.single()
            {
                ui.parked = Some((pos.0, a.clone()));
            }
            let parked = if m.parks() { ui.parked.take() } else { None };
            // In another's AH-64: its cockpit and model are the player's while he sits there,
            // his own type back after (not when the aircraft becomes his: his sortie's then).
            let apache = crate::aircraft::AircraftKind::AH64;
            let kind = if m.remote_gunner() && m.cockpit() {
                ui.parked_kind.or_else(|| {
                    (settings.aircraft != apache)
                        .then(|| std::mem::replace(&mut settings.aircraft, apache))
                })
            } else {
                None
            };
            // Test aid: `--console-tab site` opens the site's page.
            let site = std::env::args().skip_while(|a| a != "--console-tab").nth(1);
            *ui = ConsoleUi {
                range_km: if m.gepard() { 10.0 } else { 40.0 },
                tab: if site.as_deref() == Some("site") { Tab::Site } else { Tab::Tracks },
                parked,
                parked_kind: kind,
                ..Default::default()
            };
            mfd.flir = false;
            commands.insert_resource(m);
        }
        None => {
            // Back to the parked aircraft (not into a sortie that put the aircraft somewhere).
            if let Some((p, parked)) = ui.parked.take()
                && phase != Some(Phase::Sortie)
                && let Ok((mut pos, mut a)) = player.single_mut()
            {
                pos.0 = p;
                *a = parked;
            }
            if let Some(k) = ui.parked_kind.take()
                && phase != Some(Phase::Sortie)
            {
                settings.aircraft = k;
            }
            mfd.flir = false;
            commands.remove_resource::<Manning>();
            commands.remove_resource::<ConsoleEo>();
        }
    }
}

/// Where the console's shelter is: behind its radar or Gepard (Bevy world), and the operator's
/// heading (radians clockwise from north: facing the unit).
fn shelter(combat: &Combat, unit: usize) -> Option<(DVec3, f32)> {
    let u = combat.sim.units().get(unit)?;
    let at = crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0);
    // The same spread of headings the units stand at (`sim_render`).
    let heading = (unit as f32 * 2.399_963).rem_euclid(std::f32::consts::TAU);
    let back = DVec3::new(-(heading as f64).sin(), 0.0, (heading as f64).cos()) * SHELTER_OFFSET;
    let at = crate::sim_render::on_ground(&combat.sim.hills(), at + back);
    Some((at + DVec3::Y * EYE_HEIGHT, heading))
}

/// The operator sits in the shelter (the parked aircraft stands there unseen), looking from the
/// cockpit.
fn seat_operator(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut player: Query<(&mut WorldPos, &mut Aircraft), With<Player>>,
    mut view: ResMut<ViewState>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    if m.gunner() {
        crate::gunner::seat_gunner(&m, &combat, &mut player, &mut view);
        return;
    }
    let Some((at, heading)) = shelter(&combat, m.unit) else { return };
    if let Ok((mut pos, mut a)) = player.single_mut() {
        if pos.0 != at {
            pos.0 = at;
        }
        a.orientation = Aircraft::from_euler(heading, 0.0, 0.0);
        a.velocity = 0.0;
        a.on_ground = true;
    }
    if !matches!(view.view, View::Cockpit | View::LookLeft | View::LookRight | View::LookRear) {
        view.view = View::Cockpit;
    }
}

/// Stick deflection below which the gun does not slew.
const STICK_DEADZONE: f32 = 0.05;
/// Steps of the slew sent (of the fastest): it is sent only when it changes.
const SLEW_STEPS: f32 = 32.0;

/// The slew rate (of the fastest) for a stick deflection: fine near the centre.
fn slew_of(v: f32) -> f32 {
    let a = ((v.abs() - STICK_DEADZONE) / (1.0 - STICK_DEADZONE)).clamp(0.0, 1.0);
    (v.signum() * a * a * SLEW_STEPS).round() / SLEW_STEPS
}

/// The hand controller: the flight controls' stick slews a Gepard's turret, a press of its
/// triggers fires a burst (and never the parked aircraft's weapons).
fn hand_controller(
    manning: Option<Res<Manning>>,
    mut c: Option<ResMut<ActiveCampaign>>,
    mut controls: ResMut<crate::input::FlightControls>,
    mut ui: ResMut<ConsoleUi>,
) {
    // The gunner's console has its own (`gunner`).
    let Some(m) = manning.filter(|m| !m.gunner()) else { return };
    let trigger = controls.gun || controls.fire;
    controls.gun = false;
    controls.fire = false;
    let Some(c) = c.as_deref_mut() else { return };
    if !m.gepard() {
        return;
    }
    use stealth_sim::station::{GEPARD_ELEVATE, GEPARD_TRAVERSE};
    let slew = Vec2::new(
        slew_of(controls.roll) * GEPARD_TRAVERSE,
        slew_of(controls.pitch) * GEPARD_ELEVATE,
    );
    if slew != ui.slew {
        ui.slew = slew;
        c.actions.push(Action::Station(StationCmd::Slew { az: slew.x, el: slew.y }));
    }
    if trigger && !ui.trigger {
        c.actions.push(Action::Station(StationCmd::Burst));
    }
    ui.trigger = trigger;
}

/// How often a held stick direction steps the zoom again (s).
const VR_ZOOM_REPEAT: f32 = 0.35;

/// VR at a console (not the drone command center's, which has its own): the left controller
/// works the camera and the lock as the console's buttons do. Stick up / down zooms in / out
/// (again while held), its click toggles the FLIR, X tracks (locks the selected track; the
/// gunner's pod's point track), Y drops the lock (the gunner's: the other gun), stick left /
/// right lays the 40 mm / 105 mm at the gunner's console.
fn vr_console(
    manning: Option<Res<Manning>>,
    settings: Res<crate::settings::Settings>,
    hands: Res<crate::cockpit3d::Hands>,
    menu: Res<crate::flight_menu::FlightMenu>,
    time: Res<Time<Real>>,
    mut wait: Local<f32>,
    mut out: MessageWriter<ConsoleButton>,
    gunner: Res<crate::gunner::GunnerState>,
) {
    use crate::cockpit3d::{
        BUTTON_PRIMARY, BUTTON_SECONDARY, BUTTON_STICK_DOWN, BUTTON_STICK_LEFT,
        BUTTON_STICK_RIGHT, BUTTON_STICK_UP, BUTTON_THUMB,
    };
    let Some(m) = manning.filter(|m| !m.drones() && m.console()) else { return };
    let left = &hands.hands[0];
    if !settings.vr || !left.active || menu.open || left.scrolling {
        return;
    }
    let mut press = |k: ConsoleKey| {
        out.write(ConsoleButton(k));
    };
    // Zoom: a step on the push, then again every `VR_ZOOM_REPEAT` while held.
    *wait -= time.delta_secs();
    for (bit, key) in [(BUTTON_STICK_UP, ConsoleKey::ZoomIn), (BUTTON_STICK_DOWN, ConsoleKey::ZoomOut)] {
        if left.just_pressed(bit) || (left.held(bit) && *wait <= 0.0) {
            press(key);
            *wait = VR_ZOOM_REPEAT;
        }
    }
    if left.just_pressed(BUTTON_THUMB) {
        press(ConsoleKey::Flir);
    }
    if left.just_pressed(BUTTON_PRIMARY) {
        press(ConsoleKey::Lock);
    }
    if left.just_pressed(BUTTON_SECONDARY) {
        press(if m.gunner() { ConsoleKey::Gun(2 - gunner.gun_laid()) } else { ConsoleKey::Drop });
    }
    if m.gunner() {
        if left.just_pressed(BUTTON_STICK_LEFT) {
            press(ConsoleKey::Gun(1));
        }
        if left.just_pressed(BUTTON_STICK_RIGHT) {
            press(ConsoleKey::Gun(2));
        }
    }
}

/// The console's buttons and screen clicks.
#[allow(clippy::too_many_arguments)]
fn buttons(
    manning: Option<Res<Manning>>,
    mut c: Option<ResMut<ActiveCampaign>>,
    combat: Option<Res<Combat>>,
    mut ui: ResMut<ConsoleUi>,
    mut keys: MessageReader<ConsoleButton>,
    mut clicks: MessageReader<ScreenClick>,
    mut actions: MessageReader<crate::input::ActionEvent>,
    mut zoom_axis: MessageReader<crate::hotas::TgpZoomAxis>,
    mut mfd: ResMut<crate::mfd::MfdState>,
    time: Res<Time<Real>>,
) {
    let (Some(m), Some(c), Some(combat)) =
        (manning.filter(|m| !m.gunner()), c.as_deref_mut(), combat)
    else {
        keys.clear();
        clicks.clear();
        actions.clear();
        zoom_axis.clear();
        return;
    };
    // The TGP zoom axis sets the camera's zoom step (the drone center's: `drone`).
    if let Some(crate::hotas::TgpZoomAxis(z)) = zoom_axis.read().last()
        && !m.drones()
    {
        ui.zoom = Some(*z);
    }
    // The bound camera zoom controls work the console's camera too.
    for crate::input::ActionEvent(a) in actions.read() {
        match a {
            crate::input::Action::CameraZoomIn => zoom_step(&mut ui, 1),
            crate::input::Action::CameraZoomOut => zoom_step(&mut ui, -1),
            _ => {}
        }
    }
    let sim = &combat.sim;
    let launchers = if m.gepard() { vec![m.unit] } else { sim.battery_launchers(m.unit) };
    let radar_off = sim.station_of(c.me.0).is_some_and(|s| s.radar_off);
    let mut send = |cmd: StationCmd| c.actions.push(Action::Station(cmd));
    for ConsoleButton(k) in keys.read() {
        match *k {
            ConsoleKey::Tracks => ui.tab = Tab::Tracks,
            ConsoleKey::Site => ui.tab = Tab::Site,
            ConsoleKey::Lock => match ui.selected {
                Some(t) => {
                    ui.zoom = None;
                    send(StationCmd::Lock(Some(t as u16)));
                }
                None => ui.note = Some((stealth_i18n::tf!("st-select-track").into(), 3.0)),
            },
            ConsoleKey::Drop => send(StationCmd::Lock(None)),
            ConsoleKey::Guided => send(StationCmd::Mode(Guidance::Guided)),
            ConsoleKey::Semi => send(StationCmd::Mode(Guidance::SemiActive)),
            ConsoleKey::Fire if m.gepard() => send(StationCmd::Burst),
            ConsoleKey::Fire => send(StationCmd::Fire { launcher: ui.launcher.map(|l| l as u16) }),
            ConsoleKey::Launcher(n) => {
                let l = launchers.get(n.saturating_sub(1) as usize).copied();
                ui.launcher = if ui.launcher == l { None } else { l };
            }
            ConsoleKey::Refill(n) => {
                if let Some(&l) = launchers.get(n.saturating_sub(1) as usize) {
                    send(StationCmd::Refill { unit: l as u16 });
                }
            }
            ConsoleKey::Flir => mfd.flir = !mfd.flir,
            ConsoleKey::ZoomIn => zoom_step(&mut ui, 1),
            ConsoleKey::ZoomOut => zoom_step(&mut ui, -1),
            ConsoleKey::RangeIn => step_range(&mut ui, -1),
            ConsoleKey::RangeOut => step_range(&mut ui, 1),
            ConsoleKey::Leave => send(StationCmd::Leave),
            ConsoleKey::Radar => send(StationCmd::Radar(radar_off)),
            // The drone command center's (`drone`).
            ConsoleKey::Auto
            | ConsoleKey::Manual
            | ConsoleKey::Track
            | ConsoleKey::Slave
            | ConsoleKey::View
            | ConsoleKey::Gun(_) => {}
        }
    }
    for click in clicks.read() {
        match click.screen {
            ConsoleScreen::Scope => {
                if let Some(t) = scope::pick(&ui, click.uv) {
                    ui.selected = Some(t);
                }
            }
            ConsoleScreen::Board => match board::click(&ui, sim, &launchers, click.uv) {
                Some(board::Clicked::Track(t)) => ui.selected = Some(t),
                Some(board::Clicked::Tab(t)) => ui.tab = t,
                Some(board::Clicked::Launcher(l)) => {
                    ui.launcher = if ui.launcher == Some(l) { None } else { Some(l) };
                }
                Some(board::Clicked::Resupply) => send(StationCmd::RequestResupply),
                None => {}
            },
            ConsoleScreen::Eo | ConsoleScreen::Info | ConsoleScreen::Map | ConsoleScreen::Feed => {}
        }
    }
    // Test aid: `--console-lock` locks the nearest hostile track once there is one (any
    // track with `--console-lock-any`), `--console-flir` turns the camera thermal.
    let any = std::env::args().any(|a| a == "--console-lock-any");
    if (any || std::env::args().any(|a| a == "--console-lock"))
        && sim.station_of(c.me.0).is_some_and(|s| s.lock.is_none())
        && ui.selected.is_none()
        && let Some(t) = ui.snapshots.front().and_then(|s| {
            s.tracks
                .iter()
                .filter(|t| (any || t.side == scope::Side::Hostile) && t.unit.is_some())
                .min_by(|a, b| a.range_km().total_cmp(&b.range_km()))
                .and_then(|t| t.unit)
        })
    {
        if std::env::args().any(|a| a == "--console-flir") {
            mfd.flir = true;
        }
        ui.selected = Some(t);
        send(StationCmd::Lock(Some(t as u16)));
    }
    // Test aid: `--console-fire` fires once at the lock.
    if std::env::args().any(|a| a == "--console-fire")
        && !ui.test_fired
        && sim.station_of(c.me.0).is_some_and(|s| s.lock.is_some())
    {
        ui.test_fired = true;
        send(if m.gepard() { StationCmd::Burst } else { StationCmd::Fire { launcher: None } });
    }
    // Test aid: `--console-radar-off` switches the radar off once.
    if std::env::args().any(|a| a == "--console-radar-off") && !ui.test_radar_off {
        ui.test_radar_off = true;
        send(StationCmd::Radar(false));
    }
    // Test aid: `--console-resupply` asks for a new stockpile once.
    if std::env::args().any(|a| a == "--console-resupply") && !ui.test_resupply {
        ui.test_resupply = true;
        send(StationCmd::RequestResupply);
    }
    // A refused command (the host's answer; a client's comes back from the server).
    if let Some(e) = c.ui.error.take() {
        ui.note = Some((e.tr_flight(), 4.0));
    }
    if let Some((_, t)) = ui.note.as_mut() {
        *t -= time.delta_secs();
        if *t <= 0.0 {
            ui.note = None;
        }
    }
    // A launcher lost or gone from the battery is not chosen any more.
    if ui.launcher.is_some_and(|l| !launchers.contains(&l)) {
        ui.launcher = None;
    }
}

/// How fast the FIRE button blinks (Hz).
const FIRE_BLINK: f32 = 2.0;

/// The console's buttons: a Gepard's without the missile battery's (launchers, guidance, the
/// second and third reload); LOCK lit with a lock, RADAR with the radar on, FLIR with the
/// thermal picture, FIRE lit with a lock out of reach and blinking with a hostile one in reach.
fn panel(
    manning: Option<Res<Manning>>,
    c: Option<Res<ActiveCampaign>>,
    combat: Option<Res<Combat>>,
    mfd: Res<crate::mfd::MfdState>,
    time: Res<Time<Real>>,
    mut panel: ResMut<ConsolePanel>,
) {
    let (Some(m), Some(c), Some(combat)) = (manning.filter(|m| !m.gunner()), c, combat) else {
        if *panel != ConsolePanel::default() {
            *panel = ConsolePanel::default();
        }
        return;
    };
    // The drone command center lights its own (`drone`).
    if m.drones() {
        return;
    }
    let sim = &combat.sim;
    let hidden = if m.gepard() {
        vec![
            ConsoleKey::Guided,
            ConsoleKey::Semi,
            ConsoleKey::Launcher(1),
            ConsoleKey::Launcher(2),
            ConsoleKey::Launcher(3),
            ConsoleKey::Refill(2),
            ConsoleKey::Refill(3),
        ]
    } else {
        Vec::new()
    };
    let lock = sim.station_of(c.me.0).and_then(|s| s.lock).map(|l| l as usize);
    let mut lit = Vec::new();
    if lock.is_some() {
        lit.push((ConsoleKey::Lock, scope::GREEN));
    }
    if sim.station_of(c.me.0).is_some_and(|s| !s.radar_off) {
        lit.push((ConsoleKey::Radar, scope::GREEN));
    }
    if mfd.flir {
        lit.push((ConsoleKey::Flir, scope::GREEN));
    }
    if let Some(j) = lock {
        let engage = sim.in_reach(m.unit, j)
            && sim.flights().get(j).is_some_and(|f| !sim.friendly_unit(j, f.spec));
        if !engage || (time.elapsed_secs() * FIRE_BLINK).fract() < 0.5 {
            lit.push((ConsoleKey::Fire, scope::RED));
        }
    }
    let want = ConsolePanel { hidden, lit };
    if *panel != want {
        *panel = want;
    }
}

fn zoom_step(ui: &mut ConsoleUi, d: i32) {
    use crate::mfd::{CAM_ZOOM_DEFAULT, CAM_ZOOM_MAX};
    let z = ui.zoom.unwrap_or(CAM_ZOOM_DEFAULT) as i32 + d;
    ui.zoom = Some(z.clamp(0, CAM_ZOOM_MAX as i32) as u8);
}

fn step_range(ui: &mut ConsoleUi, d: i32) {
    let i = RANGES.iter().position(|&r| r >= ui.range_km).unwrap_or(RANGES.len() - 1) as i32;
    ui.range_km = RANGES[(i + d).clamp(0, RANGES.len() as i32 - 1) as usize];
}

/// Metres (east, north, up) of a flight unit.
fn flight_metres(u: &stealth_sim::FlightUnit) -> [f64; 3] {
    let m = stealth_sim::aaa::M_PER_MAP / 32.0;
    [u.wx as f64 * m, (0x10_0000 - u.wy) as f64 * m, u.alt.max(0) as f64 * m]
}
