//! A coop session over loopback: a dedicated-style server and a client with a mirror.

use std::time::{Duration, Instant};

use stealth_campaign::{Aircraft, CampaignSetup, PlayerId};
use stealth_host::{Host, HostEvent, Rules, theater};
use stealth_net::client::{Client, ClientEvent};
use stealth_net::protocol::{PlayerStatus, ToClient, ToServer, encode};
use stealth_net::server::Server;
use stealth_sim::{PlayerState, Sim};

struct World {
    host: Host,
    sim: Sim,
    server: Server,
    mirror: Sim,
}

fn world() -> World {
    let setup = CampaignSetup {
        theater: 3,
        theater_code: "central_europe".into(),
        roe: 1,
        opponents: 1,
        seed: 99,
        sky_ratio: 24.0,
        start_hour: 8.0,
        dynamic_weather: true,
        crew_scaling: true,
    };
    let data = theater::game_data().unwrap();
    let sim_data = theater::sim_data(3, "central_europe").unwrap();
    let mut host = Host::new(setup, &data, Rules::default()).unwrap();
    let mut sim = host.new_sim(&sim_data);
    host.begin(&mut sim);
    let mirror = host.new_sim(&sim_data);
    let mut server = Server::new("Test", 0, 8, None).unwrap();
    // Every frame: the mirror is compared with the server's simulation as of the same frame.
    server.send_every = 1;
    World { host, sim, server, mirror }
}

/// One server frame at `now`.
fn serve(w: &mut World, now: Instant) {
    let ev = w.server.pump(&mut w.host, &mut w.sim, now);
    let _ = ev;
    let inputs = w.server.inputs();
    let mut events = w.sim.step_pilots(&inputs, 15);

    events.extend(w.host.take_sim_events(&mut w.sim));
    w.server.after_step(&events);
    let hev = w.host.tick(&mut w.sim, 1.0 / 15.0, &events);
    w.server.route(&hev);
    w.server.send_frame(&w.host, &mut w.sim, now);
    std::thread::sleep(Duration::from_millis(1));
}

/// One server frame at `now` and the client's side of it.
fn frame(
    w: &mut World,
    c: &mut Client,
    input: Option<&PlayerState>,
    now: Instant,
) -> Vec<ClientEvent> {
    serve(w, now);
    if let Some(ps) = input {
        c.send_input(ps);
    }
    c.flush(now);
    let got = c.poll(now);
    for e in &got {
        match e {
            ClientEvent::Welcome { id, .. } => w.mirror.make_mirror(id.0),
            ClientEvent::Slow(s) => w.mirror.apply_world_slow(s),
            ClientEvent::Fast { world, tick, pilot, .. } => {
                w.mirror.apply_world_fast(world);
                if let Some(pilot) = pilot {
                    w.mirror.apply_pilot_snap(pilot);
                }
                w.mirror.apply_pilot_tick(tick);
            }
            _ => {}
        }
    }
    w.mirror.mirror_frame();
    got
}

