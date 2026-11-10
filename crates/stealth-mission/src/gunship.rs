//! Gunship missions (the AC-130): an enemy column on the move near an enemy ground target, out of
//! the immediate reach of the enemy's SAMs. The primary target is the column (mission type
//! [`COLUMN`]: most of its vehicles destroyed), the secondary the ground target it set off from
//! (destroy). The gunship flies there from its base, orbits the column with its guns on it and
//! lands back home.
//!
//! About half the missions are quiet ([`Mission::quiet`]): the target well away from every SAM
//! site and airfield, no enemy fighters about. The others are nearer the front, SAM sites some
//! way off and enemy fighters up, and two friendly fighters escort the gunship
//! ([`Mission::escorts`]).

use serde::{Deserialize, Serialize};

use crate::airlift::sam_sites;
use crate::world::{WorldObject, flags as F};
use crate::{GameData, MapPoint, Mission, OrdersLine, Target};

/// Mission type: destroy the enemy column (most of its vehicles).
pub const COLUMN: i16 = 11;
/// The column's vehicles: a T-90 tank, a BMP-2 infantry fighting vehicle, a ZSU-23-4 gun.
pub const T90: u8 = 0;
pub const BMP2: u8 = 1;
pub const ZSU: u8 = 2;
/// Map units a kilometre.
const KM: f64 = 64.0;
/// A quiet mission's column is caught this far from every SAM site (map units), the enemy's
/// fighters on the ground.
const QUIET_SAM: f64 = 30.0 * KM;
/// A front-line target: its nearest SAM site this far (map units); never nearer than the lower
/// bound (the "immediate reach" of a SAM launcher the gunship's orbit must stay out of).
const FRONT_SAM: (f64, f64) = (15.0 * KM, 45.0 * KM);
/// Fighters escorting a front-line gunship.
pub const ESCORTS: u8 = 2;
/// The column's road: from this far behind its place to this far ahead of it, toward the
/// friendly side (map units).
const COLUMN_BACK: f64 = 2.0 * KM;
const COLUMN_AHEAD: f64 = 14.0 * KM;

/// A gunship mission's enemy column: its vehicles ([`T90`], [`BMP2`], [`ZSU`]) and its road
/// (map units, x east, y south), set off when the gunship comes near.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Column {
    pub kinds: Vec<u8>,
    pub route: Vec<(i32, i32)>,
}

fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
    (((a.0 - b.0) as f64).powi(2) + ((a.1 - b.1) as f64).powi(2)).sqrt()
}

fn pos(o: &WorldObject) -> (i32, i32) {
    (o.x as u16 as i32, o.y as u16 as i32)
}

/// An enemy place on land the column sets off from: not a SAM or radar site, an airfield, a
/// ship or a carrier, and named.
fn ground_target(o: &WorldObject) -> bool {
    o.flags & (F::FRIENDLY | F::NEUTRAL | F::DISABLED | F::AIRFIELD | F::CARRIER | F::SHIP) == 0
        && o.unit_type == 0
        && o.name != 0
}

/// A deterministic roll from the mission's seed.
fn roll(m: &Mission) -> u32 {
    let mut x = (m.rng.holdrand_entry as u64) << 32 | m.rng.holdrand_exit as u64;
    x ^= 0x6A09_E667_F3BC_C908;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((x ^ (x >> 31)) >> 32) as u32
}

