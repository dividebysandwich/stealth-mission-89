//! Campaign mode (`SimOptions::campaign`) on the shipped game data.

use std::sync::OnceLock;

use stealth_mission::{GameData, Mission, MissionKind, MissionSettings, Seed};
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

/// A Central Europe campaign simulation on the theater's own world. The seed sets the campaign's
/// dice too: with some, a strike's single weapon misses (`GROUND_PK`) and the strike tests fail.
fn campaign() -> (Mission, Sim) {
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
    let sim = Sim::with_options(&d.sim, &m, SimOptions { campaign: true, ..Default::default() });
    (m, sim)
}

fn idle() -> PlayerState {
    PlayerState { on_ground: true, fuel: 10000, ..Default::default() }
}

fn find(sim: &Sim, flags: i32, name_like: Option<&str>) -> usize {
    (3..sim.objects().len())
        .find(|&i| {
            sim.objects()[i].flags & 0xF0F == flags
                && name_like.is_none_or(|n| sim.place_name(i).eq_ignore_ascii_case(n))
        })
        .expect("object")
}

/// No radars, no guns.
fn quiet(sim: &mut Sim) {
    for o in sim.objects_mut().iter_mut() {
        o.unit_type = 0;
    }
    sim.units_mut().clear();
}

fn run(sim: &mut Sim, ps: &PlayerState, secs: i32) -> Vec<SimEvent> {
    let mut ev = Vec::new();
    for _ in 0..secs * 15 {
        ev.extend(sim.step(ps, 15));
    }
    ev
}

#[test]
fn the_absent_player_is_left_alone() {
    let (_, mut sim) = campaign();
    assert!(sim.player_absent());
    // Hovering over an enemy SAM site's position would get him shot at, were he there.
    let o = sim.objects()[3];
    let (x, y) = Sim::map_to_world(o.x, o.y);
    let ps = PlayerState {
        x,
        y,
        altitude: 3000,
        height: 3000,
        velocity: 400 * 27,
        gear_up: true,
        fuel: 10000,
        ..Default::default()
    };
    let ev = run(&mut sim, &ps, 120);
    assert!(
        !ev.iter().any(|e| matches!(
            e,
            SimEvent::MissileLaunched { .. } | SimEvent::Rwr(_) | SimEvent::PlayerHit { .. }
        )),
        "{ev:?}"
    );
    assert!(sim.mission_end().is_none());
    assert!(sim.emitters().is_empty() || sim.player_absent());
}

#[test]
fn a_crash_ends_the_sortie_not_the_simulation() {
    let (_, mut sim) = campaign();
    let base = find(&sim, 0x401, None);
    let st = sim.respawn_player(base, None);
    assert!(!sim.player_absent());
    run(&mut sim, &st, 2);
    // Fast on the ground, off the runway: a crash.
    let o = sim.objects()[base];
    let (x, y) = Sim::map_to_world(o.x + 0x200, o.y);
    let ps = PlayerState { x, y, velocity: 200 * 27, fuel: 10000, ..Default::default() };
    let airborne = PlayerState { altitude: 500, height: 500, on_ground: false, ..ps.clone() };
    run(&mut sim, &airborne, 2);
    let ps = PlayerState { on_ground: true, ..ps };
    let ev = run(&mut sim, &ps, 2);
    let end = ev.iter().find_map(|e| match e {
        SimEvent::SortieEnded(e) => Some(*e),
        _ => None,
    });
    assert_eq!(end.map(|e| e.outcome), Some(1), "{ev:?}");
    assert!(sim.player_absent());
    assert!(sim.mission_end().is_none());
    // The world goes on, and the next sortie starts clean.
    run(&mut sim, &idle(), 5);
    let st = sim.respawn_player(base, None);
    assert_eq!(sim.damage_effects().bits, 0);
    assert!(st.on_ground);
}

#[test]
fn a_safe_landing_keeps_the_sortie_going() {
    let (_, mut sim) = campaign();
    let base = find(&sim, 0x401, None);
    let st = sim.respawn_player(base, None);
    let air = PlayerState {
        x: st.x + 0x8000,
        altitude: 800,
        height: 800,
        on_ground: false,
        velocity: 250 * 27,
        gear_up: true,
        ..st.clone()
    };
    run(&mut sim, &air, 3);
    let ev = run(&mut sim, &st, 5);
    assert!(
        ev.iter().any(|e| matches!(e, SimEvent::LandedAtBase { object } if *object == base)),
        "{ev:?}"
    );
    assert!(!ev.iter().any(|e| matches!(e, SimEvent::SortieEnded(_))));
    assert!(!sim.player_absent());
}

#[test]
fn an_insertion_helicopter_lands_at_its_goal() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    let i = sim.spawn_insertion(from, to, 42, 0);
    assert_eq!(sim.campaign_flight(i).map(|f| f.tag), Some(42));
    let mut landed = None;
    for _ in 0..40 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            if let SimEvent::CampaignFlightLanded { tag, x, y, .. } = e {
                landed = Some((tag, x, y));
            }
        }
        if landed.is_some() {
            break;
        }
    }
    let (tag, x, y) = landed.expect("landed within 40 minutes");
    let t = sim.objects()[to];
    assert_eq!(tag, 42);
    assert!((x - t.x).abs() < 0x40 && (y - t.y).abs() < 0x40, "{x},{y} vs {},{}", t.x, t.y);
    // Its slot is free for the next one.
    assert!(sim.campaign_flight(i).is_none());
}

#[test]
fn enemy_defences_shoot_down_helicopters() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    // A SAM site on the way.
    let (a, b) = (sim.objects()[from], sim.objects()[to]);
    let site = &mut sim.objects_mut()[3];
    site.unit_type = 7;
    site.x = (a.x + b.x) / 2;
    site.y = (a.y + b.y) / 2;
    site.flags = 0;
    for n in 0..4 {
        sim.spawn_insertion(from, to, 100 + n, n as usize);
    }
    let mut lost = Vec::new();
    for _ in 0..30 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            if let SimEvent::CampaignFlightDestroyed { tag, by_player, .. } = e {
                assert!(!by_player);
                lost.push(tag);
            }
        }
        if !lost.is_empty() {
            break;
        }
    }
    assert!(!lost.is_empty(), "the SA-13 let them all through");
}

