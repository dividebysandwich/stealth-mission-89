//! The AC-130's gunner's console (`station::StationKind::Gunner`): a big screen with the
//! targeting pod's picture and the side guns' fire control, a hand controller that slews the
//! pod at a rate (as the Gepard's turret) and fires, the numeric keypad's slew keys too.
//!
//! * The pilot moves to it from his seat (the in-flight menu's seat entry): his own flight
//!   model flies on, the autopilot flying a left-hand orbit round the point the pod holds
//!   ([`Orbit`]), and his pod and trigger go to the simulation as ever.
//! * Another player sits at it (`Phase::Crew`): his own aircraft parked away, the eye rides in
//!   the pilot's aircraft as everyone sees it, his pod runs here on its motion and goes to the
//!   server with the trigger (`CrewCmd::Aim`), the server's fire control answers in his
//!   `CrewView`.
//!
//! The AH-64's gunner sits in its front cockpit instead (`Manning::cockpit`, the cockpit and eye
//! `Settings::gunner_seat`'s): the stick slews the TADS (the targeting pod), the chin gun follows
//! it and the weapons fire as from the pilot's seat (another's go to the server: the trigger,
//! the station selected, `CrewCmd::Fire`). The pilot's own seat empty, the AI flies on with the
//! autopilot or the auto-hover (`crate::flight::crew_autopilot`), at the height over the ground
//! it was at (`hold_height`). The OH-58D's gunner sits in its left seat the same way, the stick
//! slewing the mast-mounted sight; he fires only the missiles (the gun pods fire along the
//! nose, the pilot's).

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::crew::{CrewCmd, GunnerAim, Seat};

use crate::avionics::{Format, LockRequest, PodDesignate, PodTrack, TgpMode};
use crate::camera::{View, ViewState};
use crate::canvas::CanvasTexture;
use crate::cockpit3d::{ConsoleButton, ConsoleCanvas, ConsoleKey, ConsolePanel, ConsoleScreen};
use crate::combat::Combat;
use crate::flight::{Aircraft, FlightSystems, Player};
use crate::flight_menu::{FlightMenu, SeatChoice};
use crate::font::{Align, stroke_text_at};
use crate::mfd::ContactRef;
use crate::settings::AppState;
use crate::station::Manning;
use crate::world::{WU, WorldPos, bevy_to_wu};

pub struct GunnerPlugin;

