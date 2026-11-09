//! Realtime dynamic campaign: the campaign model (crate stealth-campaign) driving one long
//! flight session with the simulation in campaign mode (`stealth_sim::campaign`), through the
//! campaign host (crate stealth-host).
//!
//! * The 3D world stays loaded for the whole campaign. Between sorties the player is absent
//!   from the simulation and the Ops screen (mission board, war map, players, debrief, log;
//!   `ops.rs`) is up over the cockpit, parked at a friendly base; the war goes on meanwhile.
//! * A sortie respawns the aircraft on the chosen base's runway with the mission's stores and
//!   route, in the aircraft chosen in the briefing. Crashing, ejecting (once the chute is down)
//!   or ending the sortie from the in-flight menu brings up the debrief and the board again;
//!   landing at a friendly base offers to rearm.
//! * The campaign clock is real time; the pause menu does not stop it. The sky runs
//!   `sky_ratio` times faster.
//!
//! Who runs the war ([`Role`]):
//! * Alone (`Local`) and as a listen server (`Host`): the game holds the [`Host`]; it is saved
//!   (`campaigns/<name>.json` in the player's folder) every minute and on leaving. Only the
//!   campaign is saved, not the simulation: on loading, what was in the air starts again. A
//!   listen server also serves the players who joined (`crate::net`).
//! * A coop client (`Client`): the server runs the war; the game shows its copy of the
//!   campaign, sends the player's commands and flies his aircraft.
//!
//! The Ops screen asks for things through [`Action`]s, carried out here for every role.

mod air_defence;
mod designate;
mod logistics;
mod menu;
mod ops;
pub(crate) mod picture;
pub mod unit_view;

pub use menu::CampaignWindow;
pub(crate) use menu::{SAVES_HEIGHT, confirm_delete, delete_popup};
pub use unit_view::{UnitCamera, UnitView};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use stealth_campaign::{
    Aircraft as Airframe, CampaignEvent, CampaignSetup, CampaignState, MissionKind, PlaceId,
    PlayerId, Side, Sortie,
};
use stealth_host::{Debrief, Host, HostEvent, Rules, Stores, save};
use stealth_i18n::tf;
use stealth_mission::Mission;
use stealth_net::protocol::{PlayerInfo, PlayerStatus, ToServer};
use stealth_sim::station::StationCmd;
use stealth_sim::{PlayerState, SimEvent, oflags};

pub use stealth_host::glue::describe;
use stealth_host::glue::sortie_mission;

use crate::combat::{Combat, CombatSystems};
use crate::flight::{Aircraft, FlightConstants, FlightEnd, Player, bam};
use crate::hud::{HudMessages, Waypoint, Waypoints};
use crate::mission::{ActiveMission, MissionData, PlannedRoute};
use crate::net::{NetInbox, NetLink};
use crate::settings::AppState;
use crate::world::{TileOverrides, WorldPos, wu_to_bevy};

/// The local player alone or on a listen server (a client gets his id from the server).
pub const HOST_PLAYER: PlayerId = PlayerId(0);
/// Seconds the end of a sortie stays in view before the debrief.
const END_SECONDS: f32 = 4.0;
/// Seconds between autosaves.
const SAVE_EVERY: f32 = 60.0;

pub struct CampaignPlugin;

impl Plugin for CampaignPlugin {
    fn build(&self, app: &mut App) {
        let running =
            |c: Option<Res<ActiveCampaign>>, k: Option<Res<Combat>>| c.is_some() && k.is_some();
        app.add_systems(
            Update,
            (begin, tick, ops::block_input)
                .chain()
                .after(CombatSystems)
                .run_if(in_state(AppState::Flight))
                .run_if(running),
        )
        .add_systems(Update, ops::overlay.run_if(in_state(AppState::Flight)))
        .add_systems(
            Update,
            crate::planner::prepare_map
                .run_if(in_state(AppState::Flight))
                .run_if(|c: Option<Res<ActiveCampaign>>| c.is_some_and(|c| c.screen_open())),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ops::ops_ui.run_if(in_state(AppState::Flight)).run_if(running),
        )
        // `--open-campaign`: start in the campaign window (screenshots).
        .insert_resource(CampaignWindow::opened(std::env::args().any(|a| a == "--open-campaign")))
        .init_resource::<UnitView>()
        .init_resource::<crate::camera::HideOwnAircraft>()
        .add_systems(
            Update,
            unit_view::follow
                .after(crate::sim_render::SimRenderSystems)
                .run_if(in_state(AppState::Flight)),
        )
        .add_systems(
            Update,
            unit_view::infrared_background
                .after(unit_view::follow)
                .run_if(in_state(AppState::Flight)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            unit_view::window
                .after(ops::ops_ui)
                .run_if(in_state(AppState::Flight))
                .run_if(|c: Option<Res<ActiveCampaign>>| c.is_some_and(|c| c.screen_open())),
        )
        // Before the menu's OPTIONS window, so the Escape that closes that does not close this.
        .add_systems(
            EguiPrimaryContextPass,
            menu::campaign_window.before(crate::menu::menu_ui).run_if(in_state(AppState::Menu)),
        )
        .add_systems(OnExit(AppState::Flight), leave);
        #[cfg(feature = "xr")]
        app.add_systems(
            Update,
            unit_view::vr_panel.after(unit_view::follow).run_if(in_state(AppState::Flight)),
        );
    }
}

/// What the player is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// On the Ops screen (absent from the simulation).
    Ops,
    /// Flying a sortie.
    Sortie,
    /// At the console of a friendly Patriot or NASAMS battery or Gepard (AAA unit; absent from the
    /// simulation as on the Ops screen).
    Station { unit: usize },
}

/// Who runs the war.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Alone.
    Local,
    /// A listen server: the war is run here, for the players who joined too.
    Host,
    /// A coop client: the server runs the war.
    Client,
}

/// What the Ops screen asks for.
#[derive(Debug, Clone)]
pub enum Action {
    /// In the air: take the urgent mission the AWACS diverts the pilot to.
    Retask {
        offer: u32,
    },
    TakeMission {
        offer: u32,
        base: Option<u16>,
        aircraft: Airframe,
        loadout: crate::loadout::PlannedLoadout,
        route: Option<PlannedRoute>,
    },
    OrderInsertion {
        place: PlaceId,
    },
    /// Call in AI SEAD for an offer's place.
    RequestSupport {
        offer: u32,
    },
    /// Designate a new site (a Patriot or NASAMS battery, a Gepard, a drone command center) with its
    /// helipad at (x, y), map units.
    DesignateSite {
        x: i32,
        y: i32,
        kind: stealth_campaign::DefenceKind,
    },
    /// Build new site `id` at once for materiel.
    FastTrackSite {
        id: u32,
    },
    /// The crate of load `leg` of new site `id` hooked under the helicopter (see `sling.rs`).
    HookCrate {
        id: u32,
        leg: u8,
    },
    /// It is set down at (`east`, `north`) metres from the site's helipad, heading `heading`.
    DropCrate {
        id: u32,
        leg: u8,
        east: f32,
        north: f32,
        heading: f32,
    },
    /// The sortie ends (the simulation's outcome codes: 0 landed and done, 7 aborted).
    EndSortie {
        outcome: u8,
    },
    Rearm {
        loadout: crate::loadout::PlannedLoadout,
    },
    /// Plan a mission of the board with the other players on it (None: plan none).
    SelectMission {
        offer: Option<u32>,
        base: u16,
    },
    /// The plan's base and route changed.
    EditPlan {
        offer: u32,
        base: u16,
        route: Vec<crate::mission::RoutePoint>,
    },
    /// Man a friendly air defence console, leave it, or fight it.
    Station(StationCmd),
    /// Take over aircraft `tag` of a friendly AI flight in the air, with the stores its
    /// mission's briefing gives.
    TakeControl {
        tag: u32,
        loadout: Option<crate::loadout::PlannedLoadout>,
    },
    /// In the air: leave the aircraft to the AI.
    LeaveAircraft,
}