#[test]
fn a_player_joins_flies_and_returns() {
    let mut w = world();
    let port = w.server.port().unwrap();
    let mut c = Client::connect(&format!("127.0.0.1:{port}"), "Maverick").unwrap();
    let mut now = Instant::now();
    let mut me = None;
    let mut sizes = (0usize, 0usize);
    for _ in 0..300 {
        now += Duration::from_millis(66);
        for e in frame(&mut w, &mut c, None, now) {
            if let ClientEvent::Welcome { id, state, .. } = e {
                assert_eq!(state.player_name(id), Some("Maverick"));
                me = Some(id);
            }
        }
        if me.is_some() {
            break;
        }
    }
    let me = me.expect("welcome");
    assert_eq!(me, PlayerId(1), "player 0 is the host's");
    // Wait for the board, then fly its first mission in an F-16.
    let mut start = None;
    for _ in 0..300 {
        now += Duration::from_millis(66);
        if start.is_none()
            && let Some(o) = w.host.state.board.iter().find(|o| !o.kind.ai_flown())
        {
            c.send(&ToServer::TakeMission {
                offer: o.id,
                base: None,
                aircraft: Aircraft::F16,
                stores: None,
            });
        }
        for e in frame(&mut w, &mut c, None, now) {
            if let ClientEvent::Host(HostEvent::SortieStarted { player, start: s, .. }) = e {
                assert_eq!(player, me);
                start = Some(s);
            }
        }
        if start.is_some() {
            break;
        }
    }
    let mut ps = start.expect("sortie started");
    ps.on_ground = false;
    ps.altitude = 2000;
    ps.height = ps.altitude;
    ps.velocity = 300 * 27;
    ps.gear_up = true;
    let mut roster = None;
    for _ in 0..60 {
        now += Duration::from_millis(66);
        ps.y += 300;
        for e in frame(&mut w, &mut c, Some(&ps), now) {
            if let ClientEvent::Roster(r) = e {
                roster = Some(r);
            }
        }
    }
    assert!(!w.sim.pilot_absent(me.0));
    assert_eq!(w.sim.pilot_view(me.0).map(|p| p.alt), Some(2000), "the server flies his reports");
    assert!(!w.mirror.player_absent(), "the mirror knows he flies");
    let live =
        |s: &Sim| -> Vec<_> { s.flights().iter().filter(|u| u.flags != 0).copied().collect() };
    assert_eq!(live(&w.mirror), live(&w.sim));
    let r = roster.expect("roster");
    assert!(matches!(r[0].status, PlayerStatus::Flying { aircraft: Aircraft::F16, .. }), "{r:?}");
    // The server list's live map: where he flies, and the campaign's places.
    let report = stealth_net::master::report(&w.server, &w.host, &w.sim, true);
    let p = report.players.iter().find(|p| p.name == "Maverick").expect("on the report");
    let at = w.sim.pilot_view(me.0).unwrap();
    assert_eq!(p.pos.map(|p| (p.x, p.y, p.alt)), Some((at.mx, at.my, 2000)));
    let map = report.map.expect("map");
    assert_eq!(map.theater, "central_europe");
    assert_eq!(map.places.len(), w.host.state.places.len());
    let fast = encode(&ToClient::Fast {
        step: 0,
        world: w.sim.world_fast(),
        tick: w.sim.pilot_tick(me.0),
        pilot: Some(w.sim.pilot_snap(me.0)),
    });
    sizes.0 = fast.len();
    let slow = w.sim.world_slow(&mut Default::default());
    sizes.1 = encode(&ToClient::Slow(slow)).len();
    eprintln!("frame {} bytes, join {} bytes", sizes.0, sizes.1);
    assert!(sizes.0 < 16_000);
    // He drops out (no more packets): the aircraft waits, then a new connection under his name
    // takes it back.
    drop(c);
    for _ in 0..10 {
        now += Duration::from_secs(2);
        w.server.pump(&mut w.host, &mut w.sim, now);
    }
    assert!(!w.sim.pilot_absent(me.0), "the aircraft waits for him");
    let mut c = Client::connect(&format!("127.0.0.1:{port}"), "maverick").unwrap();
    let mut resumed = None;
    for _ in 0..300 {
        now += Duration::from_millis(66);
        for e in frame(&mut w, &mut c, None, now) {
            if let ClientEvent::Welcome { id, resume, .. } = e {
                assert_eq!(id, me);
                resumed = Some(resume.is_some());
            }
        }
        if resumed.is_some() {
            break;
        }
    }
    assert_eq!(resumed, Some(true));
    // Gone for good: after the grace period the sortie ends.
    drop(c);
    for _ in 0..30 {
        now += Duration::from_secs(2);
        w.server.pump(&mut w.host, &mut w.sim, now);
    }
    assert!(w.sim.pilot_absent(me.0));
    assert!(w.host.state.sortie(me).is_none());
}

