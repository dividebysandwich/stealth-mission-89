//! The campaign host without the renderer, on the shipped game data.

use stealth_campaign::{Aircraft, CampaignSetup, PlayerId};
use stealth_host::{Host, HostEvent, Rules, theater};
use stealth_sim::PlayerState;

fn setup(code: &str) -> CampaignSetup {
    CampaignSetup {
        theater: theater::theater_index(code).unwrap(),
        theater_code: code.into(),
        roe: 1,
        opponents: 1,
        seed: 1234,
        sky_ratio: 24.0,
        start_hour: 8.0,
        dynamic_weather: true,
        crew_scaling: true,
    }
}

/// Two players fly sorties for a while; the war goes on.
fn two_players(code: &str) {
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let rules = Rules { continuous_time: true, ..Default::default() };
    let mut host = Host::new(s, &data, rules).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    let me = PlayerId(0);
    let wing = host.join(&mut sim, "Wingman", |_| true).unwrap();
    assert_eq!(wing, PlayerId(1));
    assert_eq!(
        host.join(&mut sim, "wingman", |_| true).unwrap(),
        wing,
        "the same name is the same player"
    );
    // (Not the new sites' cargo: a helicopter's.)
    let flown: Vec<u32> = host
        .state
        .board
        .iter()
        .filter(|o| !o.kind.ai_flown() && o.kind != stealth_campaign::MissionKind::Logistics)
        .map(|o| o.id)
        .collect();
    assert!(flown.len() >= 2, "{:?}", host.state.board);
    let mut inputs: Vec<(u8, PlayerState)> = Vec::new();
    for (p, &offer, aircraft) in [(me, &flown[0], Aircraft::F117), (wing, &flown[1], Aircraft::F16)]
    {
        let ev = host.take_mission(&mut sim, p, offer, None, aircraft, None).unwrap();
        let start = ev
            .iter()
            .find_map(|e| match e {
                HostEvent::SortieStarted { player, start, .. } if *player == p => {
                    Some(start.clone())
                }
                _ => None,
            })
            .expect("sortie started");
        inputs.push((p.0, start));
    }
    assert!(!sim.pilot_absent(0) && !sim.pilot_absent(1));
    for _ in 0..15 * 60 {
        let ev = sim.step_pilots(&inputs, 15);
        let mut ev2 = host.take_sim_events(&mut sim);
        ev2.extend(ev);
        host.tick(&mut sim, 1.0 / 15.0, &ev2);
    }
    assert!(host.state.clock > 59.0);
    let ev = host.end_sortie(&mut sim, wing, 7);
    assert!(
        ev.iter().any(|e| matches!(e, HostEvent::SortieFinished { player, .. } if *player == wing)),
        "{ev:?}"
    );
    assert!(sim.pilot_absent(1) && !sim.pilot_absent(0));
}

#[test]
fn two_players_in_central_europe() {
    two_players("central_europe");
}

#[test]
fn a_remake_theater_without_the_renderer() {
    two_players("ukraine");
}

#[test]
fn the_server_sees_by_the_moon() {
    // Noon is day, midnight dark (the light of a moon at most).
    let lat = stealth_host::sky::latitude("central_europe");
    assert_eq!(stealth_host::sky::visual_light(7000.0, 12.0, lat), 255);
    assert!(stealth_host::sky::visual_light(7000.0, 0.0, lat) <= 85);
}

/// The friendly side's air defences stand from the start: Patriots in the rear, Gepards at the
/// airfields.
#[test]
fn friendly_air_defences_stand_from_the_start() {
    use stealth_sim::aaa::GroundKind;
    for code in ["central_europe", "ukraine"] {
        let s = setup(code);
        let data = theater::game_data().unwrap();
        let sim_data = theater::sim_data(s.theater, code).unwrap();
        let mut host = Host::new(s, &data, Rules::default()).unwrap();
        let mut sim = host.new_sim(&sim_data);
        host.begin(&mut sim);
        let blue = |sim: &stealth_sim::Sim, kind| {
            (0..sim.units().len())
                .filter(|&k| sim.units()[k].kind == kind && sim.aaa_blue(k))
                .count()
        };
        let batteries = host.state.patriot_sites().len();
        assert!(batteries > 0, "{code}");
        assert_eq!(blue(&sim, GroundKind::PatriotRadar), batteries, "{code}");
        assert_eq!(blue(&sim, GroundKind::PatriotLauncher), 3 * batteries, "{code}");
        assert!(blue(&sim, GroundKind::Gepard) > 0, "{code}");
        assert_eq!(blue(&sim, GroundKind::Zsu), 0, "{code}");
        // No two batteries in one place, on a second begin neither.
        host.begin(&mut sim);
        assert_eq!(blue(&sim, GroundKind::PatriotRadar), batteries, "{code}");
    }
}

/// The enemy's offensive played out in the simulation: its raid and its insertion end one way
/// or the other (the strike aircraft bomb, are stopped or called off with their target gone, the
/// Hinds land or are shot down, or find the garrison still holding the place).
#[test]
fn the_enemy_offensive_plays_out() {
    use stealth_campaign::{InsertionState, RaidState, Side, tuning};
    let s = CampaignSetup { opponents: 3, ..setup("central_europe") };
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "central_europe").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    // Both due at once.
    host.state.raid_at = -tuning::RAID_EVERY[3];
    host.state.enemy_insert_at = -tuning::ENEMY_INSERT_EVERY[3];
    for _ in 0..15 * 60 * 60 {
        let e = sim.step_pilots(&[], 15);
        let mut e2 = host.take_sim_events(&mut sim);
        e2.extend(e);
        host.tick(&mut sim, 1.0 / 15.0, &e2);
        let raid = host.state.raids.first().map(|r| r.state);
        let ins = host.state.insertions.iter().find(|i| i.side == Side::Red).map(|i| i.state);
        let done = |s: Option<bool>| s == Some(true);
        if done(
            raid.map(|r| matches!(r, RaidState::Hit | RaidState::Repelled | RaidState::CalledOff)),
        ) && done(ins.map(|i| matches!(i, InsertionState::Captured | InsertionState::Failed)))
        {
            return;
        }
    }
    panic!(
        "not over in an hour: raids {:?}, insertions {:?}",
        host.state.raids,
        host.state.insertions.iter().map(|i| (i.side, i.state)).collect::<Vec<_>>()
    );
}

/// A campaign keeps the rules it started with, whoever serves it later.
#[test]
fn a_campaign_keeps_its_rules() {
    let data = theater::game_data().unwrap();
    let hard = Rules { realistic_damage: true, friendly_fire: true, ..Default::default() };
    let host = Host::new(setup("central_europe"), &data, hard).unwrap();
    let saved: stealth_campaign::CampaignState =
        serde_json::from_str(&serde_json::to_string(&host.state).unwrap()).unwrap();
    let again = Host::resume(saved, &data, Rules::default()).unwrap();
    assert_eq!(again.rules, hard);
    // An older save takes the server's.
    let mut old = host.state.clone();
    old.rules = None;
    let easy = Rules { no_crashes: true, ..Default::default() };
    assert_eq!(Host::resume(old, &data, easy).unwrap().rules, easy);
}

