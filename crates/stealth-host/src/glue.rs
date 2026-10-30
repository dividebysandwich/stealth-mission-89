//! Between the campaign (stealth-campaign) and the simulation (stealth-sim): what the
//! simulation shows the campaign, and the missions the campaign's sorties become.

use stealth_campaign::{
    CampaignSetup, CampaignState, FlightView, GroundView, MissionKind, PlayerId, PlayerPos, Report,
    Side, Sortie, SortieEnd, WorldView,
};
use stealth_mission::{GameData, Mission, MissionKind as GenKind, MissionSettings, Seed};
use stealth_sim::{Sim, fflags, oflags};

/// Metres per world unit.
const WU: f64 = 512_000.0 / 1_048_576.0;

pub fn side_of(blue: bool) -> Side {
    if blue { Side::Blue } else { Side::Red }
}

/// The campaign's mission: a strike mission of the theater (for its settings, briefing date,
/// time and weather) with the theater's own world and no aircraft of its own.
pub fn base_mission(setup: &CampaignSetup, data: &GameData) -> anyhow::Result<Mission> {
    let settings = MissionSettings {
        theater: setup.theater,
        roe: setup.roe,
        opponents: setup.opponents,
        kind: GenKind::Strike,
        ..Default::default()
    };
    let mut m =
        stealth_mission::generate(data, &settings, Seed::Srand((setup.seed & 0xFFFF) as u16))?;
    let td = data.theaters.get(setup.theater).ok_or_else(|| anyhow::anyhow!("no theater"))?;
    m.world = td.world.clone();
    m.drones = None;
    m.time.deadline_ticks = 0;
    m.time.has_deadline = false;
    Ok(m)
}

/// The theater's land and water, for the campaign to put its forward batteries on land.
pub fn terrain(setup: &CampaignSetup, data: &GameData) -> stealth_campaign::Terrain {
    stealth_campaign::Terrain(data.theaters.get(setup.theater).cloned().map(std::sync::Arc::new))
}

/// What the simulation shows the campaign.
pub fn world_view(sim: &Sim) -> WorldView {
    let aaa = sim
        .units()
        .iter()
        .enumerate()
        .map(|(k, u)| GroundView {
            x: u.x,
            y: u.y,
            side: side_of(sim.aaa_blue(k)),
            alive: !u.destroyed,
        })
        .collect();
    let flights = sim
        .flights()
        .iter()
        .enumerate()
        .filter(|(_, u)| {
            u.flags & (fflags::EXISTS | fflags::AIRBORNE) == fflags::EXISTS | fflags::AIRBORNE
                && u.flags & fflags::DESTROYED == 0
                && u.speed > 0
        })
        .filter_map(|(i, u)| {
            let tag = sim.campaign_flight(i).map_or(0, |f| f.tag);
            let blue = sim.friendly_unit(i, u.spec);
            // The base traffic is no part of the war.
            (tag != 0 || !blue).then_some(FlightView { x: u.x, y: u.y, side: side_of(blue), tag })
        })
        .collect();
    let players = (0..sim.pilot_count() as u8)
        .filter_map(|id| {
            let p = sim.pilot_view(id)?;
            // A helicopter flies low: off the ground is airborne.
            let landed = sim.pilot_landed(id);
            let airborne = if sim.pilot_helicopter(id) { !landed } else { p.alt > 50 };
            let hover_over = sim.pilot_over_unit(id).map(|k| k as u32);
            Some(PlayerPos { id: PlayerId(id), x: p.mx, y: p.my, airborne, landed, hover_over })
        })
        .collect();
    WorldView { units: aaa, flights, players }
}

