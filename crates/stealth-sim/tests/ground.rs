//! The campaign's ground war: convoys driving, seeing and fighting.

use std::sync::OnceLock;

use stealth_mission::{GameData, Mission, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::GroundKind;
use stealth_sim::*;

struct Data {
    sim: SimData,
    gdata: GameData,
}

fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let sim = SimData::load_json(&assets.join("sim")).expect("assets/sim");
        let gdata = GameData::load_json(&assets.join("mission")).expect("assets/mission");
        Data { sim, gdata }
    })
}

/// A Central Europe campaign simulation without any air defences.
fn campaign() -> (Mission, Sim) {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 1,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(78)).unwrap();
    m.world = d.gdata.theaters[3].world.clone();
    m.weather = Default::default();
    let mut sim =
        Sim::with_options(&d.sim, &m, SimOptions { campaign: true, ..Default::default() });
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
    }
    sim.units_mut().clear();
    (m, sim)
}

fn idle() -> PlayerState {
    PlayerState { on_ground: true, fuel: 10000, ..Default::default() }
}

fn run(sim: &mut Sim, secs: i32) -> Vec<SimEvent> {
    let ps = idle();
    let mut ev = Vec::new();
    for _ in 0..secs * 15 {
        ev.extend(sim.step(&ps, 15));
    }
    ev
}

/// Open country in the north-west of Central Europe (no relief between these points).
const Y: i32 = 0x1400;
const X: i32 = 0x0C00;
/// Map units per kilometre.
const KM: i32 = 64;

fn metres(sim: &Sim, k: usize) -> [f64; 2] {
    let m = sim.units()[k].metres();
    [m[0], m[1]]
}

#[test]
fn a_convoy_drives_its_route_at_road_speed() {
    let (_, mut sim) = campaign();
    let kinds = [GroundKind::Leopard2, GroundKind::Gepard, GroundKind::Bradley];
    sim.spawn_convoy(7, true, &kinds, &[(X, Y), (X + 40 * KM, Y)]);
    assert_eq!(sim.convoys().len(), 1);
    assert_eq!(sim.units().len(), 3);
    let start = metres(&sim, 0);
    // The column strings out behind its head, 60 m apart, facing east.
    assert!((start[0] - metres(&sim, 2)[0] - 120.0).abs() < 1.0);
    run(&mut sim, 60);
    let head = metres(&sim, 0);
    assert!((head[0] - start[0] - 1200.0).abs() < 25.0, "{head:?} from {start:?}");
    assert!((head[1] - start[1]).abs() < 1.0);
    let f = sim.units()[0].fine.unwrap();
    assert!((f.heading - std::f32::consts::FRAC_PI_2).abs() < 1e-3);
    assert!(sim.unit_moving(1) && sim.unit_convoy(1) == 7);
    sim.end_convoy(7);
    run(&mut sim, 5);
    assert_eq!(metres(&sim, 0), head);
}

#[test]
fn ground_units_see_within_sight_range() {
    let (_, mut sim) = campaign();
    sim.spawn_convoy(1, true, &[GroundKind::Leopard2], &[(X, Y), (X + KM, Y)]);
    sim.spawn_convoy(2, false, &[GroundKind::T90], &[(X + 3 * KM, Y), (X + 4 * KM, Y)]);
    sim.spawn_convoy(3, false, &[GroundKind::Bmp2], &[(X + 9 * KM, Y), (X + 10 * KM, Y)]);
    sim.end_convoy(1);
    sim.end_convoy(2);
    sim.end_convoy(3);
    run(&mut sim, 2);
    assert!(sim.unit_seen_by(1, true), "T-90 at 3 km");
    assert!(!sim.unit_seen_by(2, true), "BMP-2 at 9 km");
    assert!(sim.unit_seen_by(0, false));
    assert!(sim.unit_seen_by(0, true), "own side");
}

