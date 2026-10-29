//! Campaign: resupply helicopters for the friendly air defence (`supply`).

use std::sync::OnceLock;

use stealth_mission::{GameData, MissionKind, MissionSettings, Seed};
use stealth_sim::aaa::{AaaKind, M_PER_MAP};
use stealth_sim::station::{PATRIOT_STOCK, SavedStock};
use stealth_sim::supply::{DROP_RADIUS, HOOK_ALT, HOVER_MAX, SET_ALT, SupplyRun};
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

/// Metres between a flight unit and a point (world units).
fn metres_at(u: &FlightUnit, p: (i32, i32)) -> f64 {
    ((u.wx - p.0) as f64).hypot((u.wy - p.1) as f64) * M_PER_MAP / 32.0
}

/// Flies helicopter `i` (tag `tag`) until it is back on its pad: it hooks its crate low over
/// `pickup` and holds there while it is lifted, and carries it until it is delivered. Returns
/// the helicopter at the delivery and where it landed (map units).
fn sling_run(sim: &mut Sim, i: usize, tag: u32, pickup: (i32, i32)) -> (FlightUnit, (i32, i32)) {
    let run = |sim: &Sim| sim.campaign_flight(i).and_then(|f| f.supply);
    let (mut hooked, mut lifted, mut delivered) = (None, None, None);
    for frame in 0..15 * 900 {
        let ev = sim.step(&idle(), 15);
        let u = sim.flights()[i];
        if let Some(r) = run(sim) {
            if r.hooked && hooked.is_none() {
                // Over the crate, the hook in reach of it.
                assert!(metres_at(&u, pickup) < 8.0, "{}", metres_at(&u, pickup));
                assert!(u.alt <= HOOK_ALT + 2, "{}", u.alt);
                hooked = Some(frame);
            }
            if r.pickup.is_none() && hooked.is_some() && lifted.is_none() {
                // It held the spot while the crate came up.
                assert!(metres_at(&u, pickup) < 8.0, "{}", metres_at(&u, pickup));
                lifted = Some(frame);
            }
            if lifted.is_some() && !r.delivered {
                assert!(r.hooked, "the crate hangs under it on the way");
            }
        }
        for e in ev {
            match e {
                SimEvent::CampaignFlightDelivered { tag: t, .. } if t == tag => {
                    let r: SupplyRun = run(sim).expect("run");
                    assert!(r.set_down && !r.hooked, "{r:?}");
                    // Hovering low enough for the crate on its rope to stand on the ground.
                    assert!(u.alt > 0 && u.alt <= SET_ALT, "{}", u.alt);
                    assert!(metres_at(&u, r.drop) < 8.0, "{}", metres_at(&u, r.drop));
                    delivered = Some((frame, u));
                }
                SimEvent::CampaignFlightLanded { tag: t, x, y, .. } if t == tag => {
                    let (hooked, lifted) = (hooked.expect("hooked"), lifted.expect("lifted"));
                    assert!(lifted - hooked >= 15 * 3, "{hooked} {lifted}");
                    let (at, u) = delivered.expect("delivered");
                    assert!(at > lifted + 15 * 10, "it flew there with the crate");
                    return (u, (x, y));
                }
                _ => {}
            }
        }
    }
    panic!("never landed: hooked {hooked:?}, lifted {lifted:?}, delivered {delivered:?}");
}

/// Metres between a flight unit and the middle of an AAA unit.
fn metres_to(u: &FlightUnit, a: &stealth_sim::aaa::AaaUnit) -> f64 {
    let (cx, cy) = ((a.x << 5) + 16, (a.y << 5) + 16);
    ((u.wx - cx) as f64).hypot((u.wy - cy) as f64) * M_PER_MAP / 32.0
}

#[test]
fn the_ai_helicopter_slings_a_crate_to_the_battery_and_lands_back_on_its_pad() {
    let (mut sim, home, radar) = battery();
    let pad = sim.helipad(home);
    let o = sim.objects()[home];
    // The airfield's own helipad (from the theater's scenery), not the spot beside it.
    assert_ne!(pad, ((o.x + 6) << 5, o.y << 5));
    // Its crate 500 m from the pad.
    let pickup = (pad.0 + 1024, pad.1);
    let i = sim.spawn_supply(home, radar, Some(pickup), 42);
    let a = sim.aaa_units()[radar];
    let (u, (x, y)) = sling_run(&mut sim, i, 42, pickup);
    assert!(metres_to(&u, &a) <= DROP_RADIUS, "{}", metres_to(&u, &a));
    assert!(u.alt as f64 * M_PER_MAP / 32.0 <= HOVER_MAX, "{}", u.alt);
    assert!((x - (pad.0 >> 5)).abs() <= 1 && (y - (pad.1 >> 5)).abs() <= 1, "{x},{y} {pad:?}");
    // Landed: gone, its slot free.
    assert!(sim.campaign_flight(i).is_none());
    assert_eq!(sim.flights()[i].flags, 0);
}

#[test]
fn the_ai_helicopter_without_a_crate_hovers_at_the_battery() {
    let (mut sim, home, radar) = battery();
    let i = sim.spawn_supply(home, radar, None, 42);
    let a = sim.aaa_units()[radar];
    let mut delivered = false;
    for _ in 0..15 * 900 {
        for e in sim.step(&idle(), 15) {
            if let SimEvent::CampaignFlightDelivered { tag: 42, aaa, .. } = e {
                let u = sim.flights()[i];
                assert_eq!(aaa, radar);
                assert!(metres_to(&u, &a) <= DROP_RADIUS, "{}", metres_to(&u, &a));
                delivered = true;
            }
        }
        if delivered {
            break;
        }
    }
    assert!(delivered);
    let r = sim.campaign_flight(i).and_then(|f| f.supply).unwrap();
    assert!(!r.hooked && !r.set_down, "{r:?}");
}

#[test]
fn a_delivery_fills_the_stockpile() {
    let (mut sim, _, radar) = battery();
    let a = sim.aaa_units()[radar];
    sim.restore_stocks(&[SavedStock {
        x: a.x,
        y: a.y,
        kind: AaaKind::PatriotRadar,
        stock: 1,
        stock2: 0,
    }]);
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

#[test]
fn a_cargo_helicopter_slings_its_crate_to_a_new_sites_helipad_and_flies_home() {
    let (mut sim, home, _) = battery();
    let pad = sim.helipad(home);
    let o = sim.objects()[home];
    // A new site's helipad 6 km west of the airfield (map units), the crate set down 9 m off it.
    let (px, py) = (o.x - 0x180, o.y + 0x20);
    let drop = ((px << 5) + 16 + 18, (py << 5) + 16);
    let pickup = (pad.0, pad.1 - 1024);
    let i = sim.spawn_cargo(home, Some(pickup), drop, 43);
    let (u, (x, y)) = sling_run(&mut sim, i, 43, pickup);
    assert!(metres_at(&u, drop) < 8.0);
    assert!((x - (pad.0 >> 5)).abs() <= 1 && (y - (pad.1 >> 5)).abs() <= 1, "{x},{y} {pad:?}");
}

#[test]
fn a_new_gepard_is_friendly() {
    let (mut sim, home, _) = battery();
    let o = sim.objects()[home];
    let n = sim.aaa_units().len();
    sim.place_gepard(o.x - 0x180, o.y);
    assert_eq!(sim.aaa_units().len(), n + 1);
    assert_eq!(sim.aaa_units()[n].kind, AaaKind::Gepard);
    assert!(sim.aaa_blue(n));
}
