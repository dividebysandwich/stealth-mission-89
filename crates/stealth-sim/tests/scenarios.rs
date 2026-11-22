//! Scenario tests on the shipped game data (`assets/sim`, `assets/mission`).

use std::sync::OnceLock;

use stealth_mission::{GameData, Mission, MissionKind, MissionSettings, Seed, Weather};
use stealth_sim::*;

/// A HUD message in English.
fn en(t: &stealth_i18n::Text) -> String {
    t.tr_in(stealth_i18n::Language::En)
}

/// Whether `e` is the HUD message `text` (in English).
fn is_msg(e: &SimEvent, text: &str) -> bool {
    matches!(e, SimEvent::Message(t) if en(t) == text)
}

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

fn mission(d: &Data, theater: usize, seed: u16, opponents: u8) -> Mission {
    let s = MissionSettings {
        theater,
        roe: 2,
        opponents,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    stealth_mission::generate(&d.gdata, &s, Seed::Srand(seed)).unwrap()
}

/// A clean sandbox: no radars, no aircraft, no friendly bases; returns a map cell centre with
/// flat / open terrain.
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

fn player_at(mx: i32, my: i32, alt: u16, heading: u16, knots: u16) -> PlayerState {
    let (x, y) = Sim::map_to_world(mx, my);
    PlayerState {
        x,
        y,
        altitude: alt,
        height: alt,
        heading,
        velocity: knots * 27,
        gear_up: true,
        fuel: 10000,
        hud_visible: true,
        ..Default::default()
    }
}

fn place_site(sim: &mut Sim, i: usize, ty: i32, x: i32, y: i32) {
    let o = &mut sim.objects_mut()[i];
    o.unit_type = ty;
    o.x = x;
    o.y = y;
    o.flags = 0;
    o.alert = 0;
    o.timer = 0;
}

/// The units file builds the tables the simulation indexes: radars, aircraft, missiles and
/// stores where the theaters and the code expect them, game values from the real figures.
#[test]
fn sim_data_tables() {
    let d = data();
    let s = &d.sim;
    assert_eq!(s.threats[1].name, "SA-2");
    // SA-10: a 300 km search radar of 1978 with Doppler.
    assert_eq!((s.threats[4].lethality, s.threats[4].tier, s.threats[4].flags), (270, 5, 1));
    assert_eq!(s.threats[21].lethality, 500, "OTH radar at the cap");
    assert_eq!(s.aircraft[4].name, "Su-27");
    // MiG-29: Mach 2.25, the N019 radar.
    assert_eq!(
        (s.aircraft[2].max_speed, s.aircraft[2].agility, s.aircraft[2].radar, s.aircraft[2].model),
        (730, 5, 19, 16)
    );
    // SA-2: 45 km, Mach 3.5, 6 g, command guidance.
    assert_eq!(
        (
            s.missiles[1].lock_range,
            s.missiles[1].max_speed,
            s.missiles[1].class,
            s.missiles[1].turn
        ),
        (124, 2000, 1, 3)
    );
    assert_eq!(s.missiles[38].name, "Equipment");
    assert_eq!(s.weapons[6].long_name, "Paveway");
    assert_eq!(s.weapons[16].spec, -1);
    assert_eq!(s.compat[8][1], 8, "Durandal against runways");
    assert!(s.compat[0].iter().all(|&v| v == 0), "Sidewinders against nothing on the ground");
    assert_eq!(s.trig.lut[64], 32767);
    assert_eq!(s.maneuver.len(), 192);
    assert_eq!(s.stores_init, [0, 15, 30, 2]);
    assert_eq!(s.theaters.len(), 9);
}

#[test]
fn site_detects_and_launches() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    // SA-2 site 10 km north of the player; slow, high, bays open: high EMV.
    place_site(&mut sim, 3, 1, mx, my - 640);
    let mut p = player_at(mx, my, 6000, 0x4000, 200);
    p.cmd.toggle_bay = true;
    let mut launched = None;
    let mut detected_msg = false;
    for f in 0..15 * 120 {
        let ev = sim.step(&p, 15);
        p.cmd = Commands::default();
        for e in &ev {
            match e {
                SimEvent::MissileLaunched { slot, spec, launcher: Launcher::Site(3) } => {
                    launched = Some((f, *slot, *spec))
                }
                SimEvent::Message(s) if en(s).ends_with(" detected") => detected_msg = true,
                _ => {}
            }
        }
        if launched.is_some() {
            break;
        }
    }
    assert!(sim.emv() > 40, "emv {}", sim.emv());
    assert!(detected_msg);
    let (_, slot, spec) = launched.expect("SA-2 launch");
    assert_eq!(spec, 1);
    assert_eq!(slot, 3);
    assert!(sim.site_alert(3) > 0xC0);
    assert!(sim.objects()[3].flags & 0x1000 != 0, "Radar ID before the launch");
    // The missile flies and eventually hits the (stationary) player.
    let mut hit = false;
    for _ in 0..15 * 120 {
        for e in sim.step(&p, 15) {
            if matches!(e, SimEvent::PlayerHit { .. }) {
                hit = true;
            }
        }
        if hit {
            break;
        }
    }
    assert!(hit, "missile {:?}", sim.projectiles()[3]);
    assert!(sim.damage_effects().hits > 2);
}

/// The radar warning receiver's list: sites in their detection range, hostile and alive,
/// ranked with a launching site first.
#[test]
fn emitters_ranked() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    // SA-2 10 km north, SA-6 20 km east, a destroyed and a friendly one near, one out of range.
    place_site(&mut sim, 3, 1, mx, my - 640);
    place_site(&mut sim, 4, 6, mx + 1280, my);
    place_site(&mut sim, 5, 1, mx - 640, my);
    sim.objects_mut()[5].flags |= 0x0080;
    place_site(&mut sim, 6, 1, mx, my + 640);
    sim.objects_mut()[6].flags |= 0x0400;
    place_site(&mut sim, 7, 1, mx, my + 64 * 250);
    let mut p = player_at(mx, my, 6000, 0x4000, 200);
    sim.step(&p, 15);
    let e = sim.emitters();
    let sites: Vec<EmitterSource> = e.iter().map(|e| e.source).collect();
    assert_eq!(sites.len(), 2, "{e:?}");
    assert!(sites.contains(&EmitterSource::Site(3)) && sites.contains(&EmitterSource::Site(4)));
    let sa2 = e.iter().find(|e| e.source == EmitterSource::Site(3)).unwrap();
    assert_eq!((sa2.range, sa2.radar_type), (10, 1));
    assert!(sa2.bearing < 0x100 || sa2.bearing > 0xFF00, "north: {:#x}", sa2.bearing);
    assert_eq!(sim.threat_name(1), "SA-2");
    // Detected, tracked and fired at: the SA-2 comes first.
    p.cmd.toggle_bay = true;
    for _ in 0..15 * 120 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
        if sim.projectiles()[3].ttl > 0 {
            break;
        }
    }
    let e = sim.emitters();
    assert_eq!(e[0].source, EmitterSource::Site(3));
    assert!(e[0].launch && e[0].tracking, "{:?}", e[0]);
}

/// A display's cursor lock (HSD, HARM) holds without the camera page, and survives the HUD
/// mode change that comes with it.
#[test]
fn display_lock_holds() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    place_site(&mut sim, 3, 1, mx, my - 640);
    let mut p = player_at(mx, my, 6000, 0, 300);
    p.camera_mfd = false;
    p.hud_mode = HudMode::Gnd;
    p.cmd.lock_object = Some(3);
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..60 {
        sim.step(&p, 15);
    }
    assert_eq!(sim.targeting().ground, Some(3));
    // A target key releases it to the normal cycling.
    p.cmd.clear_target = true;
    sim.step(&p, 15);
    assert_eq!(sim.targeting().ground, None);
}

/// External stores (F-16): the bay is open from the start and stays open, which the EMV shows.
#[test]
fn external_stores_always_open() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { external_stores: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    let mut p = player_at(mx, my, 3000, 0, 300);
    sim.step(&p, 15);
    assert!(sim.bay_doors_open());
    let open_emv = sim.emv();
    p.cmd.toggle_bay = true;
    for _ in 0..30 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    assert!(sim.bay_doors_open());
    let mut closed = Sim::new(&d.sim, &m);
    sandbox(&mut closed, &m);
    closed.step(&player_at(mx, my, 3000, 0, 300), 15);
    assert!(!closed.bay_doors_open());
    assert_eq!(open_emv, closed.emv() + 24);
}

/// A transmitting air-to-ground radar shows in the EMV while airborne, and stops with it.
#[test]
fn radar_raises_the_emv() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    let mut p = player_at(mx, my, 3000, 0, 300);
    sim.step(&p, 15);
    let quiet = sim.emv();
    p.radar = true;
    sim.step(&p, 15);
    assert_eq!(sim.emv(), quiet + stealth_sim::RADAR_EMV);
    p.radar = false;
    sim.step(&p, 15);
    assert_eq!(sim.emv(), quiet);
}

/// The EMV's altitude term counts the height above the ground: 500 ft over 2500 ft hills
/// shows as much as 500 ft over the plain.
#[test]
fn emv_counts_height_above_ground() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let emv = |alt: u16, height: u16| {
        let mut sim = Sim::new(&d.sim, &m);
        let (mx, my) = sandbox(&mut sim, &m);
        sim.step(&PlayerState { height, ..player_at(mx, my, alt, 0, 300) }, 15);
        sim.emv()
    };
    assert_eq!(emv(3000, 500), emv(500, 500));
    assert!(emv(3000, 3000) > emv(3000, 500));
}

/// Low, fast-ish, bays closed over flat ground: the same SA-2 never detects.
#[test]
fn stealthy_player_not_detected() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    place_site(&mut sim, 3, 1, mx, my - 64 * 60);
    let p = player_at(mx, my, 200, 0, 300);
    for _ in 0..15 * 60 {
        for e in sim.step(&p, 15) {
            assert!(!matches!(e, SimEvent::MissileLaunched { .. }));
        }
    }
    assert_eq!(sim.site_alert(3), 0);
}

/// Whether a site of type `ty` `km` north of a pilot crossing in front of it at `alt` ft and
/// `knots` (its doors opened or not) raises its alert within 20 s.
fn site_sees(opts: &SimOptions, ty: i32, km: f32, alt: u16, knots: u16, doors: bool) -> bool {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::with_options(&d.sim, &m, opts.clone());
    let (mx, my) = sandbox(&mut sim, &m);
    place_site(&mut sim, 3, ty, mx, my - (64.0 * km) as i32);
    let mut p = player_at(mx, my, alt, 0x4000, knots);
    p.cmd.toggle_bay = doors;
    for _ in 0..15 * 20 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    sim.site_alert(3) > 0
}

/// The RAH-66 is all but invisible to radars slow or hovering, and seen about as an F-117A is
/// near its top speed: an SA-2 sees neither hovering at 6000 ft 12 km away, both at 180 kt;
/// with the doors open an SA-13 sees the F-117A low down at any speed, the RAH-66 only fast.
#[test]
fn comanche_hides_when_slow() {
    let comanche =
        SimOptions { helicopter: true, gunship: true, comanche: true, ..Default::default() };
    let f117 = SimOptions::default();
    assert!(site_sees(&f117, 1, 12.0, 6000, 0, false));
    assert!(!site_sees(&comanche, 1, 12.0, 6000, 0, false));
    assert!(!site_sees(&comanche, 1, 12.0, 6000, 100, false));
    assert!(site_sees(&comanche, 1, 12.0, 6000, 180, false));
    for knots in [0, 60] {
        assert!(site_sees(&f117, 7, 5.0, 300, knots, true));
        assert!(!site_sees(&comanche, 7, 5.0, 300, knots, true), "{knots} kt");
        assert!(!site_sees(&comanche, 7, 1.0, 300, knots, true), "{knots} kt, 1 km");
    }
    assert!(site_sees(&comanche, 7, 5.0, 300, 180, true));
}

/// Run until a site missile is in the air and locked.
fn engaged(d: &Data, ty: i32) -> (Sim, PlayerState) {
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    // Within the site missile's reach (12 km, nearer for the short-range ones).
    let km = (d.sim.missiles[ty as usize].lock_range as i32 * 2 / 3).clamp(3, 12);
    place_site(&mut sim, 3, ty, mx, my - 64 * km);
    let mut p = player_at(mx, my, 6000, 0x4000, 200);
    p.cmd.toggle_bay = true;
    for _ in 0..15 * 300 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let m = sim.projectiles()[3];
        if m.ttl > 0 && m.locked {
            return (sim, p);
        }
    }
    panic!("no locked missile of type {ty}");
}

#[test]
fn decoy_soaks_the_missile() {
    let d = data();
    let (mut sim, mut p) = engaged(d, 1);
    sim.set_training(false);
    p.cmd.decoy = true;
    let mut hit_msg = false;
    for _ in 0..15 * 50 {
        for e in sim.step(&p, 15) {
            assert!(!matches!(e, SimEvent::PlayerHit { .. }), "decoy must soak the hit");
            if matches!(&e, SimEvent::Message(s) if en(s).starts_with("Hit by")) {
                hit_msg = true;
            }
        }
        p.cmd = Commands::default();
        assert!(!sim.lights().rad, "RAD light stays dark while the decoy runs");
    }
    assert!(!hit_msg);
    assert_eq!(sim.stores().2, d.sim.stores_init[3] as i32 - 1, "one decoy used");
}

#[test]
fn ecm_jams_beam_rider() {
    let d = data();
    // SA-2 is class 1 (beam rider): ECM jams it at any range.
    let (mut sim, mut p) = engaged(d, 1);
    p.cmd.toggle_ecm = true;
    sim.step(&p, 15);
    assert!(sim.ecm());
    assert!(!sim.projectiles()[3].locked || sim.projectiles()[3].ttl == 0);
}

#[test]
fn chaff_breaks_semi_active_lock() {
    let d = data();
    // SA-8B: class 2 semi-active, fooled by chaff in its seeker cone.
    let (mut sim, mut p) = engaged(d, 3);
    p.cmd.chaff = true;
    let ev = sim.step(&p, 15);
    assert!(ev.iter().any(|e| is_msg(e, "Chaff released")));
    assert!(!sim.projectiles()[3].locked);
    assert!(sim.lights().chaff);
}

#[test]
fn ir_jammer_vs_first_generation_ir() {
    let d = data();
    // SA-N-5 is class -1 (IR 1st generation): the IR jammer defeats it at any range.
    let (mut sim, mut p) = engaged(d, 9);
    p.cmd.toggle_irj = true;
    sim.step(&p, 15);
    assert!(sim.ir_jammer());
    assert!(!sim.projectiles()[3].locked || sim.projectiles()[3].ttl == 0);
    // Auto-off after 20 s.
    p.cmd = Commands::default();
    for _ in 0..15 * 21 {
        sim.step(&p, 15);
    }
    assert!(!sim.ir_jammer());
}

#[test]
fn aim9_kills_fighter() {
    let d = data();
    assert!(aim9_shot(d, Weather::default()));
}

/// An overcast deck.
fn deck(base_ft: i32, top_ft: i32) -> Weather {
    Weather::stratus_layer(base_ft, top_ft, 100, 1)
}

#[test]
fn clouds_blind_the_sidewinder() {
    let d = data();
    // Shooter and target inside the deck: no IR lock. A deck below both does not matter.
    assert!(!aim9_shot(d, deck(4000, 6000)));
    assert!(aim9_shot(d, deck(1000, 3000)));
}

/// Whether a radar site's missile locks on the player (6000 ft) within five minutes.
fn site_missile_locks(d: &Data, ty: i32, weather: Weather) -> bool {
    let mut m = mission(d, 3, 0x1234, 1);
    m.weather = weather;
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    // Within the site missile's reach (12 km, nearer for the short-range ones).
    let km = (d.sim.missiles[ty as usize].lock_range as i32 * 2 / 3).clamp(3, 12);
    place_site(&mut sim, 3, ty, mx, my - 64 * km);
    let mut p = player_at(mx, my, 6000, 0x4000, 200);
    p.cmd.toggle_bay = true;
    for _ in 0..15 * 300 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
        if sim.projectiles().iter().any(|m| m.ttl > 0 && m.locked) {
            return true;
        }
    }
    false
}

#[test]
fn clouds_block_ir_but_not_radar_seekers() {
    let d = data();
    // SA-N-5 (IR) from the ground at a player above a 2000-4000 ft deck; SA-8B (radar) does lock.
    assert!(site_missile_locks(d, 9, Weather::default()));
    assert!(!site_missile_locks(d, 9, deck(2000, 4000)));
    assert!(site_missile_locks(d, 3, deck(2000, 4000)));
}

/// Fires an AIM-9 at a slow transport 3 km ahead (both at 5000 ft); true if it is shot down.
fn aim9_shot(d: &Data, weather: Weather) -> bool {
    let mut m = mission(d, 3, 0x1234, 1);
    m.weather = weather;
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    // A slow An-72 (no radar) 3 km ahead, flying north like the player.
    {
        let u = &mut sim.flights_mut()[1];
        *u = FlightUnit {
            x: mx,
            y: my - 64 * 3,
            alt: 5000,
            spec: 9,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: 3,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(mx, my, 5000, 0, 300);
    p.cmd.toggle_bay = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..5 {
        sim.step(&p, 15);
    }
    assert!(sim.bay_doors_open());
    p.cmd.fire = true;
    let ev = sim.step(&p, 15);
    assert!(ev.iter().any(|e| is_msg(e, "Sidewinder fired")), "{ev:?}");
    p.cmd = Commands::default();
    let mut killed = false;
    for _ in 0..15 * 20 {
        for e in sim.step(&p, 15) {
            if matches!(e, SimEvent::AircraftDestroyed { unit: 1, .. }) {
                killed = true;
            }
        }
    }
    if killed {
        assert!(sim.flights()[1].flags & 0x20 != 0);
        assert!(sim.event_log().iter().any(|e| e.kind == 3 && e.arg == 9));
        // The An-72 has no radar and never saw the player.
        let r = sim.mission_report();
        assert_eq!((r.air_kills, r.unseen_kills, r.weapons_fired), (1, 1, 1));
        assert!(
            r.timeline
                .iter()
                .any(|t| t.text.tr_in(stealth_i18n::Language::En) == "Shot down An-72"),
            "{:?}",
            r.timeline
        );
    }
    killed
}

/// The Crimean bridge's arches (Ukraine models 114 road, 115 rail) are strike targets: the
/// generator picks them as bridges, and a bomb on one wrecks that arch alone, drawn in its own
/// wrecked state.
#[test]
fn kerch_bridge_arches_are_strike_targets() {
    let d = data();
    let arches = [114usize, 115];
    let mut found = Vec::new();
    for s in 1..3000u16 {
        let m = mission(d, 9, s, 0);
        let t = m.primary.object_index as usize;
        let shape = m.world.objects[t].shape();
        if m.primary.mission_type == 2
            && arches.contains(&shape)
            && !found.iter().any(|(f, _): &(Mission, u16)| {
                f.world.objects[f.primary.object_index as usize].shape() == shape
            })
        {
            found.push((m, s));
            if found.len() == arches.len() {
                break;
            }
        }
    }
    assert_eq!(found.len(), arches.len(), "no strikes on both of the Kerch bridge's arches");
    // The remake theaters' tiles are not in the sim tables (the host adds them).
    let ron = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/theaters/ukraine.ron"),
    );
    let tiles = TheaterTiles::read_ron(&ron.expect("ukraine.ron")).expect("ukraine tiles");
    let sd = d.sim.with_theater(9, || tiles);
    for (m, seed) in found {
        let t = m.primary.object_index as usize;
        let shape = m.world.objects[t].shape() as i32;
        let cat = (m.world.shape_class[shape as usize] & 0xF) as i32;
        let w = [7, 8, 10, 14, 15]
            .into_iter()
            .find(|&w| d.sim.compat_cat(w, cat) > 0)
            .expect("a weapon for bridges");
        let mut sim = Sim::new(&sd, &m);
        sim.set_training(true);
        let o = sim.objects()[t];
        sim.set_bay(0, w, 2);
        sim.select_bay(0);
        let mut p = player_at(o.x, o.y, 500, 0, 0);
        p.cmd.toggle_bay = true;
        for _ in 0..4 {
            sim.step(&p, 15);
            p.cmd = Commands::default();
        }
        p.cmd.fire = true;
        let mut killed = false;
        let mut wrecks = Vec::new();
        for _ in 0..15 * 10 {
            for e in sim.step(&p, 15) {
                match e {
                    SimEvent::GroundDestroyed { object, .. } if object == t => killed = true,
                    SimEvent::TileDestroyed(rec) => wrecks.push(rec.shape),
                    _ => {}
                }
            }
            p.cmd = Commands::default();
        }
        assert!(killed && sim.objectives().primary_done, "seed {seed}: arch {shape} not destroyed");
        assert_eq!(wrecks, [shape | 0x100], "seed {seed}: only the arch, in its own wreck state");
    }
}

/// Retarded bomb dropped straight onto the primary: kill iff compat << 6 / (diff + 2) > distance.
#[test]
fn bomb_vs_target_uses_effectiveness_table() {
    let d = data();
    let (m, seed) = (1..400u16)
        .map(|s| (mission(d, 3, s, 0), s))
        .find(|(m, _)| m.primary.mission_type == 2)
        .unwrap();
    let t = m.primary.object_index as usize;
    let cat = (m.world.shape_class[m.world.objects[t].shape()] & 0xF) as i32;
    // Retarded weapons (spec 29): Rockeye 7, Durandal 8, Snakeye 10, Mk 35 incendiary 14, mines 15.
    let good = [7, 8, 10, 14, 15].into_iter().find(|&w| d.sim.compat_cat(w, cat) > 0);
    let bad = [7, 8, 10, 14, 15].into_iter().find(|&w| d.sim.compat_cat(w, cat) == 0);
    for (w, expect_kill) in [(good, true), (bad, false)] {
        let Some(w) = w else { continue };
        let mut sim = Sim::new(&d.sim, &m);
        sim.set_training(true);
        let o = sim.objects()[t];
        sim.set_bay(0, w, 2);
        sim.select_bay(0);
        let mut p = player_at(o.x, o.y, 500, 0, 0);
        p.cmd.toggle_bay = true;
        for _ in 0..4 {
            sim.step(&p, 15);
            p.cmd = Commands::default();
        }
        p.cmd.fire = true;
        let mut killed = false;
        let mut msgs = Vec::new();
        for _ in 0..15 * 10 {
            for e in sim.step(&p, 15) {
                match e {
                    SimEvent::GroundDestroyed { object, .. } if object == t => killed = true,
                    SimEvent::Message(s) => msgs.push(en(&s)),
                    _ => {}
                }
            }
            p.cmd = Commands::default();
        }
        assert_eq!(killed, expect_kill, "seed {seed} weapon {w} cat {cat}: {msgs:?}");
        if expect_kill {
            assert!(sim.objectives().primary_done);
            assert_eq!(sim.objectives().waypoint, 2);
            assert!(
                sim.event_log()
                    .iter()
                    .any(|e| e.kind & 0x80 != 0 && matches!(e.kind & 0x3F, 1 | 12))
            );
            let r = sim.mission_report();
            assert!(r.primary_done && r.training && r.ground_kills >= 1);
            assert!(
                r.timeline
                    .iter()
                    .any(|t| t.text.tr_in(stealth_i18n::Language::En).ends_with("(primary)")),
                "{:?}",
                r.timeline
            );
            // Campaign damage: the next mission starts with the target destroyed.
            let damage = sim.world_damage();
            assert!(damage.objects.contains(&(t as u16)) || t < 3);
            assert!(damage.near(o.x, o.y, 0x10) && !damage.tiles.is_empty());
            let json = serde_json::to_string(&damage).unwrap();
            let back: WorldDamage = serde_json::from_str(&json).unwrap();
            assert_eq!(back, damage);
            let next = Sim::with_options(
                &d.sim,
                &m,
                SimOptions { prior_damage: Some(back), ..Default::default() },
            );
            if t >= 3 {
                assert!(next.objects()[t].flags & oflags::DESTROYED != 0);
            }
            assert_eq!(next.world_damage(), damage);
        } else {
            assert!(msgs.iter().any(|s| s.ends_with(" misses ")), "{msgs:?}");
        }
    }
}

#[test]
fn bay_closed_and_altitude_gates() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    sim.set_bay(0, 9, 3);
    sim.select_bay(0);
    let mut p = player_at(0x4000, 0x4000, 3000, 0, 300);
    p.cmd.fire = true;
    let ev = sim.step(&p, 15);
    assert!(ev.iter().any(|e| is_msg(e, "Bay closed")));
    assert_eq!(sim.bays().0[0].1, 3);
}