/// The real F-117A, chosen for one sortie or by the campaign's rule: two bays without the
/// air-to-air missiles in its briefing, only for the F-117A.
#[test]
fn a_sortie_briefs_the_real_f117() {
    use stealth_campaign::{MissionKind, Progress, Sortie};
    use stealth_mission::Plane;
    let data = theater::game_data().unwrap();
    for rule in [false, true] {
        let rules = Rules { realistic_weapons: rule, ..Default::default() };
        let mut host = Host::new(setup("central_europe"), &data, rules).unwrap();
        host.state.tick(0.1, &stealth_campaign::WorldView::default(), &[]);
        for chosen in [false, true] {
            for aircraft in [Aircraft::F117, Aircraft::F16] {
                for kind in [MissionKind::Recon, MissionKind::Strike] {
                    let offer = host.state.board.iter().find(|o| o.kind == kind).cloned().unwrap();
                    let sortie = Sortie {
                        id: 1,
                        base: offer.base,
                        offer,
                        aircraft,
                        started: 0.0,
                        progress: Progress::default(),
                        flight: 0,
                        position: 0,
                        crew: Vec::new(),
                        realistic: chosen,
                    };
                    let m = stealth_host::glue::sortie_mission(&host.mission, &host.state, &sortie);
                    let real = (rule || chosen) && aircraft == Aircraft::F117;
                    assert_eq!(
                        m.settings.plane,
                        if real { Plane::Lockheed } else { Plane::Fictional }
                    );
                    assert_eq!(m.loadout.usable_bays, if real { 2 } else { 4 });
                    let weapons: Vec<_> = m
                        .loadout
                        .bays
                        .iter()
                        .take(m.loadout.usable_bays)
                        .map(|b| b.weapon)
                        .collect();
                    let expected = match (kind, real) {
                        (MissionKind::Recon, true) => vec![16, 2],
                        (MissionKind::Recon, false) => vec![16, 0, 2, 0],
                        (_, true) => vec![6, 8],
                        (_, false) => vec![6, 8, 6, 0],
                    };
                    assert_eq!(
                        weapons, expected,
                        "{aircraft:?} {kind:?} rule={rule} chosen={chosen}"
                    );
                }
            }
        }
    }
}

/// Taking an F-117A sortie with the real loadout: two bays and no cannon in the simulation,
/// the choice kept on the sortie (an F-16 ignores it).
#[test]
fn a_real_f117_sortie_flies_without_a_cannon() {
    use stealth_campaign::MissionKind;
    let setup = setup("central_europe");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(setup.theater, &setup.theater_code).unwrap();
    for (aircraft, realistic) in
        [(Aircraft::F117, false), (Aircraft::F117, true), (Aircraft::F16, true)]
    {
        let mut host = Host::new(setup.clone(), &data, Rules::default()).unwrap();
        let mut sim = host.new_sim(&sim_data);
        host.begin(&mut sim);
        host.tick(&mut sim, 0.1, &[]);
        let offer = host.state.board.iter().find(|o| o.kind == MissionKind::Recon).unwrap().id;
        let stations =
            if realistic { vec![(16, 1), (2, 1)] } else { vec![(16, 1), (0, 1), (2, 1), (0, 1)] };
        let stores = stealth_host::Stores { stations: stations.clone(), select: 0, realistic };
        host.take_mission(&mut sim, PlayerId(0), offer, None, aircraft, Some(stores)).unwrap();
        let real = realistic && aircraft == Aircraft::F117;
        assert_eq!(host.state.sortie(PlayerId(0)).unwrap().realistic, real);
        if aircraft == Aircraft::F117 {
            assert_eq!(sim.bays().0, stations.as_slice());
            assert_eq!(sim.has_gun(), !real);
        }
    }
}

/// A drone wave over Ukraine played out in the simulation: the drones dive into their target
/// or are shot down.
#[test]
fn a_drone_wave_plays_out() {
    use stealth_campaign::{RaidState, tuning};
    let s = setup("ukraine");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "ukraine").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.state.drones_at = -tuning::DRONES_EVERY[1];
    for _ in 0..15 * 60 * 120 {
        let e = sim.step_pilots(&[], 15);
        let mut e2 = host.take_sim_events(&mut sim);
        e2.extend(e);
        host.tick(&mut sim, 1.0 / 15.0, &e2);
        if host
            .state
            .raids
            .iter()
            .any(|r| r.drones && matches!(r.state, RaidState::Hit | RaidState::Repelled))
        {
            return;
        }
    }
    panic!("{:?}", host.state.raids);
}

/// The kneeboard's targets are the campaign's: an air mission has none to photograph, a strike
/// its airfield.
#[test]
fn a_sortie_briefs_its_own_target() {
    use stealth_campaign::{MissionKind, Objective, Progress, Sortie, WorldView};
    let data = theater::game_data().unwrap();
    let mut host = Host::new(setup("central_europe"), &data, Rules::default()).unwrap();
    host.state.tick(0.1, &WorldView::default(), &[]);
    let sortie = |kind| {
        let offer = host.state.board.iter().find(|o| o.kind == kind).cloned().unwrap();
        Sortie {
            id: 1,
            base: offer.base,
            offer,
            aircraft: Default::default(),
            started: 0.0,
            progress: Progress::default(),
            flight: 0,
            position: 0,
            crew: Vec::new(),
            realistic: false,
        }
    };
    let cap =
        stealth_host::glue::sortie_mission(&host.mission, &host.state, &sortie(MissionKind::Cap));
    assert_eq!(cap.primary.mission_type, 8, "airborne: no photo");
    assert!(cap.primary.shape_name.is_empty());
    assert_eq!((cap.secondary.position.x, cap.secondary.position.y), (0, 0));
    let s = sortie(MissionKind::Strike);
    let Objective::Strike { airfield, .. } = s.offer.objective else { panic!() };
    let strike = stealth_host::glue::sortie_mission(&host.mission, &host.state, &s);
    assert_eq!(strike.primary.mission_type, 2);
    assert_eq!(strike.primary.object_index, airfield as i16);
    assert_eq!(strike.primary.shape_name, "AIRFIELD");
}

/// Every theater can be won: taking what the helicopters reach from the friendly bases, the
/// enemy's airfields all fall in the end.
#[test]
fn every_theater_can_be_won() {
    use stealth_campaign::{Side, dist, tuning};
    let data = theater::game_data().unwrap();
    for code in theater::codes() {
        let mut c = Host::new(setup(&code), &data, Rules::default()).unwrap().state;
        loop {
            let bases: Vec<(i32, i32)> =
                c.bases(Side::Blue).iter().filter_map(|&b| c.site(b).map(|s| (s.x, s.y))).collect();
            let reach = |x: i32, y: i32| {
                bases.iter().any(|&(bx, by)| dist(x - bx, y - by) <= tuning::INSERTION_RANGE)
            };
            match c.places.iter().position(|p| p.owner == Side::Red && reach(p.x, p.y)) {
                Some(k) => c.places[k].owner = Side::Blue,
                None => break,
            }
        }
        let left: Vec<&str> = c
            .places
            .iter()
            .filter(|p| p.owner == Side::Red && p.airfield.is_some())
            .map(|p| p.name.as_str())
            .collect();
        assert!(left.is_empty(), "{code}: out of reach {left:?}");
    }
}

