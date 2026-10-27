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
    let flown: Vec<u32> =
        host.state.board.iter().filter(|o| !o.kind.ai_flown()).map(|o| o.id).collect();
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
    use stealth_sim::aaa::AaaKind;
    for code in ["central_europe", "ukraine"] {
        let s = setup(code);
        let data = theater::game_data().unwrap();
        let sim_data = theater::sim_data(s.theater, code).unwrap();
        let mut host = Host::new(s, &data, Rules::default()).unwrap();
        let mut sim = host.new_sim(&sim_data);
        host.begin(&mut sim);
        let blue = |sim: &stealth_sim::Sim, kind| {
            (0..sim.aaa_units().len())
                .filter(|&k| sim.aaa_units()[k].kind == kind && sim.aaa_blue(k))
                .count()
        };
        let batteries = host.state.patriot_sites().len();
        assert!(batteries > 0, "{code}");
        assert_eq!(blue(&sim, AaaKind::PatriotRadar), batteries, "{code}");
        assert_eq!(blue(&sim, AaaKind::PatriotLauncher), 3 * batteries, "{code}");
        assert!(blue(&sim, AaaKind::Gepard) > 0, "{code}");
        assert_eq!(blue(&sim, AaaKind::Zsu), 0, "{code}");
        // No two batteries in one place, on a second begin neither.
        host.begin(&mut sim);
        assert_eq!(blue(&sim, AaaKind::PatriotRadar), batteries, "{code}");
    }
}

/// The enemy's offensive played out in the simulation: its raid and its insertion end one way
/// or the other (the strike aircraft bomb or are stopped, the Hinds land or are shot down).
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
        if done(raid.map(|r| matches!(r, RaidState::Hit | RaidState::Repelled)))
            && done(ins.map(|i| matches!(i, InsertionState::Captured | InsertionState::Failed)))
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
    use stealth_sim::aaa::AaaKind;
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
            let gepard = (0..sim.aaa_units().len()).find(|&k| {
                let u = sim.aaa_units()[k];
                u.kind == AaaKind::Gepard
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
    use stealth_sim::aaa::AaaKind;
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
    let k = (0..sim.aaa_units().len())
        .find(|&k| sim.aaa_units()[k].kind == AaaKind::Gepard && sim.aaa_blue(k))
        .unwrap();
    let u = sim.aaa_units()[k];
    sim.restore_stocks(&[SavedStock { x: u.x, y: u.y, patriot: false, stock: 0 }]);
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
    use stealth_sim::aaa::AaaKind;
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
    let radar = (0..sim.aaa_units().len())
        .find(|&k| sim.aaa_units()[k].kind == AaaKind::PatriotRadar && sim.aaa_blue(k))
        .expect("a friendly Patriot battery");
    let r = sim.aaa_units()[radar];
    // One missile left in the stockpile, one launcher fired empty.
    sim.restore_stocks(&[SavedStock { x: r.x, y: r.y, patriot: true, stock: 1 }]);
    let l = sim.battery_launchers(radar)[0];
    sim.aaa_units_mut()[l].rounds = 0;
    host.tick(&mut sim, 0.1, &[]);
    assert!(host.state.supply_requests.is_empty());
    // The battery's own refill takes the last one.
    host.refills(&mut sim, 1.0);
    host.refills(&mut sim, stealth_sim::station::REFILL_SECONDS);
    assert_eq!(sim.stockpile(radar), 0);
    let ev = host.tick(&mut sim, 0.1, &[]);
    assert!(ev.iter().any(|e| matches!(e,
        HostEvent::Campaign(stealth_campaign::CampaignEvent::Message(m)) if m.contains("out of missiles"))));
    assert_eq!(host.state.supply_requests.len(), 1);
    let q = host.state.supply_requests[0].clone();
    assert_eq!((q.aaa as usize, q.by, q.patriot), (radar, None, true));
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
