use stealth_mission::TheaterData;

/// The tests read the campaign's texts in English.
trait English {
    fn en(&self) -> String;
    fn contains(&self, s: &str) -> bool {
        self.en().contains(s)
    }
    fn starts_with(&self, s: &str) -> bool {
        self.en().starts_with(s)
    }
}

impl English for stealth_i18n::Text {
    fn en(&self) -> String {
        self.tr_in(stealth_i18n::Language::En)
    }
}

use super::*;

fn theater(code: &str) -> TheaterData {
    let path = format!("{}/../../assets/mission/theaters/{code}.json", env!("CARGO_MANIFEST_DIR"));
    TheaterData::from_json(&std::fs::read(path).expect("theater file")).expect("theater")
}

/// The theater's relief file's bytes.
fn relief(code: &str) -> Option<std::sync::Arc<Vec<u8>>> {
    let path = format!("{}/../../assets/theaters/{code}.relief", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(path).ok().map(std::sync::Arc::new)
}

fn campaign(code: &str, index: usize) -> CampaignState {
    let t = theater(code);
    let setup = CampaignSetup {
        theater: index,
        theater_code: code.into(),
        roe: 1,
        opponents: 1,
        seed: 1234,
        sky_ratio: 24.0,
        start_hour: 8.0,
        dynamic_weather: true,
    };
    CampaignState::new(setup, &t.world)
}

const P0: PlayerId = PlayerId(0);

/// Player `p` flies over object `at` long enough to count as there.
fn be_there(c: &mut CampaignState, p: PlayerId, at: ObjectId) {
    let s = c.site(at).unwrap();
    let me = WorldView {
        players: vec![PlayerPos {
            id: p,
            x: s.x,
            y: s.y,
            airborne: true,
            landed: false,
            hover_over: None,
        }],
        ..Default::default()
    };
    c.tick(tuning::PRESENCE_SECONDS + 1.0, &me, &[]);
}

fn place_named(c: &CampaignState, name: &str) -> PlaceId {
    c.places.iter().position(|p| p.name == name).expect(name) as PlaceId
}

#[test]
fn places_and_sides_from_the_theater() {
    for (code, index) in
        [("central_europe", 3), ("ukraine", 9), ("persian_gulf", 1), ("vietnam", 4)]
    {
        let c = campaign(code, index);
        assert!(c.held(Side::Blue) > 0, "{code}: no friendly places");
        assert!(c.held(Side::Red) > 0, "{code}: no enemy places");
        assert!(!c.bases(Side::Blue).is_empty(), "{code}: no friendly bases");
        assert!(!c.front(Side::Blue).is_empty(), "{code}: no front");
        // Every SAM site that belongs to a place is one of its defences.
        for s in c.sites.iter().filter(|s| s.kind == SiteKind::Sam) {
            if let Some(p) = s.place {
                assert!(c.places[p as usize].defences.contains(&s.object));
            }
        }
    }
    // Central Europe: Tabor is a city with an airfield and a SAM site.
    let c = campaign("central_europe", 3);
    let tabor = &c.places[place_named(&c, "Tabor") as usize];
    assert_eq!(tabor.owner, Side::Red);
    assert!(tabor.airfield.is_some());
    assert_eq!(tabor.defences.len(), 1);
    assert_eq!(c.places[place_named(&c, "Ramstein") as usize].owner, Side::Blue);
    assert_eq!(c.places[place_named(&c, "Vienna") as usize].owner, Side::Neutral);
}

#[test]
fn the_board_offers_missions_on_the_front() {
    let mut c = campaign("central_europe", 3);
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::BoardChanged));
    let kinds: Vec<MissionKind> = c.board.iter().map(|o| o.kind).collect();
    for k in [MissionKind::Recon, MissionKind::Insertion, MissionKind::Cap] {
        assert!(kinds.contains(&k), "no {k:?} in {kinds:?}");
    }
    assert!(kinds.contains(&MissionKind::Sead) || kinds.contains(&MissionKind::Strike));
    assert!(c.board.len() <= tuning::BOARD_SIZE);
    // Offers keep their ids from one rebuild to the next.
    let ids: Vec<u32> = c.board.iter().map(|o| o.id).collect();
    let ev = c.tick(tuning::BOARD_REFRESH + 1.0, &WorldView::default(), &[]);
    assert!(!ev.contains(&CampaignEvent::BoardChanged));
    assert_eq!(ids, c.board.iter().map(|o| o.id).collect::<Vec<_>>());
}

/// Orders an insertion against the first front place and lifts the helicopters off.
fn launch(c: &mut CampaignState) -> (PlaceId, u32, ObjectId, Vec<u32>) {
    let place = c.front(Side::Blue)[0];
    c.apply(P0, Command::OrderInsertion { place }).expect("order");
    assert!(c.apply(P0, Command::OrderInsertion { place }).is_err(), "ordered twice");
    let ev = c.tick(tuning::INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    let (id, target, tags) = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { id, target, tags, .. } => {
                Some((*id, *target, tags.clone()))
            }
            _ => None,
        })
        .expect("launched");
    assert_eq!(tags.len(), tuning::INSERTION_HELOS);
    (place, id, target, tags)
}

#[test]
fn a_landed_helicopter_captures_the_place() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (place, _, target, tags) = launch(&mut c);
    let defences = c.places[place as usize].defences.clone();
    let t = c.site(target).unwrap().clone();
    // Three shot down, the last one lands.
    let mut reports: Vec<Report> = tags[..3]
        .iter()
        .map(|&tag| Report::FlightDestroyed { tag, side: Side::Blue, x: 0, y: 0, by: None })
        .collect();
    reports.push(Report::FlightLanded { tag: tags[3], x: t.x + 10, y: t.y - 10 });
    let ev = c.tick(1.0, &WorldView::default(), &reports);
    let captured = ev.iter().find_map(|e| match e {
        CampaignEvent::PlaceCaptured { place: p, side, removed, .. } => {
            Some((*p, *side, removed.clone()))
        }
        _ => None,
    });
    assert_eq!(captured, Some((place, Side::Blue, defences.clone())));
    assert_eq!(c.places[place as usize].owner, Side::Blue);
    assert_eq!(c.owner(target), Some(Side::Blue));
    for d in defences {
        assert!(c.site(d).unwrap().removed);
    }
    // The airfield is a friendly base now, and the board moved on.
    if c.places[place as usize].airfield.is_some() {
        assert!(c.bases(Side::Blue).contains(&target));
    }
    assert!(!c.front(Side::Blue).contains(&place));
}

#[test]
fn an_insertion_fails_when_every_helicopter_is_lost() {
    let mut c = campaign("central_europe", 3);
    let (place, id, _, tags) = launch(&mut c);
    let reports: Vec<Report> = tags
        .iter()
        .map(|&tag| Report::FlightDestroyed { tag, side: Side::Blue, x: 0, y: 0, by: None })
        .collect();
    let ev = c.tick(1.0, &WorldView::default(), &reports);
    assert!(ev.contains(&CampaignEvent::InsertionFailed { id, place }));
    assert_eq!(c.places[place as usize].owner, Side::Red);
    // A landing somewhere else takes nothing either.
    assert_eq!(c.insertion(id).unwrap().state, InsertionState::Failed);
}

#[test]
fn a_recon_sortie_gives_intelligence() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Recon).unwrap().clone();
    let Objective::Recon { place, object } = offer.objective else { unreachable!() };
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F117 })
        .expect("take");
    assert!(
        c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F117 })
            .is_err()
    );
    let ev = c.tick(1.0, &WorldView::default(), &[Report::PhotoTaken { player: P0, object }]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    assert!(
        ev.iter().any(|e| matches!(e, CampaignEvent::IntelGained { place: p, .. } if *p == place))
    );
    assert!(c.has_intel(object));
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap();
    let CampaignEvent::SortieFinished { record, .. } = &ev[0] else { panic!() };
    assert_eq!(record.result, SortieResult::Success);
    // The intelligence goes stale.
    let ev = c.tick(tuning::INTEL_SECONDS, &WorldView::default(), &[]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::IntelExpired { .. })));
    assert!(!c.has_intel(object));
}

#[test]
fn strikes_stop_and_repairs_restore_an_airfield() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Strike).unwrap().clone();
    let Objective::Strike { airfield, .. } = offer.objective else { unreachable!() };
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F117 })
        .unwrap();
    let ev = c.tick(
        1.0,
        &WorldView::default(),
        &[Report::ObjectDestroyed { object: airfield, by: Some(P0) }],
    );
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    assert!(!c.bases(Side::Red).contains(&airfield));
    let ev = c.tick(tuning::REPAIR_AIRFIELD, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::Repaired { object: airfield }));
    assert!(c.bases(Side::Red).contains(&airfield));
}

#[test]
fn an_escort_is_settled_by_its_insertion() {
    let mut c = campaign("central_europe", 3);
    let place = c.front(Side::Blue)[0];
    c.apply(P0, Command::OrderInsertion { place }).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    let escort = c.board.iter().find(|o| o.kind == MissionKind::Escort).unwrap().clone();
    c.apply(P0, Command::TakeMission { offer: escort.id, base: None, aircraft: Aircraft::F117 })
        .unwrap();
    let Objective::Escort { insertion } = escort.objective else { unreachable!() };
    let from = c.insertion(insertion).unwrap().from;
    be_there(&mut c, P0, from);
    // The escort lands before the helicopters get there: pending.
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap();
    let CampaignEvent::SortieFinished { record, .. } = &ev[0] else { panic!() };
    assert_eq!(record.result, SortieResult::Pending);
    let ev = c.tick(tuning::INSERTION_LEAD, &WorldView::default(), &[]);
    let tags = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, target, .. } => Some((tags.clone(), *target)),
            _ => None,
        })
        .unwrap();
    let t = c.site(tags.1).unwrap().clone();
    c.tick(1.0, &WorldView::default(), &[Report::FlightLanded { tag: tags.0[0], x: t.x, y: t.y }]);
    assert_eq!(c.history.last().unwrap().result, SortieResult::Success);
}

#[test]
fn enemy_patrols_and_scrambles() {
    let mut c = campaign("central_europe", 3);
    let ev = c.tick(tuning::PATROL_EVERY + 1.0, &WorldView::default(), &[]);
    let patrol = ev.iter().find_map(|e| match e {
        CampaignEvent::LaunchFighters { tag, side: Side::Red, from, .. } => Some((*tag, *from)),
        _ => None,
    });
    let (tag, from) = patrol.expect("a patrol");
    assert_eq!(c.owner(from), Some(Side::Red));
    // The patrol stays tracked while the simulation has it.
    let view = WorldView {
        flights: vec![FlightView { x: 0, y: 0, side: Side::Red, tag }],
        ..Default::default()
    };
    c.tick(60.0, &view, &[]);
    assert!(c.patrols.iter().any(|p| p.0 == tag));
    c.tick(60.0, &WorldView::default(), &[]);
    assert!(c.patrols.is_empty());
}

#[test]
fn the_campaign_saves_and_loads() {
    let mut c = campaign("ukraine", 9);
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, id, _, _) = launch(&mut c);
    let json = serde_json::to_string(&c).unwrap();
    let mut d: CampaignState = serde_json::from_str(&json).unwrap();
    assert_eq!(c, d);
    d.after_load();
    assert!(matches!(d.insertion(id).unwrap().state, InsertionState::Pending { .. }));
}

#[test]
fn the_same_inputs_give_the_same_campaign() {
    let run = || {
        let mut c = campaign("central_europe", 3);
        for k in 0..200 {
            c.tick(30.0, &WorldView::default(), &[]);
            if k == 10 {
                let place = c.front(Side::Blue)[0];
                let _ = c.apply(P0, Command::OrderInsertion { place });
            }
        }
        serde_json::to_string(&c).unwrap()
    };
    assert_eq!(run(), run());
}

#[test]
fn sites_are_spotted_by_rwr_and_recon() {
    let mut c = campaign("central_europe", 3);
    let wittstock = place_named(&c, "Wittstock");
    let sam = c.places[wittstock as usize].defences[0];
    assert!(!c.site(sam).unwrap().spotted);
    c.tick(1.0, &WorldView::default(), &[Report::SiteSpotted { object: sam }]);
    assert!(c.site(sam).unwrap().spotted);
    // Recon photos map a place's defences.
    let tabor = place_named(&c, "Tabor");
    let other = c.places[tabor as usize].defences[0];
    let mut ev = Vec::new();
    c.gain_intel(tabor, &mut ev);
    assert!(c.site(other).unwrap().spotted);
}