/// Every friendly airfield has a Gepard a player can man from the start, also one whose guns
/// the campaign has lost (a save: they are wrecks until replaced).
#[test]
fn every_friendly_airfield_has_a_gepard_to_man() {
    use stealth_campaign::Side;
    use stealth_sim::aaa::GroundKind;
    use stealth_sim::station::StationCmd;
    for (code, lost) in [("central_europe", 0), ("ukraine", 0), ("central_europe", 9)] {
        let s = setup(code);
        let data = theater::game_data().unwrap();
        let sim_data = theater::sim_data(s.theater, code).unwrap();
        let mut host = Host::new(s, &data, Rules::default()).unwrap();
        for p in host.state.places.iter_mut() {
            p.guns_lost = lost;
        }
        let mut sim = host.new_sim(&sim_data);
        host.begin(&mut sim);
        let fields: Vec<(i32, i32, String)> = host
            .state
            .places
            .iter()
            .filter(|p| p.owner == Side::Blue && p.airfield.is_some())
            .map(|p| (p.x, p.y, p.name.clone()))
            .collect();
        assert!(!fields.is_empty(), "{code}");
        for (x, y, name) in fields {
            let gepard = (0..sim.units().len()).find(|&k| {
                let u = sim.units()[k];
                u.kind == GroundKind::Gepard
                    && !u.destroyed
                    && sim.aaa_blue(k)
                    && (u.x - x).abs().max((u.y - y).abs()) < 0x180
            });
            let Some(k) = gepard else { panic!("{code}: no Gepard at {name} (lost {lost})") };
            sim.station_command(0, StationCmd::Man { unit: k as u16 }).unwrap();
            sim.station_command(0, StationCmd::Leave).unwrap();
        }
    }
}

/// A player at a Gepard's console asks for a new stockpile; nobody takes the resupply, so after
/// five minutes an AI helicopter flies it from the airfield, the stockpile is full again and the
/// helicopter lands back at its base.
#[test]
fn an_ai_helicopter_brings_a_requested_resupply() {
    use stealth_campaign::{MissionKind, SupplyState};
    use stealth_sim::aaa::GroundKind;
    use stealth_sim::station::{GEPARD_STOCK, SavedStock, StationCmd};
    let code = "central_europe";
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    let me = PlayerId(0);
    let k = (0..sim.units().len())
        .find(|&k| sim.units()[k].kind == GroundKind::Gepard && sim.aaa_blue(k))
        .unwrap();
    let u = sim.units()[k];
    sim.restore_stocks(&[SavedStock {
        x: u.x,
        y: u.y,
        kind: GroundKind::Gepard,
        stock: 0,
        stock2: 0,
        stock3: 0,
        stock4: 0,
    }]);
    host.station(&mut sim, me, StationCmd::Man { unit: k as u16 }).unwrap();
    host.station(&mut sim, me, StationCmd::RequestResupply).unwrap();
    assert!(host.station(&mut sim, me, StationCmd::RequestResupply).is_err());
    host.tick(&mut sim, 0.1, &[]);
    host.state.board_at = f64::MIN;
    host.tick(&mut sim, 0.1, &[]);
    assert!(host.state.board.iter().any(|o| o.kind == MissionKind::Resupply));
    let (mut launched, mut restocked, mut landed) = (false, false, false);
    for _ in 0..15 * 1200 {
        let ev = sim.step_pilots(&[], 15);
        let mut ev2 = host.take_sim_events(&mut sim);
        ev2.extend(ev);
        landed |= ev2.iter().any(|e| {
            matches!(e.event, stealth_sim::SimEvent::CampaignFlightLanded { .. }) && restocked
        });
        let out = host.tick(&mut sim, 1.0 / 15.0, &ev2);
        for e in &out {
            match e {
                HostEvent::Campaign(stealth_campaign::CampaignEvent::SupplyLaunched { .. }) => {
                    launched = true;
                    assert!(host.state.clock >= 300.0);
                    assert!(!host.state.board.iter().any(|o| o.kind == MissionKind::Resupply));
                }
                HostEvent::Campaign(stealth_campaign::CampaignEvent::Restocked { aaa }) => {
                    assert_eq!(*aaa as usize, k);
                    restocked = true;
                }
                _ => {}
            }
        }
        if landed {
            break;
        }
    }
    assert!(launched && restocked && landed, "{launched} {restocked} {landed}");
    assert_eq!(sim.stockpile(k), GEPARD_STOCK);
    assert!(host.state.supply_requests.iter().all(|r| !matches!(r.state, SupplyState::Ai { .. })));
    assert!(host.state.supply_requests.is_empty());
}

/// An unmanned Patriot battery refills its empty launchers from its stockpile by itself: when
/// the last missiles go, a resupply is ordered by itself (once), and goes on the board.
#[test]
fn an_empty_stockpile_orders_a_resupply() {
    use stealth_campaign::{MissionKind, Objective};
    use stealth_sim::aaa::GroundKind;
    use stealth_sim::station::SavedStock;
    let code = "central_europe";
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    assert!(host.state.supply_requests.is_empty(), "{:?}", host.state.supply_requests);
    let radar = (0..sim.units().len())
        .find(|&k| sim.units()[k].kind == GroundKind::PatriotRadar && sim.aaa_blue(k))
        .expect("a friendly Patriot battery");
    let r = sim.units()[radar];
    // One missile left in the stockpile, one launcher fired empty.
    sim.restore_stocks(&[SavedStock {
        x: r.x,
        y: r.y,
        kind: GroundKind::PatriotRadar,
        stock: 1,
        stock2: 0,
        stock3: 0,
        stock4: 0,
    }]);
    let l = sim.battery_launchers(radar)[0];
    sim.units_mut()[l].rounds = 0;
    host.tick(&mut sim, 0.1, &[]);
    assert!(host.state.supply_requests.is_empty());
    // The battery's own refill takes the last one.
    host.refills(&mut sim, 1.0);
    host.refills(&mut sim, stealth_sim::station::REFILL_SECONDS);
    assert_eq!(sim.stockpile(radar), 0);
    let ev = host.tick(&mut sim, 0.1, &[]);
    assert!(ev.iter().any(|e| matches!(e,
        HostEvent::Campaign(stealth_campaign::CampaignEvent::Message(m)) if m.tr_in(stealth_i18n::Language::En).contains("out of missiles"))));
    assert_eq!(host.state.supply_requests.len(), 1);
    let q = host.state.supply_requests[0].clone();
    assert_eq!(
        (q.aaa as usize, q.by, q.kind),
        (radar, None, stealth_campaign::DefenceKind::Patriot)
    );
    // Once.
    host.tick(&mut sim, 0.1, &[]);
    assert_eq!(host.state.supply_requests.len(), 1);
    host.state.board_at = f64::MIN;
    host.tick(&mut sim, 0.1, &[]);
    assert!(
        host.state.board.iter().any(|o| o.kind == MissionKind::Resupply
            && o.objective == Objective::Resupply { request: q.id })
    );
}

