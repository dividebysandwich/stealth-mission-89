//! The submarine hunt on the shipped game data: sonobuoys hearing a submarine's bearing, the
//! layer hiding a submarine, a DICASS ping making it run, the buoys' limit, the dipping sonar's
//! cable, the MAD, the Mk 54, and the submarines torpedoing a ship or slipping
//! through a blockade.

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::M_PER_MAP;
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
/// Where the Seahawk works (m east, north): open sea as far as the test knows (no water
/// function: all sea).
const HOME: [f64; 2] = [200_000.0, 200_000.0];

/// A Seahawk's simulation with nothing else about; flat sea.
fn seahawk() -> Sim {
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
    sim
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

fn run(sim: &mut Sim, ps: &PlayerState, secs: f64) -> Vec<SimEvent> {
    let mut ev = Vec::new();
    for _ in 0..(secs * FS as f64) as usize {
        ev.extend(sim.step(ps, FS));
    }
    ev
}

/// A hunt with one submarine at `sub` (m east, north) heading `heading` (rad), the layer at
/// `layer` m, out to close on a far away world object.
fn hunt(sim: &mut Sim, sub: [f64; 2], heading: f64, layer: f32, intent: Option<Intent>) {
    let ship = (3..sim.objects().len()).find(|&i| sim.objects()[i].flags & 0x0200 == 0).unwrap();
    let setup = AswSetup {
        ocean: Ocean { layer, sea_state: 1, bottom: 800.0 },
        subs: vec![SubSetup {
            at: sub,
            heading,
            intent: intent.unwrap_or(Intent::Approach { ship }),
            nuclear: false,
        }],
        seed: 7,
    };
    sim.start_asw(&setup);
}

fn sub(sim: &Sim) -> &Sub {
    &sim.asw().subs[0]
}

fn crew(sim: &Sim) -> &AswCrew {
    sim.asw().crew(0).expect("the Seahawk's crew")
}

fn drop_at(sim: &mut Sim, e: f64, n: f64, kind: BuoyKind, deep: bool) {
    // High enough not to be heard by the submarine (it would stop and go silent).
    let ps = at(e, n, 100.0, 0.0);
    run(sim, &ps, 0.2);
    sim.asw_command(0, AswCmd::DropBuoy { kind, deep, at: None }).expect("dropped");
}

#[test]
fn difar_buoys_give_bearings_but_crossing_them_is_the_operators_work() {
    let mut sim = seahawk();
    let target = [HOME[0] + 1500.0, HOME[1] + 1500.0];
    hunt(&mut sim, target, 0.3, 30.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Difar, true);
    drop_at(&mut sim, HOME[0] + 3000.0, HOME[1], BuoyKind::Difar, true);
    drop_at(&mut sim, HOME[0], HOME[1] + 3000.0, BuoyKind::Difar, true);
    run(&mut sim, &at(HOME[0], HOME[1] - 2000.0, 100.0, 0.0), 150.0);
    let c = crew(&sim);
    assert!(c.buoys_left[0] == BUOY_LOAD[0] - 3);
    // Every buoy hears it, with a bearing near the truth and its tonal lines.
    let truth = sub(&sim).at;
    for b in sim.asw().buoys.iter() {
        let l = b.sample.lines.iter().find(|l| !l.ship && !l.ping).expect("heard");
        assert!(l.bearing.is_some() && l.freqs[0] > 0, "{:?}", b.sample);
        let want = (truth[0] - b.at[0]).atan2(truth[1] - b.at[1]).to_degrees().rem_euclid(360.0);
        let off = (l.bearing.unwrap() as f64 - want + 540.0).rem_euclid(360.0) - 180.0;
        assert!(off.abs() < 15.0, "bearing {:?}, truth {want:.0}", l.bearing);
    }
    // No automatic fixes or tracks from the bearings.
    assert!(c.fixes.is_empty() && c.tracks.is_empty(), "{:?} {:?}", c.fixes, c.tracks);
}

#[test]
fn the_layer_hides_a_deep_submarine_from_a_shallow_hydrophone() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 2000.0, HOME[1]], 0.0, 50.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Difar, false);
    drop_at(&mut sim, HOME[0], HOME[1] + 50.0, BuoyKind::Difar, true);
    run(&mut sim, &at(HOME[0], HOME[1] - 2000.0, 60.0, 0.0), 20.0);
    assert!(sub(&sim).depth > 50.0);
    let snr = |deep: bool| {
        let depth = BUOY_DEPTHS[deep as usize];
        sim.asw()
            .buoys
            .iter()
            .find(|b| b.depth == depth)
            .and_then(|b| b.sample.lines.iter().find(|l| !l.ship && !l.ping))
            .map(|l| l.snr)
    };
    assert!(snr(true).is_some(), "the deep one hears it");
    assert!(snr(false).is_none(), "the shallow one doesn't: {:?}", snr(false));
}

