//! Behaviour pinned across refactors of the simulation's layout: a digest of what the player
//! sees every frame (events, cockpit state, projectiles, aircraft, world) for scripted missions
//! and a campaign sortie. Unlike `classic_state_golden` it does not hash the serialized state,
//! so moving fields around keeps it; changing what happens does not.
//!
//! `PARITY_OUT=<dir>` writes the traces for diffing.

use std::fmt::Write;
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

/// An event as text, without the fields added for several pilots (`by`, the launcher's pilot),
/// a HUD message in English as it was written before the translations.
fn event_text(e: &SimEvent) -> String {
    if let SimEvent::Message(t) = e {
        return format!("Message({:?})", t.tr_in(stealth_i18n::Language::En));
    }
    format!("{e:?}")
        .replace(", by: None", "")
        .replace(", by: Some(0)", "")
        .replace("Player(0)", "Player")
        .replace(", heavy: false", "")
}

fn fnv(h: u64, s: &str) -> u64 {
    s.bytes().fold(h, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

/// One frame's digest line.
fn frame_line(sim: &Sim, events: &[SimEvent]) -> String {
    let mut s = String::new();
    for e in events.iter().filter(|e| !matches!(e, SimEvent::AaaDestroyed { .. })) {
        let _ = write!(s, "{};", event_text(e));
    }
    let _ = write!(
        s,
        "|p {:?} emv {} dmg {:?} l {:?} t {:?} st {:?} gun {} {:?} abs {} ch {:?} ob {:?}",
        sim.player(),
        sim.emv(),
        sim.damage_effects(),
        sim.lights(),
        sim.targeting(),
        sim.stores(),
        sim.gun_ammo(),
        sim.bays(),
        sim.player_absent(),
        sim.chute(),
        sim.objectives(),
    );
    let _ =
        write!(s, "|em {:?} sb {:?} arc {:?}", sim.emitters(), sim.signal_bars(), sim.radar_arc());
    for (k, m, _, _) in sim.active_projectiles() {
        let _ = write!(
            s,
            "|m{k} {} {} {} {} {} {} {} {} {:?} {:?}",
            m.x, m.y, m.alt, m.speed, m.heading, m.ttl, m.spec, m.target, m.fall, m.flight
        );
    }
    let (b, nb) = sim.bullets();
    for (k, r) in b.iter().enumerate().filter(|(_, r)| r.x != 0) {
        let tag = if k < nb { format!("p{k}") } else { format!("e{}", k - nb) };
        let _ = write!(s, "|b{tag} {} {} {}", r.x, r.y, r.alt);
    }
    for c in sim.countermeasures() {
        let _ = write!(s, "|c {:?}", c);
    }
    let mut h = 0xcbf2_9ce4_8422_2325;
    for (i, u, _) in sim.active_aircraft() {
        h = fnv(h, &format!("{i} {} {} {} {} {} {}", u.x, u.y, u.alt, u.heading, u.flags, u.speed));
    }
    for (i, o) in sim.objects().iter().enumerate().skip(1) {
        h = fnv(h, &format!("{i} {} {} {}", o.flags, o.alert, o.unit_type));
    }
    for u in sim.units() {
        // Without the Patriot launchers' ready rounds, the vehicles' fine positions and the
        // gone flag (added later; always 0, None and false here), under the types' names from
        // before they took armour.
        let line = format!("{u:?}")
            .replace(", rounds: 0", "")
            .replace(", fine: None", "")
            .replace(", gone: false", "");
        h = fnv(h, &line.replace("GroundUnit", "AaaUnit").replace("GroundKind", "AaaKind"));
    }
    let _ = write!(
        s,
        "|w {h:016x} {:?} {:?} {:?}",
        sim.wreck(),
        sim.alarm_state(),
        sim.identification()
    );
    s
}

/// A scripted flight: climb out towards the primary, then the secondary, with random commands.
fn scripted(sim: &mut Sim, m: &Mission, seed: u32, frames: usize, trace: &mut String) {
    let trig = &data().sim.trig;
    let mut p = sim.start_state();
    let mut rng = Rng::new(seed);
    let (mut wx, mut wy) = (p.x as i64, p.y as i64);
    let mut alt = 0i32;
    for f in 0..frames {
        let wp = m.waypoints[if f < frames / 2 { 1 } else { 2 }];
        let (tx, ty) = Sim::map_to_world(wp.x, wp.y);
        let h = stealth_sim::math::bearing(
            ((tx as i64 - wx) >> 6) as i32,
            ((ty as i64 - wy) >> 6) as i32,
        );
        p.heading = h as u16;
        alt = (alt + 20).min(if f < frames * 2 / 3 { 3000 } else { 300 });
        p.altitude = alt as u16;
        p.height = alt as u16;
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
            trigger: r % 11 == 0,
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
        let ev = sim.step(&p, 15);
        trace.push_str(&frame_line(sim, &ev));
        trace.push('\n');
        if sim.mission_end().is_some() || sim.player_absent() {
            break;
        }
    }
}

fn check(name: &str, trace: &str) -> u64 {
    if let Ok(dir) = std::env::var("PARITY_OUT") {
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(std::path::Path::new(&dir).join(format!("{name}.txt")), trace).unwrap();
    }
    fnv(0xcbf2_9ce4_8422_2325, trace)
}

fn missions() -> Vec<u64> {
    let d = data();
    let mut got = Vec::new();
    for th in 0..9 {
        for (k, opp, opts) in [
            (0u16, 0u8, SimOptions::default()),
            (1, 3, SimOptions::default()),
            (
                2,
                2,
                SimOptions {
                    realistic_damage: true,
                    realistic_guidance: true,
                    air_defence: true,
                    ..Default::default()
                },
            ),
        ] {
            let m = mission(d, th, 0x100 + th as u16 * 7 + k, opp);
            let mut sim = Sim::with_options(&d.sim, &m, opts);
            let mut trace = String::new();
            scripted(&mut sim, &m, th as u32 * 77 + k as u32, 15 * 600, &mut trace);
            got.push(check(&format!("mission-{th}-{k}"), &trace));
        }
    }
    got
}

/// A campaign: sorties from a friendly base towards the front, with the war going on.
fn campaign_run() -> u64 {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 2,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(77)).unwrap();
    m.world = d.gdata.theaters[3].world.clone();
    m.weather = Default::default();
    let opts = SimOptions {
        campaign: true,
        air_defence: true,
        realistic_guidance: true,
        ..Default::default()
    };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    let base = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x401)
        .expect("friendly base");
    let enemy = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x001)
        .expect("enemy base");
    let mut trace = String::new();
    let idle = PlayerState { on_ground: true, fuel: 10000, ..Default::default() };
    for round in 0..2u32 {
        for n in 0..2 {
            sim.spawn_insertion(base, enemy, 10 + round * 4 + n, n as usize);
        }
        let t = sim.objects()[base];
        sim.spawn_fighter(
            enemy,
            t.x,
            t.y,
            20 + round,
            false,
            stealth_sim::ai_loadout::FighterTask::Patrol,
        );
        for _ in 0..15 * 30 {
            let ev = sim.step(&idle, 15);
            trace.push_str(&frame_line(&sim, &ev));
            trace.push('\n');
        }
        let _ = sim.respawn_player(base, Some((2, enemy)));
        sim.set_loadout(&[(6, 1), (1, 2), (6, 1), (-1, 0)]);
        // Head for the enemy base.
        let mut target = m.clone();
        let e = sim.objects()[enemy];
        target.waypoints[1] = stealth_mission::MapPoint { x: e.x, y: e.y };
        target.waypoints[2] = target.waypoints[1];
        scripted(&mut sim, &target, 900 + round, 15 * 400, &mut trace);
        if !sim.player_absent() {
            sim.end_sortie(7);
            let ev = sim.take_events();
            trace.push_str(&frame_line(&sim, &ev));
            trace.push('\n');
        }
    }
    for _ in 0..15 * 60 {
        let ev = sim.step(&idle, 15);
        trace.push_str(&frame_line(&sim, &ev));
        trace.push('\n');
    }
    check("campaign", &trace)
}