#[test]
fn fighters_go_for_the_helicopters() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    let enemy = find(&sim, 0x001, Some("Komarno"));
    sim.spawn_insertion(from, to, 7, 0);
    // Scrambled (as the campaign does) when the helicopter is 80 km out.
    run(&mut sim, &idle(), 10 * 60);
    let t = sim.objects()[to];
    sim.spawn_fighter(enemy, t.x, t.y, 8, false, stealth_sim::ai_loadout::FighterTask::Patrol);
    let mut lost = false;
    for _ in 0..15 * 60 {
        if run(&mut sim, &idle(), 1)
            .iter()
            .any(|e| matches!(e, SimEvent::CampaignFlightDestroyed { tag: 7, .. }))
        {
            lost = true;
            break;
        }
    }
    assert!(lost, "the fighter never got the helicopter");
}

#[test]
fn a_far_sam_site_leaves_helicopters_alone() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    // An SA-11 20 km off the route: well in its reach, but too far to see a helicopter.
    let (a, b) = (sim.objects()[from], sim.objects()[to]);
    let site = &mut sim.objects_mut()[3];
    site.unit_type = 5;
    site.x = (a.x + b.x) / 2;
    site.y = (a.y + b.y) / 2 + 20 * 64;
    site.flags = 0;
    for n in 0..4 {
        sim.spawn_insertion(from, to, 100 + n, n as usize);
    }
    let mut landed = 0;
    for _ in 0..45 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::CampaignFlightDestroyed { tag, .. } => panic!("the SA-11 got {tag}"),
                SimEvent::CampaignFlightLanded { .. } => landed += 1,
                _ => {}
            }
        }
        if landed == 4 {
            break;
        }
    }
    assert_eq!(landed, 4);
}

#[test]
fn fighters_use_their_guns() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    let enemy = find(&sim, 0x001, Some("Komarno"));
    sim.spawn_insertion(from, to, 7, 0);
    run(&mut sim, &idle(), 10 * 60);
    let t = sim.objects()[to];
    let f =
        sim.spawn_fighter(enemy, t.x, t.y, 8, false, stealth_sim::ai_loadout::FighterTask::Patrol);
    // No missiles: only its gun.
    sim.set_fighter_loadout(f, Default::default());
    let mut lost = false;
    for _ in 0..15 * 60 {
        if run(&mut sim, &idle(), 1)
            .iter()
            .any(|e| matches!(e, SimEvent::CampaignFlightDestroyed { tag: 7, .. }))
        {
            lost = true;
            break;
        }
    }
    assert!(lost, "the fighter never gunned down the helicopter");
}

#[test]
fn places_change_hands_and_get_repaired() {
    let (_, mut sim) = campaign();
    let tabor = find(&sim, 0x001, Some("Tabor"));
    let shape = sim.object_shapes()[tabor];
    sim.remove_object(tabor);
    assert!(sim.objects()[tabor].flags & oflags::DESTROYED != 0);
    assert_eq!(sim.world_damage().tiles.len(), 1);
    sim.restore_object(tabor);
    assert!(sim.world_damage().tiles.is_empty());
    sim.set_side(tabor, true, false);
    let o = sim.objects()[tabor];
    assert!(o.flags & oflags::DESTROYED == 0 && o.flags & oflags::FRIENDLY != 0);
    assert_eq!(sim.object_shapes()[tabor], shape);
    // Friendly guns there now, the enemy's gone.
    // (In the enemy's wrecks' slots.)
    sim.place_defences(o.x, o.y, true, false);
    sim.place_defences(o.x, o.y, true, true);
    let near = |u: &GroundUnit| (u.x - o.x).abs() + (u.y - o.y).abs() <= 0x200;
    let units = sim.units();
    assert!((0..units.len()).all(|k| !near(&units[k]) || units[k].destroyed || sim.aaa_blue(k)));
    assert!((0..units.len()).any(|k| {
        near(&units[k]) && !units[k].destroyed && units[k].kind == GroundKind::Gepard
    }));
    // A sortie can start there.
    let st = sim.respawn_player(tabor, None);
    assert!(st.on_ground);
}

#[test]
fn intelligence_shortens_detection() {
    let (_, mut sim) = campaign();
    let i = 3;
    let o = sim.objects()[i];
    // An SA-13's detection range.
    let leth = data().sim.threats[7].lethality as i32;
    sim.objects_mut()[i].unit_type = 7;
    let base = find(&sim, 0x401, None);
    sim.respawn_player(base, None);
    // 90% of the detection range out: seen without intel, not with it.
    let d = leth * 64 * 9 / 10;
    let (x, y) = Sim::map_to_world(o.x, o.y + d);
    let ps = PlayerState {
        x,
        y,
        altitude: 5000,
        height: 5000,
        velocity: 400 * 27,
        gear_up: true,
        fuel: 10000,
        ..Default::default()
    };
    run(&mut sim, &ps, 1);
    let seen = |sim: &Sim| sim.emitters().iter().any(|e| e.source == EmitterSource::Site(i));
    assert!(seen(&sim));
    sim.set_intel(i, true);
    run(&mut sim, &ps, 1);
    assert!(!seen(&sim));
}

#[test]
fn a_long_campaign_stays_bounded() {
    let (_, mut sim) = campaign();
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    let enemy = find(&sim, 0x001, Some("Komarno"));
    let t = sim.objects()[to];
    let mut tag = 1;
    for minute in 0..30 {
        if minute % 3 == 0 {
            sim.spawn_insertion(from, to, tag, 0);
            sim.spawn_fighter(
                enemy,
                t.x,
                t.y,
                tag + 1,
                false,
                stealth_sim::ai_loadout::FighterTask::Patrol,
            );
            tag += 2;
        }
        run(&mut sim, &idle(), 60);
    }
    assert!(sim.flights().len() < 40, "{} flight units", sim.flights().len());
    assert!(sim.ai_missiles().len() < 50);
    assert!(sim.event_log().len() < 500);
}

use stealth_sim::aaa::{GroundKind, GroundUnit, NASAMS, PATRIOT, STINGER};
use stealth_sim::ai_loadout::{AIM9, AIM120, FighterTask};
use stealth_sim::campaign::{CAP_BUBBLE, CAP_LEASH};