#[test]
fn a_bt_buoy_reports_the_layer() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 20_000.0, HOME[1]], 0.0, 75.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Bt, false);
    assert_eq!(crew(&sim).layer, None);
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 40.0);
    assert_eq!(crew(&sim).layer, Some(75));
    // It sinks after a while: its slot frees.
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 80.0);
    assert_eq!(sim.asw().buoys.len(), 0);
}

#[test]
fn a_dicass_ping_ranges_the_submarine_and_makes_it_run() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 3000.0, HOME[1] + 1000.0], 0.0, 200.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Dicass, true);
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 5.0);
    assert_eq!(sub(&sim).posture, Posture::Transit);
    let id = sim.asw().buoys[0].id;
    sim.asw_command(0, AswCmd::BuoyMode { id, active: true }).unwrap();
    let mut ranged = None;
    for _ in 0..60 {
        run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 0.25);
        if let Some(r) = sim.asw().buoys[0].sample.lines.iter().find_map(|l| l.range) {
            ranged = Some(r);
        }
    }
    let truth = sub(&sim).at;
    let r = (truth[0] - HOME[0]).hypot(truth[1] - HOME[1]);
    let got = ranged.expect("an echo") as f64;
    assert!((got - r).abs() < 200.0, "echo at {got} m, the submarine at {r:.0} m");
    assert_eq!(sub(&sim).posture, Posture::Evade);
    assert!(crew(&sim).fixes.iter().any(|f| f.how == 1));
}

#[test]
fn no_more_than_four_buoys_at_once() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 60.0, None);
    for k in 0..4 {
        drop_at(&mut sim, HOME[0] + k as f64 * 500.0, HOME[1], BuoyKind::Difar, false);
    }
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 0.2);
    assert!(
        sim.asw_command(0, AswCmd::DropBuoy { kind: BuoyKind::Difar, deep: false, at: None })
            .is_err()
    );
    let id = sim.asw().buoys[0].id;
    sim.asw_command(0, AswCmd::Scuttle { id }).unwrap();
    // A fifth DIFAR is not aboard (four carried); a BT goes.
    assert!(
        sim.asw_command(0, AswCmd::DropBuoy { kind: BuoyKind::Difar, deep: false, at: None })
            .is_err()
    );
    assert!(
        sim.asw_command(0, AswCmd::DropBuoy { kind: BuoyKind::Bt, deep: false, at: None }).is_ok()
    );
}

#[test]
fn a_drop_on_the_map_waits_for_the_aircraft_to_get_there() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 60.0, None);
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 0.2);
    let point = (HOME[0] as i32 + 2000, HOME[1] as i32);
    sim.asw_command(0, AswCmd::DropBuoy { kind: BuoyKind::Difar, deep: false, at: Some(point) })
        .unwrap();
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 1.0);
    assert!(sim.asw().buoys.is_empty());
    // The drop is a waypoint of the route, for the aircraft to fly to.
    let crew = sim.asw().crew(0).unwrap();
    assert_eq!(
        crew.route.last().copied(),
        Some(stealth_sim::asw::wu_point([point.0 as f64, point.1 as f64]))
    );
    run(&mut sim, &at(HOME[0] + 2000.0, HOME[1], 30.0, 0.0), 1.0);
    assert_eq!(sim.asw().buoys.len(), 1);
}

