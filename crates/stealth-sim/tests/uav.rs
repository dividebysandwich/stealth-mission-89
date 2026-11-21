//! Campaign: a drone command center's drones (`uav`).

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{GroundKind, GroundUnit, M_PER_MAP};
use stealth_sim::station::{FP2_STOCK, INTERCEPTOR_STOCK, STORK_STOCK, StationCmd, X12_STOCK};
use stealth_sim::uav::{
    LOCK_RANGE, MAX_AIRBORNE, Seen, UavKind, UavMode, UavWp, WpKind, X12_LOCK_RANGE,
};
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
    sim.units_mut().clear();
    let home = (3..sim.objects().len())
        .find(|&i| {
            sim.objects()[i].flags & 0xF0F == 0x401
                && sim.place_name(i).eq_ignore_ascii_case("Neuburg")
        })
        .expect("Neuburg");
    let o = sim.objects()[home];
    sim.place_drone_center(o.x + 0x40, o.y);
    let k = sim.units().iter().position(|u| u.kind == GroundKind::DroneCenter).unwrap();
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
    let u = sim.units()[k];
    (u.x + (east / M_PER_MAP) as i32, u.y - (north / M_PER_MAP) as i32)
}

fn wp((x, y): (i32, i32), alt: f32, kind: WpKind) -> UavWp {
    UavWp { x, y, alt, kind }
}

fn launch(sim: &mut Sim, kind: UavKind, route: Vec<UavWp>) -> Result<(), String> {
    sim.station_command(ME, StationCmd::Launch { kind, route })
        .map_err(|t| t.tr_in(stealth_i18n::Language::En))
}

#[test]
fn a_stork_climbs_off_the_rail_and_circles_its_loiter_point() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 2500.0, 0.0);
    launch(&mut sim, UavKind::Stork, vec![wp(at, 600.0, WpKind::Loiter)]).unwrap();
    assert_eq!(sim.uav_stock(k), [STORK_STOCK - 1, FP2_STOCK, INTERCEPTOR_STOCK, X12_STOCK]);
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
    assert_eq!(sim.uav_stock(k), [0, FP2_STOCK - 1, INTERCEPTOR_STOCK, X12_STOCK]);
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
    sim.units_mut().push(GroundUnit {
        kind: GroundKind::PatriotLauncher,
        x: at.0,
        y: at.1,
        ..Default::default()
    });
    let target = sim.units().len() - 1;
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
    assert!(sim.units()[target].destroyed, "{blasts:?} at {at:?}");
    assert!(ev.iter().any(|e| matches!(e, SimEvent::Explosion { kind: -6, .. })));
}

