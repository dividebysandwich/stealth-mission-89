//! Campaign: a drone command center's drones (`uav`).

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{AaaKind, AaaUnit, M_PER_MAP};
use stealth_sim::station::{FP2_STOCK, STORK_STOCK, StationCmd};
use stealth_sim::uav::{MAX_AIRBORNE, Seen, UavKind, UavMode, UavWp, WpKind};
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

const ME: u8 = 0;

/// A campaign with no SAM sites and no guns, a drone command center by Neuburg manned: the
/// sim, the center, a SAM site object (its radar type cleared) and its radar type.
fn center() -> (Sim, usize, usize, i32) {
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
    let site = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].unit_type > 0 && sim.objects()[i].flags & 0x400 == 0)
        .expect("an enemy site");
    let radar = sim.objects()[site].unit_type;
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
    }
    sim.aaa_units_mut().clear();
    let home = (3..sim.objects().len())
        .find(|&i| {
            sim.objects()[i].flags & 0xF0F == 0x401
                && sim.place_name(i).eq_ignore_ascii_case("Neuburg")
        })
        .expect("Neuburg");
    let o = sim.objects()[home];
    sim.place_drone_center(o.x + 0x40, o.y);
    let k = sim.aaa_units().iter().position(|u| u.kind == AaaKind::DroneCenter).unwrap();
    sim.station_command(ME, StationCmd::Man { unit: k as u16 }).unwrap();
    (sim, k, site, radar)
}

fn idle() -> PlayerState {
    PlayerState { on_ground: true, fuel: 10000, ..Default::default() }
}

fn run(sim: &mut Sim, secs: i32) -> Vec<SimEvent> {
    let mut ev = Vec::new();
    for _ in 0..secs * 15 {
        ev.extend(sim.step(&idle(), 15));
    }
    ev
}

/// A map point `east`, `north` metres from AAA unit `k`.
fn off(sim: &Sim, k: usize, east: f64, north: f64) -> (i32, i32) {
    let u = sim.aaa_units()[k];
    (u.x + (east / M_PER_MAP) as i32, u.y - (north / M_PER_MAP) as i32)
}

fn wp((x, y): (i32, i32), alt: f32, kind: WpKind) -> UavWp {
    UavWp { x, y, alt, kind }
}

fn launch(sim: &mut Sim, kind: UavKind, route: Vec<UavWp>) -> Result<(), String> {
    sim.station_command(ME, StationCmd::Launch { kind, route })
}

#[test]
fn a_stork_climbs_off_the_rail_and_circles_its_loiter_point() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 2500.0, 0.0);
    launch(&mut sim, UavKind::Stork, vec![wp(at, 600.0, WpKind::Loiter)]).unwrap();
    assert_eq!(sim.uav_stock(k), [STORK_STOCK - 1, FP2_STOCK]);
    run(&mut sim, 1);
    let f = sim.uavs()[0].flight as usize;
    assert_eq!(sim.flights()[f].spec, UavKind::Stork.spec());
    assert!(sim.flights()[f].flags & fflags::AIRBORNE != 0);
    run(&mut sim, 300);
    let r = UavKind::Stork.perf().loiter_radius;
    let c = wp(at, 0.0, WpKind::Nav).metres();
    for _ in 0..30 {
        run(&mut sim, 4);
        let u = &sim.uavs()[0];
        let d = (u.pos[0] - c[0]).hypot(u.pos[1] - c[1]);
        assert!((0.8 * r..1.2 * r).contains(&d), "{d} m from the loiter point");
        assert!((450.0..750.0).contains(&u.pos[2]), "at {} m", u.pos[2]);
        assert_eq!(u.mode, UavMode::Auto);
    }
}

