//! Campaign multi-crew: another player at the AC-130's gunner's console lays and fires its side
//! guns, the seats come and go with the aircraft, and the aircraft goes on with the gunner when
//! its pilot leaves it.

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{Fine, GroundKind, GroundUnit, M_PER_MAP};
use stealth_sim::crew::{CrewCmd, GunnerAim, Seat};
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

/// A campaign world over flat ground: pilot 0 flies an AC-130 from a friendly base, pilots 1
/// and 2 are in the operations room; a BMP-2 2 km west of the aircraft. Returns the BMP's map
/// point.
fn crewed() -> (Sim, (i32, i32)) {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 1,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(77)).unwrap();
    m.world = d.gdata.theaters[3].world.clone();
    m.weather = Default::default();
    let mut sim =
        Sim::with_options(&d.sim, &m, SimOptions { campaign: true, ..Default::default() });
    sim.flatten_hills();
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
    }
    sim.units_mut().clear();
    for u in sim.flights_mut().iter_mut() {
        u.flags = 0;
    }
    let base = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x401)
        .expect("friendly base");
    sim.add_pilot(1);
    sim.add_pilot(2);
    let spectre = Airframe { spectre: true, ..Default::default() };
    sim.respawn_pilot(0, base, None, spectre);
    let b = sim.objects()[base];
    let at = (b.x - 0x1000, b.y);
    let e = at.0 as f64 * M_PER_MAP;
    let n = (0x8000 - at.1) as f64 * M_PER_MAP;
    *sim.units_mut() = vec![GroundUnit {
        kind: GroundKind::Bmp2,
        x: at.0,
        y: at.1,
        fine: Some(Fine { e: e as f32, n: n as f32, heading: 0.0 }),
        ..Default::default()
    }];
    sim.step_pilots(&[(0, orbit(at, false))], 15);
    (sim, at)
}

/// Pilot 0 2 km east of `at`, 3 km up, flying north banked 30 degrees left, his pod on `at`,
/// his trigger held or not.
fn orbit(at: (i32, i32), trigger: bool) -> PlayerState {
    let dx = 128;
    let me = (at.0 + dx, at.1);
    let (x, y) = Sim::map_to_world(me.0, me.1);
    let to_bam = |rad: f64| (rad / std::f64::consts::TAU * 65536.0).round() as i32;
    let alt_m = 3000.0;
    let feet = (alt_m / (M_PER_MAP / 32.0)) as u16;
    PlayerState {
        x,
        y,
        altitude: feet,
        height: feet,
        roll: to_bam((-30f64).to_radians()) as i16,
        gear_up: true,
        fuel: 10000,
        hud_visible: true,
        pod: Some(pod_on(at, dx)),
        cmd: Commands { trigger, ..Default::default() },
        ..Default::default()
    }
}

/// The pod's line of sight from `dx` map units east of `at`, 3 km up, onto `at`.
fn pod_on(at: (i32, i32), dx: i32) -> PodAim {
    let to_bam = |rad: f64| (rad / std::f64::consts::TAU * 65536.0).round() as i32;
    let d = -dx as f64 * M_PER_MAP;
    PodAim {
        heading: to_bam(d.atan2(0.0)).rem_euclid(65536) as u16,
        pitch: to_bam(-(3000f64.atan2(d.abs()))) as i16,
        ground: Some(at),
    }
}

/// Side gun rounds fired in `frames` frames of pilot 0 flying `ps`.
fn fired(sim: &mut Sim, ps: &PlayerState, frames: usize) -> usize {
    (0..frames)
        .flat_map(|_| sim.step_pilots(&[(0, ps.clone())], 15))
        .filter(|e| matches!(e.event, SimEvent::SideGunFired { .. }))
        .count()
}