#[test]
fn the_route_carries_its_drops_moved_and_deleted_with_their_points() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 60.0, None);
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 0.2);
    let a = stealth_sim::asw::wu_point([HOME[0] + 2000.0, HOME[1]]);
    let b = stealth_sim::asw::wu_point([HOME[0] + 4000.0, HOME[1]]);
    let route = |points: Vec<(i32, i32)>, drops| AswCmd::Route { rev: 10, points, drops };
    sim.asw_command(0, route(vec![a, b], vec![(1, BuoyKind::Dicass, true)])).unwrap();
    let c = sim.asw().crew(0).unwrap();
    assert_eq!(c.pending.len(), 1);
    assert_eq!(c.pending[0].2, stealth_sim::asw::wu_metres(b));
    // Moved: the drop with its point; passing the old point drops nothing.
    let moved = stealth_sim::asw::wu_point([HOME[0] + 6000.0, HOME[1]]);
    sim.asw_command(0, route(vec![a, moved], vec![(1, BuoyKind::Dicass, true)])).unwrap();
    run(&mut sim, &at(HOME[0] + 4000.0, HOME[1], 30.0, 0.0), 1.0);
    assert!(sim.asw().buoys.is_empty());
    run(&mut sim, &at(HOME[0] + 6000.0, HOME[1], 30.0, 0.0), 1.0);
    assert_eq!(sim.asw().buoys.len(), 1);
    // Deleted with its point: nothing waits.
    sim.asw_command(0, route(vec![a, b], vec![(1, BuoyKind::Difar, false)])).unwrap();
    sim.asw_command(0, route(vec![a], vec![])).unwrap();
    assert!(sim.asw().crew(0).unwrap().pending.is_empty());
    // No more drops than buoys aboard (four DIFARs).
    let many: Vec<(i32, i32)> = (0..5).map(|k| (a.0 + k * 10, a.1)).collect();
    let drops = (0..5).map(|k| (k as u8, BuoyKind::Difar, false)).collect();
    assert!(sim.asw_command(0, route(many, drops)).is_err());
}

#[test]
fn the_dipping_sonar_goes_down_from_the_hover_and_its_cable_parts_if_it_flies_off() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 4000.0, HOME[1]], 0.0, 30.0, None);
    // Too high to dip.
    run(&mut sim, &at(HOME[0], HOME[1], 60.0, 0.0), 0.2);
    assert!(sim.asw_command(0, AswCmd::Dip(true)).is_err());
    run(&mut sim, &at(HOME[0], HOME[1], 15.0, 0.0), 0.2);
    sim.asw_command(0, AswCmd::DipDepth(90)).unwrap();
    sim.asw_command(0, AswCmd::Dip(true)).unwrap();
    run(&mut sim, &at(HOME[0], HOME[1], 15.0, 0.0), 40.0);
    assert_eq!(crew(&sim).dip.state, DipState::Down);
    sim.asw_command(0, AswCmd::DipMode { active: true }).unwrap();
    let mut echo = false;
    for _ in 0..48 {
        run(&mut sim, &at(HOME[0], HOME[1], 15.0, 0.0), 0.25);
        echo |= crew(&sim).dip.sample.lines.iter().any(|l| l.range.is_some());
    }
    assert!(echo, "{:?}", crew(&sim).dip.sample);
    run(&mut sim, &at(HOME[0], HOME[1], 15.0, 30.0), 1.0);
    assert_eq!(crew(&sim).dip.state, DipState::Broken);
    assert!(sim.asw_command(0, AswCmd::Dip(true)).is_err());
}

#[test]
fn the_mad_marks_a_submarine_it_passes_over_and_nothing_far_off() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0], HOME[1] + 3000.0], 0.0, 200.0, None);
    run(&mut sim, &at(HOME[0], HOME[1] - 3000.0, 30.0, 60.0), 0.2);
    sim.asw_command(0, AswCmd::Mad(true)).unwrap();
    run(&mut sim, &at(HOME[0], HOME[1] - 3000.0, 30.0, 60.0), 5.0);
    assert!(crew(&sim).mad.abs() < 20, "{}", crew(&sim).mad);
    assert!(crew(&sim).fixes.is_empty());
    // Over it: a mark near it.
    let s = sub(&sim).at;
    run(&mut sim, &at(s[0], s[1] + 60.0, 30.0, 60.0), 1.0);
    let c = crew(&sim);
    let s = sub(&sim).at;
    let mark = c.fixes.iter().find(|f| f.how == 2).expect("a MAD mark");
    assert!((mark.at[0] - s[0]).hypot(mark.at[1] - s[1]) < 250.0);
}