/// The last sortie's debrief.
pub type SortieDebrief = Debrief;

/// The campaign being played (the flight session is its world).
#[derive(Resource)]
pub struct ActiveCampaign {
    pub name: String,
    pub role: Role,
    /// The war's authority (alone, listen server); None on a client.
    pub(super) host: Option<Host>,
    /// A client's copy of the server's campaign.
    pub(super) copy: Option<CampaignState>,
    /// The campaign's own mission (its world is the theater's), the base of the sorties'.
    pub mission: Mission,
    pub rules: Rules,
    /// This player.
    pub me: PlayerId,
    /// The server's name (a client's).
    pub server: Option<String>,
    pub phase: Phase,
    pub ui: ops::OpsUi,
    pub debrief: Option<SortieDebrief>,
    /// Landed and stopped at this friendly base: rearm, or end the sortie.
    pub landed: Option<usize>,
    /// Base the player is parked at between sorties.
    pub parked: Option<usize>,
    /// The aircraft of the next sortie (the briefing's choice).
    pub aircraft: Airframe,
    /// The players (a coop game's).
    pub roster: Vec<PlayerInfo>,
    /// Asked for by the Ops screen, carried out next frame.
    pub actions: Vec<Action>,
    /// A returning client's sortie goes on (from the server's welcome).
    pub resume: Option<stealth_net::protocol::Resume>,
    /// The sortie's callsign numbers (flight, position) from the host; None before one.
    pub callsign: Option<(u8, u8)>,
    started: bool,
    save_timer: f32,
    /// Seconds since the sortie ended (the debrief comes up after `END_SECONDS`).
    end_timer: Option<f32>,
    /// Seconds the flight model has ended the flight without the simulation following (an
    /// end asked for: negative).
    crash_timer: f32,
    /// The urgent mission the AWACS called a divert to (accepted with `AcceptTasking`).
    divert: Option<u32>,
    /// Leave the campaign (back to the menu) at the next frame.
    pub exit: bool,
    /// The stores and the route chosen in the briefing for the sortie being taken.
    pub take_loadout: Option<crate::loadout::PlannedLoadout>,
    pub take_route: Option<PlannedRoute>,
}

impl ActiveCampaign {
    /// A new campaign alone on a theater.
    pub fn new(
        name: String,
        setup: CampaignSetup,
        data: &MissionData,
        rules: Rules,
    ) -> anyhow::Result<Self> {
        let host = Host::new(setup, &data.0, rules)?;
        Ok(Self::with_host(name, host, Role::Local))
    }

    /// A saved campaign, alone.
    pub fn resume(
        name: String,
        state: CampaignState,
        data: &MissionData,
        rules: Rules,
    ) -> anyhow::Result<Self> {
        let host = Host::resume(state, &data.0, rules)?;
        Ok(Self::with_host(name, host, Role::Local))
    }

    /// A campaign alone or as a listen server.
    pub fn with_host(name: String, host: Host, role: Role) -> Self {
        let mission = host.mission.clone();
        let rules = host.rules;
        let mut c = Self::blank(name, mission, rules, role, HOST_PLAYER);
        c.host = Some(host);
        c
    }

    /// A coop client's campaign: the server's copy.
    pub fn client(
        server: String,
        state: CampaignState,
        data: &MissionData,
        rules: Rules,
        me: PlayerId,
    ) -> anyhow::Result<Self> {
        let mission = stealth_host::glue::base_mission(&state.setup, &data.0)?;
        let mut c = Self::blank(server.clone(), mission, rules, Role::Client, me);
        c.copy = Some(state);
        c.server = Some(server);
        Ok(c)
    }

    fn blank(name: String, mission: Mission, rules: Rules, role: Role, me: PlayerId) -> Self {
        Self {
            name,
            role,
            host: None,
            copy: None,
            mission,
            rules,
            me,
            server: None,
            phase: Phase::Ops,
            ui: ops::OpsUi::default(),
            debrief: None,
            landed: None,
            parked: None,
            aircraft: Airframe::F117,
            roster: Vec::new(),
            actions: Vec::new(),
            resume: None,
            callsign: None,
            started: false,
            save_timer: 0.0,
            end_timer: None,
            crash_timer: 0.0,
            divert: None,
            exit: false,
            take_loadout: None,
            take_route: None,
        }
    }

    /// The campaign as this player knows it.
    pub fn state(&self) -> &CampaignState {
        match (&self.host, &self.copy) {
            (Some(h), _) => &h.state,
            (None, Some(s)) => s,
            (None, None) => unreachable!("a campaign has a host or a copy"),
        }
    }

    /// The war's authority (alone, listen server); None on a client.
    pub fn host_mut(&mut self) -> Option<&mut Host> {
        self.host.as_mut()
    }

    /// The Ops screen (or the landing prompt) is up.
    pub fn screen_open(&self) -> bool {
        (self.phase == Phase::Ops && self.end_timer.is_none()) || self.landed.is_some()
    }

    /// On the Ops board (not flying a sortie nor at a console): the in-flight menu's END
    /// MISSION is the board's way out (`exit_id`, `ask_exit`).
    pub fn on_board(&self) -> bool {
        self.phase == Phase::Ops
    }

    /// The board's way out, SAVE & EXIT or DISCONNECT from a server: its message id and icon.
    pub fn exit_id(&self) -> &'static str {
        ops::exit_id(self.role)
    }

    pub fn exit_icon(&self) -> char {
        ops::exit_icon(self.role)
    }

    /// Asks to leave the campaign, as the board's way-out button does.
    pub fn ask_exit(&mut self) {
        self.ui.confirm_exit = true;
    }

    /// Saves the campaign (alone or as the listen server).
    pub fn save(&self) {
        if let Some(h) = &self.host
            && let Err(e) = save::save(&self.name, &h.state)
        {
            warn!("saving campaign {}: {e:#}", self.name);
        }
    }

    /// The players as the PLAYERS tab shows them (alone: just this one).
    pub fn players(&self) -> Vec<PlayerInfo> {
        if !self.roster.is_empty() {
            return self.roster.clone();
        }
        let st = self.state();
        let status = match st.sortie(self.me) {
            Some(s) if self.phase == Phase::Sortie => {
                PlayerStatus::Flying { title: s.offer.title.clone(), aircraft: s.aircraft }
            }
            _ => PlayerStatus::Ops,
        };
        vec![PlayerInfo {
            id: self.me,
            name: st.player_name(self.me).unwrap_or("Pilot").to_string(),
            status,
            ping: 0,
            host: self.role == Role::Host,
            rate: 0,
        }]
    }
}

/// Starts a campaign flight session (from the menu or the command line).
pub fn start(
    commands: &mut Commands,
    campaign: ActiveCampaign,
    setup: &mut crate::settings::MissionSetup,
    start_dir: &[i16],
) {
    let m = campaign.mission.clone();
    *setup = crate::mission::setup_from_mission(&m, start_dir);
    commands.insert_resource(ActiveMission(m.clone()));
    commands.insert_resource(PlannedRoute::of(&m));
    commands.remove_resource::<crate::career::CareerFlight>();
    commands.remove_resource::<crate::loadout::PlannedLoadout>();
    commands.insert_resource(campaign);
}

/// The rules of a campaign alone or served from this game: the game's own settings.
pub fn rules_of(s: &crate::settings::Settings, realistic_weapons: bool) -> Rules {
    Rules {
        realistic_damage: s.realistic_damage,
        realistic_guidance: s.realistic_guidance,
        no_crashes: s.no_crashes,
        continuous_time: s.continuous_time,
        realistic_weapons,
        friendly_fire: false,
        classic_terrain: !s.realistic_terrain,
        classic_ai_flight: !s.realistic_ai_flight,
        airframe_losses: false,
        no_ai_missions: false,
    }
}