#[test]
fn countermeasure_stores_run_out() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let mut p = player_at(0x4000, 0x4000, 3000, 0, 300);
    let mut exhausted = false;
    for _ in 0..20 {
        p.cmd.flare = true;
        for e in sim.step(&p, 15) {
            if is_msg(&e, "Stores exhausted") {
                exhausted = true;
            }
        }
    }
    assert!(exhausted);
    assert_eq!(sim.stores().0, 0);
}

/// Ten simulated minutes of one theater with a scripted flight and random commands.
fn ten_minute_mission(d: &Data, th: usize, k: u16, opp: u8, opts: SimOptions) -> Sim {
    ten_minute_mission_seen(d, th, k, opp, opts, &mut |_| {})
}

/// [`ten_minute_mission`], showing `seen` the events.
fn ten_minute_mission_seen(
    d: &Data,
    th: usize,
    k: u16,
    opp: u8,
    opts: SimOptions,
    seen: &mut dyn FnMut(&SimEvent),
) -> Sim {
    let trig = &d.sim.trig;
    let m = mission(d, th, 0x100 + th as u16 * 7 + k, opp);
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let mut p = sim.start_state();
    let mut rng = Rng::new(th as u32 * 77 + k as u32);
    let (mut wx, mut wy) = (p.x as i64, p.y as i64);
    let mut alt = 0i32;
    let mut ended = false;
    for f in 0..15 * 600 {
        // Climb out and fly towards the primary, then the secondary.
        let wp = m.waypoints[if f < 15 * 300 { 1 } else { 2 }];
        let (tx, ty) = Sim::map_to_world(wp.x, wp.y);
        let h = stealth_sim::math::bearing(
            ((tx as i64 - wx) >> 6) as i32,
            ((ty as i64 - wy) >> 6) as i32,
        );
        p.heading = h as u16;
        alt = (alt + 20).min(if f < 15 * 400 { 3000 } else { 300 });
        p.altitude = alt as u16;
        p.height = p.altitude;
        p.velocity = 450 * 27;
        p.on_ground = false;
        p.gear_up = true;
        p.roll = ((f % 300) as i16 - 150) * 40;
        let v = (p.velocity as i32 * 4) / 15 / 64;
        wx += trig.sin_mul(h as i32, v) as i64 * 4;
        wy += trig.cos_mul(h as i32, v) as i64 * 4;
        p.x = wx as i32;
        p.y = wy as i32;
        p.hud_mode = [HudMode::Nav, HudMode::Air, HudMode::Gnd][(f / 900) % 3];
        p.camera_mfd = f % 1000 < 600;
        let r = rng.random(1000);
        p.cmd = Commands {
            fire: r < 20,
            gun: r % 7 == 0,
            flare: r == 100,
            chaff: r == 101,
            decoy: r == 102,
            toggle_ecm: r == 103,
            toggle_irj: r == 104,
            toggle_bay: r == 105,
            next_bay: r == 106,
            designate: r == 107,
            next_target: r == 108,
            camera_dir: (r == 109).then_some(0x4000),
            nearest_target: r == 110,
            ..Default::default()
        };
        for e in sim.step(&p, 15) {
            seen(&e);
            if matches!(e, SimEvent::MissionEnd(_)) {
                ended = true;
            }
        }
        let _ = sim.targeting();
        let _ = sim.active_aircraft();
        if ended {
            break;
        }
    }
    sim
}

/// Ten simulated minutes of every theater with a scripted flight and random commands.
#[test]
fn ten_minute_missions_all_theaters() {
    let d = data();
    for th in 0..9 {
        for (k, opp) in [(0u16, 0u8), (1, 3)] {
            let sim = ten_minute_mission(d, th, k, opp, SimOptions::default());
            eprintln!(
                "theater {th}: {} s, end {:?}, launches {}, log {}",
                sim.mission_tick(),
                sim.mission_end(),
                sim.launches(),
                sim.event_log().len()
            );
            assert!(sim.mission_tick() > 0);
        }
    }
}

/// FNV-1a over the serialized state.
fn state_hash(sim: &Sim) -> u64 {
    serde_json::to_vec(sim)
        .unwrap()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

/// Classic behaviour is pinned: the end state of the ten-minute missions with default options
/// must not change when optional features are added.
#[test]
fn classic_state_golden() {
    const GOLDEN: &[u64] = &[
        0x966bd5b324c31237,
        0xe2abc43420a23196,
        0x47eba8e4a704cc8d,
        0xc4655baa9a23f4c5,
        0x937e6995db7813a0,
        0x481b9dc4f5e0dbc6,
        0x4affc0f9cbd535b1,
        0x2fa021b510014070,
        0xf70b712762d7b4a8,
        0x9f98f5a00d3296a1,
        0x598e973229491041,
        0x1bf7a41d5d137a72,
        0x2d41c046207c3beb,
        0x06af589d2cf86fef,
        0xd6332e1aa884fe35,
        0x2a666b1c7630bb36,
        0xdbe4b5cab9b22d0a,
        0x4939630621058d81,
    ];
    let d = data();
    let mut got = Vec::new();
    for th in 0..9 {
        for (k, opp) in [(0u16, 0u8), (1, 3)] {
            got.push(state_hash(&ten_minute_mission(d, th, k, opp, SimOptions::default())));
        }
    }
    let hex: Vec<String> = got.iter().map(|h| format!("0x{h:016x}")).collect();
    assert_eq!(got, GOLDEN, "new hashes: [{}]", hex.join(", "));
}

/// A saved game (the game's single mission saves): the simulation read back from its JSON goes
/// on exactly as the one that was saved, with the realistic options on.
#[test]
fn saved_state_goes_on_the_same() {
    let d = data();
    let opts = SimOptions {
        realistic_damage: true,
        realistic_guidance: true,
        air_defence: true,
        ..Default::default()
    };
    let ai_flight = SimOptions { realistic_ai_flight: true, ..opts.clone() };
    for (th, opts) in [(3, &opts), (8, &opts), (3, &ai_flight), (8, &ai_flight)] {
        let m = mission(d, th, 0x2345 + th as u16, 3);
        let mut sim = Sim::with_options(&d.sim, &m, opts.clone());
        let mut p = sim.start_state();
        let (tx, ty) = Sim::map_to_world(m.waypoints[1].x, m.waypoints[1].y);
        let input = |f: i32, p: &mut PlayerState| {
            let h = stealth_sim::math::bearing((tx - p.x) >> 6, (ty - p.y) >> 6);
            p.heading = h as u16;
            p.altitude = (f * 20).min(2000) as u16;
            p.height = p.altitude;
            p.velocity = 450 * 27;
            p.on_ground = false;
            p.gear_up = true;
            let v = (p.velocity as i32 * 4) / 15 / 64;
            p.x += d.sim.trig.sin_mul(h as i32, v) * 4;
            p.y += d.sim.trig.cos_mul(h as i32, v) * 4;
            p.cmd = Commands { fire: f % 200 == 0, flare: f % 333 == 0, ..Default::default() };
        };
        for f in 0..15 * 240 {
            input(f, &mut p);
            sim.step(&p, 15);
        }
        let json = serde_json::to_string(&sim).unwrap();
        eprintln!("theater {th}: saved state {} kB", json.len() / 1024);
        let mut loaded: Sim = serde_json::from_str(&json).unwrap();
        loaded.keep_relief_of(&sim);
        assert_eq!(state_hash(&loaded), state_hash(&sim));
        for f in 15 * 240..15 * 360 {
            input(f, &mut p);
            sim.step(&p, 15);
            loaded.step(&p, 15);
        }
        assert_eq!(state_hash(&loaded), state_hash(&sim), "theater {th}");
    }
}

/// Supply drop: unreachable in classic mode (spec vs class test), fixed behind `fix_classic_bugs`.
#[test]
fn supply_drop_fixed_and_classic() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    for fix in [true, false] {
        let mut sim = Sim::with_options(
            &d.sim,
            &m,
            SimOptions { fix_classic_bugs: fix, ..Default::default() },
        );
        sim.set_training(true);
        let o = sim.objects()[5];
        sim.set_deadline(1000);
        sim.target_slots_mut()[1] = TargetSlot { mission_type: 3, object: 5, base: 0, code: 0x12 };
        sim.set_bay(0, 18, 1);
        sim.select_bay(0);
        let mut p = player_at(o.x, o.y + 10, 1000, 0, 150);
        p.cmd.toggle_bay = true;
        for _ in 0..4 {
            sim.step(&p, 15);
            p.cmd = Commands::default();
        }
        p.cmd.fire = true;
        let mut delivered = false;
        for _ in 0..15 * 30 {
            for e in sim.step(&p, 15) {
                if is_msg(&e, "Equipment Delivered") {
                    delivered = true;
                }
            }
            p.cmd = Commands::default();
        }
        assert_eq!(delivered, fix, "fix {fix}");
        assert_eq!(sim.objectives().secondary_done, delivered);
    }
}

#[test]
fn nearest_target_skips_friendlies() {
    let d = data();
    let m = mission(d, 3, 0x1234, 0);
    let mut sim = Sim::new(&d.sim, &m);
    let (cx, cy) = sandbox(&mut sim, &m);
    for o in sim.objects_mut().iter_mut() {
        o.flags |= 0x80;
    }
    // Ahead (north): a friendly object at 300, a hostile one at 600 map units.
    place_site(&mut sim, 1, 0, cx, cy - 300);
    sim.objects_mut()[1].flags = 0x0400;
    place_site(&mut sim, 2, 0, cx, cy - 600);
    sim.set_bay(0, 9, 2);
    sim.select_bay(0);
    let mut p = player_at(cx, cy, 2000, 0, 300);
    p.camera_mfd = true;
    p.hud_mode = HudMode::Gnd;
    p.cmd.nearest_target = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    sim.step(&p, 15);
    assert_eq!(sim.targeting().ground, Some(2));
}

#[test]
fn nearest_target_skips_friendlies_and_waits_for_hud_mode() {
    let d = data();
    let m = mission(d, 3, 0x1234, 0);
    let mut sim = Sim::new(&d.sim, &m);
    let (cx, cy) = sandbox(&mut sim, &m);
    for o in sim.objects_mut().iter_mut() {
        o.flags |= 0x80;
    }
    // Ahead (north): a friendly object at 300, a hostile one at 600 map units.
    place_site(&mut sim, 1, 0, cx, cy - 300);
    sim.objects_mut()[1].flags = 0x0400;
    place_site(&mut sim, 2, 0, cx, cy - 600);
    sim.set_bay(0, 9, 2);
    sim.select_bay(0);
    let mut p = player_at(cx, cy, 2000, 0, 300);
    p.camera_mfd = true;
    // The modern avionics: the key waits for the HUD to switch to the weapon's mode.
    p.mode_weapons = true;
    p.cmd.nearest_target = true;
    // Still in NAV: the request waits for the HUD to switch.
    sim.step(&p, 15);
    p.cmd = Commands::default();
    sim.step(&p, 15);
    p.hud_mode = HudMode::Gnd;
    sim.step(&p, 15);
    assert_eq!(sim.targeting().ground, Some(2));
}

/// A sim with an enemy aircraft and a ground site ahead at the given distances (map units), no
/// mission objective among them, and the player in the given HUD mode with a missile selected.
fn air_and_ground(mode: HudMode, air_at: i32, ground_at: i32) -> (Sim, PlayerState) {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (cx, cy) = sandbox(&mut sim, &m);
    for o in sim.objects_mut().iter_mut() {
        o.flags |= 0x80;
    }
    place_site(&mut sim, 2, 0, cx, cy - ground_at);
    for slot in sim.target_slots_mut() {
        slot.object = -1;
    }
    {
        let u = &mut sim.flights_mut()[1];
        *u = FlightUnit {
            x: cx,
            y: cy - air_at,
            alt: 2000,
            spec: 9,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: 3,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    sim.set_bay(0, 9, 2);
    sim.select_bay(0);
    let mut p = player_at(cx, cy, 2000, 0, 300);
    p.camera_mfd = true;
    p.hud_mode = mode;
    (sim, p)
}

/// What the nearest-target key locks in each HUD mode with an enemy aircraft and a ground site
/// ahead: (air lock, ground lock).
fn nearest_in(mode: HudMode, air_at: i32, ground_at: i32) -> (Option<usize>, Option<usize>) {
    let (mut sim, mut p) = air_and_ground(mode, air_at, ground_at);
    p.cmd.nearest_target = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    sim.step(&p, 15);
    (sim.targeting().air, sim.targeting().ground)
}

/// NAV takes the nearer of an aircraft and a ground target, AIR only aircraft, GND only ground
/// and sea targets: whatever the selected weapon is.
#[test]
fn nearest_target_follows_the_hud_mode() {
    assert_eq!(nearest_in(HudMode::Nav, 300, 600), (Some(1), None));
    assert_eq!(nearest_in(HudMode::Nav, 600, 300), (None, Some(2)));
    // In AIR and GND the other kind is not asked for: the nearer one of the wrong kind is passed.
    assert_eq!(nearest_in(HudMode::Air, 600, 300).0, Some(1));
    assert_eq!(nearest_in(HudMode::Gnd, 300, 600).1, Some(2));
}

/// The nearest-target key picks a mission objective in view over nearer hostile targets, and
/// the nearest once the objective is done.
#[test]
fn nearest_target_prefers_the_objective() {
    let d = data();
    let m = mission(d, 3, 0x1234, 0);
    let mut sim = Sim::new(&d.sim, &m);
    let (cx, cy) = sandbox(&mut sim, &m);
    for o in sim.objects_mut().iter_mut() {
        o.flags |= 0x80;
    }
    // Ahead (north): hostile objects at 300 and 600 map units, the far one the primary.
    place_site(&mut sim, 1, 0, cx, cy - 300);
    place_site(&mut sim, 2, 0, cx, cy - 600);
    sim.target_slots_mut()[0] = TargetSlot { mission_type: 0, object: 2, base: 0, code: 0 };
    sim.target_slots_mut()[1] = TargetSlot { mission_type: 0, object: -1, base: 0, code: 0 };
    sim.set_bay(0, 9, 2);
    sim.select_bay(0);
    let mut p = player_at(cx, cy, 2000, 0, 300);
    p.camera_mfd = true;
    p.hud_mode = HudMode::Gnd;
    p.cmd.nearest_target = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    sim.step(&p, 15);
    assert_eq!(sim.targeting().ground, Some(2));
    // Done: the nearest again.
    sim.target_slots_mut()[0].object = -1;
    p.cmd.nearest_target = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    sim.step(&p, 15);
    assert_eq!(sim.targeting().ground, Some(1));
}

/// The next-target key in GND passes an aircraft by with the classic avionics (in AIR it would
/// pick it, in NAV too); the modern avionics keep picking it.
#[test]
fn next_target_in_gnd_ignores_aircraft() {
    let next = |mode, modern| {
        let (mut sim, mut p) = air_and_ground(mode, 300, 600);
        p.mode_weapons = modern;
        p.cmd.next_target = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        sim.step(&p, 15);
        sim.targeting().air
    };
    assert_eq!(next(HudMode::Gnd, false), None);
    assert_eq!(next(HudMode::Air, false), Some(1));
    assert_eq!(next(HudMode::Nav, false), Some(1));
    assert_eq!(next(HudMode::Gnd, true), Some(1));
}

fn drone_mission(d: &Data, seed: u16, opponents: u8) -> Mission {
    let theater = stealth_formats::ALL_THEATERS.iter().position(|t| *t == "ukraine").unwrap();
    let s = MissionSettings {
        theater,
        roe: 1,
        opponents,
        kind: MissionKind::DroneDefense,
        ..Default::default()
    };
    stealth_mission::generate(&d.gdata, &s, Seed::Srand(seed)).unwrap()
}

/// Nobody stops the wave: every drone reaches the target and the defense fails.
#[test]
fn unopposed_drone_wave_gets_through() {
    let d = data();
    let m = drone_mission(d, 42, 1);
    let w = m.drones.clone().unwrap();
    let mut sim = Sim::new(&d.sim, &m);
    sim.set_training(true);
    // Out of the way, high over the base.
    let p = player_at(m.takeoff.position.x, m.takeoff.position.y, 30000, 0, 300);
    let mut impacts = 0;
    for _ in 0..15 * 60 * 40 {
        impacts +=
            sim.step(&p, 15).iter().filter(|e| matches!(e, SimEvent::DroneImpact { .. })).count();
        if sim.drones().unwrap().resolved {
            break;
        }
    }
    let s = sim.drones().unwrap();
    assert!(s.resolved, "after {} s: {s:?}", sim.mission_tick());
    assert_eq!((s.leaked, s.killed, impacts), (w.count as i32, 0, w.count));
    assert!(!sim.objectives().primary_done);
    let r = sim.mission_report();
    assert_eq!(r.drones_leaked, w.count as i32);
    // They arrive spread out, 9 minutes and more after the start.
    assert!(sim.mission_tick() > 9 * 60, "{}", sim.mission_tick());
}

/// A Sidewinder from behind kills a drone; it counts toward the defense.
#[test]
fn sidewinder_kills_a_drone() {
    let d = data();
    let m = drone_mission(d, 7, 0);
    let w = m.drones.clone().unwrap();
    let mut sim = Sim::new(&d.sim, &m);
    sim.set_training(true);
    let u = sim.flights()[w.first_unit];
    // 2 km behind the first drone, at its altitude, on its heading.
    let (hs, hc) = (
        (u.heading as u16 as f64 / 65536.0 * std::f64::consts::TAU).sin(),
        (u.heading as u16 as f64 / 65536.0 * std::f64::consts::TAU).cos(),
    );
    let (px, py) = (u.x - (hs * 128.0) as i32, u.y + (hc * 128.0) as i32);
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(px, py, u.alt as u16, u.heading as u16, 200);
    p.cmd.toggle_bay = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..5 {
        sim.step(&p, 15);
    }
    p.cmd.fire = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..15 * 20 {
        sim.step(&p, 15);
        if sim.drones().unwrap().killed > 0 {
            break;
        }
    }
    assert_eq!(sim.drones().unwrap().killed, 1, "{:?}", sim.flights()[w.first_unit]);
    assert_eq!(sim.mission_report().drones_killed, 1);
}

/// Ejects at 2000 ft over map point (x, y) and steps until the chute is down.
fn eject_at(sim: &mut Sim, x: i32, y: i32) -> PlayerState {
    let mut p = player_at(x, y, 2000, 0, 250);
    p.cmd.eject = true;
    let ev = sim.step(&p, 15);
    assert!(ev.contains(&SimEvent::Ejected { success: true }), "{ev:?}");
    p.cmd = Commands::default();
    for _ in 0..15 * 60 {
        sim.step(&p, 15);
        if sim.csar().is_some() || sim.mission_end().is_some() {
            break;
        }
    }
    p
}

fn enemy_cell(m: &Mission) -> (i32, i32) {
    let g = &m.world.grid;
    let c = (0..256)
        .find(|&i| {
            g[i] & 3 == 0
                && g[i] & 0x0C >= 8
                && (3..13).contains(&(i % 16))
                && (3..13).contains(&(i / 16))
        })
        .expect("enemy land");
    (((c % 16) as i32) * 2048 + 1024, ((c / 16) as i32) * 2048 + 1024)
}

fn disable_airfields(sim: &mut Sim, friendly: bool) {
    for o in sim.objects_mut().iter_mut() {
        if o.flags & oflags::AIRFIELD != 0 && (o.flags & oflags::FRIENDLY != 0) == friendly {
            o.flags |= oflags::DISABLED;
        }
    }
}

/// Down over enemy territory with no Hinds about: the UH-60 comes, the flare brings it in.
#[test]
fn csar_rescues_the_pilot() {
    let d = data();
    let m = mission(d, 3, 0x2345, 0);
    let mut sim = Sim::new(&d.sim, &m);
    disable_airfields(&mut sim, false);
    let (x, y) = enemy_cell(&m);
    let mut p = eject_at(&mut sim, x, y);
    let c = sim.csar().expect("rescue started").clone();
    assert!(c.hunters.is_empty());
    let mut flared = false;
    for _ in 0..15 * 60 * 15 {
        let r = sim.flights()[c.rescue];
        p.cmd.flare = !flared && (r.x - x).abs() + (r.y - y).abs() < 0x100;
        flared |= p.cmd.flare;
        sim.step(&p, 15);
        if sim.mission_end().is_some() {
            break;
        }
    }
    let e = sim.mission_end().expect("mission over");
    assert_eq!((e.outcome, e.landing_type), (0, 4), "{:?}", sim.csar());
    assert!(flared);
    assert_eq!(sim.mission_report().outcome, score::Outcome::Rescued);
}

/// Down next to an enemy airfield after showing a flare: the Hinds get there first.
#[test]
fn csar_hinds_capture_the_pilot() {
    let d = data();
    let m = mission(d, 3, 0x2345, 3);
    let mut sim = Sim::new(&d.sim, &m);
    let objects = sim.objects().to_vec();
    let (bx, by) = objects
        .iter()
        .filter(|o| {
            o.flags & (oflags::AIRFIELD | oflags::FRIENDLY | oflags::NEUTRAL | oflags::CARRIER)
                == oflags::AIRFIELD
        })
        .map(|o| (o.x, o.y))
        .find(|&(x, y)| m.world.grid[((y >> 11) * 16 + (x >> 11)) as usize] & 3 == 0)
        .expect("enemy airfield in enemy territory");
    let mut p = eject_at(&mut sim, bx + 0x60, by);
    assert!(!sim.csar().expect("rescue started").hunters.is_empty());
    p.cmd.flare = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..15 * 60 * 15 {
        sim.step(&p, 15);
        if sim.mission_end().is_some() {
            break;
        }
    }
    assert_eq!(sim.mission_end().map(|e| e.outcome), Some(8), "{:?}", sim.csar());
    assert_eq!(sim.mission_report().outcome, score::Outcome::Captured);
}

/// Down over enemy territory, the pilot aborts the mission: captured.
#[test]
fn csar_abort_is_a_capture() {
    let d = data();
    let m = mission(d, 3, 0x2345, 0);
    let mut sim = Sim::new(&d.sim, &m);
    disable_airfields(&mut sim, false);
    let (x, y) = enemy_cell(&m);
    eject_at(&mut sim, x, y);
    assert!(sim.csar().is_some(), "rescue started");
    assert!(sim.surrender());
    assert_eq!(sim.mission_end().map(|e| e.outcome), Some(8));
    assert_eq!(sim.mission_report().outcome, score::Outcome::Captured);
    assert!(!sim.surrender(), "captured once");
}

#[test]
fn space_steps_through_the_weapons_then_the_gun() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    sim.set_loadout(&[(0, 1), (-1, 0), (6, 1), (0, 1)]);
    let p = player_at(0x4000, 0x4000, 3000, 0, 300);
    let mut seen = Vec::new();
    for _ in 0..4 {
        let (groups, sel) = sim.weapon_groups();
        seen.push(if sim.gun_selected() { -1 } else { groups[sel.unwrap()].0 });
        let mut q = p.clone();
        q.cmd.next_bay = true;
        sim.step(&q, 15);
    }
    assert_eq!(seen, [0, 6, -1, 0]);
    assert_eq!(sim.weapon_groups().0, [(0, 2), (6, 1)]);
}

/// Realistic avionics: Space steps through the HUD mode's own weapons, a mode change picks one
/// of the new mode's, and in NAV nothing (not even the gun) fires.
#[test]
fn realistic_avionics_keep_weapons_to_their_mode() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let types = sim.weapon_types().to_vec();
    let of = |air: bool| {
        (0..types.len() as i32)
            .find(|&w| {
                let spec = types[w as usize].spec as i32;
                spec > 0 && (d.sim.missile(spec).class == 7) == air
            })
            .expect("weapon type")
    };
    let (aa, ag) = (of(true), of(false));
    sim.set_loadout(&[(ag, 2), (aa, 2), (-1, 0), (-1, 0)]);
    let selected = |sim: &Sim| {
        let (groups, sel) = sim.weapon_groups();
        if sim.gun_selected() { -1 } else { groups[sel.unwrap()].0 }
    };
    let mut p = player_at(0x4000, 0x4000, 3000, 0, 300);
    p.mode_weapons = true;
    p.hud_mode = HudMode::Air;
    sim.step(&p, 15);
    assert_eq!(selected(&sim), aa, "AIR picks the air-to-air missile");
    let mut seen = Vec::new();
    for _ in 0..3 {
        let mut q = p.clone();
        q.cmd.next_bay = true;
        sim.step(&q, 15);
        seen.push(selected(&sim));
    }
    assert_eq!(seen, [-1, aa, -1], "the gun and the air-to-air missile only");
    p.hud_mode = HudMode::Gnd;
    sim.step(&p, 15);
    assert_eq!(selected(&sim), -1, "the gun is kept in GND");
    let mut q = p.clone();
    q.cmd.next_bay = true;
    sim.step(&q, 15);
    assert_eq!(selected(&sim), ag);
    // NAV: the trigger and the gun trigger do nothing.
    p.hud_mode = HudMode::Nav;
    sim.step(&p, 15);
    let (before, ammo) = (sim.weapon_groups().0, sim.gun_ammo());
    let mut said = false;
    for cmd in [
        Commands { fire: true, ..Default::default() },
        Commands { gun: true, ..Default::default() },
    ] {
        let q = PlayerState { cmd, ..p.clone() };
        for _ in 0..10 {
            for e in sim.step(&q, 15) {
                said |= matches!(&e, SimEvent::Message(s) if en(s).starts_with("Select A-"));
            }
        }
    }
    assert_eq!((sim.weapon_groups().0, sim.gun_ammo()), (before, ammo));
    assert!(said);
}

