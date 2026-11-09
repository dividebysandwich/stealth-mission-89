//! The remake's own theaters (shipped JSON only): every situation generates missions with
//! targets in enemy territory and friendly takeoff bases.

use std::path::Path;

use stealth_mission::*;

fn data() -> GameData {
    GameData::load_json(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/mission"))
        .expect("assets/mission")
}

#[test]
fn ukraine_generates_missions() {
    let data = data();
    let theater = stealth_formats::ALL_THEATERS.iter().position(|t| *t == "ukraine").unwrap();
    let th = &data.theaters[theater];
    let mut kinds = std::collections::BTreeMap::<String, usize>::new();
    for kind in MissionKind::ALL {
        for roe in 0..3 {
            for seed in 0..40u16 {
                let s = MissionSettings {
                    theater,
                    roe,
                    kind,
                    opponents: (seed % 4) as u8,
                    ..Default::default()
                };
                let m = generate(&data, &s, Seed::Srand(seed * 977 + 13))
                    .unwrap_or_else(|e| panic!("{kind:?} roe {roe} seed {seed}: {e:#}"));
                assert_eq!(m.theater_name, "Ukraine");
                assert!(!m.takeoff.carrier, "no carriers in Ukraine");
                let base = &th.world.objects[m.takeoff.object_index as usize];
                assert!(
                    base.flags & world::flags::FRIENDLY != 0,
                    "takeoff from {} is not friendly",
                    m.takeoff.name
                );
                assert!(m.briefing.date.ends_with("2026"), "{}", m.briefing.date);
                if kind.is_strike() {
                    for t in [&m.primary, &m.secondary] {
                        let (col, row) = (t.position.x >> 11, t.position.y >> 11);
                        let g = th.world.grid[(row * 16 + col) as usize];
                        assert_eq!(
                            g & 3,
                            0,
                            "{} at {:?} is not in enemy territory",
                            t.shape_name,
                            t.position
                        );
                    }
                }
                *kinds.entry(m.primary.shape_name.clone()).or_default() += 1;
            }
        }
    }
    eprintln!("primary targets: {kinds:?}");
    assert!(kinds.len() >= 8, "too few kinds of targets: {kinds:?}");
}

#[test]
fn ukraine_drone_defense() {
    let data = data();
    let theater = stealth_formats::ALL_THEATERS.iter().position(|t| *t == "ukraine").unwrap();
    let mut nuclear = 0;
    let mut sizes = std::collections::BTreeSet::new();
    for opponents in 0..4u8 {
        for seed in 0..30u16 {
            let s = MissionSettings {
                theater,
                roe: (seed % 3) as u8,
                kind: MissionKind::DroneDefense,
                opponents,
                ..Default::default()
            };
            let m = generate(&data, &s, Seed::Srand(seed * 131 + 7))
                .unwrap_or_else(|e| panic!("seed {seed}: {e:#}"));
            let w = m.drones.as_ref().expect("a drone wave");
            assert!(w.defend && (4..=12).contains(&w.count) && w.allowance as usize <= w.count / 2);
            assert_eq!(w.first_unit + w.count, m.world.flights.len());
            assert_eq!(m.primary.mission_type, drones::DRONE_INTERCEPT);
            assert_eq!(m.primary.object_index, w.target);
            let target = m.world.objects[w.target as usize];
            assert!(
                target.flags & world::flags::FRIENDLY != 0,
                "{} is not friendly",
                w.target_name
            );
            // The secondary is a launch site, the base a friendly airfield.
            assert_eq!(m.secondary.mission_type, 2);
            assert!(data.theaters[theater].launch_sites.contains(&m.secondary.object_index));
            assert!(
                m.world.objects[m.takeoff.object_index as usize].flags & world::flags::FRIENDLY
                    != 0
            );
            assert!(m.special_aircraft.is_none());
            for f in &m.world.flights[w.first_unit..] {
                assert!(f.plane_type == drones::GERAN2 || f.plane_type == drones::GERAN3);
                assert_eq!(f.waypoint, w.target);
                assert_eq!(f.flags, 3);
            }
            assert!(m.orders.primary.starts_with("STOP GERAN DRONES"), "{}", m.orders.primary);
            nuclear += w.nuclear as usize;
            sizes.insert(w.count);
        }
    }
    assert!(nuclear > 0, "never a nuclear plant");
    assert!(sizes.len() >= 4, "wave sizes {sizes:?}");
    // The classic theaters have no launch sites.
    let s = MissionSettings { theater: 3, kind: MissionKind::DroneDefense, ..Default::default() };
    assert!(generate(&data, &s, Seed::Srand(1)).is_err());
    let s = MissionSettings { theater: 3, kind: MissionKind::Strike, ..Default::default() };
    assert!(generate(&data, &s, Seed::Srand(1)).unwrap().drones.is_none());
    // Ukraine strike missions: a few drones crossing.
    let s = MissionSettings { theater, kind: MissionKind::Strike, ..Default::default() };
    let w = generate(&data, &s, Seed::Srand(5)).unwrap().drones.unwrap();
    assert!(!w.defend && (1..=3).contains(&w.count));
}

#[test]
fn the_water_grid_agrees_with_the_point_test() {
    let data = data();
    for (i, th) in data.theaters.iter().enumerate() {
        let dim = 512;
        let grid = th.water_grid(dim);
        let step = 0x8000 / dim as i32;
        let (mut wet, mut checked) = (0, 0);
        for r in (0..dim).step_by(7) {
            for c in (0..dim).step_by(5) {
                let (x, y) = (c as i32 * step + step / 2, r as i32 * step + step / 2);
                let w = th.is_water(x, y);
                assert_eq!(grid[r * dim + c], w, "theater {i} at ({x}, {y})");
                wet += w as i32;
                checked += 1;
            }
        }
        assert!(i != 3 || wet > 0, "no water in Central Europe of {checked}");
    }
}

/// The cargo jet's missions in every theater: off a land base (never a carrier), an airdrop
/// to friendly troops, then the delivery to a friendly airfield on land where it also lands,
/// the pallets in the hold.
#[test]
fn cargo_jet_missions_drop_and_deliver() {
    let data = data();
    let f = world::flags::FRIENDLY;
    let mut tested = 0;
    for theater in 0..stealth_formats::ALL_THEATERS.len() {
        if data.theaters.get(theater).is_none_or(|t| t.world.objects.is_empty()) {
            continue;
        }
        for (seed, kind) in (0..6u16).zip([MissionKind::Strike, MissionKind::Air].into_iter().cycle()) {
            let s = MissionSettings { theater, kind, land_bases: true, ..Default::default() };
            let Ok(mut m) = generate(&data, &s, Seed::Srand(seed * 3001 + 7)) else { continue };
            assert!(!m.takeoff.carrier, "theater {theater} seed {seed}: carrier takeoff");
            airlift::for_transport(&mut m, &data);
            let w = &m.world;
            let (p, s2) = (&m.primary, &m.secondary);
            let name = &m.theater_name;
            assert_eq!(p.mission_type, airlift::AIRDROP, "{name} {seed}");
            assert_eq!(s2.mission_type, airlift::DELIVERY, "{name} {seed}");
            let zone = w.objects[p.object_index as usize];
            let dest = w.objects[s2.object_index as usize];
            assert!(zone.flags & f != 0 && zone.flags & world::flags::AIRFIELD == 0, "{name}");
            assert!(dest.flags & f != 0 && dest.flags & world::flags::AIRFIELD != 0, "{name}");
            assert!(dest.flags & world::flags::CARRIER == 0, "{name}: delivery to a carrier");
            assert_eq!(m.landing.object_index, s2.object_index);
            assert_eq!(m.loadout.usable_bays, 1);
            assert_eq!(m.loadout.bays[0].weapon, airlift::PALLETS);
            tested += 1;
            // Done once.
            let again = m.clone();
            airlift::for_transport(&mut m, &data);
            assert_eq!(m, again);
        }
    }
    println!("{tested} cargo jet missions");
    assert!(tested >= 20, "only {tested} missions");
}

/// About half the cargo jet's missions are routine, behind the lines (no escort, the enemy's
/// fighters grounded), their routes far clearer of the enemy than the front-line drops, which
/// get two escorts.
#[test]
fn cargo_jet_missions_are_quiet_or_escorted() {
    let data = data();
    let (mut quiet, mut front) = (Vec::new(), Vec::new());
    for theater in 0..stealth_formats::ALL_THEATERS.len() {
        if data.theaters.get(theater).is_none_or(|t| t.world.objects.is_empty()) {
            continue;
        }
        for seed in 0..12u16 {
            let kind = if seed % 2 == 0 { MissionKind::Strike } else { MissionKind::Air };
            let s = MissionSettings { theater, kind, land_bases: true, ..Default::default() };
            let Ok(mut m) = generate(&data, &s, Seed::Srand(seed * 3001 + 7)) else { continue };
            airlift::for_transport(&mut m, &data);
            let t = airlift::threats(&m);
            let p = |b: &stealth_mission::MapPoint| (b.x, b.y);
            let km = airlift::route_clearance(
                &t,
                p(&m.takeoff.position),
                p(&m.primary.position),
                p(&m.secondary.position),
            ) / 64.0;
            if m.quiet {
                assert_eq!(m.escorts, 0);
                quiet.push(km);
            } else {
                assert_eq!(m.escorts, airlift::ESCORTS);
                front.push(km);
            }
        }
    }
    let median = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let n = (quiet.len() + front.len()) as f64;
    let (q, f) = (median(&mut quiet), median(&mut front));
    println!(
        "{} quiet (median clearance {q:.0} km, least {:.0}), {} front ({f:.0} km)",
        quiet.len(),
        quiet[0],
        front.len()
    );
    assert!((0.3..0.7).contains(&(quiet.len() as f64 / n)), "quiet share");
    assert!(q > f + 10.0, "quiet routes {q:.0} km, front {f:.0} km");
}
