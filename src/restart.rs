//! RESTART MISSION (the in-flight menu, asked first) and FLY AGAIN (the debrief): a single
//! mission flown again from its start, with the route, the loadout, the aircraft and the
//! realism switches it was flown with.
//!
//! Each single mission's start (not a career sortie, a campaign or a multiplayer game) is kept
//! when the flight is set up. A restart asked for in flight leaves the flight for the same
//! state (as LOAD MISSION does, `savegame.rs`): the flight is torn down, its mission with it, so
//! the kept start is made the mission to fly again once the flight is entered anew, before the
//! mission is looked for.

use bevy::prelude::*;
use stealth_mission::Mission;

use crate::flight_menu::FlightMenu;
use crate::loadout::PlannedLoadout;
use crate::mission::{ActiveMission, PlannedRoute};
use crate::settings::{AppState, MissionSetup, SetupSet};

pub struct RestartPlugin;

impl Plugin for RestartPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastStart>()
            .init_resource::<RestartRequest>()
            .add_systems(
                OnEnter(AppState::Flight),
                (enter_restart.before(SetupSet::Mission), keep_start.after(SetupSet::Spawn)),
            )
            .add_systems(Update, restart_in_flight.run_if(in_state(AppState::Flight)));
    }
}

/// The start of the single mission flown last: None after a career sortie, a campaign or a
/// multiplayer game.
#[derive(Resource, Default)]
pub struct LastStart(Option<Start>);

impl LastStart {
    /// FLY AGAIN can fly it.
    pub fn available(&self) -> bool {
        self.0.is_some()
    }
}

#[derive(Clone)]
struct Start {
    mission: Mission,
    route: PlannedRoute,
    loadout: PlannedLoadout,
}

/// The flight about to be entered is the kept start's (FLY AGAIN, RESTART MISSION).
#[derive(Resource, Default)]
pub struct RestartRequest(pub bool);

/// OnEnter(Flight), once the aircraft is in place: a single mission's start is kept.
fn keep_start(
    mut last: ResMut<LastStart>,
    mission: Option<Res<ActiveMission>>,
    route: Option<Res<PlannedRoute>>,
    loadout: Option<Res<PlannedLoadout>>,
    settings: Res<crate::settings::Settings>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    link: Option<Res<crate::net::NetLink>>,
    career: Option<Res<crate::career::CareerFlight>>,
) {
    let single = campaign.is_none() && link.is_none() && career.is_none();
    last.0 = match (mission, single) {
        (Some(m), true) => {
            let m = &m.0;
            Some(Start {
                mission: m.clone(),
                route: route
                    .filter(|r| r.fits(m))
                    .map_or_else(|| PlannedRoute::of(m), |r| r.clone()),
                loadout: loadout
                    .filter(|l| l.fits(m, settings.aircraft))
                    .map_or_else(|| PlannedLoadout::of(m, settings.aircraft), |l| l.clone()),
            })
        }
        _ => None,
    };
}

/// OnEnter(Flight), before the mission is looked for: the kept start made the mission to fly.
fn enter_restart(
    mut commands: Commands,
    mut request: ResMut<RestartRequest>,
    last: Res<LastStart>,
    mut setup: ResMut<MissionSetup>,
    tables: Res<crate::flight::SimTables>,
) {
    if !std::mem::take(&mut request.0) {
        return;
    }
    let Some(start) = last.0.clone() else {
        warn!("restart: no single mission to fly again");
        return;
    };
    fly(&mut commands, start, &mut setup, &tables.start_dir);
}

fn fly(commands: &mut Commands, start: Start, setup: &mut MissionSetup, start_dir: &[i16]) {
    *setup = crate::mission::setup_from_mission(&start.mission, start_dir);
    commands.insert_resource(ActiveMission(start.mission));
    commands.insert_resource(start.route);
    commands.insert_resource(start.loadout);
    commands.remove_resource::<crate::career::CareerFlight>();
}

/// RESTART MISSION confirmed in the in-flight menu: the same state again, the flight torn down
/// and set up from the kept start. Also says every frame whether it can.
fn restart_in_flight(
    mut menu: ResMut<FlightMenu>,
    last: Res<LastStart>,
    settings: Res<crate::settings::Settings>,
    mut request: ResMut<RestartRequest>,
    mut next: ResMut<NextState<AppState>>,
) {
    let can = last.available() && !menu.exiting;
    if menu.can_restart != can {
        menu.can_restart = can;
    }
    if !std::mem::take(&mut menu.restart_request) || !can {
        return;
    }
    crate::flight_menu::save_options(&mut menu, &settings);
    request.0 = true;
    next.set(AppState::Flight);
}

/// FLY AGAIN on the debrief.
pub fn fly_again(request: &mut RestartRequest, next: &mut NextState<AppState>) {
    request.0 = true;
    next.set(AppState::Flight);
}