/// Runs until `tag` is shot down (true) or `secs` are up; every AI missile seen goes to `seen`.
fn until_down(sim: &mut Sim, tag: u32, secs: i32, seen: &mut Vec<i32>) -> bool {
    for _ in 0..secs {
        let ev = run(sim, &idle(), 1);
        seen.extend(sim.ai_missiles().iter().map(|m| m.spec));
        if ev
            .iter()
            .any(|e| matches!(e, SimEvent::CampaignFlightDestroyed { tag: t, .. } if *t == tag))
        {
            return true;
        }
    }
    false
}

/// Runs until flight `f` is this close to (x, y).
fn until_near(sim: &mut Sim, f: usize, (x, y): (i32, i32), d: i32) {
    for _ in 0..40 * 60 {
        let u = sim.flights()[f];
        if (u.x - x).abs().max((u.y - y).abs()) < d {
            return;
        }
        run(sim, &idle(), 1);
    }
    panic!("never got near");
}

/// An enemy patrol out of Komarno over Neuburg (tag 8), and Neuberg.
fn raid(sim: &mut Sim) -> (usize, usize) {
    let home = find(sim, 0x401, Some("Neuburg"));
    let enemy = find(sim, 0x001, Some("Komarno"));
    let t = sim.objects()[home];
    let f = sim.spawn_fighter(enemy, t.x, t.y, 8, false, FighterTask::Patrol);
    // Unarmed: what is tested is what shoots it down.
    sim.set_fighter_loadout(f, Default::default());
    (home, f)
}

#[test]
fn a_patriot_battery_shoots_down_raiders() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, _) = raid(&mut sim);
    let o = sim.objects()[home];
    sim.place_patriot(o.x + 0x100, o.y);
    let kinds: Vec<GroundKind> = sim.units().iter().map(|u| u.kind).collect();
    assert_eq!(kinds.iter().filter(|&&k| k == GroundKind::PatriotLauncher).count(), 3);
    assert_eq!(kinds.iter().filter(|&&k| k == GroundKind::PatriotRadar).count(), 1);
    assert!((0..kinds.len()).all(|k| sim.aaa_blue(k)));
    let mut seen = Vec::new();
    assert!(until_down(&mut sim, 8, 40 * 60, &mut seen), "the Patriots never got it");
    assert!(seen.contains(&PATRIOT));
}

#[test]
fn a_nasams_battery_shoots_down_raiders() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, _) = raid(&mut sim);
    let o = sim.objects()[home];
    sim.place_nasams(o.x + 0x100, o.y);
    let kinds: Vec<GroundKind> = sim.units().iter().map(|u| u.kind).collect();
    assert_eq!(kinds.iter().filter(|&&k| k == GroundKind::NasamsLauncher).count(), 3);
    assert_eq!(kinds.iter().filter(|&&k| k == GroundKind::NasamsRadar).count(), 1);
    assert!((0..kinds.len()).all(|k| sim.aaa_blue(k)));
    let mut seen = Vec::new();
    assert!(until_down(&mut sim, 8, 40 * 60, &mut seen), "the NASAMS never got it");
    assert!(seen.contains(&NASAMS));
    assert!(!seen.contains(&PATRIOT));
}

#[test]
fn a_patriot_kill_is_credited_to_its_radar() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, _) = raid(&mut sim);
    let o = sim.objects()[home];
    sim.place_patriot(o.x + 0x100, o.y);
    let radar = sim.units().iter().position(|u| u.kind == GroundKind::PatriotRadar);
    for _ in 0..40 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            if let SimEvent::CampaignFlightDestroyed { tag: 8, radar: by, .. } = e {
                assert_eq!(by, radar);
                return;
            }
        }
    }
    panic!("the Patriots never got it");
}

#[test]
fn a_gepard_kill_is_credited_to_the_gun() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    // A strike aircraft, flying in low (a fighter on patrol flies above the gun).
    let home = find(&sim, 0x401, Some("Neuburg"));
    let enemy = find(&sim, 0x001, Some("Komarno"));
    sim.spawn_striker(enemy, home, 8, false);
    // On its way in (it flies at about 300 m).
    sim.place_gepard(13900, 28580);
    let gepard = sim.units().iter().position(|u| u.kind == GroundKind::Gepard);
    for _ in 0..40 * 60 {
        let ev = run(&mut sim, &idle(), 1);
        for e in ev {
            if let SimEvent::CampaignFlightDestroyed { tag: 8, radar: by, .. } = e {
                assert_eq!(by, gepard);
                return;
            }
        }
    }
    panic!("the Gepard never got it");
}

#[test]
fn an_enemy_sead_strike_takes_out_a_patriot_radar() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x001, Some("Tabor"));
    let o = sim.objects()[find(&sim, 0x401, Some("Neuburg"))];
    sim.place_patriot(o.x + 0x100, o.y);
    let radar = sim.units().iter().position(|u| u.kind == GroundKind::PatriotRadar).unwrap();
    let i = sim.spawn_sead(from, radar, 57, false);
    let (mut hit, mut lost, mut arm) = (false, false, false);
    for _ in 0..60 * 60 {
        arm |= sim.ai_missiles().iter().any(|m| m.ground == Some(radar) && m.ground_aaa);
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::AaaDestroyed { unit, .. } if unit == radar => hit = true,
                SimEvent::CampaignFlightDestroyed { tag: 57, .. } => lost = true,
                _ => {}
            }
        }
        if hit {
            break;
        }
    }
    assert!(arm, "no anti-radiation missile at the radar");
    assert!(hit, "the Patriot radar was not hit");
    assert!(sim.units()[radar].destroyed);
    assert!(!lost, "the Patriots shot the SEAD aircraft down before it fired");
    assert!(sim.campaign_flight(i).is_some_and(|f| f.strike.is_none()), "it fired");
}

#[test]
fn patriot_launchers_need_their_radar() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, _) = raid(&mut sim);
    let o = sim.objects()[home];
    sim.place_patriot(o.x + 0x100, o.y);
    for u in sim.units_mut().iter_mut() {
        if u.kind == GroundKind::PatriotRadar {
            u.destroyed = true;
        }
    }
    let mut seen = Vec::new();
    assert!(!until_down(&mut sim, 8, 30 * 60, &mut seen));
    assert!(seen.is_empty(), "{seen:?}");
}