/// A burst at an An-72 600 m ahead: the fire button fires the gun only when it is selected.
#[test]
fn trigger_fires_the_selected_gun() {
    let d = data();
    for selected in [false, true] {
        let m = mission(d, 3, 0x1234, 1);
        let mut sim = Sim::new(&d.sim, &m);
        let (mx, my) = sandbox(&mut sim, &m);
        {
            let u = &mut sim.flights_mut()[1];
            *u = FlightUnit {
                x: mx,
                y: my - 38,
                alt: 3000,
                spec: 9,
                flags: 0x43,
                speed: 200,
                timer: 30000,
                wp: 3,
                ..Default::default()
            };
            u.wx = u.x << 5;
            u.wy = u.y << 5;
        }
        sim.set_loadout(&[(0, 4)]);
        let mut p = player_at(mx, my, 3000, 0, 300);
        if selected {
            p.cmd.next_bay = true;
            sim.step(&p, 15);
            assert!(sim.gun_selected());
        }
        let ammo = sim.gun_ammo();
        p.cmd = Commands { trigger: true, ..Default::default() };
        let mut killed = false;
        for _ in 0..30 {
            for e in sim.step(&p, 15) {
                killed |= matches!(e, SimEvent::AircraftDestroyed { unit: 1, .. });
            }
        }
        assert_eq!(killed, selected, "selected {selected}");
        assert_eq!(sim.gun_ammo() < ammo, selected);
    }
}

/// Rounds keep the aircraft's speed and fall: 2 s after leaving a level aircraft a round is
/// about 20 m lower and (1050 m/s + the aircraft's speed) x 2 s ahead.
#[test]
fn gun_rounds_fly_ballistic() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    let mut p = player_at(mx, my, 5000, 0, 400);
    // One round (the gun fires every frame).
    p.cmd.gun = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    let (b0, _) = sim.bullets();
    assert_eq!(b0.iter().filter(|b| b.x != 0).count(), 1);
    let i = b0.iter().position(|b| b.x != 0).expect("a round");
    let start = b0[i].map_pos();
    for _ in 0..30 {
        sim.step(&p, 15);
    }
    let end = sim.bullets().0[i].map_pos();
    let wu = WU_METRES;
    let ahead = (start.1 - end.1) * 32.0 * wu;
    let drop = (start.2 - end.2) * wu;
    let want = (GUN_MUZZLE_SPEED + 400.0 * 0.514_444) * 2.0;
    assert!((ahead - want).abs() < want * 0.01, "{ahead} m ahead, want {want}");
    assert!((drop - 0.5 * GUN_GRAVITY * 4.0).abs() < 2.0, "{drop} m drop");
}

/// Bombs released in level flight at 500 ft and 450 knots land on the CCIP point predicted at
/// release: a slick bomb far ahead, a retarded one on a shorter arc (not straight down).
#[test]
fn bombs_land_on_the_ccip_point() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut ahead = Vec::new();
    for w in [9, 10] {
        let mut sim = Sim::new(&d.sim, &m);
        let (mx, my) = sandbox(&mut sim, &m);
        sim.set_training(true);
        sim.set_loadout(&[(w, 2)]);
        let mut p = player_at(mx, my, 500, 0, 450);
        p.hud_mode = HudMode::Gnd;
        p.cmd.toggle_bay = true;
        for _ in 0..4 {
            sim.step(&p, 15);
            p.cmd = Commands::default();
        }
        let ccip = sim.targeting().ccip.expect("CCIP point");
        p.cmd.fire = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let slot = (8..12).find(|&s| sim.projectiles()[s].ttl != 0).expect("bomb released");
        let mut frames = 0;
        while sim.projectiles()[slot].ttl != 0 && frames < 15 * 30 {
            sim.step(&p, 15);
            frames += 1;
        }
        let (x, y, alt) = sim.projectiles()[slot].map_pos();
        assert!(alt.abs() < 1e-6, "weapon {w}: landed at {alt}");
        let (x, y) = (x as i32, y as i32);
        assert!(
            (x - ccip.0).abs() <= 1 && (y - ccip.1).abs() <= 1,
            "weapon {w}: impact {x},{y} ccip {ccip:?}"
        );
        assert!((x - mx).abs() <= 1);
        ahead.push(my - y);
    }
    // 1 map unit is 32 ft: the slick bomb flies about 4,000 ft, the retarded one under 3,000.
    assert!(ahead[0] > 100 && ahead[1] > 40 && ahead[1] < ahead[0] * 3 / 4, "{ahead:?}");
}

/// CCRP: flying level at a locked target with the fire button held, the bomb is not released
/// on the press but when the target passes the release point, and lands on the target.
#[test]
fn ccrp_releases_at_the_release_point() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    for w in [9, 10] {
        let mut sim = Sim::new(&d.sim, &m);
        let (mx, my) = sandbox(&mut sim, &m);
        sim.set_training(true);
        sim.set_loadout(&[(w, 2)]);
        // The target 400 map units (12,800 ft) north of the start.
        let t = 5;
        {
            let o = &mut sim.objects_mut()[t];
            o.x = mx;
            o.y = my - 400;
            o.flags = 0;
        }
        let knots = 450u16;
        let mut p = player_at(mx, my, 500, 0, knots);
        p.hud_mode = HudMode::Gnd;
        p.ccrp = true;
        p.cmd.toggle_bay = true;
        p.cmd.lock_object = Some(t as u16);
        let step = knots as f64 * 27.0 / 16.0 / 15.0;
        let mut y = p.y as f64;
        let mut released_at = None;
        for f in 0..15 * 40 {
            if f == 5 {
                p.cmd.fire = true;
            }
            p.cmd.trigger = f >= 5;
            sim.step(&p, 15);
            p.cmd = Commands { trigger: p.cmd.trigger, ..Default::default() };
            if released_at.is_none() && sim.weapon_groups().0[0].1 < 2 {
                released_at = Some(f);
                assert!(f > 5, "weapon {w}: released on the press");
            }
            if released_at.is_some() && (8..12).all(|s| sim.projectiles()[s].ttl == 0) {
                break;
            }
            y += step;
            p.y = y as i32;
        }
        assert!(released_at.is_some(), "weapon {w}: not released");
        let slot = (8..12).find(|&s| sim.projectiles()[s].fall.is_some()).unwrap();
        let (x, y, _) = sim.projectiles()[slot].map_pos();
        let o = sim.objects()[t];
        assert!(
            (x - (o.x as f64 + 0.5)).abs() < 2.0 && (y - (o.y as f64 + 0.5)).abs() < 3.0,
            "weapon {w}: impact {x},{y} target {},{}",
            o.x,
            o.y
        );
        assert_eq!(sim.weapon_groups().0[0].1, 1, "weapon {w}: one bomb");
    }
}

/// Flies level north at 450 knots and 500 ft with `w` x4 loaded and the bay open, the fire
/// button pressed on frame 5 and held until `hold` frames; `setup` adjusts the report (bombing
/// mode, ripple, pod). Returns the bombs' impact points (map units) and the release frames.
fn bomb_run(
    w: i32,
    hold: i32,
    target: (i32, i32),
    setup: impl Fn(&mut PlayerState),
) -> (Vec<(f64, f64)>, Vec<i32>) {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(w, 4)]);
    let knots = 450u16;
    let mut p = player_at(mx, my, 500, 0, knots);
    p.hud_mode = HudMode::Gnd;
    p.cmd.toggle_bay = true;
    let (tx, ty) = (mx + target.0, my + target.1);
    setup(&mut p);
    if let Some(pod) = p.pod.as_mut() {
        pod.ground = Some((tx, ty));
    }
    let t = 5;
    {
        let o = &mut sim.objects_mut()[t];
        o.x = tx;
        o.y = ty;
        o.flags = 0;
    }
    if p.pod.is_none() {
        p.cmd.lock_object = Some(t as u16);
    }
    let step = knots as f64 * 27.0 / 16.0 / 15.0;
    let mut y = p.y as f64;
    let mut released = Vec::new();
    let mut left = 4;
    for f in 0..15 * 40 {
        p.cmd.fire = f == 5;
        p.cmd.trigger = (5..hold).contains(&f);
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let now = sim.weapon_groups().0[0].1;
        for _ in now..left {
            released.push(f);
        }
        left = now;
        if f > hold && (8..12).all(|s| sim.projectiles()[s].ttl == 0) {
            break;
        }
        y += step;
        p.y = y as i32;
    }
    let hits = (8..12)
        .filter(|&s| sim.projectiles()[s].fall.is_some())
        .map(|s| {
            let (x, y, _) = sim.projectiles()[s].map_pos();
            (x - tx as f64 - 0.5, y - ty as f64 - 0.5)
        })
        .collect();
    (hits, released)
}

/// CCRP ripple: four bombs 200 ft apart go one after the other with the button held, the
/// stick centred on the target.
#[test]
fn ccrp_ripple_straddles_the_target() {
    for w in [9, 10] {
        let (mut hits, released) = bomb_run(w, 15 * 40, (0, -400), |p| {
            p.ccrp = true;
            p.ripple = 4;
            p.ripple_ft = 200;
        });
        assert_eq!(released.len(), 4, "weapon {w}: {released:?}");
        assert!(released[0] > 5, "weapon {w}: released on the press");
        hits.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mid = hits.iter().map(|h| h.1).sum::<f64>() / 4.0;
        assert!(mid.abs() < 2.0, "weapon {w}: stick centre {mid} ({hits:?})");
        for k in 1..4 {
            let gap = (hits[k].1 - hits[k - 1].1) * 32.0;
            assert!((gap - 200.0).abs() < 40.0, "weapon {w}: spacing {gap} ({hits:?})");
            assert!(hits[k].0.abs() < 2.0, "weapon {w}: {hits:?}");
        }
    }
}

/// CCRP aims at the targeting pod's ground point with nothing locked, and still releases when
/// the aircraft passes it to one side: the bomb lands level with it, off to the side.
#[test]
fn ccrp_aims_at_the_pod_point() {
    for east in [0, 30] {
        let (hits, released) = bomb_run(9, 15 * 40, (east, -400), |p| {
            p.ccrp = true;
            p.pod = Some(PodAim::default());
        });
        assert_eq!(released.len(), 1, "{east}: {released:?}");
        let (dx, dy) = hits[0];
        assert!(dy.abs() < 3.0 && (dx + east as f64).abs() < 2.0, "{east}: {hits:?}");
    }
}

/// CCRP: let go of the button before the release point and nothing goes.
#[test]
fn ccrp_needs_the_button_held() {
    let (hits, released) = bomb_run(9, 30, (0, -400), |p| p.ccrp = true);
    assert!(released.is_empty() && hits.is_empty(), "{released:?}");
}

/// CCIP ripple: the press releases the first bomb, the rest follow at the spacing while the
/// button is held; let go, the ripple stops.
#[test]
fn ccip_ripple_while_held() {
    let ripple = |p: &mut PlayerState| {
        p.ripple = 4;
        p.ripple_ft = 300;
    };
    let (_, released) = bomb_run(9, 15 * 40, (0, -400), ripple);
    assert_eq!(released.len(), 4, "{released:?}");
    assert_eq!(released[0], 5);
    // 300 ft at 450 knots (760 ft/s): about 6 frames apart.
    for k in 1..4 {
        assert!((5..=7).contains(&(released[k] - released[k - 1])), "{released:?}");
    }
    let (_, released) = bomb_run(9, 13, (0, -400), ripple);
    assert_eq!(released.len(), 2, "{released:?}");
}

/// An SA-2 site 10 km north of a slow, high player with open bays fires until a hit; returns
/// the simulation after the first hit (None: no hit in 4 minutes).
fn sa2_hit(opts: SimOptions) -> Option<(Sim, PlayerState)> {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    place_site(&mut sim, 3, 1, mx, my - 640);
    let mut p = player_at(mx, my, 6000, 0x4000, 200);
    p.cmd.toggle_bay = true;
    for _ in 0..15 * 240 {
        let ev = sim.step(&p, 15);
        p.cmd = Commands::default();
        if ev.iter().any(|e| matches!(e, SimEvent::PlayerHit { .. })) {
            return Some((sim, p));
        }
    }
    None
}

/// Realistic damage: some missile hits destroy the aircraft (it explodes seconds later, ending
/// the mission), some start fires, control hits bend the controls; classic damage never ends
/// the mission.
#[test]
fn realistic_missile_damage() {
    let (mut destroyed, mut exploded, mut bent, mut survived) = (0, 0, 0, 0);
    for seed in 0..24u32 {
        let opts = SimOptions {
            seed: Some(seed * 7919 + 1),
            realistic_damage: true,
            ..Default::default()
        };
        let Some((mut sim, p)) = sa2_hit(opts) else { continue };
        let fx = sim.damage_effects();
        destroyed += fx.destroyed as i32;
        if fx.destroyed {
            assert_eq!(fx.throttle_cap, Some(0), "the engine dies with the airframe");
            assert!(fx.controls != ControlFault::default());
        }
        bent += (fx.controls != ControlFault::default()) as i32;
        // The site is gone: only the damage runs its course.
        sim.objects_mut()[3].unit_type = 0;
        for _ in 0..15 * 90 {
            sim.step(&p, 15);
        }
        match sim.mission_end() {
            Some(e) => {
                assert_eq!(e.outcome, 1);
                exploded += 1;
            }
            None => {
                assert!(!sim.damage_effects().destroyed);
                survived += 1;
            }
        }
    }
    assert!(
        destroyed > 0 && exploded >= destroyed && survived > 0,
        "{destroyed} {exploded} {survived}"
    );
    assert!(bent > 0);

    for seed in 0..8u32 {
        let opts = SimOptions { seed: Some(seed * 7919 + 1), ..Default::default() };
        let Some((mut sim, p)) = sa2_hit(opts) else { continue };
        assert!(!sim.damage_effects().destroyed);
        assert_eq!(sim.damage_effects().controls, ControlFault::default());
        sim.objects_mut()[3].unit_type = 0;
        for _ in 0..15 * 90 {
            sim.step(&p, 15);
        }
        assert!(sim.mission_end().is_none(), "classic damage is never fatal in the air");
    }
}

/// Realistic guidance: an SA-2 site launches at a player cruising past at 400 kt; the missile
/// flies on its own flight (no map-unit steps), trails smoke only while its motor burns, and
/// hits the moving aircraft.
#[test]
fn guided_sam_hits_a_cruising_player() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { realistic_guidance: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    place_site(&mut sim, 3, 1, mx, my - 640);
    let mut p = player_at(mx - 300, my, 6000, 0x4000, 400);
    p.cmd.toggle_bay = true;
    let (mut x, mut y) = (p.x as f64, p.y as f64);
    let (mut launched, mut hit, mut smoke_out) = (false, false, false);
    for _ in 0..15 * 240 {
        let ev = sim.step(&p, 15);
        p.cmd = Commands::default();
        // The player flies on east.
        let v = ballistics::release_velocity(p.heading as i32, p.velocity as i32, 0);
        x += v[0] / 15.0;
        y -= v[1] / 15.0;
        (p.x, p.y) = (x as i32, y as i32);
        let m = sim.projectiles()[3];
        if m.ttl > 0 && m.spec == 1 {
            assert!(m.flight.is_some(), "guided flight");
            launched = true;
            smoke_out |= !m.motor_visible();
        }
        if ev.iter().any(|e| matches!(e, SimEvent::PlayerHit { .. })) {
            hit = true;
            break;
        }
    }
    assert!(launched && hit, "launched {launched} hit {hit}");
    // An SA-2 at 10 km gets there with its sustainer still burning.
    assert!(!smoke_out);
}

/// Visual identification range: the mission's day or night, or the light level.
#[test]
fn visual_range_follows_the_light() {
    let d = data();
    for seed in [0x1234u16, 0x2345, 0x3456] {
        let m = mission(d, 3, seed, 1);
        let mut sim = Sim::new(&d.sim, &m);
        sim.set_visual_light(Some(255));
        let day = sim.visual_range();
        // A storm halves every range.
        let shift = (day == 4) as i32;
        assert_eq!(day, 8 >> shift);
        sim.set_visual_light(Some(0));
        assert_eq!(sim.visual_range(), 2 >> shift);
        sim.set_visual_light(Some(85));
        assert_eq!(sim.visual_range(), 4 >> shift);
        sim.set_visual_light(None);
        assert_eq!(sim.visual_range(), (8 >> m.night as i32) >> shift);
    }
}

fn aaa_mission(d: &Data) -> (Mission, Sim) {
    aaa_mission_with(d, SimOptions::default())
}

fn aaa_mission_with(d: &Data, opts: SimOptions) -> (Mission, Sim) {
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 2,
        kind: MissionKind::Strike,
        weather: Some(stealth_mission::Sky::Clear),
        ..Default::default()
    };
    let m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(0x4321)).unwrap();
    let sim = Sim::with_options(&d.sim, &m, SimOptions { air_defence: true, ..opts });
    (m, sim)
}

/// One unit of `kind` in a sandbox, the player hovering `north` map units north of it.
fn aaa_duel(kind: aaa::GroundKind, north: i32, alt: u16, light: u8) -> Vec<SimEvent> {
    aaa_duel_with(SimOptions::default(), kind, north, alt, light)
}

fn aaa_duel_with(
    opts: SimOptions,
    kind: aaa::GroundKind,
    north: i32,
    alt: u16,
    light: u8,
) -> Vec<SimEvent> {
    let d = data();
    let (m, mut sim) = aaa_mission_with(d, opts);
    let (x, y) = sandbox(&mut sim, &m);
    sim.set_visual_light(Some(light));
    let unit = aaa::GroundUnit {
        kind,
        x,
        y,
        destroyed: false,
        tracking: false,
        aim_az: 0.0,
        aim_el: 0.0,
        burst: 0,
        cooldown: 0,
        rounds: 0,
        fine: None,
        gone: false,
    };
    *sim.units_mut() = vec![unit];
    let p = player_at(x, y - north, alt, 0x4000, 0);
    let mut events = Vec::new();
    for _ in 0..15 * 30 {
        events.extend(sim.step(&p, 15));
    }
    events
}

#[test]
fn air_defence_only_with_the_option() {
    let d = data();
    let (m, sim) = aaa_mission(d);
    let units = sim.units();
    assert!(units.iter().any(|u| u.kind == aaa::GroundKind::Zsu), "{units:?}");
    assert!(units.iter().any(|u| u.kind == aaa::GroundKind::Manpads));
    let plain = Sim::new(&d.sim, &m);
    assert!(plain.units().is_empty());
    // The same mission places the same units.
    assert_eq!(aaa_mission(d).1.units(), units);
}

/// The MANPADS teams of a town stand round its edge, none in its middle.
#[test]
fn town_defenders_round_the_edge() {
    let d = data();
    let (_, sim) = aaa_mission(d);
    let towns = &d.sim.theaters[3].towns;
    assert!(!towns.is_empty());
    let dist = |u: &aaa::GroundUnit, t: &Town| ((u.x - t.x) as f64).hypot((u.y - t.y) as f64);
    let manpads: Vec<_> =
        sim.units().iter().filter(|u| u.kind == aaa::GroundKind::Manpads).collect();
    let on_edge = manpads
        .iter()
        .filter(|u| towns.iter().any(|t| (0.75..1.1).contains(&(dist(u, t) / t.r as f64))))
        .count();
    assert!(on_edge > 0, "{manpads:?}");
    for u in &manpads {
        assert!(towns.iter().all(|t| dist(u, t) > 0.75 * t.r as f64), "in a town's middle: {u:?}");
    }
}

#[test]
fn shilka_hits_low_and_slow_but_not_high() {
    // 1.5 km away at 600 ft: hit within 30 s.
    let low = aaa_duel(aaa::GroundKind::Zsu, 96, 600, 255);
    assert!(low.iter().any(|e| is_msg(e, "Hit by AAA")), "never hit");
    // 20000 ft: above its reach, it does not even fire.
    let high = aaa_duel(aaa::GroundKind::Zsu, 96, 20000, 255);
    assert!(!high.iter().any(|e| matches!(e, SimEvent::Message(m) if en(m).starts_with("Hit"))));
}

#[test]
fn s60_bursts_flak() {
    let ev = aaa_duel(aaa::GroundKind::S60, 160, 8000, 255);
    assert!(ev.iter().any(|e| matches!(e, SimEvent::Flak { .. })));
}

#[test]
fn manpads_need_to_see_the_player() {
    let fired = |ev: &[SimEvent]| {
        ev.iter()
            .any(|e| matches!(e, SimEvent::MissileLaunched { launcher: Launcher::Manpads(0), .. }))
    };
    // 3 km away at 2000 ft: by day the team sees and fires, on a moonless night it does not.
    assert!(fired(&aaa_duel(aaa::GroundKind::Manpads, 192, 2000, 255)));
    assert!(!fired(&aaa_duel(aaa::GroundKind::Manpads, 192, 2000, 0)));
}