/// A sortie's briefing names the base it flies from and back to (an airfield or a carrier), leaves
/// now and plans the fuel for its own route, in every theater.
#[test]
fn a_sortie_briefs_its_own_base_and_fuel() {
    use stealth_campaign::{Progress, SiteKind, Sortie, WorldView};
    let data = theater::game_data().unwrap();
    let mut carriers = 0;
    for code in theater::codes() {
        let mut host = Host::new(setup(&code), &data, Rules::default()).unwrap();
        host.state.tick(0.1, &WorldView::default(), &[]);
        let generated = &host.mission;
        for base in host.state.player_bases() {
            let site = host.state.site(base).unwrap();
            carriers += (site.kind == SiteKind::Carrier) as usize;
            for offer in &host.state.board {
                let s = Sortie {
                    id: 1,
                    base,
                    offer: offer.clone(),
                    aircraft: Default::default(),
                    started: 0.0,
                    progress: Progress::default(),
                    flight: 0,
                    position: 0,
                    crew: Vec::new(),
                    realistic: false,
                };
                let m = stealth_host::glue::sortie_mission(generated, &host.state, &s);
                let b = &m.briefing;
                let what = format!("{code} {} from {}", offer.title, site.name);
                assert_eq!(
                    (m.takeoff.name.as_str(), m.landing.name.as_str()),
                    (&*site.name, &*site.name)
                );
                for l in [&b.base_date_line, &b.depart_line, &b.land_line] {
                    assert!(l.contains(&site.name), "{what}: {l:?}");
                    for old in [&generated.takeoff.name, &generated.landing.name] {
                        assert!(*old == site.name || !l.contains(old.as_str()), "{what}: {l:?}");
                    }
                }
                let p = &m.waypoints;
                let km =
                    |a: (i32, i32), b: (i32, i32)| ((a.0 - b.0) as f64).hypot((a.1 - b.1) as f64);
                let mut length = 0.0;
                let mut at = (site.x, site.y);
                for w in p {
                    length += km(at, (w.x, w.y));
                    at = (w.x, w.y);
                }
                assert_eq!(m.route_length, (length / 64.0) as i16, "{what}");
                assert_eq!(m.fuel_lbs, 12 * m.route_length, "{what}");
                assert!(b.fuel_line.contains(&m.fuel_lbs.to_string()), "{what}: {:?}", b.fuel_line);
                assert_eq!((p[3].x, p[3].y), (site.x, site.y), "{what}: lands at home");
                let secs = (host.state.sky_hours().rem_euclid(24.0) * 3600.0) as i32;
                let now = format!("{:02}:{:02}", secs / 3600, secs / 60 % 60);
                assert_eq!(m.time.depart, now, "{what}");
                assert!(b.depart_line.contains(&now), "{what}: {:?}", b.depart_line);
                let start = m.time.base_hour as i32 * 3600 + m.time.tod_ticks * 2;
                assert!((secs - start).abs() <= 1, "{what}: starts at {start} s, now {secs} s");
            }
        }
    }
    assert!(carriers > 0, "no carrier sorties tested");
}

/// A player designates a Gepard site near a friendly airfield; nobody flies its cargo, so an AI
/// helicopter lands it on the helipad, and a friendly Gepard stands beside the pad.
#[test]
fn an_ai_helicopter_supplies_a_designated_gepard_site() {
    use stealth_campaign::{CampaignEvent, MissionKind, Side};
    use stealth_sim::aaa::GroundKind;
    let code = "central_europe";
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    let me = PlayerId(0);
    let aaa = stealth_host::glue::world_view(&sim).units;
    let base = host
        .state
        .places
        .iter()
        .find(|p| p.owner == Side::Blue && p.airfield.is_some())
        .map(|p| (p.x, p.y))
        .unwrap();
    let (x, y) = (1..60)
        .flat_map(|r| (0..16).map(move |k| (r, k)))
        .map(|(r, k)| {
            let t = k as f64 / 16.0 * std::f64::consts::TAU;
            (
                base.0 + (t.cos() * r as f64 * 32.0) as i32,
                base.1 + (t.sin() * r as f64 * 32.0) as i32,
            )
        })
        .find(|&(x, y)| {
            host.state.deploy_refusal(x, y, stealth_campaign::DefenceKind::Gepard, &aaa).is_none()
        })
        .expect("a free spot");
    // Not on a unit.
    let u = sim.units().iter().find(|u| !u.destroyed).copied().unwrap();
    assert!(
        host.designate_site(&mut sim, me, u.x, u.y + 3, stealth_campaign::DefenceKind::Gepard)
            .is_err()
    );
    host.designate_site(&mut sim, me, x, y, stealth_campaign::DefenceKind::Gepard).unwrap();
    host.state.board_at = f64::MIN;
    host.tick(&mut sim, 0.1, &[]);
    assert!(host.state.board.iter().any(|o| o.kind == MissionKind::Logistics));
    // Nobody took it for long enough.
    host.state.deployments[0].open_since = -1e9;
    let n = sim.units().len();
    let (mut launched, mut built) = (false, false);
    for _ in 0..15 * 1200 {
        let ev = sim.step_pilots(&[], 15);
        let mut ev2 = host.take_sim_events(&mut sim);
        ev2.extend(ev);
        for e in host.tick(&mut sim, 1.0 / 15.0, &ev2) {
            match e {
                HostEvent::Campaign(CampaignEvent::CargoLaunched { id, tag, .. }) => {
                    // It goes for its crate where the clients draw it, beside the runway.
                    let run = (0..sim.flights().len())
                        .filter_map(|i| sim.campaign_flight(i))
                        .find(|f| f.tag == tag)
                        .and_then(|f| f.supply)
                        .expect("its run");
                    let leg = host.state.deployment(id).unwrap().ai[0].0;
                    let (home, _) =
                        stealth_host::cargo::crate_home(&sim, &host.state, id, leg).unwrap();
                    assert_eq!(run.pickup, Some(stealth_host::cargo::world_units(home)));
                    assert!(!sim.runways().is_empty());
                    launched = true;
                }
                HostEvent::Campaign(CampaignEvent::SiteBuilt { .. }) => built = true,
                _ => {}
            }
        }
        if built {
            break;
        }
    }
    assert!(launched && built, "{launched} {built}");
    let spot = host.state.deployments[0].unit_at();
    let k = (n..sim.units().len())
        .find(|&k| (sim.units()[k].x, sim.units()[k].y) == spot)
        .expect("the new unit");
    let g = sim.units()[k];
    assert_eq!(g.kind, GroundKind::Gepard);
    assert!(sim.aaa_blue(k));
    assert!(!host.state.board.iter().any(|o| o.kind == MissionKind::Logistics));
    // A new simulation (a loaded save) has it again.
    let mut again = host.new_sim(&sim_data);
    let mut host2 = Host::resume(host.state.clone(), &data, Rules::default()).unwrap();
    host2.begin(&mut again);
    assert!((0..again.units().len()).any(|k| {
        let u = again.units()[k];
        u.kind == GroundKind::Gepard && again.aaa_blue(k) && (u.x, u.y) == (g.x, g.y)
    }));
}