#[test]
fn a_programmed_torpedo_finds_and_sinks_the_submarine() {
    let mut sim = seahawk();
    // The submarine 600 m north-east, running east.
    hunt(&mut sim, [HOME[0] + 450.0, HOME[1] + 450.0], std::f64::consts::FRAC_PI_2, 200.0, None);
    sim.set_loadout(&[(20, 4), (MK54_WEAPON, 1), (MK54_WEAPON, 1)]);
    let ps = at(HOME[0], HOME[1], 25.0, 0.0);
    run(&mut sim, &ps, 0.5);
    let program = TorpProgram {
        heading: Some(45),
        delay: 5,
        pattern: Pattern::Circle,
        ceiling: 30,
        floor: 250,
    };
    sim.asw_command(0, AswCmd::Program(program)).unwrap();
    assert!(sim.asw_command(0, AswCmd::Release { station: 1 }).is_err(), "not armed");
    sim.asw_command(0, AswCmd::Arm(true)).unwrap();
    sim.asw_command(0, AswCmd::Release { station: 1 }).unwrap();
    let mut ev = run(&mut sim, &ps, 1.0);
    assert_eq!(sim.bays().0[1], (MK54_WEAPON, 0));
    assert_eq!(sim.asw().torpedoes.len(), 1);
    assert!(!crew(&sim).armed);
    for _ in 0..240 {
        ev.extend(run(&mut sim, &ps, 1.0));
        if !sub(&sim).alive {
            break;
        }
    }
    assert!(!sub(&sim).alive, "torpedo {:?} sub {:?}", sim.asw().torpedoes, sub(&sim));
    assert_eq!(sim.asw().result, AswResult::Won);
    assert!(
        ev.iter().any(|e| matches!(e, SimEvent::AswSplash { kind: 1, .. })),
        "the entry splash"
    );
    assert!(ev.iter().any(|e| matches!(e, SimEvent::AswSplash { kind: 2, .. })), "the plume");
}

#[test]
fn the_buoys_hear_a_torpedo_running_where_it_is() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 30.0, None);
    drop_at(&mut sim, HOME[0] - 3000.0, HOME[1], BuoyKind::Difar, true);
    sim.set_loadout(&[(20, 4), (MK54_WEAPON, 1), (MK54_WEAPON, 1)]);
    let ps = at(HOME[0], HOME[1], 25.0, 0.0);
    run(&mut sim, &ps, 0.5);
    sim.asw_command(0, AswCmd::Arm(true)).unwrap();
    sim.asw_command(0, AswCmd::Release { station: 1 }).unwrap();
    run(&mut sim, &ps, 20.0);
    let t = sim.asw().torpedoes[0].p;
    let b = &sim.asw().buoys[0];
    let l = b.sample.lines.iter().find(|l| l.torpedo).expect("the torpedo heard");
    let truth = ((t[0] - b.at[0]).atan2(t[1] - b.at[1]).to_degrees().rem_euclid(360.0)) as i32;
    let got = l.bearing.expect("its bearing") as i32;
    let off = ((got - truth + 540) % 360 - 180).abs();
    assert!(off < 15, "bearing {got}, truly {truth}");
}

#[test]
fn a_torpedo_is_not_dropped_too_fast_or_high() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 60.0, None);
    sim.set_loadout(&[(20, 4), (MK54_WEAPON, 1), (MK54_WEAPON, 1)]);
    let ps = at(HOME[0], HOME[1], 300.0, 140.0);
    run(&mut sim, &ps, 0.5);
    sim.asw_command(0, AswCmd::Arm(true)).unwrap();
    sim.asw_command(0, AswCmd::Release { station: 2 }).unwrap();
    run(&mut sim, &ps, 1.0);
    assert_eq!(sim.bays().0[2], (MK54_WEAPON, 1));
    assert!(sim.asw().torpedoes.is_empty());
}

#[test]
fn a_submarine_left_alone_torpedoes_its_ship() {
    let mut sim = seahawk();
    let ship = (3..sim.objects().len()).find(|&i| sim.objects()[i].flags & 0x0200 == 0).unwrap();
    let o = sim.objects()[ship];
    let target = object_metres(&o);
    hunt(
        &mut sim,
        [target[0] - 9000.0, target[1]],
        std::f64::consts::FRAC_PI_2,
        60.0,
        Some(Intent::Approach { ship }),
    );
    let ps = at(HOME[0], HOME[1], 30.0, 0.0);
    let mut ev = Vec::new();
    for _ in 0..1200 {
        ev.extend(run(&mut sim, &ps, 1.0));
        if sim.asw().result != AswResult::Hunting {
            break;
        }
    }
    assert_eq!(sim.asw().result, AswResult::ShipLost);
    assert!(sim.objects()[ship].flags & oflags::DESTROYED != 0);
    assert!(ev.iter().any(|e| matches!(e, SimEvent::AswSplash { kind: 3, .. })));
}

