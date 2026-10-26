//! Campaign: friendly air defence manned by a player (`station`).

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{AaaKind, PATRIOT};
use stealth_sim::ai_loadout::FighterTask;
use stealth_sim::station::{
    BURST, GEPARD_ROUNDS, Guidance, PATRIOT_STOCK, REFILL_SECONDS, SavedStock, StationCmd,
};
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

fn campaign() -> Sim {
    let d = data();
    let s = MissionSettings {
        theater: 3,
        roe: 2,
        opponents: 1,
        kind: MissionKind::Strike,
        ..Default::default()
    };
    let mut m = stealth_mission::generate(&d.gdata, &s, Seed::Srand(78)).unwrap();
    m.world = d.gdata.theaters[3].world.clone();
    m.weather = Default::default();
    let mut sim =
        Sim::with_options(&d.sim, &m, SimOptions { campaign: true, ..Default::default() });
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
    }
    sim.aaa_units_mut().clear();
    sim
}

fn idle() -> PlayerState {
    PlayerState { on_ground: true, fuel: 10000, ..Default::default() }
}

fn find(sim: &Sim, flags: i32, name_like: &str) -> usize {
    (3..sim.objects().len())
        .find(|&i| {
            sim.objects()[i].flags & 0xF0F == flags
                && sim.place_name(i).eq_ignore_ascii_case(name_like)
        })
        .expect("object")
}

fn run(sim: &mut Sim, secs: i32) -> Vec<SimEvent> {
    let mut ev = Vec::new();
    for _ in 0..secs * 15 {
        ev.extend(sim.step(&idle(), 15));
    }
    ev
}

const ME: u8 = 0;

/// A friendly Patriot battery by Neuburg and an unarmed enemy fighter (tag 8) on its way
/// there: the battery's radar, the fighter's flight unit.
fn battery() -> (Sim, usize, usize) {
    let mut sim = campaign();
    let home = find(&sim, 0x401, "Neuburg");
    let enemy = find(&sim, 0x001, "Komarno");
    let t = sim.objects()[home];
    let f = sim.spawn_fighter(enemy, t.x, t.y, 8, false, FighterTask::Patrol);
    sim.set_fighter_loadout(f, Default::default());
    sim.place_patriot(t.x + 0x100, t.y);
    let radar = kind_index(&sim, AaaKind::PatriotRadar);
    // In the PAC-3's reach: 30 km east, at 2 km.
    let at = sim.aaa_units()[radar];
    hold(&mut sim, f, (at.x + 0x780, at.y, 4000));
    (sim, radar, f)
}

fn kind_index(sim: &Sim, kind: AaaKind) -> usize {
    sim.aaa_units().iter().position(|u| u.kind == kind).expect("unit")
}

/// The fighter held over a point (map units, world-unit altitude) for a moment.
fn hold(sim: &mut Sim, f: usize, (x, y, alt): (i32, i32, i32)) {
    let u = &mut sim.flights_mut()[f];
    (u.x, u.y, u.alt) = (x, y, alt);
    (u.wx, u.wy) = (u.x << 5, u.y << 5);
    u.flags |= fflags::AIRBORNE;
}

fn launcher_rounds(sim: &Sim, radar: usize) -> i32 {
    sim.battery_launchers(radar).iter().map(|&k| sim.aaa_units()[k].rounds).sum()
}

#[test]
fn a_manned_battery_fires_on_its_lock() {
    let (mut sim, radar, f) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    assert_eq!(sim.station_of(ME).map(|s| s.unit as usize), Some(radar));
    assert_eq!(sim.manned_by(radar), Some(ME));
    assert!(sim.station_command(ME, StationCmd::Fire { launcher: None }).is_err());
    run(&mut sim, 1);
    sim.station_command(ME, StationCmd::Lock(Some(f as u16))).unwrap();
    let before = launcher_rounds(&sim, radar);
    sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap();
    assert_eq!(launcher_rounds(&sim, radar), before - 1);
    let m: Vec<_> = sim.ai_missiles().iter().filter(|m| m.station.is_some()).collect();
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].spec, PATRIOT);
    assert_eq!(m[0].target, f);
    assert_eq!(m[0].guidance, Some(Guidance::Guided));
    // The same launcher waits; another one fires.
    sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap();
    assert_eq!(launcher_rounds(&sim, radar), before - 2);
}

#[test]
fn another_player_cannot_man_a_manned_battery() {
    let (mut sim, radar, _) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    assert!(sim.station_command(1, StationCmd::Man { unit: radar as u16 }).is_err());
    sim.station_command(ME, StationCmd::Leave).unwrap();
    sim.station_command(1, StationCmd::Man { unit: radar as u16 }).unwrap();
    let launcher = kind_index(&sim, AaaKind::PatriotLauncher);
    assert!(sim.station_command(ME, StationCmd::Man { unit: launcher as u16 }).is_err());
}

