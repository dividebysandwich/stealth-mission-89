//! Free flight (single missions): no enemy at all. Every place and base in the theater is
//! friendly, the SAM and radar sites are gone and no enemy aircraft fly (the simulation's quiet
//! mission: only the bases' friendly traffic). The route is a round trip over three random
//! points back to the takeoff base; the targets are only the places nearest them, with nothing to
//! do there.

use crate::text::{label, translated};
use crate::world::{WorldObject, flags as F};
use crate::{GameData, MapPoint, Mission, MscRand, OrdersLine, Target};

/// Map units a kilometre.
const KM: f64 = 64.0;
/// Each leg's length (map units): a jet's, a helicopter's.
const LEG: (f64, f64) = (25.0 * KM, 70.0 * KM);
const HELI_LEG: (f64, f64) = (6.0 * KM, 18.0 * KM);

fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
    (((a.0 - b.0) as f64).powi(2) + ((a.1 - b.1) as f64).powi(2)).sqrt()
}

fn pos(o: &WorldObject) -> (i32, i32) {
    (o.x as u16 as i32, o.y as u16 as i32)
}

/// A named place on land (a route point is named after the nearest).
fn place(o: &WorldObject) -> bool {
    o.name != 0 && o.flags & (F::SHIP | F::CARRIER) == 0 && (o.x, o.y) != (0, 0)
}

/// The text `id` in the player's language, else `english`.
fn say(id: &str, english: &str, vars: &[(&str, &str)]) -> String {
    translated(id, vars).unwrap_or_else(|| {
        vars.iter().fold(english.to_string(), |s, (k, v)| s.replace(&format!("{{{k}}}"), v))
    })
}