#[test]
fn a_submarine_through_the_blockade_fails_the_hunt() {
    let mut sim = seahawk();
    // The line runs east-west 2 km north of it; it heads north for a point beyond.
    let s = [HOME[0], HOME[1]];
    let a = [s[0] - 10_000.0, s[1] + 2000.0];
    let b = [s[0] + 10_000.0, s[1] + 2000.0];
    let goal = [s[0], s[1] + 6000.0];
    hunt(&mut sim, s, 0.0, 60.0, Some(Intent::Blockade { a, b, goal }));
    let ps = at(HOME[0] - 20_000.0, HOME[1], 30.0, 0.0);
    for _ in 0..900 {
        run(&mut sim, &ps, 1.0);
        if sim.asw().result != AswResult::Hunting {
            break;
        }
    }
    assert_eq!(sim.asw().result, AswResult::Escaped);
    assert!(sub(&sim).at[1] > a[1]);
}

#[test]
fn the_buoys_hear_the_ships_too_but_the_tracker_leaves_them_out() {
    let mut sim = seahawk();
    let ship = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0x0200 != 0)
        .expect("a carrier");
    let at_ship = object_metres(&sim.objects()[ship]);
    hunt(&mut sim, [at_ship[0] + 30_000.0, at_ship[1]], 0.0, 30.0, None);
    drop_at(&mut sim, at_ship[0] + 3000.0, at_ship[1], BuoyKind::Difar, false);
    drop_at(&mut sim, at_ship[0], at_ship[1] + 3000.0, BuoyKind::Difar, false);
    run(&mut sim, &at(at_ship[0], at_ship[1] - 5000.0, 60.0, 0.0), 30.0);
    let b = &sim.asw().buoys[0];
    assert!(b.sample.lines.iter().any(|l| l.ship && l.bearing.is_some()), "{:?}", b.sample);
    assert!(crew(&sim).tracks.is_empty(), "{:?}", crew(&sim).tracks);
}

#[test]
fn the_others_never_see_the_submarines() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 1000.0, HOME[1]], 0.0, 60.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Difar, true);
    let c = sim.asw().for_clients();
    assert!(c.subs.is_empty());
    assert_eq!(c.buoys.len(), 1);
}

#[test]
fn a_campaign_hunt_spawns_off_the_carrier_and_its_sinking_is_the_hunters_objective() {
    let mut sim = seahawk();
    let carrier = (0..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0x0200 != 0)
        .expect("a carrier");
    let c = object_metres(&sim.objects()[carrier]);
    sim.set_pilot_target(0, Some((HUNT_MISSION, carrier)));
    sim.start_hunt_for(0, HuntTarget::Carrier(carrier), 99);
    let s = sub(&sim).clone();
    let d = (s.at[0] - c[0]).hypot(s.at[1] - c[1]);
    assert!((21_000.0..=29_000.0).contains(&d), "{d:.0} m off the carrier");
    assert_eq!(s.hunter, Some(0));
    // Another sortie's hunt replaces it.
    sim.start_hunt_for(0, HuntTarget::Carrier(carrier), 7);
    assert_eq!(sim.asw().subs.len(), 1);
    // A torpedo onto it, from close by.
    let near = [sub(&sim).at[0] - 450.0, sub(&sim).at[1] - 450.0];
    sim.set_loadout(&[(20, 4), (MK54_WEAPON, 1), (MK54_WEAPON, 1)]);
    let ps = at(near[0], near[1], 25.0, 0.0);
    run(&mut sim, &ps, 0.5);
    let heading =
        (sub(&sim).at[0] - near[0]).atan2(sub(&sim).at[1] - near[1]).to_degrees().rem_euclid(360.0)
            as u16;
    let program = TorpProgram {
        heading: Some(heading),
        delay: 5,
        pattern: Pattern::Circle,
        ceiling: 30,
        floor: 250,
    };
    sim.asw_command(0, AswCmd::Program(program)).unwrap();
    sim.asw_command(0, AswCmd::Arm(true)).unwrap();
    sim.asw_command(0, AswCmd::Release { station: 1 }).unwrap();
    let mut ev = Vec::new();
    for _ in 0..300 {
        ev.extend(run(&mut sim, &ps, 1.0));
        if !sub(&sim).alive {
            break;
        }
    }
    assert!(!sub(&sim).alive);
    assert!(ev.iter().any(|e| matches!(e, SimEvent::ObjectiveDone { primary: true })));
}