#[test]
fn a_stork_finds_a_hidden_sam_site_that_never_finds_it() {
    let (mut sim, k, site, radar) = center();
    let s = sim.objects()[site];
    sim.objects_mut()[site].unit_type = radar;
    // The center 20 km off the site's west: the Stork circles 4 km short of it (never closer
    // than 3 km, where the site can't find it).
    let c = sim.units()[k];
    let d = (20_000.0 / M_PER_MAP) as i32;
    (sim.units_mut()[k].x, sim.units_mut()[k].y) = (s.x - d, s.y);
    let at = (s.x - (4000.0 / M_PER_MAP) as i32, s.y);
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
    // Rolled into a bank and let go: it turns.
    let fly = StationCmd::Fly { uav: id, roll: 1.0, pitch: 0.0, yaw: 0.0, throttle: 1.0 };
    sim.station_command(ME, fly).unwrap();
    frames(&mut sim, 8);
    stick(&mut sim, id, 0.0, 0.0, 1.0);
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
    for (dx, kind) in [(0, GroundKind::Zsu), (8, GroundKind::Zsu), (-8, GroundKind::S60)] {
        sim.units_mut().push(GroundUnit { kind, x: at.0 + dx, y: at.1, ..Default::default() });
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
    (sim.units_mut()[k].x, sim.units_mut()[k].y) = (s.x - d, s.y + d / 3);
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

#[test]
fn guns_beyond_two_km_never_find_a_drone() {
    let (mut sim, k, _, _) = center();
    let guns = off(&sim, k, 6000.0, 0.0);
    for (dx, kind) in [(0, GroundKind::Zsu), (8, GroundKind::S60), (-8, GroundKind::Manpads)] {
        sim.units_mut().push(GroundUnit { kind, x: guns.0 + dx, y: guns.1, ..Default::default() });
    }
    // Circling 3.5 km short of them (2.6 km at the closest): in the S-60's and the MANPADS'
    // reach, but they never find it.
    let at = off(&sim, k, 2500.0, 0.0);
    launch(&mut sim, UavKind::Stork, vec![wp(at, 500.0, WpKind::Loiter)]).unwrap();
    let f = sim.uavs()[0].flight as usize;
    run(&mut sim, 600);
    assert_eq!(sim.uavs().len(), 1, "shot down");
    assert!(sim.ai_missiles().iter().all(|m| m.target != f));
}

#[test]
fn a_sam_site_finds_a_drone_within_two_km() {
    let (mut sim, k, site, radar) = center();
    let s = sim.objects()[site];
    sim.objects_mut()[site].unit_type = radar;
    let d = (6000.0 / M_PER_MAP) as i32;
    (sim.units_mut()[k].x, sim.units_mut()[k].y) = (s.x - d, s.y);
    launch(&mut sim, UavKind::Stork, vec![wp((s.x, s.y), 800.0, WpKind::Loiter)]).unwrap();
    let f = sim.uavs()[0].flight as usize;
    let mut fired = false;
    for _ in 0..600 {
        run(&mut sim, 1);
        fired |= sim.ai_missiles().iter().any(|m| m.target == f);
        if fired || sim.uavs().is_empty() {
            break;
        }
    }
    assert!(fired || sim.uavs().is_empty(), "the SAM site never engaged it");
}

/// An FP-2 flying straight with its wings level (climbing towards a loiter point 20 km off at
/// 800 m), flown by hand: the sim, its id.
fn fp2_by_hand() -> (Sim, u16) {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 20_000.0, 0.0);
    launch(&mut sim, UavKind::Fp2, vec![wp(at, 800.0, WpKind::Loiter)]).unwrap();
    let id = sim.uavs()[0].id;
    run(&mut sim, 120);
    sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
    let u = &sim.uavs()[0];
    assert!(u.bank.abs() < 0.05 && u.pitch.abs() < 0.2, "bank {} pitch {}", u.bank, u.pitch);
    (sim, id)
}

#[test]
fn a_winged_drone_loops_with_the_stick_held_back() {
    let (mut sim, id) = fp2_by_hand();
    let h0 = sim.uavs()[0].heading;
    // Speed first: down 30 degrees at full throttle.
    stick(&mut sim, id, 0.0, -1.0, 1.0);
    while sim.uavs()[0].pitch > -0.5 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    run(&mut sim, 8);
    let v0 = sim.uavs()[0].speed;
    assert!(v0 > 75.0, "{v0} m/s");
    // Up through the vertical, over the top on its back, down and level again.
    stick(&mut sim, id, 0.0, 1.0, 1.0);
    let (mut steepest, mut inverted, mut slowest) = (0.0_f64, false, v0);
    let mut was_over = false;
    for _ in 0..30 * 15 {
        frames(&mut sim, 1);
        let u = &sim.uavs()[0];
        steepest = steepest.max(u.pitch);
        slowest = slowest.min(u.speed);
        inverted |= u.bank.abs() > 3.0 && u.pitch.abs() < 0.3;
        was_over |= inverted;
        if was_over && u.bank.abs() < 0.5 && u.pitch > -0.1 {
            break;
        }
    }
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    frames(&mut sim, 1);
    let u = &sim.uavs()[0];
    assert!(steepest > 1.5, "steepest {steepest}");
    assert!(inverted, "never on its back over the top");
    assert!(slowest < v0 - 20.0, "{v0} m/s, over the top {slowest} m/s");
    assert!(u.pitch.abs() < 0.3 && u.bank.abs() < 0.5, "pitch {} bank {}", u.pitch, u.bank);
    assert!(wrap(u.heading - h0).abs() < 0.3, "heading {h0} -> {}", u.heading);
}

#[test]
fn a_winged_drone_rolls_all_the_way_round() {
    let (mut sim, id) = fp2_by_hand();
    stick(&mut sim, id, 1.0, 0.0, 1.0);
    let mut inverted = false;
    // A whole turn at 1.2 rad/s.
    // (Its rate builds up after the stick as it dies away after it.)
    let turn = (std::f64::consts::TAU / 1.2 * 15.0).round() as i32;
    for _ in 0..turn {
        frames(&mut sim, 1);
        inverted |= sim.uavs()[0].bank.abs() > 3.0;
    }
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    run(&mut sim, 1);
    let u = &sim.uavs()[0];
    assert!(inverted, "never on its back");
    assert!(u.bank.abs() < 0.25, "bank {}", u.bank);
}

/// An interceptor launched to loiter 2 km out at 500 m, flown by hand from 60 s on (level, at
/// the loiter's height): the sim, its center, its id.
fn interceptor() -> (Sim, usize, u16) {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 2000.0, 0.0);
    launch(&mut sim, UavKind::Interceptor, vec![wp(at, 500.0, WpKind::Loiter)]).unwrap();
    assert_eq!(sim.uav_stock(k), [STORK_STOCK, FP2_STOCK, INTERCEPTOR_STOCK - 1, X12_STOCK]);
    let id = sim.uavs()[0].id;
    run(&mut sim, 60);
    sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
    (sim, k, id)
}

fn fly(sim: &mut Sim, id: u16, pitch: f32, throttle: f32) {
    stick(sim, id, 0.0, pitch, throttle);
}

fn stick(sim: &mut Sim, id: u16, roll: f32, pitch: f32, throttle: f32) {
    sim.station_command(ME, StationCmd::Fly { uav: id, roll, pitch, yaw: 0.0, throttle }).unwrap();
}

fn rudder(sim: &mut Sim, id: u16, yaw: f32, throttle: f32) {
    let fly = StationCmd::Fly { uav: id, roll: 0.0, pitch: 0.0, yaw, throttle };
    sim.station_command(ME, fly).unwrap();
}

/// `n` frames (15 a second).
fn frames(sim: &mut Sim, n: i32) {
    for _ in 0..n {
        sim.step(&idle(), 15);
    }
}