#[test]
fn a_full_campaign_hands_an_idle_slot_to_a_new_name() {
    // Slot 0 is the local player's.
    let mut c = campaign("central_europe", 3);
    let b = PlayerId(1);
    assert_eq!(c.join("Alpha", 3, |_| false), Some(b));
    assert_eq!(c.join("Bravo", 3, |_| false), Some(PlayerId(2)));
    assert_eq!(c.join("alpha", 3, |_| false), Some(b), "the same name, the same slot");
    // Everybody connected: no room.
    assert_eq!(c.join("Charlie", 3, |_| true), None);
    // Alpha left: Charlie gets his slot.
    assert_eq!(c.join("Charlie", 3, |id| id != b), Some(b));
    assert_eq!(c.player_name(b), Some("Charlie"));
}

#[test]
fn players_share_a_plan_locked_while_it_is_flown() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let b = PlayerId(1);
    c.join("Bravo", 3, |_| false);
    let offer = c.board.iter().find(|o| !o.kind.ai_flown()).unwrap().clone();
    let bases = c.bases(Side::Blue);
    let other = *bases.iter().find(|&&x| x != offer.base).expect("two bases");
    let select = |base| Command::Select { offer: Some(offer.id), base };
    c.apply(P0, select(offer.base)).expect("select");
    // The second player joins on the plan's base, whatever he chose.
    let ev = c.apply(b, select(other)).expect("join");
    let CampaignEvent::PlansChanged(plans) = &ev[0] else { panic!() };
    assert_eq!(plans[0].crew, vec![P0, b]);
    assert_eq!(plans[0].base, offer.base);
    let route = vec![
        RoutePoint { x: 100, y: 100, kind: RouteKind::Nav },
        RoutePoint { x: offer.x, y: offer.y, kind: RouteKind::Primary },
    ];
    let plan = |route: Vec<RoutePoint>| Command::Plan { offer: offer.id, base: offer.base, route };
    c.apply(b, plan(route.clone())).expect("edit");
    assert_eq!(c.plan(offer.id).unwrap().route, route);
    // Flown: locked; the pilot leaves the crew.
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F117 })
        .expect("take");
    assert_eq!(c.plan(offer.id).unwrap().flying, vec![P0]);
    assert_eq!(c.plan(offer.id).unwrap().crew, vec![b]);
    assert!(c.apply(b, plan(Vec::new())).is_err());
    // Nobody flying any more: editable again, the route kept.
    c.apply(P0, Command::EndSortie { end: SortieEnd::Aborted }).expect("end");
    assert_eq!(c.plan(offer.id).unwrap().route, route);
    c.apply(b, plan(Vec::new())).expect("edit again");
    // A new base resets the route.
    c.apply(b, plan(route.clone())).unwrap();
    c.apply(b, Command::Plan { offer: offer.id, base: other, route: route.clone() }).unwrap();
    assert_eq!(c.plan(offer.id).unwrap().base, other);
    assert!(c.plan(offer.id).unwrap().route.is_empty());
    // Leaving; the plan stays while its offer is on the board.
    c.apply(b, Command::Select { offer: None, base: 0 }).unwrap();
    assert!(c.plan(offer.id).unwrap().crew.is_empty());
    c.board.retain(|o| o.id != offer.id);
    assert!(c.prune_plans());
    assert!(c.plan(offer.id).is_none());
}

/// The friendly CAPs launched from `base` by these events (tags).
fn caps_from(ev: &[CampaignEvent], base: ObjectId) -> Vec<u32> {
    ev.iter()
        .filter_map(|e| match e {
            CampaignEvent::LaunchFighters { tag, side: Side::Blue, from, task, .. } => {
                assert_eq!(*task, FighterTask::Cap);
                (*from == base).then_some(*tag)
            }
            _ => None,
        })
        .collect()
}

/// A friendly airfield with enemy aircraft 20 km off it.
fn raided_base(c: &CampaignState, raiders: u32) -> (ObjectId, WorldView) {
    let base = c.bases(Side::Blue)[0];
    let s = c.site(base).unwrap();
    let flights = (0..raiders)
        .map(|k| FlightView {
            x: s.x + 0x500,
            y: s.y + k as i32 * 0x20,
            side: Side::Red,
            tag: 900 + k,
        })
        .collect();
    (base, WorldView { flights, ..Default::default() })
}

/// The view with these friendly fighters in the air too.
fn with_caps(view: &WorldView, tags: &[u32]) -> WorldView {
    let mut v = view.clone();
    v.flights.extend(tags.iter().map(|&tag| FlightView { x: 0, y: 0, side: Side::Blue, tag }));
    v
}

#[test]
fn a_raid_scrambles_a_cap() {
    let mut c = campaign("central_europe", 3);
    let (base, view) = raided_base(&c, 1);
    let caps = caps_from(&c.tick(1.0, &view, &[]), base);
    assert!((1..=2).contains(&caps.len()), "{caps:?}");
    // None more while it flies.
    assert!(caps_from(&c.tick(60.0, &with_caps(&view, &caps), &[]), base).is_empty());
    // Two raiders: two fighters.
    let mut c = campaign("central_europe", 3);
    let (base, view) = raided_base(&c, 2);
    assert_eq!(caps_from(&c.tick(1.0, &view, &[]), base).len(), 2);
    // Nothing near: no CAP.
    let mut c = campaign("central_europe", 3);
    assert!(caps_from(&c.tick(1.0, &WorldView::default(), &[]), base).is_empty());
}

#[test]
fn a_lost_cap_is_not_replaced_for_five_minutes() {
    let mut c = campaign("central_europe", 3);
    let (base, view) = raided_base(&c, 2);
    let caps = caps_from(&c.tick(1.0, &view, &[]), base);
    let lost = Report::FlightDestroyed { tag: caps[0], side: Side::Blue, x: 0, y: 0, by: None };
    let landed = Report::FlightLanded { tag: caps[1], x: 0, y: 0 };
    assert!(caps_from(&c.tick(1.0, &view, &[lost, landed]), base).is_empty());
    assert!(c.caps.iter().all(|cap| cap.base != base));
    // The raiders are still there.
    assert!(caps_from(&c.tick(tuning::AI_CAP_RESPAWN - 10.0, &view, &[]), base).is_empty());
    assert!(!caps_from(&c.tick(20.0, &view, &[]), base).is_empty());
}

#[test]
fn a_landed_cap_can_go_again() {
    let mut c = campaign("central_europe", 3);
    let (base, view) = raided_base(&c, 1);
    let caps = caps_from(&c.tick(1.0, &view, &[]), base);
    let landed: Vec<Report> =
        caps.iter().map(|&tag| Report::FlightLanded { tag, x: 0, y: 0 }).collect();
    c.tick(1.0, &WorldView::default(), &landed);
    assert!(c.caps.iter().all(|cap| cap.base != base));
    assert!(c.cap_lockout.iter().all(|l| l.0 != base));
    assert!(!caps_from(&c.tick(1.0, &view, &[]), base).is_empty());
}

#[test]
fn a_cap_fighter_gone_from_the_air_counts_as_lost() {
    let mut c = campaign("central_europe", 3);
    let (base, view) = raided_base(&c, 2);
    let caps = caps_from(&c.tick(1.0, &view, &[]), base);
    c.tick(40.0, &with_caps(&view, &caps), &[]);
    // One vanishes before its loss is reported; the report comes after.
    c.tick(1.0, &with_caps(&view, &caps[1..]), &[]);
    let lost = Report::FlightDestroyed { tag: caps[0], side: Side::Blue, x: 0, y: 0, by: None };
    c.tick(1.0, &with_caps(&view, &caps[1..]), &[lost]);
    assert_eq!(c.cap_lockout.iter().filter(|l| l.0 == base).count(), 1);
    assert!(caps_from(&c.tick(60.0, &with_caps(&view, &caps[1..]), &[]), base).is_empty());
}

#[test]
fn patriots_stand_in_the_rear() {
    for (code, index) in
        [("central_europe", 3), ("ukraine", 9), ("persian_gulf", 1), ("vietnam", 4)]
    {
        let c = campaign(code, index);
        let sites = c.patriot_sites();
        assert!(!sites.is_empty(), "{code}: no Patriots");
        assert!(sites.len() <= tuning::PATRIOT_BATTERIES + c.setup.roe as usize);
        let front = |p: PlaceId| {
            let p = &c.places[p as usize];
            c.places
                .iter()
                .filter(|q| q.owner == Side::Red)
                .map(|q| dist(q.x - p.x, q.y - p.y))
                .min()
                .unwrap_or(i32::MAX)
        };
        for &(p, x, y) in &sites {
            let place = &c.places[p as usize];
            assert!(
                place.owner == Side::Blue && place.airfield.is_some(),
                "{code}: {}",
                place.name
            );
            assert!(sites.len() == 1 || front(p) >= tuning::PATRIOT_REAR, "{code}: {}", place.name);
            assert!(dist(x - place.x, y - place.y) <= 2 * tuning::PATRIOT_OFFSET);
        }
    }
}

/// Ticks until the enemy orders its first insertion; the order's place.
fn enemy_order(c: &mut CampaignState) -> PlaceId {
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(tuning::ENEMY_INSERT_EVERY[c.setup.opponents as usize], &WorldView::default(), &[]);
    let ins = c.insertions.iter().find(|i| i.side == Side::Red).expect("an enemy insertion");
    assert_eq!(c.places[ins.place as usize].owner, Side::Blue);
    ins.place
}

/// Lifts the enemy's helicopters off: their insertion id, target and tags.
fn enemy_launch(c: &mut CampaignState) -> (u32, ObjectId, Vec<u32>) {
    let ev = c.tick(tuning::ENEMY_INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    ev.iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { id, side: Side::Red, target, tags, .. } => {
                Some((*id, *target, tags.clone()))
            }
            _ => None,
        })
        .expect("enemy helicopters launched")
}

#[test]
fn the_enemy_takes_a_friendly_place() {
    let mut c = campaign("central_europe", 3);
    let place = enemy_order(&mut c);
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    assert!(c.board.iter().any(|o| o.kind == MissionKind::Intercept), "an intercept offer");
    // No escort or insertion offers for the enemy's insertion.
    assert!(c.board.iter().all(|o| o.objective != Objective::Insertion { place }));
    let (_, target, tags) = enemy_launch(&mut c);
    let t = c.site(target).unwrap().clone();
    let ev = c.tick(
        1.0,
        &WorldView::default(),
        &[Report::FlightLanded { tag: tags[0], x: t.x, y: t.y }],
    );
    assert!(ev.iter().any(|e| matches!(
        e,
        CampaignEvent::PlaceCaptured { place: p, side: Side::Red, .. } if *p == place
    )));
    assert_eq!(c.places[place as usize].owner, Side::Red);
    assert!(!c.player_bases().contains(&target));
}

#[test]
fn an_intercept_succeeds_when_the_helicopters_go_down() {
    let mut c = campaign("central_europe", 3);
    let place = enemy_order(&mut c);
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Intercept).unwrap().clone();
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F16 })
        .unwrap();
    let (id, _, tags) = enemy_launch(&mut c);
    let down: Vec<Report> = tags
        .iter()
        .map(|&tag| Report::FlightDestroyed { tag, side: Side::Red, x: 0, y: 0, by: Some(P0) })
        .collect();
    let ev = c.tick(1.0, &WorldView::default(), &down);
    assert!(ev.contains(&CampaignEvent::InsertionFailed { id, place }));
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    assert_eq!(c.places[place as usize].owner, Side::Blue);
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap();
    let CampaignEvent::SortieFinished { record, .. } = &ev[0] else { panic!() };
    assert_eq!(record.result, SortieResult::Success);
    assert_eq!(record.air_kills, tuning::INSERTION_HELOS as u32);
}

/// The intercept is offered on the helicopters' track, with their timing.
#[test]
fn an_intercept_meets_the_helicopters_on_their_way() {
    let mut c = campaign("central_europe", 3);
    enemy_order(&mut c);
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let o = c.board.iter().find(|o| o.kind == MissionKind::Intercept).unwrap().clone();
    let Objective::Intercept { insertion } = o.objective else { unreachable!() };
    let ins = c.insertion(insertion).unwrap().clone();
    let (a, b) = (c.site(ins.from).unwrap(), c.site(ins.target).unwrap());
    let track = dist(a.x - b.x, a.y - b.y);
    // Between the enemy's base and its goal, well away from both.
    assert!(dist(o.x - a.x, o.y - a.y) < track && dist(o.x - b.x, o.y - b.y) < track);
    assert!(dist(o.x - b.x, o.y - b.y) > track / 4, "{o:?}");
    let t = c.insertion_status(insertion).unwrap();
    assert!(t.contains("lift off") && t.contains(&a.name), "{t}");
    enemy_launch(&mut c);
    assert!(c.insertion_status(insertion).unwrap().contains("due in"));
}