#[test]
fn friendly_guns_and_stingers_defend_an_airfield() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, f) = raid(&mut sim);
    let o = sim.objects()[home];
    sim.place_defences(o.x, o.y, true, true);
    let kinds: Vec<GroundKind> = sim.units().iter().map(|u| u.kind).collect();
    assert!(kinds.contains(&GroundKind::Gepard) && kinds.contains(&GroundKind::Manpads));
    assert!(!kinds.contains(&GroundKind::Zsu));
    // Low over the field, in the guns' reach (its patrol would circle it out of their reach).
    run(&mut sim, &idle(), 60);
    let mut seen = Vec::new();
    let mut down = false;
    for _ in 0..10 * 60 {
        let u = &mut sim.flights_mut()[f];
        if u.flags & fflags::DESTROYED == 0 {
            (u.x, u.y, u.alt) = (o.x + 0x30, o.y, 1500);
            (u.wx, u.wy) = (u.x << 5, u.y << 5);
        }
        let ev = run(&mut sim, &idle(), 1);
        seen.extend(sim.ai_missiles().iter().map(|m| m.spec));
        if ev.iter().any(|e| matches!(e, SimEvent::CampaignFlightDestroyed { tag: 8, .. })) {
            down = true;
            break;
        }
    }
    assert!(down, "the airfield's guns never got it");
    assert!(seen.iter().all(|&s| s == STINGER), "{seen:?}");
}

#[test]
fn a_cap_defends_its_airfield_with_missiles() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, raider) = raid(&mut sim);
    let o = sim.objects()[home];
    // Scrambled as the campaign does, with the raider 30 km out.
    until_near(&mut sim, raider, (o.x, o.y), 0x780);
    let cap = sim.spawn_fighter(home, o.x, o.y, 9, true, FighterTask::Cap);
    let f = sim.campaign_flight(cap).unwrap();
    assert_eq!(f.task, FighterTask::Cap);
    assert_eq!(f.anchor, (o.x, o.y));
    assert!(f.loadout.stores.iter().all(|s| s.count == 0 || [AIM120, AIM9].contains(&s.missile)));
    let mut seen = Vec::new();
    assert!(until_down(&mut sim, 8, 15 * 60, &mut seen), "the CAP never got the raider");
    assert!(seen.iter().any(|s| [AIM120, AIM9].contains(s)), "{seen:?}");
}

#[test]
fn a_cap_stays_home_and_lands() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let home = find(&sim, 0x401, Some("Neuburg"));
    let enemy = find(&sim, 0x001, Some("Komarno"));
    let (o, e) = (sim.objects()[home], sim.objects()[enemy]);
    assert!((e.x - o.x).abs().max((e.y - o.y).abs()) > CAP_LEASH, "Komarno too close");
    // An enemy patrol over its own base, out of the bubble.
    sim.spawn_fighter(enemy, e.x, e.y, 8, false, FighterTask::Patrol);
    let cap = sim.spawn_fighter(home, o.x, o.y, 9, true, FighterTask::Cap);
    let mut landed = false;
    for _ in 0..30 * 60 {
        for ev in run(&mut sim, &idle(), 1) {
            match ev {
                SimEvent::CampaignFlightDestroyed { tag, .. } => panic!("{tag} shot down"),
                SimEvent::CampaignFlightLanded { tag: 9, .. } => landed = true,
                _ => {}
            }
        }
        if landed {
            break;
        }
        if let Some(f) = sim.campaign_flight(cap) {
            assert!(f.target.is_none(), "went for an aircraft out of its bubble");
            let u = sim.flights()[cap];
            assert!(
                (u.x - o.x).abs().max((u.y - o.y).abs()) < CAP_BUBBLE,
                "strayed {},{} from home",
                u.x - o.x,
                u.y - o.y
            );
        }
    }
    assert!(landed, "the CAP never landed");
}

#[test]
fn a_cap_does_not_follow_into_enemy_territory() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (home, raider) = raid(&mut sim);
    let o = sim.objects()[home];
    until_near(&mut sim, raider, (o.x, o.y), 0x780);
    let cap = sim.spawn_fighter(home, o.x, o.y, 9, true, FighterTask::Cap);
    sim.set_fighter_loadout(cap, Default::default());
    // Wait until the CAP has the raider.
    let mut engaged = false;
    for _ in 0..30 * 60 {
        run(&mut sim, &idle(), 1);
        if sim.campaign_flight(cap).and_then(|f| f.target) == Some(raider) {
            engaged = true;
            break;
        }
    }
    assert!(engaged, "the CAP never went for the raider");
    // Dragged out past the leash: it lets go.
    let u = &mut sim.flights_mut()[cap];
    u.x = o.x + CAP_LEASH + 0x100;
    u.wx = u.x << 5;
    run(&mut sim, &idle(), 2);
    assert_eq!(sim.campaign_flight(cap).and_then(|f| f.target), None);
}

#[test]
fn a_friendly_fighter_leaves_the_player_alone() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let home = find(&sim, 0x401, Some("Neuburg"));
    let o = sim.objects()[home];
    let cap = sim.spawn_fighter(home, o.x, o.y, 9, true, FighterTask::Cap);
    let st = sim.respawn_player(home, None);
    let (x, y) = Sim::map_to_world(o.x + 0x100, o.y);
    let ps = PlayerState {
        x,
        y,
        altitude: 3000,
        height: 3000,
        velocity: 400 * 27,
        gear_up: true,
        on_ground: false,
        fuel: 10000,
        ..st
    };
    for _ in 0..120 {
        sim.raise_alarm();
        run(&mut sim, &ps, 1);
        assert_ne!(sim.flights()[cap].mode, 1, "the CAP went for the player");
    }
}

#[test]
fn an_enemy_strike_bombs_the_airfield_and_flies_home() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x001, Some("Tabor"));
    let to = find(&sim, 0x401, Some("Neuburg"));
    let i = sim.spawn_striker(from, to, 55, false);
    assert!(!sim.campaign_flight(i).unwrap().blue);
    let (mut bombed, mut landed) = (false, false);
    for _ in 0..60 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::GroundDestroyed { object, by: None, .. } if object == to => bombed = true,
                SimEvent::CampaignFlightLanded { tag: 55, .. } => landed = true,
                _ => {}
            }
        }
        if landed {
            break;
        }
    }
    assert!(bombed, "the airfield was not bombed");
    assert!(sim.objects()[to].flags & oflags::DESTROYED != 0);
    assert!(landed, "the strike aircraft did not get home");
}