/// The local pilot's name for the campaign (the multiplayer screen's).
fn pilot_name() -> String {
    let p = crate::prefs::Prefs::load();
    let n = p.player_name.trim();
    if n.is_empty() { "Pilot".to_string() } else { n.to_string() }
}

/// `--campaign <name>`: straight into a saved campaign (or a new one under that name, in
/// `--theater <code>` with `--roe`, `--opponents`, `--sky-ratio` and `--start-hour`).
pub fn from_args(app: &mut App) {
    let args: Vec<String> = std::env::args().collect();
    let get = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    let Some(name) = get("--campaign") else { return };
    let data = MissionData::load().expect("mission data");
    let tables = crate::flight::SimTables::load();
    let rules =
        rules_of(&crate::settings::Settings::from_args(), args.iter().any(|a| a == "--lockheed"));
    let saved = save::load(&name).unwrap_or_else(|e| {
        warn!("campaign {name}: {e:#}");
        None
    });
    let c = match saved {
        Some(state) => ActiveCampaign::resume(name, state, &data, rules),
        None => {
            let code =
                get("--theater").unwrap_or_else(|| "central_europe".into()).to_ascii_lowercase();
            let setup = CampaignSetup {
                theater: crate::mission::theater_index(&code).unwrap_or(3),
                theater_code: code,
                roe: get("--roe").and_then(|s| s.parse().ok()).unwrap_or(1),
                opponents: get("--opponents").and_then(|s| s.parse().ok()).unwrap_or(1),
                seed: get("--seed").and_then(|s| s.parse().ok()).unwrap_or_else(rand::random),
                sky_ratio: get("--sky-ratio").and_then(|s| s.parse().ok()).unwrap_or(24.0),
                start_hour: get("--start-hour").and_then(|s| s.parse().ok()).unwrap_or(8.0),
                dynamic_weather: !args.iter().any(|a| a == "--clear-sky"),
                crew_scaling: true,
            };
            ActiveCampaign::new(name, setup, &data, rules)
        }
    };
    let c = match c {
        Ok(c) => c,
        Err(e) => {
            warn!("campaign: {e:#}");
            return;
        }
    };
    let m = c.mission.clone();
    let mut setup = crate::mission::setup_from_mission(&m, &tables.start_dir);
    setup.freecam = false;
    app.insert_resource(setup)
        .insert_resource(PlannedRoute::of(&m))
        .insert_resource(ActiveMission(m))
        .insert_resource(data)
        .insert_resource(c)
        .insert_state(AppState::Flight);
}

/// The flight session's parts a sortie start or end touches.
#[derive(SystemParam)]
pub struct Session<'w, 's> {
    commands: Commands<'w, 's>,
    combat: ResMut<'w, Combat>,
    player: Query<'w, 's, (&'static mut Aircraft, &'static mut WorldPos), With<Player>>,
    constants: ResMut<'w, FlightConstants>,
    settings: ResMut<'w, crate::settings::Settings>,
    wps: ResMut<'w, Waypoints>,
    orders: ResMut<'w, crate::mfd::MissionOrders>,
    kneeboard: ResMut<'w, crate::kneeboard::Kneeboard>,
    msgs: ResMut<'w, HudMessages>,
    tiles: ResMut<'w, TileOverrides>,
    theater: Option<ResMut<'w, crate::world::Theater>>,
    view: ResMut<'w, crate::camera::ViewState>,
    /// The campaign clock: virtual time (never paused in a campaign; `--time-scale` speeds the
    /// whole war up for tests), and the wall clock (the air defence's refills).
    time: Res<'w, Time>,
    real: Res<'w, Time<Real>>,
    menu: ResMut<'w, crate::flight_menu::FlightMenu>,
    link: Option<ResMut<'w, NetLink>>,
    inbox: ResMut<'w, NetInbox>,
    next: ResMut<'w, NextState<AppState>>,
    notice: ResMut<'w, crate::net::Notice>,
    players: ResMut<'w, crate::player_notices::PlayerNotices>,
    atc: Option<ResMut<'w, crate::atc::AtcState>>,
    input: Res<'w, crate::hotas::InputConfig>,
}

/// First frame of the session: the simulation is made to match the campaign.
fn begin(mut c: ResMut<ActiveCampaign>, mut s: Session) {
    if c.started {
        return;
    }
    c.started = true;
    let c = &mut *c;
    c.aircraft = airframe_of(s.settings.aircraft);
    match c.role {
        Role::Client => s.combat.sim.make_mirror(c.me.0),
        Role::Local | Role::Host => {
            let host = c.host.as_mut().expect("host");
            host.state.rename(HOST_PLAYER, &pilot_name());
            let events = host.begin(&mut s.combat.sim);
            if let Some(NetLink::Host(server)) = s.link.as_deref_mut() {
                server.route(&events);
            }
            handle(c, &mut s, events);
            drain_sim(c, &mut s);
        }
    }
    if let Some(t) = s.theater.as_deref_mut() {
        mark_friendly(t, &s.combat.sim);
    }
    let base = stealth_host::glue::default_base(c.state());
    park(c, &mut s, base);
    c.ui.tab = ops::test_tab().unwrap_or(ops::Tab::Board);
    // A returning client's sortie goes on where his aircraft is.
    if let Some(r) = c.resume.take() {
        begin_sortie(c, &mut s, &r.sortie, &r.state);
    }
    // Test aids: `--convoy-now` sends both sides' armour columns at once, `--convoy-speed
    // <m/s>` drives them faster.
    if let Some(host) = c.host.as_mut() {
        if std::env::args().any(|a| a == "--convoy-now") {
            host.state.convoys_due();
        }
        if let Some(v) = std::env::args()
            .skip_while(|a| a != "--convoy-speed")
            .nth(1)
            .and_then(|v| v.parse::<f32>().ok())
        {
            s.combat.sim.set_convoy_speed(v);
        }
    }
    // Test aid: `--campaign-insert` orders an insertion against the nearest enemy place.
    if std::env::args().any(|a| a == "--campaign-insert")
        && let Some(&place) = c.state().front(Side::Blue).first()
    {
        c.actions.push(Action::OrderInsertion { place });
    }
}

/// The flight model's landing corridors follow the places' owners (friendly bases are the
/// ones it lands on), as the simulation has them.
fn mark_friendly(t: &mut crate::world::Theater, sim: &stealth_sim::Sim) {
    for (i, so) in sim.objects().iter().enumerate().skip(3) {
        if so.flags & (oflags::AIRFIELD | oflags::CARRIER) == 0 {
            continue;
        }
        let Some(o) = t.world.objects.get_mut(i) else { continue };
        if (o.x as i32, o.y as i32) != (so.x, so.y) {
            continue;
        }
        let mut f = o.target_flags as u16 & !0x500;
        f |= (so.flags & (oflags::FRIENDLY | oflags::NEUTRAL)) as u16;
        o.target_flags = f as i16;
    }
}

/// Events the host's calls raised in the simulation (outside its steps): tiles destroyed and
/// repaired, rearming; on a listen server the players get them too.
fn drain_sim(c: &mut ActiveCampaign, s: &mut Session) {
    let Some(host) = c.host.as_mut() else { return };
    let events = host.take_sim_events(&mut s.combat.sim);
    if let Some(NetLink::Host(server)) = s.link.as_deref_mut() {
        server.after_step(&events);
    }
    for e in events {
        if e.event.is_private() && PlayerId(e.pilot) != c.me {
            continue;
        }
        match e.event {
            SimEvent::TileDestroyed(t) => {
                s.tiles.set(t.level as usize, t.col, t.row, t.index, t.shape)
            }
            SimEvent::TileRestored(t) => s.tiles.clear(t.level as usize, t.col, t.row, t.index),
            SimEvent::Resupplied => {
                let fuel = s.constants.fuel;
                if let Ok((mut a, _)) = s.player.single_mut() {
                    a.fuel = fuel;
                    a.ended = None;
                    a.has_flown = false;
                }
            }
            _ => {}
        }
    }
}