impl Plugin for GunnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OwnSeat>().init_resource::<GunnerState>().add_systems(
            Update,
            (
                single_mission_seat.before(FlightSystems),
                console
                    .after(crate::input::InputSystemsSet)
                    .after(crate::script::ScriptSet)
                    .before(FlightSystems),
                orbit.before(FlightSystems).after(console),
                (send_aim, panel, draw_screen, draw_info)
                    .chain()
                    .after(crate::trackcam::update_camera),
            )
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// The player's seat in his own aircraft (in a campaign the simulation's, `Sim::owner_seat`;
/// in a single mission his own choice).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OwnSeat(pub Seat);

/// The console's own state.
#[derive(Resource, Default)]
pub struct GunnerState {
    /// At the console now (the kind), and what it changed to put back on leaving: the avionics
    /// (the pod needs the modern ones), the MFD formats and the sensor of interest.
    at: Option<bool>,
    saved: Option<(bool, [Format; 4], Option<Format>)>,
    /// The aim last sent (another's console) and seconds since.
    sent: Option<GunnerAim>,
    since: f32,
    /// The gun laid next at another's console (0: 40 mm, 1: 105 mm).
    gun: u8,
    /// The trigger held at another's console (sent with the aim).
    trigger: bool,
    /// The gun the console shows laid (its own or another's).
    shown_gun: u8,
    /// The AI's orbit: the speed it holds (knots), the throttle it sets (`flight::step_flight`
    /// flies with it) and its speed hold's integral part.
    orbit_speed: f32,
    pub orbit_throttle: f32,
    throttle_trim: f32,
    /// A helicopter's autopilot in the gunner's seat: the height over the ground it holds
    /// (altitude units).
    height: Option<f32>,
    /// `--seat gunner@<s>` done.
    test_seat: bool,
    /// Another's AH-64: the station whose missiles he selected (None: the chin gun).
    station: Option<u8>,
}

impl GunnerState {
    /// The gun laid next (0: 40 mm, 1: 105 mm), as the console last showed it.
    pub fn gun_laid(&self) -> u8 {
        self.shown_gun
    }
}

/// How often the aim goes again unchanged (s): it goes unreliable.
const AIM_REFRESH: f32 = 0.25;
/// The orbit's bank: the radius is the turn's at this bank and the speed, a margin under the
/// autopilot's 30 degrees.
const ORBIT_BANK: f32 = 0.47;
/// The orbit is at least this wide (m).
const ORBIT_LEAST: f32 = 800.0;
/// The AI's speed hold in the orbit: the speed held is the one the pilot left the seat at, at
/// least this much over the stall; the throttle by the error (per knot) and its integral (per
/// knot second).
const ORBIT_STALL_MARGIN: f32 = 1.3;
const SPEED_GAIN: f32 = 0.03;
const SPEED_INTEGRAL: f32 = 0.004;
/// The guns look this far down from the horizon at least (rad): the autopilot climbs to see
/// the point from there (the gimbal's 5 degrees under the wing, in the bank).
const ORBIT_DEPRESSION: f32 = 0.70;
/// The orbit clears what stands ahead by this much at least (altitude units, ~500 ft).
const ORBIT_CLEARANCE: f32 = 300.0;
/// A helicopter's autopilot in the gunner's seat holds at least this height over the ground,
/// and clears what stands on it by up to this much (altitude units, ~50 and ~100 ft).
const HEIGHT_LEAST: f32 = 30.0;
const OBSTACLE_CLEARANCE: f32 = 60.0;

/// Where the gunner's console's eye is: in the pilot's aircraft as it is drawn (another's,
/// between the server's frames), the view kept inside; its engines run as his flight model
/// reports them, for the sound.
pub fn seat_gunner(
    m: &Manning,
    combat: &Combat,
    player: &mut Query<(&mut WorldPos, &mut Aircraft), With<Player>>,
    view: &mut ViewState,
    dt: f32,
) {
    if !m.remote_gunner() {
        return;
    }
    let owner = m.unit as u8;
    let Some(p) = combat.sim.remote_pilots().into_iter().find(|p| p.id == owner) else { return };
    let (at, rot) = crate::sim_render::remote_pose(combat, &p);
    let v = combat.remote_track.view_at(owner, combat.remote_time, p.view);
    let engine = combat.sim.crew_view(combat.sim.active_pilot()).map(|c| c.engine);
    if let Ok((mut pos, mut a)) = player.single_mut() {
        pos.0 = at;
        a.orientation = rot;
        a.altitude = v.alt as f32;
        a.ground_altitude = 0.0;
        a.velocity = v.velocity as f32;
        if let Some(e) = engine {
            a.show_engine(p.helicopter, e, dt);
        }
        a.on_ground = false;
        a.ended = None;
        // The owner's flight model has the stall: left at the gunner's own last aircraft's, the
        // view shook with its buffet at the owner's (helicopter's) speeds all the time.
        a.stall_speed = 0.0;
    }
    if !matches!(view.view, View::Cockpit | View::LookLeft | View::LookRight | View::LookRear) {
        view.view = View::Cockpit;
    }
}

/// A single mission's seat entry in the menu (a campaign's is the campaign's), the switch, and
/// the `--seat gunner[@s]` test aid.
fn single_mission_seat(
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    mut menu: ResMut<FlightMenu>,
    mut seat: ResMut<OwnSeat>,
    mut state: ResMut<GunnerState>,
    settings: Res<crate::settings::Settings>,
    player: Query<&Aircraft, With<Player>>,
    time: Res<Time>,
) {
    let test = std::env::args().skip_while(|a| a != "--seat").nth(1);
    let test_at = test.as_deref().and_then(|t| {
        let (seat, at) = t.split_once('@').unwrap_or((t, "0"));
        (seat == "gunner").then(|| at.parse::<f32>().unwrap_or(0.0))
    });
    if let Some(c) = campaign.as_deref_mut() {
        if let Some(at) = test_at
            && !state.test_seat
            && time.elapsed_secs() >= at
            && menu.seat == SeatChoice::ToGunner
        {
            state.test_seat = true;
            c.actions.push(crate::campaign::Action::Crew(CrewCmd::Seat(Seat::Gunner)));
        }
        return;
    }
    let flying = player.single().is_ok_and(|a| a.ended.is_none());
    menu.seat = match seat.0 {
        _ if !settings.aircraft.crew_seat() || !flying => SeatChoice::None,
        Seat::Pilot => SeatChoice::ToGunner,
        Seat::Gunner => SeatChoice::ToPilot,
    };
    if let Some(at) = test_at
        && !state.test_seat
        && time.elapsed_secs() >= at
        && menu.seat == SeatChoice::ToGunner
    {
        state.test_seat = true;
        menu.seat_request = true;
    }
    if std::mem::take(&mut menu.seat_request) {
        seat.0 = match menu.seat {
            SeatChoice::ToGunner => Seat::Gunner,
            _ => Seat::Pilot,
        };
    }
    if !settings.aircraft.crew_seat() || !flying {
        seat.0 = Seat::Pilot;
    }
}

/// The hand controller and keys at the console: the stick slews the pod at a rate (the slew
/// keys too), its triggers fire the laid gun; the pod's page is the sensor of interest.
#[allow(clippy::too_many_arguments)]
fn console(
    manning: Option<Res<Manning>>,
    mut state: ResMut<GunnerState>,
    mut controls: ResMut<crate::input::FlightControls>,
    mut mfd: ResMut<crate::mfd::MfdState>,
    mut player: Query<&mut Aircraft, With<Player>>,
    mut keys: MessageReader<ConsoleButton>,
    mut out: MessageWriter<crate::input::ActionEvent>,
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    mut menu: ResMut<FlightMenu>,
    (combat, settings): (Option<Res<Combat>>, Res<crate::settings::Settings>),
) {
    let now = manning.as_deref().filter(|m| m.gunner()).map(|m| m.remote_gunner());
    let cockpit = manning.as_deref().is_some_and(|m| m.cockpit());
    if now != state.at {
        match (state.at, now) {
            // To the console: the pod's page up as the sensor of interest.
            (None, Some(_)) => {
                state.saved = Some((mfd.modern, mfd.av.formats, mfd.av.soi));
                state.sent = None;
            }
            // Back: the avionics as they were, the autopilot off.
            (Some(_), None) => {
                if let Some((modern, formats, soi)) = state.saved.take() {
                    mfd.modern = modern;
                    mfd.av.formats = formats;
                    mfd.av.soi = soi;
                }
                if let Ok(mut a) = player.single_mut()
                    && a.orbit.take().is_some()
                {
                    a.autopilot_alt = 0.0;
                }
            }
            _ => {}
        }
        state.at = now;
    }
    let Some(remote) = now else {
        keys.clear();
        return;
    };
    mfd.modern = true;
    // On the console's screen, the AH-64 gunner's TEDAC (his third, centre display), else the
    // gunner's right MFD.
    if mfd.av.shown(Format::Tgp).is_none() {
        let tedac = settings.cockpit().0 == crate::cockpit3d::layout::PanelStyle::Ah64Cpg;
        let i = if cockpit && tedac { 2 } else { 1 };
        mfd.av.formats[i] = Format::Tgp;
    }
    mfd.av.soi = Some(Format::Tgp);
    // The stick slews the pod, as fine near the centre as the Gepard's turret: pushed (the up
    // arrow) the pod looks up, as the slew keys move it, not as the nose goes.
    let curve = |v: f32| {
        let a = ((v.abs() - 0.05) / 0.95).clamp(0.0, 1.0);
        v.signum() * a * a
    };
    let stick = Vec2::new(curve(controls.roll), curve(-controls.pitch));
    controls.slew = Vec2::new(
        crate::input::stronger(controls.slew.x, stick.x),
        crate::input::stronger(controls.slew.y, stick.y),
    )
    .clamp(Vec2::NEG_ONE, Vec2::ONE);
    controls.roll = 0.0;
    controls.pitch = 0.0;
    controls.yaw = 0.0;
    // A gunner's cockpit: the weapons fire as from the pilot's seat (the chin gun along the
    // TADS); another's trigger and release go to the server (`send_aim`).
    if cockpit {
        keys.clear();
        if remote {
            state.trigger = controls.gun || (state.station.is_none() && controls.fire);
            controls.gun = false;
            controls.fire = false;
        }
        return;
    }
    // The triggers fire the side guns: the pilot's own trigger at his own console (the
    // simulation's side guns follow it), another's goes to the server.
    let trigger = controls.gun || controls.fire;
    controls.gun = false;
    controls.fire = !remote && trigger;
    if remote {
        state.trigger = trigger;
    }
    let gun_now = |state: &GunnerState| {
        if remote {
            state.gun
        } else {
            combat.as_deref().and_then(|c| c.sim.side_guns()).map_or(0, |g| g.gun)
        }
    };
    let select = |state: &mut GunnerState, gun: u8, out: &mut MessageWriter<_>| {
        if remote {
            state.gun = gun.min(1);
        } else if gun_now(state) != gun {
            out.write(crate::input::ActionEvent(crate::input::Action::SelectWeapon));
        }
    };
    for ConsoleButton(k) in keys.read() {
        match *k {
            ConsoleKey::Gun(n) => select(&mut state, n.saturating_sub(1), &mut out),
            ConsoleKey::Lock => {
                out.write(crate::input::ActionEvent(crate::input::Action::LockCursor));
            }
            ConsoleKey::Flir => mfd.flir = !mfd.flir,
            ConsoleKey::ZoomIn => zoom(&mut mfd, 1),
            ConsoleKey::ZoomOut => zoom(&mut mfd, -1),
            ConsoleKey::Fire => {}
            // Up from the console: another's back to operations, his own back to the
            // pilot's seat.
            ConsoleKey::Leave => {
                let cmd = if remote { CrewCmd::Leave } else { CrewCmd::Seat(Seat::Pilot) };
                match campaign.as_deref_mut() {
                    Some(c) => c.actions.push(crate::campaign::Action::Crew(cmd)),
                    None => {
                        menu.seat = SeatChoice::ToPilot;
                        menu.seat_request = true;
                    }
                }
            }
            _ => {}
        }
    }
}

/// The camera's zoom a step in or out (`mfd::CAM_ZOOMS`).
fn zoom(mfd: &mut crate::mfd::MfdState, step: i32) {
    let n = crate::mfd::CAM_ZOOMS.len() as i32;
    let z = mfd.cam_zoom.map_or(2, |z| z as i32);
    mfd.cam_zoom = Some((z + step).clamp(0, n - 1) as u8);
}

/// The AI flies the pilot's aircraft while he is at its gunner's console: the autopilot on a
/// left-hand orbit round the point the pod holds, high enough for the guns to bear.
#[allow(clippy::too_many_arguments)]
fn orbit(
    manning: Option<Res<Manning>>,
    mfd: Res<crate::mfd::MfdState>,
    k: Res<crate::flight::FlightConstants>,
    mut player: Query<(&mut Aircraft, &WorldPos), With<Player>>,
    theater: Option<Res<crate::world::Theater>>,
    combat: Option<Res<Combat>>,
    mut state: ResMut<GunnerState>,
    time: Res<Time>,
) {
    let Ok((mut a, pos)) = player.single_mut() else { return };
    let own = manning.as_deref().is_some_and(|m| m.gunner() && !m.remote_gunner());
    // A helicopter flies on with its autopilot or hovers, at its height over the ground.
    if k.heli.is_some() {
        if own && crate::flight::crew_autopilot(&mut a, &k) {
            hold_height(&mut a, &mut state.height);
        } else {
            state.height = None;
        }
        return;
    }
    if !own || a.on_ground {
        if a.orbit.take().is_some() {
            a.autopilot_alt = 0.0;
        }
        return;
    }
    // The point the pod holds (not while it is slewed), else where the orbit was, else here.
    let locked = combat.as_deref().is_some_and(|c| c.sim.targeting().ground.is_some());
    let held = match mfd.av.tgp.mode {
        TgpMode::Point(_) => mfd.av.tgp.los_ground,
        TgpMode::Slaved if locked => mfd.av.tgp.los_ground,
        _ => None,
    };
    let flat_y = crate::world::wu_to_bevy(0.0, 0.0, a.ground_altitude as f64).y;
    let ground_at = |p: DVec3| {
        let hills = theater.as_deref().map_or(0.0, |t| t.hills.ground_m(p.x, -p.z).max(0.0));
        DVec3::new(p.x, flat_y.max(hills as f64), p.z)
    };
    let centre = held.or(a.orbit).unwrap_or_else(|| ground_at(pos.0));
    let first = a.orbit.is_none();
    a.orbit = Some(centre);
    // The speed held: the one he left the seat at (well clear of the stall), the throttle from
    // where it was.
    if first {
        let stall = a.stall_speed / 27.0;
        state.orbit_speed = a.knots().max(stall * ORBIT_STALL_MARGIN);
        state.throttle_trim = (a.set_thrust / 100.0).clamp(0.0, 1.0);
    }
    let err = state.orbit_speed - a.knots();
    let dt = time.delta_secs();
    state.throttle_trim = (state.throttle_trim + err * SPEED_INTEGRAL * dt).clamp(0.0, 1.0);
    state.orbit_throttle = (state.throttle_trim + err * SPEED_GAIN).clamp(0.0, 1.0);
    let v = (state.orbit_speed * 0.514_444).max(40.0);
    let radius = (v * v / (9.81 * ORBIT_BANK.tan())).max(ORBIT_LEAST);
    // High enough over the point to see it under the wing.
    let ground_m = ground_at(centre).y.max(centre.y) as f32;
    let least_wu = (ground_m + radius * ORBIT_DEPRESSION.tan()) / WU as f32;
    if first || a.autopilot_alt <= 0.0 {
        a.autopilot_alt = crate::flight::view_z(a.altitude.max(least_wu));
    } else {
        a.autopilot_alt = a.autopilot_alt.max(crate::flight::view_z(least_wu));
    }
    // Over whatever stands ahead on the way round.
    if let Some(t) = a.terrain_ahead {
        let clear = crate::flight::view_z(t.top.max(t.ground) + ORBIT_CLEARANCE);
        a.autopilot_alt = a.autopilot_alt.max(clear);
    }
    // Tangent to the circle, the point on the left, turning in when outside it.
    let (e0, n0, _) = bevy_to_wu(pos.0);
    let (e1, n1, _) = bevy_to_wu(centre);
    let (de, dn) = ((e1 - e0) * WU, (n1 - n0) * WU);
    let d = de.hypot(dn) as f32;
    let brg = (de as f32).atan2(dn as f32);
    let heading = brg + std::f32::consts::FRAC_PI_2 - ((d - radius) / radius).clamp(-1.0, 1.0);
    a.waypoint_bearing = crate::flight::to_bam(heading).rem_euclid(65536.0);
}

/// A helicopter's autopilot in the gunner's seat: the altitude it holds kept at the height over
/// the ground it had when the AI took over (`height`), over the hills ahead in time to climb
/// (`Aircraft::terrain_ahead`) and clear of the buildings, trees and masts there.
fn hold_height(a: &mut Aircraft, height: &mut Option<f32>) {
    let Some(t) = a.terrain_ahead else { return };
    let h = *height.get_or_insert((a.altitude - a.ground_altitude).max(HEIGHT_LEAST));
    let alt = (t.ground + h).max(t.top + h.min(OBSTACLE_CLEARANCE));
    a.autopilot_alt = crate::flight::view_z(alt).max(1.0);
}

/// Another's console: the pod's aim and the trigger to the server, with the designations.
#[allow(clippy::too_many_arguments)]
fn send_aim(
    manning: Option<Res<Manning>>,
    mut state: ResMut<GunnerState>,
    mfd: Res<crate::mfd::MfdState>,
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    combat: Option<Res<Combat>>,
    mut designate: MessageReader<PodDesignate>,
    mut track: MessageReader<PodTrack>,
    mut locks: MessageReader<LockRequest>,
    mut actions: MessageReader<crate::input::ActionEvent>,
    (time, settings): (Res<Time>, Res<crate::settings::Settings>),
) {
    let remote = manning.as_deref().is_some_and(|m| m.remote_gunner());
    let (Some(c), true) = (campaign.as_deref_mut(), remote) else {
        designate.clear();
        track.clear();
        locks.clear();
        actions.clear();
        return;
    };
    // The weapon key steps the gun laid next; in an AH-64 the chin gun and the kinds of
    // missiles left (as the pilot's own), in an OH-58D only the missiles (not the gun pods'
    // stations); the release fires the missile selected.
    let cockpit = manning.as_deref().is_some_and(|m| m.cockpit());
    let gun = settings.aircraft.spec().turret;
    let mut stores = combat
        .as_deref()
        .and_then(|c| c.sim.crew_view(c.sim.active_pilot()))
        .map_or([(-1, 0); 6], |v| v.stores);
    if let Some(c) = combat.as_deref() {
        for s in stores.iter_mut().filter(|s| c.sim.gun_pod(s.0 as i32)) {
            *s = (-1, 0);
        }
    }
    let next = |from: Option<u8>| {
        next_station(from, &stores).or_else(|| (!gun).then(|| next_station(None, &stores))?)
    };
    if cockpit && !gun && state.station.is_none() {
        state.station = next(None);
    }
    let mut cmds = Vec::new();
    for crate::input::ActionEvent(a) in actions.read() {
        match *a {
            crate::input::Action::SelectWeapon if cockpit => {
                state.station = next(state.station);
            }
            crate::input::Action::SelectWeapon => state.gun ^= 1,
            crate::input::Action::FireWeapon if cockpit => {
                if let Some(station) = state.station {
                    cmds.push(CrewCmd::Fire { station });
                }
            }
            _ => {}
        }
    }
    if let Some(s) = state.station
        && stores.get(s as usize).is_none_or(|&(w, n)| w < 0 || n <= 0)
    {
        // That station empty: another of its kind, else the next kind left.
        let kind = stores.get(s as usize).map_or(-1, |w| w.0);
        let same = stores.iter().position(|&(w, n)| w >= 0 && w == kind && n > 0);
        state.station = same.map(|i| i as u8).or_else(|| next(Some(s))).filter(|_| cockpit);
    }
    for PodDesignate(x, y) in designate.read() {
        cmds.push(CrewCmd::Designate { at: Some((*x, *y)), unit: None });
    }
    for PodTrack(k) in track.read() {
        cmds.push(CrewCmd::Designate { at: None, unit: u16::try_from(*k).ok() });
    }
    for LockRequest(target) in locks.read() {
        match *target {
            ContactRef::Point(x, y) => {
                cmds.push(CrewCmd::Designate { at: Some((x, y)), unit: None })
            }
            ContactRef::Object(i) => {
                if let Some(o) = combat.as_deref().and_then(|c| c.sim.objects().get(i)) {
                    cmds.push(CrewCmd::Designate { at: Some((o.x, o.y)), unit: None });
                }
            }
            _ => {}
        }
    }
    let aim = GunnerAim {
        pod: crate::combat::pod_aim(&mfd.av.tgp),
        trigger: state.trigger,
        gun: state.gun,
        station: if cockpit { state.station } else { None },
    };
    state.since += time.delta_secs();
    if state.sent != Some(aim) || state.since >= AIM_REFRESH {
        state.sent = Some(aim);
        state.since = 0.0;
        cmds.push(CrewCmd::Aim(aim));
    }
    c.actions.extend(cmds.into_iter().map(crate::campaign::Action::Crew));
}

/// The station of the next kind of missile left after `from`'s (None, the chin gun, first and
/// after the last kind; `stores` without the gun pods).
fn next_station(from: Option<u8>, stores: &[(i8, i8); 6]) -> Option<u8> {
    let kind = |i: usize| stores[i].0;
    // Each kind left once, at its first station.
    let kinds: Vec<usize> = (0..stores.len())
        .filter(|&i| stores[i].0 >= 0 && stores[i].1 > 0)
        .filter(|&i| (0..i).all(|j| kind(j) != kind(i) || stores[j].1 <= 0))
        .collect();
    let at = from.and_then(|s| kinds.iter().position(|&i| kind(i) == kind(s as usize)));
    match at {
        None => kinds.first().map(|&i| i as u8),
        Some(k) => kinds.get(k + 1).map(|&i| i as u8),
    }
}

/// The console's buttons lit: the gun laid, the camera's FLIR.
fn panel(
    manning: Option<Res<Manning>>,
    mut state: ResMut<GunnerState>,
    combat: Option<Res<Combat>>,
    mfd: Res<crate::mfd::MfdState>,
    mut panel: ResMut<ConsolePanel>,
) {
    let Some(m) = manning.filter(|m| m.gunner() && m.console()) else { return };
    let gun = if m.remote_gunner() {
        state.gun
    } else {
        combat.as_deref().and_then(|c| c.sim.side_guns()).map_or(0, |g| g.gun)
    };
    state.shown_gun = gun;
    use crate::station::scope::{GREEN, YELLOW};
    let mut lit = vec![(ConsoleKey::Gun(gun + 1), YELLOW)];
    if mfd.flir {
        lit.push((ConsoleKey::Flir, GREEN));
    }
    let want = ConsolePanel { hidden: Vec::new(), lit };
    if *panel != want {
        *panel = want;
    }
}

/// The pod's picture's overlay on the big screen: the crosshair, the pod's track and zoom, the
/// gun laid, its rounds and whether it bears, the gimbal's scope and the way to slew the pod
/// into it.
#[allow(clippy::too_many_arguments)]
fn draw_screen(
    manning: Option<Res<Manning>>,
    mfd: Res<crate::mfd::MfdState>,
    cues: Res<crate::hud::HudCues>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
    time: Res<Time<Real>>,
    player: Query<&Aircraft, With<Player>>,
) {
    if !manning.is_some_and(|m| m.gunner() && m.console()) {
        return;
    }
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Eo) else {
        return;
    };
    use crate::station::scope::{GREEN, RED, WHITE, YELLOW};
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(crate::canvas::TRANSPARENT);
    let (w, h) = (cv.w as f32, cv.h as f32);
    let col = if mfd.flir { WHITE } else { GREEN };
    let text = |cv: &mut crate::canvas::Canvas, x: f32, y: f32, s: &str, col: u8, align| {
        stroke_text_at(cv, x, y, s, h * 0.032, 1.1, col, align);
    };
    let o = Vec2::new(w / 2.0, h / 2.0);
    let arm = h * 0.12;
    for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        cv.stroke(o + d * arm * 0.25, o + d * arm, 1.4, col);
    }
    // The pod's track and the picture.
    let track = match mfd.av.tgp.mode {
        TgpMode::Slaved => "SLV",
        TgpMode::Area(_) => "AREA",
        TgpMode::Point(_) if mfd.av.tgp.track.is_some() => "PNT VEH",
        TgpMode::Point(_) => "PNT",
        TgpMode::Snowplow => "SP",
    };
    text(cv, 10.0, 14.0, if mfd.flir { "FLIR WHOT" } else { "TV" }, col, Align::Left);
    text(cv, 10.0, 28.0, track, col, Align::Left);
    text(cv, w - 10.0, 14.0, crate::mfd::cam_zoom_label(mfd.cam_zoom), col, Align::Right);
    // The gun laid and its solution.
    let Some(g) = cues.side_guns else { return };
    let gun = stealth_sim::gunship::gun_name(g.gun as usize);
    let y = h - 16.0;
    text(cv, 10.0, y - 14.0, &format!("{gun}  {}", g.rounds[g.gun as usize]), YELLOW, Align::Left);
    let other = 1 - g.gun as usize;
    let rest = format!("{} {}", stealth_sim::gunship::gun_name(other), g.rounds[other]);
    text(cv, 10.0, y, &rest, col, Align::Left);
    let blink = (time.elapsed_secs() * 3.0) as i32 % 2 == 0;
    let (state, scol) = match (g.in_gimbal, g.tof) {
        (true, Some(t)) if g.ready => (format!("READY  TOF {t:.1}"), GREEN),
        (true, Some(t)) => (format!("LOADING  TOF {t:.1}"), YELLOW),
        (true, None) => ("IN GIMBAL".to_string(), GREEN),
        (false, _) => ("GIMBAL LIMIT".to_string(), RED),
    };
    if g.in_gimbal || blink {
        text(cv, w - 10.0, y - 14.0, &state, scol, Align::Right);
    }
    if g.range > 0.0 {
        text(cv, w - 10.0, y, &format!("RNG {:.1} KM", g.range / 1000.0), col, Align::Right);
    }
    // The gimbal's scope top right: the guns' reach and where the pod looks; out of it, an
    // arrow at the crosshair the way to slew the pod to bring it back in.
    let Some(los) = mfd.av.tgp.los else { return };
    let Ok(a) = player.single() else { return };
    let look = gimbal_look(a.orientation, los);
    // The reach green while the guns bear, red while the pod looks out of it.
    let bears = match (g.in_gimbal, look.inside) {
        (true, _) => GREEN,
        (false, false) => RED,
        (false, true) => YELLOW,
    };
    let (at, size) = (Vec2::new(w - 92.0, 24.0), Vec2::new(82.0, 70.0));
    draw_gimbal_scope(cv, at, size, &look, bears, col, blink);
    if let Some(d) = look.slew {
        let (from, to) = (o + d * arm * 1.25, o + d * arm * 2.3);
        cv.stroke(from, to, 2.0, RED);
        let back = -d * 9.0;
        let side = d.perp() * 6.0;
        cv.stroke(to, to + back + side, 2.0, RED);
        cv.stroke(to, to + back - side, 2.0, RED);
        if blink {
            text(cv, w / 2.0, o.y + arm * 2.6 + 10.0, "SLEW TO GIMBAL", RED, Align::Centre);
        }
    }
}