#[test]
fn a_drone_keeps_its_attitude_when_the_stick_is_let_go() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 5000.0, 0.0);
    launch(&mut sim, UavKind::Fp2, vec![wp(at, 300.0, WpKind::Loiter)]).unwrap();
    let id = sim.uavs()[0].id;
    run(&mut sim, 10);
    sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
    stick(&mut sim, id, 0.0, 0.0, 0.5);
    frames(&mut sim, 2);
    let b0 = sim.uavs()[0].bank;
    stick(&mut sim, id, -1.0, 1.0, 0.5);
    frames(&mut sim, 6);
    stick(&mut sim, id, 0.0, 0.0, 0.5);
    // Its rates die away after the stick (its inertia) ...
    frames(&mut sim, 8);
    let (b1, p1) = (sim.uavs()[0].bank, sim.uavs()[0].pitch);
    assert!(b1 < b0 - 0.2 && p1 > 0.1, "bank {b0} -> {b1}, pitch {p1}");
    // ... and it holds its attitude (its wing slowly rolling it out of the slip).
    run(&mut sim, 1);
    let u = &sim.uavs()[0];
    assert!((u.bank - b1).abs() < 0.1 && (u.pitch - p1).abs() < 0.1, "{} {}", u.bank, u.pitch);
}

#[test]
fn a_winged_drone_s_roll_carries_on_a_little_after_the_stick() {
    let (mut sim, id) = fp2_by_hand();
    let t = sim.uavs()[0].ctrl[2];
    stick(&mut sim, id, 1.0, 0.0, t);
    // Its rate takes its time to build up: under its full 1.2 rad/s at first.
    frames(&mut sim, 3);
    let early = sim.uavs()[0].bank;
    assert!((0.05..0.2).contains(&early), "bank {early} after 0.2 s");
    frames(&mut sim, 7);
    let b0 = sim.uavs()[0].bank;
    stick(&mut sim, id, 0.0, 0.0, t);
    frames(&mut sim, 15);
    let b1 = sim.uavs()[0].bank;
    assert!(b1 - b0 > 0.15, "bank {b0} -> {b1} after the stick was let go");
    frames(&mut sim, 8);
    assert!((sim.uavs()[0].bank - b1).abs() < 0.05, "still rolling");
}

#[test]
fn a_winged_drone_trades_speed_for_height() {
    let (mut sim, id) = fp2_by_hand();
    let t = sim.uavs()[0].ctrl[2];
    let (v0, alt0) = (sim.uavs()[0].speed, sim.uavs()[0].pos[2]);
    // Nose 30 degrees up at the same throttle: it climbs and slows.
    stick(&mut sim, id, 0.0, 1.0, t);
    while sim.uavs()[0].pitch < 0.5 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, t);
    run(&mut sim, 5);
    let u = &sim.uavs()[0];
    assert!(
        u.pos[2] > alt0 + 50.0 && u.speed < v0 - 10.0,
        "{alt0} m {v0} m/s -> {} m {} m/s",
        u.pos[2],
        u.speed
    );
    // Nose 30 degrees down, the throttle closed: it gains it back.
    stick(&mut sim, id, 0.0, -1.0, 0.0);
    while sim.uavs()[0].pitch > -0.5 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, 0.0);
    run(&mut sim, 6);
    assert!(sim.uavs()[0].speed > v0, "{v0} -> {} m/s", sim.uavs()[0].speed);
}

#[test]
fn a_winged_drone_stalls_slow_and_drops_its_nose_till_it_flies_again() {
    let (mut sim, id) = fp2_by_hand();
    // The throttle closed, the nose held 20 degrees up: it slows till the wing stalls.
    stick(&mut sim, id, 0.0, 1.0, 0.0);
    while sim.uavs()[0].pitch < 0.35 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, 0.0);
    let mut stalled = false;
    for _ in 0..15 * 15 {
        frames(&mut sim, 1);
        stalled |= sim.uavs()[0].stalled();
        if stalled {
            break;
        }
    }
    let u = &sim.uavs()[0];
    assert!(stalled && u.speed < 40.0, "never stalled ({} m/s)", u.speed);
    let v0 = u.speed;
    // Its nose drops by itself; held nose up it mushes on at the stall's edge ...
    let p0 = u.pitch;
    run(&mut sim, 3);
    let u = &sim.uavs()[0];
    assert!(u.pitch < p0 - 0.1, "pitch {p0} -> {}", u.pitch);
    assert!(u.aoa() > 0.15, "aoa {}", u.aoa());
    // ... pushed level, it picks up speed and flies again.
    stick(&mut sim, id, 0.0, -1.0, 0.0);
    while sim.uavs()[0].pitch > -0.1 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, 0.0);
    run(&mut sim, 3);
    let u = &sim.uavs()[0];
    assert!(!u.stalled() && u.speed > v0 + 5.0, "aoa {} {v0} -> {} m/s", u.aoa(), u.speed);
}

/// The angle (rad) between two directions.
fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
        * (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    (d / l).clamp(-1.0, 1.0).acos()
}

fn nose(u: &stealth_sim::uav::Uav) -> [f64; 3] {
    [u.heading.sin() * u.pitch.cos(), u.heading.cos() * u.pitch.cos(), u.pitch.sin()]
}