/// The player waits at a base (absent from the simulation; the aircraft stands on the runway).
fn park(c: &mut ActiveCampaign, s: &mut Session, base: Option<usize>) {
    c.phase = Phase::Ops;
    c.landed = None;
    c.end_timer = None;
    c.crash_timer = 0.0;
    let base = base.or(c.parked);
    c.parked = base;
    s.combat.campaign_events.clear();
    // The aircraft waiting unseen is an F-117A whatever the last sortie flew (a helicopter
    // would idle there with its rotor turning); the sortie puts the chosen one there.
    s.settings.aircraft = crate::aircraft::AircraftKind::F117;
    *s.constants = crate::flight::constants_for(s.settings.aircraft, c.rules.no_crashes);
    if let Some(b) = base {
        let st = s.combat.sim.start_state_at(b);
        place_player(s, &st, b);
    }
    s.view.view = crate::camera::View::Cockpit;
}

/// Puts the aircraft at a state: fresh on the runway, or in the air where a returning
/// player's aircraft is.
fn place_player(s: &mut Session, st: &PlayerState, base: usize) {
    let carrier = s.combat.sim.objects().get(base).is_some_and(|o| o.flags & oflags::CARRIER != 0);
    let fuel = s.constants.fuel;
    let Ok((mut a, mut pos)) = s.player.single_mut() else { return };
    let p = wu_to_bevy(st.x as f64, st.y as f64, if carrier { 128.0 } else { 0.0 });
    *a = Aircraft::new_on_runway(fuel);
    a.altitude = (p.y / crate::world::WU) as f32;
    a.orientation = Aircraft::from_euler(bam(st.heading as f32), 0.0, 0.0);
    if !st.on_ground {
        a.altitude = st.altitude as f32;
        a.orientation =
            Aircraft::from_euler(bam(st.heading as f32), bam(st.pitch as f32), bam(st.roll as f32));
        a.on_ground = false;
        a.gear_down = !st.gear_up;
        a.velocity = st.velocity as f32;
        a.has_flown = true;
        // Cruise power (the start state has no throttle).
        a.thrust = 85.0;
        a.set_thrust = 85.0;
    }
    pos.0 = p;
}