#[test]
fn going_semi_active_or_changing_the_lock_releases_guided_missiles() {
    let (mut sim, radar, f) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    run(&mut sim, 1);
    sim.station_command(ME, StationCmd::Lock(Some(f as u16))).unwrap();
    sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap();
    let guided = |sim: &Sim| {
        sim.ai_missiles().iter().filter(|m| m.guidance == Some(Guidance::Guided)).count()
    };
    run(&mut sim, 1);
    assert_eq!(guided(&sim), 1, "a guided missile stays guided on its lock");
    sim.station_command(ME, StationCmd::Mode(Guidance::SemiActive)).unwrap();
    assert_eq!(guided(&sim), 0);
    sim.station_command(ME, StationCmd::Mode(Guidance::Guided)).unwrap();
    run(&mut sim, 8);
    sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap();
    assert_eq!(guided(&sim), 1);
    sim.station_command(ME, StationCmd::Lock(None)).unwrap();
    run(&mut sim, 1);
    assert_eq!(guided(&sim), 0, "the lock changed");
    let semi = sim.ai_missiles().iter().filter(|m| m.guidance == Some(Guidance::SemiActive));
    assert!(semi.count() >= 1, "semi-active missiles fly on at their target");
}

#[test]
fn a_launcher_refills_from_the_stockpile_in_real_time() {
    let (mut sim, radar, _) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    let l = sim.battery_launchers(radar)[0];
    assert!(sim.station_command(ME, StationCmd::Refill { unit: l as u16 }).is_err(), "not empty");
    sim.aaa_units_mut()[l].rounds = 0;
    assert_eq!(sim.stockpile(l), PATRIOT_STOCK);
    sim.station_command(ME, StationCmd::Refill { unit: l as u16 }).unwrap();
    // The game's own clock does not refill it.
    run(&mut sim, 10);
    sim.tick_refills(REFILL_SECONDS - 1.0);
    assert_eq!(sim.aaa_units()[l].rounds, 0);
    assert!(sim.refill_left(l) > 0.0);
    sim.tick_refills(1.0);
    assert_eq!(sim.aaa_units()[l].rounds, 4);
    assert_eq!(sim.stockpile(l), PATRIOT_STOCK - 4);
    // Short of missiles: what there is.
    let at = sim.aaa_units()[radar];
    sim.restore_stocks(&[SavedStock { x: at.x, y: at.y, patriot: true, stock: 2 }]);
    let l2 = sim.battery_launchers(radar)[1];
    sim.aaa_units_mut()[l2].rounds = 0;
    sim.station_command(ME, StationCmd::Refill { unit: l2 as u16 }).unwrap();
    sim.tick_refills(REFILL_SECONDS);
    assert_eq!(sim.aaa_units()[l2].rounds, 2);
    assert_eq!(sim.stockpile(l2), 0);
}

#[test]
fn an_unmanned_battery_refills_itself() {
    let (mut sim, radar, _) = battery();
    let l = sim.battery_launchers(radar)[0];
    sim.aaa_units_mut()[l].rounds = 0;
    sim.tick_refills(1.0);
    assert!(sim.refill_left(l) > 0.0);
    sim.tick_refills(REFILL_SECONDS);
    assert_eq!(sim.aaa_units()[l].rounds, 4);
}

#[test]
fn the_ai_leaves_a_manned_battery_alone() {
    let (mut sim, radar, f) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    let at = sim.aaa_units()[radar];
    for _ in 0..60 {
        hold(&mut sim, f, (at.x + 0x200, at.y, 4000));
        run(&mut sim, 1);
        assert!(sim.ai_missiles().is_empty(), "the AI fired from a manned battery");
    }
    // Unmanned, it fires.
    sim.station_command(ME, StationCmd::Leave).unwrap();
    let mut fired = false;
    for _ in 0..60 {
        hold(&mut sim, f, (at.x + 0x200, at.y, 4000));
        run(&mut sim, 1);
        fired |= !sim.ai_missiles().is_empty();
    }
    assert!(fired);
}

#[test]
fn losing_the_radar_ends_the_station() {
    let (mut sim, radar, f) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    run(&mut sim, 1);
    sim.station_command(ME, StationCmd::Lock(Some(f as u16))).unwrap();
    sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap();
    sim.aaa_units_mut()[radar].destroyed = true;
    run(&mut sim, 1);
    assert!(sim.station_of(ME).is_none());
    assert!(sim.ai_missiles().iter().all(|m| m.guidance.is_none()), "its missiles are lost");
}