/// Enemy SAM sites a friendly aircraft detects now: their radar painted a player (his RWR)
/// or they fire at friendly aircraft, or a friendly aircraft is within their radar's range and
/// above its horizon (its RWR picks the radar up), or a drone of a command center found it
/// (the drones carry cameras, not RWRs).
pub fn spotted(state: &CampaignState, sim: &Sim) -> Vec<Report> {
    let found: Vec<usize> = sim
        .uav_known()
        .iter()
        .filter_map(|s| match *s {
            stealth_sim::uav::Seen::Object(o) => Some(o as usize),
            stealth_sim::uav::Seen::Aaa(_) => None,
        })
        .collect();
    let eyes: Vec<(i32, i32, f64)> = sim
        .flights()
        .iter()
        .enumerate()
        .filter(|(_, u)| {
            u.flags & (fflags::EXISTS | fflags::AIRBORNE) == fflags::EXISTS | fflags::AIRBORNE
                && u.flags & fflags::DESTROYED == 0
                && u.speed > 0
        })
        .filter(|(i, u)| sim.friendly_unit(*i, u.spec) && sim.uav_of_flight(*i).is_none())
        .map(|(_, u)| (u.x, u.y, u.alt.max(0) as f64 * WU))
        .collect();
    let objects = sim.objects();
    state
        .sites
        .iter()
        .filter(|s| s.kind == stealth_campaign::SiteKind::Sam && s.alive() && !s.spotted)
        .filter_map(|s| {
            let i = s.object as usize;
            let o = objects.get(i)?;
            if o.unit_type == 0 {
                return None;
            }
            let rwr = o.flags & (oflags::PAINTED | oflags::TRACKING) != 0 || sim.site_engaging(i);
            let range = (sim.radar_range(o.unit_type) * 64) as f64;
            let seen = eyes.iter().any(|&(x, y, alt)| {
                // Radar horizon (map units) of a target at `alt` metres.
                let horizon = 4120.0 * (alt.sqrt() + 3.2) / 15.625;
                ((x - o.x) as f64).hypot((y - o.y) as f64) < range.min(horizon)
            });
            (rwr || seen || found.contains(&i)).then_some(Report::SiteSpotted { object: s.object })
        })
        .collect()
}

/// How a sortie ended, from the simulation's end record.
pub fn sortie_end(e: &stealth_sim::MissionEnd) -> (SortieEnd, &'static str) {
    match (e.outcome, e.landing_type) {
        (0, 2) => (SortieEnd::Ejected, "Ejected"),
        (0, 4) => (SortieEnd::Rescued, "Ejected - extracted by the rescue helicopter"),
        (8, _) => (SortieEnd::Captured, "Ejected - captured by the enemy"),
        (0, _) => (SortieEnd::Landed, "Sortie ended on the ground"),
        (6, _) => (SortieEnd::Crashed, "Ejection failed - the pilot is recovered"),
        (7, _) => (SortieEnd::Aborted, "Sortie aborted"),
        (5, _) => (SortieEnd::Crashed, "Bad landing"),
        _ => (SortieEnd::Crashed, "Shot down / crashed - the pilot is recovered"),
    }
}

/// Stores for a kind of mission (weapon table indices, the generator's bays).
pub fn stores(kind: MissionKind) -> [i16; 4] {
    match kind {
        MissionKind::Recon => [16, 0, 2, 0],
        MissionKind::Sead => [2, 2, 5, 0],
        MissionKind::Cas => [5, 7, 5, 0],
        MissionKind::Strike | MissionKind::Interdiction => [6, 8, 6, 0],
        MissionKind::AntiShip => [4, 3, 5, 0],
        MissionKind::Cap
        | MissionKind::Escort
        | MissionKind::Insertion
        | MissionKind::Intercept
        | MissionKind::Defend
        | MissionKind::Csar
        | MissionKind::Awacs
        | MissionKind::Resupply
        | MissionKind::Logistics => [1, 0, 1, 0],
    }
}

/// The objective a sortie puts in the simulation's primary slot (mission type 1 photo, 2
/// destroy; object).
pub fn sim_objective(sortie: &Sortie) -> Option<(i32, usize)> {
    use stealth_campaign::Objective;
    match sortie.offer.objective {
        Objective::Recon { object, .. } => Some((1, object as usize)),
        Objective::Strike { airfield, .. } => Some((2, airfield as usize)),
        Objective::Interdict { object, .. } => Some((2, object as usize)),
        Objective::AntiShip { ship } => Some((2, ship as usize)),
        Objective::Sead { ref sites, .. } => sites.first().map(|&o| (2, o as usize)),
        _ => None,
    }
}

