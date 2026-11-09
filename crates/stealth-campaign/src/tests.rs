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
    fn ends_with(&self, s: &str) -> bool {
        self.en().ends_with(s)
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
        crew_scaling: true,
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

/// A friendly Patriot radar (AAA index 0) at a rear airfield, alive or not.
fn patriot_view(c: &CampaignState, alive: bool) -> WorldView {
    let (_, x, y) = c.patriot_sites()[0];
    WorldView { units: vec![GroundView::gun(x, y, Side::Blue, alive)], ..Default::default() }
}

#[test]
fn a_sam_kill_draws_enemy_sead_at_the_battery() {
    let mut c = campaign("central_europe", 3);
    let view = patriot_view(&c, true);
    c.tick(1.0, &view, &[]);
    let kill = Report::SamKill { aaa: 0, side: Side::Red };
    c.tick(1.0, &view, &[kill.clone()]);
    let sead: Vec<&Raid> = c.raids.iter().filter(|r| r.sam.is_some()).collect();
    assert_eq!(sead.len(), 1, "{:?}", c.raids);
    let (id, sam) = (sead[0].id, sead[0].sam.unwrap());
    assert_eq!((sead[0].side, sam.aaa), (Side::Red, 0));
    let offer = c.board.iter().find(|o| o.objective == Objective::Defend { raid: id });
    assert!(
        offer.is_some_and(|o| o.title.ends_with("SAM battery") && (o.x, o.y) == (sam.x, sam.y))
    );
    // One raid per battery, and not again so soon.
    c.tick(1.0, &view, &[kill.clone()]);
    assert_eq!(c.raids.iter().filter(|r| r.sam.is_some()).count(), 1);
    // The escort, then the SEAD aircraft.
    c.tick(tuning::RAID_LEAD - tuning::RAID_ESCORT_LEAD + 1.0, &view, &[]);
    let ev = c.tick(tuning::RAID_ESCORT_LEAD, &view, &[]);
    let tags =
        ev.iter()
            .find_map(|e| match e {
                CampaignEvent::RaidLaunched {
                    id: i, side: Side::Red, sam: Some(0), tags, ..
                } if *i == id => Some(tags.clone()),
                _ => None,
            })
            .expect("SEAD aircraft launched");
    assert_eq!(tags.len(), tuning::RAID_AIRCRAFT);
    // Their missiles take out the radar.
    let dead = patriot_view(&c, false);
    c.tick(1.0, &dead, &[Report::AaaDestroyed { index: 0, by: None }]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::Hit);
}

#[test]
fn no_enemy_sead_for_a_kill_by_a_dead_radar_or_an_enemy_one() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let kill = |side| Report::SamKill { aaa: 0, side };
    c.tick(1.0, &patriot_view(&c, false), &[kill(Side::Red)]);
    c.tick(1.0, &patriot_view(&c, true), &[kill(Side::Blue)]);
    assert!(c.raids.iter().all(|r| r.sam.is_none()));
}

#[test]
fn enemy_sead_is_called_off_once_its_radar_is_gone() {
    let mut c = campaign("central_europe", 3);
    let view = patriot_view(&c, true);
    c.tick(1.0, &view, &[Report::SamKill { aaa: 0, side: Side::Red }]);
    let id = c.raids.iter().find(|r| r.sam.is_some()).expect("SEAD ordered").id;
    c.tick(1.0, &patriot_view(&c, false), &[]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::CalledOff);
}

#[test]
fn enemy_sead_finds_its_radar_anew_after_a_reload() {
    let mut c = campaign("central_europe", 3);
    let view = patriot_view(&c, true);
    c.tick(1.0, &view, &[Report::SamKill { aaa: 0, side: Side::Red }]);
    let id = c.raids.iter().find(|r| r.sam.is_some()).expect("SEAD ordered").id;
    // The reloaded simulation numbers the radar 2.
    let mut moved = view.clone();
    moved.units.insert(0, GroundView::gun(1, 1, Side::Blue, true));
    moved.units.insert(0, GroundView::gun(2, 2, Side::Blue, true));
    c.tick(1.0, &moved, &[]);
    assert_eq!(c.raid(id).unwrap().sam.unwrap().aaa, 2);
    assert!(c.raid(id).unwrap().open());
    c.tick(1.0, &moved, &[Report::AaaDestroyed { index: 0, by: None }]);
    assert!(c.raid(id).unwrap().open(), "another unit's loss is no hit");
}

#[test]
fn enemy_sead_scores_no_hit_on_a_radar_a_player_destroyed() {
    let mut c = campaign("central_europe", 3);
    let view = patriot_view(&c, true);
    c.tick(1.0, &view, &[Report::SamKill { aaa: 0, side: Side::Red }]);
    let id = c.raids.iter().find(|r| r.sam.is_some()).expect("SEAD ordered").id;
    c.tick(tuning::RAID_LEAD + 1.0, &view, &[]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::Enroute);
    let ev =
        c.tick(1.0, &patriot_view(&c, false), &[Report::AaaDestroyed { index: 0, by: Some(P0) }]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::CalledOff);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Recall { .. })));
}