#[test]
fn single_player_parity() {
    const GOLDEN: &[u64] = &[
        0xd88c26bc6cdeee05,
        0x5d347f9240a6553d,
        0xbc46a430accd79d7,
        0xb92b7d5331cd5932,
        0xcb956320ed773b32,
        0xbac9b842c3c1ddf4,
        0x6204b36206eebdaf,
        0x824104335afcb32e,
        0x0f76c4719abe7230,
        0x765674a9ead8c722,
        0x92924f6b9de030e0,
        0x715f2f1c939ee20e,
        0xefa52df57d504872,
        0x949ce5f8a60c6630,
        0x2abb41d961425a3c,
        0x04a21d9c9a0b4327,
        0x1de22b0d262f85e2,
        0xf236b24f6724cbc3,
        0xdc04984802aa000d,
        0x31076a46996bfbc5,
        0x7f32326de1508b26,
        0x8a4c809f316d7c20,
        0x24b531800edfdde2,
        0x15f37b44f99a422b,
        0xbc7cf0265e10b973,
        0xf7b4a4c462fba7bb,
        0x63ca390d2344befc,
        0x3ae6337e7c135d29,
    ];
    let mut got = missions();
    got.push(campaign_run());
    let hex: Vec<String> = got.iter().map(|h| format!("0x{h:016x}")).collect();
    assert_eq!(got, GOLDEN, "new hashes: [{}]", hex.join(", "));
}

/// Realistic AI flight ([`SimOptions::realistic_ai_flight`]): the scripted missions of every
/// theater with the AI's jets flying the flight model and defending against missiles.
#[test]
fn ai_flight_parity() {
    const GOLDEN: &[u64] = &[
        0xd3bebfcd1802cd0a,
        0x97b78a9e34237543,
        0xc9e53fe5e3f140df,
        0x1209bc86fcbc4de4,
        0x6175e5260c2bceec,
        0x7061c546ecf54319,
        0xdee44fae2a976584,
        0xe56f7fcd014e55ec,
        0x8329c0858e8682e5,
    ];
    let d = data();
    let mut got = Vec::new();
    for th in 0..9 {
        let k = 3u16;
        let m = mission(d, th, 0x100 + th as u16 * 7 + k, 3);
        let opts = SimOptions {
            realistic_guidance: true,
            realistic_ai_flight: true,
            ..Default::default()
        };
        let mut sim = Sim::with_options(&d.sim, &m, opts);
        let mut trace = String::new();
        scripted(&mut sim, &m, th as u32 * 77 + k as u32, 15 * 600, &mut trace);
        got.push(check(&format!("ai-flight-{th}"), &trace));
    }
    let hex: Vec<String> = got.iter().map(|h| format!("0x{h:016x}")).collect();
    assert_eq!(got, GOLDEN, "new hashes: [{}]", hex.join(", "));
}