#[test]
fn a_full_campaign_is_saved_and_resumed() {
    let w = world();
    let dir = std::env::temp_dir().join(format!("sm89-net-{}", std::process::id()));
    let path = dir.join("dedicated.json");
    stealth_host::save::save_to(&path, &w.host.state).unwrap();
    let back = stealth_host::save::load_from(&path).unwrap().unwrap();
    assert_eq!(back, w.host.state);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn sixty_four_pilots_fit_in_a_frame() {
    let mut w = world();
    let sim = &mut w.sim;
    let base = (3..sim.objects().len())
        .find(|&i| sim.objects()[i].flags & 0xF0F == 0x401)
        .expect("friendly base");
    let b = sim.objects()[base];
    let mut inputs = Vec::new();
    for id in 0..64u8 {
        let airframe =
            stealth_sim::types::Airframe { external_stores: id % 2 == 0, ..Default::default() };
        sim.respawn_pilot(id, base, None, airframe);
        // Half of them together near the base, the others spread out far away.
        let (dx, dy) = if id < 32 { (id as i32 * 8, 0) } else { (id as i32 * 0x300, 0x2000) };
        let (x, y) = Sim::map_to_world(b.x + dx, b.y - 0x400 - dy);
        let ps = PlayerState {
            x,
            y,
            altitude: 3000 + id as u16 * 50,
            height: 3000 + id as u16 * 50,
            heading: id as u16 * 1000,
            velocity: 400 * 27,
            gear_up: true,
            fuel: 10000,
            ..Default::default()
        };
        inputs.push((id, ps));
    }
    for _ in 0..4 {
        for (_, ps) in inputs.iter_mut() {
            ps.y += 200;
        }
        sim.step_pilots(&inputs, 15);
    }
    let world = sim.world_fast();
    assert_eq!(world.pilots.len(), 64);
    let size = |pilots: usize| {
        let mut world = world.clone();
        world.pilots.truncate(pilots);
        encode(&ToClient::Fast { step: 0, world, tick: sim.pilot_tick(1), pilot: None }).len()
    };
    let (none, all) = (size(0), size(64));
    eprintln!("frame without pilots {none} bytes, with 64 {all} bytes");
    assert!((all - none) / 64 < 60, "a pilot costs {} bytes", (all - none) / 64);
    assert!(all < 6_000);
}

/// Server frames until `done` holds for the clients' events (each client's, every frame).
fn until(
    w: &mut World,
    clients: &mut [&mut Client],
    now: &mut Instant,
    mut done: impl FnMut(usize, &ClientEvent) -> bool,
) -> bool {
    for _ in 0..300 {
        *now += Duration::from_millis(66);
        serve(w, *now);
        let mut hit = false;
        for (k, c) in clients.iter_mut().enumerate() {
            c.flush(*now);
            for e in c.poll(*now) {
                hit |= done(k, &e);
            }
        }
        if hit {
            return true;
        }
    }
    false
}

#[test]
fn players_plan_a_mission_together() {
    use stealth_campaign::{CampaignEvent, MissionPlan, RouteKind, RoutePoint};
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut a = Client::connect(&addr, "Alpha").unwrap();
    let mut b = Client::connect(&addr, "Bravo").unwrap();
    let mut now = Instant::now();
    let mut ids = [None, None];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Welcome { id, .. } = e {
            ids[k] = Some(*id);
        }
        ids.iter().all(Option::is_some)
    }));
    let [pa, pb] = ids.map(Option::unwrap);
    let o = w.host.state.board.iter().find(|o| !o.kind.ai_flown()).unwrap().clone();
    let plans = |k: usize, e: &ClientEvent| match e {
        ClientEvent::Host(HostEvent::Campaign(CampaignEvent::PlansChanged(p))) if k == 1 => {
            p.iter().find(|p| p.offer == o.id).cloned()
        }
        _ => None,
    };
    // Both plan it: Bravo sees Alpha's route at once.
    a.send(&ToServer::Select { offer: Some(o.id), base: o.base });
    b.send(&ToServer::Select { offer: Some(o.id), base: o.base });
    let mut seen: Option<MissionPlan> = None;
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        seen = plans(k, e).or(seen.take());
        seen.as_ref().is_some_and(|p| p.crew.len() == 2)
    }));
    let route = vec![
        RoutePoint { x: 0x2000, y: 0x2000, kind: RouteKind::Nav },
        RoutePoint { x: o.x, y: o.y, kind: RouteKind::Primary },
        RoutePoint { x: o.x, y: o.y, kind: RouteKind::Secondary },
        RoutePoint { x: 0x3000, y: 0x3000, kind: RouteKind::Land },
    ];
    a.send(&ToServer::Plan { offer: o.id, base: o.base, route: route.clone() });
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        plans(k, e).is_some_and(|p| p.route == route)
    }));
    // Alpha flies it: locked for Bravo.
    a.send(&ToServer::TakeMission {
        offer: o.id,
        base: None,
        aircraft: Aircraft::F117,
        stores: None,
    });
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        plans(k, e).is_some_and(|p| p.flying == vec![pa] && p.crew == vec![pb])
    }));
    b.send(&ToServer::Plan { offer: o.id, base: o.base, route: Vec::new() });
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        k == 1 && matches!(e, ClientEvent::Error(_))
    }));
    // He aborts: editable again, the route kept.
    a.send(&ToServer::EndSortie { outcome: 7 });
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        plans(k, e).is_some_and(|p| p.flying.is_empty() && p.route == route)
    }));
    b.send(&ToServer::Plan { offer: o.id, base: o.base, route: Vec::new() });
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        plans(k, e).is_some_and(|p| p.route.is_empty())
    }));
    // Bravo leaves: off the plan.
    b.leave();
    for _ in 0..5 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
    }
    assert!(w.host.state.plan(o.id).unwrap().crew.is_empty());
}

/// A chat message stays with the campaign: a player joining later finds it in his welcome.
#[test]
fn chat_is_kept_with_the_campaign() {
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let join = |w: &mut World, c: &mut Client, now: &mut Instant| {
        for _ in 0..300 {
            *now += Duration::from_millis(66);
            for e in frame(w, c, None, *now) {
                if let ClientEvent::Welcome { state, .. } = e {
                    return state;
                }
            }
        }
        panic!("no welcome");
    };
    let mut now = Instant::now();
    let mut first = Client::connect(&addr, "Maverick").unwrap();
    join(&mut w, &mut first, &mut now);
    first.chat("Wheels up");
    for _ in 0..30 {
        now += Duration::from_millis(66);
        frame(&mut w, &mut first, None, now);
    }
    let kept: Vec<_> = w.host.state.chat.iter().map(|m| (&*m.from, &*m.text)).collect();
    assert_eq!(kept, [("Maverick", "Wheels up")]);
    let mut second = Client::connect(&addr, "Goose").unwrap();
    let state = join(&mut w, &mut second, &mut now);
    assert_eq!(state.chat.len(), 1);
    assert_eq!(state.chat[0].text, "Wheels up");
}

