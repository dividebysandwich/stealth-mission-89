//! Anti-submarine missions (the MH-60R Seahawk): from a friendly carrier, find and sink the
//! enemy submarines before they do what they are out to do. Three kinds ([`Scenario`]):
//!
//! - **Blockade**: a submarine slips out of an enemy port; the line it must not cross lies
//!   between there and the carrier group.
//! - **Convoy**: it closes on one of the carrier group's escorts (a theater ship near the
//!   carrier, picked by the simulation) and torpedoes it.
//! - **Carrier**: it goes for the carrier itself.
//!
//! The primary target (mission type [`HUNT`]) is the reported datum, where the submarine was
//! last seen; it is achieved when every submarine of the mission is sunk ([`Hunt::subs`], run by
//! the simulation's `asw`). Only theaters with a friendly carrier have these missions; elsewhere
//! the generated mission stays as it is.

use serde::{Deserialize, Serialize};

use crate::world::{WorldObject, flags as F};
use crate::{Base, GameData, MapPoint, Mission, OrdersLine, Target};

/// Mission type: hunt the submarines.
pub const HUNT: i16 = 12;
/// Map units a kilometre.
const KM: f64 = 64.0;
/// How far from what it goes for a submarine starts (map units).
const START: (f64, f64) = (22.0 * KM, 28.0 * KM);
/// How far off its true start the reported datum is at most (map units).
const DATUM_ERROR: f64 = 3.0 * KM;

/// What the submarines are out to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Scenario {
    Blockade,
    Convoy,
    #[default]
    Carrier,
}

impl Scenario {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "blockade" => Some(Scenario::Blockade),
            "convoy" => Some(Scenario::Convoy),
            "carrier" => Some(Scenario::Carrier),
            _ => None,
        }
    }
}

/// What one submarine is after (map units, x east, y south; world object indices).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubIntent {
    Blockade { a: (i32, i32), b: (i32, i32), goal: (i32, i32) },
    Approach { object: i16 },
    Attack { object: i16 },
    /// One of this carrier's escorts (the simulation picks the theater ship).
    Escort { carrier: i16 },
}

/// A submarine of the mission: where it starts (map units), heading to begin with (degrees),
/// its aim, nuclear or diesel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubPlan {
    pub at: (i32, i32),
    pub heading: i16,
    pub intent: SubIntent,
    pub nuclear: bool,
}

/// An anti-submarine mission's hunt: the water (the layer's depth in m, the sea state) and the
/// submarines.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Hunt {
    pub scenario: Scenario,
    pub subs: Vec<SubPlan>,
    pub layer: u16,
    pub sea_state: u8,
    /// The reported datum (map units).
    pub datum: (i32, i32),
}

fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
    (((a.0 - b.0) as f64).powi(2) + ((a.1 - b.1) as f64).powi(2)).sqrt()
}

fn pos(o: &WorldObject) -> (i32, i32) {
    (o.x as u16 as i32, o.y as u16 as i32)
}

/// A deterministic roll from the mission's seed (another stream than the gunship's).
fn roll(m: &Mission, salt: u64) -> u32 {
    let mut x = (m.rng.holdrand_entry as u64) << 32 | m.rng.holdrand_exit as u64;
    x ^= 0x3C6E_F372_FE94_F82B ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((x ^ (x >> 31)) >> 32) as u32
}

