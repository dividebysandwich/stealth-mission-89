//! Cargo jet missions (the C-5): the generated targets become an airdrop and a delivery. The
//! aircraft drops pallets to the friendly troops nearest the generated primary target (mission
//! type [`AIRDROP`]: released within the simulation's drop radius of the beacon), then lands its
//! cargo at the friendly airfield nearest them (type [`DELIVERY`]: stopped on that airfield's
//! runway), which ends the flight. It takes off from a land base (the generator's
//! `land_bases`), never a carrier.

use crate::world::{WorldObject, flags as F};
use crate::{Base, Bay, GameData, MapPoint, Mission, OrdersLine, Target};

/// Mission types: a supply drop at the target's beacon, a landing at the target airfield.
pub const AIRDROP: i16 = 3;
pub const DELIVERY: i16 = 4;
/// The pallets: special equipment (units.ron `weapons`), and how many the hold takes.
pub const PALLETS: i16 = 18;
pub const PALLET_COUNT: u8 = 4;
/// The target's code word bit that makes the simulation call out its beacon.
const BEACON: i16 = 0x10;

fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
    (((a.0 - b.0) as f64).powi(2) + ((a.1 - b.1) as f64).powi(2)).sqrt()
}

fn pos(o: &WorldObject) -> (i32, i32) {
    (o.x as u16 as i32, o.y as u16 as i32)
}

/// A named friendly place on the ground, not an airfield: where the troops wait for the drop.
fn drop_zone(o: &WorldObject) -> bool {
    o.flags & F::FRIENDLY != 0
        && o.flags & (F::AIRFIELD | F::CARRIER | F::SHIP | F::NEUTRAL | F::DISABLED) == 0
        && o.name != 0
}

/// A friendly airfield on land the cargo jet can land at.
fn airfield(o: &WorldObject) -> bool {
    o.flags & (F::AIRFIELD | F::FRIENDLY | F::NEUTRAL | F::CARRIER | F::DISABLED)
        == F::AIRFIELD | F::FRIENDLY
}

