//! The AC-130's side guns on the shipped game data: what it takes to destroy a tank or an
//! infantry fighting vehicle, how many rounds it takes to hit from an orbit, and the gimbal.

use std::sync::OnceLock;

use stealth_mission::{GameData, Mission, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{Fine, GroundKind, GroundUnit, M_PER_MAP};
use stealth_sim::gunship::{self, GUNS};
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

/// A gunship mission over flat ground with nothing about but the units the test places; a map
/// point in open country.
fn gunship_sim() -> (Mission, Sim, (i32, i32)) {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 2,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    let m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(0x4321)).unwrap();
    let mut sim = Sim::with_options(&d.sim, &m, SimOptions { spectre: true, ..Default::default() });
    sim.flatten_hills();
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
        o.flags &= !0x0501;
        o.alert = 0;
    }
    for u in sim.flights_mut().iter_mut() {
        u.flags = 0;
    }
    let g = &m.world.grid;
    let c = (0..256)
        .find(|&i| g[i] & 0x0C >= 8 && (3..13).contains(&(i % 16)) && (3..13).contains(&(i / 16)))
        .expect("flat cell");
    let at = (((c % 16) as i32) * 2048 + 1024, ((c / 16) as i32) * 2048 + 1024);
    (m, sim, at)
}

/// A vehicle standing at map point `at`.
fn vehicle(kind: GroundKind, at: (i32, i32)) -> GroundUnit {
    let e = at.0 as f64 * M_PER_MAP;
    let n = (0x8000 - at.1) as f64 * M_PER_MAP;
    GroundUnit {
        kind,
        x: at.0,
        y: at.1,
        fine: Some(Fine { e: e as f32, n: n as f32, heading: 0.0 }),
        ..Default::default()
    }
}

/// Hits on a unit of `kind` with gun `gun` until it is destroyed.
fn hits_to_destroy(kind: GroundKind, gun: usize, seed: usize) -> usize {
    let (_, mut sim, at) = gunship_sim();
    *sim.units_mut() = vec![vehicle(kind, at)];
    // Vary the damage rolls.
    for k in 0..seed {
        sim.drop_side_shell(0, gun, [0.0, 0.0, -1000.0 - k as f64]);
    }
    let m = sim.units()[0].metres();
    for n in 1..=40 {
        sim.drop_side_shell(0, gun, [m[0] + 0.5, m[1], 0.0]);
        if sim.units()[0].destroyed {
            return n;
        }
    }
    panic!("{kind:?} survived 40 hits of gun {gun}");
}

#[test]
fn side_gun_damage() {
    for seed in 0..12 {
        let t40 = hits_to_destroy(GroundKind::T90, 0, seed);
        assert!((5..=10).contains(&t40), "40 mm on a T-90: {t40} hits");
        let b40 = hits_to_destroy(GroundKind::Bmp2, 0, seed);
        assert!((2..=3).contains(&b40), "40 mm on a BMP-2: {b40} hits");
        let z40 = hits_to_destroy(GroundKind::Zsu, 0, seed);
        assert!((2..=3).contains(&z40), "40 mm on a ZSU-23-4: {z40} hits");
        let t105 = hits_to_destroy(GroundKind::Leopard2, 1, seed);
        assert!((2..=3).contains(&t105), "105 mm on a Leopard 2: {t105} hits");
        assert_eq!(hits_to_destroy(GroundKind::Bradley, 1, seed), 1);
        assert_eq!(hits_to_destroy(GroundKind::S60, 1, seed), 1);
    }
}

#[test]
fn the_105_blast_takes_soft_targets_nearby_but_not_tanks() {
    let (_, mut sim, at) = gunship_sim();
    *sim.units_mut() = vec![vehicle(GroundKind::Bmp2, at), vehicle(GroundKind::T90, at)];
    let m = sim.units()[0].metres();
    sim.drop_side_shell(0, 1, [m[0] + 8.0, m[1], 0.0]);
    assert!(sim.units()[0].destroyed, "the BMP-2 8 m away survived");
    assert!(!sim.units()[1].destroyed);
}