/// An enemy insertion without helicopters is not flown, and nobody stopped it.
#[test]
fn an_insertion_without_helicopters_is_called_off() {
    let mut c = campaign("central_europe", 3);
    enemy_order(&mut c);
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    take(&mut c, P0, MissionKind::Intercept);
    c.red.helicopters = 0;
    let ev = c.tick(tuning::ENEMY_INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    let ins = c.insertions.iter().find(|i| i.side == Side::Red).unwrap();
    assert_eq!(ins.state, InsertionState::CalledOff);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(t) if t.contains("called off"))));
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    let r = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap());
    assert_eq!(r.result, SortieResult::Failed);
}

/// Ticks until the enemy's first raid takes off: the raid's id, target and tags.
fn raid(c: &mut CampaignState) -> (u32, ObjectId, Vec<u32>) {
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(tuning::RAID_EVERY[c.setup.opponents as usize], &WorldView::default(), &[]);
    assert!(c.raids.iter().any(|r| r.open()), "a raid ordered");
    // The escort takes off first.
    let ev = c.tick(tuning::RAID_LEAD - tuning::RAID_ESCORT_LEAD + 1.0, &WorldView::default(), &[]);
    let escorts = ev
        .iter()
        .filter(|e| matches!(e, CampaignEvent::LaunchFighters { side: Side::Red, .. }))
        .count();
    assert!(escorts >= tuning::RAID_ESCORTS, "{ev:?}");
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::RaidLaunched { .. })));
    let ev = c.tick(tuning::RAID_ESCORT_LEAD, &WorldView::default(), &[]);
    ev.iter()
        .find_map(|e| match e {
            CampaignEvent::RaidLaunched { id, side: Side::Red, target, tags, .. } => {
                Some((*id, *target, tags.clone()))
            }
            _ => None,
        })
        .expect("strike aircraft launched")
}

#[test]
fn a_raid_grounds_the_ai_but_never_the_players() {
    let mut c = campaign("central_europe", 3);
    let (id, target, tags) = raid(&mut c);
    assert_eq!(tags.len(), tuning::RAID_AIRCRAFT);
    assert_eq!(c.owner(target), Some(Side::Blue));
    assert!(c.board.iter().any(|o| o.objective == Objective::Defend { raid: id }));
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: target, by: None }]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::Hit);
    // The AI does not fly from the bombed airfield; the players still do.
    assert!(!c.bases(Side::Blue).contains(&target));
    assert!(c.player_bases().contains(&target));
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| !o.kind.ai_flown()).unwrap().clone();
    c.apply(
        P0,
        Command::TakeMission { offer: offer.id, base: Some(target), aircraft: Aircraft::F117 },
    )
    .expect("take off from the bombed airfield");
}

#[test]
fn a_defended_airfield_repels_the_raid() {
    let mut c = campaign("central_europe", 3);
    let (id, target, tags) = raid(&mut c);
    let offer = c.board.iter().find(|o| o.objective == Objective::Defend { raid: id }).unwrap();
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F16 })
        .unwrap();
    // One shot down, the other turns back: repelled.
    let ev = c.tick(
        1.0,
        &WorldView::default(),
        &[
            Report::FlightDestroyed { tag: tags[0], side: Side::Red, x: 0, y: 0, by: Some(P0) },
            Report::FlightLanded { tag: tags[1], x: 0, y: 0 },
        ],
    );
    assert_eq!(c.raid(id).unwrap().state, RaidState::Repelled);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    assert!(c.bases(Side::Blue).contains(&target));
}

#[test]
fn a_defence_ended_early_is_settled_by_the_raid() {
    let mut c = campaign("central_europe", 3);
    let (id, target, _) = raid(&mut c);
    let offer = c.board.iter().find(|o| o.objective == Objective::Defend { raid: id }).unwrap();
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F16 })
        .unwrap();
    be_there(&mut c, P0, target);
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap();
    let CampaignEvent::SortieFinished { record, .. } = &ev[0] else { panic!() };
    assert_eq!(record.result, SortieResult::Pending);
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: target, by: None }]);
    assert_eq!(c.history.last().unwrap().result, SortieResult::Failed);
}

#[test]
fn the_war_is_won_and_lost() {
    let mut c = campaign("central_europe", 3);
    for p in c.places.iter_mut().filter(|p| p.owner == Side::Red) {
        p.owner = Side::Blue;
    }
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::Ended { winner: Side::Blue }));
    assert!(c.board.is_empty());
    assert_eq!(c.ended_for(), Some(0.0));
    // Over: nothing happens any more.
    let ev = c.tick(tuning::RAID_EVERY[3] * 4.0, &WorldView::default(), &[]);
    assert!(ev.is_empty(), "{ev:?}");
    assert_eq!(c.ended_for(), Some(tuning::RAID_EVERY[3] * 4.0));

    let mut c = campaign("central_europe", 3);
    for p in c.places.iter_mut().filter(|p| p.owner == Side::Blue && p.airfield.is_some()) {
        p.owner = Side::Red;
    }
    for s in c.sites.iter_mut().filter(|s| s.kind == SiteKind::Carrier) {
        s.removed = true;
    }
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::Ended { winner: Side::Red }));
    // No theater is over before it starts.
    for (code, index) in
        [("central_europe", 3), ("ukraine", 9), ("persian_gulf", 1), ("vietnam", 4)]
    {
        let mut c = campaign(code, index);
        c.tick(1.0, &WorldView::default(), &[]);
        assert!(c.outcome.is_none(), "{code}");
    }
}

#[test]
fn lost_guns_stay_lost_until_replaced() {
    let mut c = campaign("central_europe", 3);
    let tabor = place_named(&c, "Tabor");
    let (x, y) = (c.places[tabor as usize].x, c.places[tabor as usize].y);
    let gun = GroundView::gun(x + 20, y, Side::Red, false);
    let view = WorldView { units: vec![gun], ..Default::default() };
    c.tick(1.0, &view, &[Report::AaaDestroyed { index: 0, by: Some(P0) }]);
    assert_eq!(c.places[tabor as usize].guns_lost, 1);
    // Saved with the campaign.
    let d: CampaignState = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(d.places[tabor as usize].guns_lost, 1);
    let ev = c.tick(tuning::REPAIR_GUN, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::GunRepaired { place: tabor }));
    assert_eq!(c.places[tabor as usize].guns_lost, 0);
}

#[test]
fn an_older_save_still_loads() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    launch(&mut c);
    let mut v = serde_json::to_value(&c).unwrap();
    let o = v.as_object_mut().unwrap();
    for k in [
        "raids",
        "enemy_insert_at",
        "raid_at",
        "outcome",
        "blue",
        "red",
        "resupply_at",
        "income_at",
        "downed",
        "rules",
        "awacs",
        "awacs_at",
        "awacs_down",
        "drones_at",
    ] {
        o.remove(k);
    }
    for p in o["places"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("guns_lost");
        p.as_object_mut().unwrap().remove("guns_repair_at");
    }
    for i in o["insertions"].as_array_mut().unwrap() {
        i.as_object_mut().unwrap().remove("side");
    }
    let d: CampaignState = serde_json::from_value(v).expect("an older save");
    assert!(d.insertions.iter().all(|i| i.side == Side::Blue));
    assert!(d.outcome.is_none() && d.raids.is_empty());
}

#[test]
fn places_and_cities_bring_in_materiel() {
    let mut c = campaign("central_europe", 3);
    let (places, cities) = c.producers(Side::Red);
    assert_eq!(places, c.held(Side::Red));
    assert!(cities > 0);
    let income = c.income(Side::Red);
    assert_eq!(
        income,
        tuning::INCOME_BASE
            + tuning::INCOME_PLACE * places as f64
            + tuning::INCOME_CITY * cities as f64
    );
    c.red.materiel = 0.0;
    c.tick(1800.0, &WorldView::default(), &[]);
    // Half an hour of income (less what the enemy spent meanwhile).
    assert!((c.red.ledger.earned - income / 2.0).abs() < 1.0, "{:?}", c.red.ledger);
    let spent = c.red.ledger.repairs + c.red.ledger.operations + c.red.ledger.aircraft;
    assert!((c.red.materiel - (c.red.ledger.earned - spent)).abs() < 0.01);
    // A city bombed: materiel lost at once, and less income while it is in ruins.
    let city = c
        .sites
        .iter()
        .find(|s| s.kind == SiteKind::City && c.owner(s.object) == Some(Side::Red))
        .unwrap()
        .object;
    c.red.materiel = 100.0;
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: city, by: Some(P0) }]);
    assert_eq!(c.red.ledger.lost, tuning::CITY_LOSS);
    assert_eq!(c.income(Side::Red), income - tuning::INCOME_CITY);
}

#[test]
fn repairs_wait_for_materiel() {
    let mut c = campaign("central_europe", 3);
    let airfield = c.bases(Side::Red)[0];
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: airfield, by: None }]);
    c.red.materiel = 0.0;
    c.red.fighters = 0;
    c.income_at = f64::MAX;
    c.tick(tuning::REPAIR_AIRFIELD, &WorldView::default(), &[]);
    assert!(!c.bases(Side::Red).contains(&airfield), "repaired without materiel");
    c.red.materiel = tuning::REPAIR_COST_AIRFIELD;
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::Repaired { object: airfield }));
    assert_eq!(c.red.ledger.repairs, tuning::REPAIR_COST_AIRFIELD);
}

#[test]
fn the_ai_flies_the_aircraft_it_has() {
    let mut c = campaign("central_europe", 3);
    // Only the fighters a raid needs: no patrol.
    let reserve = (tuning::RAID_AIRCRAFT + tuning::RAID_ESCORTS) as u32;
    c.red.fighters = reserve;
    let ev = c.tick(tuning::PATROL_EVERY + 1.0, &WorldView::default(), &[]);
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::LaunchFighters { side: Side::Red, .. })));
    // One more: it goes, and comes back to the ground when it lands.
    c.red.fighters = reserve + 1;
    let ev = c.tick(tuning::PATROL_EVERY + 1.0, &WorldView::default(), &[]);
    let tag = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::LaunchFighters { tag, side: Side::Red, .. } => Some(*tag),
            _ => None,
        })
        .expect("a patrol");
    assert_eq!(c.red.fighters, reserve);
    assert_eq!(c.airborne(Side::Red).0, 1);
    c.tick(1.0, &WorldView::default(), &[Report::FlightLanded { tag, x: 0, y: 0 }]);
    assert_eq!(c.red.fighters, reserve + 1);
    // A friendly CAP only with fighters left.
    let mut c = campaign("central_europe", 3);
    c.blue.fighters = 0;
    let (base, view) = raided_base(&c, 2);
    assert!(caps_from(&c.tick(1.0, &view, &[]), base).is_empty());
}

#[test]
fn a_players_insertion_always_has_helicopters() {
    let mut c = campaign("central_europe", 3);
    c.blue.helicopters = 0;
    // The campaign's own needs them.
    let place = c.front(Side::Blue)[0];
    assert!(c.order_insertion(None, place).is_err());
    c.apply(P0, Command::OrderInsertion { place }).expect("a player's order");
    let ev = c.tick(tuning::INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    let tags = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, side: Side::Blue, .. } => Some(tags.clone()),
            _ => None,
        })
        .expect("launched");
    assert_eq!(tags.len(), tuning::RESERVE_HELICOPTERS);
    assert_eq!(c.blue.helicopters, 0);
}

#[test]
fn deep_strikes_hit_the_enemys_rear() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let deep: Vec<_> =
        c.board.iter().filter(|o| o.kind == MissionKind::Interdiction).cloned().collect();
    assert!(!deep.is_empty(), "{:?}", c.board.iter().map(|o| &o.title).collect::<Vec<_>>());
    assert!(deep.len() <= tuning::DEEP_TARGETS);
    let front = c.front(Side::Blue);
    for o in &deep {
        let (Objective::Interdict { place, .. } | Objective::Strike { place, .. }) = o.objective
        else {
            panic!("{o:?}");
        };
        assert!(!front.contains(&place));
    }
    // A city where there is one in reach (Ukraine has plenty), else an airfield.
    let mut c = campaign("ukraine", 9);
    c.tick(1.0, &WorldView::default(), &[]);
    let o = c
        .board
        .iter()
        .find(|o| matches!(o.objective, Objective::Interdict { .. }))
        .expect("a city to interdict")
        .clone();
    let Objective::Interdict { object, .. } = o.objective else { unreachable!() };
    c.apply(P0, Command::TakeMission { offer: o.id, base: None, aircraft: Aircraft::F117 })
        .unwrap();
    let before = c.red.materiel;
    let ev =
        c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object, by: Some(P0) }]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    assert!(c.red.materiel < before);
}