/// A player hooks the crate for a new Gepard site under his UH-60 and leaves the helicopter to
/// the AI on the way: it flies on from where it is with the crate still slung, sets it down by
/// the site's helipad and the site is built (it neither turns home nor loses the crate).
#[test]
fn a_helicopter_left_to_the_ai_delivers_its_slung_crate() {
    use stealth_campaign::{
        CampaignEvent, CrateState, DefenceKind, LeftCargo, MissionKind, Objective, Side,
    };
    let code = "central_europe";
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    let me = PlayerId(0);
    let aaa = stealth_host::glue::world_view(&sim).units;
    let base = host
        .state
        .places
        .iter()
        .find(|p| p.owner == Side::Blue && p.airfield.is_some())
        .map(|p| (p.x, p.y))
        .unwrap();
    // A few kilometres out, so the hand-over happens on the way.
    let (x, y) = (60..200)
        .flat_map(|r| (0..16).map(move |k| (r, k)))
        .map(|(r, k)| {
            let t = k as f64 / 16.0 * std::f64::consts::TAU;
            (
                base.0 + (t.cos() * r as f64 * 32.0) as i32,
                base.1 + (t.sin() * r as f64 * 32.0) as i32,
            )
        })
        .find(|&(x, y)| host.state.deploy_refusal(x, y, DefenceKind::Gepard, &aaa).is_none())
        .expect("a free spot");
    host.designate_site(&mut sim, me, x, y, DefenceKind::Gepard).unwrap();
    host.state.board_at = f64::MIN;
    host.tick(&mut sim, 0.1, &[]);
    let offer = host
        .state
        .board
        .iter()
        .find(|o| o.kind == MissionKind::Logistics)
        .cloned()
        .expect("a cargo offer");
    let Objective::Logistics { site, leg } = offer.objective else { panic!() };
    let ev = host.take_mission(&mut sim, me, offer.id, None, Aircraft::UH60, None).unwrap();
    let mut st = ev
        .iter()
        .find_map(|e| match e {
            HostEvent::SortieStarted { start, .. } => Some(start.clone()),
            _ => None,
        })
        .expect("sortie started");
    // In the air over its base, with the crate hooked.
    st.altitude = 300;
    st.on_ground = false;
    st.gear_up = true;
    st.velocity = 40 * 27;
    for _ in 0..30 {
        let ev = sim.step_pilots(&[(me.0, st.clone())], 15);
        host.tick(&mut sim, 1.0 / 15.0, &ev);
    }
    host.hook_crate(&mut sim, me, site, leg).unwrap();
    assert_eq!(host.state.deployment(site).unwrap().crate_state(leg), CrateState::Carried(me));
    let out = host.leave_aircraft(&mut sim, me).unwrap();
    let tag = out
        .iter()
        .find_map(|e| match e {
            HostEvent::Campaign(CampaignEvent::CargoLeft { tag, cargo, hooked, .. }) => {
                assert!(
                    matches!(cargo, LeftCargo::Site { id, leg: l, .. } if *id == site && *l == leg)
                );
                assert!(*hooked);
                Some(*tag)
            }
            _ => None,
        })
        .expect("left as a cargo run");
    assert!(host.state.sortie(me).is_none());
    let i = (0..sim.flights().len())
        .find(|&i| sim.campaign_flight(i).is_some_and(|f| f.tag == tag))
        .expect("its helicopter");
    let run = sim.campaign_flight(i).and_then(|f| f.supply).expect("a cargo run");
    assert!(run.hooked && run.pickup.is_none() && !run.delivered);
    let at = (sim.flights()[i].wx, sim.flights()[i].wy);
    assert_eq!(at, (st.x, 0x10_0000 - st.y), "from where he left it");
    let d = host.state.deployment(site).unwrap();
    assert!(d.ai.contains(&(leg, tag)) && !d.crate_waiting(leg));
    let mut built = false;
    for _ in 0..15 * 1200 {
        let ev = sim.step_pilots(&[], 15);
        let mut ev2 = host.take_sim_events(&mut sim);
        ev2.extend(ev);
        for e in host.tick(&mut sim, 1.0 / 15.0, &ev2) {
            match e {
                HostEvent::Campaign(CampaignEvent::SiteBuilt { id, .. }) if id == site => {
                    built = true
                }
                HostEvent::Campaign(CampaignEvent::CargoLaunched { .. }) => {
                    panic!("its crate went back to the base")
                }
                _ => {}
            }
        }
        if built {
            break;
        }
    }
    assert!(built, "{:?}", host.state.deployment(site));
}

/// A player designates a NASAMS site and fast-tracks it: its Sentinel radar and three
/// launchers of six AIM-120s stand by the helipad, mannable, with their stockpile; a loaded
/// save has them again, stockpile and all.
#[test]
fn a_fast_tracked_nasams_site_stands_and_survives_a_save() {
    use stealth_campaign::{DefenceKind, Side};
    use stealth_sim::aaa::GroundKind;
    use stealth_sim::station::{NASAMS_STOCK, StationCmd};
    let code = "central_europe";
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    let me = PlayerId(0);
    let aaa = stealth_host::glue::world_view(&sim).units;
    let base = host
        .state
        .places
        .iter()
        .find(|p| p.owner == Side::Blue && p.airfield.is_some())
        .map(|p| (p.x, p.y))
        .unwrap();
    let (x, y) = (1..60)
        .flat_map(|r| (0..16).map(move |k| (r, k)))
        .map(|(r, k)| {
            let t = k as f64 / 16.0 * std::f64::consts::TAU;
            (
                base.0 + (t.cos() * r as f64 * 32.0) as i32,
                base.1 + (t.sin() * r as f64 * 32.0) as i32,
            )
        })
        .find(|&(x, y)| host.state.deploy_refusal(x, y, DefenceKind::Nasams, &aaa).is_none())
        .expect("a free spot");
    host.designate_site(&mut sim, me, x, y, DefenceKind::Nasams).unwrap();
    let d = host.state.deployments[0].clone();
    assert_eq!((d.kind, d.loads), (DefenceKind::Nasams, 2));
    host.state.blue.materiel = 1e6;
    host.fast_track_site(&mut sim, me, d.id).unwrap();
    host.tick(&mut sim, 0.1, &[]);
    let spot = host.state.deployments[0].unit_at();
    let radar = (0..sim.units().len())
        .find(|&k| sim.units()[k].kind == GroundKind::NasamsRadar && sim.aaa_blue(k))
        .expect("the Sentinel radar");
    assert_eq!((sim.units()[radar].x, sim.units()[radar].y), spot);
    let launchers = sim.battery_launchers(radar);
    assert_eq!(launchers.len(), 3);
    assert!(launchers.iter().all(|&l| sim.units()[l].rounds == 6 && sim.aaa_blue(l)));
    sim.station_command(me.0, StationCmd::Man { unit: radar as u16 }).unwrap();
    assert_eq!(sim.stockpile(radar), NASAMS_STOCK);
    sim.station_command(me.0, StationCmd::Leave).unwrap();
    // A loaded save.
    let json = serde_json::to_string(&host.state).unwrap();
    let state = serde_json::from_str(&json).unwrap();
    let mut again = host.new_sim(&sim_data);
    let mut host2 = Host::resume(state, &data, Rules::default()).unwrap();
    host2.begin(&mut again);
    let k = (0..again.units().len())
        .find(|&k| {
            let u = again.units()[k];
            u.kind == GroundKind::NasamsRadar && again.aaa_blue(k) && (u.x, u.y) == spot
        })
        .expect("the radar after the load");
    assert_eq!(again.battery_launchers(k).len(), 3);
}