/// The pod's line of sight against the side guns' gimbal (`stealth_sim::gunship::GIMBAL_AZ`,
/// `GIMBAL_EL`): degrees forward of the left wing's line and below the wings' plane, whether
/// that is within the guns' reach, and if not the way to slew the pod (on its picture: x right,
/// y down, a unit) to the nearest line of sight that is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GimbalLook {
    pub forward: f32,
    pub depression: f32,
    pub inside: bool,
    pub slew: Option<Vec2>,
}

/// The guns' reach in degrees: either way of the wing's line, and the depression's limits.
fn gimbal_limits() -> (f32, f32, f32) {
    let (lo, hi) = stealth_sim::gunship::GIMBAL_EL;
    let az = stealth_sim::gunship::GIMBAL_AZ;
    (az.to_degrees() as f32, lo.to_degrees() as f32, hi.to_degrees() as f32)
}

/// Where the pod (world line of sight `los`) looks against the gimbal of an AC-130 at
/// `orientation`; the pod's picture is upright (`trackcam::aim`: world up at the top).
pub fn gimbal_look(orientation: Quat, los: Vec3) -> GimbalLook {
    let (az, dep) = crate::avionics::side_gun_angles(orientation, los);
    let forward = (az + 90.0 + 180.0).rem_euclid(360.0) - 180.0;
    let (lim, lo, hi) = gimbal_limits();
    let inside = forward.abs() <= lim && (lo..=hi).contains(&dep);
    let slew = (!inside)
        .then(|| {
            // The nearest line of sight in reach, in the world, seen on the picture.
            let (f, d) = (forward.clamp(-lim, lim).to_radians(), dep.clamp(lo, hi).to_radians());
            let az = f - std::f32::consts::FRAC_PI_2;
            let body = Vec3::new(az.sin() * d.cos(), -d.sin(), -az.cos() * d.cos());
            let to = orientation * body;
            let right = los.cross(Vec3::Y).try_normalize()?;
            let up = right.cross(los);
            Vec2::new(to.dot(right), -to.dot(up)).try_normalize()
        })
        .flatten();
    GimbalLook { forward, depression: dep, inside, slew }
}

