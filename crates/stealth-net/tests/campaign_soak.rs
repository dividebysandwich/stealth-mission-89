//! A campaign left to the AI (no players), run as fast as it goes: every campaign aircraft is
//! watched once a second and what looks wrong is reported (crashes nobody caused, flights that
//! never land, landings away from an airfield, strikes that turn home unfired, circling).
//!
//! `cargo test --release -p stealth-net --test campaign_soak -- --ignored --nocapture`, with
//! `SOAK_THEATER` (central_europe), `SOAK_HOURS` (campaign-clock hours of sim time, 3),
//! `SOAK_SEED` (1), `SOAK_CREW` (players online for the enemy's tempo, none flying; 1).

use std::collections::BTreeMap;

use stealth_campaign::{CampaignSetup, Side};
use stealth_host::{Host, Rules, theater};
use stealth_sim::campaign::Role;
use stealth_sim::types::fflags;
use stealth_sim::{AircraftKind, SimEvent};

const FS: u8 = 15;

#[derive(Default)]
struct Track {
    tag: u32,
    unit: usize,
    blue: bool,
    role: Option<Role>,
    spec: i32,
    first: f64,
    last: f64,
    rtb_since: Option<f64>,
    final_since: Option<f64>,
    still_since: Option<f64>,
    min_agl: i32,
    turned: f64,
    turn_window: Vec<(f64, f64, i32)>,
    notes: Vec<String>,
    end: Option<String>,
    destroyed: bool,
    rtb_armed: bool,
    /// The last half minute, a line a second.
    trail: std::collections::VecDeque<String>,
    dumped: usize,
}

impl Track {
    fn note(&mut self, n: String) {
        self.notes.push(n);
        if self.dumped < 3 {
            self.dumped += 1;
            let trail: Vec<String> = self.trail.iter().cloned().collect();
            self.notes.extend(trail.into_iter().map(|l| format!("    | {l}")));
        }
    }
}

/// A ground unit's class for the losses table.
fn ground_class(k: stealth_sim::aaa::GroundKind) -> &'static str {
    use stealth_sim::aaa::GroundKind as G;
    match k {
        G::PatriotRadar | G::PatriotLauncher | G::NasamsRadar | G::NasamsLauncher => "sam",
        G::Gepard | G::Zsu | G::Zsu572 | G::S60 => "aaa",
        G::Manpads => "manpads",
        k if k.armour() => "armour",
        k if k.supply() => "supply",
        _ => "other",
    }
}