/// One frame of the campaign.
fn tick(mut c: ResMut<ActiveCampaign>, mut s: Session) {
    let c = &mut *c;
    let dt = s.time.delta_secs();
    if c.exit {
        return;
    }
    // END SORTIE from the in-flight menu; down waiting for the rescue helicopter, SKIP
    // EXTRACTION (the rescue goes on the board). At an air defence console: leave it, back to
    // operations.
    if std::mem::take(&mut s.combat.quit) {
        match c.phase {
            Phase::Sortie => {
                let outcome = if s.combat.sim.csar().is_some() { 0 } else { 7 };
                c.actions.push(Action::EndSortie { outcome });
            }
            Phase::Station { .. } => c.actions.push(Action::Station(StationCmd::Leave)),
            _ => {}
        }
    }
    divert_call(c, &mut s);
    // LEAVE AIRCRAFT in the flight menu: in the air on a sortie that is not ending.
    let flying = s.player.single().is_ok_and(|(a, _)| !a.on_ground && a.ended.is_none());
    s.menu.can_leave = c.phase == Phase::Sortie && c.end_timer.is_none() && flying;
    // The flight model ended the flight (its ground model differs in detail) and the
    // simulation did not follow: the sortie ends anyway.
    if c.phase == Phase::Sortie && c.end_timer.is_none() && !s.combat.sim.player_absent() {
        let ended = s.player.single().ok().and_then(|(a, _)| a.ended);
        match ended {
            Some(FlightEnd::Crash | FlightEnd::BadLanding) if s.combat.sim.chute().is_none() => {
                if c.crash_timer >= 0.0 {
                    c.crash_timer += dt;
                }
                if c.crash_timer > 3.0 {
                    c.crash_timer = -1.0;
                    let outcome = if ended == Some(FlightEnd::BadLanding) { 5 } else { 1 };
                    c.actions.push(Action::EndSortie { outcome });
                }
            }
            // Stopped on a runway the simulation does not count as a landing: the prompt.
            Some(FlightEnd::Landed) if c.landed.is_none() => {
                c.crash_timer += dt;
                if c.crash_timer > 3.0 {
                    let p = s.combat.sim.player();
                    c.landed = c.state().nearest_player_base(p.mx, p.my).map(|b| b as usize);
                }
            }
            _ => c.crash_timer = 0.0,
        }
    }
    // Test aid: take a mission of the board at once.
    if c.phase == Phase::Ops
        && c.state().clock > 2.0
        && !c.ui.test_taken
        && let Some(n) = ops::test_take()
    {
        let flown: Vec<(u32, MissionKind)> = c
            .state()
            .board
            .iter()
            .filter(|o| o.kind.flyable_by(c.aircraft))
            .map(|o| (o.id, o.kind))
            .collect();
        // A kind of mission (`logistics`) waits for its first offer.
        let pick = match n {
            ops::TestTake::Index(n) => flown.get(n.min(flown.len().saturating_sub(1))),
            ops::TestTake::Kind(k) => flown.iter().find(|o| o.1.name().eq_ignore_ascii_case(&k)),
        };
        c.ui.test_taken = pick.is_some() || flown.is_empty();
        if let Some(&(offer, _)) = pick {
            let loadout = briefed_loadout(c, offer, None, c.aircraft);
            if let Some(loadout) = loadout {
                let aircraft = c.aircraft;
                c.actions.push(Action::TakeMission {
                    offer,
                    base: None,
                    aircraft,
                    loadout,
                    route: None,
                });
            }
        }
    }
    // Test aids: `--campaign-ai-now` makes the boards' offers due for the AI at once (a host's or
    // a single player's campaign); `--campaign-take-control [secs]` takes over the first
    // friendly AI aircraft up that long (default 60); `--campaign-leave [secs]` leaves the
    // aircraft to the AI that long into the sortie (default 20).
    let arg = |flag: &str, default: f64| {
        let mut a = std::env::args().skip_while(|a| a != flag);
        a.next().map(|_| a.next().and_then(|v| v.parse().ok()).unwrap_or(default))
    };
    if std::env::args().any(|a| a == "--campaign-ai-now")
        && let Some(h) = c.host.as_mut()
    {
        let (now, wait) = (h.state.clock, stealth_campaign::tuning::AI_MISSION_WAIT);
        for o in h.state.board.iter_mut().chain(h.state.red_board.iter_mut()) {
            o.posted = o.posted.min(now - wait);
        }
    }
    if c.phase == Phase::Ops
        && !c.ui.test_controlled
        && let Some(after) = arg("--campaign-take-control", 60.0)
    {
        let now = c.state().clock;
        let tag = c
            .state()
            .ai_missions
            .iter()
            .filter(|m| m.side == stealth_campaign::Side::Blue && now - m.launched >= after)
            .filter(|m| m.open())
            .find_map(|m| m.takeable());
        if let Some(tag) = tag.filter(|&t| s.combat.sim.takeable(t)) {
            c.ui.test_controlled = true;
            c.actions.push(Action::TakeControl { tag, loadout: control_loadout(c, tag) });
        }
    }
    if c.phase == Phase::Sortie
        && !c.ui.test_left
        && s.menu.can_leave
        && let Some(after) = arg("--campaign-leave", 20.0)
        && c.state().sortie(c.me).is_some_and(|s| c.state().clock - s.started >= after)
    {
        c.ui.test_left = true;
        c.actions.push(Action::LeaveAircraft);
    }
    // Test aid: `--campaign-drones` puts a drone command center at the friendly place nearest
    // the enemy's air defences and mans it (a host's or a single player's campaign);
    // `--campaign-drones front` 8 km short of the nearest enemy air defence unit;
    // `--campaign-drones front <n>` puts n of them there, side by side (one for each player).
    if c.phase == Phase::Ops
        && c.state().clock > 2.0
        && !c.ui.test_manned
        && c.role != Role::Client
        && std::env::args().any(|a| a == "--campaign-drones")
    {
        let sim = &mut s.combat.sim;
        let st = c.state();
        let enemy: Vec<(i32, i32)> = (0..sim.units().len())
            .filter(|&k| !sim.aaa_blue(k) && !sim.units()[k].destroyed)
            .map(|k| (sim.units()[k].x, sim.units()[k].y))
            .collect();
        let gap = |p: &stealth_campaign::Place| {
            enemy.iter().map(|&(x, y)| stealth_campaign::dist(p.x - x, p.y - y)).min().unwrap_or(0)
        };
        let place = st
            .places
            .iter()
            .filter(|p| p.owner == stealth_campaign::Side::Blue)
            .min_by_key(|p| gap(p))
            .map(|p| (p.x, p.y));
        let front = std::env::args().skip_while(|a| a != "--campaign-drones").nth(1);
        let place = match (place, front.as_deref()) {
            (Some((px, py)), Some("front")) => enemy
                .iter()
                .min_by_key(|&&(x, y)| stealth_campaign::dist(px - x, py - y))
                .map(|&(x, y)| {
                    let d = ((px - x) as f64).hypot((py - y) as f64).max(1.0);
                    let k = 8000.0 / stealth_sim::aaa::M_PER_MAP / d;
                    (x + ((px - x) as f64 * k) as i32, y + ((py - y) as f64 * k) as i32)
                }),
            (p, _) => p.map(|(x, y)| (x + 0x30, y - 0x30)),
        };
        let count: usize = std::env::args()
            .skip_while(|a| a != "--campaign-drones")
            .nth(2)
            .and_then(|n| n.parse().ok())
            .unwrap_or(1);
        if let Some((x, y)) = place {
            for i in 0..count.max(1) as i32 {
                sim.place_drone_center(x + i * 0x20, y);
            }
            let k = sim.units().len() - count.max(1);
            c.ui.test_manned = true;
            c.actions.push(Action::Station(StationCmd::Man { unit: k as u16 }));
        }
    }
    // Test aid: `--fp2-strike [m]` (with `--convoy-now`): once the enemy column has driven so
    // far (0), a drone command center 3.5 km off its road ahead launches an attack drone at
    // where its Shilka will be when the drone gets there.
    if c.phase == Phase::Ops
        && c.state().clock > 20.0
        && !c.ui.test_strike
        && c.role != Role::Client
        && std::env::args().any(|a| a == "--fp2-strike")
    {
        use stealth_sim::aaa::{GroundKind, M_PER_MAP};
        use stealth_sim::uav::{UavKind, UavWp, WpKind};
        let sim = &mut s.combat.sim;
        let red = sim.convoys().iter().find(|v| !v.blue).cloned();
        let zsu = red.as_ref().and_then(|v| {
            v.units.iter().position(|&k| sim.units()[k as usize].kind == GroundKind::Zsu)
        });
        let after: f64 = std::env::args()
            .skip_while(|a| a != "--fp2-strike")
            .nth(1)
            .and_then(|m| m.parse().ok())
            .unwrap_or(0.0);
        if let (Some(v), Some(i)) = (red, zsu)
            && v.along >= after
        {
            const OFF: f64 = 3500.0;
            let along = v.along - i as f64 * stealth_sim::ground::GAP;
            let speed = sim.convoy_speed();
            let fp2 = UavKind::Fp2.perf().cruise;
            // The side of the road further from the enemy's other guns.
            let guns: Vec<[f64; 2]> = (0..sim.units().len())
                .filter(|&k| !sim.aaa_blue(k) && !sim.units()[k].destroyed && !sim.unit_moving(k))
                .map(|k| {
                    let m = sim.units()[k].metres();
                    [m[0], m[1]]
                })
                .collect();
            let clear = |p: [f64; 2]| {
                guns.iter().map(|g| (g[0] - p[0]).hypot(g[1] - p[1])).fold(f64::MAX, f64::min)
            };
            let mut ahead = 1500.0;
            let (mut aim, mut center) = ([0.0; 2], [0.0; 2]);
            for _ in 0..8 {
                let (p, h) = v.point(along + ahead);
                aim = p;
                let side = |s: f64| [p[0] + s * h.cos() * OFF, p[1] - s * h.sin() * OFF];
                center = if clear(side(1.0)) >= clear(side(-1.0)) { side(1.0) } else { side(-1.0) };
                // Its dive is faster than its cruise.
                ahead = speed * (OFF / fp2 - 4.0);
            }
            let map = |p: [f64; 2]| ((p[0] / M_PER_MAP) as i32, 0x8000 - (p[1] / M_PER_MAP) as i32);
            let ((cx, cy), (ax, ay)) = (map(center), map(aim));
            sim.place_drone_center(cx, cy);
            let k = sim.units().len() - 1;
            let alt = UavKind::Fp2.perf().altitude as f32;
            let route = vec![UavWp { x: ax, y: ay, alt, kind: WpKind::Terminate }];
            sim.test_launch_uav(k, UavKind::Fp2, route);
            info!(
                "test: FP-2 launched at {:.0} s from ({cx}, {cy}) at ({ax}, {ay}), {:.0} m from a gun",
                c.state().clock,
                clear(center)
            );
            c.ui.test_strike = true;
        }
    }
    // Test aid: `--campaign-geran <n> [km]` (with `--campaign-drones`): once an interceptor is
    // up and level, a raid of n enemy Geran-2s km (2) ahead of its nose at its height, from the
    // enemy place nearest there and flying at the friendly place behind the interceptor.
    if matches!(c.phase, Phase::Station { .. })
        && c.ui.test_manned
        && !c.ui.test_geran
        && c.role != Role::Client
        && let Some(n) = std::env::args()
            .skip_while(|a| a != "--campaign-geran")
            .nth(1)
            .and_then(|n| n.parse::<u32>().ok())
        && let Some(u) = s.combat.sim.uavs().iter().find(|u| {
            u.kind == stealth_sim::uav::UavKind::Interceptor
                && u.mode == stealth_sim::uav::UavMode::Auto
                && u.pos[2] > 150.0
                && u.pitch.abs() < 0.1
        })
    {
        c.ui.test_geran = true;
        let km: f64 = std::env::args()
            .skip_while(|a| a != "--campaign-geran")
            .nth(2)
            .and_then(|n| n.parse().ok())
            .unwrap_or(2.0);
        // Map units along the interceptor's nose (y runs south).
        let (x0, y0) = u.map();
        let ahead = |m: f64| {
            let d = m / stealth_sim::aaa::M_PER_MAP;
            (x0 + (u.heading.sin() * d) as i32, y0 - (u.heading.cos() * d) as i32)
        };
        let st = c.state();
        let nearest = |side: stealth_campaign::Side, (x, y): (i32, i32)| {
            st.places
                .iter()
                .filter(|p| p.owner == side && !p.members.is_empty())
                .min_by_key(|p| stealth_campaign::dist(p.x - x, p.y - y))
                .map(|p| p.airfield.unwrap_or(p.members[0]) as usize)
        };
        let (start, behind) = (ahead(km * 1000.0), ahead(-10_000.0));
        let alt = (u.pos[2] * 32.0 / stealth_sim::aaa::M_PER_MAP) as i32;
        if let Some(from) = nearest(stealth_campaign::Side::Red, start)
            && let Some(to) = nearest(stealth_campaign::Side::Blue, behind)
        {
            let sim = &mut s.combat.sim;
            for i in 0..n {
                let j = sim.spawn_drone(from, to, 0xF000 + i, false);
                let (x, y) = (start.0 + 0x8 * i as i32, start.1);
                let f = &mut sim.flights_mut()[j];
                (f.wx, f.wy) = (x << 5, y << 5);
                (f.x, f.y) = (x, y);
                f.alt = alt;
            }
        }
    }
    // Test aid: man the n-th friendly air defence console (Patriot batteries first, then the
    // NASAMS batteries), or with `--campaign-man drone` (`nasams`) the first drone command
    // center (NASAMS battery) nobody mans.
    // (A client waits for the server's units.)
    if c.phase == Phase::Ops
        && c.state().clock > 2.0
        && !c.ui.test_manned
        && let Some(n) = ops::test_man()
    {
        let sim = &s.combat.sim;
        use stealth_sim::aaa::GroundKind;
        let only = match n {
            ops::TEST_MAN_DRONES => Some(GroundKind::DroneCenter),
            ops::TEST_MAN_NASAMS => Some(GroundKind::NasamsRadar),
            _ => None,
        };
        let mut units: Vec<usize> = (0..sim.units().len())
            .filter(|&k| sim.aaa_blue(k) && stealth_sim::station::mannable(sim.units()[k].kind))
            .filter(|&k| {
                only.is_none_or(|kind| sim.units()[k].kind == kind && sim.manned_by(k).is_none())
            })
            .collect();
        units.sort_by_key(|&k| match sim.units()[k].kind {
            GroundKind::PatriotRadar => 0,
            GroundKind::NasamsRadar => 1,
            _ => 2,
        });
        let n = if only.is_some() { 0 } else { n };
        if let Some(&k) = units.get(n.min(units.len().saturating_sub(1))) {
            c.ui.test_manned = true;
            c.actions.push(Action::Station(StationCmd::Man { unit: k as u16 }));
        }
    }
    let events = match c.role {
        Role::Client => client_frame(c, &mut s, dt),
        Role::Local | Role::Host => host_frame(c, &mut s, dt),
    };
    handle(c, &mut s, events);
    drain_sim(c, &mut s);
    station_phase(c, &s);
    // The debrief comes up a few seconds after the end (the chute is down by then).
    if let Some(t) = c.end_timer.as_mut() {
        *t += dt;
        if *t > END_SECONDS {
            let base = c.parked;
            park(c, &mut s, base);
            c.ui.tab = ops::test_tab().unwrap_or(ops::Tab::Debrief);
        }
    }
    // The sky's clock.
    let day = 86_400.0 * 30.0;
    s.combat.sky_seconds = Some(((c.state().sky_hours() * 3600.0) % day) as f32);
    s.combat.sky_ratio = c.state().setup.sky_ratio;
    // Saved now and then.
    c.save_timer += dt;
    if c.save_timer > SAVE_EVERY {
        c.save_timer = 0.0;
        c.save();
    }
}