/// The sortie as a mission (briefing, kneeboard, route, stores): from the base to the target
/// and back.
pub fn sortie_mission(base: &Mission, state: &CampaignState, sortie: &Sortie) -> Mission {
    use stealth_mission::{Base, Bay, MapPoint, OrdersLine};
    let mut m = base.clone();
    let o = &sortie.offer;
    let b = state.site(sortie.base);
    let (bx, by) = b.map_or((0x4000, 0x4000), |s| (s.x, s.y));
    let home = Base {
        object_index: sortie.base as i16,
        name: b.map(|s| s.name.clone()).unwrap_or_default(),
        position: MapPoint { x: bx, y: by },
        carrier: b.is_some_and(|s| s.kind == stealth_campaign::SiteKind::Carrier),
        neutral: false,
        distance: 0,
    };
    // The briefing's times name the generated mission's bases: the sortie's instead.
    let bl = &mut m.briefing;
    for l in [&mut bl.base_date_line, &mut bl.depart_line] {
        *l = l.replace(&m.takeoff.name, &home.name);
    }
    bl.land_line = bl.land_line.replace(&m.landing.name, &home.name);
    // It leaves now (the campaign's clock), not at the generated mission's time.
    let secs = (state.sky_hours().rem_euclid(24.0) * 3600.0) as i32;
    let depart = format!("{:02}:{:02}", secs / 3600, secs / 60 % 60);
    bl.depart_line = bl.depart_line.replace(&m.time.depart, &depart);
    m.time.base_hour = (secs / 3600) as u8;
    m.time.tod_ticks = secs % 3600 / 2;
    m.time.depart = depart;
    m.takeoff = home.clone();
    m.landing = home;
    let target = MapPoint { x: o.x, y: o.y };
    let mid = MapPoint { x: (bx + o.x) / 2, y: (by + o.y) / 2 };
    // An intercept meets the helicopters on their track, then follows them to their goal.
    let then = match o.objective {
        stealth_campaign::Objective::Intercept { insertion } => state
            .insertion(insertion)
            .and_then(|i| state.site(i.target))
            .map_or(target, |s| MapPoint { x: s.x, y: s.y }),
        _ => target,
    };
    m.waypoints = [mid, target, then, MapPoint { x: bx, y: by }];
    // The fuel planned for that route (as for the drone missions), not the generated one's.
    let mut at = (bx, by);
    let mut length = 0.0;
    for p in &m.waypoints {
        length += ((p.x - at.0) as f64).hypot((p.y - at.1) as f64);
        at = (p.x, p.y);
    }
    m.route_length = (length / 64.0) as i16;
    let old_fuel = m.fuel_lbs.to_string();
    m.fuel_lbs = 12 * m.route_length;
    m.briefing.fuel_line = m.briefing.fuel_line.replace(&old_fuel, &m.fuel_lbs.to_string());
    // The targets are the campaign's: nothing of the generated strike mission's stays.
    let (object, shape_name, area) = primary_target(state, &o.objective);
    let p = &mut m.primary;
    // "SEAD - Wittstock": the place (the kneeboard puts the kind before it).
    p.place_name = o.title.split_once(" - ").map_or(o.title.clone(), |t| t.1.to_string());
    p.position = object.and_then(|k| state.site(k)).map_or(target, |s| MapPoint { x: s.x, y: s.y });
    p.mission_type = match o.kind {
        MissionKind::Recon => 1,
        k if airborne(k) => 8,
        _ => 2,
    };
    p.type_name = o.kind.name().into();
    p.object_index = object.map_or(-1, |k| k as i16);
    p.shape_name = shape_name;
    p.area_target = area;
    p.placed_by_generator = false;
    p.special_model = None;
    let s = &mut m.secondary;
    s.position = MapPoint { x: 0, y: 0 };
    s.mission_type = 0;
    s.object_index = -1;
    s.shape_name = String::new();
    s.place_name = String::new();
    s.special_model = None;
    let what = match o.objective {
        stealth_campaign::Objective::Battery { .. } => BATTERY_TASK,
        _ if sortie.aircraft.transport() => describe_heli(o.kind),
        _ => describe(o.kind),
    };
    m.briefing.title = o.title.clone();
    m.briefing.primary_summary = summary(state, sortie);
    m.briefing.primary_objective = what.to_string();
    m.briefing.secondary_summary = String::new();
    m.briefing.secondary_objective = String::new();
    m.orders.primary = o.title.to_ascii_uppercase();
    m.orders.secondary = String::new();
    m.orders.return_to = format!("RETURN TO {}", m.landing.name.to_ascii_uppercase());
    m.orders.lines = vec![
        OrdersLine { text: o.title.clone(), color: 15 },
        OrdersLine { text: what.into(), color: 11 },
        OrdersLine { text: m.orders.return_to.clone(), color: 11 },
    ];
    m.loadout.bays =
        stores(o.kind).iter().map(|&w| Bay { weapon: w, name: String::new(), count: 1 }).collect();
    m.loadout.usable_bays = if sortie.aircraft.transport() {
        0
    } else if m.settings.plane == stealth_mission::Plane::Lockheed {
        2
    } else {
        4
    };
    m
}