#[test]
fn an_interceptor_turns_its_nose_at_once_and_its_course_after_it() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 1.0);
    run(&mut sim, 10);
    let h0 = sim.uavs()[0].heading;
    // Rolled on its side and pulled: the nose comes round a right angle in under a second.
    stick(&mut sim, id, 1.0, 0.0, 1.0);
    frames(&mut sim, 6);
    stick(&mut sim, id, 0.0, 1.0, 1.0);
    frames(&mut sim, 8);
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    frames(&mut sim, 1);
    let u = sim.uavs()[0].clone();
    let turned = (u.heading - h0).rem_euclid(std::f64::consts::TAU);
    assert!((1.0..3.0).contains(&turned), "nose turned {turned} rad");
    // Its course lags behind...
    assert!(angle(nose(&u), u.vel) > 0.5, "course {} rad off the nose", angle(nose(&u), u.vel));
    // ... and catches up, the attitude held.
    run(&mut sim, 3);
    let u = &sim.uavs()[0];
    assert!(angle(nose(u), u.vel) < 0.15, "course {} rad off the nose", angle(nose(u), u.vel));
    assert!(u.bank.abs() > 1.0, "bank {}", u.bank);
}

#[test]
fn an_interceptor_yaws_as_fast_as_it_pitches() {
    let (mut sim, _, id) = interceptor();
    run(&mut sim, 10);
    let u0 = sim.uavs()[0].clone();
    // Half a second of full rudder: the nose 1.5 rad round, as far as the stick pitches it.
    rudder(&mut sim, id, 1.0, 1.0);
    frames(&mut sim, 7);
    rudder(&mut sim, id, 0.0, 1.0);
    frames(&mut sim, 1);
    let u = sim.uavs()[0].clone();
    let turned = angle(nose(&u0), nose(&u));
    assert!((1.3..1.7).contains(&turned), "nose turned {turned} rad");
    assert!(wrap(u.heading - u0.heading) > 0.0, "turned left");
    // Let go, it holds its attitude.
    run(&mut sim, 2);
    assert!(angle(nose(&u), nose(&sim.uavs()[0])) < 0.05);
}

#[test]
fn a_winged_drone_s_rudder_skids_its_nose_round() {
    let heading = |yaw: f32| {
        let (mut sim, k, _, _) = center();
        let at = off(&sim, k, 5000.0, 0.0);
        launch(&mut sim, UavKind::Fp2, vec![wp(at, 300.0, WpKind::Loiter)]).unwrap();
        let id = sim.uavs()[0].id;
        run(&mut sim, 10);
        sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
        let t = sim.uavs()[0].ctrl[2];
        let b0 = sim.uavs()[0].bank;
        rudder(&mut sim, id, yaw, t);
        run(&mut sim, 2);
        let u = &sim.uavs()[0];
        assert!(yaw == 0.0 || (u.bank - b0) * yaw as f64 > 0.1, "bank {b0} -> {}", u.bank);
        u.heading
    };
    // Full left rudder for 2 s: it skids, its wing rolls it left and it turns left.
    let skid = wrap(heading(-1.0) - heading(0.0));
    assert!(skid < -0.12, "{skid}");
}

/// A heading difference in -pi..pi.
fn wrap(a: f64) -> f64 {
    (a + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

#[test]
fn an_interceptor_flies_eight_minutes_at_a_fifth_of_its_throttle() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 10);
    let u = &sim.uavs()[0];
    let full = u.endurance() / u.battery as f64;
    assert!((470.0..490.0).contains(&full), "{full} s on a full battery");
    // Its wings level (it was circling) ...
    while sim.uavs()[0].bank.abs() > 0.03 {
        let roll = -sim.uavs()[0].bank.signum() as f32 * 0.1;
        stick(&mut sim, id, roll, 0.0, 0.2);
        frames(&mut sim, 1);
    }
    // ... level at that throttle, its nose a little up for its wings.
    let up = |sim: &mut Sim, pitch: f64, throttle: f32| {
        let sign = (pitch - sim.uavs()[0].pitch).signum() as f32;
        fly(sim, id, sign * 0.2, throttle);
        while (pitch - sim.uavs()[0].pitch) * sign as f64 > 0.0 {
            frames(sim, 1);
        }
        fly(sim, id, 0.0, throttle);
    };
    up(&mut sim, 0.06, 0.2);
    run(&mut sim, 3);
    let alt = sim.uavs()[0].pos[2];
    run(&mut sim, 20);
    assert!((sim.uavs()[0].pos[2] - alt).abs() < 30.0, "{alt} -> {}", sim.uavs()[0].pos[2]);
    // Full throttle: 600 km/h, the battery five times as quick.
    fly(&mut sim, id, 0.0, 1.0);
    run(&mut sim, 15);
    let u = &sim.uavs()[0];
    assert!(u.speed * 3.6 > 560.0, "{} km/h", u.speed * 3.6);
    let full = u.endurance() / u.battery as f64;
    assert!((90.0..100.0).contains(&full), "{full} s on a full battery");
    // Pointed straight up at a fifth it holds itself up; under it, it can't.
    fly(&mut sim, id, 0.0, 0.2);
    up(&mut sim, 1.44, 0.0);
    // (Its motors stopped till it has coasted up to a stop.)
    while sim.uavs()[0].vel[2] > 1.0 {
        frames(&mut sim, 1);
    }
    fly(&mut sim, id, 0.0, 0.2);
    let alt = sim.uavs()[0].pos[2];
    run(&mut sim, 5);
    let u = &sim.uavs()[0];
    assert!(
        (u.pos[2] - alt).abs() < 10.0 && u.speed < 10.0,
        "{alt} -> {} m, {} m/s",
        u.pos[2],
        u.speed
    );
    fly(&mut sim, id, 0.0, 0.05);
    run(&mut sim, 5);
    assert!(sim.uavs()[0].pos[2] < alt - 20.0, "{alt} -> {}", sim.uavs()[0].pos[2]);
}