/// A radio transmission goes to the other players, not back to its sender, and a flood of
/// packets is cut to the rate limit.
#[test]
fn voice_is_relayed_and_rate_limited() {
    use stealth_net::voice::VoicePacket;
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut now = Instant::now();
    let mut a = Client::connect(&addr, "Maverick").unwrap();
    let mut b = Client::connect(&addr, "Goose").unwrap();
    let mut welcomed = [false; 2];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        welcomed[k] |= matches!(e, ClientEvent::Welcome { .. });
        welcomed == [true, true]
    }));
    let packet = |seq| VoicePacket { seq, end: false, data: vec![0; 249] };
    // 200 packets at once: the burst allowance gets through.
    for seq in 0..200 {
        a.voice(packet(seq));
    }
    let mut heard = [0usize; 2];
    until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Voice { from, packet } = e {
            assert_eq!(*from, PlayerId(1));
            assert_eq!(packet.data.len(), 249);
            heard[k] += 1;
        }
        false
    });
    assert_eq!(heard[0], 0, "the sender doesn't hear himself");
    assert!((30..=50).contains(&heard[1]), "{} relayed", heard[1]);
    // Too long a packet is dropped.
    a.voice(VoicePacket { seq: 0, end: true, data: vec![0; 2000] });
    assert!(!until(&mut w, &mut [&mut a, &mut b], &mut now, |_, e| {
        matches!(e, ClientEvent::Voice { .. })
    }));
}

/// Two players go for the same AI aircraft: one takes it over in the air, the other is told
/// no; he leaves it to the AI again and it flies on.
#[test]
fn a_coop_player_takes_over_an_ai_aircraft_and_hands_it_back() {
    use stealth_campaign::SortieEnd;
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut a = Client::connect(&addr, "Alpha").unwrap();
    let mut b = Client::connect(&addr, "Bravo").unwrap();
    let mut now = Instant::now();
    let mut ids = [None, None];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Welcome { id, .. } = e {
            ids[k] = Some(*id);
        }
        ids.iter().all(Option::is_some)
    }));
    // The board's offers are due for the AI: its flights take off and get up and away.
    let wait = stealth_campaign::tuning::AI_MISSION_WAIT;
    let clock = w.host.state.clock;
    for o in &mut w.host.state.board {
        o.posted = clock - wait;
    }
    let mut tag = None;
    for _ in 0..15 * 240 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        for c in [&mut a, &mut b] {
            c.flush(now);
            c.poll(now);
        }
        let st = &w.host.state;
        tag = st.ai_missions.iter().filter(|m| m.open()).flat_map(|m| m.flying()).find(|&t| {
            w.sim.takeable(t) && w.sim.flights()[w.sim.flight_of_tag(t).unwrap()].alt > 300
        });
        if tag.is_some() {
            break;
        }
    }
    let tag = tag.expect("an AI aircraft up and away");
    let at = w.sim.flights()[w.sim.flight_of_tag(tag).unwrap()];
    a.send(&ToServer::TakeControl { tag, stores: None, seat: Default::default() });
    b.send(&ToServer::TakeControl { tag, stores: None, seat: Default::default() });
    let mut started = [None, None];
    let mut refused = [false, false];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        match e {
            ClientEvent::Host(HostEvent::SortieStarted { start, .. }) => {
                started[k] = Some(start.clone())
            }
            ClientEvent::Error(_) => refused[k] = true,
            _ => {}
        }
        started.iter().filter(|s| s.is_some()).count() + refused.iter().filter(|&&r| r).count() == 2
    }));
    let k = started.iter().position(Option::is_some).expect("one of them took it");
    assert!(refused[1 - k] && started[1 - k].is_none(), "the other was told no");
    let me = ids[k].unwrap();
    let start = started[k].clone().unwrap();
    assert!(!start.on_ground);
    assert_eq!(start.x, at.wx, "where the AI's aircraft was");
    assert!(w.sim.flight_of_tag(tag).is_none() && !w.sim.pilot_absent(me.0));
    // He flies it a moment, then leaves it to the AI.
    let c = if k == 0 { &mut a } else { &mut b };
    for _ in 0..15 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        c.send_input(&start);
        c.flush(now);
        c.poll(now);
    }
    c.send(&ToServer::LeaveAircraft);
    let mut end = None;
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |j, e| {
        if let ClientEvent::Host(HostEvent::SortieFinished { debrief, .. }) = e
            && j == k
        {
            end = Some(debrief.record.end);
        }
        end.is_some()
    }));
    assert_eq!(end, Some(SortieEnd::HandedOver));
    assert!(w.sim.pilot_absent(me.0));
    assert!(w.host.state.sortie(me).is_none());
    let i = w.sim.flight_of_tag(tag).expect("the AI flies it again");
    assert!(w.sim.campaign_flight(i).is_some_and(|f| f.blue));
}