/// Missions against aircraft or over a point: nothing on the ground to photograph.
pub fn airborne(kind: MissionKind) -> bool {
    matches!(
        kind,
        MissionKind::Cap
            | MissionKind::Escort
            | MissionKind::Insertion
            | MissionKind::Intercept
            | MissionKind::Defend
            | MissionKind::Csar
            | MissionKind::Awacs
            | MissionKind::Resupply
            | MissionKind::Logistics
    )
}

/// What an enemy forward battery's offer asks for.
pub const BATTERY_TASK: &str = "Destroy the guns of the enemy battery pushed forward near our base";

/// The world object an objective is about (None: aircraft or an area), what it is for the
/// kneeboard, and whether it is an area target.
fn primary_target(
    state: &CampaignState,
    o: &stealth_campaign::Objective,
) -> (Option<u16>, String, bool) {
    use stealth_campaign::{Objective, SiteKind};
    let object = match *o {
        Objective::Recon { object, .. }
        | Objective::Interdict { object, .. }
        | Objective::AntiShip { ship: object }
        | Objective::Strike { airfield: object, .. } => Some(object),
        Objective::Sead { ref sites, .. } => sites.first().copied(),
        Objective::Cas { .. } => return (None, "GUNS AND MANPADS".into(), true),
        Objective::Battery { .. } => return (None, "GUN BATTERY".into(), true),
        _ => return (None, String::new(), false),
    };
    let name = object.and_then(|k| state.site(k)).map_or(String::new(), |s| {
        match s.kind {
            SiteKind::Airfield => "AIRFIELD",
            SiteKind::Carrier => "CARRIER",
            SiteKind::City => "CITY",
            SiteKind::Sam => "SAM SITE",
            SiteKind::Ship => "SHIP",
        }
        .into()
    });
    (object, name, false)
}

/// The intelligence summary of a sortie's target.
pub fn summary(state: &CampaignState, sortie: &Sortie) -> String {
    use stealth_campaign::Objective;
    let o = &sortie.offer;
    let site = |k: u16| state.site(k).map_or(String::new(), |s| s.name.clone());
    match o.objective {
        Objective::Recon { object, .. } => format!("Photograph {} and its defences.", site(object)),
        Objective::Sead { ref sites, .. } => {
            format!("{} SAM sites and radars defend the place.", sites.len())
        }
        Objective::Cas { ref aaa, .. } => {
            format!("{} guns and MANPADS teams defend the place.", aaa.len())
        }
        Objective::Battery { ref aaa, .. } => format!(
            "An enemy battery pushed forward: {} guns and a MANPADS team. Destroy them.",
            aaa.len()
        ),
        Objective::Strike { airfield, .. } => format!("The airfield of {}.", site(airfield)),
        Objective::Interdict { object, .. } => {
            format!("The city of {}: its stores and factories feed the enemy.", site(object))
        }
        Objective::AntiShip { .. } => "An enemy ship off the coast.".into(),
        Objective::Resupply { request } => match state.supply_request(request) {
            Some(r) => format!("The {} needs a new stockpile. {}", r.title(), describe(o.kind)),
            None => describe(o.kind).into(),
        },
        Objective::Logistics { site, leg } => match state.deployment(site) {
            Some(d) => format!(
                "Cargo {} of {} for the new {}. {}",
                leg + 1,
                d.loads,
                d.title(),
                describe(o.kind)
            ),
            None => describe(o.kind).into(),
        },
        Objective::Intercept { insertion } | Objective::Escort { insertion } => {
            match state.insertion_status(insertion) {
                Some(t) => format!("{t}. {}", describe(o.kind)),
                None => describe(o.kind).into(),
            }
        }
        _ => describe(o.kind).into(),
    }
}