/// Both sides send an armour column at once; they drive, fight and end, one at a time each.
#[test]
fn convoys_drive_fight_and_take_places() {
    let s = setup("ukraine");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "ukraine").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.state.convoys_due();
    // Quicker than on the road, for the test.
    sim.set_convoy_speed(80.0);
    let mut launched = Vec::new();
    let mut ended = Vec::new();
    let mut shots = 0;
    for frame in 0..15 * 60 * 60 {
        let ev = sim.step_pilots(&[], 15);
        let mut ev2 = host.take_sim_events(&mut sim);
        ev2.extend(ev);
        shots += ev2
            .iter()
            .filter(|e| matches!(e.event, stealth_sim::SimEvent::GroundShot { .. }))
            .count();
        for e in host.tick(&mut sim, 1.0 / 15.0, &ev2) {
            match e {
                HostEvent::Campaign(stealth_campaign::CampaignEvent::ConvoyLaunched {
                    id,
                    side,
                    ..
                }) => launched.push((id, side, frame)),
                HostEvent::Campaign(stealth_campaign::CampaignEvent::ConvoyEnded { id }) => {
                    ended.push(id)
                }
                _ => {}
            }
        }
        let moving = host.state.convoys.iter().filter(|c| c.active()).count();
        assert!(moving <= 2, "one convoy per side");
        assert!(sim.convoys().len() <= 2);
        if ended.len() >= 2 {
            break;
        }
    }
    for c in &host.state.convoys {
        let name = |p: u16| host.state.places[p as usize].name.clone();
        eprintln!(
            "{:?} convoy {} {} -> {}: {:?}, {} left",
            c.side,
            c.id,
            name(c.from),
            name(c.place),
            c.state,
            c.units.len()
        );
    }
    eprintln!("{shots} shots");
    // A column that took its place handed over to the garrison: none of it is left.
    for c in host.state.convoys.iter().filter(|c| {
        c.state == stealth_campaign::ConvoyState::Captured
            && host.state.places[c.place as usize].owner == c.side
    }) {
        let left = (0..sim.units().len())
            .filter(|&k| sim.unit_convoy(k) == c.id && !sim.units()[k].destroyed)
            .count();
        assert_eq!(left, 0, "convoy {}", c.id);
    }
    assert!(launched.iter().any(|l| l.1 == stealth_campaign::Side::Blue));
    assert!(launched.iter().any(|l| l.1 == stealth_campaign::Side::Red));
    assert!(shots > 0);
    assert!(ended.len() >= 2, "{launched:?} {ended:?}");
}

/// A host on Central Europe, begun.
fn central_europe() -> (Host, stealth_sim::Sim) {
    let s = setup("central_europe");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "central_europe").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    (host, sim)
}

#[test]
fn a_sortie_takes_its_stores_out_of_its_base() {
    use stealth_campaign::Munition;
    let (mut host, mut sim) = central_europe();
    let o = host.state.board.iter().find(|o| !o.kind.ai_flown()).unwrap().clone();
    let base = o.base;
    let before = host.state.available(base, Munition::Aim120);
    let stores = stealth_host::Stores {
        stations: vec![(1, 4), (0, 2), (-1, 0)],
        select: 0,
        ..Default::default()
    };
    host.take_mission(&mut sim, PlayerId(0), o.id, None, Aircraft::F16, Some(stores.clone()))
        .unwrap();
    assert_eq!(host.state.available(base, Munition::Aim120), before - 4);
    // A base out of them refuses them.
    let wing = host.join(&mut sim, "Wingman", |_| true).unwrap();
    let n = host.state.available(base, Munition::Aim120);
    host.state.draw(base, Munition::Aim120, n);
    let err = host.take_mission(&mut sim, wing, o.id, None, Aircraft::F16, Some(stores));
    assert!(err.unwrap_err().tr_in(stealth_i18n::Language::En).contains("out of stock"));
}

#[test]
fn a_cargo_aircraft_flies_munitions_from_the_production_base() {
    use stealth_campaign::{Munition, Side, dist};
    let (mut host, mut sim) = central_europe();
    let st = &host.state;
    let p = st.production_base(Side::Blue).unwrap();
    let ps = st.site(p).unwrap().clone();
    // The friendly base nearest the production base.
    let to = st
        .stocks
        .iter()
        .filter(|(b, s)| **b != p && s.side == Some(Side::Blue))
        .map(|(b, _)| *b)
        .min_by_key(|b| {
            let s = st.site(*b).unwrap();
            dist(s.x - ps.x, s.y - ps.y)
        })
        .unwrap();
    // The enemy's air defence was counted: its stocks are sized for it.
    let red: u32 = host.state.site_demand.values().map(|d| d[0] + d[1] + d[2]).sum();
    assert!(red > 0, "{:?}", host.state.site_demand);
    let n = host.state.available(to, Munition::Pac3);
    host.state.draw(to, Munition::Pac3, n);
    let mut tag = None;
    let mut landed = false;
    for _ in 0..90 * 60 * 15 {
        let ev = sim.step_pilots(&[], 15);
        let mut all = host.take_sim_events(&mut sim);
        all.extend(ev);
        for e in &all {
            if let stealth_sim::SimEvent::CampaignFlightLanded { tag: t, .. } = e.event
                && Some(t) == tag
            {
                landed = true;
            }
        }
        host.tick(&mut sim, 1.0 / 15.0, &all);
        if tag.is_none() {
            tag = host.state.cargo_runs.iter().find(|r| r.to == to).map(|r| r.tag);
        }
        if landed {
            break;
        }
    }
    let tag = tag.expect("a cargo flight was scheduled");
    assert!(landed, "it landed");
    assert!(!host.state.cargo_lost.contains(&tag));
    assert_eq!(host.state.available(to, Munition::Pac3), 12, "half a full stock delivered");
}

/// The guns, MANPADS teams and garrisons stand on land: round the places of a campaign
/// (coastal towns, towns on rivers) and round a single mission's targets (ships too).
#[test]
fn air_defences_stand_on_land() {
    use stealth_mission::{MissionKind, MissionSettings, Seed};
    let data = theater::game_data().unwrap();
    let mut ships = 0;
    for code in ["persian_gulf", "north_cape", "cuba", "ukraine", "gulf_of_finland", "suwalki"] {
        let s = CampaignSetup { roe: 2, ..setup(code) };
        let index = s.theater;
        let sim_data = theater::sim_data(index, code).unwrap();
        let mut host = Host::new(s, &data, Rules::default()).unwrap();
        let mut sim = host.new_sim(&sim_data);
        host.begin(&mut sim);
        let live = sim.units().iter().filter(|u| !u.destroyed).count();
        assert!(live > 50, "{code}: {live} units");
        for u in sim.units().iter().filter(|u| !u.destroyed) {
            assert_eq!(host.state.terrain.is_water(u.x, u.y), Some(false), "{code}: {u:?}");
        }
        // Single missions, among them ships to strike or photograph.
        let th = std::sync::Arc::new(data.theaters[index].clone());
        for seed in 0..40u16 {
            let ms = MissionSettings {
                theater: index,
                roe: (seed % 3) as u8,
                kind: MissionKind::Strike,
                ..Default::default()
            };
            let m = stealth_mission::generate(&data, &ms, Seed::Srand(seed)).unwrap();
            ships += (m.primary.weapon_class == 12) as i32;
            let opts = stealth_sim::SimOptions { air_defence: true, ..Default::default() };
            let t = th.clone();
            let water = stealth_sim::aaa::Water::new(move |x, y| t.is_water(x, y));
            let sim = stealth_sim::Sim::with_water(&sim_data, &m, opts, water);
            for u in sim.units() {
                assert!(!th.is_water(u.x, u.y), "{code} seed {seed}: {u:?}");
            }
        }
    }
    assert!(ships > 0, "no ship targets");
}