/// Player 0 ejects over the enemy's place `name`.
fn eject_over(c: &mut CampaignState, name: &str) -> (Vec<CampaignEvent>, i32, i32) {
    c.tick(1.0, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| !o.kind.ai_flown()).unwrap().id;
    c.apply(P0, Command::TakeMission { offer, base: None, aircraft: Aircraft::F16 }).unwrap();
    let p = &c.places[place_named(c, name) as usize];
    let (x, y) = (p.x + 30, p.y);
    let view = WorldView {
        players: vec![PlayerPos { id: P0, x, y, airborne: true, landed: false, hover_over: None }],
        ..Default::default()
    };
    c.tick(1.0, &view, &[]);
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Ejected }).unwrap();
    (ev, x, y)
}

#[test]
fn a_pilot_down_behind_the_lines_is_rescued() {
    let mut c = campaign("central_europe", 3);
    c.income_at = f64::MAX;
    c.resupply_at = f64::MAX;
    let (materiel, fighters) = (c.blue.materiel, c.blue.fighters);
    let (ev, x, y) = eject_over(&mut c, "Tabor");
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.contains("down near"))));
    // He is missing: his worth is lost until he is out.
    assert_eq!(c.blue.fighters, fighters - 1);
    assert!(c.blue.materiel <= materiel - tuning::CAPTURE_COST + 0.01);
    let d = c.downed[0].clone();
    assert_eq!((d.x, d.y, d.state), (x, y, DownedState::Waiting));
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Csar).unwrap().clone();
    assert_eq!(offer.objective, Objective::Rescue { downed: d.id });
    // The wingman covers the helicopter in.
    let b = PlayerId(1);
    c.join("Bravo", 4, |_| false);
    c.apply(b, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::F16 })
        .unwrap();
    let over = WorldView {
        players: vec![PlayerPos {
            id: b,
            x: x + 0x40,
            y,
            airborne: true,
            landed: false,
            hover_over: None,
        }],
        ..Default::default()
    };
    let mut rescued = false;
    for _ in 0..(tuning::RESCUE_SECONDS as i32 / 10 + 1) {
        rescued |= c
            .tick(10.0, &over, &[])
            .iter()
            .any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { player, .. } if *player == b));
    }
    assert!(rescued);
    assert_eq!(c.downed_pilot(d.id).unwrap().state, DownedState::Rescued);
    assert_eq!((c.blue.materiel, c.blue.fighters), (materiel, fighters));
    // Over friendly territory: no rescue needed.
    let mut c = campaign("central_europe", 3);
    eject_over(&mut c, "Ramstein");
    assert!(c.downed.is_empty());
}

#[test]
fn a_pilot_nobody_rescues_is_captured() {
    let mut c = campaign("central_europe", 3);
    c.income_at = f64::MAX;
    c.resupply_at = f64::MAX;
    let (materiel, fighters) = (c.blue.materiel, c.blue.fighters);
    eject_over(&mut c, "Tabor");
    // No helicopter left for the campaign's own try.
    c.blue.helicopters = 0;
    let ev = c.tick(tuning::RESCUE_WINDOW, &WorldView::default(), &[]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.contains("captured"))));
    assert_eq!(c.downed[0].state, DownedState::Captured);
    // What he cost when he came down stays lost (and no more).
    assert!(c.blue.materiel <= materiel - tuning::CAPTURE_COST + 0.01);
    assert!(c.blue.materiel >= materiel - tuning::CAPTURE_COST - 0.01);
    assert_eq!(c.blue.fighters, fighters - 1);
    // The players still fly.
    let offer = c.board.iter().find(|o| !o.kind.ai_flown()).unwrap().id;
    c.apply(P0, Command::TakeMission { offer, base: None, aircraft: Aircraft::F16 }).unwrap();
}

/// Player 0's parachute comes down near the enemy's place `name` and the simulation sends its
/// rescue helicopter.
fn down_over(c: &mut CampaignState, name: &str) -> (i32, i32) {
    c.income_at = f64::MAX;
    c.resupply_at = f64::MAX;
    c.tick(1.0, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| !o.kind.ai_flown()).unwrap().id;
    c.apply(P0, Command::TakeMission { offer, base: None, aircraft: Aircraft::F16 }).unwrap();
    let p = &c.places[place_named(c, name) as usize];
    let (x, y) = (p.x + 30, p.y);
    c.tick(1.0, &WorldView::default(), &[Report::PilotDown { player: P0, x, y }]);
    (x, y)
}

#[test]
fn an_extracted_pilot_gives_back_what_he_cost() {
    let mut c = campaign("central_europe", 3);
    let (materiel, fighters) = (c.blue.materiel, c.blue.fighters);
    down_over(&mut c, "Tabor");
    assert_eq!(c.blue.fighters, fighters - 1);
    assert!(c.blue.materiel < materiel);
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Rescued }).unwrap();
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.contains("extracted"))));
    assert_eq!((c.blue.materiel, c.blue.fighters), (materiel, fighters));
    assert!(c.downed.is_empty());
    // Captured waiting for the helicopter: lost.
    let mut c = campaign("central_europe", 3);
    let (materiel, fighters) = (c.blue.materiel, c.blue.fighters);
    down_over(&mut c, "Tabor");
    c.apply(P0, Command::EndSortie { end: SortieEnd::Captured }).unwrap();
    assert_eq!(c.blue.fighters, fighters - 1);
    assert!(c.blue.materiel <= materiel - tuning::CAPTURE_COST + 0.01);
    assert!(c.downed.is_empty());
}

#[test]
fn skipping_the_extraction_leaves_the_rescue_to_the_board() {
    let mut c = campaign("central_europe", 3);
    let (materiel, fighters) = (c.blue.materiel, c.blue.fighters);
    let (x, y) = down_over(&mut c, "Tabor");
    c.apply(P0, Command::EndSortie { end: SortieEnd::Ejected }).unwrap();
    let d = c.downed[0].clone();
    assert_eq!((d.x, d.y, d.state), (x, y, DownedState::Waiting));
    // Charged once, given back by the board's rescue.
    assert_eq!(c.blue.fighters, fighters - 1);
    let mut ev = Vec::new();
    c.rescued(d.id, &mut ev);
    assert_eq!((c.blue.materiel, c.blue.fighters), (materiel, fighters));
    // Ending the sortie from the menu (or leaving the game) skips it too.
    let mut c = campaign("central_europe", 3);
    down_over(&mut c, "Tabor");
    c.apply(P0, Command::EndSortie { end: SortieEnd::Aborted }).unwrap();
    assert_eq!(c.downed.len(), 1);
}

#[test]
fn aircraft_in_the_air_are_back_on_the_ground_after_a_load() {
    let mut c = campaign("central_europe", 3);
    let fighters = c.red.fighters;
    c.tick(tuning::PATROL_EVERY + 1.0, &WorldView::default(), &[]);
    assert_eq!(c.red.fighters, fighters - 1);
    let mut d: CampaignState = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    d.after_load();
    assert_eq!(d.red.fighters, fighters);
}

/// Player `p` takes the first offer of `kind` (its id and objective).
fn take(c: &mut CampaignState, p: PlayerId, kind: MissionKind) -> (u32, Objective) {
    let o = c.board.iter().find(|o| o.kind == kind).unwrap_or_else(|| panic!("no {kind:?}"));
    let (id, obj) = (o.id, o.objective.clone());
    c.apply(p, Command::TakeMission { offer: id, base: None, aircraft: Aircraft::F16 }).unwrap();
    (id, obj)
}

fn finished(ev: &[CampaignEvent]) -> SortieRecord {
    ev.iter()
        .find_map(|e| match e {
            CampaignEvent::SortieFinished { record, .. } => Some(record.clone()),
            _ => None,
        })
        .expect("finished")
}

#[test]
fn sorties_earn_points_and_pilots_keep_their_record() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, obj) = take(&mut c, P0, MissionKind::Recon);
    let Objective::Recon { object, .. } = obj else { unreachable!() };
    let kill = Report::FlightDestroyed { tag: 999, side: Side::Red, x: 0, y: 0, by: Some(P0) };
    c.tick(1.0, &WorldView::default(), &[Report::PhotoTaken { player: P0, object }, kill]);
    let r = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap());
    assert_eq!(
        r.points,
        MissionKind::Recon.points() + tuning::POINTS_AIR_KILL + tuning::POINTS_LANDED
    );
    assert_eq!(r.pilot, "Player");
    // The leaderboard; a newcomer taking the idle slot does not take the record.
    let alpha = c.join("Alpha", 2, |_| false).unwrap();
    assert_eq!(c.pilot_records()[0].points, r.points);
    let charlie = c.join("Charlie", 2, |id| id != P0).unwrap();
    assert_eq!(charlie, P0);
    assert_eq!(c.records(charlie).count(), 0);
    let board = c.pilot_records();
    assert_eq!(board[0].name, "Player");
    assert_eq!(board[0].points, r.points);
    assert!(board.iter().any(|p| p.name == "Charlie" && p.points == 0));
    assert!(board.iter().any(|p| p.name == "Alpha"));
    let _ = alpha;
}

#[test]
fn a_pending_sortie_earns_its_points_when_settled() {
    let mut c = campaign("central_europe", 3);
    let place = c.front(Side::Blue)[0];
    c.apply(P0, Command::OrderInsertion { place }).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, obj) = take(&mut c, P0, MissionKind::Escort);
    let Objective::Escort { insertion } = obj else { unreachable!() };
    let from = c.insertion(insertion).unwrap().from;
    be_there(&mut c, P0, from);
    let r = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap());
    assert_eq!(r.points, 0);
    let ev = c.tick(tuning::INSERTION_LEAD, &WorldView::default(), &[]);
    let (tags, target) = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, target, .. } => Some((tags.clone(), *target)),
            _ => None,
        })
        .unwrap();
    let t = c.site(target).unwrap().clone();
    c.tick(1.0, &WorldView::default(), &[Report::FlightLanded { tag: tags[0], x: t.x, y: t.y }]);
    assert_eq!(c.history.last().unwrap().points, MissionKind::Escort.points());
}

#[test]
fn objectives_flown_together_are_a_package() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let b = PlayerId(1);
    c.join("Bravo", 4, |_| false);
    // SEAD and strike on the same place.
    let sead = c
        .board
        .iter()
        .find(|o| {
            o.kind == MissionKind::Sead
                && c.board.iter().any(|s| {
                    s.kind == MissionKind::Strike
                        && c.objective_place(&s.objective) == c.objective_place(&o.objective)
                })
        })
        .expect("SEAD and strike on one place")
        .clone();
    let place = c.objective_place(&sead.objective).unwrap();
    let strike = c
        .board
        .iter()
        .find(|o| o.kind == MissionKind::Strike && c.objective_place(&o.objective) == Some(place))
        .unwrap()
        .clone();
    for (p, o) in [(P0, &sead), (b, &strike)] {
        c.apply(p, Command::TakeMission { offer: o.id, base: None, aircraft: Aircraft::F16 })
            .unwrap();
    }
    let Objective::Sead { sites, .. } = &sead.objective else { unreachable!() };
    let down: Vec<Report> =
        sites.iter().map(|&object| Report::ObjectDestroyed { object, by: Some(P0) }).collect();
    c.tick(1.0, &WorldView::default(), &down);
    // The SEAD pilot lands before the strike: the bonus reaches him all the same.
    let first = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap());
    assert_eq!(first.bonus, 0);
    c.tick(60.0, &WorldView::default(), &[]);
    let Objective::Strike { airfield, .. } = strike.objective else { unreachable!() };
    let ev = c.tick(
        1.0,
        &WorldView::default(),
        &[Report::ObjectDestroyed { object: airfield, by: Some(b) }],
    );
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.starts_with("Package"))));
    let sead_record = c.history.iter().find(|r| r.id == first.id).unwrap();
    assert_eq!(sead_record.bonus, tuning::PACKAGE_BONUS);
    assert_eq!(sead_record.points, first.points + tuning::PACKAGE_BONUS);
    let second = finished(&c.apply(b, Command::EndSortie { end: SortieEnd::Landed }).unwrap());
    assert_eq!(second.bonus, tuning::PACKAGE_BONUS);
}