#[test]
fn a_destroyed_patriot_battery_stays_gone() {
    let mut c = campaign("central_europe", 3);
    let (place, _, _) = c.patriot_sites()[0];
    let guns: Vec<u8> = c.places.iter().map(|p| p.guns_lost).collect();
    c.tick(1.0, &patriot_view(&c, false), &[Report::AaaDestroyed { index: 0, by: None }]);
    assert!(c.patriot_sites().iter().all(|s| s.0 != place));
    // No garrison gun to replace.
    assert_eq!(c.places.iter().map(|p| p.guns_lost).collect::<Vec<_>>(), guns);
    // Saved with the campaign.
    let d: CampaignState = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(d.patriots_lost, vec![place]);
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
    // A sunk carrier is no refuge either.
    let mut c = campaign("persian_gulf", 1);
    for p in c.places.iter_mut().filter(|p| p.owner == Side::Blue && p.airfield.is_some()) {
        p.owner = Side::Red;
    }
    let carriers: Vec<ObjectId> = c
        .sites
        .iter()
        .filter(|s| s.kind == SiteKind::Carrier && s.side == Side::Blue)
        .map(|s| s.object)
        .collect();
    assert!(!carriers.is_empty());
    let sunk: Vec<Report> =
        carriers.iter().map(|&object| Report::ObjectDestroyed { object, by: None }).collect();
    let ev = c.tick(1.0, &WorldView::default(), &sunk);
    assert!(ev.contains(&CampaignEvent::Ended { winner: Side::Red }), "{ev:?}");
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
        due: Vec::new(),
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
        sam: None,
        feint: false,
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
        Aircraft::OH58,
        Aircraft::CH47,
        Aircraft::F22,
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
    assert_eq!(Cas.pick(Aircraft::OH58), Aircraft::OH58);
    assert_eq!(Insertion.pick(Aircraft::OH58), Aircraft::UH60);
    assert_eq!(Cas.pick(Aircraft::AV8B), Aircraft::AV8B);
    assert_eq!(Sead.pick(Aircraft::AV8B), Aircraft::F16);
    assert_eq!(Interdiction.pick(Aircraft::RAH66), Aircraft::F117);
    // The CH-47 slings cargo and does nothing else.
    assert_eq!(Logistics.pick(Aircraft::CH47), Aircraft::CH47);
    assert_eq!(Resupply.pick(Aircraft::CH47), Aircraft::CH47);
    // The F-22A flies the jets' missions, never close support.
    assert_eq!(Cap.pick(Aircraft::F22), Aircraft::F22);
    assert_eq!(Strike.pick(Aircraft::F22), Aircraft::F22);
    assert_eq!(Cas.pick(Aircraft::F22), Aircraft::A10);
    assert_eq!(Insertion.pick(Aircraft::CH47), Aircraft::UH60);
    assert_eq!(Csar.pick(Aircraft::CH47), Aircraft::UH60);
    assert_eq!(Strike.pick(Aircraft::CH47), Aircraft::F117);
    assert!(!Insertion.flyable_by(Aircraft::CH47) && !Csar.flyable_by(Aircraft::CH47));
    assert!(!Cas.flyable_by(Aircraft::CH47) && !Escort.flyable_by(Aircraft::CH47));
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
    let hills =
        std::fs::read(format!("{}/../../assets/theaters/{code}.hills", env!("CARGO_MANIFEST_DIR")))
            .ok()
            .and_then(stealth_mission::hills::Hills::from_bytes)
            .unwrap_or_default();
    c.terrain = Terrain::new(Some(std::sync::Arc::new(theater(code))), relief(code), hills);
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
    assert!(MissionKind::Resupply.flyable_by(Aircraft::CH47));
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
fn a_nasams_site_takes_two_loads() {
    let mut c = campaign("central_europe", 3);
    let at = deploy_spot(&c, DefenceKind::Nasams, &[]);
    let id = designate(&mut c, at, DefenceKind::Nasams).unwrap();
    c.tick(1.0, &WorldView::default(), &[]);
    let offers: Vec<_> = c.board.iter().filter(|o| o.kind == MissionKind::Logistics).collect();
    assert_eq!(offers.len(), 2);
    assert!(offers[0].title.starts_with("NASAMS 1/2"), "{}", offers[0].title);
    assert!(c.deployment(id).unwrap().title().starts_with("NASAMS site near"));
    let s = DefenceStock { x: 1, y: 2, kind: DefenceKind::Nasams, stock: 9, stock2: 0, stock3: 0 };
    let back: DefenceStock = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(back, s);
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
fn a_fast_tracked_site_is_built_for_materiel_and_its_helicopters_fly_home() {
    let (mut c, id, _) = patriot_sortie();
    // One load in by the player, one on its way by AI helicopter.
    c.apply(P0, Command::HookCrate { id, leg: 0 }).unwrap();
    c.apply(P0, Command::DropCrate { id, leg: 0, east: 0.0, north: 0.0, heading: 0.0 }).unwrap();
    let ev = c.tick(tuning::DEPLOY_AI_WAIT + 1.0, &WorldView::default(), &[]);
    let Some(&CampaignEvent::CargoLaunched { tag, .. }) =
        ev.iter().find(|e| matches!(e, CampaignEvent::CargoLaunched { .. }))
    else {
        panic!("no cargo helicopter: {ev:?}")
    };
    let cost = fast_track_cost(c.deployment(id).unwrap());
    assert_eq!(cost, 2.0 * tuning::DEPLOY_FAST_TRACK_COST);
    // Not enough materiel: nothing happens.
    c.blue.materiel = cost - 1.0;
    assert!(c.apply(P0, Command::FastTrackSite { id }).is_err());
    assert!(!c.deployment(id).unwrap().built);
    c.blue.materiel = cost + 5.0;
    let ev = c.apply(P0, Command::FastTrackSite { id }).unwrap();
    assert_eq!(c.blue.materiel, 5.0);
    assert!(ev.contains(&CampaignEvent::Recall { tag }));
    assert!(
        ev.iter().any(|e| matches!(e, CampaignEvent::SiteBuilt { kind: DefenceKind::Patriot, .. }))
    );
    let d = c.deployment(id).unwrap();
    assert!(d.built && d.ai.is_empty() && d.crates.is_empty());
    // Its loads leave the board.
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let load =
        |o: &MissionOffer| matches!(o.objective, Objective::Logistics { site, .. } if site == id);
    assert!(!c.board.iter().any(load));
    // Built already: not again.
    assert!(c.apply(P0, Command::FastTrackSite { id }).is_err());
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

#[test]
fn sorties_get_unique_callsigns_by_mission() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (b, d, e) = (PlayerId(1), PlayerId(2), PlayerId(3));
    for name in ["Bravo", "Delta", "Echo"] {
        c.join(name, 4, |_| false);
    }
    let offers: Vec<u32> = c
        .board
        .iter()
        .filter(|o| !o.kind.ai_flown() && o.kind.flyable_by(Aircraft::F16))
        .map(|o| o.id)
        .collect();
    assert!(offers.len() >= 2, "two missions");
    let fly = |c: &mut CampaignState, p: PlayerId, offer: u32| {
        let cmd = Command::TakeMission { offer, base: None, aircraft: Aircraft::F16 };
        c.apply(p, cmd).unwrap();
        let s = c.sortie(p).unwrap();
        (s.flight, s.position)
    };
    assert_eq!(fly(&mut c, P0, offers[0]), (1, 1));
    // The same mission: the same flight, the next position.
    assert_eq!(fly(&mut c, b, offers[0]), (1, 2));
    // Another mission: a flight of its own.
    assert_eq!(fly(&mut c, d, offers[1]), (2, 1));
    // The first one home, his position is free again; flight 1 stays with the mission.
    c.apply(P0, Command::EndSortie { end: SortieEnd::Aborted }).unwrap();
    assert_eq!(fly(&mut c, e, offers[0]), (1, 1));
}

#[test]
fn the_crew_sets_the_enemys_pace() {
    assert_eq!(crew_scale(1.0), 1.0);
    assert!((crew_scale(2.0) - 1.516).abs() < 0.01);
    assert!((crew_scale(4.0) - 2.297).abs() < 0.01);
    assert_eq!(crew_scale(64.0), tuning::CREW_SCALE_MAX);
    let mut c = campaign("central_europe", 3);
    assert_eq!(c.pace(), 1.0);
    // Four players: the count rises one player per CREW_RISE, the pace with it.
    c.set_crew(4);
    c.tick(tuning::CREW_RISE, &WorldView::default(), &[]);
    assert!((c.crew.level - 2.0).abs() < 1e-9);
    c.tick(2.0 * tuning::CREW_RISE, &WorldView::default(), &[]);
    assert_eq!(c.crew.level, 4.0);
    assert_eq!(c.pace(), crew_scale(4.0));
    assert_eq!(c.board_size(), tuning::BOARD_SIZE + 3 * tuning::CREW_OFFERS);
    // One drops out for a minute: the pace hardly moves.
    c.set_crew(3);
    c.tick(60.0, &WorldView::default(), &[]);
    assert!(c.crew.level > 3.8);
    // Saved with the campaign; an older save plays as one player.
    let d: CampaignState = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(d.crew, c.crew);
    let mut v = serde_json::to_value(&c).unwrap();
    v.as_object_mut().unwrap().remove("crew");
    v["setup"].as_object_mut().unwrap().remove("crew_scaling");
    let d: CampaignState = serde_json::from_value(v).unwrap();
    assert_eq!((d.crew, d.pace()), (Crew::default(), 1.0));
    // Off: the same war for any crew.
    c.setup.crew_scaling = false;
    assert_eq!((c.pace(), c.board_size()), (1.0, tuning::BOARD_SIZE));
}

#[test]
fn more_players_draw_more_enemy_patrols() {
    let patrols = |players: usize| {
        let mut c = campaign("central_europe", 3);
        c.set_crew(players);
        // The fighters stay up.
        for _ in 0..60 {
            let flights =
                c.patrols.iter().map(|p| FlightView { x: 0, y: 0, side: p.1, tag: p.0 }).collect();
            c.tick(60.0, &WorldView { flights, ..Default::default() }, &[]);
        }
        c.patrols.iter().filter(|p| p.1 == Side::Red).count()
    };
    let (one, four) = (patrols(1), patrols(4));
    assert!(four >= one + 3, "{one} patrols alone, {four} for four");
    // Four: a pace of 2.3, 1.3 steps of 2 more each.
    let mut c = campaign("central_europe", 3);
    let roe = 1;
    assert_eq!(c.patrol_cap(), tuning::PATROLS + roe);
    c.set_crew(4);
    c.tick(3.0 * tuning::CREW_RISE, &WorldView::default(), &[]);
    assert_eq!(c.patrol_cap(), tuning::PATROLS + roe + 3);
}

#[test]
fn repair_crews_work_on_a_few_sites_at_a_time_and_slower_each_time() {
    let mut c = campaign("central_europe", 3);
    let crews = c.repair_crews(Side::Red);
    assert!((1..=tuning::REPAIR_CREWS_MAX).contains(&crews));
    // More SAM sites destroyed than the enemy has crews: the rest wait.
    let sams: Vec<ObjectId> = c
        .sites
        .iter()
        .filter(|s| s.kind == SiteKind::Sam && s.alive() && c.owner(s.object) == Some(Side::Red))
        .map(|s| s.object)
        .take(crews + 2)
        .collect();
    assert_eq!(sams.len(), crews + 2);
    let hit: Vec<Report> =
        sams.iter().map(|&object| Report::ObjectDestroyed { object, by: Some(P0) }).collect();
    c.red.materiel = tuning::MATERIEL_MAX;
    c.tick(1.0, &WorldView::default(), &hit);
    let at_work = |c: &CampaignState| c.sites.iter().filter(|s| s.repair_from.is_some()).count();
    assert_eq!(at_work(&c), crews);
    let ev = c.tick(tuning::REPAIR_SITE, &WorldView::default(), &[]);
    let repaired = ev.iter().filter(|e| matches!(e, CampaignEvent::Repaired { .. })).count();
    assert_eq!(repaired, crews);
    // The next ones are under way now.
    assert_eq!(at_work(&c), 2.min(crews));
    // A site destroyed again soon after its repair takes longer.
    let again = sams.iter().copied().find(|&o| c.site(o).unwrap().alive()).unwrap();
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: again, by: None }]);
    let s = c.site(again).unwrap();
    assert_eq!(s.hits, 2);
    assert_eq!(s.repair_time(), tuning::REPAIR_SITE * tuning::REPAIR_SLOWDOWN);
}