/// Turns a generated mission into a gunship's: the column and the ground target it set off
/// from, quiet or escorted (see the module). Done once; a theater without enemy ground targets
/// on land keeps the generated mission.
pub fn for_gunship(m: &mut Mission, data: &GameData) {
    if m.primary.mission_type == COLUMN {
        return;
    }
    let theater = data.theaters.get(m.settings.theater);
    let water = |p: (i32, i32)| theater.is_some_and(|t| t.is_water(p.0, p.1));
    let w = &m.world;
    let n = w.objects.len();
    let tp = (m.takeoff.position.x, m.takeoff.position.y);
    let lp = (m.landing.position.x, m.landing.position.y);
    let sams = sam_sites(m);
    let nearest = |p: (i32, i32), list: &[(i32, i32)]| {
        list.iter().map(|&q| dist(p, q)).fold(f64::MAX, f64::min)
    };
    let r = roll(m);
    let quiet = r.is_multiple_of(2);
    let candidates: Vec<usize> =
        (3..n).filter(|&i| ground_target(&w.objects[i]) && !water(pos(&w.objects[i]))).collect();
    if candidates.is_empty() {
        return;
    }
    let aim = (m.primary.position.x, m.primary.position.y);
    // Quiet: a place well away from the SAM sites near the generated target, one of the few
    // nearest by the seed; else the place farthest from them (its column caught farther on).
    let pick_quiet = || {
        let mut ok: Vec<(f64, usize)> = candidates
            .iter()
            .map(|&i| (pos(&w.objects[i]), i))
            .filter(|&(p, _)| nearest(p, &sams) >= QUIET_SAM && dist(tp, p) >= 20.0 * KM)
            .map(|(p, i)| (dist(p, aim), i))
            .collect();
        ok.sort_by(|a, b| a.0.total_cmp(&b.0));
        let few = ok.len().min(4);
        (few > 0).then(|| ok[(r >> 8) as usize % few].1)
    };
    // Front-line: the place nearest the generated target with a SAM site some way off.
    let pick_front = || {
        candidates
            .iter()
            .map(|&i| (pos(&w.objects[i]), i))
            .filter(|&(p, _)| {
                let s = nearest(p, &sams);
                (FRONT_SAM.0..=FRONT_SAM.1).contains(&s) && dist(tp, p) >= 20.0 * KM
            })
            .min_by(|a, b| dist(a.0, aim).total_cmp(&dist(b.0, aim)))
            .map(|(_, i)| i)
    };
    let safest = || {
        candidates.iter().copied().max_by(|&a, &b| {
            nearest(pos(&w.objects[a]), &sams).total_cmp(&nearest(pos(&w.objects[b]), &sams))
        })
    };
    let nearest_aim = || {
        candidates.iter().copied().min_by(|&a, &b| {
            dist(pos(&w.objects[a]), aim).total_cmp(&dist(pos(&w.objects[b]), aim))
        })
    };
    let place =
        if quiet { pick_quiet().or_else(safest) } else { pick_front().or_else(nearest_aim) };
    let Some(place) = place else { return };
    let po = w.objects[place];
    let pp = pos(&po);
    let place_name = w.name(po.name as i32).to_string();

    // The column's road: from just behind the place on toward the friendly base, bending a
    // little, kept off the water. Where it is caught is out of the immediate reach of every SAM
    // site (a quiet mission's well out of it): farther along its way when its place is nearer
    // one than that, or set off another way when that way runs past one.
    let (dx, dy) = ((tp.0 - pp.0) as f64, (tp.1 - pp.1) as f64);
    let l = dx.hypot(dy).max(1.0);
    let home = (dx / l, dy / l);
    let bend = if r & 0x100 != 0 { 0.25 } else { -0.25 };
    let catch = |need: f64| {
        // Homeward first, then turning farther and farther off it.
        let ways = [0.0f64, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0].map(|d| {
            let a = home.1.atan2(home.0) + d.to_radians();
            (a.cos(), a.sin())
        });
        let mut found = None;
        for (ux, uy) in ways {
            if let Some(s) = (0..=80).map(|k| k as f64 * KM).find(|&s| {
                let p = ((pp.0 as f64 + ux * s) as i32, (pp.1 as f64 + uy * s) as i32);
                nearest(p, &sams) >= need + KM && !water(p)
            }) {
                found = Some(((ux, uy), s));
                break;
            }
        }
        found
    };
    let (quiet, ((ux, uy), caught)) =
        match (quiet, catch(if quiet { QUIET_SAM } else { FRONT_SAM.0 })) {
            (q, Some(c)) => (q, c),
            (true, None) => match catch(FRONT_SAM.0) {
                Some(c) => (false, c),
                None => return,
            },
            (false, None) => return,
        };
    let at = |along: f64, side: f64| {
        let along = along + caught;
        (
            (pp.0 as f64 + ux * along - uy * side).round() as i32,
            (pp.1 as f64 + uy * along + ux * side).round() as i32,
        )
    };
    let mut route = vec![
        at(if caught > 0.0 { 0.0 } else { -COLUMN_BACK }, 0.0),
        at(COLUMN_AHEAD * 0.35, COLUMN_AHEAD * 0.35 * bend),
        at(COLUMN_AHEAD * 0.7, COLUMN_AHEAD * 0.35 * bend),
        at(COLUMN_AHEAD, 0.0),
    ];
    // Cut the road short where it would run into the water.
    if let Some(k) = route.iter().position(|&p| water(p)) {
        route.truncate(k.max(2));
    }
    let kinds: Vec<u8> = match (r >> 12) % 3 {
        0 => vec![BMP2, T90, BMP2, ZSU, T90],
        1 => vec![T90, BMP2, BMP2, ZSU, BMP2, T90],
        _ => vec![BMP2, BMP2, ZSU, T90, BMP2],
    };
    let start = route[0];
    m.column = Some(Column { kinds: kinds.clone(), route });
    m.quiet = quiet;
    m.escorts = if quiet { 0 } else { ESCORTS };

    let class = |o: &WorldObject| w.shape_class.get(o.shape()).copied().unwrap_or(0) & 0xF;
    let primary_summary = if quiet {
        format!(
            "An enemy column of {} armoured vehicles is moving up from {place_name}, well away \
             from the enemy's SAM sites. Find it with the targeting pod and destroy it from a \
             left-hand orbit.",
            kinds.len()
        )
    } else {
        format!(
            "An enemy column of {} armoured vehicles is moving up from {place_name}, near the \
             front. SAM sites are some way off and enemy fighters are up: two fighters escort \
             you. Destroy the column from a left-hand orbit.",
            kinds.len()
        )
    };
    let primary_objective =
        "Destroy most of the column's vehicles with the 40 mm and 105 mm guns.".to_string();
    let secondary_summary = format!("The column set off from {place_name}.");
    let secondary_objective = format!("Destroy {place_name}.");
    m.primary = Target {
        mission_type: COLUMN,
        type_name: "column".into(),
        code: 0,
        has_deadline: false,
        area_target: false,
        rubble_at_deadline: false,
        special_model: None,
        object_index: -1,
        place_name: place_name.clone(),
        shape: po.shape() as u8,
        shape_name: w.name(po.shape() as i32).into(),
        placed_by_generator: false,
        position: MapPoint { x: start.0, y: start.1 },
        role: None,
        weapon_class: class(&po),
        dist: 0,
        intelligence: primary_summary.clone(),
        objective: primary_objective.clone(),
        ..m.primary.clone()
    };
    m.secondary = Target {
        mission_type: 2,
        type_name: "destroy".into(),
        code: 0,
        has_deadline: false,
        area_target: false,
        rubble_at_deadline: false,
        special_model: None,
        object_index: place as i16,
        place_name: place_name.clone(),
        shape: po.shape() as u8,
        shape_name: w.name(po.shape() as i32).into(),
        placed_by_generator: false,
        position: MapPoint { x: pp.0, y: pp.1 },
        role: w.role(po.shape()),
        weapon_class: class(&po),
        dist: 0,
        intelligence: secondary_summary.clone(),
        objective: secondary_objective.clone(),
        ..m.secondary.clone()
    };
    // Route: halfway, the column, its place, home.
    let mid = ((tp.0 + start.0) / 2, (tp.1 + start.1) / 2);
    let point = |p: (i32, i32)| MapPoint { x: p.0, y: p.1 };
    m.waypoints = [point(mid), point(start), point(pp), point(lp)];
    m.route_length = ((dist(tp, start) + dist(start, lp)) / KM) as i16;
    m.time.has_deadline = false;
    m.time.deadline_ticks = i16::MAX;
    // The guns are the aircraft's own: nothing in the bays.
    m.loadout.usable_bays = 0;

    let b = &mut m.briefing;
    b.title = "Gunship".into();
    b.primary_summary = primary_summary;
    b.primary_objective = primary_objective;
    b.secondary_summary = secondary_summary;
    b.secondary_objective = secondary_objective;
    b.armament = vec!["256 rounds 40 mm".into(), "100 rounds 105 mm".into()];
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
        OrdersLine { text: "Destroy the enemy column".into(), color: 0x11 },
        OrdersLine { text: format!("near {place_name}"), color: 0x11 },
        OrdersLine { text: "Destroy".into(), color: 0x29 },
        OrdersLine { text: place_name.clone(), color: 0x29 },
    ];
    o.primary = format!("DESTROY THE COLUMN NEAR {}", place_name.to_uppercase());
    o.secondary = format!("DESTROY {}", place_name.to_uppercase());
}