#[test]
fn a_gunner_sits_at_a_free_console_only() {
    let (mut sim, _) = crewed();
    assert!(sim.gunner_seat_free(0));
    assert!(sim.crew_command(0, CrewCmd::Join { owner: 0 }).unwrap_err().is("crew-own-aircraft"));
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    assert_eq!(sim.crew_of(1), Some((0, Seat::Gunner)));
    assert_eq!(sim.gunner_of(0), Some(1));
    assert!(sim.crew_command(2, CrewCmd::Join { owner: 0 }).unwrap_err().is("crew-seat-taken"));
    // The pilot can't move to the gunner's seat while it is taken.
    assert!(sim.crew_command(0, CrewCmd::Seat(Seat::Gunner)).unwrap_err().is("crew-seat-taken"));
    sim.crew_command(1, CrewCmd::Leave).unwrap();
    assert_eq!(sim.crew_of(1), None);
    // Now he can; nobody else then.
    sim.crew_command(0, CrewCmd::Seat(Seat::Gunner)).unwrap();
    assert_eq!(sim.owner_seat(0), Seat::Gunner);
    assert!(!sim.gunner_seat_free(0));
    assert!(sim.crew_command(2, CrewCmd::Join { owner: 0 }).is_err());
    sim.crew_command(0, CrewCmd::Seat(Seat::Pilot)).unwrap();
    assert!(sim.crews().is_empty(), "an owner alone in the pilot's seat keeps no record");
}

#[test]
fn the_gunner_lays_and_fires_the_side_guns() {
    let (mut sim, at) = crewed();
    // Alone, the pilot's own trigger fires them.
    assert!(fired(&mut sim, &orbit(at, true), 30) > 0);
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    // With a gunner the pilot's trigger does nothing, nor does the gunner's without his pod.
    for _ in 0..30 {
        sim.step_pilots(&[(0, orbit(at, false))], 15);
    }
    assert_eq!(fired(&mut sim, &orbit(at, true), 30), 0, "the pilot's trigger fired");
    sim.crew_command(1, CrewCmd::Aim(GunnerAim { pod: None, trigger: true, ..Default::default() }))
        .unwrap();
    assert_eq!(fired(&mut sim, &orbit(at, false), 30), 0, "fired without a pod");
    // The gunner's pod on the BMP and his trigger: the 40 mm fires, his choice of gun goes.
    let aim = GunnerAim { pod: Some(pod_on(at, 128)), trigger: true, ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    let mut pilot = orbit(at, false);
    pilot.pod = None;
    assert!(fired(&mut sim, &pilot, 30) > 0, "the gunner's trigger did not fire");
    sim.crew_command(1, CrewCmd::Aim(GunnerAim { gun: 1, ..aim })).unwrap();
    sim.step_pilots(&[(0, pilot.clone())], 15);
    assert_eq!(sim.side_guns().unwrap().gun, 1);
    // His designation goes to the pilot's.
    sim.crew_command(1, CrewCmd::Designate { at: None, unit: Some(0) }).unwrap();
    sim.step_pilots(&[(0, pilot)], 15);
    let view = sim.crew_view(1).expect("crew view");
    assert_eq!((view.owner, view.spi_unit), (0, Some(0)));
    assert_eq!(view.pod, Some(pod_on(at, 128)));
}

#[test]
fn the_seats_go_with_the_aircraft() {
    let (mut sim, _) = crewed();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    sim.end_sortie_pilot(0, 0);
    sim.step_pilots(&[], 15);
    assert_eq!(sim.crew_of(1), None, "the gunner's seat outlived the aircraft");
    assert!(sim.crews().is_empty());
}

#[test]
fn the_aircraft_goes_on_with_its_gunner() {
    let (mut sim, at) = crewed();
    for _ in 0..15 {
        sim.step_pilots(&[(0, orbit(at, true))], 15);
    }
    let rounds = sim.side_guns().unwrap().rounds;
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    sim.take_pilot_events();
    let start = sim.transfer_pilot(0, 1).expect("transferred");
    assert!(sim.pilot_absent(0));
    assert!(sim.pilot_in_the_air(1));
    assert_eq!(start.altitude, orbit(at, false).altitude);
    assert_eq!(sim.crew_of(1), Some((1, Seat::Gunner)));
    assert_eq!(sim.owner_seat(1), Seat::Gunner);
    let ended: Vec<_> = sim
        .take_pilot_events()
        .into_iter()
        .filter(|e| matches!(e.event, SimEvent::SortieEnded(_)))
        .collect();
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0].pilot, 0);
    let SimEvent::SortieEnded(e) = &ended[0].event else { unreachable!() };
    assert_eq!(e.outcome, OUTCOME_HANDED_OVER);
    sim.as_pilot(1, |s| assert_eq!(s.side_guns().unwrap().rounds, rounds, "a fresh load"));
}