#[test]
fn the_awacs_sends_patrols_at_each_player_in_turn() {
    let mut c = campaign("central_europe", 3);
    c.tick(tuning::AWACS_EVERY + 1.0, &WorldView::default(), &[]);
    assert!(c.awacs.is_some());
    let at = |id, x, y| PlayerPos {
        id: PlayerId(id),
        x,
        y,
        airborne: true,
        landed: false,
        hover_over: None,
    };
    let players = vec![at(0, 0x4000, 0x4000), at(1, 0x5000, 0x3000)];
    c.patrols.clear();
    let mut goals = Vec::new();
    for _ in 0..30 {
        // Fighters to spare (none ever land here).
        c.red.fighters = 30;
        c.red.materiel = tuning::MATERIEL_MAX;
        // No raids (their escorts patrol too); the patrols stay up.
        c.raids.clear();
        c.raid_at = c.clock;
        let flights =
            c.patrols.iter().map(|p| FlightView { x: 0, y: 0, side: p.1, tag: p.0 }).collect();
        let view = WorldView { players: players.clone(), flights, ..Default::default() };
        for e in c.tick(60.0, &view, &[]) {
            if let CampaignEvent::LaunchFighters { side: Side::Red, x, y, .. } = e {
                goals.push((x, y));
            }
        }
    }
    for p in &players {
        assert!(goals.contains(&(p.x, p.y)), "no patrol at {:?}: {goals:?}", (p.x, p.y));
    }
}

#[test]
fn enemy_patrols_guard_the_place_the_players_strike() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let sam = c
        .sites
        .iter()
        .find(|s| {
            s.kind == SiteKind::Sam
                && s.alive()
                && s.place.is_some()
                && c.owner(s.object) == Some(Side::Red)
        })
        .unwrap();
    let (object, place) = (sam.object, sam.place.unwrap());
    let (px, py) = (c.places[place as usize].x, c.places[place as usize].y);
    c.red.fighters = 30;
    c.raid_at = f64::MAX;
    c.patrol_at = f64::MIN;
    let ev =
        c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object, by: Some(P0) }]);
    let goal = ev.iter().find_map(|e| match e {
        CampaignEvent::LaunchFighters { side: Side::Red, x, y, .. } => Some((*x, *y)),
        _ => None,
    });
    assert_eq!(goal, Some((px, py)));
    // Long after, the patrols go back to the front.
    c.tick(tuning::PRESSURE_MEMORY, &WorldView::default(), &[]);
    assert!(c.places[place as usize].struck_at.is_some());
    c.patrols.clear();
    c.patrol_at = f64::MIN;
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    let goal = ev.iter().find_map(|e| match e {
        CampaignEvent::LaunchFighters { side: Side::Red, x, y, .. } => Some((*x, *y)),
        _ => None,
    });
    assert!(goal.is_some_and(|g| g != (px, py)), "{goal:?}");
}

/// Player `p` airborne at (x, y).
fn up_at(p: PlayerId, x: i32, y: i32) -> PlayerPos {
    PlayerPos { id: p, x, y, airborne: true, landed: false, hover_over: None }
}

#[test]
fn a_struck_airfield_holds_back_the_enemys_raids_and_the_debrief_says_so() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, obj) = take(&mut c, P0, MissionKind::Strike);
    let Objective::Strike { airfield, .. } = obj else { unreachable!() };
    c.raid_at = f64::MIN / 2.0;
    c.tick(
        1.0,
        &WorldView::default(),
        &[Report::ObjectDestroyed { object: airfield, by: Some(P0) }],
    );
    // No raid for the delay, whatever its interval says.
    let every = tuning::RAID_EVERY[1];
    assert!(c.raid_at >= c.clock + tuning::STRUCK_AIRFIELD_DELAY - every - 2.0);
    let record = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap());
    assert!(
        record.effects.iter().any(|e| e.contains("airfield out of action")),
        "{:?}",
        record.effects
    );
}

#[test]
fn a_struck_city_holds_back_the_enemys_convoys_and_insertions() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    take(&mut c, P0, MissionKind::Strike);
    let object = c
        .sites
        .iter()
        .find(|s| s.kind == SiteKind::City && c.owner(s.object) == Some(Side::Red))
        .unwrap()
        .object;
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object, by: Some(P0) }]);
    let now = c.clock;
    let insert = tuning::ENEMY_INSERT_EVERY[1];
    let convoy = tuning::ENEMY_CONVOY_EVERY[1];
    assert!(c.enemy_insert_at >= now + tuning::STRUCK_CITY_DELAY - insert - 1.0);
    assert!(c.convoy_at[1] >= now + tuning::STRUCK_CITY_DELAY - convoy - 1.0);
    let s = c.sortie(P0).unwrap();
    assert!(s.progress.effects.iter().any(|e| e.contains("in ruins")), "{:?}", s.progress.effects);
}

#[test]
fn a_player_on_cap_keeps_enemy_raids_off_the_places_under_it() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, _) = take(&mut c, P0, MissionKind::Cap);
    // Over a friendly airfield: the enemy raids it no more.
    let (x, y) = c
        .places
        .iter()
        .find(|p| p.owner == Side::Blue && p.airfield.is_some())
        .map(|p| (p.x, p.y))
        .unwrap();
    c.slot_mut(P0).sortie.as_mut().unwrap().offer.objective = Objective::Cap { x, y };
    let view = WorldView { players: vec![up_at(P0, x, y)], ..Default::default() };
    c.tick(1.0, &view, &[]);
    let place = c.places.iter().position(|p| (p.x, p.y) == (x, y)).unwrap() as PlaceId;
    assert!(c.capped(place));
    let mut raided = 0;
    for _ in 0..8 {
        c.raid_at = f64::MIN / 2.0;
        c.tick(1.0, &view, &[]);
        raided += c.raids.iter().filter(|r| r.side == Side::Red && r.place == place).count();
    }
    assert_eq!(raided, 0);
    let s = c.sortie(P0).unwrap();
    assert!(
        s.progress.effects.iter().any(|e| e.contains("Your CAP kept")),
        "{:?}",
        s.progress.effects
    );
}

#[test]
fn an_escort_keeps_the_enemys_fighters_off_the_helicopters() {
    let scrambled = |escorted: bool| {
        let mut c = campaign("central_europe", 3);
        let place = c.front(Side::Blue)[0];
        c.apply(P0, Command::OrderInsertion { place }).unwrap();
        c.tick(1.0, &WorldView::default(), &[]);
        let escort = c.board.iter().find(|o| o.kind == MissionKind::Escort).unwrap().clone();
        c.apply(P0, Command::TakeMission { offer: escort.id, base: None, aircraft: Aircraft::F16 })
            .unwrap();
        let Objective::Escort { insertion } = escort.objective else { unreachable!() };
        c.tick(tuning::INSERTION_LEAD, &WorldView::default(), &[]);
        let ins = c.insertion(insertion).unwrap().clone();
        assert_eq!(ins.state, InsertionState::Enroute);
        // The helicopters over an enemy airfield, the escort with them (or not).
        let t = c.site(ins.target).unwrap();
        let b = c.nearest_base(Side::Red, t.x, t.y).unwrap();
        let (x, y) = (c.site(b).unwrap().x, c.site(b).unwrap().y);
        let flights =
            ins.helicopters.iter().map(|&tag| FlightView { x, y, side: Side::Blue, tag }).collect();
        let players = if escorted { vec![up_at(P0, x, y)] } else { Vec::new() };
        let view = WorldView { players, flights, ..Default::default() };
        c.tick(1.0, &view, &[]);
        let ev = c.tick(1.0, &view, &[]);
        let fx = c.sortie(P0).unwrap().progress.effects.clone();
        (c.insertion(insertion).unwrap().scrambled, ev, fx)
    };
    let (alone, _, fx) = scrambled(false);
    assert!(alone && fx.is_empty());
    let (_, ev, fx) = scrambled(true);
    assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::LaunchFighters { side: Side::Red, .. })));
    assert!(fx.iter().any(|e| e.contains("Your escort kept")), "{fx:?}");
}