#[test]
fn a_sead_strike_gets_its_missile_off_at_a_live_sam_site() {
    let (_, mut sim) = campaign();
    sim.units_mut().clear();
    let from = find(&sim, 0x401, None);
    let b = sim.objects()[from];
    // The enemy SAM site nearest the base keeps its radar; every other one is off.
    let sam = (3..sim.objects().len())
        .filter(|&i| {
            let o = sim.objects()[i];
            o.flags & 0xF0F == 0 && o.unit_type != 0 && sim.site_lock_range(o.unit_type) > 0
        })
        .min_by_key(|&i| {
            let o = sim.objects()[i];
            (o.x - b.x).abs().max((o.y - b.y).abs())
        })
        .expect("a SAM site");
    for (i, o) in sim.objects_mut().iter_mut().enumerate() {
        if i != sam {
            o.unit_type = 0;
        }
    }
    let i = sim.spawn_striker(from, sam, 56, true);
    let (mut hit, mut lost) = (false, false);
    for _ in 0..60 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::GroundDestroyed { object, by: None, .. } if object == sam => hit = true,
                SimEvent::CampaignFlightDestroyed { tag: 56, .. } => lost = true,
                _ => {}
            }
        }
        if hit {
            break;
        }
    }
    assert!(hit, "the SAM site was not hit");
    assert!(!lost, "the SAM site shot the strike aircraft down");
    assert!(sim.campaign_flight(i).is_some_and(|f| f.strike.is_none()), "it fired");
}

#[test]
fn an_enemy_helicopter_lands_at_a_friendly_place() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x001, Some("Tabor"));
    let to = find(&sim, 0x401, Some("Neuburg"));
    let i = sim.spawn_insertion_for(from, to, 9, 0, false);
    let f = sim.campaign_flight(i).unwrap();
    assert!(!f.blue && !sim.friendly_unit(i, sim.flights()[i].spec));
    let mut landed = false;
    for _ in 0..40 * 60 {
        landed |= run(&mut sim, &idle(), 1)
            .iter()
            .any(|e| matches!(e, SimEvent::CampaignFlightLanded { tag: 9, .. }));
        if landed {
            break;
        }
    }
    assert!(landed, "the Hind did not land within 40 minutes");
}

#[test]
fn lost_guns_are_wrecks_until_replaced() {
    let (_, mut sim) = campaign();
    sim.units_mut().clear();
    let o = sim.objects()[find(&sim, 0x001, Some("Tabor"))];
    sim.place_defences(o.x, o.y, true, false);
    let alive = |sim: &Sim| sim.units().iter().filter(|u| !u.destroyed).count();
    let n = alive(&sim);
    assert!(n >= 2);
    sim.wreck_defences(o.x, o.y, false, 2);
    assert_eq!(alive(&sim), n - 2);
    sim.restore_defence(o.x, o.y, false);
    assert_eq!(alive(&sim), n - 1);
}

#[test]
fn an_attack_drone_dives_into_its_target() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x001, Some("Tabor"));
    let to = find(&sim, 0x401, Some("Neuburg"));
    sim.spawn_drone(from, to, 77, false);
    let t = sim.objects()[to];
    let (mut hit, mut gone) = (false, false);
    for _ in 0..4 * 60 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::GroundDestroyed { object, by: None, .. } if object == to => hit = true,
                // It blows up on the target, on the ground (not short of it in the air).
                SimEvent::Explosion { x, y, alt, kind: -6 } => {
                    assert_eq!(alt, 0);
                    assert!((x - t.x).abs() < 0x20 && (y - t.y).abs() < 0x20, "{x},{y}");
                }
                SimEvent::CampaignFlightDestroyed { tag: 77, by_player, by, .. } => {
                    assert!(!by_player && by.is_none(), "nobody's kill");
                    gone = true;
                }
                SimEvent::CampaignFlightLanded { tag: 77, .. } => panic!("a drone does not land"),
                _ => {}
            }
        }
        if hit && gone {
            return;
        }
    }
    panic!("hit {hit}, gone {gone}");
}

#[test]
fn an_awacs_orbits_until_called_home() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x001, Some("Tabor"));
    let o = sim.objects()[from];
    let (gx, gy) = (o.x + 0x600, o.y);
    let i = sim.spawn_awacs(from, gx, gy, 88, false);
    run(&mut sim, &idle(), 20 * 60);
    let u = sim.flights()[i];
    assert!(sim.campaign_flight(i).is_some_and(|f| f.tag == 88), "still up");
    assert!((u.x - gx).abs() < 0x300 && (u.y - gy).abs() < 0x300, "{},{}", u.x, u.y);
    sim.recall_flight(88);
    let mut landed = false;
    for _ in 0..60 * 60 {
        landed |= run(&mut sim, &idle(), 1)
            .iter()
            .any(|e| matches!(e, SimEvent::CampaignFlightLanded { tag: 88, .. }));
        if landed {
            break;
        }
    }
    assert!(landed, "the AWACS did not land");
}

#[test]
fn a_drone_that_crashes_short_destroys_nothing() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x001, Some("Tabor"));
    let to = find(&sim, 0x401, Some("Neuburg"));
    let i = sim.spawn_drone(from, to, 78, false);
    // On the ground far from its target: it crashes there.
    sim.flights_mut()[i].alt = 0;
    let mut gone = false;
    for _ in 0..60 {
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::GroundDestroyed { object, .. } if object == to => panic!("destroyed"),
                SimEvent::CampaignFlightDestroyed { tag: 78, .. } => gone = true,
                _ => {}
            }
        }
    }
    assert!(gone);
    assert_eq!(sim.objects()[to].flags & oflags::DESTROYED, 0);
}

#[test]
fn a_forward_battery_is_enemy_guns_around_its_point() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let (x, y) = (0x4000, 0x4000);
    sim.place_battery(x, y);
    let units = sim.units();
    assert!((5..=7).contains(&units.len()), "{} units", units.len());
    for (k, u) in units.iter().enumerate() {
        assert!(!sim.aaa_blue(k) && !u.destroyed);
        let d = stealth_sim::math::approx(u.x - x, u.y - y);
        assert!(d <= 0x60, "unit {k} {d} away");
    }
}