/// A player sits at another's AC-130 gunner's console: his pod and trigger fire its side guns;
/// its pilot leaves it, and it goes on with the gunner as its pilot (the AI flying for him).
#[test]
fn a_gunner_crews_another_players_gunship_and_takes_it_over() {
    use stealth_campaign::{MissionKind, SortieEnd};
    use stealth_sim::PodAim;
    use stealth_sim::crew::{CrewCmd, GunnerAim, Seat};
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut a = Client::connect(&addr, "Alpha").unwrap();
    let mut b = Client::connect(&addr, "Bravo").unwrap();
    let mut now = Instant::now();
    let mut ids = [None, None];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Welcome { id, .. } = e {
            ids[k] = Some(*id);
        }
        ids.iter().all(Option::is_some)
    }));
    let (pa, pb) = (ids[0].unwrap(), ids[1].unwrap());
    // Alpha flies a close air support mission in an AC-130 (the first one it may fly).
    let mut started = None;
    for _ in 0..15 * 600 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        for c in [&mut a, &mut b] {
            c.flush(now);
            c.poll(now);
        }
        let offers: Vec<u32> = w
            .host
            .state
            .board
            .iter()
            .filter(|o| o.kind == MissionKind::Cas)
            .map(|o| o.id)
            .collect();
        for offer in offers {
            if let Ok(ev) = w.host.take_mission(&mut w.sim, pa, offer, None, Aircraft::AC130, None)
            {
                w.server.route(&ev);
                started = Some(());
                break;
            }
        }
        if started.is_some() {
            break;
        }
    }
    started.expect("a close air support mission for the AC-130");
    // Up at 3 km, heading north, banked 30 degrees left.
    let mut ps = w.sim.pilot_input(pa.0);
    ps.altitude = (3000.0 / 0.488) as u16;
    ps.height = ps.altitude;
    ps.on_ground = false;
    ps.gear_up = true;
    ps.velocity = 180 * 27;
    ps.heading = 0;
    ps.pitch = 0;
    ps.roll = -5461;
    // Its engines near full (the gunner hears them).
    ps.engine = [87, 0, 0];
    let fly = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, n: usize| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    fly(&mut w, &mut a, &mut b, &mut now, 30);
    assert!(w.sim.pilot_in_the_air(pa.0));
    // Bravo sits at its gunner's console.
    b.send(&ToServer::Crew(CrewCmd::Join { owner: pa.0 }));
    fly(&mut w, &mut a, &mut b, &mut now, 15);
    assert_eq!(w.sim.crew_of(pb.0), Some((pa.0, Seat::Gunner)));
    assert_eq!(w.host.state.sortie(pa).unwrap().crew, vec![pb]);
    // Out of the world himself, Bravo is sent Alpha's aircraft in every frame and told it is
    // there (his client draws it between the frames, his eye riding in it), with its engines.
    let (mut frames, mut engine) = (0, None);
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let (1, ClientEvent::Fast { world, pilot, .. }) = (k, e) {
            assert!(world.present.contains(&pa.0), "{:?}", world.present);
            assert!(world.pilots.iter().any(|p| p.id == pa.0));
            frames += 1;
            if let Some(c) = pilot.as_ref().and_then(|p| p.crew) {
                engine = Some(c.engine);
            }
        }
        frames >= 4 && engine.is_some()
    }));
    assert_eq!(engine, Some(ps.engine));
    // His pod off the left wing, 50 degrees down, the trigger held: the 40 mm fires.
    let before = w.sim.as_pilot(pa.0, |s| s.side_guns().unwrap().rounds[0]);
    let aim = GunnerAim {
        pod: Some(PodAim { heading: 0xC000, pitch: -9102, ground: None }),
        trigger: true,
        gun: 0,
        station: None,
    };
    for _ in 0..30 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        a.send_input(&ps);
        b.send_unreliable(&ToServer::Crew(CrewCmd::Aim(aim)));
        for c in [&mut a, &mut b] {
            c.flush(now);
            c.poll(now);
        }
    }
    let after = w.sim.as_pilot(pa.0, |s| s.side_guns().unwrap().rounds[0]);
    assert!(after < before, "the gunner's trigger fired nothing ({before} rounds)");
    // Alpha leaves the aircraft: it goes on with Bravo, at its gunner's console.
    a.send(&ToServer::LeaveAircraft);
    let (mut handed, mut taken) = (None, None);
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        match e {
            ClientEvent::Host(HostEvent::SortieFinished { debrief, player }) if k == 0 => {
                assert_eq!(*player, pa);
                handed = Some(debrief.record.end);
            }
            ClientEvent::Host(HostEvent::SortieStarted { player, start, .. }) if k == 1 => {
                assert_eq!(*player, pb);
                taken = Some(start.clone());
            }
            _ => {}
        }
        handed.is_some() && taken.is_some()
    }));
    assert_eq!(handed, Some(SortieEnd::HandedOver));
    let start = taken.unwrap();
    assert!(!start.on_ground && start.altitude == ps.altitude, "where Alpha's aircraft was");
    assert!(w.sim.pilot_absent(pa.0) && w.sim.pilot_in_the_air(pb.0));
    assert_eq!(w.sim.owner_seat(pb.0), Seat::Gunner);
    assert_eq!(w.sim.as_pilot(pb.0, |s| s.side_guns().unwrap().rounds[0]), after);
    let s = w.host.state.sortie(pb).expect("Bravo's sortie");
    assert_eq!(s.crew, vec![pa]);
    assert!(w.host.state.sortie(pa).is_none());
    // The roster has Bravo flying it now.
    let roster = w.server.roster(&w.host, &w.sim);
    let bravo = roster.iter().find(|p| p.id == pb).unwrap();
    assert!(matches!(bravo.status, PlayerStatus::Flying { aircraft: Aircraft::AC130, .. }));
}