#[test]
fn an_interceptor_with_a_flat_battery_drops() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.5);
    run(&mut sim, 5);
    sim_set_battery(&mut sim, 0.0);
    let f = sim.uavs()[0].flight as usize;
    run(&mut sim, 3);
    assert!(sim.uavs()[0].pitch < -0.5, "pitch {}", sim.uavs()[0].pitch);
    run(&mut sim, 40);
    assert!(sim.uavs().is_empty());
    assert_eq!(sim.flights()[f].flags & fflags::AIRBORNE, 0);
}

/// An enemy Geran-2 put `ahead` m in front of the interceptor at its height (flying on to
/// a place 50 km off): its flight unit.
fn geran_ahead(sim: &mut Sim, ahead: f64) -> usize {
    drone_ahead(sim, ahead, false)
}

/// A Geran-2 of a side (`blue`: the friendly one's) put `ahead` m in front of the interceptor.
fn drone_ahead(sim: &mut Sim, ahead: f64, blue: bool) -> usize {
    drone_at(sim, ahead, 0.0, blue)
}

/// A Geran-2 put `ahead` m in front of the interceptor and `right` m to its right.
fn drone_at(sim: &mut Sim, ahead: f64, right: f64, blue: bool) -> usize {
    let home = (3..sim.objects().len())
        .find(|&i| sim.place_name(i).eq_ignore_ascii_case("Neuburg"))
        .expect("Neuburg");
    let h = sim.objects()[home];
    let far = (3..sim.objects().len())
        .find(|&i| {
            let o = sim.objects()[i];
            ((o.x - h.x) as f64).hypot((o.y - h.y) as f64) * M_PER_MAP > 50_000.0
        })
        .expect("a place 50 km off");
    let j = sim.spawn_drone(home, far, 77, blue);
    run(sim, 2);
    let u = sim.uavs()[0].clone();
    let (s, c) = u.heading.sin_cos();
    let (e, n) = (u.pos[0] + s * ahead + c * right, u.pos[1] + c * ahead - s * right);
    let wu = 512_000.0 / 1_048_576.0;
    let f = &mut sim.flights_mut()[j];
    f.wx = (e / wu) as i32;
    f.wy = 0x10_0000 - (n / wu) as i32;
    (f.x, f.y) = (f.wx >> 5, f.wy >> 5);
    f.alt = (u.pos[2] / wu) as i32;
    f.flags |= fflags::AIRBORNE;
    j
}

/// The player fires: the drone locks on (or lets go).
fn lock(sim: &mut Sim, id: u16, on: bool) -> Result<(), stealth_i18n::Text> {
    sim.station_command(ME, StationCmd::UavLock { uav: id, on })
}

#[test]
fn an_interceptor_locks_on_to_a_drone_ahead_and_destroys_it_at_full_throttle() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    let j = geran_ahead(&mut sim, LOCK_RANGE * 0.8);
    run(&mut sim, 1);
    // Only when the player fires.
    assert_eq!(sim.uavs()[0].lock, None);
    lock(&mut sim, id, true).unwrap();
    assert_eq!(sim.uavs()[0].lock, Some(j as u16));
    fly(&mut sim, id, 0.0, 1.0);
    let mut ev = Vec::new();
    for _ in 0..30 {
        ev.extend(run(&mut sim, 1));
        if sim.uavs().is_empty() {
            break;
        }
    }
    assert!(sim.uavs().is_empty(), "never hit it");
    assert!(sim.flights()[j].flags & fflags::DESTROYED != 0);
    assert!(ev.iter().any(|e| matches!(e, SimEvent::AircraftDestroyed { unit, .. } if *unit == j)));
}

#[test]
fn an_interceptor_cannot_tell_a_friendly_drone_from_an_enemy() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    let j = drone_ahead(&mut sim, LOCK_RANGE * 0.8, true);
    run(&mut sim, 1);
    lock(&mut sim, id, true).unwrap();
    assert_eq!(sim.uavs()[0].lock, Some(j as u16));
}

