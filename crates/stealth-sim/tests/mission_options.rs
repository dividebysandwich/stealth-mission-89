//! A single mission's options on the shipped game data: how many air defences and enemy
//! aircraft, and the SEAD package taking off ahead of the player.

use std::sync::OnceLock;

use stealth_mission::world::flags as F;
use stealth_mission::{Amount, GameData, Mission, MissionKind, MissionSettings, Seed};
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

fn mission(
    theater: usize,
    seed: u16,
    kind: MissionKind,
    f: impl Fn(&mut MissionSettings),
) -> Mission {
    let mut s = MissionSettings { theater, roe: 2, opponents: 1, kind, ..Default::default() };
    f(&mut s);
    stealth_mission::generate(&data().gdata, &s, Seed::Srand(seed)).unwrap()
}

/// The enemy's SAM sites standing, and the sum of their radar types (newer is higher).
fn sams(m: &Mission) -> (usize, i32) {
    let sites = stealth_mission::defences::sam_sites(&m.world, &data().gdata.tables);
    (sites.len(), sites.iter().map(|&i| m.world.objects[i].unit_type as i32).sum())
}

/// None leaves no SAM site, low fewer, high more or newer; the targets, bases and route stay
/// as the classic mission's.
#[test]
fn air_defences_level() {
    for theater in [0, 3, 5, 9] {
        for seed in [7u16, 1234, 0x4242] {
            let base = mission(theater, seed, MissionKind::Strike, |_| {});
            let with = |a| mission(theater, seed, MissionKind::Strike, |s| s.air_defences = a);
            let (none, low, high) = (with(Amount::None), with(Amount::Low), with(Amount::High));
            let n = sams(&base);
            assert!(n.0 > 0, "{theater} {seed}: no SAM site at all");
            assert_eq!(sams(&none).0, 0, "{theater} {seed}");
            assert!(sams(&low).0 < n.0, "{theater} {seed}: {:?} {n:?}", sams(&low));
            assert!(sams(&high).0 >= n.0 && sams(&high).1 > n.1, "{theater} {seed}");
            for m in [&none, &low, &high] {
                assert_eq!(m.primary, base.primary);
                assert_eq!(m.secondary, base.secondary);
                assert_eq!(m.waypoints, base.waypoints);
                assert_eq!(m.world.flights, base.world.flights);
            }
        }
    }
}

/// The realistic air defence's guns: none, about half, about twice.
#[test]
fn air_defence_guns_follow_the_level() {
    let d = data();
    let opts = SimOptions { air_defence: true, ..Default::default() };
    let guns = |a| {
        let m = mission(3, 1234, MissionKind::Strike, |s| s.air_defences = a);
        Sim::with_options(&d.sim, &m, opts.clone()).units().len()
    };
    let n = guns(Amount::Normal);
    assert!(n > 4, "{n}");
    assert_eq!(guns(Amount::None), 0);
    // (Every other of the guns before the cap on all units.)
    let low = guns(Amount::Low);
    assert!(low * 3 > n && low * 3 < n * 2, "{low} {n}");
    assert!(guns(Amount::High) > n * 3 / 2, "{} {n}", guns(Amount::High));
}

/// Enemy aircraft up at the start (not the bases' traffic) and parked.
fn enemy(sim: &Sim) -> (usize, usize) {
    let fl = sim.flights();
    let n = fl.len().saturating_sub(4);
    let up = fl[..n].iter().filter(|u| u.flags & 3 == 3).count();
    let parked = fl[..n].iter().filter(|u| u.flags & 3 == 1).count();
    (up, parked)
}

/// None: nobody up, nobody parked to scramble; low: half the patrols; high: more up.
#[test]
fn enemy_aircraft_level() {
    let d = data();
    for theater in [0, 3, 5] {
        for seed in [7u16, 1234] {
            let sim = |a| {
                let m = mission(theater, seed, MissionKind::Strike, |s| s.enemy_aircraft = a);
                Sim::new(&d.sim, &m)
            };
            let n = enemy(&sim(Amount::Normal));
            assert!(n.0 > 0, "{theater} {seed}: no patrol");
            assert_eq!(enemy(&sim(Amount::None)), (0, 0), "{theater} {seed}");
            assert_eq!(enemy(&sim(Amount::Low)).0, n.0.div_ceil(2), "{theater} {seed}");
            if n.1 > 0 {
                assert!(enemy(&sim(Amount::High)).0 > n.0, "{theater} {seed}");
            }
        }
    }
}

/// An air mission's target still flies with no enemy aircraft.
#[test]
fn no_enemy_aircraft_keeps_an_air_missions_target() {
    let d = data();
    let mut checked = 0;
    for seed in 1..60u16 {
        let m = mission(3, seed, MissionKind::Air, |s| s.enemy_aircraft = Amount::None);
        if m.escort != 0 {
            continue;
        }
        let sim = Sim::new(&d.sim, &m);
        assert!(sim.flights()[0].flags & 1 != 0, "{seed}: the target is gone");
        checked += 1;
    }
    assert!(checked > 0);
}

/// The SEAD package: its F-16s take off from the player's base one after another while he
/// waits on the runway, and their HARMs destroy the SAM sites on his route, no other object.
#[test]
fn sead_package_destroys_the_sams_on_the_route() {
    let d = data();
    let mut flown = 0;
    for seed in [7u16, 1234, 0x4242, 99] {
        let m = mission(3, seed, MissionKind::Strike, |s| s.sead = true);
        let mut sim = Sim::new(&d.sim, &m);
        let targets = sim.sead_targets(&m);
        if targets.is_empty() {
            continue;
        }
        assert!(targets.len() <= sead::SEAD_MAX);
        for &t in &targets {
            let o = m.world.objects[t];
            assert!(o.unit_type > 0 && o.flags & (F::FRIENDLY | F::NEUTRAL) == 0);
        }
        flown += 1;
        let p = sim.start_state();
        let (mut takeoffs, mut destroyed) = (0, Vec::new());
        for _ in 0..15 * 60 * 20 {
            for e in sim.step(&p, 15) {
                match e {
                    SimEvent::AircraftTakeoff { unit, base } => {
                        if sim.friendly_aircraft(sim.flights()[unit].spec)
                            && base == m.takeoff.object_index as usize
                        {
                            takeoffs += 1;
                        }
                    }
                    SimEvent::GroundDestroyed { object, .. } => destroyed.push(object),
                    SimEvent::MissionEnd(end) => panic!("{seed}: ended {end:?}"),
                    _ => {}
                }
            }
        }
        assert!(takeoffs >= targets.len(), "{seed}: {takeoffs} took off for {targets:?}");
        assert!(!destroyed.is_empty(), "{seed}: no site destroyed of {targets:?}");
        assert!(destroyed.iter().all(|o| targets.contains(o)), "{seed}: {destroyed:?} {targets:?}");
        assert_eq!(sim.sead_waiting(), 0);
    }
    assert!(flown > 0, "no SAM site on any route");
}

/// Without the options the mission and the simulation are the classic ones.
#[test]
fn options_off_change_nothing() {
    let a = mission(3, 1234, MissionKind::Strike, |_| {});
    let b = mission(3, 1234, MissionKind::Strike, |s| {
        s.air_defences = Amount::Normal;
        s.enemy_aircraft = Amount::Normal;
    });
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    assert!(!serde_json::to_string(&a.settings).unwrap().contains("air_defences"));
}
