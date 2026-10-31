//! Campaign: a drone command center's drones (`uav`).

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{GroundKind, GroundUnit, M_PER_MAP};
use stealth_sim::station::{FP2_STOCK, INTERCEPTOR_STOCK, STORK_STOCK, StationCmd};
use stealth_sim::uav::{LOCK_RANGE, MAX_AIRBORNE, Seen, UavKind, UavMode, UavWp, WpKind};
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
    assert_eq!(sim.uav_stock(k), [STORK_STOCK - 1, FP2_STOCK, INTERCEPTOR_STOCK]);
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
    assert_eq!(sim.uav_stock(k), [0, FP2_STOCK - 1, INTERCEPTOR_STOCK]);
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
    let fly = StationCmd::Fly { uav: id, roll: 1.0, pitch: 0.0, yaw: 0.0, throttle: 1.0 };
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

/// An interceptor launched to loiter 2 km out at 500 m, flown by hand from 60 s on (level, at
/// the loiter's height): the sim, its center, its id.
fn interceptor() -> (Sim, usize, u16) {
    let (mut sim, k, _, _) = center();
    let at = off(&sim, k, 2000.0, 0.0);
    launch(&mut sim, UavKind::Interceptor, vec![wp(at, 500.0, WpKind::Loiter)]).unwrap();
    assert_eq!(sim.uav_stock(k), [STORK_STOCK, FP2_STOCK, INTERCEPTOR_STOCK - 1]);
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
    frames(&mut sim, 1);
    let (b1, p1) = (sim.uavs()[0].bank, sim.uavs()[0].pitch);
    assert!(b1 < b0 - 0.2 && p1 > 0.1, "bank {b0} -> {b1}, pitch {p1}");
    run(&mut sim, 3);
    let u = &sim.uavs()[0];
    assert!((u.bank - b1).abs() < 1e-6 && (u.pitch - p1).abs() < 1e-6, "{} {}", u.bank, u.pitch);
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
        rudder(&mut sim, id, yaw, t);
        run(&mut sim, 2);
        sim.uavs()[0].heading
    };
    // Full left rudder for 2 s: 0.3 rad further left than without.
    let skid = wrap(heading(-1.0) - heading(0.0));
    assert!((-0.35..-0.25).contains(&skid), "{skid}");
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
    // Level at that throttle.
    let alt = u.pos[2];
    run(&mut sim, 20);
    assert!((sim.uavs()[0].pos[2] - alt).abs() < 30.0, "{alt} -> {}", sim.uavs()[0].pos[2]);
    // Full throttle: 600 km/h, the battery five times as quick.
    fly(&mut sim, id, 0.0, 1.0);
    run(&mut sim, 15);
    let u = &sim.uavs()[0];
    assert!(u.speed * 3.6 > 560.0, "{} km/h", u.speed * 3.6);
    let full = u.endurance() / u.battery as f64;
    assert!((90.0..100.0).contains(&full), "{full} s on a full battery");
    // Under a fifth it can't hold its height, even with the stick back.
    fly(&mut sim, id, 1.0, 0.05);
    let alt = sim.uavs()[0].pos[2];
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
    let (e, n) = (u.pos[0] + u.heading.sin() * ahead, u.pos[1] + u.heading.cos() * ahead);
    let wu = 512_000.0 / 1_048_576.0;
    let f = &mut sim.flights_mut()[j];
    f.wx = (e / wu) as i32;
    f.wy = 0x10_0000 - (n / wu) as i32;
    (f.x, f.y) = (f.wx >> 5, f.wy >> 5);
    f.alt = (u.pos[2] / wu) as i32;
    f.flags |= fflags::AIRBORNE;
    j
}

#[test]
fn an_interceptor_locks_on_to_a_drone_ahead_and_destroys_it_at_full_throttle() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    let j = geran_ahead(&mut sim, LOCK_RANGE * 0.8);
    run(&mut sim, 1);
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
    assert_eq!(sim.uavs()[0].lock, Some(j as u16));
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
fn an_interceptor_ignores_a_drone_behind_it_or_too_far() {
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    geran_ahead(&mut sim, -300.0);
    run(&mut sim, 1);
    assert_eq!(sim.uavs()[0].lock, None);
    let (mut sim, _, id) = interceptor();
    fly(&mut sim, id, 0.0, 0.2);
    run(&mut sim, 3);
    geran_ahead(&mut sim, LOCK_RANGE * 2.0);
    run(&mut sim, 1);
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