/// The crews spot a helicopter by eye much closer in than a jet, the lower the closer, the
/// RAH-66 closer still, and closer by night.
#[test]
fn low_helicopters_get_close_unseen() {
    let heli = SimOptions { helicopter: true, ..Default::default() };
    let comanche = SimOptions { gunship: true, comanche: true, ..heli.clone() };
    let flak = |ev: &[SimEvent]| ev.iter().any(|e| matches!(e, SimEvent::Flak { .. }));
    let fired = |ev: &[SimEvent]| {
        ev.iter()
            .any(|e| matches!(e, SimEvent::MissileLaunched { launcher: Launcher::Manpads(0), .. }))
    };
    let s60 = aaa::GroundKind::S60;
    // 3 km from an S-60 at 100 ft by day: a jet draws flak, a helicopter not until it climbs.
    assert!(flak(&aaa_duel(s60, 192, 100, 255)));
    assert!(!flak(&aaa_duel_with(heli.clone(), s60, 192, 100, 255)));
    assert!(flak(&aaa_duel_with(heli.clone(), s60, 192, 1000, 255)));
    // 1.5 km at 50 ft: the helicopter is seen, the RAH-66 not; by night neither.
    assert!(flak(&aaa_duel_with(heli.clone(), s60, 96, 50, 255)));
    assert!(!flak(&aaa_duel_with(comanche.clone(), s60, 96, 50, 255)));
    assert!(!flak(&aaa_duel_with(heli.clone(), s60, 96, 50, 85)));
    // A MANPADS team 3 km away fires at one up at 2000 ft, not at one down at 100.
    let manpads = aaa::GroundKind::Manpads;
    assert!(fired(&aaa_duel_with(heli.clone(), manpads, 192, 2000, 255)));
    assert!(!fired(&aaa_duel_with(heli, manpads, 192, 100, 255)));
}

/// The scripted ten-minute flights with the air defences, in every theater.
#[test]
fn ten_minute_missions_with_air_defence() {
    let d = data();
    let opts = SimOptions { air_defence: true, ..Default::default() };
    let mut fired = 0;
    for th in 0..9 {
        let sim = ten_minute_mission(d, th, 1, 3, opts.clone());
        assert!(!sim.units().is_empty(), "theater {th}");
        fired += sim.units().iter().filter(|u| u.tracking).count();
    }
    eprintln!("units tracking at the end: {fired}");
}

/// The S-60s lead a crossing target: their shells burst around it, not where it was.
#[test]
fn flak_leads_a_moving_target() {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (x, y) = sandbox(&mut sim, &m);
    sim.set_visual_light(Some(255));
    let unit = aaa::GroundUnit {
        kind: aaa::GroundKind::S60,
        x,
        y,
        destroyed: false,
        tracking: false,
        aim_az: 0.0,
        aim_el: 0.0,
        burst: 0,
        cooldown: 0,
        rounds: 0,
        fine: None,
        gone: false,
    };
    *sim.units_mut() = vec![unit];
    let mut misses = Vec::new();
    // Flying east at 15 map units a second (234 m/s), 1500 ft, 2.5 km north of the gun.
    for f in 0..15 * 20 {
        let px = x - 200 + f;
        let p = player_at(px, y - 160, 1500, 0x4000, 450);
        for e in sim.step(&p, 15) {
            if let SimEvent::Flak { x: bx, y: by, alt } = e {
                let dx = (bx - px) as f64 * 15.625;
                let dy = (by - (y - 160)) as f64 * 15.625;
                let dz = (alt - 1500) as f64 * 0.3048;
                misses.push((dx * dx + dy * dy + dz * dz).sqrt());
            }
        }
    }
    misses.sort_by(f64::total_cmp);
    assert!(misses.len() > 5);
    let median = misses[misses.len() / 2];
    assert!(median < 80.0, "median miss {median:.0} m of {misses:?}");
}

/// The recon cue: range and aim against the limit for the locked photo target, the step the
/// camera still needs, and gone once the target is photographed.
#[test]
fn photo_cue_guides_to_the_recon_target() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim = Sim::new(&d.sim, &m);
    sim.set_training(true);
    let (cx, cy) = sandbox(&mut sim, &m);
    place_site(&mut sim, 5, 0, cx, cy - 200);
    sim.target_slots_mut()[0] = TargetSlot { mission_type: 1, object: 5, base: 0, code: 0x12 };
    let camera = sim.weapon_types().iter().position(|w| w.spec == -1).unwrap();
    sim.set_bay(0, camera as i32, 1);
    sim.select_bay(0);
    let mut p = player_at(cx, cy, 1000, 0, 300);
    sim.step(&p, 15);
    assert_eq!(sim.targeting().photo, None);
    sim.set_ground_lock(Some(5));
    sim.step(&p, 15);
    let cue = sim.targeting().photo.unwrap();
    assert_eq!(cue.needs, PhotoNeeds::BayOpen);
    assert!((195..=205).contains(&cue.range) && cue.range + cue.aim < cue.limit, "{cue:?}");
    p.cmd.toggle_bay = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..30 {
        sim.step(&p, 15);
    }
    assert_eq!(sim.targeting().photo.unwrap().needs, PhotoNeeds::CameraPage);
    p.camera_mfd = true;
    sim.step(&p, 15);
    assert_eq!(sim.targeting().photo.unwrap().needs, PhotoNeeds::Nothing);
    p.cmd.fire = true;
    for _ in 0..6 {
        sim.step(&p, 15);
    }
    assert!(sim.objectives().primary_done);
    assert_eq!(sim.targeting().photo, None);
}

/// A Maverick fired from 3000 ft at a target `range` map units northeast (classic guidance
/// does not steer for a bearing of exactly north), the nose on it: the
/// launch zone at the shot and whether it hit (None: no lock from there).
fn maverick_shot(guided: bool, range: i32) -> Option<(dlz::LaunchZone, bool)> {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { realistic_guidance: guided, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(5, 2)]);
    let t = 5;
    {
        let o = &mut sim.objects_mut()[t];
        let diag = (range as f64 / std::f64::consts::SQRT_2).round() as i32;
        o.x = mx + diag;
        o.y = my - diag;
        o.flags = 0;
    }
    let mut p = player_at(mx, my, 3000, 0x2000, 450);
    p.hud_mode = HudMode::Gnd;
    p.cmd.toggle_bay = true;
    p.cmd.lock_object = Some(t as u16);
    for _ in 0..4 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    // Nose down onto the target until the oval locks, diving along it.
    (0..96).map(|k| -k * 64).find(|&pitch| {
        p.pitch = pitch as i16;
        let angle = pitch as f64 * std::f64::consts::TAU / 65536.0;
        p.climb_rate = (450.0 * 27.0 / 16.0 * angle.sin()).round() as i32;
        sim.step(&p, 15);
        sim.targeting().lock_valid
    })?;
    let zone = sim.launch_zone().expect("launch zone");
    p.cmd.fire = true;
    let mut hit = false;
    for _ in 0..15 * 90 {
        for e in sim.step(&p, 15) {
            if let SimEvent::Message(s) = e {
                hit |= en(&s).contains("hit by AGM-65");
            }
        }
        p.cmd = Commands::default();
    }
    Some((zone, hit))
}

#[test]
fn launch_zone_predicts_maverick_hits() {
    for guided in [false, true] {
        let shots: Vec<_> =
            (1..=40).filter_map(|k| maverick_shot(guided, k * 20).map(|s| (k * 20, s))).collect();
        assert!(shots.len() > 20, "guided {guided}: {} locks", shots.len());
        let mut hits = 0;
        for &(range, (z, hit)) in &shots {
            let (lo, hi) = z.zone.expect("a zone");
            assert!(hi <= z.lock_max, "guided {guided}: {z:?}");
            assert!((z.range - range).abs() <= range / 10 + 2, "{z:?}");
            // Right but for the edges.
            let near_edge = (z.range - lo).abs() < 30 || (z.range - hi).abs() < 30;
            assert!(
                z.in_zone() == hit || near_edge,
                "guided {guided} range {range}: {z:?} hit {hit}"
            );
            hits += hit as i32;
        }
        assert!(hits > 5, "guided {guided}: {hits} hits");
    }
}

/// Anti-ship store `w` (3 Penguin, 4 Harpoon) fired level from 1000 ft at a ship `range` map
/// units northeast, locked on the display: the launch zone at the shot and whether it hit.
fn ship_shot(guided: bool, w: i32, range: i32) -> (dlz::LaunchZone, bool) {
    let d = data();
    let (m, mut sim, ship) = (0..8)
        .flat_map(|th| (0..8u16).map(move |k| (th, k)))
        .find_map(|(th, k)| {
            let m = mission(d, th, 0x1234 + k, 1);
            let opts = SimOptions { realistic_guidance: guided, ..Default::default() };
            let sim = Sim::with_options(&d.sim, &m, opts);
            let ship = (3..sim.objects().len()).find(|&i| sim.object_is_ship(i))?;
            Some((m, sim, ship))
        })
        .expect("a mission with a ship");
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(w, 2)]);
    {
        let o = &mut sim.objects_mut()[ship];
        let diag = (range as f64 / std::f64::consts::SQRT_2).round() as i32;
        o.x = mx + diag;
        o.y = my - diag;
        o.flags = 0;
    }
    let mut p = player_at(mx, my, 1000, 0x2000, 450);
    p.hud_mode = HudMode::Gnd;
    p.cmd.toggle_bay = true;
    p.cmd.lock_object = Some(ship as u16);
    for _ in 0..4 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    let zone = sim.launch_zone().expect("launch zone");
    p.cmd.fire = true;
    let mut hit = false;
    for _ in 0..15 * 240 {
        for e in sim.step(&p, 15) {
            if let SimEvent::Message(s) = e {
                hit |= en(&s).contains("hit by");
            }
        }
        p.cmd = Commands::default();
    }
    (zone, hit)
}

#[test]
fn launch_zone_predicts_anti_ship_hits() {
    for guided in [false, true] {
        for w in [3, 4] {
            let mut hits = 0;
            for range in (1..=12).map(|k| k * 100) {
                let (z, hit) = ship_shot(guided, w, range);
                assert!(z.at_lock && z.zone.is_some(), "guided {guided} w {w}: {z:?}");
                assert!((z.range - range).abs() <= range / 10 + 2, "{z:?}");
                // Classic missiles fired from close in may dive into the water by the ship and
                // hit it with the blast, which the zone does not count on; at the far edge
                // (the zone takes the ship as standing still) they may dive in short of it.
                let far_edge = z.zone.is_some_and(|(_, hi)| (hi - range).abs() < 40);
                assert!(
                    z.in_zone() == hit || (!guided && (hit && z.too_close() || far_edge)),
                    "guided {guided} w {w} range {range}: {z:?} hit {hit}"
                );
                hits += hit as i32;
            }
            // Classic missiles dropped clear of the F-117A's bays hit less often by luck from
            // close in.
            let least = if guided { 4 } else { 2 };
            assert!(hits >= least, "guided {guided} w {w}: {hits} hits");
        }
    }
}

/// Air-to-air store `w` fired from 5000 ft at a slow An-72 `range` map units northeast, flying
/// straight on `heading` (classic guidance does not steer for a bearing of exactly north): the
/// launch zone at the shot and whether it shot the An-72 down.
fn air_shot(guided: bool, w: i32, range: i32, heading: i32) -> (dlz::LaunchZone, bool) {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { realistic_guidance: guided, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(w, 2)]);
    let diag = (range as f64 / std::f64::consts::SQRT_2).round() as i32;
    let (x, y) = (mx + diag, my - diag);
    // It flies for its waypoint far ahead, at 1000 ft above the player.
    let wp = 6;
    {
        let ahead = if heading == 0x2000 { 1 } else { -1 } * 1000;
        let o = &mut sim.objects_mut()[wp];
        (o.x, o.y) = (x + ahead, y - ahead);
    }
    {
        let u = &mut sim.flights_mut()[1];
        *u = FlightUnit {
            x,
            y,
            alt: 6000,
            heading,
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
    let mut p = player_at(mx, my, 5000, 0x2000, 300);
    p.hud_mode = HudMode::Air;
    p.cmd.toggle_bay = true;
    p.cmd.lock_unit = Some(1);
    for _ in 0..6 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    let zone = sim.launch_zone().expect("launch zone");
    p.cmd.fire = true;
    let mut killed = false;
    for _ in 0..15 * 120 {
        for e in sim.step(&p, 15) {
            killed |= matches!(e, SimEvent::AircraftDestroyed { unit: 1, .. });
        }
        p.cmd = Commands::default();
        if killed {
            break;
        }
    }
    (zone, killed)
}

#[test]
fn launch_zone_predicts_air_to_air_hits() {
    for guided in [false, true] {
        for w in [0, 1] {
            // Tail chase and head-on.
            for heading in [0x2000, 0xA000u16 as i16 as i32] {
                let shots: Vec<_> =
                    (1..=30).map(|k| air_shot(guided, w, k * 100, heading)).collect();
                let mut hits = 0;
                for (k, (z, hit)) in shots.iter().enumerate() {
                    let range = (k as i32 + 1) * 100;
                    let (lo, hi) = z.zone.unwrap_or((i32::MAX, i32::MAX));
                    let edge = |e: i32| (z.range - e).abs() < 40 + e / 20;
                    // A missile passing just outside its fuse radius misses here and there in
                    // the zone: a miss between hits counts as a hit, as in the zone.
                    let hit_at = |k: usize| shots.get(k).is_some_and(|s| s.1);
                    let near_miss = k > 0 && hit_at(k - 1) && hit_at(k + 1);
                    assert!(
                        z.in_zone() == (*hit || near_miss) || edge(lo) || edge(hi),
                        "guided {guided} weapon {w} heading {heading:#x} range {range}: {z:?} \
                         hit {hit}"
                    );
                    if let Some(ne) = z.no_escape {
                        assert!(ne <= hi, "{z:?}");
                    }
                    hits += *hit as i32;
                }
                assert!(hits > 3, "guided {guided} weapon {w} heading {heading:#x}: {hits}");
            }
        }
    }
}
/// The air-to-air launch zone against an An-72 circling while the player closes, every 3
/// frames: its edges move smoothly (a near miss here and there in the zone does not cut it).
#[test]
fn launch_zone_holds_steady_on_a_turning_target() {
    let d = data();
    for guided in [false, true] {
        for w in [0, 1] {
            let m = mission(d, 3, 0x1234, 1);
            let opts = SimOptions { realistic_guidance: guided, ..Default::default() };
            let mut sim = Sim::with_options(&d.sim, &m, opts);
            let (mx, my) = sandbox(&mut sim, &m);
            sim.set_training(true);
            sim.set_loadout(&[(w, 2)]);
            {
                let u = &mut sim.flights_mut()[1];
                *u = FlightUnit {
                    x: mx + 1400,
                    y: my - 1400,
                    alt: 5000,
                    heading: 0x2000,
                    spec: 9,
                    flags: 0x43,
                    speed: 200,
                    timer: 30000,
                    wp: 3,
                    ..Default::default()
                };
                u.wx = u.x << 5;
                u.wy = u.y << 5;
            }
            let mut p = player_at(mx, my, 5000, 0x2000, 300);
            p.hud_mode = HudMode::Air;
            p.cmd.toggle_bay = true;
            p.cmd.lock_unit = Some(1);
            let mut y = p.y as f64;
            let mut last: Option<dlz::LaunchZone> = None;
            for f in 0..15 * 40 {
                sim.step(&p, 15);
                p.cmd = Commands::default();
                y += 300.0 * 27.0 / 16.0 / 15.0 * 0.7;
                p.y = y as i32;
                if f % 3 != 0 || f < 10 {
                    continue;
                }
                let z = sim.launch_zone().unwrap();
                let (lo, hi) = z.zone.expect("a zone");
                if let Some((l, h)) = last.and_then(|l| l.zone) {
                    let ne = |z: &dlz::LaunchZone| z.no_escape.unwrap_or(0);
                    let jump =
                        (lo - l).abs().max((hi - h).abs()).max((ne(&z) - ne(&last.unwrap())).abs());
                    assert!(jump < 400, "guided {guided} weapon {w} frame {f}: {last:?} -> {z:?}");
                }
                last = Some(z);
            }
        }
    }
}

/// A helicopter lands anywhere flat: stopped near an insertion target (mission type 9) the
/// troops get out after `INSERT_SECONDS`; far from it nothing happens, and the flight goes on.
#[test]
fn helicopter_inserts_troops_off_base() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let mut sim =
        Sim::with_options(&d.sim, &m, SimOptions { helicopter: true, ..Default::default() });
    let (x, y) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.objects_mut()[5].x = x;
    sim.objects_mut()[5].y = y;
    sim.target_slots_mut()[0] = TargetSlot { mission_type: 9, object: 5, base: 0, code: 0 };
    let landed = |mx: i32, my: i32| PlayerState {
        on_ground: true,
        gear_up: false,
        velocity: 0,
        ..player_at(mx, my, 0, 0, 0)
    };
    // Fly in, then land 1.5 km away: nothing.
    for _ in 0..15 * 2 {
        sim.step(&player_at(x + 0x60, y, 300, 0, 60), 15);
    }
    for _ in 0..15 * 15 {
        sim.step(&landed(x + 0x60, y), 15);
    }
    assert!(sim.mission_end().is_none(), "{:?}", sim.mission_end());
    assert!(!sim.objectives().primary_done);
    // 300 m from the target: the troops are out after 10 s, not before.
    let mut inserted = None;
    for f in 0..15 * 15 {
        for e in sim.step(&landed(x + 0x14, y), 15) {
            if is_msg(&e, "Troops inserted") {
                inserted = Some(f);
            }
        }
    }
    let f = inserted.expect("troops inserted");
    assert!((15 * 9..15 * 12).contains(&f), "after {f} frames");
    assert!(sim.objectives().primary_done && sim.mission_end().is_none());
}

/// The AH-1Z (else the UH-60) in a sandbox, hovering 1000 ft up heading northeast with
/// Hellfires on a pylon.
fn helicopter(gunship: bool, guided: bool) -> (Sim, i32, i32, PlayerState) {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions {
        external_stores: true,
        helicopter: true,
        gunship,
        realistic_guidance: guided,
        ..Default::default()
    };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(20, 4)]);
    let p = player_at(mx, my, 1000, 0x2000, 0);
    (sim, mx, my, p)
}

/// A Hellfire flies to wherever the targeting pod looks: fired at one target with the pod on
/// it, it hits the other one the pod is moved to during its flight.
#[test]
fn hellfire_follows_the_pod() {
    for guided in [false, true] {
        let (mut sim, mx, my, mut p) = helicopter(true, guided);
        let (first, second) = (5, 6);
        let place = |sim: &mut Sim, t: usize, x: i32, y: i32| {
            let o = &mut sim.objects_mut()[t];
            o.x = x;
            o.y = y;
            o.flags = 0;
        };
        place(&mut sim, first, mx + 100, my - 100);
        place(&mut sim, second, mx + 110, my - 88);
        p.altitude = 500;
        p.height = p.altitude;
        let pod =
            |x: i32, y: i32| Some(PodAim { heading: 0x2000, pitch: -0x400, ground: Some((x, y)) });
        p.pod = pod(mx + 100, my - 100);
        sim.step(&p, 15);
        p.cmd.fire = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let slot = (8..12).find(|&s| sim.projectiles()[s].ttl != 0).expect("Hellfire fired");
        assert_eq!(sim.projectiles()[slot].target, first as i32);
        let mut at = None;
        for frame in 0..15 * 30 {
            if frame == 15 {
                p.pod = pod(mx + 110, my - 88);
            }
            for e in sim.step(&p, 15) {
                if let SimEvent::Explosion { x, y, .. } = e {
                    at.get_or_insert((x, y));
                }
            }
            if at.is_some() {
                break;
            }
        }
        assert_eq!(at, Some((mx + 110, my - 88)), "guided {guided}");
    }
}

/// A jet's Hellfires (three on an F-16's rack) fly to the pod's spot too: fired 3,000 ft up at
/// 400 kt, some 4.5 km out, one hits the target the pod looks at.
#[test]
fn a_jets_hellfire_follows_the_pod() {
    for guided in [false, true] {
        let d = data();
        let m = mission(d, 3, 0x1234, 1);
        let opts =
            SimOptions { external_stores: true, realistic_guidance: guided, ..Default::default() };
        let mut sim = Sim::with_options(&d.sim, &m, opts);
        let (mx, my) = sandbox(&mut sim, &m);
        sim.set_training(true);
        sim.set_loadout(&[(20, 3)]);
        let t = 5;
        let o = &mut sim.objects_mut()[t];
        (o.x, o.y, o.flags) = (mx + 200, my - 200, 0);
        let mut p = player_at(mx, my, 3000, 0x2000, 400);
        p.pod = Some(PodAim { heading: 0x2000, pitch: -0x800, ground: Some((mx + 200, my - 200)) });
        sim.step(&p, 15);
        p.cmd.fire = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let slot = (8..12).find(|&s| sim.projectiles()[s].ttl != 0).expect("Hellfire fired");
        assert_eq!(sim.projectiles()[slot].target, t as i32, "guided {guided}");
        let mut at = None;
        for _ in 0..15 * 30 {
            for e in sim.step(&p, 15) {
                if let SimEvent::Explosion { x, y, .. } = e {
                    at.get_or_insert((x, y));
                }
            }
            if at.is_some() {
                break;
            }
        }
        assert_eq!(at, Some((mx + 200, my - 200)), "guided {guided}");
    }
}

/// The RAH-66 carries its Hellfires in bays: fired with the doors shut, the missile waits for them
/// to open, then they shut again a few seconds after the last launch.
#[test]
fn comanche_doors_open_to_fire() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { helicopter: true, gunship: true, comanche: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(20, 3), (20, 3)]);
    let o = &mut sim.objects_mut()[5];
    (o.x, o.y, o.flags) = (mx + 100, my - 100, 0);
    let mut p = player_at(mx, my, 500, 0x2000, 0);
    p.pod = Some(PodAim { heading: 0x2000, pitch: -0x400, ground: Some((mx + 100, my - 100)) });
    sim.step(&p, 15);
    assert!(!sim.bay_doors_open());
    let flying = |sim: &Sim| (8..12).filter(|&s| sim.projectiles()[s].ttl != 0).count();
    p.cmd.fire = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    assert_eq!(flying(&sim), 0, "waits for the doors");
    let mut frames = 0;
    while flying(&sim) == 0 && frames < 15 {
        sim.step(&p, 15);
        frames += 1;
    }
    assert!(flying(&sim) == 1 && sim.bay_doors_open(), "fired after {frames} frames");
    // Open while it fires, shut some 4 s after the launch.
    for _ in 0..15 * 3 {
        sim.step(&p, 15);
    }
    assert!(sim.bay_doors_open());
    for _ in 0..15 * 2 {
        sim.step(&p, 15);
    }
    assert_eq!(sim.bay_door_anim(), 0, "shut again");
}

