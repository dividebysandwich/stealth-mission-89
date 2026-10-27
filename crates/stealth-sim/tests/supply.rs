//! Campaign: resupply helicopters for the friendly air defence (`supply`).

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{AaaKind, M_PER_MAP};
use stealth_sim::station::{PATRIOT_STOCK, SavedStock};
use stealth_sim::supply::{DROP_RADIUS, HOVER_MAX};
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

/// A Central Europe campaign with a friendly Patriot battery 4 km east of Leipheim: the sim,
/// Leipheim's airfield and the battery's radar.
fn battery() -> (Sim, usize, usize) {
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
    let home = (3..sim.objects().len())
        .find(|&i| {
            sim.objects()[i].flags & 0xF0F == 0x401
                && sim.place_name(i).eq_ignore_ascii_case("Leipheim")
        })
        .expect("Leipheim");
    let t = sim.objects()[home];
    sim.place_patriot(t.x + 0x100, t.y);
    let radar = sim.aaa_units().iter().position(|u| u.kind == AaaKind::PatriotRadar).unwrap();
    (sim, home, radar)
}

fn idle() -> PlayerState {
    PlayerState { on_ground: true, fuel: 10000, ..Default::default() }
}

/// Metres between a flight unit and the middle of an AAA unit.
fn metres_to(u: &FlightUnit, a: &stealth_sim::aaa::AaaUnit) -> f64 {
    let (cx, cy) = ((a.x << 5) + 16, (a.y << 5) + 16);
    ((u.wx - cx) as f64).hypot((u.wy - cy) as f64) * M_PER_MAP / 32.0
}

#[test]
fn the_ai_helicopter_hovers_at_the_battery_and_lands_back_on_its_pad() {
    let (mut sim, home, radar) = battery();
    let pad = sim.helipad(home);
    let o = sim.objects()[home];
    // The airfield's own helipad (from the theater's scenery), not the spot beside it.
    assert_ne!(pad, ((o.x + 6) << 5, o.y << 5));
    let i = sim.spawn_supply(home, radar, 42);
    let a = sim.aaa_units()[radar];
    let (mut delivered, mut landed) = (None, None);
    let mut last = sim.flights()[i];
    for frame in 0..15 * 900 {
        let ev = sim.step(&idle(), 15);
        for e in ev {
            match e {
                SimEvent::CampaignFlightDelivered { tag: 42, aaa, .. } => {
                    let u = sim.flights()[i];
                    assert_eq!(aaa, radar);
                    assert!(metres_to(&u, &a) <= DROP_RADIUS, "{}", metres_to(&u, &a));
                    assert!(u.alt as f64 * M_PER_MAP / 32.0 <= HOVER_MAX, "{}", u.alt);
                    delivered = Some(frame);
                }
                SimEvent::CampaignFlightLanded { tag: 42, x, y, .. } => landed = Some((x, y)),
                _ => {}
            }
        }
        if landed.is_some() {
            break;
        }
        last = sim.flights()[i];
    }
    let delivered = delivered.expect("delivered");
    assert!(delivered > 15 * 10, "it flew there first");
    let (x, y) = landed.expect("landed");
    assert!((x - (pad.0 >> 5)).abs() <= 1 && (y - (pad.1 >> 5)).abs() <= 1, "{x},{y} {pad:?}");
    assert!(last.alt < 10, "{}", last.alt);
    // Landed: gone, its slot free.
    assert!(sim.campaign_flight(i).is_none());
    assert_eq!(sim.flights()[i].flags, 0);
}

#[test]
fn a_delivery_fills_the_stockpile() {
    let (mut sim, _, radar) = battery();
    let a = sim.aaa_units()[radar];
    sim.restore_stocks(&[SavedStock { x: a.x, y: a.y, patriot: true, stock: 1 }]);
    assert_eq!(sim.stockpile(radar), 1);
    sim.restock(radar);
    assert_eq!(sim.stockpile(radar), PATRIOT_STOCK);
}

#[test]
fn a_player_hovers_low_and_close_to_deliver() {
    let (mut sim, home, radar) = battery();
    let heli = Airframe { helicopter: true, ..Default::default() };
    sim.respawn_pilot(0, home, None, heli);
    let a = sim.aaa_units()[radar];
    // 30 m east of the radar (world units, y north), at `h` world units over the ground.
    let at = |h: u16| PlayerState {
        x: (a.x << 5) + 16 + 61,
        y: 0x10_0000 - ((a.y << 5) + 16),
        altitude: h,
        height: h,
        on_ground: false,
        fuel: 10000,
        ..Default::default()
    };
    sim.step(&at(14), 15);
    assert_eq!(sim.pilot_over_unit(0), Some(radar));
    // 40 m up is too high.
    sim.step(&at(82), 15);
    assert_eq!(sim.pilot_over_unit(0), None);
    // 80 m off is too far.
    let far = PlayerState { x: (a.x << 5) + 16 + 164, ..at(14) };
    sim.step(&far, 15);
    assert_eq!(sim.pilot_over_unit(0), None);
}
