//! The fleet's war at sea on the shipped game data: a submarine's sea-skimmers and the attack
//! boats against the carrier's group, its Phalanx mounts fighting them, a submarine surfacing
//! to fire MANPADS at a helicopter low near it.

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{GroundKind, M_PER_MAP};
use stealth_sim::asw::*;
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

const M_PER_WU: f64 = M_PER_MAP / 32.0;
const FS: u8 = 30;

/// A helicopter's simulation in a theater with a carrier, nothing else about; flat sea (no
/// water function: all sea).
fn fleet() -> (Sim, usize, [f64; 2]) {
    let d = data();
    let s = MissionSettings {
        theater: 1,
        roe: 2,
        opponents: 1,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    let m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(0x5EA)).unwrap();
    let opts =
        SimOptions { seahawk: true, helicopter: true, external_stores: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    sim.flatten_hills();
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
        o.flags &= !0x0501;
        o.alert = 0;
    }
    for u in sim.flights_mut().iter_mut() {
        u.flags = 0;
    }
    sim.units_mut().clear();
    let carrier = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0x0200 != 0)
        .expect("a carrier");
    let at = object_metres(&sim.objects()[carrier]);
    (sim, carrier, at)
}

/// The aircraft at (m east, north), `height` m over the sea, heading north at `kt` knots.
fn at(e: f64, n: f64, height: f64, kt: f64) -> PlayerState {
    PlayerState {
        x: (e / M_PER_WU) as i32,
        y: (n / M_PER_WU) as i32,
        altitude: (height / M_PER_WU) as u16,
        height: (height / M_PER_WU) as u16,
        velocity: (kt * 27.0) as u16,
        gear_up: true,
        ..Default::default()
    }
}

fn msg(e: &SimEvent) -> Option<String> {
    match e {
        SimEvent::Message(t) => Some(t.tr_in(stealth_i18n::Language::En)),
        _ => None,
    }
}

/// A salvo at the carrier: its Phalanx mounts open fire and knock some down; every missile
/// ends, shot down or on a ship.
#[test]
fn the_phalanx_fights_a_salvo_of_sea_skimmers() {
    let (mut sim, _, c) = fleet();
    let ps = at(c[0], c[1] - 20_000.0, 300.0, 0.0);
    assert_eq!(sim.test_sea_salvo(4, 25.0), 4);
    assert!(sim.naval().mounts.len() >= 2, "{:?}", sim.naval().mounts);
    let (mut firing, mut down, mut hits) = (0, 0, 0);
    for _ in 0..FS as usize * 200 {
        for e in sim.step(&ps, FS) {
            if matches!(e, SimEvent::CiwsFiring { .. }) {
                firing += 1;
            }
            match msg(&e).as_deref() {
                Some("Phalanx: missile destroyed") => down += 1,
                Some(m) if m.contains("by a missile") => hits += 1,
                _ => {}
            }
        }
        if !sim.projectiles()[..8].iter().any(|p| p.ttl != 0) {
            break;
        }
    }
    println!("mounts fired {firing} times, {down} shot down, {hits} hits");
    assert!(firing > 0, "the mounts never fired");
    assert!(down > 0, "none shot down");
    assert_eq!(down + hits, 4, "every missile shot down or on a ship");
    assert!(!sim.naval().shells.is_empty() || firing > 0);
}

/// A swarm of attack boats weaves in at the fleet; the Phalanx mounts may sink some, but not
/// all: the others reach a ship (stopping them is the aircraft's work).
#[test]
fn attack_boats_close_on_the_fleet_and_the_phalanx_fights_them() {
    let (mut sim, _, c) = fleet();
    let ps = at(c[0], c[1] - 20_000.0, 300.0, 0.0);
    assert_eq!(sim.spawn_usv_swarm(5, 6.0), 5);
    let boats = |sim: &Sim| {
        (0..sim.units().len())
            .filter(|&i| sim.units()[i].kind == GroundKind::Usv)
            .collect::<Vec<_>>()
    };
    let first = boats(&sim)[0];
    let d0 = {
        let m = sim.units()[first].metres();
        (m[0] - c[0]).hypot(m[1] - c[1])
    };
    assert!((5_000.0..7_000.0).contains(&d0), "{d0:.0} m out");
    let (mut sunk_by_gun, mut hits) = (0, 0);
    for _ in 0..FS as usize * 600 {
        for e in sim.step(&ps, FS) {
            if let SimEvent::AaaDestroyed { by: None, .. } = e {
                sunk_by_gun += 1;
            }
            if msg(&e).is_some_and(|m| m.contains("by an attack boat")) {
                hits += 1;
            }
        }
        if boats(&sim).iter().all(|&i| sim.units()[i].destroyed) {
            break;
        }
    }
    println!("{sunk_by_gun} boats sunk by the Phalanx, {hits} reached a ship");
    assert!(boats(&sim).iter().all(|&i| sim.units()[i].destroyed), "boats still going");
    assert_eq!(sunk_by_gun + hits, 5);
    assert!(hits > 0, "the Phalanx alone stops a swarm: nothing left for the aircraft to do");
}