/// Turn a generated mission into a free flight.
pub fn for_free_flight(m: &mut Mission, data: &GameData) {
    let w = &mut m.world;
    // Nobody hostile: every object friendly (neutral ones stay neutral), no radar or SAM.
    for o in &mut w.objects {
        o.unit_type = 0;
        if o.flags & F::NEUTRAL == 0 {
            o.flags |= F::FRIENDLY;
        }
    }
    // All friendly territory, no hostile concentrations.
    for g in &mut w.grid {
        if *g & 3 == 0 {
            *g |= 1;
        }
        *g &= !0x10;
    }
    m.hostile_concentrations = false;
    // No enemy aircraft: only the bases' traffic.
    m.quiet = true;
    m.escort = -1;
    m.escorts = 0;
    m.special_aircraft = None;
    m.drones = None;
    m.column = None;
    m.hunt = None;

    // The route: three random points, each a leg from the last, then home.
    let w = &m.world;
    let home = (m.takeoff.position.x, m.takeoff.position.y);
    let (lo, hi) = if m.settings.helicopter { HELI_LEG } else { LEG };
    let mut rng =
        MscRand::new(m.rng.holdrand_entry ^ 0x4652_4545 ^ ((m.settings.theater as u32) << 20));
    let places: Vec<usize> = (3..w.objects.len()).filter(|&i| place(&w.objects[i])).collect();
    // Within the theater: the box around its places.
    let pad = 5.0 * KM;
    let (x0, x1, y0, y1) = places
        .iter()
        .map(|&i| pos(&w.objects[i]))
        .fold((home.0, home.0, home.1, home.1), |(x0, x1, y0, y1), (x, y)| {
            (x0.min(x), x1.max(x), y0.min(y), y1.max(y))
        });
    let inside = |(x, y): (i32, i32)| {
        (x0 as f64 - pad..=x1 as f64 + pad).contains(&(x as f64))
            && (y0 as f64 - pad..=y1 as f64 + pad).contains(&(y as f64))
    };
    let theater = data.theaters.get(m.settings.theater);
    let land = |(x, y): (i32, i32)| theater.is_none_or(|t| !t.is_water(x, y));
    let mut stops: Vec<(i32, i32)> = Vec::new();
    let mut at = home;
    for _ in 0..3 {
        // Of the tries, the first over land near home, else over land, else anywhere inside.
        let mut best: Option<(u8, (i32, i32))> = None;
        for _ in 0..64 {
            let a = rng.r(3600) as f64 / 3600.0 * std::f64::consts::TAU;
            let d = lo + rng.r((hi - lo) as i32) as f64;
            let p = (at.0 + (d * a.sin()) as i32, at.1 - (d * a.cos()) as i32);
            if !inside(p) {
                continue;
            }
            let rank = match (land(p), dist(home, p) <= hi * 1.2) {
                (true, true) => 0,
                (true, false) => 1,
                _ => 2,
            };
            if best.is_none_or(|(r, _)| rank < r) {
                best = Some((rank, p));
            }
            if rank == 0 {
                break;
            }
        }
        let p = best.map_or(at, |(_, p)| p);
        stops.push(p);
        at = p;
    }
    let point = |p: (i32, i32)| MapPoint { x: p.0, y: p.1 };
    let stop = |k: usize| stops.get(k).copied().unwrap_or(home);
    m.waypoints = [point(stop(0)), point(stop(1)), point(stop(2)), point(home)];
    let legs = [home, stop(0), stop(1), stop(2), home];
    m.route_length = (legs.windows(2).map(|l| dist(l[0], l[1])).sum::<f64>() / KM) as i16;
    // Each point by its nearest place.
    let nearest =
        |k: usize| places.iter().copied().min_by_key(|&i| dist(stop(k), pos(&w.objects[i])) as i64);
    let name = |k: usize| {
        nearest(k)
            .map_or_else(|| m.takeoff.name.clone(), |i| w.name(w.objects[i].name as i32).into())
    };
    let names = [name(0), name(1), name(2)];

    // The targets: the second and third places, nothing to do there.
    let target = |k: usize, t: &Target| -> Target {
        let i = nearest(k);
        let o = i.map(|i| w.objects[i]);
        Target {
            mission_type: 0,
            type_name: "free flight".into(),
            template: t.template,
            template_id: String::new(),
            code: 0,
            has_deadline: false,
            area_target: false,
            rubble_at_deadline: false,
            special_model: None,
            object_index: i.map_or(m.takeoff.object_index, |i| i as i16),
            base_index: m.takeoff.object_index,
            place_name: names[k].clone(),
            shape: o.map_or(0, |o| o.shape() as u8),
            shape_name: o.map_or(String::new(), |o| w.name(o.shape() as i32).into()),
            placed_by_generator: false,
            position: point(stop(k)),
            role: None,
            weapon_class: 0,
            onc: t.onc.clone(),
            onc_bytes: t.onc_bytes,
            dist: 0,
            intelligence: String::new(),
            objective: String::new(),
        }
    };
    m.primary = target(1, &m.primary);
    m.secondary = target(2, &m.secondary);
    let old_landing = m.landing.name.clone();
    m.landing = m.takeoff.clone();
    // No hurry.
    m.time.has_deadline = false;
    m.time.deadline_ticks = i16::MAX;

    // Briefing and orders.
    let route = names.join(" - ");
    let home_name = m.takeoff.name.clone();
    let summary = say(
        "ff-summary",
        "No enemy forces anywhere: only friendly aircraft and units. Fly wherever you like.",
        &[],
    );
    let objective = say(
        "ff-objective",
        "Suggested route: {route}, then back to {base}.",
        &[("route", &route), ("base", &home_name)],
    );
    let b = &mut m.briefing;
    b.title = say("ff-title", "Free Flight", &[]);
    b.primary_summary = summary;
    b.primary_objective = objective;
    b.secondary_summary = String::new();
    b.secondary_objective = String::new();
    b.land_line = b.land_line.replace(&old_landing, &home_name);
    b.roe_text = String::new();
    let l = &data.briefings.labels;
    let mut lines = vec![
        b.header.clone(),
        b.title.clone(),
        b.pilot_line.clone(),
        b.base_date_line.clone(),
        b.primary_summary.clone(),
        label("bt-label-objective", &l.objective),
        b.primary_objective.clone(),
        label("bt-label-flight-plan", &l.flight_plan),
        b.depart_line.clone(),
        b.land_line.clone(),
        label("bt-label-armament", &l.armament),
    ];
    lines.extend(b.armament.iter().cloned());
    lines.push(b.fuel_line.clone());
    lines.retain(|l| !l.is_empty());
    b.lines = lines;
    let o = &mut m.orders;
    o.lines = vec![
        OrdersLine { text: "Free flight".into(), color: 0x11 },
        OrdersLine { text: "No enemy forces".into(), color: 0x11 },
        OrdersLine { text: "Fly the waypoints".into(), color: 0x29 },
        OrdersLine { text: "or anywhere you like".into(), color: 0x29 },
    ];
    o.primary = "FREE FLIGHT".into();
    o.secondary = "FLY THE WAYPOINTS".into();
    o.return_to = format!("RETURN TO {}", home_name.to_uppercase());
}
