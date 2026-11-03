//! A campaign left to the AI (no players), run as fast as it goes: every campaign aircraft is
//! watched once a second and what looks wrong is reported (crashes nobody caused, flights that
//! never land, landings away from an airfield, strikes that turn home unfired, circling).
//!
//! `cargo test --release -p stealth-net --test campaign_soak -- --ignored --nocapture`, with
//! `SOAK_THEATER` (central_europe), `SOAK_HOURS` (campaign-clock hours of sim time, 3),
//! `SOAK_SEED` (1).

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
    let sim_data = theater::sim_data(setup.theater, &code).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    let relief = sim.relief();
    let hills = sim.hills();
    let dt = 1.0 / FS as f64;
    let frames = (hours * 3600.0 * FS as f64) as u64;
    let mut tracks: BTreeMap<u32, Track> = BTreeMap::new();
    let mut by_unit: BTreeMap<usize, u32> = BTreeMap::new();
    let mut t = 0.0f64;
    let mut log: Vec<String> = Vec::new();
    let mut threats: BTreeMap<usize, Vec<i32>> = BTreeMap::new();
    let mut news: Vec<String> = Vec::new();
    let mut missions: BTreeMap<u32, stealth_campaign::AiMission> = BTreeMap::new();
    let mut shelled: BTreeMap<usize, usize> = BTreeMap::new();
    for frame in 0..frames {
        t = frame as f64 * dt;
        // What is after each aircraft (for the report of what shot it down).
        threats.clear();
        for m in sim.ai_missiles().iter().filter(|m| m.ground.is_none()) {
            threats.entry(m.target).or_default().push(m.spec);
        }
        shelled.clear();
        for (target, _) in sim.ai_shell_targets() {
            *shelled.entry(target).or_default() += 1;
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
        host.refills(&mut sim, dt as f32);
        host.take_sim_events(&mut sim);
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
            "#{:3} {:?} {:?} {:?} {:?} {:?} launched {:.0} ended {:?} achieved {} : {}",
            m.id,
            m.side,
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
    println!("\n== Flights seen");
    for (k, n) in counts {
        println!("{n:4} {k}");
    }
    println!("{t:.0} s simulated");
}