#[test]
fn ai_sead_goes_for_the_sam_sites_on_request() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let o = c.board.iter().find(|o| o.kind == MissionKind::Sead).unwrap().clone();
    let place = c.objective_place(&o.objective).unwrap();
    let (materiel, fighters) = (c.blue.materiel, c.blue.fighters);
    c.apply(P0, Command::RequestSupport { offer: o.id }).expect("support");
    assert!(c.apply(P0, Command::RequestSupport { offer: o.id }).is_err(), "twice");
    assert_eq!(c.blue.materiel, materiel - tuning::SUPPORT_COST);
    c.income_at = f64::MAX;
    let ev = c.tick(tuning::SUPPORT_LEAD + 1.0, &WorldView::default(), &[]);
    let (id, target) = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::RaidLaunched { id, side: Side::Blue, target, drones: false, .. } => {
                Some((*id, *target))
            }
            _ => None,
        })
        .expect("AI SEAD launched");
    assert!(c.places[place as usize].defences.contains(&target));
    assert!(c.blue.fighters <= fighters - tuning::RAID_AIRCRAFT as u32);
    // No defence offers for the friendly side's own raid.
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    assert!(c.board.iter().all(|o| o.objective != Objective::Defend { raid: id }));
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: target, by: None }]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::Hit);
    assert!(!c.site(target).unwrap().alive());
}

#[test]
fn the_enemy_awacs_is_worth_shooting_down() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let ev = c.tick(tuning::AWACS_EVERY, &WorldView::default(), &[]);
    let tag = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::LaunchAwacs { tag, side: Side::Red, from, .. } => {
                assert_eq!(c.owner(*from), Some(Side::Red));
                Some(*tag)
            }
            _ => None,
        })
        .expect("an AWACS");
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, obj) = take(&mut c, P0, MissionKind::Awacs);
    assert_eq!(obj, Objective::Awacs { tag });
    let red = c.red.materiel;
    let ev = c.tick(
        1.0,
        &WorldView::default(),
        &[Report::FlightDestroyed { tag, side: Side::Red, x: 0, y: 0, by: Some(P0) }],
    );
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
    assert!(c.awacs.is_none());
    assert!(c.red.materiel < red);
    // Not shot down: called home after its time.
    let mut c = campaign("central_europe", 3);
    c.tick(tuning::AWACS_EVERY + 1.0, &WorldView::default(), &[]);
    let tag = c.awacs.as_ref().unwrap().tag;
    let ev = c.tick(tuning::AWACS_ENDURANCE, &WorldView::default(), &[]);
    assert!(ev.contains(&CampaignEvent::Recall { tag }));
    assert!(c.awacs.is_none());
}

#[test]
fn drone_waves_hit_ukraine() {
    let mut c = campaign("ukraine", 9);
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(tuning::DRONES_EVERY[1], &WorldView::default(), &[]);
    let r = c.raids.iter().find(|r| r.drones).expect("a drone wave").clone();
    assert_eq!(c.owner(r.target), Some(Side::Blue));
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let o = c.board.iter().find(|o| o.objective == Objective::Defend { raid: r.id }).unwrap();
    assert!(o.title.starts_with("Drone defense"), "{}", o.title);
    let fighters = c.red.fighters;
    let ev = c.tick(tuning::DRONE_LEAD, &WorldView::default(), &[]);
    let tags = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::RaidLaunched { tags, drones: true, .. } => Some(tags.clone()),
            _ => None,
        })
        .expect("launched");
    assert_eq!(tags.len(), tuning::DRONE_WAVE);
    // Drones are no fighters: only the fighters launched meanwhile are gone.
    let launched = ev
        .iter()
        .filter(|e| matches!(e, CampaignEvent::LaunchFighters { side: Side::Red, .. }))
        .count() as u32;
    let strike: u32 = ev
        .iter()
        .filter_map(|e| match e {
            CampaignEvent::RaidLaunched { side: Side::Red, drones: false, tags, .. } => {
                Some(tags.len() as u32)
            }
            _ => None,
        })
        .sum();
    assert_eq!(c.red.fighters + launched + strike, fighters);
    // Not in other theaters.
    let mut c = campaign("central_europe", 3);
    c.tick(tuning::DRONES_EVERY[1] + 1.0, &WorldView::default(), &[]);
    assert!(c.raids.iter().all(|r| !r.drones));
}

/// Taking an escort and aborting on the runway earns nothing, however the insertion goes.
#[test]
fn a_sortie_nobody_flew_is_not_settled() {
    let mut c = campaign("central_europe", 3);
    let place = c.front(Side::Blue)[0];
    c.apply(P0, Command::OrderInsertion { place }).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    take(&mut c, P0, MissionKind::Escort);
    let r = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Aborted }).unwrap());
    assert_eq!(r.result, SortieResult::Failed);
    let ev = c.tick(tuning::INSERTION_LEAD, &WorldView::default(), &[]);
    let (tags, target) = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, target, .. } => Some((tags.clone(), *target)),
            _ => None,
        })
        .unwrap();
    let t = c.site(target).unwrap().clone();
    c.tick(1.0, &WorldView::default(), &[Report::FlightLanded { tag: tags[0], x: t.x, y: t.y }]);
    let r = c.history.last().unwrap();
    assert_eq!((r.result, r.points), (SortieResult::Failed, 0));
}

/// A destroyed target leaves the board at once: nobody takes a mission that is already done.
#[test]
fn a_destroyed_target_leaves_the_board_at_once() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let o = c.board.iter().find(|o| o.kind == MissionKind::Strike).unwrap().clone();
    let Objective::Strike { airfield, .. } = o.objective else { unreachable!() };
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: airfield, by: None }]);
    assert!(c.board.iter().all(|b| b.id != o.id));
    let take = Command::TakeMission { offer: o.id, base: None, aircraft: Aircraft::F117 };
    assert!(c.apply(P0, take).is_err());
}

/// A friendly loss over the CAP point starts its clock again; kills there still count.
#[test]
fn a_cap_survives_a_friendly_loss() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, obj) = take(&mut c, P0, MissionKind::Cap);
    let Objective::Cap { x, y } = obj else { unreachable!() };
    let me = WorldView {
        players: vec![PlayerPos { id: P0, x, y, airborne: true, landed: false, hover_over: None }],
        ..Default::default()
    };
    c.tick(300.0, &me, &[]);
    let lost = Report::FlightDestroyed { tag: 4242, side: Side::Blue, x, y, by: None };
    c.tick(1.0, &me, &[lost]);
    assert!(c.sortie(P0).unwrap().progress.on_station < 2.0, "the clock starts again");
    let kills: Vec<Report> = (1..=2)
        .map(|k| Report::FlightDestroyed { tag: 5000 + k, side: Side::Red, x, y, by: Some(P0) })
        .collect();
    let ev = c.tick(1.0, &me, &kills);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })));
}

/// The players order insertions on the front only (what the board offers).
#[test]
fn insertions_go_where_the_board_offers_them() {
    let mut c = campaign("central_europe", 3);
    let front = c.front(Side::Blue);
    let far = (0..c.places.len() as PlaceId)
        .find(|&p| c.places[p as usize].owner == Side::Red && !front.contains(&p))
        .unwrap();
    assert!(c.apply(P0, Command::OrderInsertion { place: far }).is_err());
    assert!(c.apply(P0, Command::OrderInsertion { place: front[0] }).is_ok());
}

/// Drones in the air when the campaign is saved are spent, not fighters back home.
#[test]
fn drones_in_the_air_are_no_fighters_after_a_load() {
    let mut c = campaign("central_europe", 3);
    let before = c.red.fighters;
    let target = c.places[0].members[0];
    c.raids.push(Raid {
        id: 9999,
        side: Side::Red,
        place: 0,
        target,
        from: 0,
        aircraft: (1..=6).map(|k| 90000 + k).collect(),
        alive: vec![true; 6],
        state: RaidState::Enroute,
        ended: None,
        escorted: true,
        drones: true,
        enroute_at: Some(0.0),
    });
    c.after_load();
    assert_eq!(c.red.fighters, before);
}

/// An insertion still in the air after an hour is called home; helicopters landing short of
/// the place take nothing.
#[test]
fn insertions_in_the_air_do_not_go_on_for_ever() {
    let mut c = campaign("central_europe", 3);
    let place = c.front(Side::Blue)[0];
    c.apply(P0, Command::OrderInsertion { place }).unwrap();
    let ev = c.tick(tuning::INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    let tags = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, side: Side::Blue, .. } => Some(tags.clone()),
            _ => None,
        })
        .unwrap();
    let ev = c.tick(tuning::INSERTION_TIMEOUT + 1.0, &WorldView::default(), &[]);
    assert_eq!(c.insertions[0].state, InsertionState::Failed);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Recall { tag } if *tag == tags[0])));
    // The next one (the campaign may have ordered it meanwhile) lands short.
    let _ = c.apply(P0, Command::OrderInsertion { place });
    let k = c.insertions.iter().rposition(|i| i.place == place).unwrap();
    assert!(matches!(c.insertions[k].state, InsertionState::Pending { .. }));
    let ev = c.tick(tuning::INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    let tags = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, side: Side::Blue, .. } => Some(tags.clone()),
            _ => None,
        })
        .unwrap();
    let short: Vec<Report> =
        tags.iter().map(|&tag| Report::FlightLanded { tag, x: 0, y: 0 }).collect();
    c.tick(1.0, &WorldView::default(), &short);
    assert_eq!(c.insertions[k].state, InsertionState::Failed);
    assert_eq!(c.places[place as usize].owner, Side::Red);
}

/// A player's insertion flies on the reserve, but the reserve does not swell the pool.
#[test]
fn the_reserve_helicopters_do_not_swell_the_pool() {
    let mut c = campaign("central_europe", 3);
    let most = c.aircraft_limits(Side::Blue).1;
    c.blue.helicopters = most;
    let place = c.front(Side::Blue)[0];
    c.apply(P0, Command::OrderInsertion { place }).unwrap();
    let ev = c.tick(tuning::INSERTION_LEAD + 1.0, &WorldView::default(), &[]);
    let tags = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { tags, .. } => Some(tags.clone()),
            _ => None,
        })
        .unwrap();
    // Topped up meanwhile; every helicopter comes home.
    c.blue.helicopters = most;
    let home: Vec<Report> =
        tags.iter().map(|&tag| Report::FlightLanded { tag, x: 0, y: 0 }).collect();
    c.tick(1.0, &WorldView::default(), &home);
    assert_eq!(c.blue.helicopters, most);
}

/// Player 0 at (x, y): in the air or landed in his helicopter.
fn heli_at(x: i32, y: i32, landed: bool) -> WorldView {
    WorldView {
        players: vec![PlayerPos { id: P0, x, y, airborne: !landed, landed, hover_over: None }],
        ..Default::default()
    }
}

/// Player 0 takes the board's insertion in the UH-60 and lifts off with his wingmen: the
/// insertion, its target and the wingmen's tags.
fn fly_insertion(c: &mut CampaignState) -> (u32, ObjectId, Vec<u32>) {
    c.tick(1.0, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Insertion).unwrap().id;
    c.apply(P0, Command::TakeMission { offer, base: None, aircraft: Aircraft::UH60 }).unwrap();
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    let (id, target, tags) = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::InsertionLaunched { id, target, tags, .. } => {
                Some((*id, *target, tags.clone()))
            }
            _ => None,
        })
        .expect("launched");
    assert_eq!(tags.len(), tuning::INSERTION_HELOS - 1, "the wingmen");
    assert_eq!(c.insertion(id).unwrap().pilots, vec![P0]);
    (id, target, tags)
}

#[test]
fn the_helicopter_flies_insertions_and_rescues() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let insertion = c.board.iter().find(|o| o.kind == MissionKind::Insertion).unwrap().id;
    let recon = c.board.iter().find(|o| o.kind == MissionKind::Recon).unwrap().id;
    let take = |c: &mut CampaignState, offer, aircraft| {
        c.apply(P0, Command::TakeMission { offer, base: None, aircraft })
    };
    assert!(take(&mut c, recon, Aircraft::UH60).is_err());
    assert!(take(&mut c, insertion, Aircraft::F16).is_err());
    assert!(
        MissionKind::Csar.flyable_by(Aircraft::UH60) && MissionKind::Csar.flyable_by(Aircraft::F16)
    );
    take(&mut c, insertion, Aircraft::UH60).unwrap();
}