#[test]
fn under_the_airframe_rule_a_crashed_jet_is_one_fighter_fewer() {
    for rule in [false, true] {
        let mut c = campaign("central_europe", 3);
        c.rules = Some(CampaignRules { airframe_losses: rule, ..Default::default() });
        c.tick(1.0, &WorldView::default(), &[]);
        take(&mut c, P0, MissionKind::Strike);
        let fighters = c.blue.fighters;
        let record =
            finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap());
        assert_eq!(c.blue.fighters, fighters - rule as u32);
        assert_eq!(record.effects.iter().any(|e| e.contains("one fighter fewer")), rule);
        // Landed: nothing lost.
        take(&mut c, P0, MissionKind::Strike);
        c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap();
        assert_eq!(c.blue.fighters, fighters - rule as u32);
    }
}

#[test]
fn urgent_missions_say_when_they_are_due() {
    let mut c = campaign("central_europe", 3);
    let (id, _, _) = {
        // An enemy raid ordered: its Defend offer is due at lift-off, then now.
        c.tick(1.0, &WorldView::default(), &[]);
        c.raid_at = f64::MIN / 2.0;
        c.tick(1.0, &WorldView::default(), &[]);
        let r = c.raids.iter().find(|r| r.side == Side::Red).expect("a raid");
        (r.id, r.place, r.target)
    };
    let RaidState::Pending { launch_at } = c.raid(id).unwrap().state else { panic!() };
    assert_eq!(c.due(&Objective::Defend { raid: id }), Some(Due::At(launch_at)));
    c.tick(launch_at - c.clock + 1.0, &WorldView::default(), &[]);
    assert_eq!(c.due(&Objective::Defend { raid: id }), Some(Due::Now));
    assert_eq!(c.due(&Objective::Cap { x: 0, y: 0 }), None);
}

#[test]
fn the_front_moves_for_who_takes_places_and_destroys_sites() {
    let mut c = campaign("central_europe", 3);
    assert_eq!(c.momentum(), 0);
    let place = c.front(Side::Blue)[0];
    let mut ev = Vec::new();
    c.capture_place(place, Side::Blue, &mut ev);
    assert_eq!(c.momentum(), tuning::MOMENTUM_CAPTURE as i32);
    let city = c
        .sites
        .iter()
        .find(|s| s.kind == SiteKind::City && s.alive() && c.owner(s.object) == Some(Side::Blue))
        .unwrap()
        .object;
    c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: city, by: None }]);
    assert_eq!(c.momentum(), tuning::MOMENTUM_CAPTURE as i32 - 1);
    // Forgotten after a while.
    c.clock += tuning::MOMENTUM_WINDOW + 1.0;
    assert_eq!(c.momentum(), 0);
}

#[test]
fn a_prepared_operation_sends_the_insertion_in_at_once() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    // A front place with SAM sites to take down.
    let front = c.front(Side::Blue);
    let place = *front.iter().find(|&&p| !c.places[p as usize].defences.is_empty()).unwrap();
    let op = c.operation(place).unwrap();
    assert_eq!(op.done(), 0);
    assert!(op.steps.iter().any(|s| s.0 == MissionKind::Sead));
    // Not on the front: no operation.
    let rear = (0..c.places.len() as PlaceId).find(|p| !front.contains(p)).unwrap();
    assert!(c.operation(rear).is_none());
    // Recon, SEAD and the strike done (the guns are not in the view).
    c.places[place as usize].intel_until = Some(c.clock + 3600.0);
    let targets: Vec<ObjectId> = c.places[place as usize]
        .defences
        .iter()
        .copied()
        .chain(c.places[place as usize].airfield)
        .collect();
    let hit: Vec<Report> =
        targets.iter().map(|&object| Report::ObjectDestroyed { object, by: None }).collect();
    c.auto_insert_at = c.clock;
    let ev = c.tick(1.0, &WorldView::default(), &hit);
    let op = c.operation(place).unwrap();
    assert!(op.ready(), "{op:?}");
    let ev: Vec<_> = ev.into_iter().chain(c.tick(1.0, &WorldView::default(), &[])).collect();
    assert!(c.insertions.iter().any(|i| i.side == Side::Blue && i.place == place), "{:?}", ev);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(t) if t.contains("prepared"))));
}

#[test]
fn the_awacs_diverts_a_pilot_to_a_raid_near_him() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let (_, strike) = take(&mut c, P0, MissionKind::Strike);
    // An enemy raid ordered: its Defend offer on the board.
    c.raid_at = f64::MIN / 2.0;
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    let defend =
        c.board.iter().find(|o| matches!(o.objective, Objective::Defend { .. })).unwrap().clone();
    // Not seen in the air yet: nothing to divert.
    assert!(c.divert_for(P0).is_none());
    let view = WorldView { players: vec![up_at(P0, defend.x, defend.y)], ..Default::default() };
    c.tick(1.0, &view, &[Report::ObjectDestroyed { object: 0, by: Some(P0) }]);
    let kills = c.sortie(P0).unwrap().progress.sites_destroyed.clone();
    assert_eq!(c.divert_for(P0).map(|o| o.id), Some(defend.id));
    let ev = c.apply(P0, Command::Retask { offer: defend.id }).unwrap();
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::SortieRetasked { .. })));
    let s = c.sortie(P0).unwrap();
    assert_eq!(s.offer.id, defend.id);
    assert_eq!(s.progress.sites_destroyed, kills);
    assert!(c.plans.iter().any(|p| p.offer == defend.id && p.flying.contains(&P0)));
    // On an urgent mission now: no divert from it, none back to the strike.
    assert!(c.divert_for(P0).is_none());
    let back = c.board.iter().find(|o| o.objective == strike).map(|o| o.id);
    if let Some(back) = back {
        assert!(c.apply(P0, Command::Retask { offer: back }).is_err());
    }
}

#[test]
fn the_enemy_raids_the_sites_the_players_built() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let spot = deploy_spot(&c, DefenceKind::Gepard, &[]);
    let id = designate(&mut c, spot, DefenceKind::Gepard).unwrap();
    let k = c.deployments.iter().position(|d| d.id == id).unwrap();
    c.deployments[k].built = true;
    let (x, y) = c.deployments[k].unit_at();
    let view =
        WorldView { units: vec![GroundView::gun(x, y, Side::Blue, true)], ..Default::default() };
    // One raid in a few goes for it.
    let mut raid = None;
    for _ in 0..40 {
        c.red.fighters = 30;
        c.red.materiel = tuning::MATERIEL_MAX;
        c.raid_at = f64::MIN / 2.0;
        c.tick(1.0, &view, &[]);
        raid = c.raids.iter().find(|r| r.sam.is_some_and(|s| s.site == Some(id))).cloned();
        if raid.is_some() {
            break;
        }
        c.raids.retain(|r| r.sam.is_some());
    }
    let r = raid.expect("a raid on the Gepard");
    assert_eq!(r.sam.unwrap().aaa, 0);
    c.tick(tuning::BOARD_REFRESH, &view, &[]);
    let o = c.board.iter().find(|o| o.objective == Objective::Defend { raid: r.id }).unwrap();
    assert!(o.title.contains("Gepard near"), "{}", o.title.en());
}

#[test]
fn a_feint_turns_back_on_its_way() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let mut feint = None;
    for _ in 0..40 {
        c.red.fighters = 30;
        c.red.materiel = tuning::MATERIEL_MAX;
        c.raid_at = f64::MIN / 2.0;
        c.tick(1.0, &WorldView::default(), &[]);
        feint = c.raids.iter().find(|r| r.feint && r.open()).map(|r| r.id);
        if feint.is_some() {
            break;
        }
        c.raids.clear();
    }
    let id = feint.expect("a feint");
    // It goes with a real raid on another airfield.
    let real: Vec<&Raid> = c.raids.iter().filter(|r| r.open() && !r.feint).collect();
    assert!(real.iter().any(|r| r.target != c.raid(id).unwrap().target));
    c.tick(tuning::RAID_LEAD + 1.0, &WorldView::default(), &[]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::Enroute);
    let ev = c.tick(tuning::FEINT_TURN + 1.0, &WorldView::default(), &[]);
    assert_eq!(c.raid(id).unwrap().state, RaidState::Repelled);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Recall { .. })));
}