#[test]
fn routes_end_at_a_loiter_or_terminate_point_and_storks_never_terminate() {
    let (mut sim, k, _, _) = center();
    let a = off(&sim, k, 2000.0, 0.0);
    let bad = vec![wp(a, 300.0, WpKind::Loiter), wp(a, 300.0, WpKind::Nav)];
    assert!(launch(&mut sim, UavKind::Fp2, bad).is_err());
    assert!(launch(&mut sim, UavKind::Stork, vec![wp(a, 300.0, WpKind::Terminate)]).is_err());
    assert!(launch(&mut sim, UavKind::Stork, vec![]).is_err());
    assert!(sim.uavs().is_empty());
}

#[test]
fn five_drones_fly_at_once_and_the_rail_needs_its_time() {
    let (mut sim, k, _, _) = center();
    let a = off(&sim, k, 3000.0, 3000.0);
    let route = || vec![wp(a, 500.0, WpKind::Loiter)];
    launch(&mut sim, UavKind::Stork, route()).unwrap();
    let why = launch(&mut sim, UavKind::Stork, route()).unwrap_err();
    assert!(why.contains("Rail"), "{why}");
    for kind in [UavKind::Stork, UavKind::Stork, UavKind::Stork, UavKind::Fp2] {
        run(&mut sim, 16);
        launch(&mut sim, kind, route()).unwrap();
    }
    assert_eq!(sim.uavs().len(), MAX_AIRBORNE);
    assert_eq!(sim.uav_stock(k), [0, FP2_STOCK - 1]);
    run(&mut sim, 16);
    let why = launch(&mut sim, UavKind::Fp2, route()).unwrap_err();
    assert!(why.contains("in the air"), "{why}");
    let ids: Vec<u16> = sim.uavs().iter().map(|u| u.id).collect();
    assert_eq!(ids, [1, 2, 3, 4, 5]);
}

#[test]
fn an_fp2_dives_into_its_terminate_point_and_destroys_the_unit_there() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 3000.0, -2000.0);
    // An enemy Patriot launcher (no gun to shoot back) there.
    sim.aaa_units_mut().push(AaaUnit {
        kind: AaaKind::PatriotLauncher,
        x: at.0,
        y: at.1,
        ..Default::default()
    });
    let target = sim.aaa_units().len() - 1;
    let mid = off(&sim, k, 1500.0, 500.0);
    let route = vec![wp(mid, 300.0, WpKind::Nav), wp(at, 300.0, WpKind::Terminate)];
    launch(&mut sim, UavKind::Fp2, route).unwrap();
    let mut dived = false;
    let mut ev = Vec::new();
    for _ in 0..300 {
        ev.extend(run(&mut sim, 1));
        dived |= sim.uavs().first().is_some_and(|u| u.mode == UavMode::Dive);
        if sim.uavs().is_empty() {
            break;
        }
    }
    assert!(dived);
    assert!(sim.uavs().is_empty());
    let blasts: Vec<_> = ev.iter().filter(|e| matches!(e, SimEvent::Explosion { .. })).collect();
    assert!(sim.aaa_units()[target].destroyed, "{blasts:?} at {at:?}");
    assert!(ev.iter().any(|e| matches!(e, SimEvent::Explosion { kind: -6, .. })));
}

#[test]
fn a_stork_finds_a_hidden_sam_site_that_never_fires_at_it() {
    let (mut sim, k, site, radar) = center();
    let s = sim.objects()[site];
    sim.objects_mut()[site].unit_type = radar;
    // The center 20 km off the site's west: the Stork loiters 2 km short of it.
    let c = sim.aaa_units()[k];
    let d = (20_000.0 / M_PER_MAP) as i32;
    (sim.aaa_units_mut()[k].x, sim.aaa_units_mut()[k].y) = (s.x - d, s.y);
    let at = (s.x - (2000.0 / M_PER_MAP) as i32, s.y);
    launch(&mut sim, UavKind::Stork, vec![wp(at, 800.0, WpKind::Loiter)]).unwrap();
    assert!(!sim.uav_known().contains(&Seen::Object(site as u16)));
    let f = sim.uavs()[0].flight as usize;
    run(&mut sim, 1100);
    assert!(sim.uav_known().contains(&Seen::Object(site as u16)), "{:?}", sim.uav_known());
    assert_eq!(sim.uavs().len(), 1, "the SAM site shot it down");
    assert!(sim.ai_missiles().iter().all(|m| m.target != f));
    let _ = c;
}