#[test]
fn the_briefing_picks_an_aircraft_made_for_the_mission() {
    use MissionKind::*;
    let kinds = [
        Recon,
        Sead,
        Cap,
        Cas,
        AntiShip,
        Strike,
        Escort,
        Insertion,
        Intercept,
        Defend,
        Interdiction,
        Csar,
        Awacs,
    ];
    let all = [
        Aircraft::F117,
        Aircraft::F16,
        Aircraft::UH60,
        Aircraft::AH1Z,
        Aircraft::Tornado,
        Aircraft::A10,
        Aircraft::RAH66,
        Aircraft::AV8B,
    ];
    for k in kinds {
        for last in all {
            assert!(k.flyable_by(k.pick(last)), "{k:?} after {last:?}");
        }
    }
    // No jet on a rescue, no helicopter deep in enemy territory.
    assert_eq!(Csar.pick(Aircraft::F117), Aircraft::UH60);
    assert_eq!(Interdiction.pick(Aircraft::UH60), Aircraft::F117);
    assert_eq!(Interdiction.pick(Aircraft::AH1Z), Aircraft::F117);
    assert_eq!(Strike.pick(Aircraft::F16), Aircraft::F16);
    assert_eq!(Cas.pick(Aircraft::AH1Z), Aircraft::AH1Z);
    assert_eq!(Interdiction.pick(Aircraft::Tornado), Aircraft::Tornado);
    assert_eq!(Cas.pick(Aircraft::A10), Aircraft::A10);
    assert_eq!(Cas.pick(Aircraft::UH60), Aircraft::A10);
    assert_eq!(Escort.pick(Aircraft::RAH66), Aircraft::RAH66);
    assert_eq!(Cas.pick(Aircraft::AV8B), Aircraft::AV8B);
    assert_eq!(Sead.pick(Aircraft::AV8B), Aircraft::F16);
    assert_eq!(Interdiction.pick(Aircraft::RAH66), Aircraft::F117);
}

#[test]
fn a_player_insertion_takes_the_place_by_landing() {
    let mut c = campaign("central_europe", 3);
    let (id, target, _) = fly_insertion(&mut c);
    let place = c.insertion(id).unwrap().place;
    let t = c.site(target).unwrap().clone();
    // Hovering over it takes nothing.
    for _ in 0..4 {
        c.tick(10.0, &heli_at(t.x + 0x20, t.y, false), &[]);
    }
    assert_eq!(c.places[place as usize].owner, Side::Red);
    // Landed within the capture radius: the troops are out after the unload time.
    c.tick(10.0, &heli_at(t.x + 0x20, t.y, true), &[]);
    assert_eq!(c.places[place as usize].owner, Side::Red);
    let ev = c.tick(10.0, &heli_at(t.x + 0x20, t.y, true), &[]);
    assert_eq!(c.places[place as usize].owner, Side::Blue);
    assert_eq!(c.insertion(id).unwrap().state, InsertionState::Captured);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { player: P0, .. })));
}

#[test]
fn a_player_insertion_goes_on_without_the_wingmen_and_fails_with_him() {
    let mut c = campaign("central_europe", 3);
    let (id, _, tags) = fly_insertion(&mut c);
    let reports: Vec<Report> = tags
        .iter()
        .map(|&tag| Report::FlightDestroyed { tag, side: Side::Blue, x: 0, y: 0, by: None })
        .collect();
    c.tick(1.0, &WorldView::default(), &reports);
    assert_eq!(c.insertion(id).unwrap().state, InsertionState::Enroute);
    c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap();
    assert_eq!(c.insertion(id).unwrap().state, InsertionState::Failed);
}

#[test]
fn a_helicopter_rescue_lands_by_the_pilot_and_brings_him_home() {
    for lost in [false, true] {
        let mut c = campaign("central_europe", 3);
        let (_, x, y) = eject_over(&mut c, "Tabor");
        let d = c.downed[0].id;
        c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
        let offer = c.board.iter().find(|o| o.kind == MissionKind::Csar).unwrap().clone();
        c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::UH60 })
            .unwrap();
        // Hovering over him (long enough to cover a jet's rescue): he waits.
        for _ in 0..20 {
            c.tick(10.0, &heli_at(x + 5, y, false), &[]);
        }
        assert_eq!(c.downed_pilot(d).unwrap().state, DownedState::Waiting);
        // Landed by him: he boards.
        for _ in 0..4 {
            c.tick(10.0, &heli_at(x + 5, y, true), &[]);
        }
        assert_eq!(c.downed_pilot(d).unwrap().state, DownedState::Aboard);
        // Long past the rescue window: he stays aboard.
        c.tick(tuning::RESCUE_WINDOW, &heli_at(x + 0x400, y, false), &[]);
        assert_eq!(c.downed_pilot(d).unwrap().state, DownedState::Aboard);
        if lost {
            c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap();
            assert_eq!(c.downed_pilot(d).unwrap().state, DownedState::Captured);
            continue;
        }
        // Landed at home.
        let home = c.site(offer.base).unwrap().clone();
        let ev = c.tick(1.0, &heli_at(home.x, home.y, true), &[]);
        assert!(
            ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { player: P0, .. }))
        );
        assert_eq!(c.downed_pilot(d).unwrap().state, DownedState::Rescued);
    }
}

/// A campaign that knows the theater's land and water (the host's).
fn campaign_on_terrain(code: &str, index: usize) -> CampaignState {
    let mut c = campaign(code, index);
    c.terrain = Terrain::new(Some(std::sync::Arc::new(theater(code))), relief(code));
    c
}

/// The battery the enemy pushes forward on the second tick (the first builds the board).
fn deployed(c: &mut CampaignState) -> Option<(u32, i32, i32)> {
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(1.0, &WorldView::default(), &[]).iter().find_map(|e| match *e {
        CampaignEvent::BatteryDeployed { id, x, y } => Some((id, x, y)),
        _ => None,
    })
}

/// Where nothing is a short helicopter flight from a player base, the enemy pushes a battery
/// forward: on land, 15 to 35 km from a base. Cuba has none: the sea is all there is that near
/// Key West. Nor has the Middle East: southern Lebanon and the Golan are in reach of Israel's
/// bases already.
#[test]
fn forward_batteries_go_on_land_near_a_base() {
    for (code, index) in [
        ("libya", 0),
        ("persian_gulf", 1),
        ("north_cape", 2),
        ("central_europe", 3),
        ("middle_east", 5),
        ("desert_storm", 6),
        ("cuba", 7),
        ("korea", 8),
        ("ukraine", 9),
    ] {
        let mut c = campaign_on_terrain(code, index);
        let battery = deployed(&mut c);
        if code == "cuba" || code == "middle_east" {
            assert_eq!(battery, None, "{code}");
            continue;
        }
        let Some((_, x, y)) = battery else { panic!("{code}: no battery") };
        let t = c.terrain.0.clone().unwrap();
        assert!(!t.is_water(x, y), "{code}: battery in the water at {x}, {y}");
        let near = c
            .player_bases()
            .iter()
            .filter_map(|&b| c.site(b))
            .map(|s| dist(s.x - x, s.y - y))
            .min()
            .unwrap();
        assert!(
            (tuning::BATTERY_NEAR..=tuning::BATTERY_FAR + 2).contains(&near),
            "{code}: {near} from the nearest base"
        );
        eprintln!("{code}: battery near {} {} km from a base", c.batteries[0].name, near / 64);
    }
}

/// A target already in reach of a base: no battery.
#[test]
fn no_battery_with_a_target_in_reach() {
    // Vientiane is 32 km from Udorn.
    let mut c = campaign_on_terrain("vietnam", 4);
    assert_eq!(deployed(&mut c), None);
    assert!(c.batteries.is_empty());
}

/// Without the theater's terrain (a coop client's state) the campaign puts none anywhere.
#[test]
fn no_battery_without_terrain() {
    let mut c = campaign("cuba", 7);
    assert_eq!(deployed(&mut c), None);
}

/// The battery's guns stand: it is on the board, flown like CAS; once they are all down it is
/// gone.
#[test]
fn a_forward_battery_is_offered_until_its_guns_are_down() {
    let mut c = campaign_on_terrain("desert_storm", 6);
    let (id, x, y) = deployed(&mut c).expect("battery");
    let guns = |alive: bool| WorldView {
        units: (0..4).map(|k| GroundView::gun(x + 10 * k, y, Side::Red, alive)).collect(),
        ..Default::default()
    };
    c.tick(1.0, &guns(true), &[]);
    let o = c
        .board
        .iter()
        .find(|o| matches!(o.objective, Objective::Battery { battery, .. } if battery == id))
        .expect("battery offer")
        .clone();
    assert_eq!(o.kind, MissionKind::Cas);
    assert!(o.title.starts_with("CAS - battery near"), "{}", o.title);
    assert!(matches!(&o.objective, Objective::Battery { aaa, .. } if aaa.len() == 4));
    let ev = c.tick(1.0, &guns(false), &[]);
    assert!(c.batteries.is_empty());
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.contains("destroyed"))));
    assert!(!c.board.iter().any(|o| matches!(o.objective, Objective::Battery { .. })));
}

/// A friendly Patriot radar (AAA unit 0) 4 km east of Ramstein: the campaign, its view, where
/// it stands.
fn battery_view() -> (CampaignState, WorldView, (i32, i32)) {
    let mut c = campaign("central_europe", 3);
    let p = &c.places[place_named(&c, "Ramstein") as usize];
    let (x, y) = (p.x + 0x100, p.y);
    let view =
        WorldView { units: vec![GroundView::gun(x, y, Side::Blue, true)], ..Default::default() };
    c.tick(1.0, &view, &[]);
    (c, view, (x, y))
}

fn request(c: &mut CampaignState, (x, y): (i32, i32)) -> Vec<CampaignEvent> {
    c.apply(P0, Command::RequestResupply { aaa: 0, x, y, kind: DefenceKind::Patriot }).unwrap()
}

#[test]
fn nobody_takes_a_resupply_and_the_ai_flies_it() {
    let (mut c, view, at) = battery_view();
    request(&mut c, at);
    assert!(
        c.apply(
            P0,
            Command::RequestResupply { aaa: 0, x: at.0, y: at.1, kind: DefenceKind::Patriot }
        )
        .is_err()
    );
    c.tick(1.0, &view, &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Resupply).unwrap().clone();
    assert_eq!((offer.x, offer.y), at);
    // From the nearest airfield, in the UH-60 only.
    assert_eq!(c.site(offer.base).unwrap().kind, SiteKind::Airfield);
    assert!(MissionKind::Resupply.flyable_by(Aircraft::UH60));
    assert!(!MissionKind::Resupply.flyable_by(Aircraft::F16));
    assert!(!MissionKind::Resupply.flyable_by(Aircraft::AH1Z));
    // Not yet: the players have five minutes.
    let ev = c.tick(tuning::SUPPLY_AI_WAIT - 10.0, &view, &[]);
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::SupplyLaunched { .. })));
    let ev = c.tick(11.0, &view, &[]);
    let Some(&CampaignEvent::SupplyLaunched { from, aaa, tag, .. }) =
        ev.iter().find(|e| matches!(e, CampaignEvent::SupplyLaunched { .. }))
    else {
        panic!("no AI helicopter: {ev:?}")
    };
    assert_eq!((from, aaa), (offer.base, 0));
    // Off the board at once.
    assert!(!c.board.iter().any(|o| o.kind == MissionKind::Resupply));
    // Shot down: back on the board.
    let lost = Report::FlightDestroyed { tag, side: Side::Blue, x: at.0, y: at.1, by: None };
    c.tick(1.0, &view, &[lost]);
    assert!(c.board.iter().any(|o| o.kind == MissionKind::Resupply));
    let ev = c.tick(tuning::SUPPLY_AI_WAIT + 1.0, &view, &[]);
    let Some(&CampaignEvent::SupplyLaunched { tag, .. }) =
        ev.iter().find(|e| matches!(e, CampaignEvent::SupplyLaunched { .. }))
    else {
        panic!("no second helicopter")
    };
    let ev = c.tick(1.0, &view, &[Report::SupplyDelivered { tag }]);
    assert!(ev.contains(&CampaignEvent::Restocked { aaa: 0 }));
    assert!(c.supply_requests.is_empty());
    // A new request is possible again.
    request(&mut c, at);
}

#[test]
fn a_player_delivers_a_resupply_by_hovering_at_the_unit() {
    let (mut c, view, at) = battery_view();
    request(&mut c, at);
    c.tick(1.0, &view, &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Resupply).unwrap().id;
    let take = |a| Command::TakeMission { offer, base: None, aircraft: a };
    assert!(c.apply(P0, take(Aircraft::F16)).is_err());
    c.apply(P0, take(Aircraft::UH60)).unwrap();
    // While he flies it, no AI helicopter.
    let flying = WorldView {
        players: vec![PlayerPos {
            id: P0,
            x: at.0 + 0x200,
            y: at.1,
            airborne: true,
            landed: false,
            hover_over: None,
        }],
        ..view.clone()
    };
    for _ in 0..40 {
        let ev = c.tick(10.0, &flying, &[]);
        assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::SupplyLaunched { .. })));
    }
    let hover = WorldView {
        players: vec![PlayerPos {
            id: P0,
            x: at.0,
            y: at.1,
            airborne: true,
            landed: false,
            hover_over: Some(0),
        }],
        ..view.clone()
    };
    let mut ev = Vec::new();
    for _ in 0..(tuning::SUPPLY_HOVER_SECONDS as i32 + 1) {
        ev.extend(c.tick(1.0, &hover, &[]));
    }
    assert!(ev.contains(&CampaignEvent::Restocked { aaa: 0 }), "{ev:?}");
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { player: P0, .. })));
    assert!(c.supply_requests.is_empty());
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap();
    let Some(CampaignEvent::SortieFinished { record, .. }) =
        ev.iter().find(|e| matches!(e, CampaignEvent::SortieFinished { .. }))
    else {
        panic!()
    };
    assert_eq!(record.result, SortieResult::Success);
}