#[test]
fn enemy_insertions_go_for_what_a_place_is_worth() {
    let mut c = campaign("central_europe", 3);
    c.tick(1.0, &WorldView::default(), &[]);
    let front = c.front(Side::Red);
    let with_airfield = front.iter().copied().find(|&p| {
        c.places[p as usize].airfield.is_some_and(|a| c.site(a).is_some_and(Site::alive))
    });
    if let Some(p) = with_airfield {
        let v = c.place_value(p);
        let a = c.places[p as usize].airfield.unwrap();
        c.tick(1.0, &WorldView::default(), &[Report::ObjectDestroyed { object: a, by: None }]);
        assert_eq!(c.place_value(p), v - tuning::AIRFIELD_VALUE);
    }
    // Undefended (no guns in the view), the most valuable ones are the targets.
    let mut values: Vec<(i32, PlaceId)> = front.iter().map(|&p| (c.place_value(p), p)).collect();
    values.sort_by_key(|v| std::cmp::Reverse(v.0));
    let best = values[tuning::ENEMY_TARGET_CHOICE.min(values.len()) - 1].0;
    c.enemy_insert_at = f64::MIN / 2.0;
    c.tick(1.0, &WorldView::default(), &[]);
    let ins = c.insertions.iter().find(|i| i.side == Side::Red).expect("an enemy insertion");
    assert!(c.place_value(ins.place) >= best, "{values:?} -> {}", ins.place);
}

/// A campaign with the AI flying the boards (the default rules).
fn ai_campaign() -> CampaignState {
    let mut c = campaign("central_europe", 3);
    c.rules = Some(CampaignRules::default());
    c.tick(1.0, &WorldView::default(), &[]);
    c
}