/// The gimbal's scope at `at` (top left), `size` across: a frame for 35 degrees either way of
/// the left wing's line (forward on the right, as the pod's picture out of the left side) and
/// 0 to 70 degrees down; the guns' reach in it (`bears` while they bear), the wing's line
/// dashed, and the pod's line of sight a cross (at the frame's edge, blinking, when farther
/// off), red out of reach.
fn draw_gimbal_scope(
    cv: &mut crate::canvas::Canvas,
    at: Vec2,
    size: Vec2,
    look: &GimbalLook,
    bears: u8,
    col: u8,
    blink: bool,
) {
    use crate::station::scope::{GREEN, RED};
    let (span, dep_max) = (35.0, 70.0);
    let (lim, lo, hi) = gimbal_limits();
    let point = |f: f32, d: f32| {
        at + Vec2::new(size.x / 2.0 + f / span * (size.x / 2.0), d / dep_max * size.y)
    };
    cv.frame(at, at + size, 1.0, col);
    cv.frame(point(-lim, lo), point(lim, hi), 1.6, bears);
    let mut y = at.y;
    while y < at.y + size.y {
        let x = at.x + size.x / 2.0;
        cv.stroke(Vec2::new(x, y), Vec2::new(x, (y + 3.0).min(at.y + size.y)), 1.0, col);
        y += 6.0;
    }
    let shown = (look.forward.abs() <= span && (0.0..=dep_max).contains(&look.depression)) || blink;
    if shown {
        let m = point(look.forward.clamp(-span, span), look.depression.clamp(0.0, dep_max));
        let c = if look.inside { GREEN } else { RED };
        for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            cv.stroke(m + d * 2.0, m + d * 7.0, 1.6, c);
        }
    }
    let label = |cv: &mut crate::canvas::Canvas, x: f32, s: &str| {
        stroke_text_at(cv, x, at.y + size.y + 11.0, s, 7.0, 1.0, col, Align::Centre);
    };
    label(cv, at.x + 9.0, "AFT");
    label(cv, at.x + size.x - 9.0, "FWD");
}