/// A Hellfire fired from the AH-1Z 500 ft up at a target `range` map units northeast, the pod's
/// spot on it: the launch zone at the shot and whether it hit.
fn hellfire_shot(guided: bool, range: i32) -> (Option<dlz::LaunchZone>, bool) {
    let (mut sim, mx, my, mut p) = helicopter(true, guided);
    let diag = (range as f64 / std::f64::consts::SQRT_2).round() as i32;
    let (tx, ty) = (mx + diag, my - diag);
    {
        let o = &mut sim.objects_mut()[5];
        o.x = tx;
        o.y = ty;
        o.flags = 0;
    }
    p.altitude = 500;
    p.height = p.altitude;
    p.hud_mode = HudMode::Gnd;
    p.pod = Some(PodAim { heading: 0x2000, pitch: -0x100, ground: Some((tx, ty)) });
    sim.step(&p, 15);
    let zone = sim.launch_zone();
    p.cmd.fire = true;
    let mut hit = false;
    for _ in 0..15 * 150 {
        for e in sim.step(&p, 15) {
            if let SimEvent::Explosion { x, y, .. } = e {
                hit |= (x - tx).abs() <= 2 && (y - ty).abs() <= 2;
            }
        }
        p.cmd = Commands::default();
    }
    (zone, hit)
}

/// The Hellfire's launch zone is at the pod's spot, without a lock; with realistic guidance it
/// glides only so far: it slows once burnt out, stalls and falls short.
#[test]
fn hellfire_launch_zone_at_the_spot() {
    for guided in [false, true] {
        let (z, hit) = hellfire_shot(guided, 300);
        let z = z.expect("a launch zone");
        let (lo, hi) = z.zone.expect("in reach");
        assert!(z.in_zone() && hit, "guided {guided}: {z:?} hit {hit}");
        assert!(lo < 100 && (400..=512).contains(&hi), "guided {guided}: {z:?}");
        if guided {
            let (z, hit) = hellfire_shot(guided, hi + 100);
            assert!(!z.unwrap().in_zone() && !hit, "past the zone: {z:?} hit {hit}");
            let (z, hit) = hellfire_shot(guided, hi - 60);
            assert!(z.unwrap().in_zone() && hit, "inside the zone: {z:?} hit {hit}");
        }
    }
}

/// With realistic guidance a Hellfire leaves the rail along the aircraft's nose and flies on
/// straight for 1 s before turning onto the spot (it can be lobbed); here 30 degrees off the
/// nose, in its seeker's view.
#[test]
fn guided_hellfire_flies_straight_off_the_rail() {
    let (mut sim, mx, my, mut p) = helicopter(true, true);
    let (tx, ty) = (mx + 260, my - 150);
    {
        let o = &mut sim.objects_mut()[5];
        o.x = tx;
        o.y = ty;
        o.flags = 0;
    }
    p.altitude = 500;
    p.height = p.altitude;
    p.hud_mode = HudMode::Gnd;
    p.pod = Some(PodAim { heading: 0x4000, pitch: -0x100, ground: Some((tx, ty)) });
    sim.step(&p, 15);
    p.cmd.fire = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    let slot = (8..12).find(|&s| sim.projectiles()[s].ttl != 0).expect("Hellfire fired");
    let mut hit = false;
    for frame in 0..15 * 30 {
        if frame < 13 {
            let m = sim.projectiles()[slot];
            assert!((m.heading - 0x2000).abs() < 0x80, "frame {frame}: heading {:#x}", m.heading);
            assert!(m.pitch.abs() < 0x100, "frame {frame}: pitch {:#x}", m.pitch);
        }
        for e in sim.step(&p, 15) {
            if let SimEvent::Explosion { x, y, .. } = e {
                hit |= (x - tx).abs() <= 2 && (y - ty).abs() <= 2;
            }
        }
    }
    assert!(hit, "it turns onto the spot after the straight flight");
}

/// A point designated with the targeting pod away from any object is the ground lock itself
/// (not the nearest object, which may be kilometres away): it holds while rounds hit the ground
/// elsewhere, and the Hellfire's launch zone ranges to it. Next to an object, the object.
#[test]
fn pod_designates_a_ground_point() {
    let (mut sim, mx, my, mut p) = helicopter(true, true);
    let o = 5;
    {
        let ob = &mut sim.objects_mut()[o];
        ob.x = mx + 300;
        ob.y = my - 300;
        ob.flags = 0;
    }
    p.altitude = 500;
    p.height = p.altitude;
    p.hud_mode = HudMode::Gnd;
    let (x, y) = (mx + 150, my - 150);
    p.pod = Some(PodAim { heading: 0x2000, pitch: -0x100, ground: Some((x, y)) });
    p.cmd.designate_at = Some((x, y));
    sim.step(&p, 15);
    p.cmd = Commands::default();
    for _ in 0..30 {
        let mut q = p.clone();
        q.cmd.gun = true;
        sim.step(&q, 15);
    }
    let t = sim.targeting();
    assert_eq!(t.ground, Some(0), "{t:?}");
    let g = sim.objects()[0];
    assert_eq!((g.x, g.y, g.shape), (x, y, 0));
    let z = sim.launch_zone().expect("a launch zone");
    assert!((z.range - 212).abs() <= 3 && z.in_zone(), "{z:?}");
    // Close to the object: the object.
    p.cmd.designate_at = Some((mx + 303, my - 297));
    sim.step(&p, 15);
    assert_eq!(sim.targeting().ground, Some(o));
}

/// Slewing the pod moves the launch zone's target with it (the SPI): the Hellfire's, and a
/// Maverick's without a lock; the zone is against the lock again once the pod rests on it.
#[test]
fn launch_zone_follows_the_slewed_pod() {
    let (mut sim, mx, my, mut p) = helicopter(true, true);
    p.altitude = 500;
    p.height = p.altitude;
    p.hud_mode = HudMode::Gnd;
    let pod =
        |x: i32, y: i32| Some(PodAim { heading: 0x2000, pitch: -0x100, ground: Some((x, y)) });
    p.pod = pod(mx + 150, my - 150);
    p.cmd.designate_at = Some((mx + 150, my - 150));
    sim.step(&p, 15);
    p.cmd = Commands::default();
    let z = sim.launch_zone().unwrap();
    assert!(z.at_lock && (z.range - 212).abs() <= 3, "{z:?}");
    p.pod = pod(mx + 300, my - 300);
    sim.step(&p, 15);
    let z = sim.launch_zone().unwrap();
    assert!(!z.at_lock && (z.range - 424).abs() <= 3, "{z:?}");

    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { realistic_guidance: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(5, 2)]);
    let mut p = player_at(mx, my, 3000, 0x2000, 450);
    p.hud_mode = HudMode::Gnd;
    p.pod = pod(mx + 200, my - 200);
    sim.step(&p, 15);
    let z = sim.launch_zone().expect("a zone against the pod's point");
    assert!(!z.at_lock && (z.range - 283).abs() <= 3, "{z:?}");
}

/// The AH-1Z's cannon fires where the pod looks, as far as its turret turns; the UH-60 has none.
#[test]
fn turret_gun_follows_the_pod() {
    let round = |pod_heading: u16, gunship: bool| {
        let (mut sim, _, _, mut p) = helicopter(gunship, false);
        p.heading = 0;
        p.pod = Some(PodAim { heading: pod_heading, pitch: -0x800, ground: None });
        p.cmd.gun = true;
        sim.step(&p, 15);
        sim.bullets().0.iter().find(|b| b.x != 0).copied()
    };
    // East and down: the round flies east (x grows), falling.
    let b = round(0x4000, true).expect("a round");
    assert!(b.vx > 0 && b.vy.abs() < b.vx / 50 && b.vz < 0, "{b:?}");
    // Straight behind: the turret stops 110 degrees left (west and a little aft, south).
    let b = round(0x8000, true).expect("a round");
    assert!(b.vx < 0 && b.vy > 0 && b.vy < -b.vx, "{b:?}");
    assert!(round(0x4000, false).is_none(), "the UH-60 has no cannon");
}

/// The AH-1Z's cannon is two weapon entries: slaved to the helmet sight, then to the pod; its
/// turret follows the one it is slaved to.
#[test]
fn gunship_gun_slaved_to_helmet_then_pod() {
    let (mut sim, _, _, mut p) = helicopter(true, false);
    p.heading = 0;
    let space = |sim: &mut Sim, p: &PlayerState| {
        let mut q = p.clone();
        q.cmd.next_bay = true;
        sim.step(&q, 15);
    };
    // Hellfires, then the gun on the helmet, then on the pod, then the Hellfires again.
    assert!(!sim.gun_selected());
    space(&mut sim, &p);
    assert!(sim.gun_selected() && !sim.gun_on_pod());
    space(&mut sim, &p);
    assert!(sim.gun_selected() && sim.gun_on_pod());
    space(&mut sim, &p);
    assert!(!sim.gun_selected() && sim.gun_on_pod(), "the turret stays on the pod");
    space(&mut sim, &p);
    assert!(sim.gun_selected() && !sim.gun_on_pod(), "selected again: on the helmet");
    // The helmet looks east, the pod west.
    p.helmet = Some(PodAim { heading: 0x4000, pitch: -0x400, ground: None });
    p.pod = Some(PodAim { heading: 0xC000, pitch: -0x400, ground: None });
    let fire = |sim: &mut Sim, p: &PlayerState| {
        let mut q = p.clone();
        q.cmd.gun = true;
        sim.step(&q, 15);
        let b = sim.bullets().0.iter().find(|b| b.x != 0).copied().expect("a round");
        q.cmd.gun = false;
        for _ in 0..30 {
            sim.step(&q, 15);
        }
        b
    };
    assert!(fire(&mut sim, &p).vx > 0, "along the helmet");
    space(&mut sim, &p);
    assert!(fire(&mut sim, &p).vx < 0, "along the pod");
    // No helmet sight (an outside view): along the pod.
    space(&mut sim, &p);
    space(&mut sim, &p);
    assert!(!sim.gun_on_pod());
    p.helmet = None;
    assert!(fire(&mut sim, &p).vx < 0);
}

/// The gunship's, and a fixed-wing aircraft's with the HMCS (the F-16's).
#[test]
fn helmet_seeker_locks_in_its_circle() {
    let gunship = SimOptions { helicopter: true, gunship: true, ..Default::default() };
    helmet_seeker_circle(gunship);
    helmet_seeker_circle(SimOptions::default());
}