/// A campaign world as [`crewed`] with pilot 0 in an AH-64 instead, Stingers on the wingtips and
/// Hellfires on the pylons (the left wingtip's Stingers selected), hovering 300 m up 1 km east
/// of the BMP. Returns the BMP's map point.
fn apache() -> (Sim, (i32, i32)) {
    let (mut sim, _) = crewed();
    let base = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x401)
        .expect("friendly base");
    // The BMP 1.5 km north-east of the base.
    let b = sim.objects()[base];
    let at = (b.x + 0x60, b.y - 0x40);
    let (e, n) = (at.0 as f64 * M_PER_MAP, (0x8000 - at.1) as f64 * M_PER_MAP);
    let u = &mut sim.units_mut()[0];
    (u.x, u.y) = at;
    u.fine = Some(Fine { e: e as f32, n: n as f32, heading: 0.0 });
    let airframe = Airframe {
        external_stores: true,
        helicopter: true,
        gunship: true,
        apache: true,
        ..Default::default()
    };
    sim.respawn_pilot(0, base, None, airframe);
    sim.set_pilot_loadout(0, &[(21, 2), (20, 4), (20, 4), (20, 4), (20, 4), (21, 2)], 0);
    sim.step_pilots(&[(0, hover(at, false))], 15);
    (sim, at)
}

/// Map units east of the BMP the AH-64 hovers.
const HOVER_DX: i32 = 64;

/// Pilot 0's AH-64 hovering 300 m up `HOVER_DX` east of `at` facing north, his helmet sight
/// looking north along the nose, his own gun trigger held or not.
fn hover(at: (i32, i32), gun: bool) -> PlayerState {
    let (x, y) = Sim::map_to_world(at.0 + HOVER_DX, at.1);
    let feet = (300.0 / (M_PER_MAP / 32.0)) as u16;
    PlayerState {
        x,
        y,
        altitude: feet,
        height: feet,
        fuel: 10000,
        hud_visible: true,
        hud_mode: HudMode::Gnd,
        helmet: Some(PodAim { heading: 0, pitch: -0x100, ground: None }),
        cmd: Commands { gun, ..Default::default() },
        ..Default::default()
    }
}

/// The TADS's line of sight from the hover onto `at`.
fn tads_on(at: (i32, i32)) -> PodAim {
    let to_bam = |rad: f64| (rad / std::f64::consts::TAU * 65536.0).round() as i32;
    let d = HOVER_DX as f64 * M_PER_MAP;
    PodAim { heading: 0xC000, pitch: to_bam(-(300f64.atan2(d))) as i16, ground: Some(at) }
}

/// The gun rounds in the air: (east, north) velocity of the newest.
fn rounds(sim: &Sim) -> Vec<(i32, i32)> {
    let (b, n) = sim.bullets();
    b[..n].iter().filter(|b| b.x != 0).map(|b| (b.vx, -b.vy)).collect()
}

#[test]
fn the_apache_has_a_gunner_seat() {
    let (mut sim, _) = apache();
    assert!(sim.gunner_seat_free(0));
    sim.crew_command(0, CrewCmd::Seat(Seat::Gunner)).unwrap();
    assert_eq!(sim.owner_seat(0), Seat::Gunner);
    sim.crew_command(0, CrewCmd::Seat(Seat::Pilot)).unwrap();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    assert_eq!(sim.crew_of(1), Some((0, Seat::Gunner)));
    let v = sim.crew_view(1).expect("the gunner's view");
    assert_eq!(v.stores[1], (20, 4));
    assert!(v.gun_ammo > 0);
}