/// The console's small screen: who flies the aircraft, the orbit, the guns.
fn draw_info(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    cues: Res<crate::hud::HudCues>,
    player: Query<&Aircraft, With<Player>>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat)) = (manning.filter(|m| m.gunner() && m.console()), combat) else {
        return;
    };
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Info) else {
        return;
    };
    use crate::station::scope::{BLACK, GREEN, YELLOW};
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let (w, h) = (cv.w as i32, cv.h as i32);
    cv.rect(1, 1, w - 2, h - 2, YELLOW);
    let line = |cv: &mut crate::canvas::Canvas, row: i32, s: &str, col: u8| {
        stroke_text_at(cv, 12.0, 22.0 + row as f32 * 26.0, s, 9.0, 1.3, col, Align::Left);
    };
    line(cv, 0, "AC-130 GUNNER", YELLOW);
    let name = |id: u8| {
        campaign
            .as_deref()
            .and_then(|c| c.state().player_name(stealth_campaign::PlayerId(id)))
            .unwrap_or("")
            .to_ascii_uppercase()
    };
    let _ = &combat;
    let pilot = if m.remote_gunner() { name(m.unit as u8) } else { "AI (ORBIT)".to_string() };
    line(cv, 1, &format!("PILOT  {pilot}"), GREEN);
    if let Ok(a) = player.single() {
        line(cv, 2, &format!("ALT    {:.0} FT", a.altitude), GREEN);
        line(cv, 3, &format!("SPEED  {:.0} KT", a.knots()), GREEN);
        if a.orbit.is_some() {
            line(cv, 4, "ORBIT  LEFT", GREEN);
        }
    }
    if let Some(g) = cues.side_guns {
        line(cv, 5, &format!("40MM   {}", g.rounds[0]), if g.gun == 0 { YELLOW } else { GREEN });
        line(cv, 6, &format!("105MM  {}", g.rounds[1]), if g.gun == 1 { YELLOW } else { GREEN });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line of sight `forward` degrees ahead of the left wing's line and `down` below the
    /// wings, from a level aircraft heading north (Bevy: forward -Z, the left wing -X).
    fn los(forward: f32, down: f32) -> Vec3 {
        let (az, d) = ((forward - 90.0).to_radians(), down.to_radians());
        Vec3::new(az.sin() * d.cos(), -d.sin(), -az.cos() * d.cos())
    }

    #[test]
    fn gimbal_look_shows_the_way_into_the_guns_reach() {
        let level = Quat::IDENTITY;
        let inside = gimbal_look(level, los(5.0, 30.0));
        assert!(inside.inside && inside.slew.is_none(), "{inside:?}");
        assert!((inside.forward - 5.0).abs() < 0.1 && (inside.depression - 30.0).abs() < 0.1);
        // Too far forward: the nose is to the picture's right, so slew left (aft).
        let ahead = gimbal_look(level, los(40.0, 30.0)).slew.unwrap();
        assert!(ahead.x < -0.9, "{ahead}");
        let aft = gimbal_look(level, los(-40.0, 30.0)).slew.unwrap();
        assert!(aft.x > 0.9, "{aft}");
        // Too steep: up on the picture (y down); too shallow: down.
        let steep = gimbal_look(level, los(0.0, 75.0)).slew.unwrap();
        assert!(steep.y < -0.9, "{steep}");
        let shallow = gimbal_look(level, los(0.0, 1.0)).slew.unwrap();
        assert!(shallow.y > 0.9, "{shallow}");
        // Off the right side: round to the left wing.
        let right = gimbal_look(level, los(180.0, 30.0));
        assert!(!right.inside && right.forward.abs() > 170.0, "{right:?}");
    }

    #[test]
    fn gimbal_look_is_in_the_body_frame() {
        // Banked 30 degrees left: a line of sight 30 degrees below the horizon off the left
        // wing is level with the wings, out of reach (above it).
        let bank = Quat::from_rotation_z(30f32.to_radians());
        let look = gimbal_look(bank, los(0.0, 30.0));
        assert!(look.depression.abs() < 0.5, "{look:?}");
        assert!(!look.inside && look.slew.unwrap().y > 0.9, "{look:?}");
    }
}