#[test]
fn a_resupply_for_a_lost_unit_is_called_off() {
    let (mut c, mut view, at) = battery_view();
    request(&mut c, at);
    c.tick(1.0, &view, &[]);
    assert!(c.board.iter().any(|o| o.kind == MissionKind::Resupply));
    view.units[0].alive = false;
    c.tick(1.0, &view, &[]);
    c.tick(tuning::BOARD_REFRESH, &view, &[]);
    assert!(c.supply_requests.is_empty());
    assert!(!c.board.iter().any(|o| o.kind == MissionKind::Resupply));
}

#[test]
fn a_unit_out_of_stock_orders_its_resupply_once() {
    let (mut c, view, at) = battery_view();
    let ev = c.order_resupply(0, at.0, at.1, DefenceKind::Patriot);
    assert!(
        ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.contains("out of missiles")))
    );
    assert_eq!(c.supply_requests[0].by, None);
    assert!(c.order_resupply(0, at.0, at.1, DefenceKind::Patriot).is_empty());
    // A player's request for it is the same one.
    assert!(
        c.apply(
            P0,
            Command::RequestResupply { aaa: 0, x: at.0, y: at.1, kind: DefenceKind::Patriot }
        )
        .is_err()
    );
    c.tick(1.0, &view, &[]);
    assert_eq!(c.board.iter().filter(|o| o.kind == MissionKind::Resupply).count(), 1);
}

/// The nearest spot to Ramstein where a new site may go.
fn deploy_spot(c: &CampaignState, kind: DefenceKind, aaa: &[GroundView]) -> (i32, i32) {
    let p = &c.places[place_named(c, "Ramstein") as usize];
    (1..40)
        .flat_map(|r| (0..16).map(move |k| (r, k)))
        .map(|(r, k)| {
            let t = k as f64 / 16.0 * std::f64::consts::TAU;
            (p.x + (t.cos() * r as f64 * 32.0) as i32, p.y + (t.sin() * r as f64 * 32.0) as i32)
        })
        .find(|&(x, y)| c.deploy_refusal(x, y, kind, aaa).is_none())
        .expect("a free spot")
}

fn designate(c: &mut CampaignState, (x, y): (i32, i32), kind: DefenceKind) -> Result<u32, String> {
    c.apply(P0, Command::DesignateSite { x, y, kind }).map_err(|t| t.en())?;
    Ok(c.deployments.last().unwrap().id)
}

fn landed_at(x: i32, y: i32) -> WorldView {
    WorldView {
        players: vec![PlayerPos { id: P0, x, y, airborne: false, landed: true, hover_over: None }],
        ..Default::default()
    }
}

fn flying_at(x: i32, y: i32) -> WorldView {
    WorldView {
        players: vec![PlayerPos { id: P0, x, y, airborne: true, landed: false, hover_over: None }],
        ..Default::default()
    }
}

#[test]
fn new_sites_keep_clear_of_places_units_and_the_enemy() {
    let c = campaign("central_europe", 3);
    let ramstein = &c.places[place_named(&c, "Ramstein") as usize];
    let why = c.deploy_refusal(ramstein.x, ramstein.y, DefenceKind::Gepard, &[]).unwrap();
    assert!(why.contains("Too close"), "{why}");
    let enemy = c.places.iter().find(|p| p.owner == Side::Red).unwrap();
    let off = (enemy.x + 0x100, enemy.y + 0x100);
    let why = c.deploy_refusal(off.0, off.1, DefenceKind::Gepard, &[]).unwrap();
    assert!(why.contains("friendly territory") || why.contains("Too close"), "{why}");
    let at = deploy_spot(&c, DefenceKind::Patriot, &[]);
    let gun = [GroundView::gun(at.0 + 4, at.1, Side::Blue, true)];
    assert!(c.deploy_refusal(at.0, at.1, DefenceKind::Patriot, &gun).unwrap().contains("units"));
    // A wreck is no unit.
    let wreck = [GroundView { alive: false, ..gun[0] }];
    assert!(c.deploy_refusal(at.0, at.1, DefenceKind::Patriot, &wreck).is_none());
}

#[test]
fn only_three_sites_wait_for_cargo_at_once() {
    let mut c = campaign("central_europe", 3);
    for _ in 0..tuning::DEPLOY_OPEN_MAX {
        let at = deploy_spot(&c, DefenceKind::Gepard, &[]);
        designate(&mut c, at, DefenceKind::Gepard).unwrap();
    }
    let p = &c.places[place_named(&c, "Ramstein") as usize];
    let why = c.deploy_refusal(p.x + 0x400, p.y + 0x400, DefenceKind::Gepard, &[]).unwrap();
    assert!(why.contains("waiting"), "{why}");
}

#[test]
fn a_gepard_site_is_built_by_one_load_of_cargo() {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Gepard, &[]);
    let id = designate(&mut c, at, DefenceKind::Gepard).unwrap();
    // Not twice in one place.
    assert!(designate(&mut c, at, DefenceKind::Gepard).is_err());
    c.tick(1.0, &WorldView::default(), &[]);
    let offers: Vec<_> = c.board.iter().filter(|o| o.kind == MissionKind::Logistics).collect();
    assert_eq!(offers.len(), 1);
    let offer = offers[0].clone();
    assert_eq!((offer.x, offer.y), at);
    assert_eq!(c.site(offer.base).unwrap().kind, SiteKind::Airfield);
    let take = |a| Command::TakeMission { offer: offer.id, base: None, aircraft: a };
    assert!(c.apply(P0, take(Aircraft::F16)).is_err());
    c.apply(P0, take(Aircraft::UH60)).unwrap();
    // Landed at the pad without the cargo: nothing.
    let mut ev = Vec::new();
    for _ in 0..30 {
        ev.extend(c.tick(1.0, &landed_at(at.0, at.1), &[]));
    }
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::SiteBuilt { .. })));
    // Hooked at the base, set down by the pad.
    let base = c.site(offer.base).unwrap();
    let over_base = flying_at(base.x, base.y);
    let far = flying_at(base.x + 0x200, base.y);
    assert_eq!(c.hook_refusal(P0, id, 0, &far).unwrap(), "Too far from the cargo");
    assert_eq!(c.hook_refusal(P0, id, 0, &over_base), None);
    c.apply(P0, Command::HookCrate { id, leg: 0 }).unwrap();
    assert_eq!(c.deployment(id).unwrap().crate_state(0), CrateState::Carried(P0));
    c.tick(1.0, &over_base, &[]);
    assert!(c.hook_refusal(P0, id, 0, &over_base).is_some());
    let drop = |east| Command::DropCrate { id, leg: 0, east, north: 3.0, heading: 0.0 };
    assert!(c.apply(P0, drop(400.0)).is_err());
    ev.extend(c.apply(P0, drop(5.0)).unwrap());
    ev.extend(c.tick(1.0, &flying_at(at.0, at.1), &[]));
    let unit = (at.0, at.1 - tuning::DEPLOY_UNIT_OFFSET);
    assert!(
        ev.contains(&CampaignEvent::SiteBuilt {
            id,
            x: unit.0,
            y: unit.1,
            kind: DefenceKind::Gepard
        }),
        "{ev:?}"
    );
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { player: P0, .. })));
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    assert!(!c.board.iter().any(|o| o.kind == MissionKind::Logistics));
    // It stands, then it is destroyed: the site is gone.
    let mut gun = WorldView {
        units: vec![GroundView::gun(unit.0, unit.1, Side::Blue, true)],
        ..Default::default()
    };
    c.tick(1.0, &gun, &[]);
    assert!(c.deployment(id).is_some_and(|d| d.built && d.seen));
    gun.units[0].alive = false;
    c.tick(1.0, &gun, &[]);
    assert!(c.deployments.is_empty());
}

#[test]
fn a_drone_command_center_takes_one_load() {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Drones, &[]);
    let id = designate(&mut c, at, DefenceKind::Drones).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    let offers: Vec<_> = c.board.iter().filter(|o| o.kind == MissionKind::Logistics).collect();
    assert_eq!(offers.len(), 1);
    assert!(offers[0].title.starts_with("Drones 1/1"), "{}", offers[0].title);
    assert!(c.deployment(id).unwrap().title().starts_with("Drone command center near"));
}

#[test]
fn saves_from_before_the_drones_say_patriot() {
    let old = r#"{"x":1,"y":2,"patriot":true,"stock":5}"#;
    let s: DefenceStock = serde_json::from_str(old).unwrap();
    assert_eq!((s.kind, s.stock, s.stock2, s.stock3), (DefenceKind::Patriot, 5, 0, -1));
    let old = r#"{"x":1,"y":2,"patriot":false,"stock":5}"#;
    assert_eq!(serde_json::from_str::<DefenceStock>(old).unwrap().kind, DefenceKind::Gepard);
    let new =
        DefenceStock { x: 1, y: 2, kind: DefenceKind::Drones, stock: 3, stock2: 7, stock3: 2 };
    let back: DefenceStock = serde_json::from_str(&serde_json::to_string(&new).unwrap()).unwrap();
    assert_eq!(back, new);
}

#[test]
fn a_patriot_site_takes_three_loads_and_the_ai_brings_the_untaken_ones() {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Patriot, &[]);
    let id = designate(&mut c, at, DefenceKind::Patriot).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(c.board.iter().filter(|o| o.kind == MissionKind::Logistics).count(), 3);
    let mut built = false;
    for n in 1..=3 {
        let ev = c.tick(tuning::DEPLOY_AI_WAIT + 1.0, &WorldView::default(), &[]);
        let Some(&CampaignEvent::CargoLaunched { tag, x, y, .. }) =
            ev.iter().find(|e| matches!(e, CampaignEvent::CargoLaunched { .. }))
        else {
            panic!("no cargo helicopter: {ev:?}")
        };
        assert_eq!((x, y), at);
        assert_eq!(c.board.iter().filter(|o| o.kind == MissionKind::Logistics).count(), 3 - n);
        let ev = c.tick(1.0, &WorldView::default(), &[Report::SupplyDelivered { tag }]);
        assert_eq!(c.deployment(id).unwrap().delivered.len(), n);
        built = ev
            .iter()
            .any(|e| matches!(e, CampaignEvent::SiteBuilt { kind: DefenceKind::Patriot, .. }));
        assert_eq!(built, n == 3);
    }
    assert!(built);
}

#[test]
fn a_site_not_supplied_in_a_day_loses_its_helipad() {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Patriot, &[]);
    designate(&mut c, at, DefenceKind::Patriot).unwrap();
    // Lost cargo helicopters don't bring it: the loads stay open.
    let mut t = 0.0;
    while t < tuning::DEPLOY_TIMEOUT - 3600.0 {
        let ev = c.tick(600.0, &WorldView::default(), &[]);
        let lost: Vec<Report> = ev
            .iter()
            .filter_map(|e| match *e {
                CampaignEvent::CargoLaunched { tag, .. } => {
                    Some(Report::FlightDestroyed { tag, side: Side::Blue, x: 0, y: 0, by: None })
                }
                _ => None,
            })
            .collect();
        c.tick(1.0, &WorldView::default(), &lost);
        t += 601.0;
    }
    assert_eq!(c.deployments.len(), 1);
    let ev = c.tick(3600.0, &WorldView::default(), &[]);
    assert!(c.deployments.is_empty());
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(m) if m.contains("helipad"))));
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    assert!(!c.board.iter().any(|o| o.kind == MissionKind::Logistics));
}

#[test]
fn deployments_save_and_load() {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Gepard, &[]);
    designate(&mut c, at, DefenceKind::Gepard).unwrap();
    let json = serde_json::to_string(&c).unwrap();
    let back: CampaignState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.deployments, c.deployments);
}

/// A Patriot site on the board with P0 on its first load's sortie, over the crates' base.
fn patriot_sortie() -> (CampaignState, u32, WorldView) {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Patriot, &[]);
    let id = designate(&mut c, at, DefenceKind::Patriot).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    let offer = c.board.iter().find(|o| o.kind == MissionKind::Logistics).unwrap().clone();
    c.apply(P0, Command::TakeMission { offer: offer.id, base: None, aircraft: Aircraft::UH60 })
        .unwrap();
    let b = c.site(c.deployment(id).unwrap().base.unwrap()).unwrap();
    let view = flying_at(b.x, b.y);
    (c, id, view)
}

