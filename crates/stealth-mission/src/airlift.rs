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

/// An enemy SAM or radar site, airfield or ship: what a routine airlift keeps away from.
fn threat(o: &WorldObject) -> bool {
    o.flags & (F::FRIENDLY | F::NEUTRAL | F::DISABLED) == 0
        && (o.unit_type != 0 || o.flags & (F::AIRFIELD | F::CARRIER | F::SHIP) != 0)
}

/// Distance (map units) from p to the segment a-b.
fn to_segment(p: (i32, i32), a: (i32, i32), b: (i32, i32)) -> f64 {
    let (ax, ay, bx, by) = (a.0 as f64, a.1 as f64, b.0 as f64, b.1 as f64);
    let (px, py) = (p.0 as f64, p.1 as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((px - ax) * dx + (py - ay) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
}

/// How far (map units) the route takeoff - drop zone - airfield keeps from the nearest threat.
pub fn route_clearance(threats: &[(i32, i32)], t: (i32, i32), z: (i32, i32), d: (i32, i32)) -> f64 {
    threats.iter().map(|&p| to_segment(p, t, z).min(to_segment(p, z, d))).fold(f64::MAX, f64::min)
}

/// The enemy SAM and radar sites, airfields and ships of a mission's world.
pub fn threats(m: &Mission) -> Vec<(i32, i32)> {
    let o = &m.world.objects;
    o[3.min(o.len())..].iter().filter(|o| threat(o)).map(pos).collect()
}

/// The enemy SAM and radar sites of a mission's world.
pub fn sam_sites(m: &Mission) -> Vec<(i32, i32)> {
    let o = &m.world.objects;
    o[3.min(o.len())..].iter().filter(|o| threat(o) && o.unit_type != 0).map(pos).collect()
}

/// How far (map units) a front-line airlift's route keeps from the enemy's SAM sites.
const SAM_CLEARANCE: f64 = 110.0 * 64.0;
/// An airlift's shortest leg to the drop zone (map units, 64 a km).
const MIN_LEG: f64 = 30.0 * 64.0;
/// How far (map units) a routine airlift's route keeps from every threat: beyond the long
/// range SAMs' reach; a theater without such a route flies front-line drops only.
const QUIET_CLEARANCE: f64 = 80.0 * 64.0;
/// Fighters escorting a front-line airlift.
pub const ESCORTS: u8 = 2;

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
        (3..n).filter(|&i| i != not && ok(&w.objects[i as usize])).min_by(|&a, &b| {
            dist(pos(&w.objects[a as usize]), from)
                .total_cmp(&dist(pos(&w.objects[b as usize]), from))
        })
    };
    let takeoff = m.takeoff.object_index as i32;
    let tp = (m.takeoff.position.x, m.takeoff.position.y);
    // The airfield nearest a drop zone; back home if it is the only one.
    let dest_of = |zp: (i32, i32)| near(zp, &airfield, takeoff).or_else(|| near(zp, &airfield, -1));
    // About half the airlifts are routine, behind the lines (`Mission::quiet`): the drop zone
    // whose route keeps farthest from the enemy's SAMs and airfields. The others drop to the
    // troops nearest the generated target, near the front, a fighter escort along.
    let roll = {
        let mut x = (m.rng.holdrand_entry as u64) << 32 | m.rng.holdrand_exit as u64;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((x ^ (x >> 31)) >> 32) as u32
    };
    // Some theaters have no route clear enough: more than half are tried.
    let quiet = roll % 10 < 7;
    let threats = threats(m);
    let rear = || {
        let mut zones: Vec<(f64, i32, i32)> = (3..n)
            .filter(|&i| drop_zone(&w.objects[i as usize]))
            .filter_map(|i| {
                let zp = pos(&w.objects[i as usize]);
                let d = dest_of(zp)?;
                let dp = pos(&w.objects[d as usize]);
                (dist(tp, zp) >= MIN_LEG).then(|| (route_clearance(&threats, tp, zp, dp), i, d))
            })
            .collect();
        zones.sort_by(|a, b| b.0.total_cmp(&a.0));
        // Of the few clearest (out of the SAMs' reach), the seed picks one.
        zones.retain(|z| z.0 >= QUIET_CLEARANCE);
        let few = zones.len().min(3);
        (few > 0).then(|| zones[(roll >> 8) as usize % few])
    };
    // Front-line drops: the troops nearest the target whose route stays out of the SAMs'
    // reach (the escort sees to the fighters).
    let aim = (m.primary.position.x, m.primary.position.y);
    let sams = sam_sites(m);
    let front = || {
        (3..n)
            .filter(|&i| drop_zone(&w.objects[i as usize]))
            .filter_map(|i| {
                let zp = pos(&w.objects[i as usize]);
                let d = dest_of(zp)?;
                let dp = pos(&w.objects[d as usize]);
                (dist(tp, zp) >= MIN_LEG && route_clearance(&sams, tp, zp, dp) >= SAM_CLEARANCE)
                    .then_some((i, d))
            })
            .min_by(|a, b| {
                let d = |i: i32| dist(pos(&w.objects[i as usize]), aim);
                d(a.0).total_cmp(&d(b.0))
            })
    };
    // Neither kind to be had (a small theater crowded with SAMs): the route keeping farthest
    // from them however short, escorted.
    let safest = || {
        (3..n)
            .filter(|&i| drop_zone(&w.objects[i as usize]))
            .filter_map(|i| {
                let zp = pos(&w.objects[i as usize]);
                let d = dest_of(zp)?;
                let dp = pos(&w.objects[d as usize]);
                Some((route_clearance(&sams, tp, zp, dp), i, d))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, i, d)| (i, d))
    };
    let (quiet, (zone, dest)) = match (if quiet { rear() } else { None }, front()) {
        (Some((_, z, d)), _) => (true, (z, d)),
        (None, Some(zd)) => (false, zd),
        (None, None) => match rear() {
            Some((_, z, d)) => (true, (z, d)),
            None => {
                let Some(zd) = safest() else { return };
                (false, zd)
            }
        },
    };
    m.quiet = quiet;
    m.escorts = if m.quiet { 0 } else { ESCORTS };
    let zo = w.objects[zone as usize];
    let zp = pos(&zo);
    let dobj = w.objects[dest as usize];
    let dp = pos(&dobj);
    let names = |o: &WorldObject| w.name(o.name as i32).to_string();
    let (zone_name, dest_name) = (names(&zo), names(&dobj));
    let class = |o: &WorldObject| w.shape_class.get(o.shape()).copied().unwrap_or(0) & 0xF;

    let primary_summary = if m.quiet {
        format!(
            "Friendly troops at {zone_name}, well behind the lines, are short of supplies. Drop \
             them ammunition and equipment from the air."
        )
    } else {
        format!(
            "Friendly troops at {zone_name} are cut off from their supplies. Drop them \
             ammunition and equipment from the air. Two fighters escort you; stay low to keep \
             under the enemy's radars."
        )
    };
    let primary_objective =
        format!("Release the supply pallets over {zone_name}, within 4 km of its beacon.");
    let secondary_summary = format!(
        "The airbase at {dest_name} is short of munitions for its squadrons: the rest of the \
         cargo is theirs."
    );
    let secondary_objective =
        format!("Land at {dest_name} and stop on its runway to unload the cargo.");
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