#[test]
fn a_pilot_down_behind_the_lines_waits_for_the_rescue_helicopter() {
    let (_, mut sim) = campaign();
    let base = find(&sim, 0x401, None);
    sim.respawn_player(base, None);
    // Nearest to an enemy airfield, far from the friendly ones.
    let o = (3..sim.objects().len())
        .map(|i| sim.objects()[i])
        .find(|o| o.flags & 0xF0F == oflags::AIRFIELD)
        .expect("enemy airfield");
    let (x, y) = Sim::map_to_world(o.x + 0x80, o.y);
    let mut ps = PlayerState {
        x,
        y,
        altitude: 2000,
        height: 2000,
        velocity: 250 * 27,
        gear_up: true,
        fuel: 10000,
        on_ground: false,
        ..Default::default()
    };
    ps.cmd.eject = true;
    let mut ev = sim.step(&ps, 15);
    ps.cmd = Commands::default();
    ev.extend(run(&mut sim, &ps, 120));
    assert!(ev.iter().any(|e| matches!(e, SimEvent::PilotDown { .. })), "{ev:?}");
    assert!(!ev.iter().any(|e| matches!(e, SimEvent::SortieEnded(_))));
    let c = sim.csar().expect("rescue on its way").clone();
    assert!(!sim.player_absent());
    // The friendly air defence leaves the Black Hawk alone, the enemy's goes for it.
    let side = |i: usize| sim.friendly_unit(i, sim.flights()[i].spec);
    assert!(side(c.rescue));
    assert!(c.hunters.iter().all(|&i| !side(i)));
    // Skip the extraction: the sortie ends as an ejection, the helicopters go home.
    sim.end_sortie(0);
    let ev = sim.take_pilot_events();
    let end = ev.iter().find_map(|e| match e.event {
        SimEvent::SortieEnded(e) => Some(e),
        _ => None,
    });
    assert_eq!(end.map(|e| (e.outcome, e.landing_type)), Some((0, 2)));
    assert!(sim.csar().is_none());
    for i in std::iter::once(c.rescue).chain(c.hunters) {
        let u = sim.flights()[i];
        assert!(u.flags & fflags::DESTROYED != 0 || u.flags & fflags::RTB != 0, "{u:?}");
    }
}

use stealth_sim::ai_missions::{A10, AH1Z, AiTask, F16, F117};

#[test]
fn an_ai_recon_flight_photographs_its_object_and_flies_home() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    let i = sim.spawn_ai_mission(from, F117, AiTask::Recon(to), 61, true);
    assert_eq!(sim.flights()[i].spec, F117);
    let (mut photo, mut landed) = (false, false);
    for _ in 0..60 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            match e {
                SimEvent::CampaignPhotographed { tag: 61, .. } => photo = true,
                SimEvent::CampaignFlightLanded { tag: 61, .. } => landed = true,
                _ => {}
            }
        }
        if landed {
            break;
        }
    }
    assert!(photo, "no photo");
    assert!(landed, "it did not get home");
}

#[test]
fn ai_gunships_fire_at_the_enemys_guns() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let b = sim.objects()[from];
    let (x, y) = (b.x + 0x300, b.y);
    sim.place_defences(x, y, false, false);
    assert!(!sim.units().is_empty());
    for tag in [71, 72] {
        sim.spawn_ai_mission(from, AH1Z, AiTask::Ground(x, y), tag, true);
    }
    let mut seen = Vec::new();
    until_down(&mut sim, 0, 20 * 60, &mut seen);
    assert!(seen.contains(&27), "no rockets at the guns: {seen:?}");
}

/// A CAS flight of two Tornados fires its rockets at the units of its objective (not at the
/// enemy unit nearest its point), one unit each, in several passes, flying out and back in
/// between, and goes home once the objective is gone or its passes are flown.
#[test]
fn ai_cas_flies_several_passes_at_its_objectives_units() {
    use stealth_sim::aaa::GroundKind;
    use stealth_sim::ai_missions::TORNADO;
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let b = sim.objects()[from];
    let (x, y) = (b.x + 0x600, b.y);
    sim.place_defences(x, y, false, false);
    let n = sim.units().len();
    assert!(n >= 4, "{n} units");
    // Tanks (they do not shoot at aircraft); the first right on the point, not the objective's.
    for (k, u) in sim.units_mut().iter_mut().enumerate() {
        u.kind = GroundKind::T90;
        (u.x, u.y) = if k == 0 { (x, y) } else { (x + 0x40, y + 0x10 * k as i32) };
    }
    let objective: Vec<u32> = (1..n as u32).collect();
    for tag in [71, 72] {
        sim.spawn_ai_mission(from, TORNADO, AiTask::Ground(x, y), tag, true);
        sim.set_cas_units(tag, &objective);
    }
    let (mut aimed, mut fired, mut home) = (Vec::new(), 0, 0);
    let mut seen: Vec<(usize, usize)> = Vec::new();
    for _ in 0..30 * 60 {
        for e in run(&mut sim, &idle(), 1) {
            if matches!(e, SimEvent::CampaignFlightLanded { tag: 71 | 72, .. }) {
                home += 1;
            }
        }
        for m in sim.ai_missiles() {
            if let Some(g) = m.ground
                && !seen.contains(&(m.target, g))
            {
                seen.push((m.target, g));
                aimed.push(g);
                fired += 1;
            }
        }
        if home == 2 {
            break;
        }
    }
    eprintln!("rockets at {aimed:?}");
    assert_eq!(home, 2, "not both home");
    assert!(fired > 2, "only {fired} salvos: one pass each");
    assert!(!aimed.contains(&0), "rockets at the unit outside the objective: {aimed:?}");
}