#[test]
fn a_crate_in_the_air_is_back_at_its_base_after_a_reload_and_set_down_ones_stay() {
    let (mut c, id, view) = patriot_sortie();
    assert_eq!(c.deployment(id).unwrap().crates, vec![CrateState::AtBase; 3]);
    c.apply(P0, Command::HookCrate { id, leg: 1 }).unwrap();
    c.apply(P0, Command::DropCrate { id, leg: 1, east: 4.0, north: -6.0, heading: 1.0 }).unwrap();
    // Not his sortie's load, but it is in.
    assert_eq!(c.deployment(id).unwrap().delivered, vec![1]);
    c.apply(P0, Command::HookCrate { id, leg: 2 }).unwrap();
    // Taken: not on the board, and no AI helicopter brings it.
    c.tick(tuning::BOARD_REFRESH, &view, &[]);
    let legs: Vec<_> = c
        .board
        .iter()
        .filter_map(|o| match o.objective {
            Objective::Logistics { leg, .. } => Some(leg),
            _ => None,
        })
        .collect();
    assert_eq!(legs, vec![0]);
    let ev = c.tick(tuning::DEPLOY_AI_WAIT + 1.0, &view, &[]);
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::CargoLaunched { .. })), "{ev:?}");
    let json = serde_json::to_string(&c).unwrap();
    let mut back: CampaignState = serde_json::from_str(&json).unwrap();
    back.after_load();
    let d = back.deployment(id).unwrap();
    assert_eq!(
        d.crates,
        vec![
            CrateState::AtBase,
            CrateState::Delivered { east: 4.0, north: -6.0, heading: 1.0 },
            CrateState::AtBase
        ]
    );
}

#[test]
fn a_crate_goes_back_when_its_helicopter_is_gone() {
    let (mut c, id, view) = patriot_sortie();
    c.apply(P0, Command::HookCrate { id, leg: 0 }).unwrap();
    c.tick(1.0, &view, &[]);
    assert_eq!(c.deployment(id).unwrap().crate_state(0), CrateState::Carried(P0));
    c.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(c.deployment(id).unwrap().crate_state(0), CrateState::AtBase);
}

#[test]
fn the_last_crate_builds_the_site_and_its_crates_go() {
    let (mut c, id, _) = patriot_sortie();
    let mut ev = Vec::new();
    for leg in 0..3 {
        c.apply(P0, Command::HookCrate { id, leg }).unwrap();
        let drop = Command::DropCrate { id, leg, east: leg as f32 * 5.0, north: 0.0, heading: 0.0 };
        ev.extend(c.apply(P0, drop).unwrap());
    }
    assert!(
        ev.iter().any(|e| matches!(e, CampaignEvent::SiteBuilt { kind: DefenceKind::Patriot, .. }))
    );
    let d = c.deployment(id).unwrap();
    assert!(d.built && d.crates.is_empty());
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { player: P0, .. })));
}

#[test]
fn a_save_from_before_the_crates_places_them() {
    let (mut c, id, _) = patriot_sortie();
    let ev = c.tick(tuning::DEPLOY_AI_WAIT + 1.0, &WorldView::default(), &[]);
    let Some(&CampaignEvent::CargoLaunched { tag, .. }) =
        ev.iter().find(|e| matches!(e, CampaignEvent::CargoLaunched { .. }))
    else {
        panic!("no cargo helicopter: {ev:?}")
    };
    c.tick(1.0, &WorldView::default(), &[Report::SupplyDelivered { tag }]);
    let leg = c.deployment(id).unwrap().delivered[0];
    let d = c.deployments.iter_mut().find(|d| d.id == id).unwrap();
    d.crates.clear();
    d.base = None;
    c.after_load();
    let d = c.deployment(id).unwrap();
    assert!(d.base.is_some());
    assert_eq!(d.crates.len(), 3);
    assert_eq!(d.crates[leg as usize], crate_slot(leg));
}

/// Convoy routes in every theater: from every place to the enemy places in reach, never over
/// mountains or wide water (checked every 100 m), and each theater with some. `GROUND_PGM=<dir>`
/// writes each theater's grid (black blocked, grey fords, white land).
#[test]
fn convoy_routes_stay_on_passable_ground() {
    let codes = [
        "libya",
        "persian_gulf",
        "north_cape",
        "central_europe",
        "vietnam",
        "middle_east",
        "desert_storm",
        "cuba",
        "korea",
        "ukraine",
    ];
    for (index, code) in codes.iter().enumerate() {
        let c = campaign_on_terrain(code, index);
        let t0 = std::time::Instant::now();
        let g = c.terrain.ground().expect("grid");
        let built = t0.elapsed();
        if let Ok(dir) = std::env::var("GROUND_PGM") {
            let d = ground::DIM;
            let mut out = format!("P5 {d} {d} 255\n").into_bytes();
            for r in 0..d as i32 {
                for col in 0..d as i32 {
                    let (x, y) = (col * ground::CELL + 8, r * ground::CELL + 8);
                    out.push(if g.dry(x, y) {
                        255
                    } else if g.passable(x, y) {
                        128
                    } else {
                        0
                    });
                }
            }
            std::fs::write(format!("{dir}/{code}.pgm"), out).unwrap();
        }
        let (mut pairs, mut routes) = (0, 0);
        let t1 = std::time::Instant::now();
        for a in &c.places {
            for b in &c.places {
                if a.owner == b.owner || a.owner == Side::Neutral || b.owner == Side::Neutral {
                    continue;
                }
                if dist(a.x - b.x, a.y - b.y) > tuning::CONVOY_MAX_ROUTE {
                    continue;
                }
                pairs += 1;
                let Some(r) = g.route((a.x, a.y), (b.x, b.y), tuning::CONVOY_MAX_ROUTE) else {
                    continue;
                };
                routes += 1;
                for w in r.windows(2) {
                    let n = (dist(w[1].0 - w[0].0, w[1].1 - w[0].1) / 6).max(1);
                    // The ends are the places' centres (a town may stand on a hill).
                    let inner = r.len() > 2 || n > 2;
                    for k in 0..=n {
                        let (x, y) = (
                            w[0].0 + (w[1].0 - w[0].0) * k / n,
                            w[0].1 + (w[1].1 - w[0].1) * k / n,
                        );
                        let end = (w[0] == r[0] && k * 6 < 0x60)
                            || (w[1] == *r.last().unwrap() && (n - k) * 6 < 0x60);
                        if inner && !end {
                            assert!(
                                g.passable(x, y),
                                "{code}: {} to {} crosses ({x}, {y})",
                                a.name,
                                b.name
                            );
                        }
                    }
                }
            }
        }
        eprintln!(
            "{code}: grid {built:?}, {routes} of {pairs} pairs in reach have a route ({:?})",
            t1.elapsed()
        );
    }
}

/// A Ukraine campaign that has just launched both sides' convoys: the blue one.
fn launched() -> (CampaignState, usize) {
    let mut c = campaign_on_terrain("ukraine", 9);
    c.convoys_due();
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    let n = ev.iter().filter(|e| matches!(e, CampaignEvent::ConvoyLaunched { .. })).count();
    assert_eq!(n, 2, "{ev:?}");
    // Not another while these are on their way.
    c.convoys_due();
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::ConvoyLaunched { .. })));
    let k = c.convoys.iter().position(|v| v.side == Side::Blue).unwrap();
    (c, k)
}

/// The blue convoy at its target with these vehicles, and the place's garrison alive or not.
fn at_target(c: &CampaignState, k: usize, units: &[ConvoyUnit], garrison: bool) -> WorldView {
    let v = &c.convoys[k];
    let p = &c.places[v.place as usize];
    let mut view = WorldView::default();
    for &u in units {
        let mut g = GroundView::gun(p.x + 40, p.y, Side::Blue, true);
        g.vehicle = Some(u);
        g.convoy = v.id;
        view.units.push(g);
    }
    let mut g = GroundView::gun(p.x - 30, p.y, Side::Red, garrison);
    g.vehicle = Some(ConvoyUnit::Tank);
    view.units.push(g);
    view
}

#[test]
fn a_convoy_with_troops_takes_a_place_without_a_garrison() {
    let (mut c, k) = launched();
    let place = c.convoys[k].place;
    assert!(c.convoys[k].route.len() >= 2);
    let view = at_target(&c, k, &[ConvoyUnit::Tank, ConvoyUnit::Carrier], true);
    c.tick(1.0, &view, &[]);
    assert_eq!(c.places[place as usize].owner, Side::Red, "the garrison holds");
    let view = at_target(&c, k, &[ConvoyUnit::Tank, ConvoyUnit::Carrier], false);
    let ev = c.tick(1.0, &view, &[]);
    assert_eq!(c.places[place as usize].owner, Side::Blue);
    assert_eq!(c.convoys[k].state, ConvoyState::Captured);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::PlaceCaptured { .. })));
    assert!(
        ev.iter().any(|e| matches!(*e, CampaignEvent::ConvoyEnded { id } if id == c.convoys[k].id))
    );
}

#[test]
fn a_convoy_without_troops_calls_for_helicopters() {
    let (mut c, k) = launched();
    let place = c.convoys[k].place;
    let view = at_target(&c, k, &[ConvoyUnit::Tank, ConvoyUnit::Escort], false);
    c.tick(1.0, &view, &[]);
    assert_eq!(c.places[place as usize].owner, Side::Red);
    assert!(
        c.insertions.iter().any(|i| i.place == place && i.side == Side::Blue),
        "{:?} {:?}",
        c.convoys[k].state,
        c.log.last()
    );
    assert_eq!(c.convoys[k].state, ConvoyState::Captured);
}

#[test]
fn a_destroyed_convoy_fails_and_the_next_one_comes() {
    let (mut c, k) = launched();
    let view = at_target(&c, k, &[], true);
    let mut view2 = view.clone();
    let mut dead = GroundView::gun(0, 0, Side::Blue, false);
    dead.vehicle = Some(ConvoyUnit::Tank);
    dead.convoy = c.convoys[k].id;
    view2.units.push(dead);
    c.tick(1.0, &view2, &[]);
    assert_eq!(c.convoys[k].state, ConvoyState::Failed);
    c.convoys_due();
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ConvoyLaunched { side: Side::Blue, .. })));
}

/// No convoy where no enemy place can be driven to: Cuba's are over the sea.
#[test]
fn no_convoy_over_the_sea() {
    let mut c = campaign_on_terrain("cuba", 7);
    c.convoys_due();
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::ConvoyLaunched { .. })));
    assert!(c.convoys.is_empty());
}

#[test]
fn a_convoy_resumes_from_where_it_was() {
    let (mut c, k) = launched();
    let r = c.convoys[k].route.clone();
    let mid = ((r[0].0 + r[1].0) / 2, (r[0].1 + r[1].1) / 2);
    c.convoys[k].at = mid;
    let left = c.convoys[k].route_left();
    assert_eq!(left[0], mid);
    assert_eq!(left[1..], r[1..]);
    let json = serde_json::to_string(&c).unwrap();
    let back: CampaignState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.convoys, c.convoys);
}

/// The board asks to stop the enemy's column and to clear the way for ours.
#[test]
fn the_board_offers_cas_for_the_columns() {
    let (mut c, k) = launched();
    let red = c.convoys.iter().find(|v| v.side == Side::Red).unwrap().id;
    let mut view = at_target(&c, k, &[ConvoyUnit::Tank, ConvoyUnit::Carrier], true);
    let mut enemy = GroundView::gun(1000, 1000, Side::Red, true);
    enemy.vehicle = Some(ConvoyUnit::Tank);
    enemy.convoy = red;
    view.units.push(enemy);
    c.board_at = f64::MIN;
    c.tick(1.0, &view, &[]);
    let armour = c.board.iter().find(|o| matches!(o.objective, Objective::Armour { .. }));
    let support = c.board.iter().find(|o| matches!(o.objective, Objective::Support { .. }));
    let Some(Objective::Armour { convoy, aaa }) = armour.map(|o| o.objective.clone()) else {
        panic!("{:?}", c.board)
    };
    assert_eq!((convoy, aaa), (red, vec![3]));
    let Some(Objective::Support { aaa, .. }) = support.map(|o| o.objective.clone()) else {
        panic!("{:?}", c.board)
    };
    assert_eq!(aaa, vec![2], "the garrison's tank");
    assert!(armour.unwrap().kind == MissionKind::Cas && support.unwrap().kind == MissionKind::Cas);
}