#[test]
fn opposing_columns_halt_and_fight() {
    let (_, mut sim) = campaign();
    let blue =
        [GroundKind::Leopard2, GroundKind::Leopard2, GroundKind::Bradley, GroundKind::Gepard];
    let red = [GroundKind::T90, GroundKind::T90, GroundKind::Bmp2, GroundKind::Zsu];
    sim.spawn_convoy(1, true, &blue, &[(X, Y), (X + 30 * KM, Y)]);
    sim.spawn_convoy(2, false, &red, &[(X + 12 * KM, Y), (X - 20 * KM, Y)]);
    let ev = run(&mut sim, 600);
    let shots = ev.iter().filter(|e| matches!(e, SimEvent::GroundShot { .. })).count();
    let kills: Vec<usize> = ev
        .iter()
        .filter_map(|e| match *e {
            SimEvent::AaaDestroyed { unit, by: None } => Some(unit),
            _ => None,
        })
        .collect();
    assert!(shots > 4, "{shots} shots");
    assert!(!kills.is_empty());
    // Every kill is the other side's: no one fires on his own.
    for e in &ev {
        if let SimEvent::GroundShot { from, to, .. } = *e {
            assert_ne!(sim.aaa_blue(from), sim.aaa_blue(to));
        }
    }
    // They stopped to fight: neither got anywhere near the other's start.
    assert!(metres(&sim, 0)[0] < metres(&sim, 4)[0] + 100.0);
}

#[test]
fn a_garrison_fights_the_convoy_that_attacks_it() {
    let (_, mut sim) = campaign();
    let (gx, gy) = (X + 10 * KM, Y);
    sim.place_defences(gx, gy, false, false);
    let garrison: Vec<usize> =
        (0..sim.units().len()).filter(|&k| sim.units()[k].kind.armour()).collect();
    assert_eq!(garrison.len(), 3);
    assert!(garrison.iter().all(|&k| !sim.aaa_blue(k)));
    let first = sim.units().len();
    let blue =
        [GroundKind::Leopard2, GroundKind::Leopard2, GroundKind::Leopard2, GroundKind::Bradley];
    sim.spawn_convoy(5, true, &blue, &[(X, Y), (gx, gy)]);
    let ev = run(&mut sim, 900);
    assert!(
        ev.iter()
            .any(|e| matches!(*e, SimEvent::GroundShot { from, .. } if garrison.contains(&from)))
    );
    assert!(ev.iter().any(|e| matches!(*e, SimEvent::GroundShot { from, .. } if from >= first)));
    // Whoever is left of the convoy closed in on the place.
    let live: Vec<usize> =
        (first..sim.units().len()).filter(|&k| !sim.units()[k].destroyed).collect();
    let goal = stealth_sim::ground::metres_of(gx, gy);
    for k in live {
        let p = metres(&sim, k);
        assert!((p[0] - goal[0]).hypot(p[1] - goal[1]) < 2100.0, "{k} at {p:?}");
    }
}

#[test]
fn a_client_mirrors_the_convoys() {
    let (_, mut host) = campaign();
    let (_, mut client) = campaign();
    sim_mirror(&mut host, &mut client);
    host.spawn_convoy(3, false, &[GroundKind::T90, GroundKind::Bmp2], &[(X, Y), (X, Y + 30 * KM)]);
    let mut base = stealth_sim::net::SlowBase::default();
    for _ in 0..40 {
        run(&mut host, 1);
        client.apply_world_slow(&host.world_slow(&mut base));
        client.apply_world_fast(&host.world_fast());
    }
    assert_eq!(client.units().len(), 2);
    for k in 0..2 {
        assert_eq!(client.units()[k].fine, host.units()[k].fine);
        assert_eq!(
            (client.units()[k].x, client.units()[k].y),
            (host.units()[k].x, host.units()[k].y)
        );
    }
    assert!(client.unit_moving(0));
}

fn sim_mirror(_host: &mut Sim, client: &mut Sim) {
    client.units_mut().clear();
}