fn helmet_seeker_circle(opts: SimOptions) {
    let d = data();
    let m = mission(d, 3, 0x1234, 2);
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    // Two An-72s: 3 km north, and 2 km away 30 degrees right of north.
    for (k, (dx, dy)) in [(0, -192), (64, -111)].into_iter().enumerate() {
        let u = &mut sim.flights_mut()[k + 1];
        *u = FlightUnit {
            x: mx + dx,
            y: my + dy,
            alt: 5000,
            spec: 9,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: 3,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(mx, my, 5000, 0, 0);
    p.hud_mode = HudMode::Air;
    let mut look = |sim: &mut Sim, heading: u16| {
        p.helmet = Some(PodAim { heading, pitch: 0, ground: None });
        sim.step(&p, 15);
        sim.targeting()
    };
    // Looking north: the one ahead, the nearer one outside the circle.
    let t = look(&mut sim, 0);
    assert!(sim.helmet_seeker().is_some());
    assert_eq!(t.air, Some(1));
    // Towards the other: the first leaves the circle, the seeker takes the second.
    assert_eq!(look(&mut sim, 0x1555).air, Some(2));
    // Nothing in the circle: no lock.
    let t = look(&mut sim, 0x4000);
    assert_eq!(t.air, None);
    assert!(!t.lock_valid);
    // AIM-120s selected: no helmet seeker.
    sim.set_bay(0, 1, 4);
    look(&mut sim, 0);
    assert!(sim.helmet_seeker().is_none());
}

#[test]
fn helmet_seeker_stops_at_its_gimbals() {
    let d = data();
    let m = mission(d, 3, 0x1234, 2);
    let opts = SimOptions { helicopter: true, gunship: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    // An An-72 3 km off, 70 degrees right of the nose.
    {
        let u = &mut sim.flights_mut()[1];
        *u = FlightUnit {
            x: mx + 180,
            y: my - 66,
            alt: 5000,
            spec: 9,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: 3,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(mx, my, 5000, 0, 0);
    p.hud_mode = HudMode::Air;
    // Looking at it: the seeker stops at its gimbal limit off the nose and sees nothing.
    let heading = (70.0f64 / 360.0 * 65536.0) as u16;
    p.helmet = Some(PodAim { heading, pitch: 0, ground: None });
    sim.step(&p, 15);
    let los = sim.helmet_seeker().expect("seeker");
    assert!((los[1].acos() - SEEKER_GIMBAL).abs() < 1e-3, "{los:?}");
    assert_eq!(sim.targeting().air, None);
}

#[test]
fn helmet_seeker_locks_the_one_it_looks_at() {
    let d = data();
    let m = mission(d, 3, 0x1234, 2);
    let opts = SimOptions { helicopter: true, gunship: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    // An An-72 2.3 km ahead, 4 degrees right, and another 4.7 km straight ahead behind it: both
    // in the circle.
    for (k, (dx, dy)) in [(10, -150), (0, -300)].into_iter().enumerate() {
        let u = &mut sim.flights_mut()[k + 1];
        *u = FlightUnit {
            x: mx + dx,
            y: my + dy,
            alt: 5000,
            spec: 9,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: 3,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(mx, my, 5000, 0, 0);
    p.hud_mode = HudMode::Air;
    p.helmet = Some(PodAim { heading: 0, pitch: 0, ground: None });
    for _ in 0..3 {
        sim.step(&p, 15);
        assert_eq!(sim.targeting().air, Some(2));
    }
}

#[test]
fn helmet_seeker_locks_any_aircraft() {
    let d = data();
    let m = mission(d, 3, 0x1234, 2);
    let opts = SimOptions { helicopter: true, gunship: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    // An aircraft of a type the nearest-target key passes over, 3 km ahead.
    let spec = (0..sim.aircraft_types.len() as i32)
        .find(|&s| sim.friendly_aircraft(s))
        .expect("a friendly type");
    {
        let u = &mut sim.flights_mut()[1];
        *u = FlightUnit {
            x: mx,
            y: my - 192,
            alt: 5000,
            spec,
            flags: 0x43,
            speed: 200,
            timer: 30000,
            wp: 3,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(mx, my, 5000, 0, 0);
    p.hud_mode = HudMode::Air;
    p.helmet = Some(PodAim { heading: 0, pitch: 0, ground: None });
    sim.step(&p, 15);
    assert_eq!(sim.targeting().air, Some(1));
}

/// An HSD lock holds with the AIM-9 selected and the head turned away: the seeker looks at it
/// instead of the helmet sight.
#[test]
fn helmet_seeker_slaved_to_hsd_lock() {
    let d = data();
    let m = mission(d, 3, 0x1234, 2);
    let mut sim = Sim::with_options(&d.sim, &m, SimOptions::default());
    let (mx, my) = sandbox(&mut sim, &m);
    // An An-72 3 km north.
    let u = &mut sim.flights_mut()[1];
    *u = FlightUnit {
        x: mx,
        y: my - 192,
        alt: 5000,
        spec: 9,
        flags: 0x43,
        speed: 200,
        timer: 30000,
        wp: 3,
        ..Default::default()
    };
    u.wx = u.x << 5;
    u.wy = u.y << 5;
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(mx, my, 5000, 0, 0);
    p.hud_mode = HudMode::Air;
    p.helmet = Some(PodAim { heading: 0x4000, pitch: 0, ground: None });
    p.cmd.lock_unit = Some(1);
    for _ in 0..6 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    let t = sim.targeting();
    assert_eq!(t.air, Some(1));
    assert!(t.lock_valid);
    let los = sim.helmet_seeker().expect("seeker");
    assert!(los[1] > 0.99, "{los:?}");
}

/// The aircraft parked on a carrier's deck (base traffic) stand clear of the player's start.
#[test]
fn carrier_traffic_clear_of_the_player() {
    let d = data();
    let mut checked = 0;
    for theater in 0..d.gdata.theaters.len() {
        let m = mission(d, theater, 0x2468, 1);
        let mut sim = Sim::new(&d.sim, &m);
        let carriers: Vec<usize> = (0..sim.objects().len())
            .filter(|&i| sim.objects()[i].flags & oflags::CARRIER != 0)
            .collect();
        for b in carriers {
            let p = sim.start_state_at(b);
            for _ in 0..4 {
                sim.step(&p, 15);
            }
            let (px, py) = (p.x >> 5, 0x8000 - (p.y >> 5));
            for u in
                sim.flights().iter().filter(|u| u.wp == b as i32 && u.speed == 0 && u.flags != 0)
            {
                let d = (u.x - px).abs().max((u.y - py).abs());
                assert!(
                    d >= 4,
                    "theater {theater} carrier {b}: a parked aircraft {d} map units from the player"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "no carrier traffic checked");
}

/// The HUD picks a Geran ahead for the Sidewinders by itself in AIR mode, also after a ground
/// lock held in GND mode (entering AIR lets go of it).
#[test]
fn sidewinders_pick_a_drone_after_a_ground_lock() {
    let d = data();
    let m = drone_mission(d, 7, 0);
    let w = m.drones.clone().unwrap();
    let mut sim = Sim::new(&d.sim, &m);
    sim.set_training(true);
    let u = sim.flights()[w.first_unit];
    let a = u.heading as u16 as f64 / 65536.0 * std::f64::consts::TAU;
    let (px, py) = (u.x - (a.sin() * 192.0) as i32, u.y + (a.cos() * 192.0) as i32);
    sim.set_bay(0, 0, 4);
    sim.select_bay(0);
    let mut p = player_at(px, py, u.alt as u16 + 500, u.heading as u16, 200);
    // A ground point locked first (the pod, the HSD).
    p.hud_mode = HudMode::Gnd;
    p.cmd.designate_at = Some((px, py - 300));
    sim.step(&p, 15);
    p.cmd = Commands::default();
    sim.step(&p, 15);
    assert!(sim.targeting().ground.is_some());
    p.hud_mode = HudMode::Air;
    for _ in 0..5 {
        sim.step(&p, 15);
    }
    let t = sim.targeting();
    assert_eq!(t.air, Some(w.first_unit));
    assert!(t.lock_valid && t.in_range);
}

/// Patrolling fighters know nothing of a player no radar has seen: over 20 minutes they do
/// not close on a RAH-66 hovering low in enemy territory (they used to steer for its bearing).
#[test]
fn patrols_ignore_an_unseen_player() {
    let d = data();
    let opts = SimOptions { helicopter: true, gunship: true, comanche: true, ..Default::default() };
    let (mut closing, mut n) = (0i64, 0i64);
    for (th, seed) in [(0usize, 0x4321u16), (1, 0x55), (3, 0x1234), (7, 0x22), (9, 0x99)] {
        let m = mission(d, th, seed, 3);
        let mut sim = Sim::with_options(&d.sim, &m, opts.clone());
        let (mx, my) = enemy_cell(&m);
        let p = player_at(mx, my, 60, 0, 0);
        let mut last = vec![None; sim.flights().len()];
        for f in 0..15 * 1200 {
            sim.step(&p, 15);
            assert_eq!(sim.alarm_state().0, 0, "theater {th}: the player was seen");
            if f % 150 != 0 {
                continue;
            }
            for (k, u) in sim.flights().iter().enumerate() {
                let patrol = u.flags & (fflags::EXISTS | fflags::AIRBORNE | fflags::DESTROYED)
                    == fflags::EXISTS | fflags::AIRBORNE
                    && u.mode == 3
                    && u.alt > 1000
                    && !sim.friendly_aircraft(u.spec);
                let r = ((mx - u.x) as f64).hypot((my - u.y) as f64) as i64;
                if let (true, Some(l)) = (patrol, last[k]) {
                    closing += l - r;
                    n += 1;
                }
                last[k] = patrol.then_some(r);
            }
        }
    }
    let mean = closing / n.max(1);
    assert!(n > 500 && mean < 25, "{n} samples, closing {mean} units per 10 s");
}

/// The OH-58D's gun is its machine gun pods: the pod stations are not selectable, the gun is
/// the entry after the missiles with 500 rounds a pod, it fires along the nose, and the model's
/// pylons show what is left on them.
#[test]
fn kiowa_gun_pods_are_its_gun() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts =
        SimOptions { helicopter: true, external_stores: true, kiowa: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    sim.set_loadout(&[(-1, 0), (-1, 0)]);
    assert!(!sim.has_gun(), "no gun without a pod");
    sim.set_loadout(&[(20, 2), (22, 1)]);
    assert!(sim.has_gun());
    assert_eq!(sim.gun_ammo(), GUN_POD_ROUNDS);
    assert_eq!(sim.bays(), (&[(20, 2), (22, 1)][..], 0));
    assert!(!sim.gun_selected());
    assert_eq!(sim.pylon_looks(sim.bays().0), [1, 3]);
    assert_eq!(sim.pylon_looks(&[(20, 1), (21, 0)]), [5, 10]);
    // Space: Hellfires, the gun, the Hellfires again.
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    let mut p = player_at(mx, my, 500, 0, 100);
    for want in [true, false] {
        p.cmd.next_bay = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        assert_eq!(sim.gun_selected(), want);
        assert_eq!(sim.bays().1, 0, "never the pod's station");
    }
    // A pod-only loadout starts on the gun, with both pods' rounds; firing uses them.
    sim.set_loadout(&[(22, 1), (22, 1)]);
    sim.select_bay(0);
    assert!(sim.gun_selected());
    assert_eq!(sim.gun_ammo(), 2 * GUN_POD_ROUNDS);
    p.cmd.trigger = true;
    for _ in 0..15 {
        sim.step(&p, 15);
    }
    assert!(sim.gun_ammo() < 2 * GUN_POD_ROUNDS);
}

/// Store `w` fired level from 5000 ft heading northeast (`external`: off a pylon, else out of
/// the F-117A's bays; from station 0 of `hardpoints`): the frames until its motor lit, how far
/// (wu) it sank below the launch altitude by then, and where it started (wu: east, south, up).
fn motor_lights(
    guided: bool,
    external: bool,
    w: i32,
    hardpoints: &[[f32; 3]],
) -> (u32, f64, [f64; 3]) {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts =
        SimOptions { realistic_guidance: guided, external_stores: external, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(w, 2)]);
    sim.set_hardpoints(hardpoints);
    let mut p = player_at(mx, my, 5000, 0x2000, 450);
    p.hud_mode = if w <= 1 { HudMode::Air } else { HudMode::Gnd };
    p.cmd.toggle_bay = !external;
    for _ in 0..4 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    sim.step(&p, 15);
    let start = sim.active_projectiles().into_iter().find(|m| m.0 >= 8).expect("fired").1;
    let fine = stealth_sim::ROUND_FINE as f64;
    let at = |m: &Projectile| match m.flight {
        Some(f) => [f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine],
        None => [m.x as f64 * 32.0, m.y as f64 * 32.0, m.alt as f64],
    };
    let launch = at(&start);
    let mut frames = 0;
    p.cmd = Commands::default();
    loop {
        let m = sim.active_projectiles().into_iter().find(|m| m.0 >= 8).expect("flying").1;
        if m.motor_visible() {
            let a = at(&m);
            return (frames, launch[2] - a[2], launch);
        }
        assert!(frames < 60, "no motor");
        sim.step(&p, 15);
        frames += 1;
    }
}

#[test]
fn dropped_missiles_light_their_motors_clear() {
    for guided in [false, true] {
        // The Maverick drops off its pylon for half a second (8 frames, the first the one it
        // was fired in); the AMRAAM and the Sidewinder light theirs on the rail.
        assert_eq!(motor_lights(guided, true, 5, &[]).0, 7, "guided {guided}");
        assert_eq!(motor_lights(guided, true, 1, &[]).0, 0, "guided {guided}");
        assert_eq!(motor_lights(guided, true, 0, &[]).0, 0, "guided {guided}");
        // Everything drops out of the F-117A's bays first, sinking well clear.
        let (frames, sank, _) = motor_lights(guided, false, 1, &[]);
        assert_eq!(frames, 8, "guided {guided}");
        assert!(sank >= 8.0, "guided {guided}: {sank}");
    }
    // Off a station 2 m right of the origin and 1 m under it, not 20 wu under the middle.
    let wu = stealth_sim::WU_METRES;
    let (_, _, a) = motor_lights(true, true, 1, &[]);
    let (_, _, b) = motor_lights(true, true, 1, &[[2.0, 0.0, -1.0]]);
    let side = 2.0 / wu * std::f64::consts::FRAC_1_SQRT_2;
    let want = [side, side, 20.0 - 1.0 / wu];
    for k in 0..3 {
        assert!((b[k] - a[k] - want[k]).abs() < 0.6, "{a:?} -> {b:?}, want {want:?}");
    }
}

/// A Maverick dropped while climbing nose high stays below the flight path (the nose is above
/// it by the angle of attack) until its motor lights, rather than rising over the wing.
#[test]
fn dropped_missiles_fall_below_a_climb() {
    for guided in [false, true] {
        let d = data();
        let m = mission(d, 3, 0x1234, 1);
        let opts =
            SimOptions { realistic_guidance: guided, external_stores: true, ..Default::default() };
        let mut sim = Sim::with_options(&d.sim, &m, opts);
        let (mx, my) = sandbox(&mut sim, &m);
        sim.set_training(true);
        sim.set_loadout(&[(5, 2)]);
        // 400 kt (675 ft/s) climbing at 5 degrees, the nose 12.7 degrees up.
        let mut p = player_at(mx, my, 5000, 0x2000, 400);
        p.hud_mode = HudMode::Gnd;
        p.pitch = 0x0900;
        p.climb_rate = 59;
        for _ in 0..4 {
            sim.step(&p, 15);
            p.cmd = Commands::default();
        }
        p.cmd.fire = true;
        sim.step(&p, 15);
        p.cmd = Commands::default();
        let fine = stealth_sim::ROUND_FINE as f64;
        let alt = |m: &Projectile| match m.flight {
            Some(f) => f.alt as f64 / fine,
            None => m.alt as f64,
        };
        let first = sim.active_projectiles().into_iter().find(|m| m.0 >= 8).expect("fired").1;
        let start = alt(&first);
        for frame in 1..=8 {
            sim.step(&p, 15);
            let m = sim.active_projectiles().into_iter().find(|m| m.0 >= 8).expect("flying").1;
            // The flight path's height after this long (frame 0 was the launch frame); classic
            // altitudes are whole feet. Along the nose it would rise 6 ft a frame above it.
            let path = start + 59.0 * frame as f64 / 15.0 + if guided { 0.0 } else { 1.5 };
            assert!(alt(&m) < path, "guided {guided} frame {frame}: {} above {path}", alt(&m));
        }
    }
}

/// A CBU-105 run: an aircraft `alt` ft up at 450 kt heading north in a sandbox, the targeting
/// pod on a point 4 km ahead and 300 m to the right with `units` around it (map offsets from
/// the point), the burst altitude `burst` ft. Returns the simulation and the point after the
/// release press.
fn cbu105_run(
    alt: u16,
    burst: u16,
    units: &[(aaa::GroundKind, i32, i32)],
) -> (Sim, PlayerState, (i32, i32), Vec<SimEvent>) {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(CBU105_WEAPON, 2)]);
    let aim = (mx + 20, my - 256);
    *sim.units_mut() = units
        .iter()
        .map(|&(kind, dx, dy)| aaa::GroundUnit {
            kind,
            x: aim.0 + dx,
            y: aim.1 + dy,
            ..Default::default()
        })
        .collect();
    let mut p = player_at(mx, my, alt, 0, 450);
    p.hud_mode = HudMode::Gnd;
    p.burst_ft = burst;
    p.pod = Some(PodAim { heading: 0, pitch: -0x400, ground: Some(aim) });
    p.cmd.toggle_bay = true;
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(sim.step(&p, 15));
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    events.extend(sim.step(&p, 15));
    p.cmd = Commands::default();
    (sim, p, aim, events)
}

/// The CBU-105 is not released below its burst altitude, nor without a marked point.
#[test]
fn cbu105_refuses_release_below_burst_altitude() {
    let (sim, _, _, events) = cbu105_run(1000, 1500, &[]);
    assert!((8..12).all(|s| sim.projectiles()[s].ttl == 0), "released below the burst altitude");
    assert!(events.iter().any(|e| is_msg(e, "Below burst altitude (1,500 ft)")
        || is_msg(e, "Below burst altitude (1500 ft)")));
    let (sim, _, _, _) = cbu105_run(3000, 1500, &[]);
    assert!((8..12).any(|s| sim.projectiles()[s].ttl != 0), "released above it");
    let (sim, _, _, _) = cbu105_run(3000, 500, &[]);
    assert!((8..12).any(|s| sim.projectiles()[s].ttl != 0), "released above a lower one");
}

/// Without a point marked with the pod (or a ground lock) there is nothing to fly to.
#[test]
fn cbu105_needs_a_marked_point() {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(CBU105_WEAPON, 2)]);
    let mut p = player_at(mx, my, 10000, 0, 450);
    p.hud_mode = HudMode::Gnd;
    p.cmd.toggle_bay = true;
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(sim.step(&p, 15));
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    events.extend(sim.step(&p, 15));
    assert!((8..12).all(|s| sim.projectiles()[s].ttl == 0));
    assert!(events.iter().any(|e| is_msg(e, "No target designated")));
}

/// Released 15,000 ft up, the CBU-105 flies to the point marked at release (the pod looking
/// elsewhere after), opens at its burst altitude and its submunitions settle over the point.
#[test]
fn cbu105_guides_to_the_marked_point() {
    for burst in [500, 1500, 3000] {
        let (mut sim, mut p, aim, _) = cbu105_run(15000, burst, &[]);
        p.pod = Some(PodAim { heading: 0x4000, pitch: -0x400, ground: Some((aim.0 + 300, aim.1)) });
        let mut opened = None;
        let mut settled = None;
        for _ in 0..15 * 120 {
            for e in sim.step(&p, 15) {
                if let SimEvent::SfwDispense { alt, .. } = e {
                    opened = Some(alt as f64 / 256.0);
                }
            }
            let b = &sim.sfw().blus;
            if opened.is_some()
                && b.len() == 10
                && b.iter().all(|b| b.phase != sfw::Phase::Eject && b.phase != sfw::Phase::Chute)
            {
                let n = b.len() as f64;
                let c = b.iter().fold((0.0, 0.0), |a, b| (a.0 + b.p[0] / n, a.1 + b.p[1] / n));
                settled = Some(c);
                break;
            }
        }
        let opened = opened.expect("dispenser opened");
        assert!((opened - burst as f64).abs() < 60.0, "burst {burst}: opened at {opened} ft");
        let (cx, cy) = settled.expect("submunitions came down");
        let (ax, ay) = (aim.0 as f64 * 32.0 + 16.0, aim.1 as f64 * 32.0 + 16.0);
        let off = ((cx - ax).hypot(cy - ay)) / 3.28;
        eprintln!(
            "burst {burst}: pattern centre {off:.0} m off ({:.0}, {:.0} ft)",
            cx - ax,
            cy - ay
        );
        assert!(off < 60.0, "burst {burst}: pattern centre {off:.0} m from the point");
    }
}

/// The skeets fire down at the vehicles under the pattern and destroy most of them; those that
/// find nothing destroy themselves in the air; a vehicle far off is left alone.
#[test]
fn skeets_strike_vehicles_under_the_pattern() {
    use aaa::GroundKind::*;
    let units = [(T90, 0, 0), (Bmp2, 3, 0), (T90, -3, 2), (Bmp2, 0, -3), (T90, 0, 52)];
    let (mut sim, p, _, _) = cbu105_run(15000, 1500, &units);
    let (mut fired, mut bursts, mut killed) = (0, 0, Vec::new());
    for _ in 0..15 * 120 {
        for e in sim.step(&p, 15) {
            match e {
                SimEvent::SkeetFire { .. } => fired += 1,
                SimEvent::SkeetBurst { .. } => bursts += 1,
                SimEvent::AaaDestroyed { unit, by } => {
                    assert_eq!(by, Some(0), "the kill is the pilot's");
                    killed.push(unit);
                }
                _ => {}
            }
        }
        if fired + bursts == 40 {
            break;
        }
    }
    eprintln!("fired {fired}, self-destructed {bursts}, killed {killed:?}");
    assert_eq!(fired + bursts, 40, "every skeet fired or burst");
    assert!(killed.iter().filter(|&&k| k < 4).count() >= 3, "most of the column: {killed:?}");
    assert!(!killed.contains(&4), "the vehicle 800 m off is left alone");
    assert!(sim.sfw().is_empty() || sim.sfw().skeets.is_empty());
}

/// What became of an AI fighter shot at under realistic AI flight.
#[derive(Debug, Default)]
struct Evasion {
    killed: bool,
    /// Height lost (alt units) within the first 6 s after the shot.
    dive: i32,
    /// The fighter's heading off its bearing to the player (BAM), each second.
    off: Vec<i32>,
    /// The most flares and chaff it had in the air at once.
    cartridges: usize,
}

/// Fires air-to-air store `w` (0 AIM-9, 1 AIM-120) from 5000 ft at a MiG-29 at 6000 ft about
/// `range` map units north, coming at the player (or, `tail`, flying away north): what the MiG
/// did, defending itself (`evade`, with flares and chaff: `cms`) or not.
fn evasion_shot(w: i32, range: i32, tail: bool, evade: bool, cms: bool, seed: u32) -> Evasion {
    let d = data();
    let mut m = mission(d, 3, 0x1234, 3);
    m.weather = Weather::default();
    let opts = SimOptions {
        realistic_guidance: true,
        realistic_ai_flight: true,
        seed: Some(seed),
        ..Default::default()
    };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_ai_evasion(evade);
    sim.set_ai_countermeasures(cms);
    sim.set_loadout(&[(w, 2)]);
    // A little different each time.
    let k = seed as i32 % 8;
    let (x, y) = (mx + 8 + range / 12 * k / 8, my - range - range / 24 * k / 8);
    let wp = 6;
    {
        let o = &mut sim.objects_mut()[wp];
        (o.x, o.y) = (x, if tail { y - 0x3000 } else { y + 0x3000 });
    }
    {
        let u = &mut sim.flights_mut()[1];
        *u = FlightUnit {
            x,
            y,
            alt: 6000,
            heading: if tail { 0 } else { 0x8000u16 as i16 as i32 },
            spec: 2,
            flags: 0x43,
            speed: 600,
            timer: 30000,
            wp: wp as i32,
            ..Default::default()
        };
        u.wx = u.x << 5;
        u.wy = u.y << 5;
    }
    let mut p = player_at(mx, my, 5000, 0, 300);
    p.hud_mode = HudMode::Air;
    p.cmd.toggle_bay = true;
    p.cmd.lock_unit = Some(1);
    for _ in 0..6 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    let mut r = Evasion::default();
    let alt0 = sim.flights()[1].alt;
    for f in 0..15 * 90 {
        for e in sim.step(&p, 15) {
            r.killed |= matches!(e, SimEvent::AircraftDestroyed { unit: 1, .. });
        }
        p.cmd = Commands::default();
        let u = sim.flights()[1];
        if f < 15 * 6 {
            r.dive = r.dive.max(alt0 - u.alt);
        }
        r.cartridges = r.cartridges.max(sim.ai_countermeasures().count());
        if f % 15 == 0 {
            let brg = math::bearing(mx - u.x, -(my - u.y));
            r.off.push(math::s16(u.heading - brg));
        }
        if r.killed {
            break;
        }
    }
    r
}

/// Kills of eight shots.
fn evasion_kills(w: i32, range: i32, tail: bool, evade: bool, cms: bool) -> usize {
    (1..9).filter(|&k| evasion_shot(w, range, tail, evade, cms, k).killed).count()
}

/// A Sidewinder fired head-on from 4 km: the MiG breaks into a dive with flares, and gets away
/// more often than one that flies on.
#[test]
fn ai_dives_and_flares_against_a_close_shot() {
    let shots: Vec<_> = (1..9).map(|k| evasion_shot(0, 64 * 4, false, true, true, k)).collect();
    assert!(shots.iter().any(|r| r.dive > 300), "{shots:?}");
    assert!(shots.iter().filter(|r| r.cartridges > 0).count() >= 6, "{shots:?}");
    let evading = shots.iter().filter(|r| r.killed).count();
    let flying_on = evasion_kills(0, 64 * 4, false, false, false);
    assert!(evading < flying_on, "{evading} of 8 evading shot down, {flying_on} flying on");
}

/// An AMRAAM fired head-on from 34 km: the MiG cranks off the missile's line to one side,
/// then the other, and the missile that has to follow kills less often.
#[test]
fn ai_cranks_a_long_shot() {
    let r = evasion_shot(1, 64 * 34, false, true, false, 1);
    let (at, first) =
        r.off.iter().take(12).copied().enumerate().max_by_key(|(_, o)| o.abs()).unwrap();
    assert!((0x1C00..0x3000).contains(&first.abs()), "crank {first:#x}: {:?}", r.off);
    // Then it turns back across to crank the other way.
    assert!(
        r.off.iter().skip(at).take(6).any(|&o| (o - first) * first.signum() < -0x1000),
        "never turned back: {:?}",
        r.off
    );
    let evading = evasion_kills(1, 64 * 34, false, true, true);
    let flying_on = evasion_kills(1, 64 * 34, false, false, false);
    assert!(evading < flying_on, "{evading} of 8 evading shot down, {flying_on} flying on");
    // From behind it weaves away from it.
    let evading = evasion_kills(1, 64 * 14, true, true, true);
    let flying_on = evasion_kills(1, 64 * 14, true, false, false);
    assert!(evading < flying_on, "tail: {evading} of 8 evading shot down, {flying_on} flying on");
}

/// Realistic AI flight over ten minutes of every theater: the jets fly (and land) as before,
/// none flies into the ground.
#[test]
fn ai_flight_missions_all_theaters() {
    let d = data();
    let mut counts = Vec::new();
    for physics in [false, true] {
        let (mut crashed, mut landed, mut takeoffs, mut flying) = (0, 0, 0, 0);
        for th in 0..9 {
            for (k, opp) in [(0u16, 0u8), (1, 3)] {
                let opts = SimOptions {
                    realistic_guidance: true,
                    realistic_ai_flight: physics,
                    ..Default::default()
                };
                let sim = ten_minute_mission_seen(d, th, k, opp, opts, &mut |e| match e {
                    SimEvent::AircraftCrashed { .. } => crashed += 1,
                    SimEvent::AircraftTakeoff { .. } => takeoffs += 1,
                    SimEvent::Message(t) if en(t).contains("landing") => landed += 1,
                    _ => {}
                });
                flying += sim.flights().iter().filter(|u| u.flags & 3 == 3).count();
            }
        }
        eprintln!(
            "physics {physics}: crashed {crashed} landed {landed} takeoffs {takeoffs} flying {flying}"
        );
        counts.push((crashed, landed, takeoffs));
    }
    let (classic, physics) = (counts[0], counts[1]);
    assert!(physics.0 <= classic.0 + 2, "{physics:?} vs classic {classic:?}");
    assert!(physics.1 * 4 >= classic.1 * 3, "{physics:?} vs classic {classic:?}");
}

/// An An-72 3 km ahead in AIR: classic avionics lock it by themselves, realistic ones only on
/// a 'B' press.
#[test]
fn realistic_avionics_lock_only_on_request() {
    let d = data();
    for modern in [false, true] {
        let m = mission(d, 3, 0x1234, 2);
        let mut sim = Sim::new(&d.sim, &m);
        let (mx, my) = sandbox(&mut sim, &m);
        {
            let u = &mut sim.flights_mut()[1];
            *u = FlightUnit {
                x: mx,
                y: my - 192,
                alt: 5000,
                spec: 9,
                flags: 0x43,
                speed: 200,
                timer: 30000,
                wp: 3,
                ..Default::default()
            };
            u.wx = u.x << 5;
            u.wy = u.y << 5;
        }
        let mut p = player_at(mx, my, 5000, 0, 0);
        p.hud_mode = HudMode::Air;
        p.mode_weapons = modern;
        for _ in 0..3 {
            sim.step(&p, 15);
        }
        assert_eq!(sim.targeting().air, (!modern).then_some(1), "modern {modern}");
        let mut q = p.clone();
        q.cmd.next_target = true;
        sim.step(&q, 15);
        sim.step(&p, 15);
        assert_eq!(sim.targeting().air, Some(1), "'B' locks it (modern {modern})");
    }
}

/// The cargo jet's mission (`stealth_mission::airlift`): the pallets released over the drop
/// zone are delivered (the primary), the stop on the delivery airfield's runway unloads the
/// cargo (the secondary) and ends the flight safely; no cannon.
#[test]
fn cargo_jet_drops_and_delivers() {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        kind: MissionKind::Strike,
        land_bases: true,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(0x2345)).unwrap();
    stealth_mission::airlift::for_transport(&mut m, &d.gdata);
    let opts = SimOptions { galaxy: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    assert!(!sim.has_gun());
    sim.set_training(true);
    let zone = sim.objects()[m.primary.object_index as usize];
    sim.set_bay(0, 18, 4);
    sim.select_bay(0);
    let mut p = player_at(zone.x, zone.y + 10, 1000, 0, 150);
    // Realistic avionics with the HUD in NAV: the pallets go in any mode.
    p.mode_weapons = true;
    // With the ramp up nothing goes.
    p.cmd.fire = true;
    let ev = sim.step(&p, 15);
    assert!(ev.iter().any(|e| is_msg(e, "Ramp closed")), "{ev:?}");
    p.cmd = Commands::default();
    // The ramp goes down in its own time.
    p.cmd.toggle_bay = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    assert!(sim.ramp().1, "on its way down");
    for _ in 0..15 * 4 {
        sim.step(&p, 15);
    }
    let (half, _) = sim.ramp();
    assert!(half > 0.3 && half < 0.7 && !sim.bay_doors_open(), "ramp {half}");
    for _ in 0..15 * 5 {
        sim.step(&p, 15);
    }
    assert_eq!(sim.ramp(), (1.0, false));
    assert!(sim.bay_doors_open());
    p.cmd.fire = true;
    let mut dropped = false;
    for _ in 0..15 * 30 {
        for e in sim.step(&p, 15) {
            dropped |= is_msg(&e, "Equipment Delivered");
        }
        p.cmd = Commands::default();
    }
    assert!(dropped, "the pallets missed the drop zone");
    assert!(sim.objectives().primary_done);
    // Stopped on the delivery airfield's runway.
    let dest = sim.objects()[m.secondary.object_index as usize];
    let mut p = player_at(dest.x, dest.y, 0, 0, 0);
    (p.on_ground, p.gear_up, p.altitude, p.height) = (true, false, 0, 0);
    let mut delivered = false;
    for _ in 0..15 * 10 {
        for e in sim.step(&p, 15) {
            delivered |= is_msg(&e, "Cargo delivered");
        }
    }
    assert!(delivered, "no delivery at {}", m.secondary.place_name);
    assert!(sim.objectives().secondary_done);
    assert_eq!(sim.mission_end().map(|e| e.outcome), Some(0));
}

/// Cargo jet airlifts (`stealth_mission::airlift`) in every theater, flown straight to the
/// drop zone at 8000 ft: a routine one behind the lines meets no enemy aircraft and takes no
/// hits; a front-line one takes two friendly fighters along, which take off after the player
/// and stay with him, and getting hit on the way is the exception.
#[test]
fn cargo_jet_airlifts_are_quiet_or_escorted() {
    use stealth_sim::fflags;
    let d = data();
    let (mut quiet, mut front, mut hit, mut escorted, mut checks) = (0, 0, 0, 0, 0);
    for theater in 0..10 {
        for seed in 0..4u16 {
            let s = MissionSettings {
                theater,
                roe: 2,
                kind: MissionKind::Strike,
                land_bases: true,
                ..Default::default()
            };
            let Ok(mut m) = stealth_mission::generate(&d.gdata, &s, Seed::Srand(seed * 977 + 5))
            else {
                continue;
            };
            stealth_mission::airlift::for_transport(&mut m, &d.gdata);
            let name = format!("{} {seed}", m.theater_name);
            let opts = SimOptions { galaxy: true, ..Default::default() };
            let mut sim = Sim::with_options(&d.sim, &m, opts);
            let (tx, ty) = (m.takeoff.position.x, m.takeoff.position.y);
            let (zx, zy) = (m.primary.position.x, m.primary.position.y);
            // On the runway for half a minute, then along the route at 300 kt (10 map units a
            // second).
            let mut p = player_at(tx, ty, 0, 0, 0);
            (p.on_ground, p.gear_up, p.altitude, p.height) = (true, false, 0, 0);
            for _ in 0..15 * 30 {
                sim.step(&p, 15);
            }
            let leg = (((zx - tx) as f64).powi(2) + ((zy - ty) as f64).powi(2)).sqrt();
            let secs = (leg / 10.0) as usize;
            let (mut hostile, mut hits) = (0, 0);
            for k in 0..secs * 15 {
                let f = k as f64 / (secs * 15) as f64;
                let x = tx + ((zx - tx) as f64 * f) as i32;
                let y = ty + ((zy - ty) as f64 * f) as i32;
                for e in sim.step(&player_at(x, y, 8000, 0, 300), 15) {
                    hits += matches!(e, SimEvent::PlayerHit { .. }) as usize;
                }
                if k % 15 != 0 {
                    continue;
                }
                let up = |u: &FlightUnit| {
                    u.flags & fflags::AIRBORNE != 0 && u.flags & fflags::DESTROYED == 0
                };
                let near = |u: &FlightUnit, km: i32| {
                    up(u) && (u.x - x).pow(2) + (u.y - y).pow(2) < (km * 64).pow(2)
                };
                let fl = sim.flights();
                // Not counting the Ukraine drone war's attack drones (type 29), which leave
                // aircraft alone.
                hostile += (0..fl.len())
                    .filter(|&i| near(&fl[i], 30) && !sim.friendly_unit(i, fl[i].spec))
                    .filter(|&i| fl[i].spec != 29)
                    .count();
                if !m.quiet && k > 15 * 120 {
                    let escort = |i: usize| sim.campaign_flight(i).is_some_and(|f| f.blue);
                    checks += 1;
                    escorted += (0..fl.len()).any(|i| escort(i) && near(&fl[i], 16)) as usize;
                }
            }
            println!(
                "{name}: quiet {} leg {:.0} km, enemy aircraft sighted {hostile}, hits {hits}",
                m.quiet,
                leg / 64.0
            );
            if m.quiet {
                assert_eq!(hostile, 0, "{name}: enemy aircraft on a quiet airlift");
                assert_eq!(hits, 0, "{name}: hit on a quiet airlift");
                quiet += 1;
            } else {
                front += 1;
                hit += (hits > 0) as usize;
            }
        }
    }
    println!("{quiet} quiet, {front} front-line ({hit} hit), escorted {escorted}/{checks}");
    assert!(quiet >= 15 && front >= 15, "quiet {quiet} front {front}");
    assert!(escorted * 10 >= checks * 9, "escorted {escorted}/{checks}");
    assert!(hit * 4 <= front, "{hit} of {front} front-line airlifts hit");
}

/// A SLAM-ER run: an F/A-18E-like jet 20,000 ft up heading north in a sandbox, the mission's
/// primary target moved `north` map units ahead of it, `rounds` SLAM-ERs on a pylon, channel 0
/// programmed `prog`. Returns the simulation, the input and the target's map point after the
/// launch press.
fn slam_run(
    north: i32,
    rounds: i32,
    prog: slam::SlamProgram,
) -> (Sim, PlayerState, (i32, i32), Vec<SimEvent>) {
    let d = data();
    let (m, mut sim) =
        aaa_mission_with(d, SimOptions { external_stores: true, ..Default::default() });
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(SLAMER_WEAPON, rounds)]);
    sim.units_mut().clear();
    let target = (mx + 10, my - north);
    let k = sim.target_slots()[0].object as usize;
    {
        let o = &mut sim.objects_mut()[k];
        o.x = target.0;
        o.y = target.1;
    }
    let mut p = player_at(mx, my, 20000, 0, 450);
    p.hud_mode = HudMode::Gnd;
    p.slam = Some(slam::SlamInput { sel: 0, prog: [prog; 4], steer: (0, 0) });
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(sim.step(&p, 15));
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    events.extend(sim.step(&p, 15));
    p.cmd = Commands::default();
    (sim, p, target, events)
}

/// Fired at the primary target, the SLAM-ER descends to its cruise altitude, stays level, turns
/// its seeker on at the programmed range and, left alone, snaps onto the target and hits it.
#[test]
fn slam_er_cruises_to_its_target_and_snaps_onto_it() {
    let prog = slam::SlamProgram { flt: 0, seeker_nm: 3, target: 0 };
    let (mut sim, p, target, _) = slam_run(4000, 1, prog);
    let fine = ROUND_FINE as f64;
    let (tx, ty) = (target.0 as f64 * 32.0 + 16.0, target.1 as f64 * 32.0 + 16.0);
    let mut low = f64::MAX;
    let mut active_at = None;
    let mut hit = None;
    for _ in 0..15 * 400 {
        let ev = sim.step(&p, 15);
        let Some((_, m)) = sim.slam_rounds().into_iter().next() else {
            hit = ev.iter().find_map(|e| match *e {
                SimEvent::Explosion { x, y, kind: 0x10, .. } => Some((x, y)),
                _ => None,
            });
            break;
        };
        assert_eq!(m.roll, 0, "always level");
        let f = m.flight.unwrap();
        let (x, y, alt) = (f.x as f64 / fine, f.y as f64 / fine, f.alt as f64 / fine);
        let g = m.slam.unwrap();
        if g.active && active_at.is_none() {
            active_at = Some((x - tx).hypot(y - ty));
        }
        if !g.active {
            low = low.min(alt);
        }
    }
    assert!((450.0..800.0).contains(&low), "cruised down to {low}");
    let seeker = 3.0 * 1852.0 / WU_METRES;
    let at = active_at.expect("seeker came on");
    assert!((at - seeker).abs() < 1000.0, "seeker on {at} wu out, want {seeker}");
    let (x, y) = hit.expect("hit the ground with a blast");
    assert!((x - target.0).abs() <= 2 && (y - target.1).abs() <= 2, "{x},{y} vs {target:?}");
}

/// The pilot's slew turns the missile once its seeker is on: right turns it east.
#[test]
fn slam_er_follows_the_slew() {
    let prog = slam::SlamProgram { flt: 1, seeker_nm: 10, target: 0 };
    let (mut sim, mut p, _, _) = slam_run(1500, 1, prog);
    let heading = |sim: &Sim| sim.slam_rounds()[0].1.heading as u16;
    for _ in 0..15 * 400 {
        sim.step(&p, 15);
        if sim.slam_rounds()[0].1.slam.unwrap().active {
            break;
        }
    }
    let before = heading(&sim);
    p.slam.as_mut().unwrap().steer = (127, 0);
    for _ in 0..30 {
        sim.step(&p, 15);
    }
    let turned = heading(&sim).wrapping_sub(before) as i16;
    // 10 degrees/s for 2 s: about 20 degrees (3640 BAM).
    assert!((3000..4300).contains(&turned), "turned {turned} BAM");
    assert_eq!(sim.slam_rounds()[0].1.roll, 0);
}

/// Four in flight take all the channels: a fifth stays on its pylon.
#[test]
fn slam_er_four_channels_at_most() {
    let prog = slam::SlamProgram::default();
    let (mut sim, mut p, _, _) = slam_run(1500, 5, prog);
    let mut events = Vec::new();
    for _ in 0..4 {
        for _ in 0..6 {
            events.extend(sim.step(&p, 15));
        }
        p.cmd.fire = true;
        events.extend(sim.step(&p, 15));
        p.cmd = Commands::default();
    }
    let rounds = sim.slam_rounds();
    assert_eq!(rounds.len(), 4);
    let mut chans: Vec<u8> = rounds.iter().map(|r| r.1.slam.unwrap().ch).collect();
    chans.sort();
    assert_eq!(chans, [0, 1, 2, 3]);
    assert!(events.iter().any(|e| is_msg(e, "All SLAM-ER channels in use")));
}

/// A stand-off weapon run: the jet `alt` ft up heading north at 450 kt in a sandbox, the
/// mission's primary target moved `north` map units ahead, `rounds` of `weapon` on a pylon and
/// an SA-2 site at index 3 `site` map units north of the jet (None: no site). Returns the
/// simulation, the input, the target's map point and the events of the launch press.
fn standoff_run(
    weapon: i32,
    north: i32,
    alt: u16,
    site: Option<i32>,
) -> (Sim, PlayerState, (i32, i32), Vec<SimEvent>) {
    standoff_run_after(weapon, north, alt, site, 4)
}

/// `standoff_run`, released after `warm` frames (a different roll of the dice).
fn standoff_run_after(
    weapon: i32,
    north: i32,
    alt: u16,
    site: Option<i32>,
    warm: usize,
) -> (Sim, PlayerState, (i32, i32), Vec<SimEvent>) {
    let d = data();
    let (m, mut sim) =
        aaa_mission_with(d, SimOptions { external_stores: true, ..Default::default() });
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(weapon, 2)]);
    sim.units_mut().clear();
    if let Some(n) = site {
        place_site(&mut sim, 3, 1, mx + 4, my - n);
    }
    let target = (mx + 10, my - north);
    let k = sim.target_slots()[0].object as usize;
    {
        let o = &mut sim.objects_mut()[k];
        o.x = target.0;
        o.y = target.1;
    }
    let mut p = player_at(mx, my, alt, 0, 450);
    p.hud_mode = HudMode::Gnd;
    let mut events = Vec::new();
    for _ in 0..warm {
        events.extend(sim.step(&p, 15));
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    events.extend(sim.step(&p, 15));
    p.cmd = Commands::default();
    (sim, p, target, events)
}

/// Released high within its reach, the JSOW glides (no motor: it only slows and sinks), dives
/// onto the primary target and hits it.
#[test]
fn jsow_glides_onto_its_target() {
    let (mut sim, p, target, _) = standoff_run(JSOW_WEAPON, 2500, 25000, None);
    assert_eq!(sim.standoff_rounds().len(), 1, "released");
    let (lo, hi) = sim.launch_zone().map(|z| z.zone.unwrap()).unwrap_or((0, 0));
    assert!(hi == 0 || hi > 2400, "zone {lo}..{hi}");
    let fine = ROUND_FINE as f64;
    let mut hit = None;
    let mut top = f64::MAX;
    for _ in 0..15 * 600 {
        let ev = sim.step(&p, 15);
        let Some((_, m)) = sim.standoff_rounds().into_iter().next() else {
            hit = ev.iter().find_map(|e| match *e {
                SimEvent::Explosion { x, y, kind: 0x10, .. } => Some((x, y)),
                _ => None,
            });
            break;
        };
        // Never gains energy (height and speed): it has no motor.
        let f = m.flight.unwrap();
        let v = [f.vx, f.vy, f.vz].map(|c| c as f64 / fine);
        let speed2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        let energy = f.alt as f64 / fine + speed2 / (2.0 * 32.174);
        assert!(energy <= top + 5.0, "energy {energy} up from {top}");
        top = energy;
    }
    let (x, y) = hit.expect("hit the ground with a blast");
    assert!((x - target.0).abs() <= 3 && (y - target.1).abs() <= 3, "{x},{y} vs {target:?}");
}

/// Too far for its glide from low down, the JSOW stays on its pylon.
#[test]
fn jsow_out_of_reach_stays_on_its_pylon() {
    let (sim, _, _, ev) = standoff_run(JSOW_WEAPON, 3000, 1500, None);
    assert!(sim.standoff_rounds().is_empty());
    assert!(ev.iter().any(|e| is_msg(e, "JSOW: target out of range")), "{ev:?}");
}

/// A MALD flown over an SA-2 site draws its missiles: the site fires at it (not at the jet far
/// behind) and in time knocks it down.
#[test]
fn sites_fire_at_a_mald_and_shoot_it_down() {
    let (mut sim, mut p, _, _) = standoff_run(MALD_WEAPON, 6000, 20000, Some(1600));
    assert_eq!(sim.standoff_rounds().len(), 1, "released");
    // The jet turns back south, away from the site.
    p.heading = 0x8000;
    let mut at_mald = 0;
    let mut down = false;
    for _ in 0..15 * 400 {
        for e in sim.step(&p, 15) {
            match e {
                SimEvent::MissileLaunched { slot, launcher: Launcher::Site(3), .. } => {
                    assert!(sim.projectiles()[slot].intercept.is_some(), "fired at the jet");
                    at_mald += 1;
                }
                SimEvent::WeaponShotDown { spec, .. } => {
                    assert_eq!(spec, MALD_SPEC);
                    down = true;
                }
                _ => {}
            }
        }
        if down {
            break;
        }
    }
    assert!(at_mald > 0, "the site never fired at the MALD");
    assert!(down, "{at_mald} missiles, MALD still flying");
}

/// A JSOW gliding past an SA-2 site close in is fired at and can be shot down.
#[test]
fn sites_shoot_down_some_jsows() {
    let mut downed = 0;
    let mut fired = 0;
    for k in 0..8 {
        let (mut sim, mut p, _, _) =
            standoff_run_after(JSOW_WEAPON, 2400, 25000, Some(1700 + k as i32 * 60), 4 + k * 7);
        p.heading = 0x8000;
        for _ in 0..15 * 300 {
            for e in sim.step(&p, 15) {
                match e {
                    SimEvent::MissileLaunched { launcher: Launcher::Site(3), .. } => fired += 1,
                    SimEvent::WeaponShotDown { .. } => downed += 1,
                    _ => {}
                }
            }
            if sim.standoff_rounds().is_empty() {
                break;
            }
        }
    }
    println!("{fired} missiles at JSOWs, {downed} of 8 shot down");
    assert!(fired > 0 && downed > 0, "{fired} fired, {downed} down");
}

/// A Maverick fired from 3000 ft at a target `range` map units northeast with a Tor (SA-15)
/// site at index 3 beside it: the Tor's missiles fired at the Maverick and whether it was shot
/// down.
fn maverick_at_a_tor(range: i32) -> (usize, bool) {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { realistic_guidance: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(5, 2)]);
    let t = 5;
    let diag = (range as f64 / std::f64::consts::SQRT_2).round() as i32;
    {
        let o = &mut sim.objects_mut()[t];
        o.x = mx + diag;
        o.y = my - diag;
        o.flags = 0;
    }
    place_site(&mut sim, 3, 26, mx + diag + 12, my - diag + 6);
    let mut p = player_at(mx, my, 3000, 0x2000, 450);
    p.hud_mode = HudMode::Gnd;
    p.cmd.toggle_bay = true;
    p.cmd.lock_object = Some(t as u16);
    for _ in 0..4 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    let locked = (0..96).map(|k| -k * 64).any(|pitch| {
        p.pitch = pitch as i16;
        let angle = pitch as f64 * std::f64::consts::TAU / 65536.0;
        p.climb_rate = (450.0 * 27.0 / 16.0 * angle.sin()).round() as i32;
        sim.step(&p, 15);
        sim.targeting().lock_valid
    });
    assert!(locked, "no lock from {range}");
    p.cmd.fire = true;
    let (mut fired, mut down) = (0, false);
    for _ in 0..15 * 60 {
        for e in sim.step(&p, 15) {
            match e {
                SimEvent::MissileLaunched { slot, launcher: Launcher::Site(3), .. }
                    if sim.projectiles()[slot].intercept.is_some() =>
                {
                    fired += 1
                }
                SimEvent::WeaponShotDown { spec, .. } => {
                    assert_eq!(spec, 27, "the Maverick");
                    down = true;
                }
                _ => {}
            }
        }
        p.cmd = Commands::default();
    }
    (fired, down)
}

/// The Tor defends against missiles: it fires at Mavericks coming its way and shoots some down.
#[test]
fn tor_shoots_down_some_mavericks() {
    let shots: Vec<_> = (0..6).map(|k| maverick_at_a_tor(180 + k * 25)).collect();
    println!("{shots:?}");
    let fired = shots.iter().filter(|s| s.0 > 0).count();
    let down = shots.iter().filter(|s| s.1).count();
    assert!(fired >= 4 && down > 0, "{shots:?}");
}

/// An SA-2 site does not shoot at missiles: only the Tor does.
#[test]
fn other_sites_let_mavericks_through() {
    let d = data();
    let m = mission(d, 3, 0x1234, 1);
    let opts = SimOptions { realistic_guidance: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(5, 2)]);
    let t = 5;
    {
        let o = &mut sim.objects_mut()[t];
        o.x = mx + 150;
        o.y = my - 150;
        o.flags = 0;
    }
    place_site(&mut sim, 3, 1, mx + 162, my - 144);
    let mut p = player_at(mx, my, 3000, 0x2000, 450);
    p.hud_mode = HudMode::Gnd;
    p.cmd.toggle_bay = true;
    p.cmd.lock_object = Some(t as u16);
    for _ in 0..4 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    assert!((0..96).map(|k| -k * 64).any(|pitch| {
        p.pitch = pitch as i16;
        let angle = pitch as f64 * std::f64::consts::TAU / 65536.0;
        p.climb_rate = (450.0 * 27.0 / 16.0 * angle.sin()).round() as i32;
        sim.step(&p, 15);
        sim.targeting().lock_valid
    }));
    p.cmd.fire = true;
    for _ in 0..15 * 60 {
        for e in sim.step(&p, 15) {
            if let SimEvent::MissileLaunched { slot, .. } = e {
                assert!(
                    sim.projectiles()[slot].intercept.is_none(),
                    "an SA-2 fired at the Maverick"
                );
            }
        }
        p.cmd = Commands::default();
    }
}

/// A paratroop drop: the C-5 at `ft` over a sandbox at 130 kt heading north, its ramp down, two
/// sticks aboard, the mission's primary target moved `north` map units ahead as the drop zone.
/// Returns the simulation, the input, the drop zone and the events of the green light.
fn para_run(ft: u16, north: i32) -> (Sim, PlayerState, (i32, i32), Vec<SimEvent>) {
    let d = data();
    let (m, mut sim) = aaa_mission_with(d, SimOptions { galaxy: true, ..Default::default() });
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(paradrop::PARA_WEAPON, 2)]);
    sim.units_mut().clear();
    let dz = (mx, my - north);
    let k = sim.target_slots()[0].object as usize;
    {
        let o = &mut sim.objects_mut()[k];
        o.x = dz.0;
        o.y = dz.1;
    }
    let id = sim.active_pilot();
    sim.set_pilot_target(id, Some((paradrop::PARA_MISSION, k)));
    let mut p = player_at(mx, my, ft, 0, 130);
    p.cmd.toggle_bay = true;
    for _ in 0..15 * 12 {
        sim.step(&p, 15);
        p.cmd = Commands::default();
    }
    p.cmd.fire = true;
    let ev = sim.step(&p, 15);
    p.cmd = Commands::default();
    (sim, p, dz, ev)
}

/// Over the drop zone, inside the limits: the stick jumps, the chutes open, they come down in
/// the zone and take it.
#[test]
fn paratroopers_dropped_on_the_zone_take_it() {
    let (mut sim, p, _, ev) = para_run(1000, 0);
    assert!(ev.iter().any(|e| is_msg(e, "Green light: go, go, go!")), "{ev:?}");
    let mut done = false;
    let mut opened = false;
    for _ in 0..15 * 240 {
        for e in sim.step(&p, 15) {
            done |= matches!(e, SimEvent::ObjectiveDone { primary: true });
        }
        opened |= sim.paradrop().jumpers.iter().any(|j| j.open() && j.landed.is_none());
        if done {
            break;
        }
    }
    assert!(opened, "no chute opened");
    assert_eq!(sim.paradrop().jumpers.len(), paradrop::STICK_C5 as usize);
    assert!(
        done,
        "zone not taken: {:?}",
        sim.paradrop().jumpers.iter().filter(|j| j.landed.is_some()).count()
    );
}

/// Too high: the stick stays aboard.
#[test]
fn paratroopers_stay_aboard_outside_the_limits() {
    let (sim, _, _, ev) = para_run(3000, 0);
    assert!(
        ev.iter().any(|e| is_msg(e, "Jump limits: 500 to 1500 ft over the ground, below 150 kt")),
        "{ev:?}"
    );
    assert!(sim.paradrop().jumpers.is_empty());
}

/// The CH-47's stick: its paratroopers are selected (not a gun), and jump in NAV mode too.
#[test]
fn the_chinook_drops_its_stick() {
    let d = data();
    let opts = SimOptions { helicopter: true, chinook: true, ..Default::default() };
    let (m, mut sim) = aaa_mission_with(d, opts);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_loadout(&[(paradrop::PARA_WEAPON, 1)]);
    sim.select_bay(0);
    assert!(!sim.gun_selected());
    let mut p = player_at(mx, my, 1000, 0, 110);
    p.mode_weapons = true;
    sim.step(&p, 15);
    p.cmd.fire = true;
    let ev = sim.step(&p, 15);
    assert!(ev.iter().any(|e| is_msg(e, "Green light: go, go, go!")), "{ev:?}");
    p.cmd = Commands::default();
    for _ in 0..15 * 30 {
        sim.step(&p, 15);
    }
    assert_eq!(sim.paradrop().jumpers.len(), paradrop::STICK_CH47 as usize);
}

/// A Growler with its three pods 19 km from an SA-2: with its transmitters off the site finds
/// it and fires; jamming the site, it does not (it sees the Growler only inside its
/// burn-through range), though the Growler's noise raises its EMV.
#[test]
fn a_growler_jamming_a_site_keeps_it_blind() {
    let run = |jam: bool| {
        let d = data();
        let m = mission(d, 3, 0x1234, 1);
        let opts =
            SimOptions { external_stores: true, hornet: true, growler: true, ..Default::default() };
        let mut sim = Sim::with_options(&d.sim, &m, opts);
        let (mx, my) = sandbox(&mut sim, &m);
        sim.set_loadout(&[
            (jamming::ALQ99_WEAPON, 1),
            (jamming::ALQ99_WEAPON, 1),
            (jamming::ALQ99_WEAPON, 1),
        ]);
        place_site(&mut sim, 3, 1, mx, my - 1200);
        let mut p = player_at(mx, my, 6000, 0x4000, 200);
        p.jam = Some(jamming::JamInput {
            on: jam,
            pods: [Some(jamming::JamTarget::Site(3)), None, None],
        });
        let mut launched = false;
        for _ in 0..15 * 120 {
            for e in sim.step(&p, 15) {
                launched |=
                    matches!(e, SimEvent::MissileLaunched { launcher: Launcher::Site(3), .. });
            }
        }
        (launched, sim.emv(), sim.site_jammed(3))
    };
    let (open, quiet_emv, _) = run(false);
    let (jammed, loud_emv, on) = run(true);
    assert!(open, "the site never fired at the Growler with its pods off");
    assert!(on && !jammed, "the jammed site fired");
    assert!(loud_emv > quiet_emv, "emv {quiet_emv} -> {loud_emv}");
}

/// One step of a JP233 run: the aircraft flies on north at its speed (the test's input does
/// not move it by itself).
fn jp233_step(sim: &mut Sim, p: &mut PlayerState) -> Vec<SimEvent> {
    let ev = sim.step(p, 15);
    p.y += (p.velocity as i32 / 16) / 15;
    ev
}

/// A JP233 run: the Tornado `alt` ft up at 450 kt heading north over a sandbox, both
/// dispensers loaded, `units` placed ahead (map offsets from the aircraft), the fire button
/// pressed `presses` times. Returns the simulation, the input and the events of the presses.
fn jp233_run(
    alt: u16,
    units: &[(aaa::GroundKind, i32, i32)],
    presses: usize,
) -> (Sim, PlayerState, (i32, i32), Vec<SimEvent>) {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(jp233::JP233_WEAPON, 1), (jp233::JP233_WEAPON, 1)]);
    *sim.units_mut() = units
        .iter()
        .map(|&(kind, dx, dy)| aaa::GroundUnit {
            kind,
            x: mx + dx,
            y: my + dy,
            ..Default::default()
        })
        .collect();
    let mut p = player_at(mx, my, alt, 0, 450);
    p.hud_mode = HudMode::Gnd;
    jp233_step(&mut sim, &mut p);
    let mut events = Vec::new();
    for _ in 0..presses {
        p.cmd.fire = true;
        events.extend(jp233_step(&mut sim, &mut p));
        p.cmd = Commands::default();
        for _ in 0..5 {
            events.extend(jp233_step(&mut sim, &mut p));
        }
    }
    (sim, p, (mx, my), events)
}