/// Coop players taking off from one carrier start on different catapults, held for the launch.
#[test]
fn coop_players_take_different_catapults() {
    use stealth_campaign::SiteKind;
    use stealth_sim::catapult::{Phase, from_report};
    let code = "persian_gulf";
    let s = setup(code);
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, code).unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    let carrier = host
        .state
        .player_bases()
        .into_iter()
        .find(|&b| host.state.site(b).is_some_and(|s| s.kind == SiteKind::Carrier))
        .expect("a carrier");
    let wing = host.join(&mut sim, "Wingman", |_| true).unwrap();
    let offers: Vec<u32> =
        host.state.board.iter().filter(|o| !o.kind.ai_flown()).map(|o| o.id).collect();
    let mut cats = Vec::new();
    for (p, offer) in [(PlayerId(0), offers[0]), (wing, offers[1])] {
        let ev = host.take_mission(&mut sim, p, offer, Some(carrier), Aircraft::F16, None).unwrap();
        let start = ev
            .iter()
            .find_map(|e| match e {
                HostEvent::SortieStarted { player, start, .. } if *player == p => {
                    Some(start.clone())
                }
                _ => None,
            })
            .expect("sortie started");
        let (cat, phase) = from_report(start.catapult).expect("on a catapult");
        assert_eq!(phase, Phase::Tension);
        let pose = sim.catapult(carrier as usize, cat);
        assert_eq!((start.x, start.y), (pose.east.round() as i32, pose.north.round() as i32));
        assert_eq!(start.heading, pose.heading);
        cats.push(cat);
    }
    assert_ne!(cats[0], cats[1], "both on catapult {}", cats[0]);
}

/// The campaign and the simulation agree on the wingmen's tag.
#[test]
fn wingman_tags_agree() {
    assert_eq!(stealth_campaign::WING_TAG, stealth_sim::wingman::WING_TAG);
}

/// A player taking a wingman along draws its stores from his base's stock too, and the
/// wingman joins him once he is up.
#[test]
fn a_wingman_draws_its_stores_from_the_base() {
    let (mut host, mut sim) = central_europe();
    let p = PlayerId(0);
    let o = host.state.board.iter().find(|o| !o.kind.ai_flown()).unwrap().clone();
    let stores = stealth_host::Stores {
        stations: vec![(1, 2), (0, 2), (-1, 0)],
        select: 0,
        ..Default::default()
    };
    let before = host.state.available(o.base, stealth_campaign::Munition::Aim120);
    host.take_mission(&mut sim, p, o.id, None, Aircraft::F16, Some(stores)).unwrap();
    host.take_wingman(&mut sim, p, Aircraft::F16).unwrap();
    let after = host.state.available(o.base, stealth_campaign::Munition::Aim120);
    assert_eq!(before - after, 4, "his two and the wingman's two");
    assert!(sim.wingman_pending(p.0));
}

/// An Iskander strike on Ukraine plays out under the logistics rules: the enemy posts an
/// Iskander site, Mi-24s fly its loads in from a red airfield, the built site's TEL is on the
/// board; the next strike's TEL drives out from the site with missiles from the site's supply
/// base, its hunt goes on the board while it prepares, it fires, and the missiles come down or
/// are shot down.
#[test]
fn an_iskander_strike_plays_out() {
    use stealth_campaign::missile_strikes::{StrikeMissile, StrikeState};
    use stealth_campaign::{DefenceKind, Munition, Objective, Side, dist};
    let s = setup("ukraine");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "ukraine").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.state.iskander_at = -1e7;
    host.state.kh101_at = 1e9;
    host.tick(&mut sim, 0.1, &[]);
    let site = host
        .state
        .deployments
        .iter()
        .find(|d| d.kind == DefenceKind::IskanderSite)
        .cloned()
        .expect("an Iskander site posted");
    assert_eq!(site.side(), Side::Red);
    assert!(host.state.missile_strikes.is_empty(), "a strike without a site");
    let base = site.base.expect("a red supply base");
    assert_eq!(host.state.owner(base), Some(Side::Red));
    // Not the players' cargo to fly.
    assert!(
        !host
            .state
            .board
            .iter()
            .any(|o| matches!(o.objective, Objective::Logistics { site: id, .. } if id == site.id)),
        "the enemy's cargo on the friendly board"
    );
    let stock = host.state.available(base, Munition::Iskander);
    let (mut hinds, mut site_hunted, mut tel_hunted, mut warned) = (0, false, false, false);
    let mut seen_tags = Vec::new();
    for _ in 0..15 * 60 * 60 * 6 {
        let e = sim.step_pilots(&[], 15);
        let mut e2 = host.take_sim_events(&mut sim);
        e2.extend(e);
        host.tick(&mut sim, 1.0 / 15.0, &e2);
        for (i, u) in sim.flights().iter().enumerate() {
            let Some(f) = sim.campaign_flight(i) else { continue };
            if f.supply.is_some() && !f.blue && !seen_tags.contains(&f.tag) {
                assert_eq!(u.spec, 16, "the enemy's cargo helicopter is a Mi-24");
                seen_tags.push(f.tag);
                hinds += 1;
            }
        }
        let built = host.state.deployment(site.id).is_some_and(|d| d.built && d.seen);
        if built && !site_hunted {
            site_hunted = host.state.board.iter().any(
                |o| matches!(o.objective, Objective::Battery { battery, .. } if battery == site.id),
            );
            // Its first strike is due.
            host.state.iskander_at = -1e7;
        }
        tel_hunted |= host.state.board.iter().any(|o| {
            matches!(o.objective, Objective::Battery { battery, .. }
                if host.state.missile_strike(battery).is_some())
        });
        warned |= host.state.log.iter().any(|l| format!("{:?}", l.text).contains("cl-tbm-launch"));
        if let Some(s) = host
            .state
            .missile_strikes
            .iter()
            .find(|s| s.missile == StrikeMissile::Iskander && s.state == StrikeState::Ended)
        {
            assert!(hinds >= 1, "no Mi-24 flew the site's cargo");
            assert!(site_hunted, "the built site not on the board");
            assert!(tel_hunted, "no TEL hunt on the board");
            assert!(warned || s.hits + s.down < s.count, "no launch warning");
            assert!(dist(s.from.0 - site.x, s.from.1 - site.y) == 0, "the TEL not from its site");
            assert_eq!(s.base, base);
            let left = host.state.available(base, Munition::Iskander);
            assert!(left + s.count as u32 <= stock + 1, "missiles not drawn: {stock} -> {left}");
            return;
        }
    }
    panic!("{:?} {:?}", host.state.missile_strikes, host.state.deployment(site.id));
}

/// A Kh-101 raid on Ukraine plays out: the Tu-95 flies to its launch point, the missiles are
/// announced, and they come down or are shot down.
#[test]
fn a_kh101_strike_plays_out() {
    use stealth_campaign::missile_strikes::{StrikeMissile, StrikeState};
    let s = setup("ukraine");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "ukraine").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.state.iskander_at = 1e9;
    host.state.kh101_at = -1e7;
    let mut inbound = false;
    for _ in 0..15 * 60 * 120 {
        let e = sim.step_pilots(&[], 15);
        let mut e2 = host.take_sim_events(&mut sim);
        e2.extend(e);
        host.tick(&mut sim, 1.0 / 15.0, &e2);
        inbound |= host
            .state
            .missile_strikes
            .iter()
            .any(|s| matches!(s.state, StrikeState::Inbound { .. }));
        if let Some(s) = host
            .state
            .missile_strikes
            .iter()
            .find(|s| s.missile == StrikeMissile::Kh101 && s.state == StrikeState::Ended)
        {
            assert!(inbound, "never fired: {s:?}");
            assert_eq!(s.hits + s.down, s.count, "{s:?}");
            return;
        }
    }
    panic!("{:?}", host.state.missile_strikes);
}