#[test]
fn a_buoy_hears_another_sonars_ping() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 60.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Difar, true);
    drop_at(&mut sim, HOME[0] + 2000.0, HOME[1], BuoyKind::Dicass, true);
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 5.0);
    let difar = sim.asw().buoys[0].id;
    let dicass = sim.asw().buoys[1].id;
    sim.asw_command(0, AswCmd::BuoyMode { id: dicass, active: true }).unwrap();
    let (mut heard, mut own) = (false, false);
    for _ in 0..48 {
        run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 0.25);
        let b = |id| sim.asw().buoys.iter().find(|b| b.id == id).unwrap().sample.clone();
        heard |= b(difar).heard
            && b(difar)
                .lines
                .iter()
                .any(|l| l.ping && l.freqs[0] == PING_LINE_HZ[0] && l.bearing.is_some());
        own |= b(dicass).heard;
    }
    assert!(heard, "the DIFAR heard the DICASS ping, a line with its bearing");
    assert!(!own, "the DICASS doesn't count its own");
}

#[test]
fn another_sonars_ping_comes_back_off_a_quiet_submarine_at_its_bearing() {
    let mut sim = seahawk();
    // Too far off the DIFAR to be heard running quietly; the DICASS 3 km from it.
    hunt(&mut sim, [HOME[0] + 8000.0, HOME[1]], 0.0, 200.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Difar, true);
    drop_at(&mut sim, HOME[0] + 8000.0, HOME[1] + 3000.0, BuoyKind::Dicass, true);
    run(&mut sim, &at(HOME[0], HOME[1], 30.0, 0.0), 5.0);
    let difar = sim.asw().buoys[0].id;
    let dicass = sim.asw().buoys[1].id;
    let b = |sim: &Sim, id| sim.asw().buoys.iter().find(|b| b.id == id).unwrap().sample.clone();
    assert!(
        !b(&sim, difar).lines.iter().any(|l| !l.ship && !l.ping && !l.torpedo),
        "not heard passively: {:?}",
        b(&sim, difar)
    );
    sim.asw_command(0, AswCmd::BuoyMode { id: dicass, active: true }).unwrap();
    // Frame by frame: when the DICASS pings, when the DIFAR hears the ping and its echo.
    let (mut sent, mut ping_at, mut echo) = (None, None, None);
    let ps = at(HOME[0], HOME[1], 30.0, 0.0);
    for _ in 0..20 * FS as usize {
        let seq = b(&sim, difar).seq;
        sim.step(&ps, FS);
        let s = b(&sim, difar);
        if s.seq == seq {
            continue;
        }
        let t = sim.asw().clock;
        if sent.is_none() && b(&sim, dicass).ping {
            sent = Some((t, sim.asw().buoys[1].at, sub(&sim).at));
        }
        if sent.is_some() && s.heard && ping_at.is_none() {
            ping_at = Some(t);
        }
        if let Some(l) = s.lines.iter().find(|l| l.echo) {
            echo = Some((t, *l));
            break;
        }
    }
    let (t0, p, s_at) = sent.expect("the DICASS pinged");
    let ping_at = ping_at.expect("the ping heard");
    let (t, l) = echo.expect("its echo off the submarine");
    // Each at the speed of sound, to the report nearest: the ping straight from the DICASS
    // (some 5.7 s), the echo the longer way round by the submarine (some 7.3 s).
    let d = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    let tick = 1.0 / SENSE_HZ;
    let direct = d(p, HOME) / 1500.0;
    assert!(
        (ping_at - t0 - direct).abs() <= tick / 2.0 + 0.05,
        "ping after {:.2} s, {direct:.2} s due",
        ping_at - t0
    );
    let round = (d(p, s_at) + d(s_at, HOME)) / 1500.0;
    assert!(
        (t - t0 - round).abs() <= tick / 2.0 + 0.05,
        "echo after {:.2} s, {round:.2} s due",
        t - t0
    );
    let brg = l.bearing.expect("a DIFAR's bearing") as i32;
    assert!((brg - 90).abs() <= 10, "the echo at {brg}, the submarine at 090");
}