/// Low over a column of vehicles: both dispensers lay their bomblets and mines along the
/// strip behind the aircraft, the vehicles under it are destroyed, mines are left lying, and
/// the empty dispensers are let go.
#[test]
fn jp233_lays_its_strip_over_a_column() {
    use aaa::GroundKind;
    // A column along the track some 200 to 700 m ahead (a map unit is 15.6 m).
    let column: Vec<_> = (0..8)
        .map(|k| (if k % 2 == 0 { GroundKind::Bmp2 } else { GroundKind::T90 }, 0, -13 - 4 * k))
        .collect();
    let (mut sim, mut p, _, ev) = jp233_run(200, &column, 2);
    assert!(ev.iter().any(|e| is_msg(e, "JP233 dispensing")), "{ev:?}");
    assert_eq!(sim.jp233().runs.len(), 2, "both dispensers at work");
    let mut most = 0;
    let mut craters = 0;
    let mut mine_bursts = 0;
    for _ in 0..15 * 20 {
        for e in jp233_step(&mut sim, &mut p) {
            if let SimEvent::Jp233Burst { crater, .. } = e {
                if crater { craters += 1 } else { mine_bursts += 1 }
            }
        }
        most = most.max(sim.jp233().bomblets.len());
    }
    assert!(sim.jp233().runs.is_empty());
    assert!(sim.jp233().bomblets.is_empty(), "all down");
    assert!(most > 100, "{most} in the air at once");
    assert_eq!(craters, 2 * jp233::CRATERING as usize);
    assert!(
        sim.jp233().mines.len() + mine_bursts >= 2 * jp233::MINES as usize - 2,
        "{} lying, {mine_bursts} gone off",
        sim.jp233().mines.len()
    );
    assert!(sim.bays().0.iter().all(|b| b.1 == 0), "dispensers let go: {:?}", sim.bays().0);
    let dead = sim.units().iter().filter(|u| u.destroyed).count();
    assert!(dead >= 4, "{dead} of 8 destroyed");
}