/// Turns a generated mission into a cargo jet's: an airdrop near its primary target, the
/// landing at the friendly airfield nearest the drop zone, the pallets in the hold. Done once
/// (a mission already turned is left alone); a theater without friendly places keeps the
/// generated targets.
pub fn for_transport(m: &mut Mission, data: &GameData) {
    if m.primary.mission_type == AIRDROP && m.secondary.mission_type == DELIVERY {
        return;
    }
    let w = &m.world;
    let n = w.objects.len() as i32;
    let near = |from: (i32, i32), ok: &dyn Fn(&WorldObject) -> bool, not: i32| {
        (3..n)
            .filter(|&i| i != not && ok(&w.objects[i as usize]))
            .min_by(|&a, &b| {
                dist(pos(&w.objects[a as usize]), from)
                    .total_cmp(&dist(pos(&w.objects[b as usize]), from))
            })
    };
    let takeoff = m.takeoff.object_index as i32;
    let aim = (m.primary.position.x, m.primary.position.y);
    let Some(zone) = near(aim, &drop_zone, -1) else { return };
    let zo = w.objects[zone as usize];
    let zp = pos(&zo);
    // The airfield nearest the drop zone; back home if it is the only one.
    let Some(dest) = near(zp, &airfield, takeoff).or_else(|| near(zp, &airfield, -1)) else {
        return;
    };
    let dobj = w.objects[dest as usize];
    let dp = pos(&dobj);
    let names = |o: &WorldObject| w.name(o.name as i32).to_string();
    let (zone_name, dest_name) = (names(&zo), names(&dobj));
    let class = |o: &WorldObject| w.shape_class.get(o.shape()).copied().unwrap_or(0) & 0xF;

    let primary_summary = format!(
        "Friendly troops at {zone_name} are cut off from their supplies. Drop them ammunition \
         and equipment from the air."
    );
    let primary_objective = format!(
        "Release the supply pallets over {zone_name}, within 4 km of its beacon."
    );
    let secondary_summary = format!(
        "The airbase at {dest_name} is short of munitions for its squadrons: the rest of the \
         cargo is theirs."
    );
    let secondary_objective = format!(
        "Land at {dest_name} and stop on its runway to unload the cargo."
    );
    m.primary = Target {
        mission_type: AIRDROP,
        type_name: "airdrop".into(),
        code: BEACON,
        has_deadline: false,
        area_target: false,
        rubble_at_deadline: false,
        special_model: None,
        object_index: zone as i16,
        place_name: zone_name.clone(),
        shape: zo.shape() as u8,
        shape_name: w.name(zo.shape() as i32).into(),
        placed_by_generator: false,
        position: MapPoint { x: zp.0, y: zp.1 },
        role: w.role(zo.shape()),
        weapon_class: class(&zo),
        dist: 0,
        intelligence: primary_summary.clone(),
        objective: primary_objective.clone(),
        ..m.primary.clone()
    };
    m.secondary = Target {
        mission_type: DELIVERY,
        type_name: "airlift".into(),
        code: 0,
        has_deadline: false,
        area_target: false,
        rubble_at_deadline: false,
        special_model: None,
        object_index: dest as i16,
        place_name: dest_name.clone(),
        shape: dobj.shape() as u8,
        shape_name: w.name(dobj.shape() as i32).into(),
        placed_by_generator: false,
        position: MapPoint { x: dp.0, y: dp.1 },
        role: w.role(dobj.shape()),
        weapon_class: class(&dobj),
        dist: 0,
        intelligence: secondary_summary.clone(),
        objective: secondary_objective.clone(),
        ..m.secondary.clone()
    };
    let old_landing = m.landing.name.clone();
    m.landing = Base {
        object_index: dest as i16,
        name: dest_name.clone(),
        position: MapPoint { x: dp.0, y: dp.1 },
        carrier: false,
        neutral: false,
        distance: dist(zp, dp) as i16,
    };
    m.exit_code = 0x0C;
    // Route: from the takeoff base to the drop zone, on to the airfield; it is also the landing.
    let tp = (m.takeoff.position.x, m.takeoff.position.y);
    let mid = ((tp.0 + zp.0) / 2, (tp.1 + zp.1) / 2);
    let point = |p: (i32, i32)| MapPoint { x: p.0, y: p.1 };
    m.waypoints = [point(mid), point(zp), point(dp), point(dp)];
    m.route_length = ((dist(tp, zp) + dist(zp, dp)) / 64.0) as i16;
    // No hurry: the cargo counts whenever it arrives.
    m.time.has_deadline = false;
    m.time.deadline_ticks = i16::MAX;

    // The pallets in the hold, nothing else.
    let x = &data.tables;
    m.loadout.bays[0] =
        Bay { weapon: PALLETS, name: x.weapon_name(PALLETS).into(), count: PALLET_COUNT };
    m.loadout.usable_bays = 1;

    // Briefing and orders.
    let b = &mut m.briefing;
    b.title = "Airlift".into();
    b.primary_summary = primary_summary;
    b.primary_objective = primary_objective;
    b.secondary_summary = secondary_summary;
    b.secondary_objective = secondary_objective;
    b.land_line = b.land_line.replace(&old_landing, &dest_name);
    b.armament = vec![format!("{PALLET_COUNT} {}", m.loadout.bays[0].name)];
    let l = &data.briefings.labels;
    let mut lines = vec![
        b.header.clone(),
        b.title.clone(),
        b.pilot_line.clone(),
        b.base_date_line.clone(),
        l.primary.clone(),
        b.primary_summary.clone(),
        l.objective.clone(),
        b.primary_objective.clone(),
        l.secondary.clone(),
        b.secondary_summary.clone(),
        l.objective.clone(),
        b.secondary_objective.clone(),
        l.flight_plan.clone(),
        b.depart_line.clone(),
        b.land_line.clone(),
        l.roe.clone(),
        b.roe_text.clone(),
        l.armament.clone(),
    ];
    lines.extend(b.armament.iter().cloned());
    lines.retain(|l| !l.is_empty());
    b.lines = lines;
    let o = &mut m.orders;
    o.lines = vec![
        OrdersLine { text: "Drop supplies over".into(), color: 0x11 },
        OrdersLine { text: zone_name.clone(), color: 0x11 },
        OrdersLine { text: "Deliver the cargo to".into(), color: 0x29 },
        OrdersLine { text: dest_name.clone(), color: 0x29 },
    ];
    o.primary = format!("DROP SUPPLIES OVER {}", zone_name.to_uppercase());
    o.secondary = format!("DELIVER THE CARGO TO {}", dest_name.to_uppercase());
    o.return_to = format!("LAND AT {}", dest_name.to_uppercase());
}