/// What a kind of mission asks the pilot for.
pub fn describe(kind: MissionKind) -> &'static str {
    match kind {
        MissionKind::Recon => {
            "Photograph the target: its defences are then known (spotted from twice the range, shorter radar range) for an hour"
        }
        MissionKind::Sead => "Destroy the SAM sites and radars defending the place",
        MissionKind::Cap => {
            "Hold the air over the point for 10 minutes or shoot down two enemy aircraft there; a friendly aircraft lost there starts the clock again"
        }
        MissionKind::Cas => "Destroy the guns and MANPADS teams around the place",
        MissionKind::AntiShip => "Sink the enemy ship",
        MissionKind::Strike => {
            "Destroy the airfield: no enemy aircraft take off from it until it is repaired"
        }
        MissionKind::Escort => "Cover the insertion helicopters until they land",
        MissionKind::Insertion => {
            "Helicopters fly troops in from the nearest friendly base; one landing takes the place"
        }
        MissionKind::Intercept => {
            "Shoot down the enemy's troop helicopters before they land: one landing takes the place"
        }
        MissionKind::Defend => "Shoot down the enemy strike aircraft before they bomb the airfield",
        MissionKind::Interdiction => {
            "Strike the enemy's rear: a city costs it materiel at once and its income until rebuilt, an airfield its aircraft there"
        }
        MissionKind::Csar => {
            "Hold the air within 5 km of the downed pilot for 2 minutes while the helicopter picks him up"
        }
        MissionKind::Awacs => {
            "Shoot down the enemy AWACS: while it is up the enemy sends more patrols, and sends them at you"
        }
        MissionKind::Resupply => {
            "Fly the stockpile from the airfield to the air defence unit: hover below 30 m within 50 m of it for 10 s while the load goes down"
        }
        MissionKind::Logistics => {
            "Hook a cargo crate beside the airfield's runway (hover over it below 50 ft) and fly it to the new site's helipad: set it down within 90 m of the pad"
        }
    }
}

/// What a kind of mission asks of the helicopter pilot (insertions and rescues).
pub fn describe_heli(kind: MissionKind) -> &'static str {
    match kind {
        MissionKind::Insertion => {
            "Fly the troops in, your wingmen with you: land within 1.5 km of the target and stay on the ground for 15 s while they get out to take the place"
        }
        MissionKind::Csar => {
            "Land within 200 m of the downed pilot and wait 30 s for him to board, then land him at a friendly base"
        }
        k => describe(k),
    }
}

/// The friendly side's base nearest the front (where a player waits between sorties).
pub fn default_base(state: &CampaignState) -> Option<usize> {
    let front = state.front(Side::Blue);
    let (x, y) =
        front.first().and_then(|&p| state.place(p)).map_or((0x4000, 0x4000), |p| (p.x, p.y));
    state.nearest_player_base(x, y).map(|b| b as usize)
}

/// The kind of player site a mannable unit is (a Patriot battery: its radar).
pub fn defence_kind(kind: stealth_sim::aaa::GroundKind) -> Option<stealth_campaign::DefenceKind> {
    use stealth_campaign::DefenceKind;
    use stealth_sim::aaa::GroundKind;
    match kind {
        GroundKind::Gepard => Some(DefenceKind::Gepard),
        GroundKind::PatriotRadar => Some(DefenceKind::Patriot),
        GroundKind::DroneCenter => Some(DefenceKind::Drones),
        _ => None,
    }
}

/// The unit that keeps a site's stockpile (a Patriot battery's radar).
pub fn station_unit(kind: stealth_campaign::DefenceKind) -> stealth_sim::aaa::GroundKind {
    use stealth_campaign::DefenceKind;
    use stealth_sim::aaa::GroundKind;
    match kind {
        DefenceKind::Gepard => GroundKind::Gepard,
        DefenceKind::Patriot => GroundKind::PatriotRadar,
        DefenceKind::Drones => GroundKind::DroneCenter,
    }
}

/// A new site the players supplied stands at (x, y).
pub fn place_site(sim: &mut Sim, kind: stealth_campaign::DefenceKind, x: i32, y: i32) {
    use stealth_campaign::DefenceKind;
    match kind {
        DefenceKind::Gepard => sim.place_gepard(x, y),
        DefenceKind::Patriot => sim.place_patriot(x, y),
        DefenceKind::Drones => sim.place_drone_center(x, y),
    }
}