/// Bravo crews Alpha's AH-64 from its front seat over the network: his TADS lays the chin gun
/// and his trigger fires it, his release a Hellfire; Alpha leaves, the aircraft goes on with
/// Bravo in the gunner's seat (the AI flying).
#[test]
fn a_gunner_crews_another_players_apache_and_takes_it_over() {
    use stealth_campaign::MissionKind;
    use stealth_sim::PodAim;
    use stealth_sim::crew::{CrewCmd, GunnerAim, Seat};
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut a = Client::connect(&addr, "Alpha").unwrap();
    let mut b = Client::connect(&addr, "Bravo").unwrap();
    let mut now = Instant::now();
    let mut ids = [None, None];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Welcome { id, .. } = e {
            ids[k] = Some(*id);
        }
        ids.iter().all(Option::is_some)
    }));
    let (pa, pb) = (ids[0].unwrap(), ids[1].unwrap());
    // Alpha flies a close air support mission in an AH-64.
    let mut started = false;
    for _ in 0..15 * 600 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        for c in [&mut a, &mut b] {
            c.flush(now);
            c.poll(now);
        }
        let offers: Vec<u32> = w
            .host
            .state
            .board
            .iter()
            .filter(|o| o.kind == MissionKind::Cas)
            .map(|o| o.id)
            .collect();
        for offer in offers {
            if let Ok(ev) = w.host.take_mission(&mut w.sim, pa, offer, None, Aircraft::AH64, None) {
                w.server.route(&ev);
                started = true;
                break;
            }
        }
        if started {
            break;
        }
    }
    assert!(started, "a close air support mission for the AH-64");
    // Hellfires on the pylons, Stingers on the tips.
    w.sim.set_pilot_loadout(pa.0, &[(21, 2), (20, 4), (20, 4), (20, 4), (20, 4), (21, 2)], 0);
    // Hovering 300 m up, facing north.
    let mut ps = w.sim.pilot_input(pa.0);
    ps.altitude = (300.0 / 0.488) as u16;
    ps.height = ps.altitude;
    ps.on_ground = false;
    ps.gear_up = false;
    ps.velocity = 0;
    ps.heading = 0;
    ps.pitch = 0;
    ps.roll = 0;
    let fly = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, n: usize| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    fly(&mut w, &mut a, &mut b, &mut now, 30);
    assert!(w.sim.pilot_in_the_air(pa.0));
    assert!(w.sim.gunner_seat_free(pa.0));
    // Bravo takes the front seat.
    b.send(&ToServer::Crew(CrewCmd::Join { owner: pa.0 }));
    fly(&mut w, &mut a, &mut b, &mut now, 15);
    assert_eq!(w.sim.crew_of(pb.0), Some((pa.0, Seat::Gunner)));
    // His TADS 20 degrees down ahead, the trigger held: the chin gun fires.
    let ammo = w.sim.as_pilot(pa.0, |s| s.gun_ammo());
    let pod = Some(PodAim { heading: 0, pitch: -3641, ground: None });
    let aim = GunnerAim { pod, trigger: true, ..Default::default() };
    let send = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, aim, n| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            b.send_unreliable(&ToServer::Crew(CrewCmd::Aim(aim)));
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    send(&mut w, &mut a, &mut b, &mut now, aim, 30);
    assert!(w.sim.as_pilot(pa.0, |s| s.gun_ammo()) < ammo, "the gunner's trigger fired nothing");
    // His Hellfire from the second station.
    let aim = GunnerAim { trigger: false, station: Some(1), ..aim };
    send(&mut w, &mut a, &mut b, &mut now, aim, 5);
    b.send(&ToServer::Crew(CrewCmd::Fire { station: 1 }));
    send(&mut w, &mut a, &mut b, &mut now, aim, 15);
    let hellfires: i32 = w.sim.pilot_bays(pa.0).iter().filter(|s| s.0 == 20).map(|s| s.1).sum();
    assert_eq!(hellfires, 15, "the gunner's Hellfire is away");
    // Alpha leaves the aircraft: it goes on with Bravo in the front seat.
    a.send(&ToServer::LeaveAircraft);
    let mut taken = false;
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Host(HostEvent::SortieStarted { player, .. }) = e
            && k == 1
        {
            assert_eq!(*player, pb);
            taken = true;
        }
        taken
    }));
    assert!(w.sim.pilot_absent(pa.0) && w.sim.pilot_in_the_air(pb.0));
    assert_eq!(w.sim.owner_seat(pb.0), Seat::Gunner);
    let hellfires: i32 = w.sim.pilot_bays(pb.0).iter().filter(|s| s.0 == 20).map(|s| s.1).sum();
    assert_eq!(hellfires, 15, "his stores went with the aircraft");
}

