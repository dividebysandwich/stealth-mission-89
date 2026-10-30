//! The campaign's ground war: convoys driving, seeing and fighting.

use std::sync::OnceLock;

use stealth_mission::{GameData, Mission, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::GroundKind;
use stealth_sim::ground::GroundGun;
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
    // A tank fires a round every 5 to 9 s; the autocannons fire bursts a few tenths apart.
    let mut last: Vec<Option<usize>> = vec![None; sim.units().len()];
    let mut frame = 0;
    let ps = idle();
    let (_, mut again) = campaign();
    again.spawn_convoy(1, true, &blue, &[(X, Y), (X + 30 * KM, Y)]);
    again.spawn_convoy(2, false, &red, &[(X + 12 * KM, Y), (X - 20 * KM, Y)]);
    for _ in 0..600 * 15 {
        for e in again.step(&ps, 15) {
            if let SimEvent::GroundShot { from, gun, kill, hit, .. } = e {
                assert!(!kill || hit);
                if let Some(t) = last[from] {
                    let gap = frame - t;
                    match gun {
                        GroundGun::Tank => assert!((70..=140).contains(&gap), "{gap} frames"),
                        _ => assert!(gap >= 1),
                    }
                }
                last[from] = Some(frame);
            }
        }
        frame += 1;
    }
    // They stopped to fight: neither got anywhere near the other's start.
    assert!(metres(&sim, 0)[0] < metres(&sim, 4)[0] + 100.0);
}

/// Two infantry fighting vehicles 1.5 km apart fire bursts of 3 to 5 at each other, a dozen
/// hits or so to knock one out.
#[test]
fn infantry_fighting_vehicles_fire_bursts() {
    let (_, mut sim) = campaign();
    sim.spawn_convoy(1, true, &[GroundKind::Bradley], &[(X, Y), (X + KM, Y)]);
    sim.spawn_convoy(2, false, &[GroundKind::Bmp2], &[(X + 3 * KM / 2, Y), (X + 3 * KM, Y)]);
    sim.end_convoy(1);
    sim.end_convoy(2);
    let ps = idle();
    let mut rounds: Vec<(usize, i32, bool)> = Vec::new();
    for frame in 0..300 * 15 {
        for e in sim.step(&ps, 15) {
            if let SimEvent::GroundShot { from, gun, hit, .. } = e {
                assert_eq!(gun, GroundGun::Cannon);
                rounds.push((from, frame, hit));
            }
        }
        if sim.units().iter().any(|u| u.destroyed) {
            break;
        }
    }
    let loser = (0..2).find(|&k| sim.units()[k].destroyed).expect("one knocked out");
    let hits = rounds.iter().filter(|r| r.0 != loser && r.2).count();
    assert!((8..=16).contains(&hits), "{hits} hits");
    // Bursts: rounds 4 or 5 frames apart, then a pause of seconds.
    let mine: Vec<i32> = rounds.iter().filter(|r| r.0 != loser).map(|r| r.1).collect();
    let gaps: Vec<i32> = mine.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(gaps.iter().all(|&g| (3..=5).contains(&g) || g >= 30), "{gaps:?}");
    let pauses = gaps.iter().filter(|&&g| g >= 30).count();
    let per_burst = mine.len() as f64 / (pauses + 1) as f64;
    assert!((3.0..=5.0).contains(&per_burst), "{per_burst}");
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
    // A vehicle knocked out on the march: its wreck stays where it was hit, on the client too.
    host.units_mut()[1].destroyed = true;
    for _ in 0..3 {
        run(&mut host, 1);
        client.apply_world_slow(&host.world_slow(&mut base));
        client.apply_world_fast(&host.world_fast());
    }
    assert!(client.units()[1].destroyed);
    assert_eq!(client.units()[1].fine, host.units()[1].fine);
    assert_eq!(client.units()[0].fine, host.units()[0].fine);
}

fn sim_mirror(_host: &mut Sim, client: &mut Sim) {
    client.units_mut().clear();
}

/// The targeting pod's lock on a vehicle: the designated point drives with it.
#[test]
fn a_designated_vehicle_carries_the_point_with_it() {
    let (_, mut sim) = campaign();
    sim.spawn_convoy(4, false, &[GroundKind::T90], &[(X, Y), (X + 30 * KM, Y)]);
    let (x, y) = Sim::map_to_world(X + 2 * KM, Y + KM);
    let mut p = PlayerState {
        x,
        y,
        altitude: 3000,
        height: 3000,
        velocity: 300 * 27,
        gear_up: true,
        fuel: 10000,
        hud_visible: true,
        hud_mode: HudMode::Gnd,
        ..Default::default()
    };
    // In the world (off a friendly airfield), then over the tank.
    let base = (3..sim.objects().len())
        .find(|&i| {
            sim.objects()[i].flags & (oflags::AIRFIELD | oflags::FRIENDLY)
                == oflags::AIRFIELD | oflags::FRIENDLY
        })
        .expect("a friendly airfield");
    sim.respawn_player(base, None);
    sim.step(&p, 15);
    p.cmd.designate_unit = Some(0);
    sim.step(&p, 15);
    p.cmd = Commands::default();
    assert_eq!(sim.targeting().ground, Some(0), "{:?}", sim.targeting());
    let start = sim.objects()[0];
    for _ in 0..15 * 10 {
        sim.step(&p, 15);
    }
    let now = sim.objects()[0];
    let u = sim.units()[0];
    assert_eq!((now.x, now.y), (u.x, u.y));
    // 200 m east in ten seconds.
    assert!((now.x - start.x - 13).abs() <= 1, "{start:?} -> {now:?}");
    assert_eq!(sim.targeting().ground, Some(0));
}

/// A column that took its place hands over to the place's garrison: its vehicles leave the
/// world (no wrecks) and the new owner's usual garrison stands there in their slots.
#[test]
fn a_column_that_took_its_place_hands_over_to_the_garrison() {
    let (_, mut sim) = campaign();
    let (gx, gy) = (X + 10 * KM, Y);
    let blue = [
        GroundKind::Leopard2,
        GroundKind::Leopard2,
        GroundKind::Bradley,
        GroundKind::Bradley,
        GroundKind::Gepard,
    ];
    sim.spawn_convoy(5, true, &blue, &[(gx - KM, gy), (gx, gy)]);
    run(&mut sim, 5);
    sim.end_convoy(5);
    let column: Vec<usize> = (0..sim.units().len()).filter(|&k| sim.unit_convoy(k) == 5).collect();
    assert_eq!(column.len(), 5);
    let before = sim.units().len();
    sim.convoys_hand_over(gx, gy, true);
    sim.place_defences(gx, gy, false, true);
    // The column is gone (not wrecks) or its slots hold the garrison now.
    for &k in &column {
        let u = sim.units()[k];
        assert!(u.gone && u.destroyed || sim.unit_convoy(k) == 0, "{k}: {u:?}");
    }
    let near = |k: usize| {
        let u = sim.units()[k];
        (u.x - gx).abs() < 0x180 && (u.y - gy).abs() < 0x180
    };
    let live: Vec<GroundKind> = (0..sim.units().len())
        .filter(|&k| near(k) && !sim.units()[k].destroyed)
        .map(|k| sim.units()[k].kind)
        .collect();
    assert_eq!(live.iter().filter(|k| k.tank()).count(), 2, "{live:?}");
    assert_eq!(live.iter().filter(|k| k.carrier()).count(), 1, "{live:?}");
    assert!(sim.units().len() <= before + 1, "the slots were reused");
}