#[test]
fn a_pilot_takes_over_an_ai_aircraft_where_it_flies_and_hands_it_back() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = sim.objects()[find(&sim, 0x001, Some("Tabor"))];
    let i = sim.spawn_ai_mission(from, F16, AiTask::Patrol(to.x, to.y), 81, true);
    run(&mut sim, &idle(), 4 * 60);
    let u = sim.flights()[i];
    assert!(u.flags & fflags::AIRBORNE != 0 && u.alt > 1000, "{u:?}");
    let airframe = Airframe { external_stores: true, ..Default::default() };
    let st = sim.take_control(0, 81, from, None, airframe).expect("taken over");
    assert_eq!((st.x, 0x10_0000 - st.y), (u.wx, u.wy));
    assert_eq!(st.heading, u.heading as u16);
    assert!(!st.on_ground && st.gear_up);
    assert!((st.velocity as f64 / 27.0) > 200.0, "{} kt", st.velocity / 27);
    assert!(sim.flight_of_tag(81).is_none(), "the AI does not fly it any more");
    assert!(!sim.pilot_absent(0) && sim.pilot_in_the_air(0));
    // No landing, no loss: the campaign hears nothing.
    let ev = run(&mut sim, &st, 1);
    assert!(!ev.iter().any(|e| matches!(
        e,
        SimEvent::CampaignFlightLanded { tag: 81, .. }
            | SimEvent::CampaignFlightDestroyed { tag: 81, .. }
    )));
    // Back to the AI where he is.
    let p = sim.pilot_input(0);
    assert!(sim.hand_over(0, 82, F16, AiTask::Patrol(to.x, to.y), from, false, true));
    let k = sim.flight_of_tag(82).expect("the AI flies it");
    let a = sim.flights()[k];
    assert_eq!((a.wx, a.wy), (p.x, 0x10_0000 - p.y));
    assert_eq!(a.heading, p.heading as i32 as i16 as i32);
    assert!(sim.pilot_absent(0));
    let ended: Vec<MissionEnd> = sim
        .take_pilot_events()
        .into_iter()
        .filter_map(|e| match e.event {
            SimEvent::SortieEnded(m) => Some(m),
            _ => None,
        })
        .collect();
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0].outcome, OUTCOME_HANDED_OVER);
    // It flies on.
    run(&mut sim, &idle(), 30);
    assert!(sim.flight_of_tag(82).is_some());
}

#[test]
fn air_defences_find_a_stealthy_ai_aircraft_closer() {
    let mut fired = Vec::new();
    for spec in [F16, stealth_sim::ai_missions::F22] {
        let (_, mut sim) = campaign();
        quiet(&mut sim);
        let from = find(&sim, 0x401, Some("Neuburg"));
        let b = sim.objects()[from];
        let (px, py) = (b.x, b.y - 0x1000);
        // A SAM site the F-16 over the point is in reach of, the F-22 not.
        let site = &mut sim.objects_mut()[3];
        site.unit_type = 5;
        site.x = px + 0xA00;
        site.y = py;
        site.flags = 0;
        let i = sim.spawn_ai_mission(from, spec, AiTask::Patrol(px, py), 91, true);
        sim.set_fighter_loadout(i, Default::default());
        let mut seen = Vec::new();
        until_down(&mut sim, 91, 15 * 60, &mut seen);
        fired.push(!seen.is_empty());
    }
    assert_eq!(fired, [true, false], "SAMs fired at the F-16, the F-22");
}

/// The AI's F-117A, F-22A and Tornado fly their strikes nap of the earth (both flight models):
/// at about 100 ft over highland hills (300 m high), never into them, all the way to the target
/// and most of the way home.
#[test]
fn low_flyers_follow_the_terrain_to_the_target_and_home() {
    use stealth_sim::ai_missions::{F22, TORNADO};
    for realistic in [true, false] {
        for spec in [F117, F22, TORNADO] {
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
            let opts =
                SimOptions { campaign: true, realistic_ai_flight: realistic, ..Default::default() };
            let mut sim = Sim::with_options(&d.sim, &m, opts);
            quiet(&mut sim);
            let from = find(&sim, 0x401, Some("Neuburg"));
            let to = find(&sim, 0x001, Some("Tabor"));
            // Highland hills everywhere but round the base.
            let b = sim.objects()[from];
            let cell = hills::CELL_M / 15.625;
            let mask = (0..hills::DIM * hills::DIM)
                .map(|k| {
                    let (col, row) = ((k % hills::DIM) as f64, (k / hills::DIM) as f64);
                    let d = (col * cell - b.x as f64).hypot(row * cell - b.y as f64);
                    if d < 0x180 as f64 { 0 } else { 255 }
                })
                .collect();
            let hills = hills::Hills::from_bytes(mask).unwrap();
            sim.set_hills(hills.clone());
            let relief = sim.relief();
            let i = sim.spawn_ai_mission(from, spec, AiTask::Strike(to), 61, true);
            let (mut agl, mut fired) = (Vec::new(), false);
            for _ in 0..30 * 60 {
                let ev = run(&mut sim, &idle(), 1);
                let name = format!("{spec} (realistic {realistic})");
                assert!(
                    !ev.iter()
                        .any(|e| matches!(e, SimEvent::AircraftCrashed { unit } if *unit == i)),
                    "{name} flew into the ground"
                );
                let u = sim.flights()[i];
                let Some(f) = sim.campaign_flight(i) else { break };
                fired |= f.strike.is_none();
                // Away from the base (climbing out, the approach).
                let out = (u.x - b.x).abs().max((u.y - b.y).abs());
                if u.flags & (fflags::TAKEOFF | fflags::FINAL | fflags::ROLLOUT) == 0 && out > 0x500
                {
                    agl.push(u.alt - relief.height(u.x, u.y).max(hills.ground_alt(u.x, u.y)));
                }
            }
            agl.sort();
            let median = agl.get(agl.len() / 2).copied().unwrap_or(0);
            eprintln!("{spec} realistic {realistic}: lowest {} ft, median {median} ft", agl[0]);
            assert!(fired, "{spec} (realistic {realistic}) never fired");
            assert!(agl.len() > 300 && median < 200, "{spec} (realistic {realistic}): {median} ft");
            assert!(agl[0] > 20, "{spec} (realistic {realistic}): {} ft", agl[0]);
        }
    }
}