/// The player at map point `at` + (`dx`, 0), `alt_m` up, heading `heading`, banked `roll`
/// degrees (left negative), at `knots`, the pod on the map point `at`, the trigger held.
fn orbit_state(
    at: (i32, i32),
    dx: i32,
    alt_m: f64,
    heading: u16,
    roll: f64,
    knots: u16,
) -> PlayerState {
    let me = (at.0 + dx, at.1);
    let (x, y) = Sim::map_to_world(me.0, me.1);
    let d = -dx as f64 * M_PER_MAP;
    let to_bam = |rad: f64| (rad / std::f64::consts::TAU * 65536.0).round() as i32;
    let pod_heading = to_bam(d.atan2(0.0)).rem_euclid(65536) as u16;
    let pod_pitch = to_bam(-(alt_m.atan2(d.abs()))) as i16;
    let feet = (alt_m / (M_PER_MAP / 32.0)) as u16;
    PlayerState {
        x,
        y,
        altitude: feet,
        height: feet,
        heading,
        roll: (to_bam(roll.to_radians())) as i16,
        velocity: knots * 27,
        gear_up: true,
        fuel: 10000,
        hud_visible: true,
        pod: Some(PodAim { heading: pod_heading, pitch: pod_pitch, ground: Some(at) }),
        cmd: Commands { trigger: true, ..Default::default() },
        ..Default::default()
    }
}

/// Fires gun `gun` for `secs` from 2 km east, 3 km up, flying north banked 30 degrees left (the
/// target off the left wing): (rounds fired, rounds that hit the vehicle, destroyed).
fn engage(gun: usize, kind: GroundKind, secs: usize) -> (usize, usize, bool) {
    let (_, mut sim, at) = gunship_sim();
    *sim.units_mut() = vec![vehicle(kind, at)];
    if gun == 1 {
        sim.step(&orbit_state(at, 128, 3000.0, 0, -30.0, 0), 15);
        let mut st = orbit_state(at, 128, 3000.0, 0, -30.0, 0);
        st.cmd = Commands { next_bay: true, ..Default::default() };
        sim.step(&st, 15);
    }
    assert_eq!(sim.side_guns().unwrap().gun as usize, gun);
    let (mut fired, mut hits) = (0, 0);
    let mut impacts = Vec::new();
    for _ in 0..15 * secs {
        for e in sim.step(&orbit_state(at, 128, 3000.0, 0, -30.0, 0), 15) {
            match e {
                SimEvent::SideGunFired { .. } => fired += 1,
                SimEvent::SideShellImpact { at: [e, n, _], armour, .. } => {
                    hits += armour as usize;
                    let m = sim.units()[0].metres();
                    impacts.push((e as f64 / 10.0 - m[0], n as f64 / 10.0 - m[1]));
                }
                _ => {}
            }
        }
    }
    let n = impacts.len().max(1) as f64;
    let (me, mn) = (
        impacts.iter().map(|i| i.0).sum::<f64>() / n,
        impacts.iter().map(|i| i.1).sum::<f64>() / n,
    );
    let sd =
        (impacts.iter().map(|i| (i.0 - me).powi(2) + (i.1 - mn).powi(2)).sum::<f64>() / n).sqrt();
    println!("gun {gun}: mean impact {me:.1} m east, {mn:.1} m north, spread {sd:.1} m");
    (fired, hits, sim.units()[0].destroyed)
}

#[test]
fn rounds_from_the_orbit_hit_a_few_at_a_time() {
    // The 40 mm on a BMP-2 for 20 s: about 40 rounds, a good share of them hits, and it goes.
    let (fired, hits, destroyed) = engage(0, GroundKind::Bmp2, 20);
    println!("40 mm: {fired} rounds, {hits} hits");
    assert!((35..=41).contains(&fired), "fired {fired}");
    assert!(hits >= 2 && hits * 100 < fired * 60, "{hits} hits of {fired}");
    assert!(destroyed);
    // The 105 mm: one round every 4 s; at most a couple of the first few miss.
    let (fired, hits, destroyed) = engage(1, GroundKind::Bmp2, 30);
    println!("105 mm: {fired} rounds, {hits} hits");
    assert!((7..=8).contains(&fired), "fired {fired}");
    assert!(destroyed && hits >= 1);
}

#[test]
fn the_guns_only_bear_off_the_left_wing() {
    let (_, mut sim, at) = gunship_sim();
    *sim.units_mut() = vec![vehicle(GroundKind::T90, at)];
    let mut fired = 0;
    // Heading south: the target is off the right wing.
    for _ in 0..15 * 5 {
        let st = orbit_state(at, 128, 3000.0, 0x8000, 30.0, 0);
        fired +=
            sim.step(&st, 15).iter().filter(|e| matches!(e, SimEvent::SideGunFired { .. })).count();
    }
    assert_eq!(fired, 0);
    assert!(sim.side_guns().unwrap().out_of_gimbal);
    // Straight overhead: below the gimbal.
    for _ in 0..15 * 5 {
        let st = orbit_state(at, 1, 3000.0, 0, 0.0, 0);
        fired +=
            sim.step(&st, 15).iter().filter(|e| matches!(e, SimEvent::SideGunFired { .. })).count();
    }
    assert_eq!(fired, 0);
}