fn ai_launched(ev: &[CampaignEvent]) -> Vec<(Side, Vec<u32>)> {
    ev.iter()
        .filter_map(|e| match e {
            CampaignEvent::AiMissionLaunched { side, tags, .. } => Some((*side, tags.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn the_ai_flies_an_offer_five_minutes_after_it_was_posted() {
    let mut c = ai_campaign();
    let recon = c.board.iter().find(|o| o.kind == MissionKind::Recon).expect("recon").clone();
    let due = c.ai_launch_at(&recon).expect("the AI flies it");
    assert!(due >= recon.posted + tuning::AI_MISSION_WAIT);
    let first = c.board.iter().filter_map(|o| c.ai_launch_at(o)).fold(f64::MAX, f64::min);
    c.tick(first - c.clock - 1.0, &WorldView::default(), &[]);
    assert!(!c.ai_missions.iter().any(|m| m.side == Side::Blue), "not before its time");
    // Then a quarter hour: each side's aircraft take off one at a time, 10 s apart at least,
    // no more than the quarter hour's.
    let mut takeoffs: Vec<(Side, f64)> = Vec::new();
    for _ in 0..15 * 60 {
        let ev = c.tick(1.0, &WorldView::default(), &[]);
        for (side, tags) in ai_launched(&ev) {
            assert_eq!(tags.len(), 1, "one at a time");
            takeoffs.push((side, c.clock));
        }
    }
    for side in [Side::Blue, Side::Red] {
        let times: Vec<f64> = takeoffs.iter().filter(|t| t.0 == side).map(|t| t.1).collect();
        assert!(!times.is_empty(), "{side:?} flies");
        // (Seen at the end of 1 s steps.)
        assert!(times.windows(2).all(|w| w[1] - w[0] >= tuning::AI_STAGGER - 1.0), "{times:?}");
        let pooled: usize = c
            .ai_missions
            .iter()
            .filter(|m| m.side == side)
            .map(|m| m.pooled.iter().filter(|&&p| p).count())
            .sum();
        assert!(pooled <= tuning::AI_AIRCRAFT_PER_WINDOW, "{side:?}: {pooled}");
    }
    // Not every offer gets a flight; a recon flies alone, every other mission a pair.
    assert!(c.board.iter().filter(|o| ai_flown(o.kind)).any(|o| c.ai_mission_for(o.id).is_none()));
    for m in &c.ai_missions {
        let n = if m.offer.kind == MissionKind::Recon { 1 } else { 2 };
        assert_eq!(m.tags.len(), n, "{:?}", m.offer.kind);
    }
}

#[test]
fn the_ai_leaves_an_offer_a_player_flies_and_flies_one_a_player_only_plans() {
    let mut c = ai_campaign();
    let (flown, _) = take(&mut c, P0, MissionKind::Recon);
    let planned = c.board.iter().find(|o| o.kind == MissionKind::Sead).expect("sead").clone();
    c.apply(PlayerId(1), Command::Select { offer: Some(planned.id), base: planned.base }).unwrap();
    for _ in 0..7 {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    assert!(c.ai_mission_for(flown).is_none(), "a player flies it");
    assert!(c.ai_mission_for(planned.id).is_some(), "planning does not keep the AI home");
}

#[test]
fn without_the_rule_nobody_flies_the_boards() {
    let mut c = campaign("central_europe", 3);
    c.rules = Some(CampaignRules { no_ai_missions: true, ..Default::default() });
    for _ in 0..8 {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    assert!(c.ai_missions.is_empty());
    assert!(c.red_board.is_empty());
}

#[test]
fn the_ai_picks_the_aircraft_for_the_job() {
    use Aircraft::*;
    let heavy = tuning::HEAVY_SAM_THREAT;
    let fighters = tuning::HEAVY_AIR_THREAT;
    // Heavy air defences: the stealthy ones, the Tornado low against the SAMs.
    assert_eq!(pick_blue(MissionKind::Strike, heavy, 0, false), F117);
    assert_eq!(pick_blue(MissionKind::Recon, heavy, 0, false), F117);
    assert_eq!(pick_blue(MissionKind::Sead, heavy, 0, false), Tornado);
    assert_eq!(pick_blue(MissionKind::Sead, heavy, fighters, false), F22);
    // The enemy's fighters up: the fighters.
    assert_eq!(pick_blue(MissionKind::Strike, 0, fighters, false), F16);
    assert_eq!(pick_blue(MissionKind::Cas, 0, fighters, false), AV8B);
    assert_eq!(pick_blue(MissionKind::Cap, 0, fighters, false), F22);
    assert_eq!(pick_blue(MissionKind::Cap, 0, 0, false), F16);
    // Close to the base: the helicopters.
    assert_eq!(pick_blue(MissionKind::Recon, 0, 0, true), OH58);
    assert_eq!(pick_blue(MissionKind::Cas, 0, 0, true), AH1Z);
    assert_eq!(pick_blue(MissionKind::Cas, heavy, 0, true), RAH66);
    // Else the one made for it.
    assert_eq!(pick_blue(MissionKind::Cas, 0, 0, false), A10);
    assert_eq!(pick_red(MissionKind::Cas, true), RedJob::Helicopter);
    assert_eq!(pick_red(MissionKind::Strike, false), RedJob::Strike);
}

#[test]
fn the_enemy_flies_its_own_board_and_strikes_what_it_scouted() {
    let mut c = ai_campaign();
    let recon =
        c.red_board.iter().find(|o| o.kind == MissionKind::Recon).expect("red recon").clone();
    assert!(!c.red_board.iter().any(|o| o.kind == MissionKind::Strike), "nothing scouted yet");
    let mut ev = Vec::new();
    for _ in 0..6 {
        ev.extend(c.tick(60.0, &WorldView::default(), &[]));
    }
    assert!(ai_launched(&ev).iter().any(|(s, _)| *s == Side::Red), "the enemy's AI flies");
    let m = c.ai_mission_for(recon.id).expect("red recon flight").clone();
    assert_eq!(m.side, Side::Red);
    assert_eq!(m.airframe, AiAirframe::Red(RedJob::Recon));
    // Its photo: the place is known, a strike on its airfield goes on its board.
    let Objective::Recon { place, .. } = recon.objective else { unreachable!() };
    c.tick(1.0, &WorldView::default(), &[Report::Photographed { tag: m.tags[0] }]);
    assert!(c.places[place as usize].red_intel(c.clock));
    c.tick(1.0, &WorldView::default(), &[]);
    if c.places[place as usize].airfield.is_some() {
        assert!(
            c.red_board
                .iter()
                .any(|o| matches!(o.objective, Objective::Strike { place: p, .. } if p == place))
        );
    }
}

#[test]
fn a_friendly_recon_flight_brings_the_intelligence() {
    let mut c = ai_campaign();
    let recon = c.board.iter().find(|o| o.kind == MissionKind::Recon).expect("recon").clone();
    let Objective::Recon { place, .. } = recon.objective else { unreachable!() };
    for _ in 0..6 {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    let tag = c.ai_mission_for(recon.id).expect("launched").tags[0];
    let ev = c.tick(1.0, &WorldView::default(), &[Report::Photographed { tag }]);
    assert!(c.places[place as usize].intel(c.clock));
    assert!(c.ai_mission_for(recon.id).unwrap().achieved);
    assert!(ev.contains(&CampaignEvent::Recall { tag }), "it flies home");
    // Back on the ground: the flight is over, the aircraft back with the side.
    let (fighters, helicopters) = (c.blue.fighters, c.blue.helicopters);
    c.ai_mission_report(&Report::FlightLanded { tag, x: 0, y: 0 }, &mut Vec::new());
    let heli = c.ai_mission_for(recon.id).unwrap().airframe.helicopter();
    assert_eq!(
        (c.blue.fighters, c.blue.helicopters),
        (fighters + !heli as u32, helicopters + heli as u32)
    );
    c.tick(1.0, &WorldView::default(), &[]);
    assert!(!c.ai_mission_for(recon.id).unwrap().open());
}

#[test]
fn a_player_takes_over_an_ai_aircraft_and_hands_it_back() {
    let mut c = ai_campaign();
    for _ in 0..6 {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    let m =
        c.ai_missions.iter().find(|m| m.side == Side::Blue && m.open()).expect("flight").clone();
    let tag = m.takeable().expect("an aircraft to take");
    let ev = c.apply(P0, Command::TakeControl { tag }).unwrap();
    let sortie = c.sortie(P0).expect("flying").clone();
    assert!(
        ev.iter().any(|e| matches!(e, CampaignEvent::ControlTaken { tag: t, .. } if *t == tag))
    );
    assert_eq!(sortie.offer.id, m.offer.id);
    assert_eq!(AiAirframe::Blue(sortie.aircraft), m.airframe);
    // Nobody else gets it, and the AI is not flying it.
    assert!(c.apply(PlayerId(1), Command::TakeControl { tag }).is_err());
    assert!(!c.ai_mission_for(m.offer.id).unwrap().flying().any(|t| t == tag));
    // Back to the AI: the same aircraft flies on, his sortie ends handed over.
    let ev = c.apply(P0, Command::LeaveAircraft).unwrap();
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ControlLeft { tag: t, .. } if *t == tag)));
    let record = finished(&c.apply(P0, Command::EndSortie { end: SortieEnd::HandedOver }).unwrap());
    assert_eq!(record.end, SortieEnd::HandedOver);
    assert!(c.sortie(P0).is_none());
    assert!(c.ai_mission_for(m.offer.id).unwrap().flying().any(|t| t == tag));
}

#[test]
fn a_player_leaves_his_own_aircraft_to_the_ai() {
    let mut c = ai_campaign();
    let (offer, _) = take(&mut c, P0, MissionKind::Strike);
    let ev = c.apply(P0, Command::LeaveAircraft).unwrap();
    let tag = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::ControlLeft { tag, aircraft, .. } => {
                assert_eq!(*aircraft, Aircraft::F16);
                Some(*tag)
            }
            _ => None,
        })
        .expect("left");
    c.apply(P0, Command::EndSortie { end: SortieEnd::HandedOver }).unwrap();
    let m = c.ai_mission_for(offer).expect("the AI flies it on");
    assert_eq!(m.tags, vec![tag]);
    assert_eq!(m.pooled, vec![false], "his own aircraft, not the side's");
    // A crash in a taken-over aircraft: it is gone from its flight.
    let tag = m.takeable().unwrap();
    c.apply(P0, Command::TakeControl { tag }).unwrap();
    c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap();
    let m = c.ai_mission_for(offer).unwrap();
    assert!(m.alive.iter().all(|a| !a));
}

#[test]
fn a_loaded_save_has_no_ai_flight_in_the_air() {
    let mut c = ai_campaign();
    for _ in 0..6 {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    assert!(c.ai_missions.iter().any(AiMission::open));
    let (fighters, helicopters) = (c.blue.fighters, c.blue.helicopters);
    c.after_load();
    assert!(!c.ai_missions.iter().any(AiMission::open));
    assert!(c.blue.fighters + c.blue.helicopters > fighters + helicopters, "back on the ground");
    // Their offers go up again after the wait (in their turn).
    let queue = c.ai_schedule(Side::Blue);
    assert!(!queue.is_empty());
    for (id, t) in queue {
        if c.ai_mission_for(id).is_some() {
            assert!(t >= c.clock + tuning::AI_MISSION_WAIT, "{t}");
        }
    }
}

#[test]
fn the_ais_offers_take_off_in_turn() {
    let c = ai_campaign();
    for side in [Side::Blue, Side::Red] {
        let queue = c.ai_schedule(side);
        assert!(!queue.is_empty(), "{side:?}");
        assert!(queue.len() <= tuning::AI_MISSIONS_MAX);
        // Each its own turn, a flight's aircraft one after the other.
        assert!(queue.windows(2).all(|w| w[1].1 - w[0].1 >= tuning::AI_STAGGER), "{queue:?}");
        let board = if side == Side::Blue { &c.board } else { &c.red_board };
        let aircraft: usize = queue
            .iter()
            .map(|(id, _)| board.iter().find(|o| o.id == *id).unwrap())
            .map(|o| if o.kind == MissionKind::Recon { 1 } else { 2 })
            .sum();
        // More than the quarter hour's aircraft: the last ones wait for it.
        if aircraft > tuning::AI_AIRCRAFT_PER_WINDOW {
            let (first, last) = (queue[0].1, queue.last().unwrap().1);
            assert!(last >= first + tuning::AI_LAUNCH_WINDOW, "{queue:?}");
        }
    }
}

#[test]
fn a_sead_offer_the_ai_does_not_fly_yet_goes_at_once_on_request() {
    let mut c = ai_campaign();
    let sead: Vec<u32> =
        c.board.iter().filter(|o| o.kind == MissionKind::Sead).map(|o| o.id).collect();
    let id = sead[0];
    // One the AI flies on its turn: nothing to ask for.
    let queued = c.board.iter().find(|o| sead.contains(&o.id) && c.ai_scheduled(o)).cloned();
    let queued = queued.expect("a SEAD offer in the queue");
    assert!(c.apply(P0, Command::RequestSupport { offer: queued.id }).is_err());
    // Last in the queue, after the flights the AI may have out.
    c.board.iter_mut().find(|o| o.id == id).unwrap().posted = c.clock + 1e6;
    let o = c.board.iter().find(|o| o.id == id).unwrap().clone();
    assert!(
        c.board.iter().filter(|o| ai_missions::ai_flown(o.kind)).count() > tuning::AI_MISSIONS_MAX
    );
    assert!(!c.ai_scheduled(&o), "not in the queue");
    let materiel = c.blue.materiel;
    c.apply(P0, Command::RequestSupport { offer: id }).expect("at once");
    assert!(c.ai_scheduled(&o));
    assert!(c.apply(P0, Command::RequestSupport { offer: id }).is_err(), "twice");
    assert!(c.raids.iter().all(|r| r.side != Side::Blue), "no raid of its own");
    c.tick(1.0, &WorldView::default(), &[]);
    let m = c.ai_mission_for(id).expect("the AI flies it");
    assert!(m.open());
    assert_eq!(c.blue.materiel, materiel - tuning::AI_MISSION_COST);
}

/// A base of `side` other than its production base.
fn other_base(c: &CampaignState, side: Side) -> ObjectId {
    let p = c.production_base(side).expect("production base");
    c.stocks
        .iter()
        .find(|(b, s)| **b != p && s.side == Some(side) && c.site(**b).is_some_and(Site::alive))
        .map(|(b, _)| *b)
        .expect("another base")
}

/// Empties a base's stock of `m`.
fn run_out(c: &mut CampaignState, base: ObjectId, m: Munition) {
    let n = c.available(base, m);
    c.draw(base, m, n);
    assert_eq!(c.available(base, m), 0);
}

#[test]
fn the_war_starts_with_every_airfield_stocked_and_a_production_base_in_the_rear() {
    let c = ai_campaign();
    for side in [Side::Blue, Side::Red] {
        let p = c.production_base(side).expect("a production base");
        assert_eq!(c.owner(p), Some(side));
        assert_eq!(c.site(p).unwrap().kind, SiteKind::Airfield);
        // The furthest of the side's airfields from the other side's places.
        let rear = |b: ObjectId| {
            let s = c.site(b).unwrap();
            c.places
                .iter()
                .filter(|q| q.owner == side.enemy())
                .map(|q| dist(q.x - s.x, q.y - s.y))
                .min()
                .unwrap()
        };
        for (&b, s) in c.stocks.iter().filter(|(_, s)| s.side == Some(side)) {
            assert!(rear(b) <= rear(p), "{b} is further back than {p}");
            for &m in Munition::of_side(side) {
                // The production base has already made a little.
                let full = m.full() as u32;
                assert!(s.count(m) == full || b == p && s.count(m) >= full, "{m:?} at {b}");
            }
        }
    }
    // The Sidewinders and iron bombs never run out.
    let b = other_base(&c, Side::Blue);
    assert_eq!(c.available(b, Munition::of_weapon(1).unwrap()), 32);
    assert!(Munition::of_weapon(0).is_none() && Munition::of_weapon(9).is_none());
    assert!(
        Munition::of_spec(17, Side::Red).is_none() && Munition::of_spec(23, Side::Blue).is_none()
    );
}

#[test]
fn the_production_base_makes_simple_munitions_faster_than_complex_ones() {
    let mut c = ai_campaign();
    let p = c.production_base(Side::Blue).unwrap();
    run_out(&mut c, p, Munition::Hellfire);
    run_out(&mut c, p, Munition::Harpoon);
    for _ in 0..60 {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    let (hellfire, harpoon) =
        (c.available(p, Munition::Hellfire), c.available(p, Munition::Harpoon));
    assert!(hellfire >= 18 && (2..=4).contains(&harpoon), "{hellfire} {harpoon}");
    // Up to three full stocks.
    for _ in 0..24 * 6 {
        c.tick(600.0, &WorldView::default(), &[]);
    }
    assert_eq!(c.available(p, Munition::Hellfire), 3 * 32);
}

#[test]
fn a_base_out_of_a_munition_gets_one_cargo_flight_and_the_enemy_goes_for_it() {
    let mut c = ai_campaign();
    let b = other_base(&c, Side::Blue);
    let p = c.production_base(Side::Blue).unwrap();
    run_out(&mut c, b, Munition::Aim120);
    let before = c.available(p, Munition::Aim120);
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert!(
        ev.iter().any(|e| matches!(e, CampaignEvent::Message(t) if t.contains("Cargo flight")))
    );
    assert_eq!(c.cargo_runs.len(), 1);
    let r = c.cargo_runs[0].clone();
    assert_eq!((r.from, r.to, r.side), (p, b, Side::Blue));
    assert_eq!(r.launch_at(), Some(r.posted + tuning::CARGO_WAIT));
    // Half a full stock of what it lacks, out of the production base's now.
    assert!(r.load.contains(&(Munition::Aim120, 16)), "{:?}", r.load);
    assert_eq!(c.available(p, Munition::Aim120), before - 16);
    c.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(c.cargo_runs.len(), 1, "one at a time");
    // The enemy's interceptors take off two minutes after it was scheduled.
    let o = c
        .red_board
        .iter()
        .find(|o| o.objective == Objective::Cargo { tag: r.tag })
        .expect("an intercept on the enemy's board")
        .clone();
    assert_eq!(o.kind, MissionKind::Intercept);
    assert_eq!(c.ai_launch_at(&o), Some(r.posted + tuning::CARGO_INTERCEPT_WAIT));
    let mut launched = None;
    let mut hunters = None;
    for _ in 0..700 {
        for e in c.tick(1.0, &WorldView::default(), &[]) {
            match e {
                CampaignEvent::CargoPlaneLaunched { tag, from, to, .. } => {
                    assert_eq!((tag, from, to), (r.tag, p, b));
                    launched = Some(c.clock);
                }
                CampaignEvent::AiMissionLaunched {
                    side: Side::Red,
                    target: AiTarget::Hunt { tag, x, y },
                    ..
                } => {
                    assert_eq!((tag, x, y), (r.tag, o.x, o.y));
                    hunters.get_or_insert(c.clock);
                }
                _ => {}
            }
        }
    }
    let (launched, hunters) = (launched.expect("it took off"), hunters.expect("hunters"));
    assert!((launched - (r.posted + tuning::CARGO_WAIT)).abs() <= 1.0);
    assert!(hunters - r.posted >= tuning::CARGO_INTERCEPT_WAIT && hunters - r.posted < 200.0);
    assert_eq!(c.cargo_runs[0].state, CargoState::Airborne);
    // It lands: the load is the base's.
    let s = c.site(b).unwrap().clone();
    let land = Report::FlightLanded { tag: r.tag, x: s.x, y: s.y };
    let ev = c.tick(1.0, &WorldView::default(), &[land]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(t) if t.contains("delivered"))));
    assert_eq!(c.available(b, Munition::Aim120), 16);
    assert!(c.cargo_runs.iter().all(|x| x.tag != r.tag));
    // The hunters are called home.
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Recall { .. })));
}

#[test]
fn a_cargo_aircraft_shot_down_loses_its_load() {
    let mut c = ai_campaign();
    // The enemy's: the friendly board has the intercept.
    let b = other_base(&c, Side::Red);
    run_out(&mut c, b, Munition::SamLong);
    c.tick(1.0, &WorldView::default(), &[]);
    let r = c.cargo_runs.iter().find(|r| r.side == Side::Red).expect("a cargo flight").clone();
    let o = c.board.iter().find(|o| o.objective == Objective::Cargo { tag: r.tag }).cloned();
    let o = o.expect("an intercept on the board");
    assert!(o.title.contains("cargo"), "{}", o.title.en());
    c.tick(tuning::CARGO_WAIT, &WorldView::default(), &[]);
    let s = c.site(b).unwrap();
    let down =
        Report::FlightDestroyed { tag: r.tag, side: Side::Red, x: s.x, y: s.y, by: Some(P0) };
    let ev = c.tick(1.0, &WorldView::default(), &[down]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(_))));
    assert_eq!(c.available(b, Munition::SamLong), 0);
    assert!(c.objective_met(&o.objective, &Progress::default()));
    c.tick(tuning::BOARD_REFRESH, &WorldView::default(), &[]);
    assert!(c.board.iter().all(|x| x.objective != o.objective));
    // The next one waits out the cooldown.
    assert!(c.cargo_runs.iter().all(|x| x.to != b), "none right after the loss");
    for _ in 0..(tuning::CARGO_LOST_COOLDOWN / 60.0) as usize {
        c.tick(60.0, &WorldView::default(), &[]);
    }
    assert!(c.cargo_runs.iter().any(|x| x.to == b), "another after the cooldown");
}