/// At an air defence console or not, as the simulation (the server's, on a client) has it; a
/// console whose radar or gun is lost sends the player back to the Ops screen.
fn station_phase(c: &mut ActiveCampaign, s: &Session) {
    let sim = &s.combat.sim;
    let manned = sim.station_of(c.me.0).map(|st| st.unit as usize);
    match (c.phase, manned) {
        (Phase::Ops, Some(unit)) => c.phase = Phase::Station { unit },
        (Phase::Station { unit }, None) => {
            c.phase = Phase::Ops;
            if let Some(u) = sim.units().get(unit).filter(|u| u.destroyed) {
                c.ui.error = Some(stealth_i18n::text!("ops-unit-destroyed", unit = u.kind.name()));
            }
        }
        (Phase::Station { unit }, Some(now)) if unit != now => {
            c.phase = Phase::Station { unit: now }
        }
        _ => {}
    }
}

/// The stores the briefing would give a mission of the board.
pub fn briefed_loadout(
    c: &ActiveCampaign,
    offer: u32,
    base: Option<u16>,
    aircraft: Airframe,
) -> Option<crate::loadout::PlannedLoadout> {
    let o = c.state().board.iter().find(|o| o.id == offer)?.clone();
    let m = sortie_mission(&c.mission, c.state(), &preview(&o, base.unwrap_or(o.base), aircraft));
    Some(crate::loadout::PlannedLoadout::of(&m, kind_of(aircraft)))
}

/// The stores the briefing gives the mission of the AI aircraft `tag` (as on its pylons).
pub fn control_loadout(c: &ActiveCampaign, tag: u32) -> Option<crate::loadout::PlannedLoadout> {
    let m = c.state().ai_missions.iter().find(|m| m.open() && m.tags.contains(&tag))?;
    let stealth_campaign::AiAirframe::Blue(aircraft) = m.airframe else { return None };
    let m = sortie_mission(&c.mission, c.state(), &preview(&m.offer, m.from, aircraft));
    Some(crate::loadout::PlannedLoadout::of(&m, kind_of(aircraft)))
}

/// A sortie not taken yet (for its briefing).
pub fn preview(o: &stealth_campaign::MissionOffer, base: u16, aircraft: Airframe) -> Sortie {
    Sortie {
        id: 0,
        offer: o.clone(),
        base,
        aircraft,
        started: 0.0,
        progress: default(),
        flight: 0,
        position: 0,
    }
}

/// The campaign is run here: the Ops screen's actions, the players' commands (listen
/// server), the campaign's frame.
fn host_frame(c: &mut ActiveCampaign, s: &mut Session, dt: f32) -> Vec<HostEvent> {
    let me = c.me;
    let mut out = Vec::new();
    let host = c.host.as_mut().expect("host");
    let combat = &mut *s.combat;
    let sim = &mut combat.sim;
    let mut server = match s.link.as_deref_mut() {
        Some(NetLink::Host(server)) => Some(server),
        _ => None,
    };
    // The players who joined: their messages.
    if let Some(server) = server.as_deref_mut() {
        out.extend(server.pump(host, sim, web_time::Instant::now()));
    }
    for a in std::mem::take(&mut c.actions) {
        let result = match a {
            Action::TakeMission { offer, base, aircraft, loadout, route } => {
                let stores = Stores { stations: loadout.stations(), select: 0 };
                c.take_loadout = Some(loadout);
                c.take_route = route;
                host.take_mission(sim, me, offer, base, aircraft, Some(stores))
            }
            Action::OrderInsertion { place } => host.order_insertion(sim, me, place),
            Action::RequestSupport { offer } => host.request_support(sim, me, offer),
            Action::DesignateSite { x, y, kind } => host.designate_site(sim, me, x, y, kind),
            Action::FastTrackSite { id } => host.fast_track_site(sim, me, id),
            Action::HookCrate { id, leg } => host.hook_crate(sim, me, id, leg),
            Action::DropCrate { id, leg, east, north, heading } => {
                host.drop_crate(sim, me, id, leg, east, north, heading)
            }
            Action::EndSortie { outcome } => Ok(host.end_sortie(sim, me, outcome)),
            Action::Retask { offer } => host.retask(sim, me, offer),
            Action::Rearm { loadout } => {
                let stores = Stores { stations: loadout.stations(), select: 0 };
                host.rearm(sim, me, &stores).map(|_| {
                    s.msgs.show(tf!("hud-rearmed"));
                    c.landed = None;
                    Vec::new()
                })
            }
            Action::SelectMission { offer, base } => host.select(sim, me, offer, base),
            Action::EditPlan { offer, base, route } => host.edit_plan(sim, me, offer, base, route),
            Action::Station(cmd) => host.station(sim, me, cmd),
            Action::TakeControl { tag, loadout } => {
                let stores = loadout.map(|l| {
                    let stores = Stores { stations: l.stations(), select: 0 };
                    c.take_loadout = Some(l);
                    stores
                });
                host.take_control(sim, me, tag, stores)
            }
            Action::LeaveAircraft => host.leave_aircraft(sim, me),
        };
        match result {
            Ok(ev) => {
                c.ui.error = None;
                if let Some(server) = server.as_deref_mut() {
                    server.route(&ev);
                }
                out.extend(ev);
            }
            Err(e) => c.ui.error = Some(e),
        }
    }
    // The simulation's events of this frame's steps.
    let events = std::mem::take(&mut combat.campaign_events);
    let ev = host.tick(sim, dt as f64, &events);
    host.refills(sim, s.real.delta_secs());
    if let Some(server) = server {
        server.route(&ev);
        if combat.steps > 0 {
            server.send_frame(host, sim, web_time::Instant::now());
        }
        let roster = server.roster(host, sim);
        s.players.roster_changed(&c.roster, &roster, me);
        c.roster = roster;
    }
    out.extend(ev);
    out
}