/// On Ukraine the war posts a friendly FP-7 launch site well clear of the airfields; built, it
/// sets up and fires at the enemy's military targets, its missiles coming down or shot down.
#[test]
fn a_launch_site_is_built_and_fires() {
    use stealth_campaign::missile_strikes::{StrikeMissile, StrikeState};
    use stealth_campaign::{DefenceKind, SiteKind, dist, tuning};
    let s = setup("ukraine");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "ukraine").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.state.iskander_at = 1e9;
    host.state.kh101_at = 1e9;
    host.tick(&mut sim, 0.1, &[]);
    let d = host
        .state
        .deployments
        .iter()
        .find(|d| d.kind == DefenceKind::Fp7Site)
        .cloned()
        .expect("an FP-7 site posted");
    assert!(d.by.is_none());
    assert!(
        host.state.sites.iter().filter(|s| s.kind == SiteKind::Airfield).all(|s| dist(
            s.x - d.x,
            s.y - d.y
        )
            >= tuning::LAUNCH_SITE_AIRFIELD_CLEAR),
        "too near an airfield"
    );
    host.fast_track_site(&mut sim, PlayerId(0), d.id).unwrap();
    let mut fired = false;
    for _ in 0..15 * 60 * 40 {
        let e = sim.step_pilots(&[], 15);
        let mut e2 = host.take_sim_events(&mut sim);
        e2.extend(e);
        host.tick(&mut sim, 1.0 / 15.0, &e2);
        let strikes = &host.state.missile_strikes;
        fired |= strikes
            .iter()
            .any(|s| s.site == Some(d.id) && matches!(s.state, StrikeState::Inbound { .. }));
        if let Some(s) = strikes
            .iter()
            .find(|s| s.missile == StrikeMissile::Fp7 && s.state == StrikeState::Ended)
        {
            assert!(fired);
            assert_eq!(s.hits + s.down, s.count, "{s:?}");
            return;
        }
    }
    panic!("{:?}", host.state.missile_strikes);
}

fn ukraine() -> (Host, stealth_sim::Sim) {
    let s = setup("ukraine");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "ukraine").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    host.tick(&mut sim, 0.1, &[]);
    (host, sim)
}

fn airfield(host: &Host, name: &str) -> stealth_campaign::ObjectId {
    host.state
        .sites
        .iter()
        .find(|s| s.kind == stealth_campaign::SiteKind::Airfield && s.name == name)
        .unwrap()
        .object
}

/// A red base on R-280 runs low: a truck convoy drives from Rostov and delivers.
#[test]
fn a_supply_convoy_drives_munitions_to_taganrog() {
    use stealth_campaign::Munition;
    let (mut host, mut sim) = ukraine();
    let to = airfield(&host, "Taganrog");
    // Something it keeps and the production base has made.
    let p = host.state.production_base(stealth_campaign::Side::Red).unwrap();
    let m = *Munition::of_side(stealth_campaign::Side::Red)
        .iter()
        .find(|&&m| host.state.full_stock(to, m) >= 4.0 && host.state.available(p, m) >= 4)
        .expect("a munition to bring");
    let n = host.state.available(to, m);
    host.state.draw(to, m, n);
    // Fast trucks (a test aid): the 40 km take a few minutes.
    sim.set_convoy_speed(200.0);
    let mut tag = None;
    let mut arrived = None;
    let mut trucks_seen = 0;
    for _ in 0..30 * 60 * 15 {
        let ev = sim.step_pilots(&[], 15);
        let mut all = host.take_sim_events(&mut sim);
        all.extend(ev);
        for e in &all {
            if let stealth_sim::SimEvent::SupplyConvoyArrived { tag: t, trucks } = e.event
                && Some(t) == tag
            {
                arrived = Some(trucks);
            }
        }
        host.tick(&mut sim, 1.0 / 15.0, &all);
        if tag.is_none() {
            tag = host.state.cargo_runs.iter().find(|r| r.to == to && r.road.is_some()).map(|r| r.tag);
        }
        trucks_seen = trucks_seen.max(sim.units().iter().filter(|u| u.kind.truck() && !u.destroyed).count());
        if arrived.is_some() {
            break;
        }
    }
    tag.expect("a supply convoy was scheduled");
    assert_eq!(arrived, Some(3), "all three trucks got there");
    assert!(trucks_seen >= 3);
    assert!(host.state.available(to, m) > 0, "delivered");
    assert!(host.state.cargo_runs.iter().all(|r| r.to != to || r.road.is_none() || Some(r.tag) != tag));
}

/// The Kerch bridge's road arch brought down cuts the road for the campaign.
#[test]
fn the_kerch_road_arch_down_cuts_the_road() {
    let (mut host, mut sim) = ukraine();
    let net = host.state.road_net().unwrap().clone();
    let edge = net.edges.iter().position(|e| e.arch.is_some()).unwrap();
    let (x, y, shape) = net.edges[edge].arch.unwrap();
    sim.wreck_tile_at(shape, [x, y]);
    let ev = host.take_sim_events(&mut sim);
    assert!(ev.iter().any(|e| matches!(e.event, stealth_sim::SimEvent::TileDestroyed(_))));
    host.tick(&mut sim, 1.0 / 15.0, &ev);
    assert_eq!(host.state.cut_roads, [edge]);
}

/// The strategic facilities stand in the simulation as buildings; one destroyed there is out of
/// action in the campaign, and stands again when its days are up.
#[test]
fn facilities_stand_fall_and_are_rebuilt() {
    let s = setup("central_europe");
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(s.theater, "central_europe").unwrap();
    let mut host = Host::new(s, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    let n = host.state.facilities.len();
    assert!(n >= 10, "{n} facilities");
    let buildings = sim.units().iter().filter(|u| u.kind.structure()).count();
    assert_eq!(buildings, n);
    for f in &host.state.facilities {
        let k = host.facility_unit(f.id).unwrap();
        let u = sim.units()[k];
        assert_eq!((u.x, u.y), (f.x, f.y));
        assert_eq!(sim.aaa_blue(k), host.state.facility_side(f) == stealth_campaign::Side::Blue);
    }
    let f = host
        .state
        .facilities_of(stealth_campaign::Side::Red, stealth_campaign::FacilityKind::PowerPlant)
        .next()
        .unwrap()
        .clone();
    let k = host.facility_unit(f.id).unwrap();
    sim.set_structure_destroyed(k, true);
    let hit = stealth_sim::PilotEvent {
        pilot: 0,
        event: stealth_sim::SimEvent::AaaDestroyed { unit: k, by: Some(0) },
    };
    host.tick(&mut sim, 1.0, &[hit]);
    assert!(!host.state.facility(f.id).unwrap().alive());
    let days = host.state.game_days(stealth_campaign::tuning::POWER_PLANT_DAYS);
    for _ in 0..(days / 60.0) as usize + 2 {
        host.tick(&mut sim, 60.0, &[]);
    }
    assert!(host.state.facility(f.id).unwrap().alive());
    assert!(!sim.units()[k].destroyed);
}

/// The campaign's drone loads are the drone command center's (the simulation's).
#[test]
fn a_drone_load_is_a_drone_command_center_s() {
    use stealth_sim::station::{FP2_STOCK, INTERCEPTOR_STOCK, STORK_STOCK, X12_STOCK};
    let stocks = [STORK_STOCK, FP2_STOCK, INTERCEPTOR_STOCK, X12_STOCK].map(|n| n as u32);
    assert_eq!(stealth_campaign::tuning::DRONE_CENTER_STOCK, stocks);
}