#[test]
fn an_empty_battery_brings_the_drone_down() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 2000.0, 0.0);
    launch(&mut sim, UavKind::Stork, vec![wp(at, 200.0, WpKind::Loiter)]).unwrap();
    run(&mut sim, 60);
    let u = &sim.uavs()[0];
    assert!(u.battery < 1.0 && u.endurance() > 2.0 * 3600.0, "{} {}", u.battery, u.endurance());
    sim_set_battery(&mut sim, 0.0);
    let f = sim.uavs()[0].flight as usize;
    run(&mut sim, 400);
    assert!(sim.uavs().is_empty());
    assert_eq!(sim.flights()[f].flags & fflags::AIRBORNE, 0);
}

#[test]
fn a_drone_flown_by_hand_turns_with_the_stick() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 5000.0, 0.0);
    launch(&mut sim, UavKind::Fp2, vec![wp(at, 300.0, WpKind::Loiter)]).unwrap();
    let id = sim.uavs()[0].id;
    assert!(sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).is_err());
    run(&mut sim, 10);
    sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
    let h0 = sim.uavs()[0].heading;
    let fly = StationCmd::Fly { uav: id, roll: 1.0, pitch: 0.0, throttle: 1.0 };
    sim.station_command(ME, fly).unwrap();
    run(&mut sim, 3);
    let h1 = sim.uavs()[0].heading;
    assert!((h1 - h0).rem_euclid(std::f64::consts::TAU) > 0.2, "{h0} -> {h1}");
    // Leaving the console puts it back on its route.
    sim.station_command(ME, StationCmd::Leave).unwrap();
    assert_eq!(sim.uavs()[0].mode, UavMode::Auto);
}

#[test]
fn enemy_guns_shoot_the_drones_down() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 4000.0, 0.0);
    for (dx, kind) in [(0, AaaKind::Zsu), (8, AaaKind::Zsu), (-8, AaaKind::S60)] {
        sim.aaa_units_mut().push(AaaUnit { kind, x: at.0 + dx, y: at.1, ..Default::default() });
    }
    launch(&mut sim, UavKind::Stork, vec![wp(at, 500.0, WpKind::Loiter)]).unwrap();
    for _ in 0..900 {
        run(&mut sim, 1);
        if sim.uavs().is_empty() {
            return;
        }
    }
    panic!("never shot down");
}

fn sim_set_battery(sim: &mut Sim, b: f32) {
    sim.set_uav_battery(0, b);
}

#[test]
fn an_fp2_terminating_on_a_sam_site_destroys_it() {
    let (mut sim, k, site, radar) = center();
    let s = sim.objects()[site];
    sim.objects_mut()[site].unit_type = radar;
    let d = (6000.0 / M_PER_MAP) as i32;
    (sim.aaa_units_mut()[k].x, sim.aaa_units_mut()[k].y) = (s.x - d, s.y + d / 3);
    launch(&mut sim, UavKind::Fp2, vec![wp((s.x, s.y), 300.0, WpKind::Terminate)]).unwrap();
    let mut ev = Vec::new();
    for _ in 0..400 {
        ev.extend(run(&mut sim, 1));
        if sim.uavs().is_empty() {
            break;
        }
    }
    assert!(sim.uavs().is_empty());
    assert!(sim.objects()[site].flags & oflags::DESTROYED != 0);
    assert!(ev.iter().any(|e| matches!(e, SimEvent::GroundDestroyed { .. })));
}