/// Bravo crews Alpha's OH-58D from its left seat over the network: his mast sight lays his
/// Hellfire, his trigger leaves the gun pods to the pilot; Alpha leaves, the aircraft goes on
/// with Bravo in the gunner's seat (the AI flying).
#[test]
fn a_gunner_crews_another_players_kiowa_and_takes_it_over() {
    use stealth_campaign::MissionKind;
    use stealth_sim::PodAim;
    use stealth_sim::crew::{CrewCmd, GunnerAim, Seat};
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut a = Client::connect(&addr, "Alpha").unwrap();
    let mut b = Client::connect(&addr, "Bravo").unwrap();
    let mut now = Instant::now();
    let mut ids = [None, None];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Welcome { id, .. } = e {
            ids[k] = Some(*id);
        }
        ids.iter().all(Option::is_some)
    }));
    let (pa, pb) = (ids[0].unwrap(), ids[1].unwrap());
    // Alpha flies a close air support mission in an OH-58D.
    let mut started = false;
    for _ in 0..15 * 600 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        for c in [&mut a, &mut b] {
            c.flush(now);
            c.poll(now);
        }
        let offers: Vec<u32> = w
            .host
            .state
            .board
            .iter()
            .filter(|o| o.kind == MissionKind::Cas)
            .map(|o| o.id)
            .collect();
        for offer in offers {
            if let Ok(ev) = w.host.take_mission(&mut w.sim, pa, offer, None, Aircraft::OH58, None) {
                w.server.route(&ev);
                started = true;
                break;
            }
        }
        if started {
            break;
        }
    }
    assert!(started, "a close air support mission for the OH-58D");
    // Hellfires on the left pylon, a gun pod on the right.
    w.sim.set_pilot_loadout(pa.0, &[(20, 2), (22, 1)], 0);
    // Hovering 300 m up, facing north.
    let mut ps = w.sim.pilot_input(pa.0);
    ps.altitude = (300.0 / 0.488) as u16;
    ps.height = ps.altitude;
    ps.on_ground = false;
    ps.gear_up = false;
    ps.velocity = 0;
    ps.heading = 0;
    ps.pitch = 0;
    ps.roll = 0;
    let fly = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, n: usize| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    fly(&mut w, &mut a, &mut b, &mut now, 30);
    assert!(w.sim.pilot_in_the_air(pa.0));
    assert!(w.sim.gunner_seat_free(pa.0));
    // Bravo takes the left seat.
    b.send(&ToServer::Crew(CrewCmd::Join { owner: pa.0 }));
    fly(&mut w, &mut a, &mut b, &mut now, 15);
    assert_eq!(w.sim.crew_of(pb.0), Some((pa.0, Seat::Gunner)));
    // His mast sight 20 degrees down ahead, the trigger held: the gun pods are the pilot's.
    let ammo = w.sim.as_pilot(pa.0, |s| s.gun_ammo());
    let pod = Some(PodAim { heading: 0, pitch: -3641, ground: None });
    let aim = GunnerAim { pod, trigger: true, ..Default::default() };
    let send = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, aim, n| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            b.send_unreliable(&ToServer::Crew(CrewCmd::Aim(aim)));
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    send(&mut w, &mut a, &mut b, &mut now, aim, 30);
    assert_eq!(w.sim.as_pilot(pa.0, |s| s.gun_ammo()), ammo, "the gunner fired the gun pods");
    // His Hellfire from the first station.
    let aim = GunnerAim { trigger: false, station: Some(0), ..aim };
    send(&mut w, &mut a, &mut b, &mut now, aim, 5);
    b.send(&ToServer::Crew(CrewCmd::Fire { station: 0 }));
    send(&mut w, &mut a, &mut b, &mut now, aim, 15);
    assert_eq!(w.sim.pilot_bays(pa.0)[0], (20, 1), "the gunner's Hellfire is away");
    // Alpha leaves the aircraft: it goes on with Bravo in the left seat.
    a.send(&ToServer::LeaveAircraft);
    let mut taken = false;
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Host(HostEvent::SortieStarted { player, .. }) = e
            && k == 1
        {
            assert_eq!(*player, pb);
            taken = true;
        }
        taken
    }));
    assert!(w.sim.pilot_absent(pa.0) && w.sim.pilot_in_the_air(pb.0));
    assert_eq!(w.sim.owner_seat(pb.0), Seat::Gunner);
    assert_eq!(w.sim.pilot_bays(pb.0), &[(20, 1), (22, 1)][..], "his stores went with the aircraft");
}