#[test]
fn weapons_hold_on_a_friendly_track() {
    let (mut sim, radar, _) = battery();
    let home = find(&sim, 0x401, "Neuburg");
    let t = sim.objects()[home];
    let friend = sim.spawn_fighter(home, t.x, t.y, 9, true, FighterTask::Cap);
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    run(&mut sim, 5);
    hold(&mut sim, friend, (t.x, t.y, 4000));
    sim.station_command(ME, StationCmd::Lock(Some(friend as u16))).unwrap();
    let e = sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap_err();
    assert!(e.contains("Friendly"), "{e}");
}

#[test]
fn a_gepard_burst_brings_down_a_low_raider() {
    let mut sim = campaign();
    let home = find(&sim, 0x401, "Neuburg");
    let enemy = find(&sim, 0x001, "Komarno");
    let o = sim.objects()[home];
    let f = sim.spawn_fighter(enemy, o.x, o.y, 8, false, FighterTask::Patrol);
    sim.set_fighter_loadout(f, Default::default());
    sim.place_defences(o.x, o.y, true, true);
    let g = kind_index(&sim, AaaKind::Gepard);
    // Only the manned Gepard.
    for (k, u) in sim.aaa_units_mut().iter_mut().enumerate() {
        u.destroyed = k != g;
    }
    assert_eq!(sim.aaa_units()[g].rounds, GEPARD_ROUNDS);
    sim.station_command(ME, StationCmd::Man { unit: g as u16 }).unwrap();
    let at = sim.aaa_units()[g];
    // Far: out of range.
    hold(&mut sim, f, (at.x + 0x200, at.y, 1500));
    run(&mut sim, 1);
    hold(&mut sim, f, (at.x + 0x200, at.y, 1500));
    sim.station_command(ME, StationCmd::Lock(Some(f as u16))).unwrap();
    let e = sim.station_command(ME, StationCmd::Burst).unwrap_err();
    assert!(e.contains("range"), "{e}");
    let mut bursts = 0;
    let mut down = false;
    for _ in 0..40 {
        hold(&mut sim, f, (at.x + 0x30, at.y, 1500));
        if sim.station_command(ME, StationCmd::Burst).is_ok() {
            bursts += 1;
        }
        let ev = run(&mut sim, 2);
        if ev.iter().any(|e| matches!(e, SimEvent::CampaignFlightDestroyed { tag: 8, .. })) {
            down = true;
            break;
        }
    }
    assert!(down, "{bursts} bursts never got it");
    // A burst stops when its target is down.
    let used = GEPARD_ROUNDS - sim.aaa_units()[g].rounds;
    assert!(
        used > (bursts - 1) * BURST && used <= bursts * BURST,
        "{used} rounds, {bursts} bursts"
    );
}

#[test]
fn one_launcher_fires_its_missiles_within_seconds() {
    let (mut sim, radar, f) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    run(&mut sim, 1);
    sim.station_command(ME, StationCmd::Lock(Some(f as u16))).unwrap();
    let l = sim.battery_launchers(radar)[0];
    let fire = StationCmd::Fire { launcher: Some(l as u16) };
    sim.station_command(ME, fire).unwrap();
    assert!(sim.station_command(ME, fire).is_err(), "not at once");
    run(&mut sim, 3);
    sim.station_command(ME, fire).unwrap();
    assert_eq!(sim.aaa_units()[l].rounds, 2);
}

#[test]
fn no_shot_beyond_the_pac3_reach() {
    let (mut sim, radar, f) = battery();
    sim.station_command(ME, StationCmd::Man { unit: radar as u16 }).unwrap();
    let at = sim.aaa_units()[radar];
    // 90 km east.
    hold(&mut sim, f, (at.x + 5760, at.y, 16000));
    run(&mut sim, 1);
    hold(&mut sim, f, (at.x + 5760, at.y, 16000));
    sim.station_command(ME, StationCmd::Lock(Some(f as u16))).unwrap();
    let e = sim.station_command(ME, StationCmd::Fire { launcher: None }).unwrap_err();
    assert!(e.contains("range"), "{e}");
}

#[test]
fn captures_reuse_the_wrecks_and_a_friendly_field_keeps_its_gepard() {
    let mut sim = campaign();
    let home = find(&sim, 0x401, "Neuburg");
    let o = sim.objects()[home];
    sim.place_defences(o.x, o.y, true, true);
    let len = sim.aaa_units().len();
    for i in 0..60 {
        let blue = i % 2 == 1;
        sim.place_defences(o.x, o.y, true, blue);
        sim.wreck_defences(o.x, o.y, blue, 10);
        if blue {
            let k =
                sim.aaa_units().iter().enumerate().position(|(k, u)| {
                    u.kind == AaaKind::Gepard && !u.destroyed && sim.aaa_blue(k)
                });
            assert!(k.is_some(), "capture {i}: no Gepard");
            assert_eq!(sim.aaa_units()[k.unwrap()].rounds, GEPARD_ROUNDS);
        }
    }
    assert!(sim.aaa_units().len() <= len + 12, "{} units from {len}", sim.aaa_units().len());
}