#[test]
fn a_submarine_goes_silent_under_a_hovering_helicopter_and_every_so_often() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 2000.0, HOME[1]], 0.0, 200.0, None);
    drop_at(&mut sim, HOME[0], HOME[1], BuoyKind::Difar, true);
    let away = at(HOME[0] - 8000.0, HOME[1], 300.0, 100.0);
    run(&mut sim, &away, 10.0);
    let heard = |sim: &Sim| {
        sim.asw().buoys[0].sample.lines.iter().any(|l| !l.ship && !l.ping && !l.torpedo && !l.echo)
    };
    assert!(heard(&sim), "heard running: {:?}", sim.asw().buoys[0].sample);
    // The helicopter comes to a hover over it: it stops dead, nothing to hear.
    let s = sub(&sim).at;
    let over = at(s[0], s[1], 15.0, 0.0);
    run(&mut sim, &over, 30.0);
    assert_eq!(sub(&sim).posture, Posture::Silent);
    assert!(sub(&sim).speed < 0.5, "stopped: {}", sub(&sim).speed);
    assert!(!heard(&sim), "silent: {:?}", sim.asw().buoys[0].sample);
    // It stays silent a while after the helicopter has gone, then runs on.
    run(&mut sim, &away, 60.0);
    assert_eq!(sub(&sim).posture, Posture::Silent);
    run(&mut sim, &away, 150.0);
    assert_ne!(sub(&sim).posture, Posture::Silent);
    // Left alone, it stops now and then of its own accord.
    let mut silent = false;
    for _ in 0..1500 {
        run(&mut sim, &away, 1.0);
        silent |= sub(&sim).posture == Posture::Silent;
    }
    assert!(silent, "a silent stop in 25 minutes: {:?}", sub(&sim));
}

#[test]
fn a_campaign_barrier_hunt_starts_on_the_enemys_side_of_the_line_and_makes_for_the_far_side() {
    let mut sim = seahawk();
    let fields: Vec<usize> =
        (3..sim.objects().len()).filter(|&i| sim.objects()[i].x != 0).collect();
    // Two places some way apart (the bases): the line between them.
    let (a, b) = fields
        .iter()
        .flat_map(|&a| fields.iter().map(move |&b| (a, b)))
        .find(|&(a, b)| {
            let (pa, pb) = (object_metres(&sim.objects()[a]), object_metres(&sim.objects()[b]));
            (40_000.0..90_000.0).contains(&(pa[0] - pb[0]).hypot(pa[1] - pb[1]))
        })
        .expect("two bases");
    let (pa, pb) = (object_metres(&sim.objects()[a]), object_metres(&sim.objects()[b]));
    let mid = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
    // The enemy to the east of the line's middle.
    let enemy = (((mid[0] + 100_000.0) / M_PER_MAP) as i32, 0x8000 - (mid[1] / M_PER_MAP) as i32);
    sim.set_pilot_target(0, Some((HUNT_MISSION, a)));
    sim.start_hunt_for(0, HuntTarget::Barrier { a, b, enemy }, 5);
    let s = sub(&sim).clone();
    let Intent::Blockade { goal, .. } = s.intent else { panic!("{:?}", s.intent) };
    let side = |p: [f64; 2]| (pb[0] - pa[0]) * (p[1] - pa[1]) - (pb[1] - pa[1]) * (p[0] - pa[0]);
    let toward_enemy = side([mid[0] + 100_000.0, mid[1]]);
    assert!(side(s.at) * toward_enemy > 0.0, "on the enemy's side");
    assert!(side(goal) * toward_enemy < 0.0, "making for the far side");
    let d = (s.at[0] - mid[0]).hypot(s.at[1] - mid[1]);
    assert!((21_000.0..40_000.0).contains(&d), "{d:.0} m off the line");
    // A narrow sea's water: shallow, the submarine off the bottom.
    assert_eq!(sim.asw().ocean.bottom, 100.0);
    assert!(s.depth <= 80.0);
    // The crew sees the line on the map.
    assert_eq!(crew(&sim).barrier, Some((pa, pb)));
}

#[test]
fn the_operator_asks_for_the_dip_hover() {
    let mut sim = seahawk();
    hunt(&mut sim, [HOME[0] + 30_000.0, HOME[1]], 0.0, 60.0, None);
    run(&mut sim, &at(HOME[0], HOME[1], 300.0, 90.0), 0.2);
    sim.asw_command(0, AswCmd::Hover(true)).unwrap();
    assert!(crew(&sim).hover_req);
    sim.asw_command(0, AswCmd::Hover(false)).unwrap();
    assert!(!crew(&sim).hover_req);
}

