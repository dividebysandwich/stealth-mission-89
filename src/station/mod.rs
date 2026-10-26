//! At the console of a friendly Patriot battery or Gepard (campaign, `Phase::Station`): the
//! player sits in the dark shelter of the battery's engagement control station
//! (`cockpit_console.glb`, a cockpit like the aircraft's: buttons, screens, VR fingertips).
//!
//! * The air picture (`scope`): the tracks around the site in APP-6 symbols with heading lines
//!   and history dots, refreshed twice a second (the radar's picture, not a live one).
//! * The board (`board`): the track list (a track clicked there or on the picture is selected
//!   on both), or the site's launchers, stocks and refills.
//! * The camera (`eo`): the locked track live (not at the picture's rate), in colour or FLIR.
//!
//! The console's commands go to the simulation through the campaign (`Action::Station`, the
//! server's on a client); what the console shows comes from the simulation's state.

mod board;
mod eo;
mod scope;
mod sound;

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::aaa::AaaKind;
use stealth_sim::station::{Guidance, StationCmd};

use crate::camera::{View, ViewState};
use crate::campaign::{Action, ActiveCampaign, Phase};
use crate::cockpit3d::{ConsoleButton, ConsoleKey, ConsoleScreen, ScreenClick};
use crate::combat::Combat;
use crate::flight::{Aircraft, FlightSystems, Player};
use crate::settings::AppState;
use crate::world::WorldPos;

pub use eo::ConsoleEo;

pub struct StationPlugin;

impl Plugin for StationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleUi>().add_systems(Startup, sound::load).add_systems(
            Update,
            (
                sync_manning,
                seat_operator.after(FlightSystems),
                (
                    buttons,
                    scope::snapshot,
                    eo::aim,
                    scope::draw,
                    board::draw,
                    eo::draw,
                    sound::play,
                )
                    .chain()
                    .after(sync_manning),
            )
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// The player mans this AAA unit's console (a Patriot radar or a Gepard).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Manning {
    pub unit: usize,
    pub gepard: bool,
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
    /// The camera's zoom (0..6; None: set from the range on a new lock).
    pub zoom: Option<u8>,
    /// The radar's picture: the latest and earlier ones, newest first.
    pub snapshots: std::collections::VecDeque<scope::Snapshot>,
    pub since_snapshot: f32,
    /// A message (a refused command) and how long it shows yet (s).
    pub note: Option<(String, f32)>,
    /// Where the aircraft waited before, to put it back.
    parked: Option<(DVec3, Quat)>,
    /// `--console-fire` fired.
    test_fired: bool,
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
            test_fired: false,
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
fn sync_manning(
    mut commands: Commands,
    c: Option<Res<ActiveCampaign>>,
    combat: Option<Res<Combat>>,
    manning: Option<Res<Manning>>,
    mut ui: ResMut<ConsoleUi>,
    mut player: Query<(&mut WorldPos, &mut Aircraft), With<Player>>,
    mut mfd: ResMut<crate::mfd::MfdState>,
) {
    let want = match (c.as_deref().map(|c| c.phase), combat.as_deref()) {
        (Some(Phase::Station { unit }), Some(combat)) => {
            let gepard =
                combat.sim.aaa_units().get(unit).is_some_and(|u| u.kind == AaaKind::Gepard);
            Some(Manning { unit, gepard })
        }
        _ => None,
    };
    if want == manning.as_deref().copied() {
        return;
    }
    match want {
        Some(m) => {
            if manning.is_none()
                && let Ok((pos, a)) = player.single()
            {
                ui.parked = Some((pos.0, a.orientation));
            }
            let parked = ui.parked;
            // Test aid: `--console-tab site` opens the site's page.
            let site = std::env::args().skip_while(|a| a != "--console-tab").nth(1);
            *ui = ConsoleUi {
                range_km: if m.gepard { 10.0 } else { 40.0 },
                tab: if site.as_deref() == Some("site") { Tab::Site } else { Tab::Tracks },
                parked,
                ..Default::default()
            };
            mfd.flir = false;
            commands.insert_resource(m);
        }
        None => {
            if let Some((p, o)) = ui.parked.take()
                && let Ok((mut pos, mut a)) = player.single_mut()
            {
                pos.0 = p;
                a.orientation = o;
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
    let u = combat.sim.aaa_units().get(unit)?;
    let at = crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0);
    // The same spread of headings the units stand at (`sim_render`).
    let heading = (unit as f32 * 2.399_963).rem_euclid(std::f32::consts::TAU);
    let back = DVec3::new(-(heading as f64).sin(), 0.0, (heading as f64).cos()) * SHELTER_OFFSET;
    Some((at + back + DVec3::Y * EYE_HEIGHT, heading))
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

/// The console's buttons and screen clicks.
#[allow(clippy::too_many_arguments)]
fn buttons(
    manning: Option<Res<Manning>>,
    mut c: Option<ResMut<ActiveCampaign>>,
    combat: Option<Res<Combat>>,
    mut ui: ResMut<ConsoleUi>,
    mut keys: MessageReader<ConsoleButton>,
    mut clicks: MessageReader<ScreenClick>,
    mut mfd: ResMut<crate::mfd::MfdState>,
    time: Res<Time<Real>>,
) {
    let (Some(m), Some(c), Some(combat)) = (manning, c.as_deref_mut(), combat) else {
        keys.clear();
        clicks.clear();
        return;
    };
    let sim = &combat.sim;
    let launchers = if m.gepard { vec![m.unit] } else { sim.battery_launchers(m.unit) };
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
                None => ui.note = Some(("Select a track first".into(), 3.0)),
            },
            ConsoleKey::Drop => send(StationCmd::Lock(None)),
            ConsoleKey::Guided => send(StationCmd::Mode(Guidance::Guided)),
            ConsoleKey::Semi => send(StationCmd::Mode(Guidance::SemiActive)),
            ConsoleKey::Fire if m.gepard => send(StationCmd::Burst),
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
            ConsoleKey::ZoomIn => ui.zoom = Some(ui.zoom.unwrap_or(2).saturating_add(1).min(6)),
            ConsoleKey::ZoomOut => ui.zoom = Some(ui.zoom.unwrap_or(2).saturating_sub(1)),
            ConsoleKey::RangeIn => step_range(&mut ui, -1),
            ConsoleKey::RangeOut => step_range(&mut ui, 1),
            ConsoleKey::Leave => send(StationCmd::Leave),
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
                None => {}
            },
            ConsoleScreen::Eo => {}
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
        send(if m.gepard { StationCmd::Burst } else { StationCmd::Fire { launcher: None } });
    }
    // A refused command (the host's answer; a client's comes back from the server).
    if let Some(e) = c.ui.error.take() {
        ui.note = Some((e, 4.0));
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

fn step_range(ui: &mut ConsoleUi, d: i32) {
    let i = RANGES.iter().position(|&r| r >= ui.range_km).unwrap_or(RANGES.len() - 1) as i32;
    ui.range_km = RANGES[(i + d).clamp(0, RANGES.len() as i32 - 1) as usize];
}

/// Metres (east, north, up) of a flight unit.
fn flight_metres(u: &stealth_sim::FlightUnit) -> [f64; 3] {
    let m = stealth_sim::aaa::M_PER_MAP / 32.0;
    [u.wx as f64 * m, (0x10_0000 - u.wy) as f64 * m, u.alt.max(0) as f64 * m]
}