#[test]
fn a_cargo_flight_saved_in_the_air_takes_off_again_with_its_load() {
    let mut c = ai_campaign();
    let b = other_base(&c, Side::Blue);
    run_out(&mut c, b, Munition::Harm);
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(tuning::CARGO_WAIT, &WorldView::default(), &[]);
    let r = c.cargo_runs[0].clone();
    assert_eq!(r.state, CargoState::Airborne);
    let json = serde_json::to_string(&c).unwrap();
    let mut back: CampaignState = serde_json::from_str(&json).unwrap();
    back.after_load();
    let again = &back.cargo_runs[0];
    assert_eq!(again.load, r.load);
    assert_eq!(again.launch_at(), Some(back.clock + tuning::CARGO_RELAUNCH));
    let ev = back.tick(tuning::CARGO_RELAUNCH, &WorldView::default(), &[]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::CargoPlaneLaunched { .. })));
    // A save from before the munitions loads, and stocks up.
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for k in ["stocks", "production", "cargo_runs", "cargo_lost", "site_magazines"] {
        v.as_object_mut().unwrap().remove(k);
    }
    let mut old: CampaignState = serde_json::from_value(v).unwrap();
    old.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(old.available(b, Munition::Harm), Munition::Harm.full() as u32);
}

#[test]
fn an_empty_base_falls_back_to_the_next_munition() {
    let mut c = ai_campaign();
    let b = other_base(&c, Side::Blue);
    run_out(&mut c, b, Munition::Aim120);
    assert_eq!(c.draw_with_fallback(b, Munition::Aim120, 4), vec![(Munition::Aim7, 4)]);
    run_out(&mut c, b, Munition::Aim7);
    assert!(c.draw_with_fallback(b, Munition::Aim120, 4).is_empty(), "AIM-9s then");
    run_out(&mut c, b, Munition::Harm);
    assert_eq!(c.draw_with_fallback(b, Munition::Harm, 1), vec![(Munition::Agm65, 1)]);
    // Without the munitions (an older test state) nothing runs out.
    let mut plain = campaign("central_europe", 3);
    assert_eq!(plain.draw(b, Munition::Aim120, 1000), 1000);
}