/// The mines wait: a vehicle driving into the minefield later sets one off.
#[test]
fn jp233_mines_wait_for_vehicles() {
    use aaa::GroundKind;
    let (mut sim, p, (mx, my), _) = jp233_run(200, &[], 1);
    for _ in 0..15 * 10 {
        sim.step(&p, 15);
    }
    let lying = sim.jp233().mines.len();
    assert!(lying > 20, "{lying} mines lying");
    // A truck parked on a mine.
    let m = sim.jp233().mines[lying / 2];
    let (e, n) = (m.p[0] as f64, m.p[1] as f64);
    let at = ((e / aaa::M_PER_MAP).round() as i32, 0x8000 - (n / aaa::M_PER_MAP).round() as i32);
    assert!((at.0 - mx).abs() < 20 && (my - at.1) > 0, "mine at {at:?} from {:?}", (mx, my));
    sim.units_mut().push(aaa::GroundUnit {
        kind: GroundKind::Bmp2,
        x: at.0,
        y: at.1,
        fine: Some(aaa::Fine { e: m.p[0] + 3.0, n: m.p[1], heading: 0.0 }),
        ..Default::default()
    });
    let mut went_off = false;
    for _ in 0..15 {
        for e in sim.step(&p, 15) {
            went_off |= matches!(e, SimEvent::Jp233Burst { crater: false, .. });
        }
    }
    assert!(went_off, "no mine went off under the vehicle");
    assert!(sim.jp233().mines.len() < lying);
}

/// Too high over the ground: it refuses.
#[test]
fn jp233_refuses_from_too_high() {
    let (sim, _, _, ev) = jp233_run(3000, &[], 1);
    assert!(
        ev.iter().any(|e| is_msg(e, "JP233: too high (above 1000 ft over the ground)")
            || is_msg(e, "JP233: too high (above 1,000 ft over the ground)")),
        "{ev:?}"
    );
    assert!(sim.jp233().runs.is_empty() && sim.jp233().bomblets.is_empty());
    assert!(sim.bays().0.iter().all(|b| b.1 == 1));
}

/// Along a runway: its craters put it out of action.
#[test]
fn jp233_craters_close_a_runway() {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(jp233::JP233_WEAPON, 1)]);
    sim.units_mut().clear();
    let metres = |o: &GroundObject| {
        [(o.x as f64 + 0.5) * aaa::M_PER_MAP, ((0x8000 - o.y) as f64 - 0.5) * aaa::M_PER_MAP]
    };
    // A runway with its runway object.
    let (r, k) = sim
        .runways()
        .iter()
        .find_map(|r| {
            let k = (1..sim.objects().len()).find(|&k| {
                let at = metres(&sim.objects()[k]);
                sim.object_category(k) == 1 && (at[0] - r.c.0).hypot(at[1] - r.c.1) < r.half_len
            })?;
            Some((*r, k))
        })
        .expect("a runway with its object");
    // In along it from 600 m short of its middle, 200 ft up.
    let start = [r.c.0 - r.axis.0 * 600.0, r.c.1 - r.axis.1 * 600.0];
    let wu = |m: f64| (m / (aaa::M_PER_MAP / 32.0)).round() as i32;
    let h = r.axis.0.atan2(r.axis.1);
    let mut p = player_at(0, 0, 200, (h / std::f64::consts::TAU * 65536.0) as i32 as u16, 450);
    (p.x, p.y) = (wu(start[0]), wu(start[1]));
    p.hud_mode = HudMode::Gnd;
    let step = |sim: &mut Sim, p: &mut PlayerState| {
        let ev = sim.step(p, 15);
        let d = (p.velocity as f64 / 16.0) / 15.0;
        p.x += (h.sin() * d).round() as i32;
        p.y += (h.cos() * d).round() as i32;
        ev
    };
    step(&mut sim, &mut p);
    p.cmd.fire = true;
    step(&mut sim, &mut p);
    p.cmd = Commands::default();
    let mut destroyed = false;
    for _ in 0..15 * 20 {
        for e in step(&mut sim, &mut p) {
            destroyed |= matches!(e, SimEvent::GroundDestroyed { object, .. } if object == k);
        }
    }
    assert!(destroyed, "runway {k} not cratered");
}

/// A rocket test: hovering `alt` ft up heading north over a sandbox with two pods of weapon
/// `w`, `units` placed (map offsets from the aircraft). Returns the simulation, the input and
/// the aircraft's map position.
fn rocket_range(
    w: i32,
    alt: u16,
    units: &[(aaa::GroundKind, i32, i32)],
) -> (Sim, PlayerState, (i32, i32)) {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    sim.set_training(true);
    sim.set_loadout(&[(w, 7), (w, 7)]);
    *sim.units_mut() = units
        .iter()
        .map(|&(kind, dx, dy)| aaa::GroundUnit {
            kind,
            x: mx + dx,
            y: my + dy,
            ..Default::default()
        })
        .collect();
    let mut p = player_at(mx, my, alt, 0, 0);
    p.hud_mode = HudMode::Gnd;
    sim.step(&p, 15);
    (sim, p, (mx, my))
}

/// Steps until the rockets in flight are down: the bursts (m east, north) and the events.
fn rockets_down(sim: &mut Sim, p: &mut PlayerState) -> (Vec<[f64; 2]>, Vec<SimEvent>) {
    let mut bursts = Vec::new();
    let mut events = Vec::new();
    for _ in 0..15 * 30 {
        for e in sim.step(p, 15) {
            if let SimEvent::RocketBurst { at, .. } = e {
                let f = stealth_sim::ROUND_FINE as f64 * 32.0;
                let m = aaa::M_PER_MAP;
                bursts.push([at.0 as f64 / f * m, (0x8000 as f64 - at.1 as f64 / f) * m]);
            }
            events.push(e);
        }
        if sim.rockets().is_empty() && !bursts.is_empty() {
            break;
        }
    }
    (bursts, events)
}

/// A press fires a pair of unguided rockets, one off each pod, and they come down by the CCIP
/// reticle's point (in a 10 degree dive: at a grazing angle their scatter spreads far along).
#[test]
fn hydra_pair_lands_at_the_ccip_point() {
    use stealth_sim::rockets;
    let (mut sim, mut p, _) = rocket_range(rockets::HYDRA_WEAPON, 1000, &[]);
    p.pitch = -0x0700;
    sim.step(&p, 15);
    let (ccip, tof) = sim.rocket_ccip().expect("a CCIP point");
    assert!(tof > 1.0, "time of flight {tof}");
    p.cmd.fire = true;
    p.cmd.trigger = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    assert_eq!(sim.bays().0.iter().map(|b| b.1).collect::<Vec<_>>(), vec![6, 6], "a pair");
    let (bursts, _) = rockets_down(&mut sim, &mut p);
    assert_eq!(bursts.len(), 2);
    for b in bursts {
        let miss = (b[0] - ccip[0]).hypot(b[1] - ccip[1]);
        assert!(miss < 60.0, "{miss:.0} m from the CCIP point {ccip:?}: {b:?}");
    }
}

/// Holding the trigger ripples a pair every quarter second until it is let go.
#[test]
fn hydra_ripples_while_the_trigger_is_held() {
    use stealth_sim::rockets;
    let (mut sim, mut p, _) = rocket_range(rockets::HYDRA_WEAPON, 300, &[]);
    p.cmd.fire = true;
    p.cmd.trigger = true;
    sim.step(&p, 15);
    p.cmd.fire = false;
    // Held for a second: three more pairs.
    for _ in 0..15 {
        sim.step(&p, 15);
    }
    p.cmd = Commands::default();
    for _ in 0..15 {
        sim.step(&p, 15);
    }
    let left: i32 = sim.bays().0.iter().map(|b| b.1).sum();
    assert_eq!(left, 14 - 8, "{:?}", sim.bays().0);
}

/// An APKWS fired along the nose turns onto the targeting pod's laser spot on a vehicle off
/// to the side and destroys it.
#[test]
fn apkws_homes_on_the_laser_spot() {
    use aaa::GroundKind;
    use stealth_sim::rockets;
    // A vehicle 1 km ahead, 150 m to the right.
    let (mut sim, mut p, (mx, my)) =
        rocket_range(rockets::APKWS_WEAPON, 300, &[(GroundKind::Bmp2, 10, -64)]);
    p.pod = Some(PodAim { heading: 0, pitch: 0, ground: Some((mx + 10, my - 64)) });
    p.cmd.fire = true;
    sim.step(&p, 15);
    p.cmd = Commands::default();
    let (bursts, events) = rockets_down(&mut sim, &mut p);
    assert_eq!(bursts.len(), 2);
    let [e, n] = stealth_sim::asw::map_metres((mx + 10, my - 64));
    for b in &bursts {
        let miss = (b[0] - e).hypot(b[1] - n);
        assert!(miss < 6.0, "{miss:.1} m off the spot");
    }
    assert!(sim.units()[0].destroyed, "{events:?}");
}

/// One step of a wingman test: the lead flies on at its speed along its heading (the test's
/// input does not move it by itself).
fn wing_step(sim: &mut Sim, p: &mut PlayerState) -> Vec<SimEvent> {
    let ev = sim.step(p, 15);
    let h = p.heading as f64 * std::f64::consts::TAU / 65536.0;
    let d = (p.velocity as f64 / 16.0) / 15.0;
    p.x += (h.sin() * d).round() as i32;
    p.y += (h.cos() * d).round() as i32;
    ev
}

/// The wingman's distance from its lead (m) along his track (+ ahead) and to his right.
fn wing_offset(sim: &Sim, p: &PlayerState) -> (f64, f64) {
    let i = sim.wingman_of(sim.active_pilot()).expect("a wingman");
    let u = sim.flights()[i];
    let (e, s) = (u.wx as f64 - p.x as f64, u.wy as f64 - (0x10_0000 - p.y) as f64);
    let h = p.heading as f64 * std::f64::consts::TAU / 65536.0;
    let m = aaa::M_PER_MAP / 32.0;
    ((e * h.sin() - s * h.cos()) * m, (e * h.cos() + s * h.sin()) * m)
}

/// A wingman (an F-16) joins its lead once he is up and flies spread on his right; told
/// to go trail it drops back behind him; told home it leaves him.
#[test]
fn a_wingman_joins_and_flies_formation() {
    use stealth_sim::wingman::{WingCmd, WingReply};
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::ai_missions::F16);
    let mut p = player_at(mx, my, 5000, 0x4000, 400);
    assert!(sim.wingman_pending(me));
    wing_step(&mut sim, &mut p);
    assert!(sim.wingman_of(me).is_some(), "joined");
    for _ in 0..15 * 40 {
        wing_step(&mut sim, &mut p);
    }
    let (along, right) = wing_offset(&sim, &p);
    assert!(
        (-300.0..0.0).contains(&along) && (300.0..500.0).contains(&right),
        "spread {along:.0} {right:.0}"
    );
    assert_eq!(sim.wing_command(me, WingCmd::Trail), WingReply::Copy);
    for _ in 0..15 * 60 {
        wing_step(&mut sim, &mut p);
    }
    let (along, right) = wing_offset(&sim, &p);
    assert!(
        (-1400.0..-1000.0).contains(&along) && right.abs() < 200.0,
        "trail {along:.0} {right:.0}"
    );
    // Nothing locked: nothing to engage; no HARM: no SAMs.
    assert_eq!(sim.wing_command(me, WingCmd::Engage), WingReply::NoTarget);
    assert_eq!(sim.wing_command(me, WingCmd::Sams), WingReply::NoHarm);
    assert_eq!(sim.wing_command(me, WingCmd::Rtb), WingReply::Copy);
    for _ in 0..15 * 30 {
        wing_step(&mut sim, &mut p);
    }
    let (along, right) = wing_offset(&sim, &p);
    assert!(along.hypot(right) > 3000.0, "still with him: {along:.0} {right:.0}");
}

/// A wingman with HARMs fires one at the SAM site its lead calls it on, then rejoins.
#[test]
fn a_wingman_fires_a_harm_at_a_sam_site() {
    use stealth_sim::wingman::{WingCmd, WingReply};
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    // A SAM site 25 km ahead (east).
    place_site(&mut sim, 5, 4, mx + 1600, my);
    sim.set_loadout(&[(2, 1), (2, 1), (0, 1), (0, 1)]);
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::ai_missions::F16);
    let mut p = player_at(mx, my, 8000, 0x4000, 400);
    for _ in 0..15 * 10 {
        wing_step(&mut sim, &mut p);
    }
    assert_eq!(sim.wing(me).map(|w| w.harms), Some(2));
    assert_eq!(sim.wing_command(me, WingCmd::Sams), WingReply::Engaging);
    let mut fired = false;
    for _ in 0..15 * 60 {
        wing_step(&mut sim, &mut p);
        fired |= sim.wing(me).is_some_and(|w| w.harms == 1);
        if fired {
            break;
        }
    }
    assert!(fired, "no HARM fired: {:?}", sim.wing(me));
    assert_eq!(sim.wing(me).map(|w| w.order), Some(stealth_sim::wingman::WingOrder::Formation));
}

/// A helicopter's wingman (an AH-64) keeps its closer station on its lead.
#[test]
fn a_helicopter_wingman_flies_close_formation() {
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::ai_missions::AH64);
    let mut p = player_at(mx, my, 300, 0x4000, 80);
    for _ in 0..15 * 60 {
        wing_step(&mut sim, &mut p);
    }
    let (along, right) = wing_offset(&sim, &p);
    assert!(
        (-100.0..10.0).contains(&along) && (90.0..150.0).contains(&right),
        "spread {along:.0} {right:.0}"
    );
}

/// A sandbox that keeps its airfields (none friendly: stopping on one ends no mission), its
/// lead taking off and landing judged by no one.
fn runway_sandbox(d: &Data) -> Sim {
    let opts = SimOptions { landing: LandingMode::NoCrashes, ..Default::default() };
    let (m, mut sim) = aaa_mission_with(d, opts);
    let flags: Vec<i32> = sim.objects().iter().map(|o| o.flags).collect();
    sandbox(&mut sim, &m);
    sim.units_mut().clear();
    for (o, f) in sim.objects_mut().iter_mut().zip(flags) {
        o.flags |= f & oflags::AIRFIELD;
    }
    sim
}

/// A runway of the sandbox's theater, and a lead on its centreline `along` m from its middle
/// (negative: towards the end it is taken off from), heading along it, at rest with the gear
/// down.
fn lead_on_runway(sim: &Sim, along: f64) -> (Runway, PlayerState) {
    let r = sim.runways()[0];
    let wu = |m: f64| (m / (aaa::M_PER_MAP / 32.0)).round() as i32;
    let h = r.axis.0.atan2(r.axis.1);
    let mut p = player_at(0, 0, 0, (h / std::f64::consts::TAU * 65536.0) as i32 as u16, 0);
    (p.x, p.y) = (wu(r.c.0 + r.axis.0 * along), wu(r.c.1 + r.axis.1 * along));
    p.on_ground = true;
    p.gear_up = false;
    (r, p)
}

/// Flight unit `i`'s place on runway `r` (m along from its middle, across).
fn on_runway(sim: &Sim, i: usize, r: &Runway) -> (f64, f64) {
    let u = sim.flights()[i];
    let m = aaa::M_PER_MAP / 32.0;
    let (e, n) = (u.wx as f64 * m, (0x10_0000 - u.wy) as f64 * m);
    let (dx, dy) = (e - r.c.0, n - r.c.1);
    (dx * r.axis.0 + dy * r.axis.1, -dx * r.axis.1 + dy * r.axis.0)
}

/// The lead flies a 3° approach to `touchdown` m along runway `r` from its middle, lands and
/// brakes at `brake` kt/s to a stop, then waits 30 s. Checks the wingman on the way: never
/// below the runway, on it once down.
fn land_on(sim: &mut Sim, p: &mut PlayerState, r: &Runway, touchdown: f64, brake: u16) {
    let me = sim.active_pilot();
    let m = aaa::M_PER_MAP / 32.0;
    let mut stopped = 0;
    for _ in 0..15 * 300 {
        wing_step(sim, p);
        let (e, n) = (p.x as f64 * m, p.y as f64 * m);
        let along = (e - r.c.0) * r.axis.0 + (n - r.c.1) * r.axis.1;
        let to_go = touchdown - along;
        if !p.on_ground {
            let ft = (to_go.max(0.0) * 0.0524 * 3.28) as u16;
            (p.altitude, p.height) = (ft, ft);
            p.on_ground = ft == 0;
            p.climb_rate = if p.on_ground { 0 } else { -12 };
        } else {
            p.velocity = p.velocity.saturating_sub(brake * 27 / 15);
        }
        if let Some(i) = sim.wingman_of(me) {
            let u = sim.flights()[i];
            assert_eq!(u.flags & fflags::DESTROYED, 0, "crashed");
            assert!(u.alt >= 12, "under the ground: {}", u.alt);
            if u.alt < 20 {
                let (a, x) = on_runway(sim, i, r);
                assert!(
                    a.abs() < r.half_len && x.abs() < r.half_width,
                    "off the runway {a:.0} {x:.0}"
                );
            }
        }
        if p.velocity == 0 {
            stopped += 1;
            if stopped > 15 * 30 {
                return;
            }
        }
    }
    panic!("never stopped");
}

/// A wingman starts on the runway beside its lead, rolls and lifts off with him, and flies
/// out to its station once they are up.
#[test]
fn a_wingman_takes_off_beside_its_lead() {
    use stealth_sim::wingman::WingPhase;
    let d = data();
    let mut sim = runway_sandbox(d);
    let (r, mut p) = lead_on_runway(&sim, -800.0);
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::ai_missions::F16);
    wing_step(&mut sim, &mut p);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Takeoff));
    let (along, right) = wing_offset(&sim, &p);
    assert!(
        (-15.0..-5.0).contains(&along) && (15.0..25.0).contains(&right),
        "{along:.0} {right:.0}"
    );
    let i = sim.wingman_of(me).unwrap();
    assert_eq!(sim.flights()[i].alt, 12);
    // Waiting a minute on the runway: it waits with him.
    for _ in 0..15 * 60 {
        wing_step(&mut sim, &mut p);
    }
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Takeoff));
    // The roll: 8 kt/s to 150 kt, then up at 20 ft/s.
    let mut up = None;
    for f in 0..15 * 90 {
        if p.on_ground {
            p.velocity += 8 * 27 / 15;
            p.on_ground = p.velocity < 150 * 27;
        } else {
            p.altitude += 2;
            p.height = p.altitude;
            p.gear_up = p.height > 100;
        }
        wing_step(&mut sim, &mut p);
        let Some(i) = sim.wingman_of(me) else { panic!("gone") };
        if sim.wing(me).map(|w| w.phase) == Some(WingPhase::Takeoff) {
            let (along, right) = wing_offset(&sim, &p);
            assert!(
                along.abs() < 20.0 && (15.0..25.0).contains(&right),
                "{f}: {along:.0} {right:.0}"
            );
            if sim.flights()[i].alt < 20 {
                let (a, x) = on_runway(&sim, i, &r);
                assert!(
                    a.abs() < r.half_len && x.abs() < r.half_width,
                    "off the runway {a:.0} {x:.0}"
                );
            }
        } else if up.is_none() {
            up = Some(p.height);
        }
    }
    assert!(up.is_some_and(|h| (300..400).contains(&h)), "up at {up:?} ft");
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Air));
    p.altitude = 3000;
    p.height = 3000;
    p.velocity = 400 * 27;
    for _ in 0..15 * 60 {
        wing_step(&mut sim, &mut p);
    }
    let (along, right) = wing_offset(&sim, &p);
    assert!(
        (-300.0..0.0).contains(&along) && (300.0..500.0).contains(&right),
        "spread {along:.0} {right:.0}"
    );
}

/// A wingman lands beside its lead on a runway wide enough for two and stops beside him.
#[test]
fn a_wingman_lands_beside_its_lead() {
    use stealth_sim::wingman::WingPhase;
    let d = data();
    let mut sim = runway_sandbox(d);
    let (r, mut p) = lead_on_runway(&sim, -r_len(&sim) - 6000.0);
    (p.on_ground, p.velocity, p.altitude, p.height) = (false, 150 * 27, 1000, 1000);
    p.climb_rate = -12;
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::ai_missions::F16);
    wing_step(&mut sim, &mut p);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Landing));
    land_on(&mut sim, &mut p, &r, -r.half_len + 250.0, 8);
    assert_eq!(sim.mission_end(), None);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Parked));
    let i = sim.wingman_of(me).expect("still there");
    assert_eq!(sim.flights()[i].speed, 0);
    let (along, right) = wing_offset(&sim, &p);
    assert!(along.abs() < 80.0 && (15.0..25.0).contains(&right), "{along:.0} {right:.0}");
}

fn r_len(sim: &Sim) -> f64 {
    sim.runways()[0].half_len
}

/// A C-5's wingman (no room for two on the runway) lands behind its lead and stops short of
/// him.
#[test]
fn a_big_wingman_lands_behind_its_lead() {
    use stealth_sim::wingman::WingPhase;
    let d = data();
    let mut sim = runway_sandbox(d);
    let (r, mut p) = lead_on_runway(&sim, -r_len(&sim) - 6000.0);
    (p.on_ground, p.velocity, p.altitude, p.height) = (false, 140 * 27, 1000, 1000);
    p.climb_rate = -12;
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::munitions::C5);
    wing_step(&mut sim, &mut p);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Landing));
    land_on(&mut sim, &mut p, &r, -r.half_len + 200.0, 8);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Parked));
    let (along, right) = wing_offset(&sim, &p);
    assert!((-600.0..-85.0).contains(&along) && right.abs() < 10.0, "{along:.0} {right:.0}");
}

/// A C-5's wingman takes off behind its lead on his track.
#[test]
fn a_big_wingman_takes_off_behind_its_lead() {
    use stealth_sim::wingman::WingPhase;
    let d = data();
    let mut sim = runway_sandbox(d);
    let (_, mut p) = lead_on_runway(&sim, -700.0);
    let me = sim.active_pilot();
    sim.request_wingman(me, stealth_sim::munitions::C5);
    wing_step(&mut sim, &mut p);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Takeoff));
    let (along, right) = wing_offset(&sim, &p);
    assert!((-160.0..-140.0).contains(&along) && right.abs() < 1.0, "{along:.0} {right:.0}");
}

/// A helicopter's wingman starts beside its lead, lifts off with him, and sets down beside
/// him when he lands.
#[test]
fn a_helicopter_wingman_lifts_off_and_lands_with_its_lead() {
    use stealth_sim::wingman::WingPhase;
    let d = data();
    let (m, mut sim) = aaa_mission(d);
    let (mx, my) = sandbox(&mut sim, &m);
    let me = sim.active_pilot();
    let mut p = player_at(mx, my, 0, 0x4000, 0);
    p.on_ground = true;
    sim.request_wingman(me, stealth_sim::ai_missions::AH64);
    wing_step(&mut sim, &mut p);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Takeoff));
    let (along, right) = wing_offset(&sim, &p);
    assert!(along.abs() < 2.0 && (15.0..30.0).contains(&right), "{along:.0} {right:.0}");
    p.on_ground = false;
    for _ in 0..15 * 10 {
        p.altitude += 1;
        p.height = p.altitude;
        wing_step(&mut sim, &mut p);
    }
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Air));
    p.velocity = 80 * 27;
    for _ in 0..15 * 30 {
        wing_step(&mut sim, &mut p);
    }
    // Down again: slowing, then setting down.
    for _ in 0..15 * 20 {
        p.velocity = p.velocity.saturating_sub(27 * 6 / 15);
        p.altitude = p.altitude.saturating_sub(1).max(if p.velocity > 0 { 60 } else { 0 });
        p.height = p.altitude;
        p.on_ground = p.altitude == 0;
        p.climb_rate = if p.on_ground { 0 } else { -15 };
        wing_step(&mut sim, &mut p);
    }
    assert!(p.on_ground);
    assert_eq!(sim.wing(me).map(|w| w.phase), Some(WingPhase::Landing));
    let i = sim.wingman_of(me).unwrap();
    assert!(sim.flights()[i].alt <= 14, "in the air at {}", sim.flights()[i].alt);
    let (along, right) = wing_offset(&sim, &p);
    assert!(along.abs() < 10.0 && (15.0..30.0).contains(&right), "{along:.0} {right:.0}");
}