/// The server runs the campaign: the Ops screen's actions go to it, its news come back.
fn client_frame(c: &mut ActiveCampaign, s: &mut Session, dt: f32) -> Vec<HostEvent> {
    let Some(NetLink::Client(client)) = s.link.as_deref_mut() else {
        return Vec::new();
    };
    for a in std::mem::take(&mut c.actions) {
        let msg = match a {
            Action::TakeMission { offer, base, aircraft, loadout, route } => {
                let stores = Stores { stations: loadout.stations(), select: 0 };
                c.take_loadout = Some(loadout);
                c.take_route = route;
                ToServer::TakeMission { offer, base, aircraft, stores: Some(stores) }
            }
            Action::OrderInsertion { place } => ToServer::OrderInsertion { place },
            Action::RequestSupport { offer } => ToServer::RequestSupport { offer },
            Action::DesignateSite { x, y, kind } => ToServer::DesignateSite { x, y, kind },
            Action::FastTrackSite { id } => ToServer::FastTrackSite { id },
            Action::HookCrate { id, leg } => ToServer::HookCrate { id, leg },
            Action::DropCrate { id, leg, east, north, heading } => {
                ToServer::DropCrate { id, leg, east, north, heading }
            }
            Action::EndSortie { outcome } => ToServer::EndSortie { outcome },
            Action::Retask { offer } => ToServer::Retask { offer },
            Action::Rearm { loadout } => {
                s.msgs.show(tf!("hud-rearmed"));
                c.landed = None;
                let fuel = s.constants.fuel;
                if let Ok((mut a, _)) = s.player.single_mut() {
                    a.fuel = fuel;
                    a.ended = None;
                    a.has_flown = false;
                }
                ToServer::Rearm { stores: Stores { stations: loadout.stations(), select: 0 } }
            }
            Action::SelectMission { offer, base } => ToServer::Select { offer, base },
            Action::EditPlan { offer, base, route } => ToServer::Plan { offer, base, route },
            Action::Station(cmd) => ToServer::Station(cmd),
            Action::TakeControl { tag, loadout } => {
                let stores = loadout.map(|l| {
                    let stores = Stores { stations: l.stations(), select: 0 };
                    c.take_loadout = Some(l);
                    stores
                });
                ToServer::TakeControl { tag, stores }
            }
            Action::LeaveAircraft => ToServer::LeaveAircraft,
        };
        client.send(&msg);
    }
    // The copy's clock runs on between the server's updates.
    if let Some(st) = c.copy.as_mut() {
        st.clock += dt as f64;
    }
    let mut out = Vec::new();
    for e in std::mem::take(&mut s.inbox.campaign) {
        use stealth_net::client::ClientEvent as E;
        match e {
            E::Host(ev) => out.push(ev),
            E::Campaign(st) => {
                let clock = c.copy.as_ref().map_or(0.0, |s| s.clock);
                let mut st = *st;
                // The Ops screen's clock does not jump back.
                st.clock = st.clock.max(clock - 1.0);
                c.copy = Some(st);
            }
            E::Error(e) => c.ui.error = Some(e),
            E::Roster(r) => {
                s.players.roster_changed(&c.roster, &r, c.me);
                c.roster = r;
            }
            E::Closed(why) => {
                s.notice.0 = Some(stealth_i18n::t!("mp-disconnected", why = why.tr()));
                c.exit = true;
                s.next.set(AppState::Menu);
            }
            _ => {}
        }
    }
    out
}

/// The campaign's airframe of a game aircraft, and back.
pub fn airframe_of(k: crate::aircraft::AircraftKind) -> Airframe {
    match k {
        crate::aircraft::AircraftKind::F117 => Airframe::F117,
        crate::aircraft::AircraftKind::F16 => Airframe::F16,
        crate::aircraft::AircraftKind::UH60 => Airframe::UH60,
        crate::aircraft::AircraftKind::AH1Z => Airframe::AH1Z,
        crate::aircraft::AircraftKind::Tornado => Airframe::Tornado,
        crate::aircraft::AircraftKind::A10 => Airframe::A10,
        crate::aircraft::AircraftKind::RAH66 => Airframe::RAH66,
        crate::aircraft::AircraftKind::OH58 => Airframe::OH58,
        crate::aircraft::AircraftKind::CH47 => Airframe::CH47,
        crate::aircraft::AircraftKind::AV8B => Airframe::AV8B,
        crate::aircraft::AircraftKind::F22 => Airframe::F22,
        crate::aircraft::AircraftKind::C5 => Airframe::C5,
    }
}

pub fn kind_of(a: Airframe) -> crate::aircraft::AircraftKind {
    match a {
        Airframe::F117 => crate::aircraft::AircraftKind::F117,
        Airframe::F16 => crate::aircraft::AircraftKind::F16,
        Airframe::UH60 => crate::aircraft::AircraftKind::UH60,
        Airframe::AH1Z => crate::aircraft::AircraftKind::AH1Z,
        Airframe::Tornado => crate::aircraft::AircraftKind::Tornado,
        Airframe::A10 => crate::aircraft::AircraftKind::A10,
        Airframe::RAH66 => crate::aircraft::AircraftKind::RAH66,
        Airframe::OH58 => crate::aircraft::AircraftKind::OH58,
        Airframe::CH47 => crate::aircraft::AircraftKind::CH47,
        Airframe::AV8B => crate::aircraft::AircraftKind::AV8B,
        Airframe::F22 => crate::aircraft::AircraftKind::F22,
        Airframe::C5 => crate::aircraft::AircraftKind::C5,
    }
}

/// The host's events for this player and everybody.
fn handle(c: &mut ActiveCampaign, s: &mut Session, events: Vec<HostEvent>) {
    let me = c.me;
    for e in events {
        match e {
            HostEvent::SortieStarted { player, sortie, start } if player == me => {
                begin_sortie(c, s, &sortie, &start);
            }
            HostEvent::Campaign(CampaignEvent::SortieRetasked { player, sortie })
                if player == me =>
            {
                retask_sortie(c, s, &sortie);
            }
            HostEvent::SortieFinished { player, debrief } if player == me => {
                s.msgs.show(debrief.title.tr_flight());
                // Down under the parachute: the debrief at once (the pilotless aircraft is
                // nothing to watch); else the end stays in view a few seconds.
                use stealth_campaign::SortieEnd;
                let ejected = matches!(
                    debrief.record.end,
                    SortieEnd::Ejected | SortieEnd::Rescued | SortieEnd::Captured
                );
                c.debrief = Some(debrief);
                c.landed = None;
                c.end_timer = Some(if ejected { END_SECONDS } else { 0.0 });
            }
            HostEvent::Landed { player, base } if player == me && c.phase == Phase::Sortie => {
                c.landed = Some(base);
                s.msgs.show(tf!("hud-landed-rearm"));
            }
            HostEvent::Campaign(CampaignEvent::Message(text)) => {
                info!("campaign: {}", text.tr_in(stealth_i18n::Language::En));
                s.msgs.show(text.tr_flight());
            }
            HostEvent::Campaign(CampaignEvent::ObjectiveAchieved { player, .. })
                if player == me =>
            {
                s.msgs.show(tf!("hud-objective-achieved"));
                s.orders.done[0] = true;
            }
            // A client's copy has the plans at once (the host's state has them already).
            HostEvent::Campaign(CampaignEvent::PlansChanged(plans)) => {
                if let Some(st) = c.copy.as_mut() {
                    st.plans = plans;
                }
            }
            HostEvent::Campaign(CampaignEvent::PlaceCaptured { .. }) => {
                if let Some(t) = s.theater.as_deref_mut() {
                    mark_friendly(t, &s.combat.sim);
                }
            }
            _ => {}
        }
    }
}