/// Turns a generated mission into a Seahawk's hunt (see the module), the scenario `choice` or one
/// by the seed. Done once; without a friendly carrier the mission is left as it is.
pub fn for_asw(m: &mut Mission, data: &GameData, choice: Option<Scenario>) {
    if m.primary.mission_type == HUNT {
        return;
    }
    let theater = data.theaters.get(m.settings.theater);
    let water = |p: (i32, i32)| theater.is_some_and(|t| t.is_water(p.0, p.1));
    let w = &m.world;
    let n = w.objects.len();
    let friendly = |o: &WorldObject| o.flags & F::FRIENDLY != 0;
    let Some(carrier) = (3..n)
        .filter(|&i| w.objects[i].flags & F::CARRIER != 0 && friendly(&w.objects[i]))
        .min_by_key(|&i| dist(pos(&w.objects[i]), (m.takeoff.position.x, m.takeoff.position.y)) as i64)
    else {
        return;
    };
    let cp = pos(&w.objects[carrier]);
    let r = roll(m, 1);
    // A point `d` map units from `from` at a bearing the seed picks, on the sea.
    let sea_point = |from: (i32, i32), d: (f64, f64), salt: u32| {
        (0..48).find_map(|k| {
            let a = ((salt.wrapping_add(k * 7919) % 360) as f64).to_radians();
            let dd = d.0 + (d.1 - d.0) * ((salt >> 8).wrapping_add(k) % 100) as f64 / 100.0;
            let p = ((from.0 as f64 + a.sin() * dd) as i32, (from.1 as f64 - a.cos() * dd) as i32);
            // Open water round it too.
            let open = (0..8).all(|j| {
                let b = (j as f64 * 45.0).to_radians();
                water(((p.0 as f64 + b.sin() * KM) as i32, (p.1 as f64 - b.cos() * KM) as i32))
            });
            (water(p) && open).then_some(p)
        })
    };
    let heading_to = |a: (i32, i32), b: (i32, i32)| {
        ((b.0 - a.0) as f64).atan2((a.1 - b.1) as f64).to_degrees().rem_euclid(360.0) as i16
    };
    let mut scenario = choice.unwrap_or(match r % 3 {
        0 => Scenario::Blockade,
        1 => Scenario::Convoy,
        _ => Scenario::Carrier,
    });
    let count = if m.settings.opponents >= 3 { 2 } else { 1 };
    let nuclear = r & 0x10 != 0;
    // An enemy port: an enemy object by the sea, nearest the carrier but some way off it.
    let port = (3..n)
        .filter(|&i| {
            let o = &w.objects[i];
            !friendly(o) && o.flags & F::NEUTRAL == 0 && o.name != 0 && !water(pos(o))
        })
        .filter(|&i| dist(pos(&w.objects[i]), cp) > 45.0 * KM)
        .filter_map(|i| sea_point(pos(&w.objects[i]), (2.0 * KM, 6.0 * KM), r ^ i as u32).map(|p| (i, p)))
        .min_by_key(|&(i, _)| dist(pos(&w.objects[i]), cp) as i64);
    if scenario == Scenario::Blockade && port.is_none() {
        scenario = Scenario::Carrier;
    }
    let mut subs = Vec::new();
    let (protect, place_name, area);
    match scenario {
        Scenario::Blockade => {
            let (pi, start) = port.expect("checked");
            let (dx, dy) = ((cp.0 - start.0) as f64, (cp.1 - start.1) as f64);
            let l = dx.hypot(dy).max(1.0);
            let (ux, uy) = (dx / l, dy / l);
            let at = |s: f64, side: f64| {
                ((start.0 as f64 + ux * s - uy * side) as i32, (start.1 as f64 + uy * s + ux * side) as i32)
            };
            let line = l * 0.4;
            let a = at(line, -15.0 * KM);
            let b = at(line, 15.0 * KM);
            let goal = at(l * 0.75, 0.0);
            for k in 0..count {
                let p = at(0.0, (k as f64 - 0.5) * KM * 2.0 * (count - 1) as f64);
                let p = if water(p) { p } else { start };
                subs.push(SubPlan { at: p, heading: heading_to(p, goal), intent: SubIntent::Blockade { a, b, goal }, nuclear });
            }
            protect = None;
            place_name = w.name(w.objects[pi].name as i32).to_string();
            area = at(line, 0.0);
        }
        Scenario::Convoy | Scenario::Carrier => {
            let target = carrier;
            let tp = pos(&w.objects[target]);
            for k in 0..count {
                let Some(p) = sea_point(tp, START, r.wrapping_add(k * 97)) else { continue };
                let intent = if scenario == Scenario::Convoy {
                    SubIntent::Escort { carrier: target as i16 }
                } else {
                    SubIntent::Attack { object: target as i16 }
                };
                subs.push(SubPlan { at: p, heading: heading_to(p, tp), intent, nuclear });
            }
            protect = Some(target);
            place_name = if scenario == Scenario::Convoy {
                format!("{}'s escorts", w.name(w.objects[target].name as i32))
            } else {
                w.name(w.objects[target].name as i32).to_string()
            };
            area = subs.first().map_or(tp, |s| s.at);
        }
    }
    if subs.is_empty() {
        return;
    }
    // The datum: where the first was reported, a few kilometres off.
    let off = (r >> 16) as f64 / 65536.0 * DATUM_ERROR;
    let ang = ((r >> 4) % 360) as f64;
    let datum = ((area.0 as f64 + ang.to_radians().sin() * off) as i32, (area.1 as f64 - ang.to_radians().cos() * off) as i32);
    let layer = 30 + (r >> 20) % 91;
    let sea_state = 1 + ((r >> 24) % 4) as u8;
    m.hunt = Some(Hunt { scenario, subs: subs.clone(), layer: layer as u16, sea_state, datum });

    // From the carrier and back to it.
    let co = w.objects[carrier];
    let base = Base {
        object_index: carrier as i16,
        name: w.name(co.name as i32).to_string(),
        position: MapPoint { x: cp.0, y: cp.1 },
        carrier: true,
        neutral: false,
        distance: 0,
    };
    m.takeoff = base.clone();
    m.landing = base;
    m.exit_code = 0x0D;
    let carrier_name = m.takeoff.name.clone();
    let boats = if subs.len() > 1 { "Two enemy submarines are" } else { "An enemy submarine is" };
    let primary_summary = match scenario {
        Scenario::Blockade => format!(
            "{boats} putting out from {place_name} to break our blockade. Hold the line between \
             there and the carrier group: none may slip through."
        ),
        Scenario::Convoy => format!(
            "{boats} closing on the {place_name}. Find {} before {} get within torpedo range.",
            if subs.len() > 1 { "them" } else { "it" },
            if subs.len() > 1 { "they" } else { "it" }
        ),
        Scenario::Carrier => format!(
            "{boats} working toward the carrier group. Find and sink {} before {} can fire on the \
             {carrier_name}.",
            if subs.len() > 1 { "them" } else { "it" },
            if subs.len() > 1 { "they" } else { "it" }
        ),
    };
    let primary_objective = format!(
        "Search from the datum with sonobuoys, the dipping sonar and the MAD; sink {} with the \
         Mk 54 torpedoes. The layer is reported at about {layer} m: drop a BT buoy to be sure.",
        if subs.len() > 1 { "both submarines" } else { "the submarine" }
    );
    let secondary_summary = format!("Recover aboard the {carrier_name}.");
    let secondary_objective = "Sink every submarine, then land back on the carrier.".to_string();
    let template = m.primary.clone();
    let hunt_target = |summary: &String, objective: &String| Target {
        mission_type: HUNT,
        type_name: "hunt".into(),
        code: 0,
        has_deadline: false,
        area_target: false,
        rubble_at_deadline: false,
        special_model: None,
        object_index: protect.map_or(-1, |p| p as i16),
        // The simulation starts at the primary target's base: the carrier.
        base_index: carrier as i16,
        place_name: place_name.clone(),
        placed_by_generator: false,
        position: MapPoint { x: datum.0, y: datum.1 },
        role: None,
        dist: 0,
        intelligence: summary.clone(),
        objective: objective.clone(),
        ..template.clone()
    };
    m.primary = hunt_target(&primary_summary, &primary_objective);
    m.secondary = hunt_target(&secondary_summary, &secondary_objective);
    let point = |p: (i32, i32)| MapPoint { x: p.0, y: p.1 };
    let mid = ((cp.0 + datum.0) / 2, (cp.1 + datum.1) / 2);
    m.waypoints = [point(mid), point(datum), point(datum), point(cp)];
    m.route_length = (2.0 * dist(cp, datum) / KM) as i16;
    m.time.has_deadline = false;
    m.time.deadline_ticks = i16::MAX;
    m.loadout.usable_bays = 0;
    m.quiet = true;
    m.escorts = 0;

    let b = &mut m.briefing;
    b.title = "Submarine hunt".into();
    b.primary_summary = primary_summary;
    b.primary_objective = primary_objective;
    b.secondary_summary = secondary_summary;
    b.secondary_objective = secondary_objective;
    b.armament = vec!["2 Mk 54 torpedoes".into(), "4 AGM-114 Hellfire".into(), "8 sonobuoys (4 DIFAR, 2 DICASS, 2 BT)".into()];
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
    let what = match scenario {
        Scenario::Blockade => format!("off {place_name}"),
        _ => format!("near the {place_name}"),
    };
    let o = &mut m.orders;
    o.lines = vec![
        OrdersLine { text: "Hunt the enemy submarine".into(), color: 0x11 },
        OrdersLine { text: what.clone(), color: 0x11 },
        OrdersLine { text: "Recover aboard".into(), color: 0x29 },
        OrdersLine { text: carrier_name.clone(), color: 0x29 },
    ];
    o.primary = format!("SINK THE SUBMARINE {}", what.to_uppercase());
    o.secondary = format!("RECOVER ABOARD {}", carrier_name.to_uppercase());
}
