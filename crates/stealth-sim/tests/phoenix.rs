//! The F-14's radar-guided missiles on the shipped game data: the AIM-54 Phoenix, guided by the
//! AWG-9 to its own track-while-scan track until its seeker comes on, and the semi-active AIM-7
//! Sparrow, which needs the radar's single target track all the way.

use std::sync::OnceLock;

use stealth_mission::{GameData, Mission, MissionKind, MissionSettings, Seed};
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

fn mission(d: &Data) -> Mission {
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 1,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    stealth_mission::generate(&d.gdata, &s, Seed::Srand(0x1234)).unwrap()
}

/// A clean sandbox (no radars, no aircraft, flat ground): a map cell centre in open country.
fn sandbox(sim: &mut Sim, m: &Mission) -> (i32, i32) {
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
    (((c % 16) as i32) * 2048 + 1024, ((c / 16) as i32) * 2048 + 1024)
}

/// The F-14 at map (mx, my), 30,000 ft, northeast at 450 kt, air-to-air.
fn f14_at(mx: i32, my: i32) -> PlayerState {
    let (x, y) = Sim::map_to_world(mx, my);
    PlayerState {
        x,
        y,
        altitude: 30000,
        height: 30000,
        heading: 0x2000,
        velocity: 450 * 27,
        gear_up: true,
        fuel: 10000,
        hud_visible: true,
        hud_mode: HudMode::Air,
        ..Default::default()
    }
}

/// An-72s (unit type 9) flying straight on northeast `range` map units northeast of (mx, my) at
/// 20,000 ft, one beside the other `spread` map units apart across the line: flights 1..=n.
fn targets(sim: &mut Sim, (mx, my): (i32, i32), range: i32, spread: i32, n: usize) {
    let diag = (range as f64 / std::f64::consts::SQRT_2).round() as i32;
    let side = (spread as f64 / std::f64::consts::SQRT_2).round() as i32;
    let wp = 6;
    {
        let o = &mut sim.objects_mut()[wp];
        (o.x, o.y) = (mx + diag + 8000, my - diag - 8000);
    }
    for k in 0..n {
        let off = k as i32 - (n as i32 - 1) / 2;
        let (x, y) = (mx + diag + off * side, my - diag + off * side);
        let u = &mut sim.flights_mut()[1 + k];
        *u = FlightUnit {
            x,
            y,
            alt: 20000,
            heading: 0x2000,
            spec: 9,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: wp as i32,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
}

fn sim_with(guided: bool, stores: &[(i32, i32)]) -> (Sim, (i32, i32)) {
    let d = data();
    let m = mission(d);
    let opts = SimOptions { realistic_guidance: guided, tomcat: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let at = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(stores);
    (sim, at)
}

/// Runs the sim with `p` until `secs` have passed; the units shot down.
fn fly(sim: &mut Sim, p: &mut PlayerState, secs: i32) -> Vec<i32> {
    let mut down = Vec::new();
    for _ in 0..15 * secs {
        for e in sim.step(p, 15) {
            if let SimEvent::AircraftDestroyed { unit, .. } = e {
                down.push(unit as i32);
            }
        }
        p.cmd = Commands::default();
    }
    down
}

/// Six Phoenixes fired in two seconds at six An-72s 60 km away, each at its own track while the
/// AWG-9 keeps tracking all six: every one goes for its own, and all six come down.
#[test]
fn six_phoenixes_kill_six_tracks() {
    for guided in [true, false] {
        let (mut sim, at) = sim_with(guided, &[(PHOENIX_WEAPON, 6)]);
        targets(&mut sim, at, 3800, 120, 6);
        let mut p = f14_at(at.0, at.1);
        p.cmd.toggle_bay = true;
        fly(&mut sim, &mut p, 1);
        p.support = (1..=6).collect();
        let mut aimed = Vec::new();
        for j in 1..=6u16 {
            p.launch_unit = Some(j);
            p.cmd.fire = true;
            fly(&mut sim, &mut p, 1);
            let slot = sim.projectiles()[8..]
                .iter()
                .filter(|m| m.ttl != 0 && m.spec == PHOENIX_SPEC)
                .map(|m| m.target)
                .collect::<Vec<_>>();
            aimed = slot;
        }
        aimed.sort();
        assert_eq!(aimed, vec![1, 2, 3, 4, 5, 6], "guided {guided}: one at each track");
        assert!(
            sim.projectiles()[8..].iter().filter(|m| m.ttl != 0).all(|m| m.midcourse.is_some()),
            "guided {guided}: in midcourse"
        );
        let mut down = fly(&mut sim, &mut p, 150);
        down.sort();
        down.dedup();
        assert_eq!(down, vec![1, 2, 3, 4, 5, 6], "guided {guided}");
    }
}

/// A Phoenix's own seeker comes on within ten miles of its target, not before.
#[test]
fn phoenix_goes_active_at_ten_miles() {
    let (mut sim, at) = sim_with(true, &[(PHOENIX_WEAPON, 1)]);
    targets(&mut sim, at, 3800, 0, 1);
    let mut p = f14_at(at.0, at.1);
    p.cmd.toggle_bay = true;
    fly(&mut sim, &mut p, 1);
    p.support = vec![1];
    p.launch_unit = Some(1);
    p.cmd.fire = true;
    let mut went_active = None;
    for _ in 0..15 * 120 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let Some(m) = sim.projectiles()[8..].iter().find(|m| m.ttl != 0) else { break };
        if m.midcourse.is_none() {
            let u = sim.flights()[1];
            let (dx, dy) = ((u.x - m.x) as f64, (u.y - m.y) as f64);
            went_active = Some(dx.hypot(dy));
            break;
        }
    }
    let r = went_active.expect("went active");
    assert!((900.0..1300.0).contains(&r), "active at {r} map units");
}

/// PH ACT: a Phoenix fired with the switch on is active off the rail.
#[test]
fn phoenix_fired_active() {
    let (mut sim, at) = sim_with(true, &[(PHOENIX_WEAPON, 1)]);
    targets(&mut sim, at, 900, 0, 1);
    let mut p = f14_at(at.0, at.1);
    p.cmd.toggle_bay = true;
    fly(&mut sim, &mut p, 1);
    p.launch_unit = Some(1);
    p.ph_active = true;
    p.cmd.fire = true;
    sim.step(&p, 15);
    let m = sim.projectiles()[8..].iter().find(|m| m.ttl != 0).expect("fired");
    assert!(m.midcourse.is_none());
}

/// The Sparrow kills its target while the radar holds it in a single target track; with the
/// lock broken after the launch it flies on and misses.
#[test]
fn sparrow_needs_the_lock() {
    for (held, guided) in [(true, true), (false, true), (true, false), (false, false)] {
        let (mut sim, at) = sim_with(guided, &[(SPARROW_WEAPON, 1)]);
        targets(&mut sim, at, 1500, 0, 1);
        let mut p = f14_at(at.0, at.1);
        p.cmd.toggle_bay = true;
        p.cmd.lock_unit = Some(1);
        fly(&mut sim, &mut p, 1);
        p.cmd.fire = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        if !held {
            p.cmd.clear_target = true;
        }
        let down = fly(&mut sim, &mut p, 90);
        assert_eq!(down.contains(&1), held, "held {held} guided {guided}");
    }
}