#[test]
fn the_gunship_carries_its_load() {
    let (_, sim, _) = gunship_sim();
    let g = sim.side_guns().expect("side guns");
    assert_eq!(g.rounds, gunship::LOAD);
    assert_eq!(GUNS, 2);
    let (flares, chaff, _) = sim.stores();
    assert_eq!((flares, chaff), (gunship::FLARES, gunship::CHAFF));
}

/// A gunship mission end to end: the column sets off as the gunship arrives, the pod on one of
/// its vehicles after another from an orbit 2 km off and 3 km up, the guns destroy most of it
/// and the primary objective is done.
#[test]
fn the_gunship_destroys_the_column() {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 0,
        kind: MissionKind::Strike,
        land_bases: true,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(0x77)).unwrap();
    stealth_mission::gunship::for_gunship(&mut m, &d.gdata);
    let col = m.column.clone().expect("a column");
    let mut sim = Sim::with_options(&d.sim, &m, SimOptions { spectre: true, ..Default::default() });
    let m_per_wu = M_PER_MAP / 32.0;
    let start = col.route[0];
    let start_m = [start.0 as f64 * M_PER_MAP, (0x8000 - start.1) as f64 * M_PER_MAP];
    let to_bam = |rad: f64| (rad / std::f64::consts::TAU * 65536.0).round() as i32;
    let alt_m = 3000.0;
    let state = |target: [f64; 3], trigger: bool, next: bool| {
        // 2 km east of the target, flying north banked left: it is off the left wing.
        let me = [target[0] + 2000.0, target[1]];
        let d = [target[0] - me[0], target[1] - me[1], target[2] - alt_m];
        let pod_heading = to_bam(d[0].atan2(d[1])).rem_euclid(65536) as u16;
        let pod_pitch = to_bam(d[2].atan2(d[0].hypot(d[1]))) as i16;
        PlayerState {
            x: (me[0] / m_per_wu) as i32,
            y: (me[1] / m_per_wu) as i32,
            altitude: (alt_m / m_per_wu) as u16,
            height: (alt_m / m_per_wu) as u16,
            heading: 0,
            roll: to_bam(-30f64.to_radians()) as i16,
            velocity: 180 * 27,
            gear_up: true,
            fuel: 10000,
            hud_visible: true,
            pod: Some(PodAim { heading: pod_heading, pitch: pod_pitch, ground: None }),
            cmd: Commands { trigger, next_bay: next, ..Default::default() },
            ..Default::default()
        }
    };
    // Arrive: the column sets off.
    sim.step(&state([start_m[0], start_m[1], 0.0], false, false), 15);
    let hills = sim.hills();
    let column: Vec<usize> =
        (0..sim.units().len()).filter(|&k| sim.units()[k].fine.is_some()).collect();
    assert_eq!(column.len(), col.kinds.len(), "the column did not set off");
    let (mut done, mut fired, mut hits, mut secs) = (false, 0, 0, 0.0);
    for frame in 0..15 * 600 {
        let Some(&k) = column.iter().find(|&&k| !sim.units()[k].destroyed) else { break };
        let at = sim.units()[k].metres_on(&hills);
        // The tanks with the 105 mm.
        let want = sim.units()[k].kind.tank() as u8;
        let next = frame % 15 == 0 && sim.side_guns().unwrap().gun != want;
        for e in sim.step(&state(at, !next, next), 15) {
            match e {
                SimEvent::SideGunFired { .. } => fired += 1,
                SimEvent::SideShellImpact { armour: true, .. } => hits += 1,
                SimEvent::ObjectiveDone { primary: true } => done = true,
                _ => {}
            }
        }
        secs = frame as f64 / 15.0;
        if done {
            break;
        }
    }
    let down = column.iter().filter(|&&k| sim.units()[k].destroyed).count();
    println!("{down} of {} destroyed in {secs:.0} s: {fired} rounds, {hits} hits", column.len());
    assert!(done, "objective not done: {down} of {} destroyed", column.len());
    assert!(fired > hits && hits > 0);
}