#[test]
fn an_interceptor_locks_on_to_the_drone_nearest_its_flight_path_and_lets_go_again() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    // The nearer one 10 degrees off the way it flies, the further one on it.
    let near = drone_at(&mut sim, 200.0, 200.0 * 0.18, false);
    let on = drone_at(&mut sim, LOCK_RANGE * 0.8, 0.0, false);
    frames(&mut sim, 1);
    lock(&mut sim, id, true).unwrap();
    assert_eq!(sim.uavs()[0].lock, Some(on as u16), "not the nearer one ({near})");
    run(&mut sim, 1);
    assert_eq!(sim.uavs()[0].lock, Some(on as u16));
    // Fired again: it lets go and stays let go.
    lock(&mut sim, id, false).unwrap();
    assert_eq!(sim.uavs()[0].lock, None);
    run(&mut sim, 1);
    assert_eq!(sim.uavs()[0].lock, None);
}

#[test]
fn an_interceptor_into_the_ground_blows_up_what_stands_there_friendly_or_not() {
    let (mut sim, k, id) = interceptor();
    // Nose down at full throttle, the command center itself kept under it.
    stick(&mut sim, id, 0.0, -1.0, 1.0);
    frames(&mut sim, 5);
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    let mut ev = Vec::new();
    for _ in 0..60 * 15 {
        let Some(u) = sim.uavs().first() else { break };
        let (x, y) = u.map();
        (sim.units_mut()[k].x, sim.units_mut()[k].y) = (x, y);
        ev.extend(sim.step(&idle(), 15));
    }
    assert!(sim.uavs().is_empty(), "never came down");
    assert!(sim.units()[k].destroyed, "the command center stands");
    assert!(ev.iter().any(|e| matches!(e, SimEvent::Explosion { kind: -6, .. })));
}

#[test]
fn an_x12_flown_into_a_sam_site_destroys_it_as_an_fp2_would() {
    let (mut sim, k, site, radar) = center();
    let s = sim.objects()[site];
    sim.objects_mut()[site].unit_type = radar;
    let at = off(&sim, k, 6000.0, 0.0);
    launch(&mut sim, UavKind::X12, vec![wp(at, 1000.0, WpKind::Loiter)]).unwrap();
    let id = sim.uavs()[0].id;
    run(&mut sim, 60);
    sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
    // Nose straight down (the stick let go then: it holds that attitude), then put right over
    // the site, a gun 45 m off it (beyond an interceptor's blast).
    let p = sim.uavs()[0].pos;
    sim.set_uav_pos(0, [p[0], p[1], 4000.0]);
    // The loiter left it banked: wings level first.
    while sim.uavs()[0].bank.abs() > 0.05 {
        let roll = -sim.uavs()[0].bank.signum() as f32 * 0.2;
        stick(&mut sim, id, roll, 0.0, 0.0);
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, -1.0, 0.0);
    while sim.uavs()[0].pitch > -1.4 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    let (e, n) = (s.x as f64 * M_PER_MAP, (0x8000 - s.y) as f64 * M_PER_MAP);
    sim.set_uav_pos(0, [e + 5.0, n + 5.0, 400.0]);
    let gun = sim.units().len();
    let unit = GroundUnit { kind: GroundKind::Zsu, x: s.x + 3, y: s.y, ..Default::default() };
    sim.units_mut().push(unit);
    let mut ev = Vec::new();
    for _ in 0..60 * 15 {
        if sim.uavs().is_empty() {
            break;
        }
        ev.extend(sim.step(&idle(), 15));
    }
    assert!(sim.uavs().is_empty(), "never came down");
    assert!(sim.objects()[site].flags & oflags::DESTROYED != 0, "the site stands");
    assert!(sim.units()[gun].destroyed, "the gun stands");
    assert!(ev.iter().any(|e| matches!(e, SimEvent::GroundDestroyed { .. })));
}

#[test]
fn an_interceptor_ignores_a_drone_behind_it_or_too_far() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    geran_ahead(&mut sim, -300.0);
    run(&mut sim, 1);
    assert!(lock(&mut sim, id, true).is_err());
    assert_eq!(sim.uavs()[0].lock, None);
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    geran_ahead(&mut sim, LOCK_RANGE * 2.0);
    run(&mut sim, 1);
    assert!(lock(&mut sim, id, true).is_err());
    assert_eq!(sim.uavs()[0].lock, None);
}

/// An X-12 launched to loiter 6 km out at 1000 m, 60 s on: the sim, its center, its id.
fn x12() -> (Sim, usize, u16) {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 6000.0, 0.0);
    launch(&mut sim, UavKind::X12, vec![wp(at, 1000.0, WpKind::Loiter)]).unwrap();
    assert_eq!(sim.uav_stock(k), [STORK_STOCK, FP2_STOCK, INTERCEPTOR_STOCK, X12_STOCK - 1]);
    let id = sim.uavs()[0].id;
    run(&mut sim, 60);
    (sim, k, id)
}