fn env<T: std::str::FromStr>(k: &str, d: T) -> T {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

#[test]
#[ignore]
fn campaign_soak() {
    let code: String = env("SOAK_THEATER", "central_europe".to_string());
    let hours: f64 = env("SOAK_HOURS", 3.0);
    let seed: u32 = env("SOAK_SEED", 1);
    let data = theater::game_data().unwrap();
    let setup = CampaignSetup {
        theater: theater::theater_index(&code).unwrap(),
        theater_code: code.clone(),
        roe: 2,
        opponents: 1,
        seed,
        sky_ratio: 24.0,
        start_hour: 8.0,
        dynamic_weather: true,
        crew_scaling: true,
    };
    let rules = Rules {
        realistic_damage: true,
        realistic_guidance: true,
        continuous_time: true,
        ..Default::default()
    };
    let mut host = Host::new(setup.clone(), &data, rules).unwrap();
    // A comparison: the enemy's bases supplied by air, not by road.
    if std::env::var("SOAK_NO_ROADS").is_ok() {
        host.state.terrain = host.state.terrain.without_roads();
    }
    let sim_data = theater::sim_data(setup.theater, &code).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    // `SOAK_CREW`: players online (none flying), as the enemy's tempo counts them.
    host.state.set_crew(env("SOAK_CREW", 1usize));
    let relief = sim.relief();
    let hills = sim.hills();
    // Place owners at the start, and every change of hands since (captured by side).
    let owners0: Vec<Side> = host.state.places.iter().map(|p| p.owner).collect();
    let mut owners = owners0.clone();
    let mut captures = [0usize; 2];
    let dt = 1.0 / FS as f64;
    let frames = (hours * 3600.0 * FS as f64) as u64;
    let mut tracks: BTreeMap<u32, Track> = BTreeMap::new();
    let mut ground_losses: BTreeMap<(&str, &str, String), usize> = BTreeMap::new();
    let mut ground_log: Vec<String> = Vec::new();
    let mut supply_log: Vec<String> = Vec::new();
    let mut friendly_fire: Vec<String> = Vec::new();
    // Ground units a gun's round is on its way to kill (target, shooter).
    let mut gun_kills: BTreeMap<usize, usize> = BTreeMap::new();
    let mut by_unit: BTreeMap<usize, u32> = BTreeMap::new();
    let mut t = 0.0f64;
    let mut log: Vec<String> = Vec::new();
    let mut threats: BTreeMap<usize, Vec<i32>> = BTreeMap::new();
    let mut news: Vec<String> = Vec::new();
    let mut missions: BTreeMap<u32, stealth_campaign::AiMission> = BTreeMap::new();
    let mut shelled: BTreeMap<usize, usize> = BTreeMap::new();
    let mut cargo: BTreeMap<u32, (stealth_campaign::CargoRun, f64)> = BTreeMap::new();
    // The enemy's SAM sites by class (object, class), their launches since the last sample,
    // and the samples.
    use stealth_campaign::Munition;
    let sam_class = |sim: &stealth_sim::Sim, i: usize| {
        let o = sim.objects()[i];
        let red = o.flags & (stealth_sim::oflags::FRIENDLY | stealth_sim::oflags::NEUTRAL) == 0;
        (o.unit_type != 0 && red && !sim.object_is_ship(i))
            .then(|| Munition::sam(sim.site_missile(o.unit_type)))
    };
    let classes = [Munition::SamLong, Munition::SamMedium, Munition::SamShort];
    let mut mags: BTreeMap<u16, i32> = BTreeMap::new();
    let mut launches = [0usize; 3];
    let mut sam_log: Vec<String> = Vec::new();
    let full_mag = stealth_sim::munitions::SITE_MAGAZINE;
    // The friendly side's munitions, sampled every 10 min: per munition the lowest total at
    // the bases, samples with a base out of it and the most bases out at once; the air
    // defence's empty units; an hourly timeline.
    let blue_items = Munition::of_side(Side::Blue);
    let mut blue_low = vec![u32::MAX; blue_items.len()];
    let mut blue_out_samples = vec![0usize; blue_items.len()];
    let mut blue_out_max = vec![0usize; blue_items.len()];
    let mut blue_log: Vec<String> = Vec::new();
    let mut ad_log: Vec<String> = Vec::new();
    let mut ad_dry = [0usize; 2];
    let mut samples = 0usize;
    // Supply by road: convoys on the road per sample, each run's time on it, and samples with
    // a road-served base out of something.
    let mut road_on: Vec<usize> = Vec::new();
    let mut road_runs: BTreeMap<u32, (f64, String)> = BTreeMap::new();
    let mut road_out = 0usize;
    let interdict: f64 = std::env::var("SOAK_INTERDICT").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let mut struck = 0usize;
    let mut fleet_dry = 0usize;
    let road_bases: Vec<u16> = host
        .state
        .sites
        .iter()
        .filter(|s| s.kind == stealth_campaign::SiteKind::Airfield)
        .filter(|s| {
            ["Taganrog", "Berdiansk", "Melitopol", "Dzhankoi", "Kirovske", "Hvardiiske", "Saky", "Belbek"]
                .contains(&s.name.as_str())
        })
        .map(|s| s.object)
        .collect();
    for frame in 0..frames {
        t = frame as f64 * dt;
        // What is after each aircraft (for the report of what shot it down).
        threats.clear();
        for m in sim.ai_missiles().iter().filter(|m| m.ground.is_none()) {
            threats.entry(m.target).or_default().push(m.spec);
            // A missile after an aircraft of its own side.
            if let Some(f) = sim.campaign_flight(m.target)
                && f.blue == m.blue
                && sim.flights()[m.target].flags & fflags::EXISTS != 0
                && friendly_fire.len() < 20
                && !friendly_fire.iter().any(|l: &String| l.contains(&format!("tag {} ", f.tag)))
            {
                friendly_fire.push(format!(
                    "{t:6.0}s {} missile {} after its own side's tag {} ({:?} {})",
                    if m.blue { "blue" } else { "red" },
                    m.spec,
                    f.tag,
                    f.role,
                    sim.aircraft_name(sim.flights()[m.target].spec)
                ));
            }
        }
        shelled.clear();
        for (target, _) in sim.ai_shell_targets() {
            *shelled.entry(target).or_default() += 1;
        }
        // Players' strikes on the supply convoys (`SOAK_INTERDICT` trucks an hour): the truck
        // furthest along the roads.
        if interdict > 0.0 && frame > 0 && frame % ((3600.0 / interdict / dt) as u64).max(1) == 0 {
            let truck = sim
                .convoys()
                .iter()
                .filter(|c| c.supply)
                .flat_map(|c| c.units.iter().map(|&k| k as usize))
                .find(|&k| sim.units()[k].kind.truck() && !sim.units()[k].destroyed);
            if let Some(k) = truck {
                sim.units_mut()[k].destroyed = true;
                struck += 1;
            }
        }
        let mut events = sim.step_pilots(&[], FS);
        events.extend(host.take_sim_events(&mut sim));
        for e in host.tick(&mut sim, dt, &events) {
            if let stealth_host::HostEvent::Campaign(stealth_campaign::CampaignEvent::Message(m)) =
                e
            {
                news.push(format!("{t:6.0}s {}", m.tr()));
            }
        }
        for m in &host.state.ai_missions {
            missions.insert(m.id, m.clone());
        }
        for r in &host.state.cargo_runs {
            // Keep the escorts it had once they are gone.
            let mut r = r.clone();
            if let Some((old, _)) = cargo.get(&r.tag) {
                for &e in &old.escorts {
                    if !r.escorts.contains(&e) {
                        r.escorts.push(e);
                    }
                }
            }
            if r.road.is_some() && r.state == stealth_campaign::CargoState::Airborne {
                let to = host.state.site(r.to).map_or(String::new(), |s| s.name.clone());
                road_runs.entry(r.tag).or_insert((t, to));
            }
            cargo.insert(r.tag, (r, t));
        }
        let now: BTreeMap<u16, i32> = sim.site_magazines().into_iter().collect();
        for i in 0..sim.ground_unit_count() {
            let Some(m) = sam_class(&sim, i) else { continue };
            let (a, b) = (
                mags.get(&(i as u16)).copied().unwrap_or(full_mag),
                now.get(&(i as u16)).copied().unwrap_or(full_mag),
            );
            if b < a {
                launches[classes.iter().position(|&c| c == m).unwrap()] += (a - b) as usize;
            }
        }
        mags = now;
        if frame % (FS as u64 * 600) == 0 {
            let st = &host.state;
            let prod = st.production_base(Side::Red);
            let mut line = format!("{:5.0} min:", t / 60.0);
            for (k, &m) in classes.iter().enumerate() {
                let sites: Vec<usize> = (0..sim.ground_unit_count())
                    .filter(|&i| sam_class(&sim, i) == Some(m))
                    .filter(|&i| sim.objects()[i].flags & stealth_sim::oflags::DESTROYED == 0)
                    .collect();
                let dry = sites.iter().filter(|&&i| mags.get(&(i as u16)) == Some(&0)).count();
                let bases: u32 = st
                    .stocks
                    .iter()
                    .filter(|(b, s)| s.side == Some(Side::Red) && Some(**b) != prod)
                    .map(|(_, s)| s.count(m))
                    .sum();
                let at_prod = prod.and_then(|p| st.stocks.get(&p)).map_or(0, |s| s.count(m));
                line += &format!(
                    "  {m:?}: fired {:3}, dry {dry:2}/{:2}, bases {bases:3}, prod {at_prod:3}",
                    launches[k],
                    sites.len()
                );
            }
            launches = [0; 3];
            sam_log.push(line);
            // Supply by road.
            road_on.push(sim.convoys().iter().filter(|c| c.supply).count());
            fleet_dry += (st.road_net().is_some() && st.trucks.ready < 3.0) as usize;
            road_out += st
                .stocks
                .keys()
                .filter(|&&b| road_bases.contains(&b) && !st.out_of(b, Side::Red).is_empty())
                .count();
            // The friendly side.
            samples += 1;
            let prod = st.production_base(Side::Blue);
            let mut hourly = format!("{:5.0} min:", t / 60.0);
            for (k, &m) in blue_items.iter().enumerate() {
                let bases: Vec<u32> = st
                    .stocks
                    .iter()
                    .filter(|(b, s)| s.side == Some(Side::Blue) && Some(**b) != prod)
                    .map(|(_, s)| s.count(m))
                    .collect();
                let total: u32 = bases.iter().sum();
                let out = bases.iter().filter(|&&n| n == 0).count();
                blue_low[k] = blue_low[k].min(total);
                blue_out_samples[k] += (out > 0) as usize;
                blue_out_max[k] = blue_out_max[k].max(out);
                if matches!(
                    m,
                    Munition::Aim120
                        | Munition::Harm
                        | Munition::Agm65
                        | Munition::Pac3
                        | Munition::GepardRounds
                ) {
                    let p = prod.and_then(|p| st.stocks.get(&p)).map_or(0, |s| s.count(m));
                    hourly += &format!("  {m:?} {total:4}/{p:4} out {out}");
                }
            }
            if frame % (FS as u64 * 3600) == 0 {
                blue_log.push(hourly);
            }
            // Patriot batteries and Gepards with nothing left to fire.
            use stealth_sim::aaa::GroundKind;
            let units = sim.units();
            let mut dry = [0usize; 2];
            let mut all = [0usize; 2];
            for k in 0..units.len() {
                let u = units[k];
                if u.destroyed || !sim.aaa_blue(k) {
                    continue;
                }
                let empty = match u.kind {
                    GroundKind::PatriotRadar => {
                        all[0] += 1;
                        let ready: i32 =
                            sim.battery_launchers(k).iter().map(|&l| units[l].rounds).sum();
                        ready == 0 && sim.stockpile(k) == 0
                    }
                    GroundKind::Gepard => {
                        all[1] += 1;
                        u.rounds == 0 && sim.stockpile(k) == 0
                    }
                    _ => continue,
                };
                if empty {
                    dry[(u.kind == GroundKind::Gepard) as usize] += 1;
                }
            }
            ad_dry[0] += dry[0];
            ad_dry[1] += dry[1];
            if dry != [0, 0] {
                ad_log.push(format!(
                    "{:5.0} min: Patriots empty {}/{}, Gepards empty {}/{}, resupplies open {}",
                    t / 60.0,
                    dry[0],
                    all[0],
                    dry[1],
                    all[1],
                    st.supply_requests.len()
                ));
            }
        }
        host.refills(&mut sim, dt as f32);
        host.take_sim_events(&mut sim);
        // Ground units lost, by side and class, and what got them: a ground gun's round, else
        // the nearest enemy aircraft (within 8 km) by role, else something else (missiles,
        // drones, ships).
        for e in &events {
            if let SimEvent::GroundShot { from, to, kill: true, .. } = e.event {
                gun_kills.insert(to, from);
            }
        }
        for e in &events {
            let SimEvent::AaaDestroyed { unit, by } = e.event else { continue };
            let Some(u) = sim.units().get(unit).copied() else { continue };
            let blue = sim.aaa_blue(unit);
            let cause = if by.is_some() {
                "player".to_string()
            } else if let Some(from) = gun_kills.remove(&unit) {
                format!("gun {:?}", sim.units()[from].kind)
            } else {
                (0..sim.flights().len())
                    .filter(|&i| sim.flights()[i].flags & fflags::EXISTS != 0)
                    .filter_map(|i| Some((i, sim.campaign_flight(i)?)))
                    .filter(|(_, f)| f.blue != blue)
                    .map(|(i, f)| {
                        let a = sim.flights()[i];
                        let d = ((a.x - u.x) as f64).hypot((a.y - u.y) as f64);
                        (d, format!("air {:?} {}", f.role, a.spec))
                    })
                    .filter(|(d, _)| *d < 512.0)
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .map_or("other".to_string(), |(_, c)| c)
            };
            let side = if blue { "blue" } else { "red" };
            *ground_losses.entry((side, ground_class(u.kind), cause)).or_insert(0) += 1;
        }
        if frame % (FS as u64 * 3600) == 0 {
            let mut line = format!("{:5.0} min:", t / 60.0);
            for blue in [true, false] {
                let mut n: BTreeMap<&str, usize> = BTreeMap::new();
                for k in 0..sim.units().len() {
                    let u = sim.units()[k];
                    if !u.destroyed && sim.aaa_blue(k) == blue {
                        *n.entry(ground_class(u.kind)).or_insert(0) += 1;
                    }
                }
                line += &format!("  {}: {n:?}", if blue { "blue" } else { "red" });
            }
            let st = &host.state;
            for side in [Side::Blue, Side::Red] {
                let f = st.forces(side);
                line += &format!(
                    "\n           {side:?} fighters {} helis {} materiel {:.0} ({:?})",
                    f.fighters, f.helicopters, f.materiel, f.ledger
                );
            }
            let flyable = st.board.iter().filter(|o| o.kind.ai_flown()).count();
            let queue: Vec<String> = st
                .ai_schedule(Side::Blue)
                .iter()
                .take(3)
                .filter_map(|(id, at)| {
                    let o = st.board.iter().find(|o| o.id == *id)?;
                    Some(format!("{:?}@{:.0}s base {}", o.kind, at - st.clock, o.base))
                })
                .collect();
            line += &format!(
                "\n           board {} (AI {flyable}), open blue AI {}, player bases {:?}, next {queue:?}",
                st.board.len(),
                st.ai_missions.iter().filter(|m| m.side == Side::Blue && m.open()).count(),
                st.player_bases()
            );
            ground_log.push(line);
        }
        for e in &events {
            match &e.event {
                SimEvent::AircraftCrashed { unit } => {
                    if let Some(tr) = by_unit.get(unit).and_then(|tag| tracks.get_mut(tag))
                        && !tr.destroyed
                        && tr.end.is_none()
                    {
                        tr.note(format!("{t:.0}s crashed into the ground unhit"));
                    }
                }
                SimEvent::CampaignFlightDestroyed { unit, tag, by_player, radar, .. } => {
                    if let Some(tr) = tracks.get_mut(tag) {
                        if tr.role == Some(Role::Supply) {
                            let u = sim.flights()[*unit];
                            supply_log.push(format!(
                                "{t:6.0}s {} supply {tag} down at ({}, {}) after {:.0}s missiles {:?} shells {}",
                                if tr.blue { "blue" } else { "red" },
                                u.x,
                                u.y,
                                t - tr.first,
                                threats.get(unit).cloned().unwrap_or_default(),
                                shelled.get(unit).copied().unwrap_or(0),
                            ));
                            if supply_log.len() <= 6 {
                                supply_log.extend(tr.trail.iter().rev().take(4).rev().map(|l| format!("    | {l}")));
                            }
                        }
                        tr.destroyed = true;
                        let u = sim.flights()[*unit];
                        let agl = u.alt - relief.height(u.x, u.y).max(hills.ground_alt(u.x, u.y));
                        tr.end = Some(format!(
                            "shot down at {t:.0}s{}{} missiles {:?} shells {} agl {agl}",
                            if *by_player { " (by player?)" } else { "" },
                            radar.map_or(String::new(), |r| format!(" by SAM radar {r}")),
                            threats.get(unit).cloned().unwrap_or_default(),
                            shelled.get(unit).copied().unwrap_or(0),
                        ));
                    }
                }
                SimEvent::CampaignFlightLanded { tag, x, y, .. } => {
                    if let Some(tr) = tracks.get_mut(tag) {
                        let kind = sim.aircraft_kind(tr.spec);
                        // Nearest airfield of its side.
                        let field = sim
                            .objects()
                            .iter()
                            .enumerate()
                            .skip(3)
                            .filter(|(_, o)| {
                                o.flags & 0x101 == 1
                                    && (o.flags & stealth_sim::types::oflags::FRIENDLY != 0)
                                        == tr.blue
                            })
                            .map(|(k, o)| (k, (o.x - x).abs().max((o.y - y).abs())))
                            .min_by_key(|p| p.1);
                        let d_km = field.map_or(-1.0, |f| f.1 as f64 * 15.625 / 1000.0);
                        tr.end = Some(format!("landed at {t:.0}s, {d_km:.1} km from a field"));
                        if kind == AircraftKind::Plane
                            && tr.role != Some(Role::Uav)
                            && (d_km > 5.0 || d_km < 0.0)
                        {
                            tr.note(format!(
                                "{t:.0}s 'landed' {d_km:.1} km from any airfield of its side"
                            ));
                        }
                        if tr.rtb_armed {
                            tr.note(format!("{t:.0}s came home without attacking"));
                        }
                    }
                }
                _ => {}
            }
        }
        if frame % FS as u64 != 0 {
            continue;
        }
        for (k, p) in host.state.places.iter().enumerate() {
            if owners.get(k).is_some_and(|&o| o != p.owner) {
                match p.owner {
                    Side::Blue => captures[0] += 1,
                    Side::Red => captures[1] += 1,
                    _ => {}
                }
                owners[k] = p.owner;
            }
        }
        // Once a second: every campaign aircraft.
        for i in 0..sim.flights().len() {
            let Some(f) = sim.campaign_flight(i) else { continue };
            let u = sim.flights()[i];
            let tr = tracks.entry(f.tag).or_insert_with(|| Track {
                tag: f.tag,
                unit: i,
                blue: f.blue,
                role: Some(f.role),
                spec: u.spec,
                first: t,
                min_agl: i32::MAX,
                ..Default::default()
            });
            by_unit.insert(i, f.tag);
            tr.last = t;
            tr.unit = i;
            tr.spec = u.spec;
            if u.flags & fflags::DESTROYED != 0 {
                tr.destroyed = true;
            }
            if u.flags & fflags::AIRBORNE == 0 || tr.destroyed {
                continue;
            }
            let ground = relief.height(u.x, u.y).max(hills.ground_alt(u.x, u.y));
            let agl = u.alt - ground;
            let goal_km = (u.x - f.goal.0).abs().max((u.y - f.goal.1).abs()) as f64 * 15.625 / 1e3;
            let base = sim.objects().get(u.wp.max(0) as usize).copied().unwrap_or_default();
            let (bx, by) = (u.x - base.x, u.y - base.y);
            tr.trail.push_back(format!(
                "{t:.0}s alt {} agl {agl} (hills {} relief {}) spd {} hdg {} pitch {} flags {:#x} mode {} goal {goal_km:.1} km base {:+.1},{:+.1} km threats {:?} strike {:?} def {} tgt {:?} timer {} wp {}",
                u.alt,
                hills.ground_alt(u.x, u.y),
                relief.height(u.x, u.y),
                u.speed,
                u.heading,
                u.pitch,
                u.flags,
                u.mode,
                bx as f64 * 15.625 / 1e3,
                by as f64 * 15.625 / 1e3,
                threats.get(&i).cloned().unwrap_or_default(),
                f.strike,
                f.defending,
                f.target,
                u.timer,
                u.wp
            ));
            if tr.trail.len() > env("SOAK_TRAIL", 30) {
                tr.trail.pop_front();
            }
            let landing = u.flags & (fflags::TAKEOFF | fflags::FINAL | fflags::ROLLOUT) != 0;
            if !landing {
                tr.min_agl = tr.min_agl.min(agl);
            }
            let kind = sim.aircraft_kind(u.spec);
            if u.flags & fflags::RTB != 0 {
                tr.rtb_since.get_or_insert(t);
                if matches!(f.role, Role::Strike | Role::Recon)
                    && f.strike.is_some()
                    && !f.kamikaze
                    && !tr.rtb_armed
                {
                    tr.rtb_armed = true;
                    tr.note(format!("{t:.0}s heading home with its target unattacked"));
                }
            } else {
                tr.rtb_since = None;
            }
            if u.flags & fflags::FINAL != 0 {
                tr.final_since.get_or_insert(t);
            } else {
                tr.final_since = None;
            }
            if u.speed <= 0 && kind == AircraftKind::Plane && !landing {
                tr.still_since.get_or_insert(t);
            } else {
                tr.still_since = None;
            }
            if let Some(s) = tr.rtb_since
                && kind == AircraftKind::Plane
                && t - s > 1800.0
                && t - s < 1801.5
            {
                tr.note(format!("{t:.0}s RTB for 30 min without landing"));
            }
            if let Some(s) = tr.final_since
                && t - s > 240.0
                && t - s < 241.5
            {
                tr.note(format!("{t:.0}s on final for 4 min"));
            }
            if let Some(s) = tr.still_since
                && t - s > 30.0
                && t - s < 31.5
            {
                tr.note(format!("{t:.0}s airborne at speed 0 for 30 s"));
            }
            if t - tr.first > 3.0 * 3600.0 && t - tr.first < 3.0 * 3600.0 + 1.5 {
                tr.note(format!("{t:.0}s three hours in the air"));
            }
            // Circling: a strike or recon aircraft (not fighting) turning round and round
            // without getting nearer its goal.
            let h = u.heading as f64 * std::f64::consts::PI / 32768.0;
            let goal_d = (u.x - f.goal.0).abs().max((u.y - f.goal.1).abs());
            if matches!(f.role, Role::Strike | Role::Recon)
                && !f.defending
                && !landing
                && kind == AircraftKind::Plane
            {
                tr.turn_window.push((t, h, goal_d));
                tr.turn_window.retain(|w| t - w.0 <= 180.0);
                let mut turned = 0.0;
                for w in tr.turn_window.windows(2) {
                    let mut d = w[1].1 - w[0].1;
                    while d > std::f64::consts::PI {
                        d -= std::f64::consts::TAU;
                    }
                    while d < -std::f64::consts::PI {
                        d += std::f64::consts::TAU;
                    }
                    turned += d;
                }
                let closer = tr.turn_window.first().map_or(0, |w| w.2) - goal_d;
                if turned.abs() > 4.0 * std::f64::consts::PI && closer < 0x40 && tr.turned == 0.0 {
                    tr.turned = t;
                    tr.note(format!(
                        "{t:.0}s circled {:.0}° in 3 min (goal {} km off, rtb {})",
                        turned.abs().to_degrees(),
                        goal_d as f64 * 15.625 / 1000.0,
                        u.flags & fflags::RTB != 0
                    ));
                }
            }
        }
        if frame % (FS as u64 * 600) == 0 {
            let up = sim.flights().iter().filter(|u| u.flags & 3 == 3).count();
            log.push(format!(
                "{:5.0} min: {} AI missions ({} open), {up} aircraft up",
                t / 60.0,
                host.state.ai_missions.len(),
                host.state.ai_missions.iter().filter(|m| m.open()).count()
            ));
        }
    }
    for l in &log {
        println!("{l}");
    }
    println!("\n== AI missions ({code}, seed {seed}, {hours} h)");
    for m in missions.values() {
        let fates: Vec<String> = m
            .tags
            .iter()
            .map(|tag| tracks.get(tag).and_then(|t| t.end.clone()).unwrap_or("-".into()))
            .collect();
        println!(
            "#{:3} {:?} from {} {:?} {:?} {:?} {:?} launched {:.0} ended {:?} achieved {} : {}",
            m.id,
            m.side,
            host.state.site(m.from).map_or(String::new(), |s| s.name.clone()),
            m.offer.kind,
            m.airframe,
            m.target,
            m.state,
            m.launched,
            m.ended.map(|e| e.round()),
            m.achieved,
            fates.join("; ")
        );
        let lost = m.alive.iter().all(|a| !a) && m.open();
        if lost {
            println!("     !! open with nobody left");
        }
    }
    println!("\n== Enemy SAM sites by the base that refills them (long / medium / short)");
    let mut per_base: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    for i in 0..sim.ground_unit_count() {
        let Some(m) = sam_class(&sim, i) else { continue };
        let o = sim.objects()[i];
        let Some(b) = host.state.nearest_base(Side::Red, o.x, o.y) else { continue };
        let name = host.state.site(b).map_or(String::new(), |s| s.name.clone());
        per_base.entry(name).or_default()[classes.iter().position(|&c| c == m).unwrap()] += 1;
    }
    for (b, n) in &per_base {
        println!("{b:14} {:2} / {:2} / {:2}", n[0], n[1], n[2]);
    }
    println!("\n== Enemy SAMs (every 10 min: launches since, dry sites, stocks)");
    for l in &sam_log {
        println!("{l}");
    }
    println!("\n== Friendly munitions (hourly: at the bases / at the production base, bases out)");
    for l in &blue_log {
        println!("{l}");
    }
    println!(
        "\n== Friendly munitions over the run (drawn, put back, asked for with none there; lowest \
         total at the bases; share of samples with a base out; most bases out)"
    );
    for (k, &m) in blue_items.iter().enumerate() {
        let [d, r, s] = host.state.munition_log.get(&m).copied().unwrap_or_default();
        println!(
            "{:14} drawn {d:6} back {r:6} short {s:6}  lowest {:6}  out {:3.0}%  max out {}  ({:.1}/h made)",
            format!("{m:?}"),
            blue_low[k],
            100.0 * blue_out_samples[k] as f64 / samples.max(1) as f64,
            blue_out_max[k],
            host.state.production_rate(m)
        );
    }
    println!(
        "\n== Friendly air defence with nothing to fire (unit-samples: Patriots {}, Gepards {})",
        ad_dry[0], ad_dry[1]
    );
    for l in &ad_log {
        println!("{l}");
    }
    println!("\n== Cargo flights");
    for (tag, (r, last)) in &cargo {
        let fate = if host.state.cargo_lost.contains(tag) {
            "shot down"
        } else if host.state.cargo_run(*tag).is_some() {
            "open"
        } else {
            "landed"
        };
        let name = |b: u16| host.state.site(b).map_or(String::new(), |s| s.name.clone());
        println!(
            "tag {tag:5} {:?} {} -> {}: posted {:.0}s, last seen {last:.0}s, {fate}, load {:?}",
            r.side,
            name(r.from),
            name(r.to),
            r.posted,
            r.load
        );
        if let Some(end) = tracks.get(tag).and_then(|t| t.end.clone()) {
            println!("     {end}");
        }
        for e in &r.escorts {
            let fate = tracks.get(e).and_then(|t| t.end.clone()).unwrap_or("-".into());
            println!("     escort {e}: {fate}");
        }
    }
    println!("\n== Supply convoys");
    let log = &host.state.supply_log;
    println!(
        "convoys over {}, trucks arrived {}, trucks lost {}, items delivered {}, lost {}, cut roads {:?}",
        log.convoys, log.trucks_arrived, log.trucks_lost, log.items_delivered, log.items_lost, host.state.cut_roads
    );
    println!("samples with no trucks ready: {fleet_dry} of {samples}");
    println!("trucks struck by the soak (SOAK_INTERDICT {interdict}/h): {struck}");
    let f = &host.state.trucks;
    println!("truck fleet at the end: {:.1} ready, {} out, {} returning", f.ready, f.out, f.returning.iter().map(|r| r.1 as u32).sum::<u32>());
    if !road_on.is_empty() {
        let mean = road_on.iter().sum::<usize>() as f64 / road_on.len() as f64;
        println!(
            "on the road: mean {mean:.1}, most {}; road-served base-samples out of something: {road_out}",
            road_on.iter().max().unwrap()
        );
    }
    for (tag, (start, to)) in &road_runs {
        let done = host.state.cargo_run(*tag).is_none();
        let end = cargo.get(tag).map_or(t, |c| c.1);
        println!(
            "tag {tag:5} -> {to}: on the road {:.0} min{}",
            (end - start) / 60.0,
            if done { "" } else { " (still)" }
        );
    }
    println!("\n== Stocks");
    for (b, s) in &host.state.stocks {
        let name = host.state.site(*b).map_or(String::new(), |s| s.name.clone());
        let side = s.side.unwrap_or(Side::Neutral);
        let prod = if host.state.production_base(side) == Some(*b) { " (production)" } else { "" };
        let items: Vec<String> = stealth_campaign::Munition::of_side(side)
            .iter()
            .map(|&m| format!("{:?} {}", m, s.count(m)))
            .collect();
        println!("{side:?} {name}{prod}: {}", items.join(", "));
    }
    println!("\n== Ground units standing (hourly)");
    for l in &ground_log {
        println!("{l}");
    }
    println!("\n== Ground units lost (side, class, cause)");
    for ((side, class, cause), n) in &ground_losses {
        println!("{side:4} {class:8} {n:4}  {cause}");
    }
    println!("\n== Missiles after their own side's aircraft");
    for l in &friendly_fire {
        println!("{l}");
    }
    println!("\n== Supply helicopters shot down");
    for l in &supply_log {
        println!("{l}");
    }
    println!("\n== New sites at the end");
    for d in &host.state.deployments {
        println!(
            "#{} {:?} {} by {:?} built {} delivered {:?} ai {:?} open since {:.0} crates {:?}",
            d.id, d.kind, d.name, d.by, d.built, d.delivered, d.ai, d.open_since, d.crates
        );
    }
    println!("\n== News");
    for n in &news {
        println!("{n}");
    }
    println!("\n== Campaign aircraft with notes");
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for tr in tracks.values() {
        let side = if tr.blue { Side::Blue } else { Side::Red };
        let key = format!("{side:?} {:?} {}", tr.role, sim.aircraft_name(tr.spec));
        *counts.entry(key.clone()).or_default() += 1;
        // A flight recalled because its mission was achieved (by another of its aircraft, or
        // the ground war) comes home armed as it should.
        let achieved = missions.values().any(|m| m.achieved && m.tags.contains(&tr.tag));
        let notes: Vec<&String> = tr
            .notes
            .iter()
            .filter(|n| {
                !(achieved && (n.contains("unattacked") || n.contains("without attacking")))
            })
            .collect();
        if notes.iter().all(|n| n.starts_with("    |")) {
            continue;
        }
        println!(
            "tag {:5} unit {:2} {key}: up {:.0}-{:.0}s, lowest {} ft, end {:?}",
            tr.tag, tr.unit, tr.first, tr.last, tr.min_agl, tr.end
        );
        for n in notes {
            println!("     {n}");
        }
    }
    println!("\n== Flights seen (shot down, landed, still up)");
    let mut fates: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    for tr in tracks.values() {
        let key = format!(
            "{} {:?} {}",
            if tr.blue { "Blue" } else { "Red" },
            tr.role,
            sim.aircraft_name(tr.spec)
        );
        let k = if tr.destroyed {
            0
        } else if tr.end.is_some() {
            1
        } else {
            2
        };
        fates.entry(key).or_default()[k] += 1;
    }
    for (k, n) in counts {
        let f = fates.get(&k).copied().unwrap_or_default();
        println!("{n:4} {k}  ({} / {} / {})", f[0], f[1], f[2]);
    }
    println!("{t:.0} s simulated");
    // One line to compare runs by (`grep BALANCE`).
    let st = &host.state;
    let held = |owners: &[Side], side: Side, airfield: bool| {
        st.places
            .iter()
            .zip(owners)
            .filter(|(p, o)| **o == side && (!airfield || p.airfield.is_some()))
            .count()
    };
    let now: Vec<Side> = st.places.iter().map(|p| p.owner).collect();
    let flights = |blue: bool, role: Option<Role>| {
        let all = tracks.values().filter(|t| t.blue == blue && (role.is_none() || t.role == role));
        let lost = all.clone().filter(|t| t.destroyed).count();
        (all.count(), lost)
    };
    let ground = |side: &str| {
        ground_losses.iter().filter(|((s, ..), _)| *s == side).map(|(_, n)| n).sum::<usize>()
    };
    let cargo_lost = |side: Side| {
        let all = cargo.values().filter(|(r, _)| r.side == side && r.road.is_none());
        let lost = all.clone().filter(|(r, _)| st.cargo_lost.contains(&r.tag)).count();
        (all.count(), lost)
    };
    let ai = |side: Side| missions.values().filter(|m| m.side == side).count();
    let (bs, rs) = (flights(true, Some(Role::Supply)), flights(false, Some(Role::Supply)));
    let (ba, ra) = (flights(true, None), flights(false, None));
    let (bc, rc) = (cargo_lost(Side::Blue), cargo_lost(Side::Red));
    println!(
        "BALANCE {code} seed {seed} crew {} | places blue {}->{} red {}->{} | fields blue {}->{} red {}->{} | taken by blue {} red {} | materiel blue {:.0} red {:.0} | fighters blue {} red {} | ground lost blue {} red {} | air lost blue {}/{} red {}/{} | ai missions blue {} red {} | supply helos blue {}/{} red {}/{} | cargo planes blue {}/{} red {}/{}",
        env("SOAK_CREW", 1usize),
        held(&owners0, Side::Blue, false),
        held(&now, Side::Blue, false),
        held(&owners0, Side::Red, false),
        held(&now, Side::Red, false),
        held(&owners0, Side::Blue, true),
        held(&now, Side::Blue, true),
        held(&owners0, Side::Red, true),
        held(&now, Side::Red, true),
        captures[0],
        captures[1],
        st.forces(Side::Blue).materiel,
        st.forces(Side::Red).materiel,
        st.forces(Side::Blue).fighters,
        st.forces(Side::Red).fighters,
        ground("blue"),
        ground("red"),
        ba.1,
        ba.0,
        ra.1,
        ra.0,
        ai(Side::Blue),
        ai(Side::Red),
        bs.1,
        bs.0,
        rs.1,
        rs.0,
        bc.1,
        bc.0,
        rc.1,
        rc.0,
    );
}