/// A submarine closing on the carrier comes up to periscope depth in missile range and fires
/// its salvo.
#[test]
fn a_submarine_fires_its_salvo_from_periscope_depth() {
    let (mut sim, carrier, c) = fleet();
    let setup = AswSetup {
        ocean: Ocean { layer: 60.0, sea_state: 1, bottom: 800.0 },
        subs: vec![SubSetup {
            at: [c[0] + 40_000.0, c[1]],
            heading: -std::f64::consts::FRAC_PI_2,
            intent: Intent::Attack { ship: carrier },
            nuclear: false,
        }],
        seed: 3,
    };
    sim.start_asw(&setup);
    let ps = at(c[0], c[1] - 20_000.0, 300.0, 0.0);
    let mut fired = 0;
    for _ in 0..FS as usize * 900 {
        for e in sim.step(&ps, FS) {
            if let SimEvent::MissileLaunched { spec, launcher: Launcher::Sub(_), .. } = e {
                assert_eq!(spec, naval::SEA_MISSILE_SPEC);
                assert!(sim.asw().subs[0].depth <= PERISCOPE_DEPTH + 2.0);
                fired += 1;
            }
        }
        if fired > 0 && sim.asw().subs[0].salvo == 0 {
            break;
        }
    }
    assert!((2..=4).contains(&fired), "{fired} missiles");
}

/// A helicopter passing low near a submarine: in time it surfaces and fires SA-14s at it; it
/// shows to everyone up there.
#[test]
fn a_submarine_surfaces_to_fire_manpads_at_a_low_helicopter() {
    let mut surfaced = 0;
    let mut shots = 0;
    for seed in 0..6u32 {
        let (mut sim, _, c) = fleet();
        let home = [c[0] + 30_000.0, c[1] + 30_000.0];
        let setup = AswSetup {
            ocean: Ocean { layer: 60.0, sea_state: 1, bottom: 800.0 },
            subs: vec![SubSetup {
                at: home,
                heading: 0.0,
                intent: Intent::Blockade {
                    a: [home[0] - 5000.0, home[1] + 50_000.0],
                    b: [home[0] + 5000.0, home[1] + 50_000.0],
                    goal: [home[0], home[1] + 60_000.0],
                },
                nuclear: false,
            }],
            seed,
        };
        sim.start_asw(&setup);
        // Passing over the sea 2 km behind it, 150 m up at 80 kt (the test keeps it there).
        let mut up = false;
        for k in 0..FS as usize * 300 {
            let s = sim.asw().subs[0].at;
            let ps = at(s[0], s[1] - 2000.0, 150.0, 80.0);
            for e in sim.step(&ps, FS) {
                if let SimEvent::MissileLaunched {
                    spec: 0x21, launcher: Launcher::Sub(_), ..
                } = e
                {
                    shots += 1;
                }
            }
            if k % FS as usize == 0 && !sim.asw().surfaced.is_empty() {
                up = true;
                assert!(!sim.asw().for_clients().surfaced.is_empty(), "the others see it");
            }
        }
        surfaced += up as usize;
    }
    println!("{surfaced} of 6 surfaced, {shots} SA-14s");
    assert!(surfaced > 0 && surfaced < 6, "{surfaced} of 6 surfaced");
    assert!(shots > 0);
}

/// A campaign sortie's raid: its boats all sunk (by whoever) is the player's objective; one
/// that reaches a ship spoils it.
#[test]
fn a_raid_sunk_is_the_players_objective() {
    for spoil in [false, true] {
        let (mut sim, carrier, c) = fleet();
        sim.set_pilot_target(0, Some((naval::RAID_MISSION, carrier)));
        sim.start_raid_for(0, carrier, 2);
        let boats: Vec<usize> =
            (0..sim.units().len()).filter(|&i| sim.units()[i].kind == GroundKind::Usv).collect();
        assert_eq!(boats.len(), 6);
        let ps = at(c[0], c[1] - 20_000.0, 300.0, 0.0);
        sim.step(&ps, FS);
        if spoil {
            // One straight onto the carrier.
            let f = sim.units()[boats[0]].fine.unwrap();
            sim.units_mut()[boats[0]].fine =
                Some(stealth_sim::aaa::Fine { e: c[0] as f32 + 20.0, n: c[1] as f32, ..f });
            sim.step(&ps, FS);
        }
        for &b in &boats {
            sim.units_mut()[b].destroyed = true;
        }
        let ev: Vec<SimEvent> = (0..FS).flat_map(|_| sim.step(&ps, FS)).collect();
        let done = ev.iter().any(|e| matches!(e, SimEvent::ObjectiveDone { primary: true }));
        assert_eq!(done, !spoil, "spoilt {spoil}: {ev:?}");
    }
}