/// A sortie starts: the aircraft (the chosen type) on the base's runway with the mission's
/// stores and route.
/// The AWACS calls a divert when an urgent mission comes up near a pilot in the air (once per
/// mission); the pilot takes it with `AcceptTasking`.
fn divert_call(c: &mut ActiveCampaign, s: &mut Session) {
    let accept = std::mem::take(&mut s.combat.accept_tasking);
    if c.phase != Phase::Sortie || s.combat.sim.player_absent() {
        c.divert = None;
        return;
    }
    let offer = c.state().divert_for(c.me).map(|o| (o.id, o.title.clone()));
    if offer.as_ref().map(|o| o.0) != c.divert {
        c.divert = offer.as_ref().map(|o| o.0);
        if let Some((_, title)) = offer {
            let key = s
                .input
                .keys_for(crate::hotas::ButtonTarget::Action(crate::input::Action::AcceptTasking))
                .next()
                .map_or(String::new(), |k| crate::input::key_label(k.mods, k.key));
            let me = s.atc.as_ref().and_then(|a| a.callsign().map(str::to_string));
            let me = me.unwrap_or_default();
            // Radio calls are in English, as the controllers' are.
            let what = title.tr_in(stealth_i18n::Language::En);
            let call = format!("{me}, {AWACS_STATION}, divert: {what}, how copy");
            let spoken = s.atc.as_mut().is_some_and(|a| a.radio(AWACS_STATION, call.clone(), None));
            if !spoken {
                s.players.chat(AWACS_STATION, &call, false);
            }
            s.msgs.show(tf!("hud-divert-accept", title = title.tr_flight(), key = key));
        }
    }
    if accept && let Some(offer) = c.divert {
        c.actions.push(Action::Retask { offer });
        let me = s.atc.as_ref().and_then(|a| a.callsign().map(str::to_string)).unwrap_or_default();
        if let Some(atc) = s.atc.as_mut() {
            let ack = format!("{AWACS_STATION}, {me}, wilco, diverting");
            atc.radio(AWACS_STATION, String::new(), Some(ack));
        }
    }
}

/// The AWACS's callsign on the radio.
const AWACS_STATION: &str = "Magic";

/// The host retasked this player's sortie: the new mission's orders, route and briefing (the
/// aircraft, its stores and where it flies stay).
fn retask_sortie(c: &ActiveCampaign, s: &mut Session, sortie: &Sortie) {
    let m = sortie_mission(&c.mission, c.state(), sortie);
    let route = PlannedRoute::of(&m);
    let mut wps = Waypoints::planned(
        route
            .points
            .iter()
            .map(|p| Waypoint { east: p.x as f64 * 32.0, north: (0x8000 - p.y) as f64 * 32.0 })
            .collect(),
        route.anchors(),
    );
    // Straight on to the target.
    let target =
        route.anchors().get(1).copied().unwrap_or(0).min(wps.points.len().saturating_sub(1));
    wps.current = target;
    wps.mission_current = target;
    *s.wps = wps;
    s.orders.set(
        m.orders.lines.iter().map(|l| (l.color, crate::mission::clean_text(&l.text))).collect(),
    );
    s.commands.insert_resource(route);
    s.commands.insert_resource(ActiveMission(m));
    s.kneeboard.new_mission();
    // The divert stays the one called (a client's copy has the old sortie a moment longer).
    s.msgs.show(sortie.offer.title.tr_flight());
}

fn begin_sortie(c: &mut ActiveCampaign, s: &mut Session, sortie: &Sortie, start: &PlayerState) {
    let base = sortie.base as usize;
    let m = sortie_mission(&c.mission, c.state(), sortie);
    let aircraft = kind_of(sortie.aircraft);
    c.aircraft = sortie.aircraft;
    c.callsign = (sortie.flight > 0).then_some((sortie.flight, sortie.position.max(1)));
    // Another aircraft than the last sortie's: its cockpit, model and flight model.
    if s.settings.aircraft != aircraft {
        s.settings.aircraft = aircraft;
    }
    *s.constants = crate::flight::constants_for(aircraft, c.rules.no_crashes);
    let loadout = c
        .take_loadout
        .take()
        .filter(|l| l.fits(&m, aircraft))
        .unwrap_or_else(|| crate::loadout::PlannedLoadout::of(&m, aircraft));
    let route = c.take_route.take().filter(|r| r.fits(&m)).unwrap_or_else(|| PlannedRoute::of(&m));
    *s.wps = Waypoints::planned(
        route
            .points
            .iter()
            .map(|p| Waypoint { east: p.x as f64 * 32.0, north: (0x8000 - p.y) as f64 * 32.0 })
            .collect(),
        route.anchors(),
    );
    s.orders.set(
        m.orders.lines.iter().map(|l| (l.color, crate::mission::clean_text(&l.text))).collect(),
    );
    s.commands.insert_resource(route);
    s.commands.insert_resource(loadout);
    s.commands.insert_resource(ActiveMission(m));
    s.kneeboard.new_mission();
    s.combat.record = default();
    s.combat.campaign_events.clear();
    place_player(s, start, base);
    s.view.view = crate::camera::View::Cockpit;
    s.menu.open = false;
    c.phase = Phase::Sortie;
    c.parked = Some(base);
    c.landed = None;
    c.end_timer = None;
    c.crash_timer = 0.0;
    s.msgs.show(sortie.offer.title.tr_flight());
}

/// Leaving the flight session: the campaign is saved and closed (a client leaves the server,
/// a listen server stops), and the Ops screen's egui context goes with it (the menu brings
/// its own).
#[allow(clippy::too_many_arguments)]
fn leave(
    mut commands: Commands,
    c: Option<Res<ActiveCampaign>>,
    mut settings: ResMut<crate::settings::Settings>,
    cams: Query<Entity, With<ops::OpsCamera>>,
    mut view: ResMut<UnitView>,
    mut sites: ResMut<crate::world::StreamSites>,
    link: Option<ResMut<NetLink>>,
) {
    for e in &cams {
        commands.entity(e).try_despawn();
    }
    view.close();
    unit_view::follow_cleanup(&mut commands, &mut view);
    commands.insert_resource(crate::camera::HideOwnAircraft(false));
    sites.follow = None;
    #[cfg(feature = "xr")]
    commands.remove_resource::<crate::vr_menu::WorldPanel>();
    if let Some(mut link) = link {
        match &mut *link {
            NetLink::Client(client) => client.leave(),
            NetLink::Host(server) => server.shutdown(stealth_i18n::text!("net-host-left")),
        }
        commands.remove_resource::<NetLink>();
    }
    if let Some(c) = c {
        // The menu keeps the campaign's aircraft choice, not the parked F-117A.
        settings.aircraft = kind_of(c.aircraft);
        c.save();
        commands.remove_resource::<ActiveCampaign>();
    }
}