/// The gunner's trigger fires the chin gun along his TADS (west, at the BMP) while the pilot's
/// helmet looks north; the pilot's own trigger still fires it along his helmet.
#[test]
fn the_apache_gunner_fires_the_chin_gun_along_his_sight() {
    let (mut sim, at) = apache();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    let aim = GunnerAim { pod: Some(tads_on(at)), trigger: true, ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    let ammo = sim.gun_ammo();
    for _ in 0..5 {
        sim.step_pilots(&[(0, hover(at, false))], 15);
    }
    assert!(sim.gun_ammo() < ammo, "the gunner fired");
    let r = rounds(&sim);
    assert!(!r.is_empty() && r.iter().all(|&(e, n)| e < 0 && n.abs() < -e / 4), "west: {r:?}");
    // He lets go; the pilot fires along his helmet, north.
    sim.crew_command(1, CrewCmd::Aim(GunnerAim { trigger: false, ..aim })).unwrap();
    for _ in 0..60 {
        sim.step_pilots(&[(0, hover(at, false))], 15);
    }
    for _ in 0..5 {
        sim.step_pilots(&[(0, hover(at, true))], 15);
    }
    let r = rounds(&sim);
    assert!(!r.is_empty() && r.iter().all(|&(e, n)| n > 0 && e.abs() < n / 4), "north: {r:?}");
}

/// The gunner's Hellfire goes from the station he selected (the pilot's Stingers stay
/// selected) and flies to his TADS's spot.
#[test]
fn the_apache_gunner_fires_a_hellfire_at_his_spot() {
    let (mut sim, at) = apache();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    let aim = GunnerAim { pod: Some(tads_on(at)), station: Some(1), ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    sim.step_pilots(&[(0, hover(at, false))], 15);
    sim.crew_command(1, CrewCmd::Fire { station: 1 }).unwrap();
    sim.step_pilots(&[(0, hover(at, false))], 15);
    let left: i32 = sim.pilot_bays(0).iter().filter(|b| b.0 == 20).map(|b| b.1).sum();
    assert_eq!(left, 15, "one Hellfire away");
    assert_eq!(sim.pilot_bays(0)[0], (21, 2), "the Stingers kept");
    let mut hit = None;
    for _ in 0..15 * 30 {
        for e in sim.step_pilots(&[(0, hover(at, false))], 15) {
            if let SimEvent::Explosion { x, y, .. } = e.event {
                hit.get_or_insert((x, y));
            }
        }
        if hit.is_some() {
            break;
        }
    }
    let (x, y) = hit.expect("the Hellfire hit");
    assert!((x - at.0).abs() <= 8 && (y - at.1).abs() <= 8, "hit {x},{y}, spot {at:?}");
}

/// A campaign world as [`apache`] with pilot 0 in an OH-58D instead, Hellfires on the left
/// pylon and a gun pod on the right, hovering as the AH-64 does. Returns the BMP's map point.
fn kiowa() -> (Sim, (i32, i32)) {
    let (mut sim, at) = apache();
    let base = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x401)
        .expect("friendly base");
    let airframe =
        Airframe { external_stores: true, helicopter: true, kiowa: true, ..Default::default() };
    sim.respawn_pilot(0, base, None, airframe);
    sim.set_pilot_loadout(0, &[(20, 2), (22, 1)], 0);
    sim.step_pilots(&[(0, hover(at, false))], 15);
    (sim, at)
}

#[test]
fn the_kiowa_has_a_gunner_seat() {
    let (mut sim, _) = kiowa();
    assert!(sim.gunner_seat_free(0));
    sim.crew_command(0, CrewCmd::Seat(Seat::Gunner)).unwrap();
    assert_eq!(sim.owner_seat(0), Seat::Gunner);
    sim.crew_command(0, CrewCmd::Seat(Seat::Pilot)).unwrap();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    assert_eq!(sim.crew_of(1), Some((0, Seat::Gunner)));
    let v = sim.crew_view(1).expect("the gunner's view");
    assert_eq!(v.stores[0], (20, 2));
    assert_eq!(v.stores[1], (22, 1));
}

/// The OH-58D's gunner's trigger does not fire the gun pods (they fire along the nose, the
/// pilot's), nor does firing the pod's station; the pilot's own trigger still fires them.
#[test]
fn the_kiowa_gunner_leaves_the_gun_pods_to_the_pilot() {
    let (mut sim, at) = kiowa();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    let aim = GunnerAim { pod: Some(tads_on(at)), trigger: true, ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    let ammo = sim.gun_ammo();
    assert!(ammo > 0);
    for _ in 0..5 {
        sim.step_pilots(&[(0, hover(at, false))], 15);
    }
    sim.crew_command(1, CrewCmd::Fire { station: 1 }).unwrap();
    sim.step_pilots(&[(0, hover(at, false))], 15);
    assert_eq!(sim.gun_ammo(), ammo, "the gunner fired no rounds");
    assert!(rounds(&sim).is_empty());
    assert_eq!(sim.pilot_bays(0), &[(20, 2), (22, 1)][..]);
    for _ in 0..5 {
        sim.step_pilots(&[(0, hover(at, true))], 15);
    }
    assert!(sim.gun_ammo() < ammo, "the pilot fired the pods");
}

/// The OH-58D's gunner's Hellfire flies to his mast sight's spot.
#[test]
fn the_kiowa_gunner_fires_a_hellfire_at_his_spot() {
    let (mut sim, at) = kiowa();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    let aim = GunnerAim { pod: Some(tads_on(at)), station: Some(0), ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    sim.step_pilots(&[(0, hover(at, false))], 15);
    sim.crew_command(1, CrewCmd::Fire { station: 0 }).unwrap();
    sim.step_pilots(&[(0, hover(at, false))], 15);
    assert_eq!(sim.pilot_bays(0)[0], (20, 1), "one Hellfire away");
    let mut hit = None;
    for _ in 0..15 * 30 {
        for e in sim.step_pilots(&[(0, hover(at, false))], 15) {
            if let SimEvent::Explosion { x, y, .. } = e.event {
                hit.get_or_insert((x, y));
            }
        }
        if hit.is_some() {
            break;
        }
    }
    let (x, y) = hit.expect("the Hellfire hit");
    assert!((x - at.0).abs() <= 8 && (y - at.1).abs() <= 8, "hit {x},{y}, spot {at:?}");
}

/// A campaign world as [`apache`] with pilot 0 in an RAH-66 instead, Hellfires on the left bay
/// door and Stingers on the right, hovering as the AH-64 does. Returns the BMP's map point.
fn comanche() -> (Sim, (i32, i32)) {
    let (mut sim, at) = apache();
    let base = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x401)
        .expect("friendly base");
    let airframe =
        Airframe { helicopter: true, gunship: true, comanche: true, ..Default::default() };
    sim.respawn_pilot(0, base, None, airframe);
    sim.set_pilot_loadout(0, &[(20, 3), (21, 6)], 0);
    sim.step_pilots(&[(0, hover(at, false))], 15);
    (sim, at)
}

#[test]
fn the_comanche_has_a_gunner_seat() {
    let (mut sim, _) = comanche();
    assert!(sim.gunner_seat_free(0));
    sim.crew_command(0, CrewCmd::Seat(Seat::Gunner)).unwrap();
    assert_eq!(sim.owner_seat(0), Seat::Gunner);
    sim.crew_command(0, CrewCmd::Seat(Seat::Pilot)).unwrap();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    assert_eq!(sim.crew_of(1), Some((0, Seat::Gunner)));
    let v = sim.crew_view(1).expect("the gunner's view");
    assert_eq!(v.stores[0], (20, 3));
    assert_eq!(v.stores[1], (21, 6));
    assert!(v.gun_ammo > 0);
}

/// The RAH-66's gunner's trigger fires the chin gun along his sight (west, at the BMP) while
/// the pilot's helmet looks north.
#[test]
fn the_comanche_gunner_fires_the_chin_gun_along_his_sight() {
    let (mut sim, at) = comanche();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    let aim = GunnerAim { pod: Some(tads_on(at)), trigger: true, ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    let ammo = sim.gun_ammo();
    for _ in 0..5 {
        sim.step_pilots(&[(0, hover(at, false))], 15);
    }
    assert!(sim.gun_ammo() < ammo, "the gunner fired");
    let r = rounds(&sim);
    assert!(!r.is_empty() && r.iter().all(|&(e, n)| e < 0 && n.abs() < -e / 4), "west: {r:?}");
}

/// The RAH-66's gunner's Hellfire leaves the bay (its doors opening for it) and flies to his
/// sight's spot.
#[test]
fn the_comanche_gunner_fires_a_hellfire_at_his_spot() {
    let (mut sim, at) = comanche();
    sim.crew_command(1, CrewCmd::Join { owner: 0 }).unwrap();
    let aim = GunnerAim { pod: Some(tads_on(at)), station: Some(0), ..Default::default() };
    sim.crew_command(1, CrewCmd::Aim(aim)).unwrap();
    sim.step_pilots(&[(0, hover(at, false))], 15);
    let mut hit = None;
    for k in 0..15 * 30 {
        // The release is held until the doors are open.
        if sim.pilot_bays(0)[0].1 == 3 && k % 5 == 0 {
            sim.crew_command(1, CrewCmd::Fire { station: 0 }).unwrap();
        }
        for e in sim.step_pilots(&[(0, hover(at, false))], 15) {
            if let SimEvent::Explosion { x, y, .. } = e.event {
                hit.get_or_insert((x, y));
            }
        }
        if hit.is_some() {
            break;
        }
    }
    assert_eq!(sim.pilot_bays(0)[0], (20, 2), "one Hellfire away");
    let (x, y) = hit.expect("the Hellfire hit");
    assert!((x - at.0).abs() <= 8 && (y - at.1).abs() <= 8, "hit {x},{y}, spot {at:?}");
}