#[test]
fn the_dipping_sonar_pings_much_farther_than_a_dicass() {
    // Beyond a DICASS's reach, well within the dipping sonar's.
    let far = [HOME[0] + 20_000.0, HOME[1]];
    let echo = |dip: bool| {
        let mut sim = seahawk();
        hunt(&mut sim, far, 0.0, 200.0, None);
        let ps = at(HOME[0], HOME[1], 12.0, 0.0);
        run(&mut sim, &ps, 0.2);
        if dip {
            sim.asw_command(0, AswCmd::DipDepth(60)).unwrap();
            sim.asw_command(0, AswCmd::Dip(true)).unwrap();
            run(&mut sim, &ps, 25.0);
            sim.asw_command(0, AswCmd::DipMode { active: true }).unwrap();
        } else {
            sim.asw_command(0, AswCmd::DropBuoy { kind: BuoyKind::Dicass, deep: true, at: None })
                .unwrap();
            run(&mut sim, &ps, 5.0);
            let id = sim.asw().buoys[0].id;
            sim.asw_command(0, AswCmd::BuoyMode { id, active: true }).unwrap();
        }
        let mut ranged = None;
        for _ in 0..60 {
            run(&mut sim, &ps, 0.25);
            let s =
                if dip { crew(&sim).dip.sample.clone() } else { sim.asw().buoys[0].sample.clone() };
            ranged = ranged.or(s.lines.iter().find_map(|l| l.range));
        }
        ranged
    };
    assert_eq!(echo(false), None, "out of the DICASS's reach");
    let r = echo(true).expect("the dipping sonar's echo") as f64;
    assert!((r - 20_000.0).abs() < 1000.0, "{r}");
}

/// A generated hunt: sinking its submarine achieves both objectives; ending the flight from
/// the menu then keeps those points (it only misses the landing's), landing adds them.
#[test]
fn a_single_missions_hunt_won_scores_even_when_ended_from_the_menu() {
    let d = data();
    let theater = (0..32)
        .find(|&t| {
            d.gdata
                .theaters
                .get(t)
                .is_some_and(|th| th.world.objects.iter().any(|o| o.flags & 0x0200 != 0))
        })
        .expect("a theater with a carrier");
    let s = MissionSettings {
        theater,
        kind: MissionKind::Strike,
        helicopter: true,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(5)).unwrap();
    stealth_mission::asw::for_asw(&mut m, &d.gdata, Some(stealth_mission::asw::Scenario::Carrier));
    let opts =
        SimOptions { seahawk: true, helicopter: true, external_stores: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    assert_eq!(sim.asw().subs.len(), 1);
    sim.set_loadout(&[(20, 4), (MK54_WEAPON, 1), (MK54_WEAPON, 1)]);
    let s0 = sub(&sim).at;
    let near = [s0[0] - 450.0, s0[1] - 450.0];
    let ps = at(near[0], near[1], 25.0, 0.0);
    run(&mut sim, &ps, 0.5);
    let heading =
        (sub(&sim).at[0] - near[0]).atan2(sub(&sim).at[1] - near[1]).to_degrees().rem_euclid(360.0)
            as u16;
    let program = TorpProgram {
        heading: Some(heading),
        delay: 5,
        pattern: Pattern::Circle,
        ceiling: 30,
        floor: 250,
    };
    sim.asw_command(0, AswCmd::Program(program)).unwrap();
    sim.asw_command(0, AswCmd::Arm(true)).unwrap();
    sim.asw_command(0, AswCmd::Release { station: 1 }).unwrap();
    for _ in 0..300 {
        run(&mut sim, &ps, 1.0);
        if !sub(&sim).alive {
            break;
        }
    }
    assert_eq!(sim.asw().result, AswResult::Won);
    assert!(sim.objectives().primary_done);
    // Ended from the menu.
    let mut quit = sim.clone();
    quit.quit();
    let r = quit.mission_report();
    assert_eq!(r.outcome, score::Outcome::Aborted);
    assert!(r.total >= 150, "{:?} {}", r.points, r.total);
    assert_ne!(r.rating, "Unsatisfactory");
    // Landed back on the carrier: the landing's points too.
    let home = m.takeoff.object_index as usize;
    let c = object_metres(&sim.objects()[home]);
    run(&mut sim, &at(c[0], c[1] - 3000.0, 100.0, 60.0), 2.0);
    let mut deck = at(c[0], c[1], 20.0, 0.0);
    deck.on_ground = true;
    deck.gear_up = false;
    run(&mut sim, &deck, 5.0);
    let landed = sim.mission_report();
    assert_eq!(landed.outcome, score::Outcome::Landed);
    assert!(landed.total > r.total);
}