#[test]
fn an_x12_flies_126_km_in_a_quarter_hour_and_700_kmh_flat_out() {
    let (mut sim, _, id) = x12();
    let u = &sim.uavs()[0];
    assert_eq!(u.mode, UavMode::Auto);
    assert!(u.pos[2] > 700.0, "climbed to {} m", u.pos[2]);
    // Level at cruise: a quarter of an hour on a full tank, 126 km.
    sim.station_command(ME, StationCmd::Manual { uav: id, on: true }).unwrap();
    let perf = UavKind::X12.perf();
    // (Still banked as it was: the drain goes with the speed alone.)
    stick(&mut sim, id, 0.0, 0.0, perf.level_throttle(perf.cruise) as f32);
    run(&mut sim, 20);
    let u = &sim.uavs()[0];
    let full = u.endurance() / u.battery as f64;
    let km = full * u.speed / 1000.0;
    assert!((870.0..950.0).contains(&full), "{full} s on a full tank");
    assert!((120.0..132.0).contains(&km), "{km} km at {} m/s", u.speed);
    // Flat out it takes its time getting there (its thrust against its drag).
    fly(&mut sim, id, 0.0, 1.0);
    run(&mut sim, 15);
    assert!(sim.uavs()[0].speed * 3.6 < 650.0, "{} km/h", sim.uavs()[0].speed * 3.6);
    run(&mut sim, 35);
    let u = &sim.uavs()[0];
    assert!((680.0..=701.0).contains(&(u.speed * 3.6)), "{} km/h", u.speed * 3.6);
}

#[test]
fn an_x12_locks_on_from_further_and_hunts_a_drone_down_by_itself() {
    let (mut sim, _, id) = x12();
    // Out of its circle, level: a Geran 2.4 km ahead (beyond an interceptor's reach).
    let ahead = X12_LOCK_RANGE * 0.8;
    assert!(ahead > LOCK_RANGE * 2.0);
    let j = geran_ahead(&mut sim, ahead);
    frames(&mut sim, 2);
    lock(&mut sim, id, true).unwrap();
    assert_eq!(sim.uavs()[0].lock, Some(j as u16));
    let mut ev = Vec::new();
    for _ in 0..40 {
        ev.extend(run(&mut sim, 1));
        if sim.uavs().is_empty() {
            break;
        }
    }
    assert!(sim.uavs().is_empty(), "never hit it");
    assert!(sim.flights()[j].flags & fflags::DESTROYED != 0);
    assert!(ev.iter().any(|e| matches!(e, SimEvent::AircraftDestroyed { unit, .. } if *unit == j)));
}

#[test]
fn an_x12_ignores_a_drone_beyond_its_reach() {
    let (mut sim, _, id) = x12();
    geran_ahead(&mut sim, X12_LOCK_RANGE * 1.3);
    frames(&mut sim, 2);
    assert!(lock(&mut sim, id, true).is_err());
    assert_eq!(sim.uavs()[0].lock, None);
}

/// Balance check (`cargo test -p stealth-sim --test uav strike_odds -- --ignored --nocapture`):
/// how often an FP-2 gets through to a ZSU-23-4 with a second one and a MANPADS team beside it,
/// from 20 directions.
#[test]
#[ignore]
fn strike_odds() {
    let mut hits = 0;
    let n = 20;
    for t in 0..n {
        let (mut sim, k, _, _) = center();
        let a = t as f64 * 0.31;
        let at = off(&sim, k, 6000.0 * a.cos(), 6000.0 * a.sin());
        for (dx, kind) in [(0, GroundKind::Zsu), (10, GroundKind::Zsu), (-10, GroundKind::Manpads)]
        {
            sim.units_mut().push(GroundUnit { kind, x: at.0 + dx, y: at.1, ..Default::default() });
        }
        let target = sim.units().len() - 3;
        launch(&mut sim, UavKind::Fp2, vec![wp(at, 300.0, WpKind::Terminate)]).unwrap();
        for _ in 0..400 {
            run(&mut sim, 1);
            if sim.uavs().is_empty() {
                break;
            }
        }
        hits += sim.units()[target].destroyed as i32;
    }
    eprintln!("{hits}/{n} got through");
}

/// A center on rolling hills launches its drones off the rail on the hill (not inside it), and
/// with high ground ahead of the rail lifts them clear of it: they climb out, never through the
/// ground.
#[test]
fn a_center_in_the_hills_launches_its_drones_clear_of_the_ground() {
    let (mut sim, k, _, _) = center();
    let hills = hills::Hills::from_bytes(vec![255; hills::DIM * hills::DIM]).unwrap();
    sim.set_hills(hills.clone());
    let at = off(&sim, k, 2500.0, 0.0);
    launch(&mut sim, UavKind::Stork, vec![wp(at, 600.0, WpKind::Loiter)]).unwrap();
    let ground = |u: &stealth_sim::uav::Uav| hills.height_m(u.pos[0], u.pos[1]) as f64;
    let u = &sim.uavs()[0];
    assert!(u.pos[2] > ground(u), "launched at {} m, ground {} m", u.pos[2], ground(u));
    for _ in 0..30 * 15 {
        sim.step(&idle(), 15);
        let Some(u) = sim.uavs().first() else { panic!("the drone crashed") };
        assert!(u.pos[2] > ground(u), "at {} m over ground at {} m", u.pos[2], ground(u));
    }
}