/// A blue strike aircraft of type `strike` 128 km short of Tabor flying east for it, and a red
/// fighter of type `fighter` (its patrol load) `dx`, `dy` map units off it heading `heading`,
/// both flying the realistic model.
fn intercept(strike: i32, fighter: i32, (dx, dy, heading): (i32, i32, i32)) -> (Sim, usize, usize) {
    use stealth_sim::ai_loadout::{AiLoadout, FighterTask};
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
    let opts = SimOptions { campaign: true, realistic_ai_flight: true, ..Default::default() };
    let mut sim = Sim::with_options(&d.sim, &m, opts);
    quiet(&mut sim);
    let from = find(&sim, 0x401, Some("Neuburg"));
    let to = find(&sim, 0x001, Some("Tabor"));
    let enemy = find(&sim, 0x001, Some("Komarno"));
    let t = sim.objects()[to];
    let i = sim.spawn_ai_mission(from, strike, AiTask::Strike(to), 61, true);
    let (sx, sy) = (t.x - 0x2000, t.y);
    place(&mut sim, i, sx, sy, 1200, 0x4000, 400);
    let f = sim.spawn_fighter(enemy, sx, sy, 8, false, FighterTask::Patrol);
    place(&mut sim, f, sx + dx, sy + dy, 4000, heading, 600);
    sim.flights_mut()[f].spec = fighter;
    let missile = if matches!(fighter, 0 | 25) { 19 } else { 21 };
    sim.set_fighter_loadout(f, AiLoadout::for_task(fighter, missile, FighterTask::Patrol));
    (sim, i, f)
}

/// Flight unit `i` in the air at map (x, y), `alt`, heading `heading`, `speed`.
fn place(sim: &mut Sim, i: usize, x: i32, y: i32, alt: i32, heading: i32, speed: i32) {
    let u = &mut sim.flights_mut()[i];
    (u.wx, u.wy, u.x, u.y, u.alt) = (x << 5, y << 5, x, y, alt);
    (u.heading, u.pitch, u.bank, u.speed) = (heading, 0, 0, speed);
    u.flags = fflags::EXISTS | fflags::AIRBORNE;
}

/// A Tornado on its way to the target, a MiG-23 coming at it head on: it keeps its Sidewinders
/// for the fight the MiG presses it into, shoots it down, and flies on to fire at its target.
#[test]
fn a_pressed_strike_aircraft_fights_off_its_attacker_and_flies_on() {
    use stealth_sim::ai_missions::TORNADO;
    let (mut sim, i, _) = intercept(TORNADO, 0, (0x900, 0, -0x4000));
    let (mut fought, mut shot_first, mut fighter_lost, mut fired_at_target) =
        (0, None, false, false);
    for _ in 0..15 * 60 {
        let ev = run(&mut sim, &idle(), 1);
        for e in &ev {
            match e {
                SimEvent::CampaignFlightDestroyed { tag: 61, .. } => panic!("the Tornado was lost"),
                SimEvent::CampaignFlightDestroyed { tag: 8, .. } => fighter_lost = true,
                _ => {}
            }
        }
        let Some(c) = sim.campaign_flight(i) else { break };
        if c.defending {
            fought += 1;
        }
        if c.loadout.missiles() < 2 && shot_first.is_none() {
            shot_first = Some(c.defending);
        }
        if c.strike.is_none() {
            fired_at_target = true;
            break;
        }
    }
    assert_eq!(shot_first, Some(true), "its Sidewinders went before it was pressed, or never");
    assert!(fighter_lost, "the MiG-23 got away");
    assert!(fought > 3 && fought < 60, "fought {fought} s");
    assert!(fired_at_target, "it never got on to its target");
}

/// An F-16 strike flight fires its AMRAAMs at a fighter coming on, without turning off its
/// course for it.
#[test]
fn a_strike_aircraft_shoots_at_an_enemy_coming_on() {
    let (mut sim, i, _) = intercept(F16, 0, (0x900, 0, -0x4000));
    for _ in 0..90 {
        run(&mut sim, &idle(), 1);
        let Some(c) = sim.campaign_flight(i) else { panic!("the F-16 was lost") };
        if c.loadout.missiles() < 4 {
            assert!(!c.defending, "it fired only once pressed");
            assert!(sim.ai_missiles().iter().any(|m| m.spec == stealth_sim::ai_loadout::AIM120));
            return;
        }
    }
    panic!("the F-16 never fired");
}

/// A MiG-29 coming at an A-10 from the side engages it, closes in and shoots it down (it does
/// not fly past it, nor round it): in the merge its AA-11s, its gun.
#[test]
fn a_fighter_engages_the_strike_aircraft_it_meets() {
    let (mut sim, i, f) = intercept(A10, 2, (0, 0x900, 0));
    let mut close = 0;
    for s in 0..5 * 60 {
        let ev = run(&mut sim, &idle(), 1);
        if ev.iter().any(|e| matches!(e, SimEvent::CampaignFlightDestroyed { tag: 61, .. })) {
            assert!(close < 60, "{close} s round the A-10 before getting it");
            return;
        }
        let (a, b) = (sim.flights()[i], sim.flights()[f]);
        if (a.x - b.x).abs().max((a.y - b.y).abs()) < 0xC0 {
            close += 1;
        }
        assert!(sim.campaign_flight(f).is_some(), "the MiG-29 was lost at {s} s");
    }
    panic!("the MiG-29 never got the A-10");
}

/// A player flying a cargo run (its tag on his aircraft): the fighters hunting or escorting
/// that tag follow him once he is in the air, not before.
#[test]
fn the_cargo_runs_hunters_follow_the_player_flying_it() {
    let (_, mut sim) = campaign();
    quiet(&mut sim);
    let base = find(&sim, 0x401, None);
    let st = sim.respawn_player(base, None);
    sim.set_cargo_tag(0, Some(77));
    let o = sim.objects()[base];
    let i = sim.spawn_ai_mission(
        base,
        stealth_sim::ai_missions::F16,
        stealth_sim::ai_missions::AiTask::Hunt(77, o.x + 0x400, o.y),
        900,
        true,
    );
    run(&mut sim, &st, 2);
    let goal = |sim: &Sim| sim.campaign_flight(i).expect("the escort").goal;
    assert_eq!(goal(&sim), (o.x + 0x400, o.y), "waits while he is on the ground");
    let (mx, my) = (o.x - 0x600, o.y + 0x300);
    let (x, y) = Sim::map_to_world(mx, my);
    let air = PlayerState {
        x,
        y,
        altitude: 3000,
        height: 3000,
        on_ground: false,
        velocity: 250 * 27,
        gear_up: true,
        ..st.clone()
    };
    run(&mut sim, &air, 2);
    // An escort keeps station 2 km to one side of him.
    let (gx, gy) = goal(&sim);
    let side = ((gx - mx).abs() - 0x80).abs();
    assert!(side <= 4 && (gy - my).abs() <= 4, "goal {:?} vs {:?}", (gx, gy), (mx, my));
}