#[test]
fn the_enemy_stocks_sam_missiles_where_its_sites_need_them() {
    let mut c = campaign("central_europe", 3);
    c.rules = Some(CampaignRules::default());
    let reds: Vec<ObjectId> = c.bases(Side::Red);
    let (a, b) = (reds[0], reds[1]);
    // `a` supplies two long range sites and a gun, `b` nothing.
    c.site_demand.insert(a, [2, 0, 0, 1]);
    c.site_demand.insert(b, [0; 4]);
    c.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(c.full_stock(a, Munition::SamLong), 2.0 * tuning::SAM_STOCK_PER_SITE);
    assert_eq!(c.available(a, Munition::SamLong), 16);
    assert_eq!(c.available(b, Munition::SamLong), 0);
    // A base without sites is not out of their missiles; one with them is when they are gone.
    assert!(c.out_of(b, Side::Red).is_empty());
    run_out(&mut c, a, Munition::SamLong);
    assert_eq!(c.out_of(a, Side::Red), vec![Munition::SamLong]);
    // The production base makes enough for the theater's sites.
    assert_eq!(c.production_rate(Munition::SamLong), 2.0 * tuning::SAM_LONG_PER_SITE_HOUR);
    assert_eq!(c.production_rate(Munition::Aa11), Munition::Aa11.per_hour());
}

/// `cargo test -p stealth-campaign theater_survey -- --ignored --nocapture`: each theater's
/// bases and production bases at the start.
#[test]
#[ignore]
fn theater_survey() {
    for code in [
        "central_europe",
        "cuba",
        "desert_storm",
        "korea",
        "libya",
        "middle_east",
        "north_cape",
        "persian_gulf",
        "ukraine",
        "vietnam",
    ] {
        let mut c = campaign(code, 0);
        c.rules = Some(CampaignRules::default());
        c.tick(1.0, &WorldView::default(), &[]);
        for side in [Side::Blue, Side::Red] {
            let kinds = |k: SiteKind| {
                c.sites
                    .iter()
                    .filter(|s| s.kind == k && !s.removed && c.owner(s.object) == Some(side))
                    .map(|s| s.name.clone())
                    .collect::<Vec<_>>()
            };
            let sams = c
                .sites
                .iter()
                .filter(|s| s.kind == SiteKind::Sam && c.owner(s.object) == Some(side))
                .count();
            let p = c.production_base(side).and_then(|b| c.site(b)).map(|s| s.name.clone());
            println!(
                "{code:15} {side:?}: airfields {:?} carriers {:?} sams {sams} production {p:?}",
                kinds(SiteKind::Airfield),
                kinds(SiteKind::Carrier)
            );
        }
    }
}

#[test]
fn a_side_left_with_carriers_takes_its_next_production_base() {
    let mut c = campaign("korea", 0);
    c.rules = Some(CampaignRules::default());
    c.tick(1.0, &WorldView::default(), &[]);
    let chunchon = c.production_base(Side::Blue).expect("Chunchon");
    // The enemy takes the only friendly airfield: the carriers fly on, never short of anything.
    let place = c.site(chunchon).unwrap().place.unwrap();
    let mut ev = Vec::new();
    c.capture_place(place, Side::Red, &mut ev);
    let ev = c.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(c.production_base(Side::Blue), None);
    let target = c.production_target(Side::Blue).expect("an airfield to take");
    assert_eq!(c.owner(target), Some(Side::Red));
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::Message(t) if t.contains("capture"))));
    let carrier = c.bases(Side::Blue)[0];
    assert_eq!(c.site(carrier).unwrap().kind, SiteKind::Carrier);
    assert_eq!(c.available(carrier, Munition::Aim120), u32::MAX);
    // Taken, it is the production base, with a full stock.
    let place = c.site(target).unwrap().place.unwrap();
    c.capture_place(place, Side::Blue, &mut Vec::new());
    c.tick(1.0, &WorldView::default(), &[]);
    assert_eq!(c.production_base(Side::Blue), Some(target));
    assert_eq!(c.production_target(Side::Blue), None);
    assert!(c.available(target, Munition::Aim120) >= 32);
}

/// A friendly cargo run waiting at the production base, its airlift offer on the board.
fn airlift_offer(c: &mut CampaignState, m: Munition) -> (ObjectId, ObjectId, MissionOffer) {
    let b = other_base(c, Side::Blue);
    let p = c.production_base(Side::Blue).unwrap();
    run_out(c, b, m);
    c.tick(1.0, &WorldView::default(), &[]);
    c.tick(1.0, &WorldView::default(), &[]);
    let o = c
        .board
        .iter()
        .find(|o| o.kind == MissionKind::Airlift)
        .expect("an airlift on the board")
        .clone();
    assert_eq!(o.base, p, "flown from the production base");
    (p, b, o)
}

#[test]
fn the_player_flies_a_cargo_run_in_the_cargo_jet() {
    let mut c = ai_campaign();
    let (p, b, o) = airlift_offer(&mut c, Munition::Aim120);
    // The cargo jet flies airlifts only, and only it does.
    let take = |c: &mut CampaignState, a: Aircraft| {
        c.apply(P0, Command::TakeMission { offer: o.id, base: None, aircraft: a })
    };
    assert!(take(&mut c, Aircraft::F16).is_err());
    let strike = c.board.iter().find(|x| x.kind == MissionKind::Strike).unwrap().id;
    assert!(
        c.apply(P0, Command::TakeMission { offer: strike, base: None, aircraft: Aircraft::C5 })
            .is_err()
    );
    let ev = take(&mut c, Aircraft::C5).expect("take");
    let s = ev
        .iter()
        .find_map(|e| match e {
            CampaignEvent::SortieStarted { sortie, .. } => Some(sortie.clone()),
            _ => None,
        })
        .expect("started");
    assert_eq!(s.base, p);
    let r = c.flown_cargo(P0).expect("his run").clone();
    assert_eq!(r.to, b);
    // The AI does not fly it meanwhile.
    for _ in 0..(tuning::CARGO_WAIT as usize + 60) {
        let ev = c.tick(1.0, &WorldView::default(), &[]);
        assert!(!ev.iter().any(|e| matches!(e, CampaignEvent::CargoPlaneLaunched { tag, .. } if *tag == r.tag)));
    }
    // A stop elsewhere delivers nothing; at its base it does.
    c.tick(1.0, &WorldView::default(), &[Report::PlayerLanded { player: P0, base: p }]);
    assert!(c.flown_cargo(P0).is_some());
    let ev = c.tick(1.0, &WorldView::default(), &[Report::PlayerLanded { player: P0, base: b }]);
    assert!(ev.iter().any(|e| matches!(e, CampaignEvent::ObjectiveAchieved { .. })), "{ev:?}");
    assert_eq!(c.available(b, Munition::Aim120), 16);
    assert!(c.cargo_runs.iter().all(|x| x.id != r.id));
    let ev = c.apply(P0, Command::EndSortie { end: SortieEnd::Landed }).unwrap();
    assert_eq!(finished(&ev).result, SortieResult::Success);
}

#[test]
fn a_cargo_run_the_player_crashes_with_is_lost_and_one_he_aborts_goes_back_to_the_ai() {
    let mut c = ai_campaign();
    let (_, b, o) = airlift_offer(&mut c, Munition::Harm);
    c.apply(P0, Command::TakeMission { offer: o.id, base: None, aircraft: Aircraft::C5 }).unwrap();
    let r = c.flown_cargo(P0).unwrap().clone();
    // Aborted: the AI's cargo aircraft flies it soon after.
    c.apply(P0, Command::EndSortie { end: SortieEnd::Aborted }).unwrap();
    let back = c.cargo_runs.iter().find(|x| x.id == r.id).expect("still there");
    assert_eq!(back.launch_at(), Some(c.clock + tuning::CARGO_RELAUNCH));
    // Taken again, and crashed: the load is lost, the enemy's intercept is met.
    c.tick(1.0, &WorldView::default(), &[]);
    let o = c.board.iter().find(|x| x.kind == MissionKind::Airlift).unwrap().clone();
    c.apply(P0, Command::TakeMission { offer: o.id, base: None, aircraft: Aircraft::C5 }).unwrap();
    c.apply(P0, Command::EndSortie { end: SortieEnd::Crashed }).unwrap();
    assert!(c.cargo_runs.iter().all(|x| x.id != r.id));
    assert!(c.objective_met(&Objective::Cargo { tag: r.tag }, &Progress::default()));
    assert_eq!(c.available(b, Munition::Harm), 0);
}