#[test]
fn propeller_drones_climb_better_by_hand_with_the_same_top_speed() {
    for kind in [UavKind::Stork, UavKind::Fp2] {
        let p = kind.perf();
        assert!(p.hand_climb > p.climb * 1.15, "{kind:?}");
        // Level flat out at `max` still, `hand_climb` to spare at cruise.
        assert!((p.thrust(p.max) - p.drag() * p.max * p.max).abs() < 1e-9, "{kind:?}");
        let spare = p.thrust(p.cruise) - p.drag() * p.cruise * p.cruise;
        assert!((spare / 9.81 * p.cruise - p.hand_climb).abs() < 1e-6, "{kind:?}");
        assert!((p.level_throttle(p.max) - 1.0).abs() < 1e-9, "{kind:?}");
    }
    // The X-12's jet is as it was.
    let p = UavKind::X12.perf();
    assert_eq!(p.thrust(0.0), p.thrust(p.max));
    // Flown: an FP-2 at full throttle, its nose held up, climbs about 9 m/s (7 before).
    let (mut sim, id) = fp2_by_hand();
    stick(&mut sim, id, 0.0, 1.0, 1.0);
    while sim.uavs()[0].pitch < 0.25 {
        frames(&mut sim, 1);
    }
    stick(&mut sim, id, 0.0, 0.0, 1.0);
    run(&mut sim, 15);
    let alt = sim.uavs()[0].pos[2];
    run(&mut sim, 5);
    let u = &sim.uavs()[0];
    let rate = (u.pos[2] - alt) / 5.0;
    assert!((8.0..10.5).contains(&rate) && u.speed > 30.0, "{rate} m/s at {} m/s", u.speed);
}

/// A supply convoy's technical at map point `at`, its two trucks 100 m apart behind it.
fn technical_at(sim: &mut Sim, at: (i32, i32)) -> usize {
    for (dx, kind) in [(0, GroundKind::Technical), (6, GroundKind::CivTruck), (12, GroundKind::ArmyTruck)] {
        sim.units_mut().push(GroundUnit { kind, x: at.0 + dx, y: at.1, ..Default::default() });
    }
    sim.units().len() - 3
}

/// The campaign's gun shells in the air.
fn shells(sim: &Sim) -> usize {
    sim.ai_shells().count()
}

#[test]
fn a_technical_fires_only_at_a_drone_within_600_m() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 4000.0, 0.0);
    technical_at(&mut sim, at);
    // A Stork circling 1.5 km off at 500 m: out of the machine gun's reach.
    let far = off(&sim, k, 2500.0, 0.0);
    launch(&mut sim, UavKind::Stork, vec![wp(far, 500.0, WpKind::Loiter)]).unwrap();
    let mut fired = 0;
    for _ in 0..120 * 15 {
        sim.step(&idle(), 15);
        fired += shells(&sim);
    }
    assert_eq!(fired, 0, "fired at a drone out of reach");
    assert_eq!(sim.uavs().len(), 1);
    // An FP-2 diving at the trucks comes into reach: the technical opens up.
    launch(&mut sim, UavKind::Fp2, vec![wp(at, 300.0, WpKind::Terminate)]).unwrap();
    let mut fired = 0;
    for _ in 0..400 * 15 {
        sim.step(&idle(), 15);
        fired += shells(&sim);
        if sim.uavs().len() < 2 {
            break;
        }
    }
    assert!(fired > 0, "never fired at the diving FP-2");
}

/// Balance check (`cargo test -p stealth-sim --test uav truck_strike_odds -- --ignored
/// --nocapture`): how often an FP-2 gets through to a supply truck with a technical beside it,
/// from 20 directions.
#[test]
#[ignore]
fn truck_strike_odds() {
    let (mut hits, mut others) = (0, 0);
    let n = 20;
    for t in 0..n {
        let (mut sim, k, _, _) = center();
        let a = t as f64 * 0.31;
        let at = off(&sim, k, 6000.0 * a.cos(), 6000.0 * a.sin());
        let truck = technical_at(&mut sim, at) + 1;
        launch(&mut sim, UavKind::Fp2, vec![wp((at.0 + 6, at.1), 300.0, WpKind::Terminate)]).unwrap();
        for _ in 0..400 {
            run(&mut sim, 1);
            if sim.uavs().is_empty() {
                break;
            }
        }
        hits += sim.units()[truck].destroyed as i32;
        others += sim.units()[truck - 1].destroyed as i32 + sim.units()[truck + 1].destroyed as i32;
    }
    eprintln!("{hits}/{n} got through, {others} vehicles beside the truck destroyed too");
}

/// An FP-2 sent at a supply convoy's truck follows it down the road and gets it.
#[test]
fn an_fp2_sent_at_a_moving_truck_follows_it() {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 3000.0, 0.0);
    let a = stealth_sim::ground::metres_of(at.0, at.1);
    let kinds = [GroundKind::CivTruck, GroundKind::ArmyTruck, GroundKind::CivTruck];
    sim.spawn_supply_convoy(5, false, &kinds, &[a, [a[0], a[1] + 30_000.0]], 0.0);
    let head = sim.units().len() - 3;
    let u = sim.units()[head];
    launch(&mut sim, UavKind::Fp2, vec![wp((u.x, u.y), 300.0, WpKind::Terminate)]).unwrap();
    for _ in 0..400 {
        run(&mut sim, 1);
        if sim.uavs().is_empty() {
            break;
        }
    }
    let n = sim.units().len();
    let hit = (n - 3..n).filter(|&t| sim.units()[t].destroyed).count();
    assert_eq!(hit, 1, "one truck of the column");
    // It drove a long way meanwhile: a fixed point would have missed.
    assert!(sim.convoys().iter().any(|c| c.tag == 5));
}