/// Bravo crews Alpha's RAH-66 from its rear seat over the network: his trigger fires the chin
/// gun along his sight, his sight lays his Hellfire out of the bay; Alpha leaves, the aircraft
/// goes on with Bravo in the gunner's seat (the AI flying).
#[test]
fn a_gunner_crews_another_players_comanche_and_takes_it_over() {
    use stealth_campaign::MissionKind;
    use stealth_sim::PodAim;
    use stealth_sim::crew::{CrewCmd, GunnerAim, Seat};
    let mut w = world();
    let port = w.server.port().unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut a = Client::connect(&addr, "Alpha").unwrap();
    let mut b = Client::connect(&addr, "Bravo").unwrap();
    let mut now = Instant::now();
    let mut ids = [None, None];
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Welcome { id, .. } = e {
            ids[k] = Some(*id);
        }
        ids.iter().all(Option::is_some)
    }));
    let (pa, pb) = (ids[0].unwrap(), ids[1].unwrap());
    // Alpha flies a close air support mission in an RAH-66.
    let mut started = false;
    for _ in 0..15 * 600 {
        now += Duration::from_millis(66);
        serve(&mut w, now);
        for c in [&mut a, &mut b] {
            c.flush(now);
            c.poll(now);
        }
        let offers: Vec<u32> = w
            .host
            .state
            .board
            .iter()
            .filter(|o| o.kind == MissionKind::Cas)
            .map(|o| o.id)
            .collect();
        for offer in offers {
            if let Ok(ev) = w.host.take_mission(&mut w.sim, pa, offer, None, Aircraft::RAH66, None)
            {
                w.server.route(&ev);
                started = true;
                break;
            }
        }
        if started {
            break;
        }
    }
    assert!(started, "a close air support mission for the RAH-66");
    // Hellfires on the left bay door, Stingers on the right.
    w.sim.set_pilot_loadout(pa.0, &[(20, 3), (21, 6)], 0);
    // Hovering 300 m up, facing north.
    let mut ps = w.sim.pilot_input(pa.0);
    ps.altitude = (300.0 / 0.488) as u16;
    ps.height = ps.altitude;
    ps.on_ground = false;
    ps.gear_up = false;
    ps.velocity = 0;
    ps.heading = 0;
    ps.pitch = 0;
    ps.roll = 0;
    let fly = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, n: usize| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    fly(&mut w, &mut a, &mut b, &mut now, 30);
    assert!(w.sim.pilot_in_the_air(pa.0));
    assert!(w.sim.gunner_seat_free(pa.0));
    // Bravo takes the rear seat.
    b.send(&ToServer::Crew(CrewCmd::Join { owner: pa.0 }));
    fly(&mut w, &mut a, &mut b, &mut now, 15);
    assert_eq!(w.sim.crew_of(pb.0), Some((pa.0, Seat::Gunner)));
    // His sight 20 degrees down ahead, the trigger held: the chin gun fires.
    let ammo = w.sim.as_pilot(pa.0, |s| s.gun_ammo());
    let pod = Some(PodAim { heading: 0, pitch: -3641, ground: None });
    let aim = GunnerAim { pod, trigger: true, ..Default::default() };
    let send = |w: &mut World, a: &mut Client, b: &mut Client, now: &mut Instant, aim, n| {
        for _ in 0..n {
            *now += Duration::from_millis(66);
            serve(w, *now);
            a.send_input(&ps);
            b.send_unreliable(&ToServer::Crew(CrewCmd::Aim(aim)));
            for c in [&mut *a, &mut *b] {
                c.flush(*now);
                c.poll(*now);
            }
        }
    };
    send(&mut w, &mut a, &mut b, &mut now, aim, 30);
    assert!(w.sim.as_pilot(pa.0, |s| s.gun_ammo()) < ammo, "the gunner fired the chin gun");
    // His Hellfire from the first station.
    let aim = GunnerAim { trigger: false, station: Some(0), ..aim };
    send(&mut w, &mut a, &mut b, &mut now, aim, 5);
    // The release held until the bay doors are open.
    for _ in 0..12 {
        if w.sim.pilot_bays(pa.0)[0].1 == 3 {
            b.send(&ToServer::Crew(CrewCmd::Fire { station: 0 }));
        }
        send(&mut w, &mut a, &mut b, &mut now, aim, 5);
    }
    assert_eq!(w.sim.pilot_bays(pa.0)[0], (20, 2), "the gunner's Hellfire is away");
    // Alpha leaves the aircraft: it goes on with Bravo in the rear seat.
    a.send(&ToServer::LeaveAircraft);
    let mut taken = false;
    assert!(until(&mut w, &mut [&mut a, &mut b], &mut now, |k, e| {
        if let ClientEvent::Host(HostEvent::SortieStarted { player, .. }) = e
            && k == 1
        {
            assert_eq!(*player, pb);
            taken = true;
        }
        taken
    }));
    assert!(w.sim.pilot_absent(pa.0) && w.sim.pilot_in_the_air(pb.0));
    assert_eq!(w.sim.owner_seat(pb.0), Seat::Gunner);
    assert_eq!(
        w.sim.pilot_bays(pb.0),
        &[(20, 2), (21, 6)][..],
        "his stores went with the aircraft"
    );
}
