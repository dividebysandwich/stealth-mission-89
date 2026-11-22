//! Free flight on the shipped game data: no enemy anywhere, only friendly aircraft about, a
//! round trip over random places.

use std::sync::OnceLock;

use stealth_mission::world::flags as F;
use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
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

fn free_flight(theater: usize, seed: u16, helicopter: bool) -> stealth_mission::Mission {
    let s = MissionSettings {
        theater,
        roe: 2,
        opponents: 0,
        kind: MissionKind::Strike,
        helicopter,
        free_flight: true,
        ..Default::default()
    };
    stealth_mission::generate(&data().gdata, &s, Seed::Srand(seed)).unwrap()
}

/// The mission: every object friendly or neutral without a radar, no enemy aircraft, three
/// distinct points a leg apart (a helicopter's short), then home.
#[test]
fn free_flight_mission_has_no_enemy() {
    for theater in 0..stealth_formats::ALL_THEATERS.len() {
        for seed in [7u16, 0x4242] {
            for heli in [false, true] {
                let m = free_flight(theater, seed, heli);
                assert!(m.quiet && m.drones.is_none() && m.special_aircraft.is_none());
                assert!(
                    m.world
                        .objects
                        .iter()
                        .all(|o| o.unit_type == 0 && o.flags & (F::FRIENDLY | F::NEUTRAL) != 0)
                );
                assert!(m.world.grid.iter().all(|g| g & 3 != 0 && g & 0x10 == 0));
                assert_eq!(m.landing.object_index, m.takeoff.object_index);
                let w = &m.waypoints;
                assert_eq!((w[3].x, w[3].y), (m.takeoff.position.x, m.takeoff.position.y));
                assert!(w[0] != w[1] && w[1] != w[2] && w[0] != w[2], "{theater}: {w:?}");
                assert_eq!((m.primary.mission_type, m.secondary.mission_type), (0, 0));
                assert!(!m.time.has_deadline);
                let km = |a: (i32, i32), b: &stealth_mission::MapPoint| {
                    ((a.0 - b.x) as f64).hypot((a.1 - b.y) as f64) / 64.0
                };
                let longest = if heli { 18.5 } else { 70.5 };
                let mut at = (m.takeoff.position.x, m.takeoff.position.y);
                for p in &w[..3] {
                    assert!(km(at, p) <= longest, "{theater} {seed} {heli}: {w:?}");
                    at = (p.x, p.y);
                }
            }
        }
    }
}

/// Ten minutes flown low over the route with the air defences option on: nothing shoots, no
/// enemy takes off, only friendly aircraft fly.
#[test]
fn free_flight_nobody_shoots() {
    let d = data();
    let trig = &d.sim.trig;
    let mut friendly_up = 0;
    for theater in 0..9 {
        let m = free_flight(theater, 0x100 + theater as u16, false);
        let opts = SimOptions { air_defence: true, ..Default::default() };
        let mut sim = Sim::with_options(&d.sim, &m, opts);
        assert!(sim.units().is_empty(), "{theater}: AAA placed");
        let mut p = sim.start_state();
        let (mut wx, mut wy) = (p.x as i64, p.y as i64);
        let mut alt = 0i32;
        for f in 0..15 * 600 {
            let wp = m.waypoints[(f / (15 * 200)).min(3)];
            let (tx, ty) = Sim::map_to_world(wp.x, wp.y);
            let h = stealth_sim::math::bearing(
                ((tx as i64 - wx) >> 6) as i32,
                ((ty as i64 - wy) >> 6) as i32,
            );
            p.heading = h as u16;
            alt = (alt + 20).min(400);
            p.altitude = alt as u16;
            p.height = p.altitude;
            p.velocity = 450 * 27;
            p.on_ground = false;
            p.gear_up = true;
            let v = (p.velocity as i32 * 4) / 15 / 64;
            wx += trig.sin_mul(h as i32, v) as i64 * 4;
            wy += trig.cos_mul(h as i32, v) as i64 * 4;
            p.x = wx as i32;
            p.y = wy as i32;
            for e in sim.step(&p, 15) {
                match e {
                    SimEvent::AircraftTakeoff { unit, .. } => {
                        let spec = sim.flights()[unit].spec;
                        assert!(sim.friendly_aircraft(spec), "{theater}: enemy {spec} took off");
                    }
                    SimEvent::MissionEnd(end) => panic!("{theater}: ended {end:?}"),
                    _ => {}
                }
            }
            for u in sim.flights() {
                if u.flags & 1 != 0 && u.flags & 2 != 0 {
                    assert!(sim.friendly_aircraft(u.spec), "{theater}: enemy {} up", u.spec);
                    friendly_up += (u.speed > 0) as usize;
                }
            }
        }
        assert_eq!(sim.launches(), 0, "{theater}: missiles fired at the player");
        assert_eq!(sim.damage_effects().hits, 2, "{theater}: hit");
    }
    // The bases' friendly traffic flies.
    assert!(friendly_up > 0);
}
